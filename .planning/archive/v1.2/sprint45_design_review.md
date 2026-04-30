# Sprint 45 — Design Review Report (G1 scoring)

**Reviewed** : 2026-04-30 (D1..D4 from `sprint45_kickoff.md` §4)
**Reviewer** : Factual verification agent
**Format** : 5-15 lines max per decision

---

## D1 — Scope réaliste S45 : suppression maximale, pas totale

**Scoring** : ✅ **VERIFIED**

Source factuelle confirmée et récente (document 2026-04-30, kickoff post-audit S44). Séparation routes autonomes vs runtime apps vérifiée :
- **Routes autonomes à porter** (invite.py 97 LOC + quarantine.py 113 LOC) : logique Rust existante (invite.rs, quarantine_queue.rs) localisée
- **Routes app-specific non portables S45** : apps.py défini clairement 4 routes `/app/{name}/...` (manifest, tabs, commands, invoke) dépendantes de `AppContext` + `NexusApp ABC` de `nexus_sdk`
- **Dépendance SDK confirmée** : events.py ligne 50 : `from nexus_sdk import AppEvents`. Fixture: `async with bus.subscribe(pattern)` impossible sans bus Rust équivalent
- **Portage Python/Rust paire** : roadmap §S45 documenta l'écart (2500+ LOC non portables), justification cohérente
- Aucune alternative observable qui remettrait en question la limite scope.

---

## D2 — Route portage batch : invite + quarantine

**Scoring** : ✅ **VERIFIED**

Existence Rust confirmée :
- ✅ `/c/Users/FlowUP/Documents/Code/nexus/crates/nexus-coordinator-rs/src/invite.rs` (porté S41 Phase B, docstring conforme)
- ✅ `/c/Users/FlowUP/Documents/Code/nexus/crates/nexus-coordinator-rs/src/quarantine_queue.rs` (porté S21 Phase D)

Existence Python confirmée :
- ✅ `/c/Users/FlowUP/Documents/Code/nexus/packages/nexus-coordinator/src/nexus_coordinator/api/invites.py` (2951 bytes, mod 2026-04-12)
- ✅ `/c/Users/FlowUP/Documents/Code/nexus/packages/nexus-coordinator/src/nexus_coordinator/api/quarantine.py` (4207 bytes, mod 2026-04-19)

Pattern S42-S44 (State extractor + Json + coordinator_db) observable dans handlers Rust existants. Pas de contradiction.

---

## D3 — Carries resolus S45

**Scoring** : ✅ **VERIFIED**, ⚠️ **One shadow**

**(a) SHA-256→BLAKE3 resolve** ✅
- Confirmé : `redundancy.rs` ligne 7 : `use sha2::{Digest, Sha256};` (actuellement SHA-256)
- Migration vers `blake3::hash()` triviale (~10 LOC annoncé), faisable atomiquement

**(b) coord dead_code cleanup** ✅
- `http.rs` ligne 141-148 : `#[allow(dead_code)]` sur `coord_http_client`, `coord_base_url`
- Grep exhaustif confirme : 0 consumer (`.coord_http_client` pattern) dans handlers nexus-shell-daemon
- **Shadow** : `resolve_coord_base_url()` appelé dans `runtime.rs:511` lors du boot — vérifier avant suppression que le contexte init ne le réclame plus ou le passer en unused

**(c) worker_state tokio::fs** ✅
- `worker_state_api.rs` ligne 35 : `std::fs::read_to_string(&path)` confirmé bloquant dans handler async
- Migration vers `tokio::fs::read_to_string()` triviale (~5 LOC)

Autres carries (d..g) : P3, pas verifiés en détail mais déclares ~10 LOC chacun, scope serré.

---

## D4 — Coordinator Python gut

**Scoring** : ✅ **VERIFIED**, ⚠️ **Compte strict D4 vs kickoff discrepancy**

12 fichiers routes Python à supprimer, tous conforme et avec équivalent Rust :
1. ✅ deploy.py → deploy.rs
2. ✅ apps.py → apps.rs
3. ✅ consent.py → consent.rs
4. ✅ files.py → files.rs
5. ✅ canary.py → canary_api.rs
6. ✅ contributor.py → contributor_api.rs
7. ✅ health.py → health_api.rs
8. ✅ shell.py → shell_api.rs
9. ✅ tasks.py → tasks_api.rs
10. ✅ kudos.py → kudos_api.rs
11. ✅ diagnostic.py → diagnostic_api.rs
12. ✅ worker_state.py → worker_state_api.rs

**Angle mort** : D4 déclare ~14 modules Python redondants à supprimer (dispatcher.py, validator.py, etc.) mais liste §D4 ne specifie pas les modules non-route à conserver vs supprimer. Risque R3 (import chain breakage) nécessite que tests Python post-suppression passent intégralement — confirmer via pytest avant nettoyage en Phase B.

---

## Synthèse d'angles morts G1

| # | Finding | Impact | Mitigation conseille |
|---|---------|--------|-----|
| Shadow-1 | `resolve_coord_base_url()` appelee au boot dans `runtime.rs:511` — verifier pas de deadlock init avant suppression | Low-Med | Grep boot path avant suppress |
| Shadow-2 | Modules Python non-route a conserver (hooks, canary_input, rerun) — liste D4 lists 14 modules mais D4.Conserver ne specifie pas lesquels garder | Medium | Clarifier D4 Conserve list avant Phase B |
| Info | events.py + apps.py non affectes S45 — ✅ scope freeze reconfirme | None | None |

---

**Verdict G1** : **EXECUTE** — toutes decisions factellement soutenables. Shadow-1 trivial (grep). Shadow-2 déjà identifié en R3, mitigé par test suite.
