// SPDX-License-Identifier: AGPL-3.0-or-later
//! Warrant canary registry — aggregates observed canary signings and
//! duress acks, computes freshness per maintainer pubkey.
//!
//! Port of `packages/nexus-coordinator/src/nexus_coordinator/canary_registry.py`
//! (366 LOC Python → Rust, Sprint 39 Phase B).

use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use time::{Date, OffsetDateTime};

const WARN_THRESHOLD_DAYS: i64 = 30;
const ALARM_THRESHOLD_DAYS: i64 = 45;
const DURESS_ACK_WARN_DAYS: i64 = 2;
const DURESS_ACK_ALARM_DAYS: i64 = 7;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CanaryObservation {
    pub version: u32,
    pub date: String,
    pub headline: String,
    pub next_update: String,
    pub pubkey_hex: String,
    pub signature_hex: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DuressAckObservation {
    pub version: u32,
    pub date: String,
    pub message: String,
    pub pubkey_hex: String,
    pub signature_hex: String,
}

#[derive(Debug, Clone, Default, Serialize)]
pub struct CanaryFreshness {
    pub pubkey_hex: String,
    pub canary_date: Option<String>,
    pub canary_age_days: Option<i64>,
    pub canary_status: String,
    pub duress_ack_date: Option<String>,
    pub duress_ack_age_days: Option<i64>,
    pub duress_ack_status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkHealth {
    pub summary: HashMap<String, i64>,
    pub maintainers: Vec<CanaryFreshness>,
    pub observed_at: String,
}

fn classify_canary_age(days: i64) -> &'static str {
    if days < WARN_THRESHOLD_DAYS {
        "fresh"
    } else if days < ALARM_THRESHOLD_DAYS {
        "warn"
    } else {
        "stale"
    }
}

fn classify_duress_age(days: i64) -> &'static str {
    if days < DURESS_ACK_WARN_DAYS {
        "fresh"
    } else if days < DURESS_ACK_ALARM_DAYS {
        "warn"
    } else {
        "stale"
    }
}

fn today_utc() -> Date {
    OffsetDateTime::now_utc().date()
}

fn parse_date(s: &str) -> Option<Date> {
    let format = time::macros::format_description!("[year]-[month]-[day]");
    Date::parse(s, &format).ok()
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct PersistedRegistry {
    canaries: Vec<CanaryObservation>,
    duress_acks: Vec<DuressAckObservation>,
}

#[derive(Debug)]
pub struct CanaryRegistry {
    canaries: HashMap<String, CanaryObservation>,
    duress_acks: HashMap<String, DuressAckObservation>,
    persist_path: PathBuf,
}

impl CanaryRegistry {
    pub fn new(persist_path: PathBuf) -> Self {
        let mut reg = Self {
            canaries: HashMap::new(),
            duress_acks: HashMap::new(),
            persist_path,
        };
        reg.load_if_exists();
        reg
    }

    fn load_if_exists(&mut self) {
        if !self.persist_path.exists() {
            return;
        }
        let text = match std::fs::read_to_string(&self.persist_path) {
            Ok(t) => t,
            Err(e) => {
                tracing::warn!(path = %self.persist_path.display(), error = %e, "canary_registry.load_failed");
                return;
            }
        };
        let raw: PersistedRegistry = match serde_json::from_str(&text) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(path = %self.persist_path.display(), error = %e, "canary_registry.parse_failed");
                return;
            }
        };
        for obs in raw.canaries {
            self.canaries.insert(obs.pubkey_hex.clone(), obs);
        }
        for obs in raw.duress_acks {
            self.duress_acks.insert(obs.pubkey_hex.clone(), obs);
        }
    }

    pub fn persist(&self) -> io::Result<()> {
        if let Some(parent) = self.persist_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let payload = PersistedRegistry {
            canaries: self.canaries.values().cloned().collect(),
            duress_acks: self.duress_acks.values().cloned().collect(),
        };
        let json = serde_json::to_string_pretty(&payload).map_err(io::Error::other)?;
        let tmp = self.persist_path.with_extension("json.tmp");
        std::fs::write(&tmp, &json)?;
        std::fs::rename(&tmp, &self.persist_path)?;
        Ok(())
    }

    pub fn observe_canary(&mut self, obs: CanaryObservation) {
        if let Some(prev) = self.canaries.get(&obs.pubkey_hex) {
            if prev.date >= obs.date {
                return;
            }
        }
        self.canaries.insert(obs.pubkey_hex.clone(), obs);
        let _ = self.persist();
    }

    pub fn observe_duress_ack(&mut self, obs: DuressAckObservation) {
        if let Some(prev) = self.duress_acks.get(&obs.pubkey_hex) {
            if prev.date >= obs.date {
                return;
            }
        }
        self.duress_acks.insert(obs.pubkey_hex.clone(), obs);
        let _ = self.persist();
    }

    pub fn known_pubkeys(&self) -> Vec<String> {
        let mut keys: std::collections::HashSet<&str> =
            self.canaries.keys().map(|k| k.as_str()).collect();
        for k in self.duress_acks.keys() {
            keys.insert(k.as_str());
        }
        let mut sorted: Vec<String> = keys.into_iter().map(|s| s.to_string()).collect();
        sorted.sort();
        sorted
    }

    pub fn freshness(&self, pubkey_hex: &str) -> CanaryFreshness {
        self.freshness_at(pubkey_hex, today_utc())
    }

    pub fn freshness_at(&self, pubkey_hex: &str, today: Date) -> CanaryFreshness {
        let mut result = CanaryFreshness {
            pubkey_hex: pubkey_hex.to_string(),
            canary_status: "missing".into(),
            duress_ack_status: "missing".into(),
            ..Default::default()
        };

        if let Some(canary) = self.canaries.get(pubkey_hex) {
            result.canary_date = Some(canary.date.clone());
            if let Some(d) = parse_date(&canary.date) {
                let age = (today - d).whole_days();
                if age >= 0 {
                    result.canary_age_days = Some(age);
                    result.canary_status = classify_canary_age(age).into();
                }
            }
        }

        if let Some(ack) = self.duress_acks.get(pubkey_hex) {
            result.duress_ack_date = Some(ack.date.clone());
            if let Some(d) = parse_date(&ack.date) {
                let age = (today - d).whole_days();
                if age >= 0 {
                    result.duress_ack_age_days = Some(age);
                    result.duress_ack_status = classify_duress_age(age).into();
                }
            }
        }

        result
    }

    pub fn network_health(&self) -> NetworkHealth {
        self.network_health_at(today_utc())
    }

    pub fn network_health_at(&self, today: Date) -> NetworkHealth {
        let pubkeys = self.known_pubkeys();
        let maintainers: Vec<CanaryFreshness> = pubkeys
            .iter()
            .map(|pk| self.freshness_at(pk, today))
            .collect();

        let mut summary = HashMap::new();
        summary.insert("maintainers_total".into(), maintainers.len() as i64);
        for status in ["fresh", "warn", "stale", "missing"] {
            summary.insert(format!("canary_{status}"), 0);
            summary.insert(format!("duress_ack_{status}"), 0);
        }
        for entry in &maintainers {
            *summary
                .entry(format!("canary_{}", entry.canary_status))
                .or_insert(0) += 1;
            *summary
                .entry(format!("duress_ack_{}", entry.duress_ack_status))
                .or_insert(0) += 1;
        }

        NetworkHealth {
            summary,
            maintainers,
            observed_at: OffsetDateTime::now_utc()
                .format(&time::macros::format_description!(
                    "[year]-[month]-[day]T[hour]:[minute]:[second]Z"
                ))
                .unwrap_or_default(),
        }
    }
}

pub fn coerce_canary_payload(payload: &serde_json::Value) -> Result<CanaryObservation, String> {
    let mut obj = match payload.as_object() {
        Some(o) => o.clone(),
        None => return Err("expected JSON object".into()),
    };
    if obj.contains_key("v") && !obj.contains_key("version") {
        if let Some(v) = obj.remove("v") {
            obj.insert("version".into(), v);
        }
    }
    serde_json::from_value(serde_json::Value::Object(obj)).map_err(|e| e.to_string())
}

pub fn coerce_duress_ack_payload(
    payload: &serde_json::Value,
) -> Result<DuressAckObservation, String> {
    let mut obj = match payload.as_object() {
        Some(o) => o.clone(),
        None => return Err("expected JSON object".into()),
    };
    if obj.contains_key("v") && !obj.contains_key("version") {
        if let Some(v) = obj.remove("v") {
            obj.insert("version".into(), v);
        }
    }
    serde_json::from_value(serde_json::Value::Object(obj)).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn make_canary(pubkey: &str, date: &str) -> CanaryObservation {
        CanaryObservation {
            version: 1,
            date: date.into(),
            headline: "all clear".into(),
            next_update: "2026-06-01".into(),
            pubkey_hex: pubkey.into(),
            signature_hex: "a".repeat(128),
        }
    }

    fn make_ack(pubkey: &str, date: &str) -> DuressAckObservation {
        DuressAckObservation {
            version: 1,
            date: date.into(),
            message: "ack".into(),
            pubkey_hex: pubkey.into(),
            signature_hex: "b".repeat(128),
        }
    }

    #[test]
    fn observe_canary_keeps_latest() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("canary.json");
        let mut reg = CanaryRegistry::new(path);
        let pk = "a".repeat(64);
        reg.observe_canary(make_canary(&pk, "2026-04-01"));
        reg.observe_canary(make_canary(&pk, "2026-04-15"));
        reg.observe_canary(make_canary(&pk, "2026-04-10"));
        assert_eq!(reg.canaries[&pk].date, "2026-04-15");
    }

    #[test]
    fn observe_duress_ack_keeps_latest() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("canary.json");
        let mut reg = CanaryRegistry::new(path);
        let pk = "b".repeat(64);
        reg.observe_duress_ack(make_ack(&pk, "2026-04-01"));
        reg.observe_duress_ack(make_ack(&pk, "2026-04-20"));
        assert_eq!(reg.duress_acks[&pk].date, "2026-04-20");
    }

    #[test]
    fn freshness_fresh() {
        let dir = TempDir::new().unwrap();
        let mut reg = CanaryRegistry::new(dir.path().join("c.json"));
        let pk = "c".repeat(64);
        reg.observe_canary(make_canary(&pk, "2026-04-25"));
        let today = Date::from_calendar_date(2026, time::Month::April, 29).unwrap();
        let f = reg.freshness_at(&pk, today);
        assert_eq!(f.canary_status, "fresh");
        assert_eq!(f.canary_age_days, Some(4));
    }

    #[test]
    fn freshness_stale() {
        let dir = TempDir::new().unwrap();
        let mut reg = CanaryRegistry::new(dir.path().join("c.json"));
        let pk = "d".repeat(64);
        reg.observe_canary(make_canary(&pk, "2026-03-01"));
        let today = Date::from_calendar_date(2026, time::Month::April, 29).unwrap();
        let f = reg.freshness_at(&pk, today);
        assert_eq!(f.canary_status, "stale");
        assert_eq!(f.canary_age_days, Some(59));
    }

    #[test]
    fn freshness_unknown_key() {
        let dir = TempDir::new().unwrap();
        let reg = CanaryRegistry::new(dir.path().join("c.json"));
        let f = reg.freshness("unknown");
        assert_eq!(f.canary_status, "missing");
        assert_eq!(f.duress_ack_status, "missing");
    }

    #[test]
    fn network_health_mixed() {
        let dir = TempDir::new().unwrap();
        let mut reg = CanaryRegistry::new(dir.path().join("c.json"));
        let pk_fresh = "e".repeat(64);
        let pk_stale = "f".repeat(64);
        let pk_ack_only = "0".repeat(64);
        reg.observe_canary(make_canary(&pk_fresh, "2026-04-25"));
        reg.observe_canary(make_canary(&pk_stale, "2026-03-01"));
        reg.observe_duress_ack(make_ack(&pk_ack_only, "2026-04-28"));
        let today = Date::from_calendar_date(2026, time::Month::April, 29).unwrap();
        let h = reg.network_health_at(today);
        assert_eq!(h.summary["maintainers_total"], 3);
        assert_eq!(h.summary["canary_fresh"], 1);
        assert_eq!(h.summary["canary_stale"], 1);
        assert_eq!(h.summary["canary_missing"], 1);
        assert_eq!(h.summary["duress_ack_fresh"], 1);
        assert_eq!(h.summary["duress_ack_missing"], 2);
    }

    #[test]
    fn persist_and_reload() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("canary.json");
        let pk = "1".repeat(64);
        {
            let mut reg = CanaryRegistry::new(path.clone());
            reg.observe_canary(make_canary(&pk, "2026-04-20"));
            reg.observe_duress_ack(make_ack(&pk, "2026-04-21"));
        }
        let reg2 = CanaryRegistry::new(path);
        assert_eq!(reg2.canaries[&pk].date, "2026-04-20");
        assert_eq!(reg2.duress_acks[&pk].date, "2026-04-21");
    }

    #[test]
    fn coerce_canary_payload_v_rename() {
        let json = serde_json::json!({
            "v": 1,
            "date": "2026-04-20",
            "headline": "ok",
            "next_update": "2026-05-20",
            "pubkey_hex": "a".repeat(64),
            "signature_hex": "b".repeat(128),
        });
        let obs = coerce_canary_payload(&json).unwrap();
        assert_eq!(obs.version, 1);
        assert_eq!(obs.date, "2026-04-20");
    }

    #[test]
    fn coerce_canary_payload_missing_field() {
        let json = serde_json::json!({"v": 1, "date": "2026-04-20"});
        assert!(coerce_canary_payload(&json).is_err());
    }
}
