// SPDX-License-Identifier: AGPL-3.0-or-later
//! TLS certificate pinning for SBFB relay clients (Sprint 19 Phase C).
//!
//! This module closes the WebPKI trust path for the iroh relay
//! fallback transport — the HTTPS WebSocket channel used when
//! QUIC hole-punching fails (symmetric NAT, CGNAT, corporate
//! firewall). QUIC direct connections between peers are
//! authenticated end-to-end via Ed25519 raw public keys (RFC 7250)
//! and do not touch WebPKI at all; the relay fallback, in
//! contrast, relies on the OS root CA store and is therefore
//! exposed to T2 (state-mandated MITM), T3 (CA compromise), T4
//! (hostile relay operator), T5 (BGP hijack + fraudulent
//! issuance). See `.planning/research/S19_phase_C_tls_cert_pinning_
//! design.md` §1 for the full threat model.
//!
//! ## What this module delivers
//!
//! - [`extract_spki_sha256`] / [`extract_spki_sha256_from_pem`] —
//!   compute the RFC 7469 §2.4 SPKI SHA-256 base64url-no-padding
//!   fingerprint of a DER- or PEM-encoded X.509 certificate.
//! - [`RelayPin`] / [`RelayPinsFile`] — the on-disk JSON schema
//!   for `~/.sbfb/relay-pins.json`, mirroring the S18
//!   `relays.json` / `tokens.json` convention.
//! - [`PinValidator`] — loads the pin file at construction, hot-
//!   reloads it on any `notify` event (pattern from
//!   `ConsentWatcher` S16 Phase C and `TokenRotator` S18 Phase D),
//!   and exposes [`PinValidator::validate`] for callers that have
//!   a DER cert in hand.
//!
//! ## What this module does NOT deliver (Sprint 19 scope)
//!
//! No hook into the iroh 0.98 relay client is wired in this
//! sprint. context7 verification of `/websites/rs_iroh` 2026-04-16
//! confirmed that `relay::client::ClientBuilder` exposes
//! `insecure_skip_cert_verify` only under `#[cfg(any(test,
//! feature = "test-utils"))]`; no public hook for a custom
//! `rustls::client::danger::ServerCertVerifier` exists. Sprint 19
//! deliberately delivers the primitive + hot-reload + tests and
//! marks the forked-connect-path wiring as tech debt tracked in
//! `docs/rust/PATTERNS.md` §T20, pending upstream iroh
//! contribution. This mirrors the Phase A (DHT quorum primitive
//! then runtime wire) and Phase B (PoW primitive then gossip
//! subscribe) pattern the sprint established for transport-layer
//! hardening.
//!
//! ## Fail-open vs fail-closed policy
//!
//! | State | Behaviour |
//! |---|---|
//! | Pinset absent / empty | Fail-open + `tracing::warn!` — the user has not opted in yet; WebPKI remains the only check |
//! | Pinset present + relay URL pinned + SPKI match | Accept |
//! | Pinset present + relay URL pinned + SPKI mismatch | Reject, return [`PinError::SpkiMismatch`] |
//! | Pinset present + relay URL **not** pinned (other relays are) | Reject, return [`PinError::NoPin`] — opted-in users should not silently accept new relays |
//! | Pin has `expires_at` in the past | Skip that specific pin; if every pin for the URL is expired → [`PinError::SpkiMismatch`] with `any_expired = true` |
//!
//! See the design doc §5.4 for the rationale on fail-open
//! default (pre-launch convivialité) and the opt-in-then-strict
//! posture.

use std::collections::HashSet;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{mpsc, Arc, RwLock};
use std::thread;
use std::time::Duration;

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use chrono::{DateTime, Utc};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;
use tracing::{debug, warn};
use x509_parser::prelude::*;

// =================================================================
// Constants
// =================================================================

/// Basename of the pin file under the SBFB home directory.
pub const RELAY_PINS_FILE_NAME: &str = "relay-pins.json";

/// Env var that, when set to a non-empty value, overrides the
/// default path resolution. Used by tests and by operators who
/// want the daemon to read pins from a non-default location
/// (multi-tenant dev machine, sandbox).
pub const CUSTOM_PINS_FILE_ENV: &str = "SBFB_RELAY_PINS_FILE";

/// Env var pointing at `~/.sbfb/`. Same variable the worker and
/// daemon already honour for [`consent.json`](crate::) and
/// [`tokens.json`](crate::). Duplicated here rather than
/// cross-crate importing so `nexus-core-rs` remains a pure
/// primitive library with no `nexus-shell-daemon-core` dep.
pub const SBFB_HOME_ENV: &str = "SBFB_HOME";

/// Current format version of `relay-pins.json`. Per the pre-launch
/// protocol policy (CLAUDE.md §"Pre-launch protocol policy"), the
/// version is frozen at 1 until the `v1.0` tag; any wire-format
/// refinement redefines v1 in place rather than bumping it.
pub const PIN_FILE_FORMAT_VERSION: u8 = 1;

/// Debounce window applied between a `notify` event firing and
/// the actual reload. 50 ms is the same value `ConsentWatcher`
/// (Sprint 16 Phase C) uses for the same reason: editors that
/// perform write+rename atomic saves emit Create+Modify in quick
/// succession.
const RELOAD_DEBOUNCE: Duration = Duration::from_millis(50);

// =================================================================
// Errors
// =================================================================

/// Errors surfaced by [`PinValidator`] and the free SPKI extract
/// helpers. `thiserror` for `Display` impls; the struct shape
/// deliberately carries contextual fields (`relay_url`, `actual`,
/// `pin_count`) so operators can diagnose a mismatch without
/// re-running the pipeline by hand.
#[derive(Debug, Error)]
pub enum PinError {
    /// I/O while reading `relay-pins.json` or its parent dir.
    #[error("relay-pins.json io error: {0}")]
    Io(#[from] io::Error),

    /// `serde_json::from_slice` rejected the pin file payload.
    #[error("relay-pins.json parse error: {0}")]
    Json(#[from] serde_json::Error),

    /// `notify` watcher failed to install or reported an error on
    /// its event channel.
    #[error("notify watcher error: {0}")]
    Notify(#[from] notify::Error),

    /// `relay-pins.json` carries a `version` field this build does
    /// not understand. Pre-launch we expect only v1.
    #[error("unsupported relay-pins.json version: {0} (expected {PIN_FILE_FORMAT_VERSION})")]
    UnknownVersion(u8),

    /// Certificate bytes could not be parsed as X.509 DER or PEM.
    #[error("cert parse error: {0}")]
    ParseCert(String),

    /// No pin is listed for this relay URL but the pinset contains
    /// entries for other relays — treat as fail-closed per the
    /// opt-in-then-strict posture.
    #[error("no pin configured for relay {0}")]
    NoPin(String),

    /// The cert presented by the relay does not match any of the
    /// non-expired pins recorded for its URL.
    #[error(
        "SPKI pin mismatch for {relay_url}: cert presents {actual}, \
         {pin_count} pin(s) on file (any expired: {any_expired})"
    )]
    SpkiMismatch {
        /// Which relay URL the mismatch was observed on.
        relay_url: String,
        /// Base64url no-padding SPKI SHA-256 of the cert the relay
        /// actually presented.
        actual: String,
        /// Number of pin entries on file for this URL. Helps
        /// operators distinguish "you forgot to pin" (0, but then
        /// we would have returned [`Self::NoPin`]) from "you
        /// pinned an old key" (≥1).
        pin_count: usize,
        /// True if at least one pin for this URL had its
        /// `expires_at` in the past. Hints that a planned
        /// rotation has elapsed without the user refreshing.
        any_expired: bool,
    },

    /// `added_at` / `expires_at` did not parse as RFC 3339.
    #[error("invalid timestamp in relay-pins.json: {0}")]
    InvalidTimestamp(String),

    /// The internal RwLock was poisoned by a panicking writer.
    /// Only reachable if the `notify` background thread panicked
    /// while holding the write lock — which itself would be a
    /// bug. Mapped to an error so callers can log and fail-closed
    /// rather than re-panic.
    #[error("pin validator state lock poisoned")]
    Poisoned,
}

// =================================================================
// On-disk schema
// =================================================================

/// Provenance of a pin entry. Has no validation effect — the
/// validator treats Bootstrap and UserOverride identically — but
/// operators and auditors want to know at a glance whether a
/// given pin came from the shipped release or a manual edit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PinSource {
    /// Pin shipped inside the SBFB binary release (the default
    /// set of n0 + federation relays, computed offline at sprint
    /// kickoff via the pipeline documented in
    /// `docs/release/RELAY_PIN_BOOTSTRAP.md`).
    Bootstrap,
    /// Pin added by the local user editing `relay-pins.json`
    /// directly (e.g., after a key roll where the operator
    /// published a new SPKI out-of-band).
    UserOverride,
}

/// One pin entry. Multiple entries per `relay_url` are legal and
/// expected during rotation (RFC 7469 §4.3 backup pin pattern) —
/// the validator accepts the cert if any non-expired entry
/// matches.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RelayPin {
    /// Full relay URL the pin applies to, including scheme.
    /// Example: `"https://relay.iroh.network"`.
    pub relay_url: String,
    /// Base64url no-padding SHA-256 of the DER-encoded
    /// SubjectPublicKeyInfo (RFC 7469 §2.4). 43 chars.
    pub spki_sha256: String,
    /// RFC 3339 timestamp of when the pin entered the store.
    /// Audit trail only; no validation effect.
    pub added_at: String,
    /// Provenance — see [`PinSource`].
    pub source: PinSource,
    /// Optional RFC 3339 timestamp past which this specific pin
    /// is considered stale and will be skipped during validation.
    /// Set by the operator during a planned rotation; `null` for
    /// steady-state pins.
    #[serde(default)]
    pub expires_at: Option<String>,
}

/// Top-level on-disk shape of `~/.sbfb/relay-pins.json`.
///
/// ```json
/// {
///   "version": 1,
///   "pins": [
///     {
///       "relay_url": "https://relay.iroh.network",
///       "spki_sha256": "Aq1c_N_zjopBnfg-mcHBozX8dgA64izVtd_zgdDioXs",
///       "added_at": "2026-04-16T10:00:00Z",
///       "source": "Bootstrap",
///       "expires_at": null
///     }
///   ]
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RelayPinsFile {
    /// Format version — see [`PIN_FILE_FORMAT_VERSION`].
    pub version: u8,
    /// Flat list of pin entries. Multiple entries per
    /// `relay_url` permitted for rotation overlap.
    #[serde(default)]
    pub pins: Vec<RelayPin>,
}

impl Default for RelayPinsFile {
    fn default() -> Self {
        Self {
            version: PIN_FILE_FORMAT_VERSION,
            pins: Vec::new(),
        }
    }
}

// =================================================================
// SPKI extract helpers
// =================================================================

/// Compute `BASE64URL_NOPAD(SHA256(SPKI_DER))` for a DER-encoded
/// X.509 certificate. SPKI_DER is the `tbsCertificate.subject_pki`
/// field as serialized (RFC 5280 §4.1.2.7), which is what RFC 7469
/// §2.4 specifies for a pin fingerprint.
///
/// The returned string is 43 chars (32 bytes × 4/3, no `=` padding).
///
/// ## Example (offline, matches the openssl pipeline in `RELAY_PIN_BOOTSTRAP.md`)
///
/// ```bash
/// openssl x509 -in cert.pem -pubkey -noout \
///   | openssl pkey -pubin -outform DER \
///   | openssl dgst -sha256 -binary \
///   | basenc --base64url --wrap=0 | tr -d '='
/// ```
pub fn extract_spki_sha256(cert_der: &[u8]) -> Result<String, PinError> {
    let (_, cert) =
        X509Certificate::from_der(cert_der).map_err(|e| PinError::ParseCert(e.to_string()))?;
    let spki_der = cert.tbs_certificate.subject_pki.raw;
    let hash = Sha256::digest(spki_der);
    Ok(URL_SAFE_NO_PAD.encode(hash))
}

/// Convenience wrapper around [`extract_spki_sha256`] that accepts
/// a PEM-encoded cert (`-----BEGIN CERTIFICATE-----` envelope).
/// Returns the same 43-char base64url no-padding fingerprint.
pub fn extract_spki_sha256_from_pem(pem_bytes: &[u8]) -> Result<String, PinError> {
    let (_, pem_obj) = parse_x509_pem(pem_bytes).map_err(|e| PinError::ParseCert(e.to_string()))?;
    extract_spki_sha256(&pem_obj.contents)
}

// =================================================================
// Path resolution
// =================================================================

/// Resolve the on-disk path to `relay-pins.json`. Honours
/// `SBFB_RELAY_PINS_FILE` (test/override) first, then falls back
/// to `$SBFB_HOME/relay-pins.json`, then `$HOME/.sbfb/relay-pins.json`
/// on Unix or `%USERPROFILE%\.sbfb\relay-pins.json` on Windows.
/// Returns `None` only on platforms where neither `HOME` nor
/// `USERPROFILE` is set — in practice a test environment that
/// deliberately un-set both.
pub fn relay_pins_file_path() -> Option<PathBuf> {
    if let Ok(custom) = env::var(CUSTOM_PINS_FILE_ENV) {
        if !custom.is_empty() {
            return Some(PathBuf::from(custom));
        }
    }
    sbfb_home().map(|d| d.join(RELAY_PINS_FILE_NAME))
}

fn sbfb_home() -> Option<PathBuf> {
    if let Ok(dir) = env::var(SBFB_HOME_ENV) {
        if !dir.is_empty() {
            return Some(PathBuf::from(dir));
        }
    }
    let home = env::var("HOME")
        .ok()
        .or_else(|| env::var("USERPROFILE").ok())?;
    if home.is_empty() {
        return None;
    }
    Some(PathBuf::from(home).join(".sbfb"))
}

fn load_from_disk(path: &Path) -> Result<RelayPinsFile, PinError> {
    match fs::read(path) {
        Ok(bytes) => {
            let file: RelayPinsFile = serde_json::from_slice(&bytes)?;
            if file.version != PIN_FILE_FORMAT_VERSION {
                return Err(PinError::UnknownVersion(file.version));
            }
            Ok(file)
        }
        Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(RelayPinsFile::default()),
        Err(e) => Err(PinError::Io(e)),
    }
}

// =================================================================
// PinValidator
// =================================================================

/// Live pin store with a hot-reload background watcher. Callers
/// hold one of these for the daemon's lifetime and call
/// [`Self::validate`] at every relay handshake. The `notify`
/// thread swaps the in-memory state atomically whenever the
/// operator or the launcher rewrites `relay-pins.json`, so a pin
/// rotation takes effect without restarting the daemon.
///
/// Construction choices:
///
/// - [`Self::new_in_memory`] — useful in tests and for callers
///   that embed the bootstrap pinset statically. No file watcher
///   is installed.
/// - [`Self::from_file_with_watch`] — production path: reads the
///   file once, installs the watcher, returns a ready-to-use
///   validator.
pub struct PinValidator {
    inner: Arc<RwLock<RelayPinsFile>>,
    _watcher: Option<RecommendedWatcher>,
    _join: Option<thread::JoinHandle<()>>,
}

impl std::fmt::Debug for PinValidator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let pin_count = self
            .inner
            .read()
            .map(|g| g.pins.len())
            .unwrap_or(usize::MAX);
        f.debug_struct("PinValidator")
            .field("pin_count", &pin_count)
            .field("has_watcher", &self._watcher.is_some())
            .finish()
    }
}

impl PinValidator {
    /// Build a validator from an already-loaded [`RelayPinsFile`]
    /// with no file watcher. Used by tests and by callers that
    /// carry their pinset in memory (e.g. bootstrap pins compiled
    /// into the release binary).
    pub fn new_in_memory(file: RelayPinsFile) -> Self {
        Self {
            inner: Arc::new(RwLock::new(file)),
            _watcher: None,
            _join: None,
        }
    }

    /// Load `relay-pins.json` from `path` and spawn a `notify`
    /// background thread that hot-reloads it on any change. The
    /// thread watches the parent directory rather than the file
    /// itself so write+rename atomic replacements (standard
    /// editor save) still trigger a reload.
    ///
    /// If the file is absent at construction time, the initial
    /// pinset is empty and [`Self::validate`] fails open until
    /// the operator drops a real file in place — at which point
    /// the watcher picks it up and every relay URL present in the
    /// file becomes enforced.
    pub fn from_file_with_watch(path: impl Into<PathBuf>) -> Result<Self, PinError> {
        let path = path.into();
        let initial = load_from_disk(&path)?;
        let inner = Arc::new(RwLock::new(initial));

        let parent = path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| PathBuf::from("."));
        if !parent.exists() {
            fs::create_dir_all(&parent)?;
        }

        let (tx, rx) = mpsc::channel::<notify::Result<Event>>();
        let mut watcher: RecommendedWatcher = notify::recommended_watcher(tx)?;
        watcher.watch(&parent, RecursiveMode::NonRecursive)?;

        let inner_thread = Arc::clone(&inner);
        let path_thread = path.clone();
        let join = thread::Builder::new()
            .name("sbfb-relay-pins-watch".into())
            .spawn(move || {
                while let Ok(evt) = rx.recv() {
                    match evt {
                        Ok(event) => {
                            if !event.paths.iter().any(|p| p == &path_thread) {
                                continue;
                            }
                            if matches!(event.kind, EventKind::Remove(_)) {
                                warn!(
                                    path = %path_thread.display(),
                                    "relay-pins.json removed — keeping in-memory pinset until recreated"
                                );
                                continue;
                            }
                            if !matches!(
                                event.kind,
                                EventKind::Modify(_) | EventKind::Create(_)
                            ) {
                                continue;
                            }
                            thread::sleep(RELOAD_DEBOUNCE);
                            match load_from_disk(&path_thread) {
                                Ok(new_file) => {
                                    if let Ok(mut g) = inner_thread.write() {
                                        *g = new_file;
                                        debug!(
                                            path = %path_thread.display(),
                                            "relay-pins.json reloaded"
                                        );
                                    }
                                }
                                Err(e) => warn!(
                                    error = %e,
                                    path = %path_thread.display(),
                                    "relay-pins.json reload failed; keeping previous pinset"
                                ),
                            }
                        }
                        Err(e) => warn!(error = %e, "relay-pins watcher event error"),
                    }
                }
            })?;

        Ok(Self {
            inner,
            _watcher: Some(watcher),
            _join: Some(join),
        })
    }

    /// Validate the cert the relay presented at `relay_url`
    /// against the in-memory pinset. See the module-level doc
    /// comment for the exact fail-open vs fail-closed matrix.
    pub fn validate(&self, relay_url: &str, cert_der: &[u8]) -> Result<(), PinError> {
        let file = self.inner.read().map_err(|_| PinError::Poisoned)?;
        if file.pins.is_empty() {
            warn!(
                relay_url = %relay_url,
                "relay-pins.json empty or missing — falling back to WebPKI validation only"
            );
            return Ok(());
        }
        let candidates: Vec<&RelayPin> = file
            .pins
            .iter()
            .filter(|p| p.relay_url == relay_url)
            .collect();
        if candidates.is_empty() {
            return Err(PinError::NoPin(relay_url.to_string()));
        }
        let actual_spki = extract_spki_sha256(cert_der)?;
        let now = Utc::now();
        let mut any_expired = false;
        for pin in &candidates {
            if let Some(exp) = &pin.expires_at {
                let exp_dt: DateTime<Utc> = exp
                    .parse()
                    .map_err(|e: chrono::ParseError| PinError::InvalidTimestamp(e.to_string()))?;
                if now > exp_dt {
                    any_expired = true;
                    continue;
                }
            }
            if pin.spki_sha256 == actual_spki {
                return Ok(());
            }
        }
        Err(PinError::SpkiMismatch {
            relay_url: relay_url.to_string(),
            actual: actual_spki,
            pin_count: candidates.len(),
            any_expired,
        })
    }

    /// Number of pin entries currently loaded. Used by health
    /// endpoints and tests.
    pub fn pin_count(&self) -> Result<usize, PinError> {
        let g = self.inner.read().map_err(|_| PinError::Poisoned)?;
        Ok(g.pins.len())
    }

    /// Set of unique `relay_url` values the validator currently
    /// has pins for. Callers typically use this to decide whether
    /// a given URL is "known pinned" vs "fail-open fallback".
    pub fn pinned_urls(&self) -> Result<HashSet<String>, PinError> {
        let g = self.inner.read().map_err(|_| PinError::Poisoned)?;
        Ok(g.pins.iter().map(|p| p.relay_url.clone()).collect())
    }

    /// Test-only escape hatch: replace the in-memory pinset
    /// without touching the file. Lets unit tests exercise
    /// [`Self::validate`] without spinning a watcher thread.
    #[doc(hidden)]
    pub fn force_set_for_test(&self, new_file: RelayPinsFile) -> Result<(), PinError> {
        let mut g = self.inner.write().map_err(|_| PinError::Poisoned)?;
        *g = new_file;
        Ok(())
    }
}

// =================================================================
// Tests
// =================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use std::sync::Mutex;
    use tempfile::TempDir;

    // Env var mutation is process-global ; serialise tests that
    // touch SBFB_HOME / SBFB_RELAY_PINS_FILE so parallel execution
    // across this module and relay_config / relay_pow_policy /
    // pkarr_resolver (which mutate the same vars) does not produce
    // flaky cross-leakage. Same pattern as those modules.
    static ENV_GUARD: Mutex<()> = Mutex::new(());

    // Bundled self-signed test cert, regenerable via the
    // openssl pipeline in docs/release/RELAY_PIN_BOOTSTRAP.md.
    // SPKI hash is hardcoded below so any regression in the
    // extraction code breaks the test loudly.
    const TEST_CERT_PEM: &[u8] = include_bytes!("../tests/fixtures/relay_test_cert.pem");
    const TEST_CERT_SPKI_SHA256: &str = "Aq1c_N_zjopBnfg-mcHBozX8dgA64izVtd_zgdDioXs";

    fn test_cert_der() -> Vec<u8> {
        let (_, pem) = parse_x509_pem(TEST_CERT_PEM).unwrap();
        pem.contents
    }

    // -------------------------------------------------------------
    // Primitive tests — SPKI extraction
    // -------------------------------------------------------------

    #[test]
    fn extract_spki_sha256_from_pem_matches_openssl_pipeline() {
        // If this test regresses, either extract_spki_sha256 is
        // broken or docs/release/RELAY_PIN_BOOTSTRAP.md drifts
        // from the implementation. Both are in-scope blockers.
        let spki = extract_spki_sha256_from_pem(TEST_CERT_PEM).unwrap();
        assert_eq!(spki, TEST_CERT_SPKI_SHA256);
        assert_eq!(
            spki.len(),
            43,
            "base64url-no-pad SHA-256 is 43 chars (32 bytes × 4/3)"
        );
        assert!(
            !spki.contains('='),
            "base64url-no-pad must not contain padding"
        );
    }

    #[test]
    fn extract_spki_sha256_from_der_matches_pem_path() {
        let der = test_cert_der();
        let from_der = extract_spki_sha256(&der).unwrap();
        let from_pem = extract_spki_sha256_from_pem(TEST_CERT_PEM).unwrap();
        assert_eq!(from_der, from_pem);
    }

    #[test]
    fn extract_spki_sha256_rejects_garbage_bytes() {
        let err = extract_spki_sha256(&[0xFF; 32]).unwrap_err();
        assert!(matches!(err, PinError::ParseCert(_)));
    }

    // -------------------------------------------------------------
    // Primitive tests — validate()
    // -------------------------------------------------------------

    fn pin_entry(url: &str, hash: &str) -> RelayPin {
        RelayPin {
            relay_url: url.to_string(),
            spki_sha256: hash.to_string(),
            added_at: "2026-04-16T10:00:00Z".to_string(),
            source: PinSource::Bootstrap,
            expires_at: None,
        }
    }

    #[test]
    fn validate_accepts_matching_pin() {
        let file = RelayPinsFile {
            version: 1,
            pins: vec![pin_entry(
                "https://relay.test.invalid",
                TEST_CERT_SPKI_SHA256,
            )],
        };
        let v = PinValidator::new_in_memory(file);
        v.validate("https://relay.test.invalid", &test_cert_der())
            .expect("matching pin must validate");
    }

    #[test]
    fn validate_rejects_mismatched_pin() {
        let file = RelayPinsFile {
            version: 1,
            pins: vec![pin_entry(
                "https://relay.test.invalid",
                "wrong_hash_wrong_hash_wrong_hash_wrong_hash",
            )],
        };
        let v = PinValidator::new_in_memory(file);
        let err = v
            .validate("https://relay.test.invalid", &test_cert_der())
            .unwrap_err();
        match err {
            PinError::SpkiMismatch {
                relay_url,
                actual,
                pin_count,
                any_expired,
            } => {
                assert_eq!(relay_url, "https://relay.test.invalid");
                assert_eq!(actual, TEST_CERT_SPKI_SHA256);
                assert_eq!(pin_count, 1);
                assert!(!any_expired);
            }
            other => panic!("expected SpkiMismatch, got {other:?}"),
        }
    }

    #[test]
    fn validate_no_pin_for_relay_when_pinset_non_empty() {
        let file = RelayPinsFile {
            version: 1,
            pins: vec![pin_entry(
                "https://other.relay.invalid",
                TEST_CERT_SPKI_SHA256,
            )],
        };
        let v = PinValidator::new_in_memory(file);
        let err = v
            .validate("https://relay.test.invalid", &test_cert_der())
            .unwrap_err();
        assert!(matches!(err, PinError::NoPin(ref url) if url == "https://relay.test.invalid"));
    }

    #[test]
    fn validate_empty_pinset_fails_open_with_warn() {
        // Empty pinset = user has not opted in yet. We fall back
        // to WebPKI-only validation (the caller already did that
        // before invoking us). See §5.4 of the design doc.
        let v = PinValidator::new_in_memory(RelayPinsFile::default());
        v.validate("https://relay.test.invalid", &test_cert_der())
            .expect("empty pinset must fail-open");
    }

    #[test]
    fn validate_expired_pin_is_skipped() {
        let mut pin = pin_entry("https://relay.test.invalid", TEST_CERT_SPKI_SHA256);
        pin.expires_at = Some("2020-01-01T00:00:00Z".to_string());
        let file = RelayPinsFile {
            version: 1,
            pins: vec![pin],
        };
        let v = PinValidator::new_in_memory(file);
        let err = v
            .validate("https://relay.test.invalid", &test_cert_der())
            .unwrap_err();
        match err {
            PinError::SpkiMismatch { any_expired, .. } => assert!(any_expired),
            other => panic!("expected SpkiMismatch any_expired=true, got {other:?}"),
        }
    }

    #[test]
    fn validate_accepts_backup_pin_when_primary_expired() {
        // RFC 7469 §4.3 backup pin: multiple entries for the same
        // URL, validator accepts if any non-expired pin matches.
        let mut expired = pin_entry(
            "https://relay.test.invalid",
            "expired_wrong_hash_wrong_hash",
        );
        expired.expires_at = Some("2020-01-01T00:00:00Z".to_string());
        let fresh = pin_entry("https://relay.test.invalid", TEST_CERT_SPKI_SHA256);
        let file = RelayPinsFile {
            version: 1,
            pins: vec![expired, fresh],
        };
        let v = PinValidator::new_in_memory(file);
        v.validate("https://relay.test.invalid", &test_cert_der())
            .expect("backup-pin rotation path must validate");
    }

    // -------------------------------------------------------------
    // Loader tests — serde round-trip + fail-loud parse
    // -------------------------------------------------------------

    #[test]
    fn loader_parses_valid_two_pin_json() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(RELAY_PINS_FILE_NAME);
        let body = format!(
            r#"{{
                "version": 1,
                "pins": [
                  {{
                    "relay_url": "https://relay.test.invalid",
                    "spki_sha256": "{TEST_CERT_SPKI_SHA256}",
                    "added_at": "2026-04-16T10:00:00Z",
                    "source": "Bootstrap"
                  }},
                  {{
                    "relay_url": "https://backup.relay.invalid",
                    "spki_sha256": "other_hash_other_hash_other_hash_other_hash",
                    "added_at": "2026-04-16T10:00:00Z",
                    "source": "UserOverride",
                    "expires_at": null
                  }}
                ]
            }}"#
        );
        std::fs::write(&path, body).unwrap();

        let v = PinValidator::from_file_with_watch(&path).unwrap();
        assert_eq!(v.pin_count().unwrap(), 2);
        let urls = v.pinned_urls().unwrap();
        assert!(urls.contains("https://relay.test.invalid"));
        assert!(urls.contains("https://backup.relay.invalid"));
    }

    #[test]
    fn loader_missing_file_yields_empty_fail_open_pinset() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(RELAY_PINS_FILE_NAME);
        // File does not exist — load_from_disk returns default().
        let v = PinValidator::from_file_with_watch(&path).unwrap();
        assert_eq!(v.pin_count().unwrap(), 0);
        v.validate("https://anything.invalid", &test_cert_der())
            .expect("empty pinset must fail-open (warn)");
    }

    #[test]
    fn loader_invalid_json_fails_loud() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(RELAY_PINS_FILE_NAME);
        std::fs::write(&path, b"{not valid json").unwrap();
        let err = PinValidator::from_file_with_watch(&path).unwrap_err();
        assert!(matches!(err, PinError::Json(_)));
    }

    #[test]
    fn loader_rejects_unknown_version() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(RELAY_PINS_FILE_NAME);
        std::fs::write(&path, br#"{"version": 99, "pins": []}"#).unwrap();
        let err = PinValidator::from_file_with_watch(&path).unwrap_err();
        assert!(matches!(err, PinError::UnknownVersion(99)));
    }

    #[test]
    fn loader_rejects_unknown_field() {
        // deny_unknown_fields: if a future version extends the
        // schema, old clients must fail loud, not silently ignore.
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(RELAY_PINS_FILE_NAME);
        std::fs::write(&path, br#"{"version": 1, "pins": [], "unexpected": true}"#).unwrap();
        let err = PinValidator::from_file_with_watch(&path).unwrap_err();
        assert!(matches!(err, PinError::Json(_)));
    }

    // -------------------------------------------------------------
    // Hot-reload test — notify file-watcher picks up a rewrite
    // -------------------------------------------------------------

    #[test]
    fn hot_reload_picks_up_external_rewrite() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join(RELAY_PINS_FILE_NAME);
        // Seed with an empty pinset.
        std::fs::write(&path, br#"{"version": 1, "pins": []}"#).unwrap();
        let v = PinValidator::from_file_with_watch(&path).unwrap();
        assert_eq!(v.pin_count().unwrap(), 0);

        // Rewrite with a real pin.
        let body = format!(
            r#"{{"version":1,"pins":[{{"relay_url":"https://relay.test.invalid","spki_sha256":"{TEST_CERT_SPKI_SHA256}","added_at":"2026-04-16T10:00:00Z","source":"Bootstrap"}}]}}"#
        );
        // Atomic rewrite (write+rename pattern the watcher must cope with).
        let tmp = path.with_extension("tmp");
        {
            let mut f = std::fs::File::create(&tmp).unwrap();
            f.write_all(body.as_bytes()).unwrap();
        }
        std::fs::rename(&tmp, &path).unwrap();

        // Poll for reload — the watcher debounces 50ms then
        // reloads. Give it up to 2 seconds on slow CI.
        let deadline = std::time::Instant::now() + Duration::from_secs(2);
        loop {
            if v.pin_count().unwrap() == 1 {
                break;
            }
            if std::time::Instant::now() > deadline {
                panic!("hot-reload did not observe the rewrite within 2s");
            }
            thread::sleep(Duration::from_millis(25));
        }
        v.validate("https://relay.test.invalid", &test_cert_der())
            .expect("post-reload pin must validate");
    }

    // -------------------------------------------------------------
    // Path resolution
    // -------------------------------------------------------------

    #[test]
    fn relay_pins_file_path_honours_custom_env_override() {
        // Hold ENV_GUARD across the whole test — env mutation is
        // process-global.
        let _lock = ENV_GUARD.lock().unwrap();
        let dir = TempDir::new().unwrap();
        let custom = dir.path().join("custom-pins.json");
        let _guard = EnvGuard::set(CUSTOM_PINS_FILE_ENV, custom.to_string_lossy().as_ref());
        let resolved = relay_pins_file_path().expect("custom env gives a path");
        assert_eq!(resolved, custom);
    }

    #[test]
    fn relay_pins_file_path_falls_back_to_sbfb_home() {
        let _lock = ENV_GUARD.lock().unwrap();
        let dir = TempDir::new().unwrap();
        let _g1 = EnvGuard::unset(CUSTOM_PINS_FILE_ENV);
        let _g2 = EnvGuard::set(SBFB_HOME_ENV, dir.path().to_string_lossy().as_ref());
        let resolved = relay_pins_file_path().expect("SBFB_HOME gives a path");
        assert_eq!(resolved, dir.path().join(RELAY_PINS_FILE_NAME));
    }

    // Minimal env guard — restores the prior value on drop so
    // concurrent tests cannot leak state. std::env is process-
    // wide and cargo test threads share it.
    struct EnvGuard {
        key: String,
        prior: Option<String>,
    }

    impl EnvGuard {
        fn set(key: &str, value: &str) -> Self {
            let prior = env::var(key).ok();
            env::set_var(key, value);
            Self {
                key: key.to_string(),
                prior,
            }
        }
        fn unset(key: &str) -> Self {
            let prior = env::var(key).ok();
            env::remove_var(key);
            Self {
                key: key.to_string(),
                prior,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match &self.prior {
                Some(v) => env::set_var(&self.key, v),
                None => env::remove_var(&self.key),
            }
        }
    }
}
