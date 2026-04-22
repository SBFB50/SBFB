# Sprint 25 — Audit Findings (Phase 0 Sprint 26)

**Date** : 2026-04-22
**Auditeur** : session fraiche, sans historique S25
**Sprint audite** : Sprint 25 (fondations securitaires pre-tool-calling)
**Commits** : `2ab0039` (G8 Phase A) a `55e42fd` (Phase D)
**Wrap-up** : `56a18b1` (Phase E) + `edae3f1` (post-S25 analysis)

---

## Verdict : PASS

0 P0, 0 P1. 5 P2 (tous pre-documentes dans `sprint25_audit_plan.md`
§4 risk zones). 2 P3 nits. Pas de fix bloquant requis. Sprint
transparent sur ses limitations connues.

---

## Track A — Key rotation ceremony (Phase B)

| # | Check | Status | Evidence |
|---|---|---|---|
| A-1 | `KeyRotationAnnouncement` struct champs | ok | `key_rotation.rs:62-81` — champs pub par design (read-only apres creation, constructeur valide) |
| A-2 | `sign()` domain separation | ok | `key_rotation.rs:131` — `canonical_bytes(&announcement, DOMAIN_KEY_ROTATION_V1)` |
| A-3 | `verify()` valide avec `old_public_key` | ok | `key_rotation.rs:154` — `crate::crypto::verify(&self.announcement.old_public_key, &bytes, &self.signature)` |
| A-4 | Canonical bytes JCS deterministic | ok | `canonical.rs:211` — `serde_jcs::to_vec`, test `announcement_canonical_deterministic` line 419 |
| A-5 | `apply_announcement` verifie sig avant insert | ok | `key_rotation.rs:262-264` — `signed.verify()?` puis `apply_verified` |
| A-6 | `is_revoked` vs `is_in_transition` semantique | ok | `key_rotation.rs:218-236` — `now >= expiry` = revoked, `now < expiry` = transition |
| A-7 | `transition_days = 0` edge case | ok | test `zero_transition_days` line 469 — revocation immediate confirmee |
| A-8 | `verify_with_revocation` check revocation AVANT sig | ok | `curator.rs:288` — `is_revoked` check en premier, puis `verify_signature` |
| A-9 | Gossip topic distinct | ok | `key_rotation.rs:49` — `nexus-grid/key-rotation/v1` (distinct canary/pow/curator topics) |
| A-10 | `KEY_ROTATION_FORMAT_VERSION = 1` | ok | `key_rotation.rs:32` |
| A-11 | Pas de tolerant decoder multi-version | ok | `key_rotation.rs:141` — `version != KEY_ROTATION_FORMAT_VERSION` = reject |

---

## Track B — StageGuardrailMap (Phase C)

| # | Check | Status | Evidence |
|---|---|---|---|
| B-1 | 5 stages valides | ok (P2 carry) | `guardrails.py:107-115` — `GUARDRAIL_STAGES` frozenset correct. Type alias `StageGuardrailMap` sans validation de cles (P2-STAGE-1, carry S26+ per Phase C review P2-C-1) |
| B-2 | Backward compat `input_chain` wrap | ok | `dispatcher.py:128-129` — `{"on_task_dispatched": input_chain}` |
| B-3 | Output chain migration | ok | `validator.py:99-101` — `output_filter` wrape dans `OutputSafetyGuardrail → GuardrailChain → {"on_result_received": chain}` |
| B-4 | Stage absent = passthrough | ok | `dispatcher.py:164` et `validator.py:262` — `.get()` retourne None, aucune action |
| B-5 | Chain error resilience | ok | `guardrails.py:91-100` — tripwire short-circuit, non-tripwire exceptions propagent (by design, distinct du HookRunner fire-and-forget) |
| B-6 | Tripwire propagation | ok | `guardrails.py:93-97` — `OutputTripwire`/`InputTripwire` raise sur tripwire |
| B-7 | Chain ordering | ok | `guardrails.py:91` — `for g in self._guardrails` preserve insertion order |

---

## Track C — D5 capabilities (Phase D)

| # | Check | Status | Evidence |
|---|---|---|---|
| C-1 | 6 capabilities correctes | ok (P3 nit plan) | `capability_store.py:34-43` — `tool_calling, streaming_bridge, mcp_server_expose, federation_canary, rag_retrieval, biometric_gate`. Conforme `CAPABILITY_TOGGLES.md`. Audit plan §Track C item 1 enumere `external_api_access, code_execution, file_system_access` = erreur dans le plan, pas dans le code |
| C-2 | All OFF par defaut | ok | `capability_store.py:130-133` — `_default()` cree tout `enabled=False` |
| C-3 | `integrity_hash` SHA-256 recalcule | ok | `capability_store.py:208-209` — `hashlib.sha256(body.encode())` a chaque `_write()` |
| C-4 | Tamper detect → all-OFF | ok | `capability_store.py:109-116` — hash mismatch = `_default(path)` + `structlog.warning` |
| C-5 | Unix admin `geteuid == 0` | ok | `admin_check.py:26` |
| C-6 | Windows admin + MIL | P2-ADMIN-1 | `admin_check.py:31-35` IsUserAnAdmin + MIL. Mais `admin_check.py:62-64` `GetSidSubAuthorityCount`/`GetSidSubAuthority` sans NULL check (risk zone #7 audit plan) |
| C-7 | `@require_capability` 403 | ok | `capability_store.py:238-246` — `HTTPException(403)` si store None ou disabled |
| C-8 | CLI enable/disable → `require_admin()` AVANT | ok | `capability.py:67` et `:80` — `require_admin()` appele avant `store.enable()` / `store.disable()` |
| C-9 | CLI audit-trail chronologique | ok | `capability_store.py:186-194` — `sorted(self._capabilities.items())` par nom, chronologie via `enabled_at` ISO |
| C-10 | Semgrep rule present et correcte | ok | `.semgrep/capability_gate.yml` — pattern `/tool/`, `/rag/`, `/mcp/` sans `@require_capability`, severity ERROR |
| C-11 | `CAPABILITY_TOGGLES.md` status updated | ok | grep confirmed `design-only → implemented` |

---

## Track D — DNS concurrent + quarantine (Phase A)

| # | Check | Status | Evidence |
|---|---|---|---|
| D-1 | P2-E-1 per-endpoint TLS name | ok | `dns_fallback.rs:202-211` — boucle `for ep in endpoints`, `tls_dns_name: Some(ep.tls_name.clone())` individuel |
| D-2 | P2-E-2 concurrent DoH+DoT `tokio::select!` | ok | `dns_fallback.rs:272-309` — `tokio::select!` race les 2 branches, premiere reponse gagne |
| D-3 | P2-E-2 both-fail erreur combinee | ok | `dns_fallback.rs:285-286` et `303-304` — `DoH={doh_err}, DoT={dot_err}` dans le message |
| D-4 | P2-D-2 quarantine alerting | ok | `quarantine_queue.py:179-185` — `structlog.warning("quarantine_curator_alert", worker_id=..., reason=..., task_id=...)` — les 3 champs presents |
| D-5 | HARDENING_ROADMAP `last_validated` | ok | verification.md row 30 PASS, grep confirmed `2026-04-22` |

---

## Track E — Process / meta

| # | Check | Status | Evidence |
|---|---|---|---|
| E-1 | G8 preflight 4/4 phases | ok | `sprint25_phase_{A,B,C,D}_preflight.md` tous presents dans `archive/v1.2/` |
| E-2 | Phase reviews | ok (P3 gap) | A + C reviews presentes. B + D reviews absentes (commit bodies riches compensent). P3-REVIEW-GAPS |
| E-3 | Commit bodies delta tests + scope cuts | ok | 4 feat commits + 4 chore(planning) G8 + 1 chore wrap-up + 1 chore post-analysis. Bodies riches avec delta cumule |
| E-4 | Dead code | ok | 0 `#[allow(dead_code)]`, 0 `#[cfg(not(test))]`, 0 `noqa F401` dans le diff S25 |
| E-5 | Pre-launch protocol | ok | `KEY_ROTATION_FORMAT_VERSION = 1` (nouveau), 6 constants `_VERSION` existantes inchangees, 0 tolerant decoder |
| E-6 | SPDX / ruff clean | ok | verification.md rows 6-7 PASS |
| E-7 | Scope cuts honores | ok | 14/14 items deferred. Grep diff: 0 scope leak (faux positifs `PartialEq` substring, `mcp_server_expose` = capability name, `redundancy_dispatcher` = parameter pre-existant) |

---

## Findings

### P2 (logged, carry S26)

| ID | Fichier | Ligne | Description | Source |
|---|---|---|---|---|
| P2-ADMIN-1 | `admin_check.py` | 62-64 | `GetSidSubAuthorityCount`/`GetSidSubAuthority` retournent des pointeurs sans check NULL. SID malformed = segfault potentiel ctypes. Tests Windows en skip (non-Windows CI). | Audit plan risk zone #7 |
| P2-CAPS-1 | `capability_store.py` | 212 | `mkdir(parents=True, exist_ok=True)` cree le repertoire `~/.sbfb/` sans permissions restrictives. Hash anti-tamper recomputable par un attaquant local sophistique. Pre-v1.0 acceptable. | Audit plan risk zone #2 |
| P2-REVOKE-1 | `key_rotation.rs` | 248 | `apply_verified()` fait `insert()` sans log/warning si entree existante pour `old_public_key`. Post-v1.0, attaquant avec ancienne cle pendant transition pourrait ecraser rotation legitime. Mitigation future : log + reject si `transition_start` anterieur. | Audit plan risk zone #5 |
| P2-HASH-1 | `capability_store.py` | 206-213 | `integrity_hash` depend du determinisme de `tomli_w.dumps()` across versions. Upgrade = hash mismatch = fallback all-OFF (fail-safe, pas fail-open, mais disruptif). | Audit plan risk zone #6 |
| P2-STAGE-1 | `guardrails.py` | 117 | `StageGuardrailMap` type alias sans validation de cles contre `GUARDRAIL_STAGES`. Typo silently ignored. | Phase C review P2-C-1, carry S26+ |

### P3 (nits)

| ID | Description |
|---|---|
| P3-AUDIT-PLAN-1 | Audit plan §Track C item 1 enumere `external_api_access, code_execution, file_system_access` mais le design doc et le code ont `streaming_bridge, federation_canary, biometric_gate`. Erreur dans le plan, pas dans le code. |
| P3-REVIEW-GAPS | Phase B et D reviews absentes de `archive/v1.2/`. Seules Phase A et C ont des `sprint25_phase_{X}_review.md`. Bodies commit riches compensent mais trace file-based incomplete. |

---

## Risk zones audit plan — disposition

| # | Zone | Verdict |
|---|---|---|
| 1 | RevocationCache concurrency (Arc\<RwLock\>) | ok — write lock bref (HashMap insert), pas de deadlock observable. Pattern identique `pow_policy_loader.rs` S20 |
| 2 | Capability store file permissions | P2-CAPS-1 (logged) |
| 3 | StageGuardrailMap validation | P2-STAGE-1 (pre-identified Phase C review, carry S26+) |
| 4 | Admin check bypass via import mock | ok — si attaquant peut modifier Python sur disque, il peut aussi editer capabilities.toml. Pas de vecteur d'attaque supplementaire |
| 5 | RevocationCache silent overwrite | P2-REVOKE-1 (logged) |
| 6 | Hash ordering determinism | P2-HASH-1 (logged, fail-safe) |
| 7 | Admin check Windows MIL null ptr | P2-ADMIN-1 (logged) |

---

## Compteurs tests verifies

| Suite | Verification.md | Audit plan entree |
|---|---|---|
| Rust nextest | 790 | 757 entree, +33 delta |
| Python coord | 372+32stale+5skip | 315+32stale+3skip entree, +57/+2 delta |
| Total | ~1712 | ~1621 entree, +91 delta |

Non re-execute (confiance verification.md 30/30 PASS, pas de
regression signal).

---

## Items carry S26

| ID | Severite | Description |
|---|---|---|
| P2-ADMIN-1 | P2 | Windows MIL null pointer guard `admin_check.py:62-64` |
| P2-CAPS-1 | P2 | Permissions restrictives `~/.sbfb/` directory |
| P2-REVOKE-1 | P2 | RevocationCache overwrite log + reject stale transition |
| P2-HASH-1 | P2 | `tomli_w` determinism guard ou hash-on-load round-trip |
| P2-STAGE-1 | P2 | StageGuardrailMap key validation (re-carry from Phase C review) |
| P2-D-1 | P2 | Redundancy persistence in-memory → SQLite (re-carry S23) |
| P2-E-1-iroh | P2 | iroh neighborhood enrichment (re-carry S23) |
