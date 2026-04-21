# Sprint 25 — Plan

**Ecrit** : 2026-04-22.
**Kickoff** : `sprint25_kickoff.md` (meme commit).
**Theme** : fondations securitaires pre-tool-calling.

---

## Phase A — P2 cleanup batch DNS concurrent + quarantine alerting

### A.1 P2-E-1 : per-endpoint TLS name

**Fichier** : `crates/nexus-core-rs/src/dns_fallback.rs`

Actuellement `build_resolver` prend `endpoints[0].tls_name.clone()`
et l'applique a tous les IPs du groupe. Refactor : chaque
`DnsEndpoint` conserve deja un champ `tls_name: String`. Le resolver
itere les endpoints et utilise le `tls_name` individuel au lieu du
premier.

**Tests** : nouveau test `per_endpoint_tls_name_used` qui configure
2 endpoints avec des TLS names differents et verifie que chaque
appel au resolver utilise le bon.

### A.2 P2-E-2 : concurrent DoH + DoT

**Fichier** : `crates/nexus-core-rs/src/dns_fallback.rs`

Actuellement `resolve_node` essaie DoH puis DoT sequentiellement
(worst-case 2×timeout). Refactor : `tokio::select!` lance les 2
branches en parallele. Premiere reponse valide gagne. Si les 2
echouent, retourner l'erreur combinee.

**Tests** : nouveau test `concurrent_doh_dot_first_wins` + test
`concurrent_both_fail_combined_error`.

### A.3 P2-D-2 : quarantine curator alerting

**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/
quarantine_queue.py`

Le hook `on_quarantine_enqueue` (S24 Phase C) fire deja quand un
worker est quarantaine. Ajouter : structured log `structlog.warning`
avec `worker_id`, `reason`, `task_id`. Le curator peut observer
via l'endpoint existant `/api/quarantine/` (S21 Phase D). Pas de
push notification (S26+ avec A3 OS audit channel).

**Tests** : test `quarantine_enqueue_fires_alert_log` verifiant le
log structure emis.

### A.4 HARDENING_ROADMAP update

Update `last_validated` → `2026-04-22` avec note S25 G2 trigger
scan result (MCP vuln active, 5 triggers inactive).

---

## Phase B — Key rotation ceremony Ed25519

### B.1 KeyRotationAnnouncement struct

**Fichier** : nouveau `crates/nexus-core-rs/src/key_rotation.rs`

```rust
pub struct KeyRotationAnnouncement {
    pub version: u16,           // KEY_ROTATION_FORMAT_VERSION = 1
    pub old_public_key: [u8; 32],
    pub new_public_key: [u8; 32],
    pub timestamp: u64,         // unix seconds
    pub reason: String,
    pub transition_days: u16,   // default 7
}
```

`sign(&self, old_signing_key: &SigningKey) -> [u8; 64]` : canonical
bytes via `DOMAIN_KEY_ROTATION_V1` + JCS (pattern canary S21 Phase E)
→ sign avec ancienne cle.

`verify(announcement: &[u8], signature: &[u8], old_public_key: &[u8; 32])
-> Result<KeyRotationAnnouncement>` : deserialize + verify sig
avec old_public_key.

### B.2 DOMAIN_KEY_ROTATION_V1

**Fichier** : `crates/nexus-core-rs/src/canonical.rs`

Ajout `pub const DOMAIN_KEY_ROTATION_V1: &[u8] = b"nexus-key-rotation-v1";`
+ `pub const KEY_ROTATION_FORMAT_VERSION: u16 = 1;`.

### B.3 RevocationCache

**Fichier** : `crates/nexus-core-rs/src/key_rotation.rs`

```rust
pub struct RevocationCache {
    entries: HashMap<[u8; 32], RevocationEntry>,
}

pub struct RevocationEntry {
    pub new_public_key: [u8; 32],
    pub transition_start: u64,
    pub transition_days: u16,
    pub reason: String,
}
```

`is_revoked(public_key: &[u8; 32]) -> bool` : check si la cle est
dans le cache ET la fenetre de transition est expiree.

`is_in_transition(public_key: &[u8; 32]) -> bool` : check si la
cle est dans le cache mais la fenetre n'a pas expire (les 2 cles
sont valides).

`apply_announcement(announcement: &KeyRotationAnnouncement) ->
Result<()>` : ajouter l'entree au cache apres verification signature.

### B.4 Update curator.rs

**Fichier** : `crates/nexus-core-rs/src/curator.rs`

`CuratorListEntry::verify_signature` : avant d'accepter, checker
le `RevocationCache`. Si la cle signataire est revoquee (transition
expiree), rejeter. Si en transition, accepter mais log warning.

Signature `verify_signature` prend un `&RevocationCache` optionnel.

### B.5 Gossip subscribe wire

**Fichier** : `crates/nexus-shell-daemon-core/src/`

Pattern `pow_policy_loader.rs` (S20 Phase C) : subscribe au topic
`nexus-grid/key-rotation/v1`, deserialize + verify chaque message,
`apply_announcement` au `RevocationCache` global (Arc<RwLock>).

### B.6 PyO3 binding

**Fichier** : `crates/nexus-core-py/src/lib.rs`

`#[pyfunction] fn verify_key_rotation(announcement_bytes: &[u8],
signature: &[u8], old_public_key: &[u8]) -> PyResult<bool>`

### B.7 Tests

- `sign_verify_rotation_announcement` : round-trip
- `wrong_key_rejects` : signature par mauvaise cle = reject
- `revocation_cache_apply_and_check` : apply + is_revoked + is_in_transition
- `transition_expired` : post-transition = revoked
- `transition_active` : mid-transition = accepted
- `curator_verify_with_revoked_key_rejects`
- `curator_verify_with_transitioning_key_warns`
- `pyo3_verify_key_rotation_roundtrip`
- `domain_separation_distinct` (DOMAIN_KEY_ROTATION_V1 ≠ existing)
- `announcement_canonical_deterministic` (JCS)
- + 10 edge cases (empty reason, 0 transition_days, future timestamp)

---

## Phase C — C3 handoffs StageGuardrailMap

### C.1 StageGuardrailMap type

**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/
guardrails.py`

```python
StageGuardrailMap = dict[str, GuardrailChain]
```

5 stages valides (identiques aux hooks S24 Phase C) :
`on_claim_broadcast`, `on_task_dispatched`, `on_result_received`,
`on_validator_post_task`, `on_quarantine_enqueue`.

### C.2 Dispatcher integration

**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/
dispatcher.py`

- Remplacer `input_chain: GuardrailChain | None` par
  `stage_guards: StageGuardrailMap | None`
- Backward compat : si `input_chain` passe, le wraper dans
  `{"on_task_dispatched": input_chain}` (transition douce)
- A chaque point de fire hook, checker `stage_guards.get(event_name)`
  et executer le chain si present

### C.3 Output chain migration

**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/
validator.py`

L'appel ad-hoc a `OutputSafetyGuardrail` dans validator.py migre
vers `stage_guards["on_result_received"]` qui contient un
`GuardrailChain([OutputSafetyGuardrail()])`.

### C.4 Tests

- `stage_guards_input_chain_preserved` : backward compat
- `stage_guards_output_chain_fires_on_result`
- `stage_guards_absent_stage_passthrough`
- `stage_guards_multiple_stages_independent`
- `stage_guards_chain_error_resilience` (pattern HookRunner fire-and-forget)
- `output_safety_migrated_from_inline`
- + 9 scenarios (empty map, None chain, ordering, tripwire propagation)

---

## Phase D — D5 capabilities gate-off-by-default

### D.1 CapabilitiesStore

**Fichier** : nouveau `packages/nexus-coordinator/src/
nexus_coordinator/capability_store.py`

- `load(path: Path) -> CapabilitiesStore` : parse TOML, verify
  integrity_hash, fallback all-OFF on tamper
- `is_enabled(cap_name: str) -> bool`
- `enable(cap_name: str, actor: str) -> None` : set enabled=True,
  update `enabled_at`/`enabled_by`, recalculate integrity_hash
- `disable(cap_name: str) -> None`
- `audit_trail() -> list[dict]` : history from TOML file

### D.2 CLI nexus-admin

**Fichier** : nouveau `packages/nexus-coordinator/src/
nexus_coordinator/cli/commands/capability.py`

Typer CLI app (pattern `quarantine.py` S21, `canary.py` S22).
5 commandes : `list`, `enable`, `disable`, `info`, `audit-trail`.
`enable`/`disable` appellent `require_admin()` avant mutation.

### D.3 Admin privilege check

**Fichier** : nouveau `packages/nexus-coordinator/src/
nexus_coordinator/admin_check.py`

`require_admin()` : euid 0 (Unix) ou IsUserAnAdmin + MIL High
(Windows). Raise `PermissionError` si non-admin. Cf.
CAPABILITY_TOGGLES.md §4.1.

### D.4 @require_capability decorator

**Fichier** : `packages/nexus-coordinator/src/nexus_coordinator/
capability_store.py` (co-locate avec le store)

FastAPI decorator : check `store.is_enabled(cap_name)`, raise
HTTPException 403 si disabled.

### D.5 Semgrep rule

**Fichier** : nouveau `.semgrep/capability_gate.yml`

Cf. CAPABILITY_TOGGLES.md §5. Pattern-match endpoints `/tool/`,
`/rag/`, `/mcp/` sans `@require_capability`.

### D.6 Tests

- `store_load_valid_toml` + `store_load_tampered_fallback_all_off`
- `store_enable_updates_hash` + `store_disable_clears`
- `cli_list_shows_all_capabilities`
- `cli_enable_requires_admin` + `cli_enable_non_admin_rejected`
- `decorator_enabled_returns_200` + `decorator_disabled_returns_403`
- `admin_check_unix_root` + `admin_check_windows_admin`
- `semgrep_rule_matches_unguarded` + `semgrep_rule_ignores_guarded`
- + 14 edge cases (missing file, corrupt TOML, unknown capability,
  double enable, audit trail chronology)

---

## Phase E — wrap-up + verification + audit plan S26

### E.1 verification.md

25+ rows fail-fast checklist. Toutes les suites de test.
Compteurs tests delta par phase.

### E.2 audit_plan S26

Tracks par phase S25 pour l'auditeur independant S26 Phase 0.

### E.3 SPRINT_LOG.md + CLAUDE.md updates

Ajouter row S25 dans la table v1.2.
Update CLAUDE.md §Etat actuel (compteurs, carries).

### E.4 Memory update

Update `nexus_grid_pivot.md` tip + compteurs.
Update `MEMORY.md` index si necessaire.

### E.5 Migration planning

`git mv .planning/active/sprint25_*.md .planning/archive/v1.2/`

---

## Projection tests delta

| Phase | Rust | Python | Frontend | Total |
|---|---|---|---|---|
| A | +4 (DNS) | +3 (quarantine) | 0 | +7 |
| B | +20 (key rotation) | +2 (PyO3) | 0 | +22 |
| C | 0 | +15 (stage guards) | 0 | +15 |
| D | 0 | +25 (store+CLI+decorator) | 0 | +25 |
| **Total** | **+24** | **+45** | **0** | **+69** |

Projection : ~1690 tests (1621 + 69).
