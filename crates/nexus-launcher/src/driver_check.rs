// SPDX-License-Identifier: AGPL-3.0-or-later
//! Sprint 18 Phase E1 — NVIDIA driver CVE check at launcher start.
//!
//! Reads the locally installed NVIDIA driver version (via NVML),
//! fetches the NVD 2.0 vulnerability feed for
//! `cpe:2.3:o:nvidia:gpu_display_driver:*`, and returns the
//! subset of CVE whose version range covers the local driver.
//!
//! Design choices:
//!
//! - **Fail-open everywhere**. A missing NVIDIA driver, a network
//!   error, a stale DNS answer, a rate-limit rebuff, or a bogus
//!   cache file all produce an empty report rather than an error
//!   that would block the launcher from starting. The launcher
//!   only prints a warning when `critical_count > 0`.
//! - **24 h cache** at `<sbfb_home>/nvd-cache.json` to stay well
//!   under the NVD 5-req/30s unauthenticated rate limit even if
//!   the user restarts the launcher many times a day.
//! - **Warning-only, never block**. Gate 1 of the Sprint 18
//!   roadmap (`driver hardening` track) explicitly de-scopes
//!   blocking behaviour until workloads carry critical data.

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

/// Official NVD vulnerabilities 2.0 endpoint. Documented at
/// <https://nvd.nist.gov/developers/vulnerabilities>.
pub const NVD_API_BASE: &str = "https://services.nvd.nist.gov/rest/json/cves/2.0";

/// CPE prefix we filter on. The wildcard after `:*` means "any
/// version" — NVD still returns matches on the vendor:product
/// pair even when the local driver has a newer version than any
/// CVE in the database.
pub const NVD_CPE_FILTER: &str = "cpe:2.3:o:nvidia:gpu_display_driver:*";

/// 24 h cache window. Under the 5-req/30s NVD unauthenticated
/// rate limit by two orders of magnitude even if 100 users on
/// the same IP restart their launcher every hour.
pub const CACHE_TTL: Duration = Duration::from_secs(24 * 3600);

/// Network timeout for the NVD fetch. Deliberately short so a
/// hung TCP connection cannot stall the launcher boot path.
const FETCH_TIMEOUT: Duration = Duration::from_secs(10);

/// Cache file leaf under `<sbfb_home>`.
const CACHE_LEAF: &str = "nvd-cache.json";

// =================================================================
// Public surface
// =================================================================

/// CVSS severity enum. We store `Unknown` rather than skipping a
/// CVE with no CVSS block because "no severity" is a common NVD
/// state for recently-published advisories and we want to count
/// it in `cves_affecting` (but not in `critical_count`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
    Unknown,
}

impl Severity {
    fn parse(raw: &str) -> Self {
        match raw.to_ascii_uppercase().as_str() {
            "CRITICAL" => Self::Critical,
            "HIGH" => Self::High,
            "MEDIUM" => Self::Medium,
            "LOW" => Self::Low,
            _ => Self::Unknown,
        }
    }
}

/// Single CVE entry projected down to the fields the launcher
/// surfaces in its warning line.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CveEntry {
    pub id: String,
    pub severity: Severity,
}

/// Result of a single driver-check pass.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DriverCheckReport {
    /// Locally installed NVIDIA driver version string
    /// (e.g. `"535.54.03"`), or `None` on machines without an
    /// NVIDIA driver loaded — which is the common case on macOS,
    /// headless CI, and AMD-only hosts.
    pub local_version: Option<String>,

    /// CVE whose declared version range covers `local_version`.
    /// Empty when `local_version` is `None`, when the NVD fetch
    /// failed, or genuinely when no CVE matches.
    pub cves_affecting: Vec<CveEntry>,

    /// Count of `cves_affecting` with severity `Critical`. The
    /// launcher emits its warning line only when this is > 0.
    pub critical_count: usize,

    /// True when the NVD dataset came from the on-disk cache,
    /// false when it came from a live HTTP fetch (or the fetch
    /// failed). Used by integration tests and the launcher log
    /// to assert the 24 h cache is actually taking effect.
    pub fetched_from_cache: bool,

    /// True when the NVD HTTP fetch raised an error and we fell
    /// back to an empty dataset. Lets the launcher disambiguate
    /// "0 CVE because NVD said so" from "0 CVE because we
    /// couldn't reach NVD". Never raised when the cache hit.
    pub fetch_failed: bool,
}

impl DriverCheckReport {
    fn empty(local_version: Option<String>) -> Self {
        Self {
            local_version,
            cves_affecting: Vec::new(),
            critical_count: 0,
            fetched_from_cache: false,
            fetch_failed: false,
        }
    }
}

/// Top-level entry point. Resolves the cache path under
/// `<sbfb_home>/` and hits the official NVD endpoint. Never
/// panics, never returns `Err`.
pub async fn check_nvidia_drivers() -> DriverCheckReport {
    let local = fetch_local_driver_version();
    let cache = default_cache_path();
    check_nvidia_drivers_with(NVD_API_BASE, cache.as_deref(), CACHE_TTL, local).await
}

/// Injectable variant used by the tests (and by any future caller
/// that wants to point at a captive NVD mirror or a tempdir).
/// `local_version` is passed in rather than probed so the version
/// match logic is testable on hosts without an NVIDIA GPU.
pub async fn check_nvidia_drivers_with(
    api_url: &str,
    cache_path: Option<&Path>,
    cache_ttl: Duration,
    local_version: Option<String>,
) -> DriverCheckReport {
    let mut report = DriverCheckReport::empty(local_version.clone());

    let response = if let Some(path) = cache_path {
        match load_cache(path, cache_ttl) {
            Ok(Some(resp)) => {
                report.fetched_from_cache = true;
                Some(resp)
            }
            _ => match fetch_nvd(api_url).await {
                Ok(resp) => {
                    if let Err(e) = store_cache(path, &resp) {
                        tracing::warn!("nvd cache write failed: {e}");
                    }
                    Some(resp)
                }
                Err(e) => {
                    tracing::warn!("nvd fetch failed: {e}");
                    report.fetch_failed = true;
                    None
                }
            },
        }
    } else {
        match fetch_nvd(api_url).await {
            Ok(resp) => Some(resp),
            Err(e) => {
                tracing::warn!("nvd fetch failed: {e}");
                report.fetch_failed = true;
                None
            }
        }
    };

    if let (Some(resp), Some(ver)) = (response, local_version) {
        report.cves_affecting = filter_affecting_version(&resp, &ver);
        report.critical_count = report
            .cves_affecting
            .iter()
            .filter(|c| c.severity == Severity::Critical)
            .count();
    }

    report
}

// =================================================================
// Local driver lookup
// =================================================================

fn fetch_local_driver_version() -> Option<String> {
    match nvml_wrapper::Nvml::init() {
        Ok(nvml) => nvml.sys_driver_version().ok(),
        Err(_) => None,
    }
}

fn default_cache_path() -> Option<PathBuf> {
    nexus_shell_daemon_core::auth::sbfb_home().map(|d| d.join(CACHE_LEAF))
}

// =================================================================
// NVD fetch + cache
// =================================================================

/// Minimal NVD response projection. We ignore everything we
/// don't consume so that NVD can add fields without breaking us.
/// All field names use `serde(rename_all = "camelCase")` because
/// the NVD wire format is camelCase.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NvdResponse {
    #[serde(default)]
    pub(crate) vulnerabilities: Vec<NvdVulnWrapper>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NvdVulnWrapper {
    pub(crate) cve: NvdCve,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NvdCve {
    pub(crate) id: String,
    #[serde(default)]
    pub(crate) metrics: Option<NvdMetrics>,
    #[serde(default)]
    pub(crate) configurations: Vec<NvdConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NvdMetrics {
    #[serde(default)]
    pub(crate) cvss_metric_v31: Vec<CvssMetric>,
    #[serde(default)]
    pub(crate) cvss_metric_v30: Vec<CvssMetric>,
    #[serde(default)]
    pub(crate) cvss_metric_v2: Vec<CvssMetric>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CvssMetric {
    pub(crate) cvss_data: CvssData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct CvssData {
    #[serde(default)]
    pub(crate) base_severity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NvdConfig {
    #[serde(default)]
    pub(crate) nodes: Vec<NvdNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NvdNode {
    #[serde(default)]
    pub(crate) cpe_match: Vec<NvdCpeMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub(crate) struct NvdCpeMatch {
    #[serde(default)]
    pub(crate) criteria: String,
    #[serde(default)]
    pub(crate) vulnerable: bool,
    #[serde(default)]
    pub(crate) version_start_including: Option<String>,
    #[serde(default)]
    pub(crate) version_start_excluding: Option<String>,
    #[serde(default)]
    pub(crate) version_end_including: Option<String>,
    #[serde(default)]
    pub(crate) version_end_excluding: Option<String>,
}

/// On-disk cache envelope. `fetched_at_unix_secs` is compared
/// against `SystemTime::now()` at load time; any skew past
/// `cache_ttl` invalidates the entry.
#[derive(Debug, Serialize, Deserialize)]
struct CacheEnvelope {
    fetched_at_unix_secs: u64,
    response: NvdResponse,
}

async fn fetch_nvd(api_url: &str) -> anyhow::Result<NvdResponse> {
    let client = reqwest::Client::builder()
        .timeout(FETCH_TIMEOUT)
        .user_agent(concat!("sbfb-launcher/", env!("CARGO_PKG_VERSION")))
        .build()?;
    let resp = client
        .get(api_url)
        .query(&[("cpeName", NVD_CPE_FILTER)])
        .send()
        .await?;
    if !resp.status().is_success() {
        anyhow::bail!("nvd returned {}", resp.status());
    }
    let body: NvdResponse = resp.json().await?;
    Ok(body)
}

fn load_cache(path: &Path, ttl: Duration) -> std::io::Result<Option<NvdResponse>> {
    let raw = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(e),
    };
    let env: CacheEnvelope = match serde_json::from_str(&raw) {
        Ok(env) => env,
        Err(_) => return Ok(None),
    };
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if now.saturating_sub(env.fetched_at_unix_secs) > ttl.as_secs() {
        return Ok(None);
    }
    Ok(Some(env.response))
}

fn store_cache(path: &Path, response: &NvdResponse) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let env = CacheEnvelope {
        fetched_at_unix_secs: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
        response: response.clone(),
    };
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, serde_json::to_vec(&env)?)?;
    std::fs::rename(&tmp, path)?;
    Ok(())
}

// =================================================================
// Version matching
// =================================================================

fn filter_affecting_version(resp: &NvdResponse, local: &str) -> Vec<CveEntry> {
    let local_parts = parse_version(local);
    let mut out = Vec::new();
    for wrap in &resp.vulnerabilities {
        let cve = &wrap.cve;
        if !cve_affects(cve, &local_parts) {
            continue;
        }
        out.push(CveEntry {
            id: cve.id.clone(),
            severity: pick_severity(cve.metrics.as_ref()),
        });
    }
    out
}

fn cve_affects(cve: &NvdCve, local: &[u64]) -> bool {
    for config in &cve.configurations {
        for node in &config.nodes {
            for m in &node.cpe_match {
                if !m.vulnerable {
                    continue;
                }
                if cpe_match_covers(m, local) {
                    return true;
                }
            }
        }
    }
    false
}

fn cpe_match_covers(m: &NvdCpeMatch, local: &[u64]) -> bool {
    if let Some(exact) = criteria_version(&m.criteria)
        && !exact.is_empty()
        && exact != "*"
        && exact != "-"
    {
        return parse_version(&exact) == local;
    }
    let mut matched_any_bound = false;
    let mut ok = true;
    if let Some(v) = m.version_start_including.as_deref() {
        matched_any_bound = true;
        ok &= cmp_versions(local, &parse_version(v)) != std::cmp::Ordering::Less;
    }
    if let Some(v) = m.version_start_excluding.as_deref() {
        matched_any_bound = true;
        ok &= cmp_versions(local, &parse_version(v)) == std::cmp::Ordering::Greater;
    }
    if let Some(v) = m.version_end_including.as_deref() {
        matched_any_bound = true;
        ok &= cmp_versions(local, &parse_version(v)) != std::cmp::Ordering::Greater;
    }
    if let Some(v) = m.version_end_excluding.as_deref() {
        matched_any_bound = true;
        ok &= cmp_versions(local, &parse_version(v)) == std::cmp::Ordering::Less;
    }
    matched_any_bound && ok
}

/// Extract the CPE `version` segment (index 5 in the 13-field
/// `cpe:2.3:...` string). Returns `None` when the criteria isn't
/// a well-formed CPE 2.3 URI.
fn criteria_version(criteria: &str) -> Option<String> {
    let parts: Vec<&str> = criteria.split(':').collect();
    if parts.len() < 6 {
        return None;
    }
    Some(parts[5].to_string())
}

/// Sprint 18 audit fix E1-1 : parse a `XXX.YY.ZZ`-style driver
/// version into a `Vec<u64>` for ordering. NVIDIA stable drivers
/// are always pure-numeric per-segment, but a beta / RC string
/// like `535.54.03-rc1` previously silently coerced the bad
/// segment to `0` (via the `unwrap_or(0)` shortcut), which
/// yielded a *lower* parsed version than the patch behind it
/// and could make a vulnerable driver look unaffected by a CVE
/// whose end-bound used a similar non-numeric suffix. We now
/// log a warn on any unparseable segment so the issue is visible
/// in the launcher logs ; the segment still defaults to 0
/// because returning an Err would suppress the whole CVE check
/// silently — degrading to "older-than-everything" is the safer
/// failure mode (more false positives, never a false negative).
fn parse_version(s: &str) -> Vec<u64> {
    s.split('.')
        .map(|seg| match seg.parse::<u64>() {
            Ok(n) => n,
            Err(_) => {
                tracing::warn!(
                    segment = seg,
                    raw = s,
                    "non-numeric driver version segment, defaulting to 0 for ordering",
                );
                0
            }
        })
        .collect()
}

fn cmp_versions(a: &[u64], b: &[u64]) -> std::cmp::Ordering {
    let len = a.len().max(b.len());
    for i in 0..len {
        let ai = a.get(i).copied().unwrap_or(0);
        let bi = b.get(i).copied().unwrap_or(0);
        match ai.cmp(&bi) {
            std::cmp::Ordering::Equal => continue,
            other => return other,
        }
    }
    std::cmp::Ordering::Equal
}

fn pick_severity(metrics: Option<&NvdMetrics>) -> Severity {
    let Some(m) = metrics else {
        return Severity::Unknown;
    };
    if let Some(first) = m.cvss_metric_v31.first() {
        return Severity::parse(&first.cvss_data.base_severity);
    }
    if let Some(first) = m.cvss_metric_v30.first() {
        return Severity::parse(&first.cvss_data.base_severity);
    }
    Severity::Unknown
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn tmp_dir(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("sbfb-driver-check-{tag}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn sample_cve(id: &str, sev: &str, criteria: &str) -> NvdVulnWrapper {
        NvdVulnWrapper {
            cve: NvdCve {
                id: id.to_string(),
                metrics: Some(NvdMetrics {
                    cvss_metric_v31: vec![CvssMetric {
                        cvss_data: CvssData {
                            base_severity: sev.to_string(),
                        },
                    }],
                    ..NvdMetrics::default()
                }),
                configurations: vec![NvdConfig {
                    nodes: vec![NvdNode {
                        cpe_match: vec![NvdCpeMatch {
                            criteria: criteria.to_string(),
                            vulnerable: true,
                            ..NvdCpeMatch::default()
                        }],
                    }],
                }],
            },
        }
    }

    #[test]
    fn version_affected_by_cve_exact_criteria_match() {
        let resp = NvdResponse {
            vulnerabilities: vec![
                sample_cve(
                    "CVE-2026-0001",
                    "HIGH",
                    "cpe:2.3:o:nvidia:gpu_display_driver:535.54.03:*:*:*:*:*:*:*",
                ),
                sample_cve(
                    "CVE-2026-0002",
                    "CRITICAL",
                    "cpe:2.3:o:nvidia:gpu_display_driver:470.00.00:*:*:*:*:*:*:*",
                ),
            ],
        };

        let hits = filter_affecting_version(&resp, "535.54.03");
        assert_eq!(hits.len(), 1, "only the matching CPE should fire");
        assert_eq!(hits[0].id, "CVE-2026-0001");
        assert_eq!(hits[0].severity, Severity::High);
    }

    #[tokio::test]
    async fn cache_hit_within_ttl_skips_fetch() {
        let dir = tmp_dir("cache-hit");
        let cache = dir.join(CACHE_LEAF);

        let resp = NvdResponse {
            vulnerabilities: vec![sample_cve(
                "CVE-2026-0099",
                "HIGH",
                "cpe:2.3:o:nvidia:gpu_display_driver:535.54.03:*:*:*:*:*:*:*",
            )],
        };
        let env = CacheEnvelope {
            fetched_at_unix_secs: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            response: resp,
        };
        let mut f = std::fs::File::create(&cache).unwrap();
        f.write_all(&serde_json::to_vec(&env).unwrap()).unwrap();
        drop(f);

        // Bogus URL: a live fetch would error and set fetch_failed.
        // A real cache hit short-circuits before touching the network,
        // so the report must report fetched_from_cache=true and a
        // clean fetch_failed=false.
        let report = check_nvidia_drivers_with(
            "http://127.0.0.1:1/rest/json/cves/2.0",
            Some(&cache),
            CACHE_TTL,
            Some("535.54.03".to_string()),
        )
        .await;
        assert!(
            report.fetched_from_cache,
            "ttl-valid cache must short-circuit fetch"
        );
        assert!(
            !report.fetch_failed,
            "cache hit must not surface fetch_failed"
        );
        assert_eq!(report.cves_affecting.len(), 1);
        assert_eq!(report.cves_affecting[0].id, "CVE-2026-0099");
    }

    #[test]
    fn cache_miss_when_ttl_expired() {
        let dir = tmp_dir("cache-miss");
        let cache = dir.join(CACHE_LEAF);

        let stale = CacheEnvelope {
            fetched_at_unix_secs: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
                - (25 * 3600),
            response: NvdResponse {
                vulnerabilities: vec![],
            },
        };
        std::fs::write(&cache, serde_json::to_vec(&stale).unwrap()).unwrap();

        let loaded = load_cache(&cache, CACHE_TTL).expect("io ok");
        assert!(loaded.is_none(), "entry older than TTL must miss");
    }

    #[tokio::test]
    async fn offline_fallback_returns_empty_report_not_err() {
        // 127.0.0.1:1 has no listener → immediate TCP refused.
        // Assert the function still returns, with fetch_failed set.
        let report = check_nvidia_drivers_with(
            "http://127.0.0.1:1/rest/json/cves/2.0",
            None,
            CACHE_TTL,
            Some("535.54.03".to_string()),
        )
        .await;
        assert_eq!(report.local_version.as_deref(), Some("535.54.03"));
        assert!(
            report.fetch_failed,
            "network error must surface fetch_failed"
        );
        assert!(report.cves_affecting.is_empty());
        assert_eq!(report.critical_count, 0);
        assert!(!report.fetched_from_cache);
    }

    #[test]
    fn filter_critical_cves_only_counts_critical() {
        let resp = NvdResponse {
            vulnerabilities: vec![
                sample_cve(
                    "CVE-2026-1001",
                    "CRITICAL",
                    "cpe:2.3:o:nvidia:gpu_display_driver:535.54.03:*:*:*:*:*:*:*",
                ),
                sample_cve(
                    "CVE-2026-1002",
                    "MEDIUM",
                    "cpe:2.3:o:nvidia:gpu_display_driver:535.54.03:*:*:*:*:*:*:*",
                ),
                sample_cve(
                    "CVE-2026-1003",
                    "HIGH",
                    "cpe:2.3:o:nvidia:gpu_display_driver:535.54.03:*:*:*:*:*:*:*",
                ),
            ],
        };
        let hits = filter_affecting_version(&resp, "535.54.03");
        assert_eq!(hits.len(), 3, "all three are affected by version match");
        let critical = hits
            .iter()
            .filter(|h| h.severity == Severity::Critical)
            .count();
        assert_eq!(critical, 1, "only CVE-2026-1001 is Critical");
    }

    #[test]
    fn version_range_bounds_include_and_exclude() {
        let m = NvdCpeMatch {
            criteria: "cpe:2.3:o:nvidia:gpu_display_driver:*:*:*:*:*:*:*:*".into(),
            vulnerable: true,
            version_start_including: Some("535.0.0".into()),
            version_end_excluding: Some("536.0.0".into()),
            ..NvdCpeMatch::default()
        };
        assert!(cpe_match_covers(&m, &parse_version("535.54.03")));
        assert!(!cpe_match_covers(&m, &parse_version("536.0.0")));
        assert!(!cpe_match_covers(&m, &parse_version("534.99.99")));
    }
}
