# Sprint 55 — Audit Plan (pour session fraiche S55)

**Sprint audite** : S54 (edition 2024 + dette pair + E2E wire + CI infra).
**Tip a auditer** : `5e12d14` (HEAD post-Phase D fixes).
**Phases** : A (edition 2024), B (dette pair), C (E2E wire), D (CI infra).
**Compteurs S54 sortie** : 1207 Rust / 250 Vitest / 42+2f PW / 6/6 size / ~1463 total.

---

## Track A — Edition 2024 integrity

**Objectif** : verifier que la migration edition 2024 est complete et
correcte.

1. `grep 'edition' Cargo.toml` → "2024"
2. `cargo clippy --workspace --all-targets --locked -- -D warnings` → 0
3. Verifier que les 3 crates downgrades (forbid→deny) ont bien
   `#![cfg_attr(test, allow(unsafe_code))]` et que `deny` est actif
   en mode production :
   ```
   grep -n "forbid\|deny.*unsafe_code\|allow.*unsafe_code" \
     crates/nexus-core-rs/src/lib.rs \
     crates/nexus-shell-daemon-core/src/lib.rs \
     crates/nexus-worker-core/src/lib.rs
   ```
4. Spot-check 3 fichiers random : les `unsafe {}` wrapping set_var
   ont des commentaires SAFETY documentes
5. Verifier qu'aucun `unsafe` n'existe hors du pattern set_var/remove_var
   dans les crates non-crypto

## Track B — Dette pair completeness

**Objectif** : verifier que les 5 items P2 S53 sont reellement resolus.

1. `grep -n set_permissions crates/nexus-shell-daemon/src/runtime.rs`
   → present avec cfg(unix) + 0o600
2. `grep -n GossipTaskConfig crates/nexus-shell-daemon/src/runtime.rs`
   → struct defini + utilise dans spawn
3. `grep -n "republish\|Duration::from_secs(45)" crates/nexus-shell-daemon/src/runtime.rs`
   → timer dans select! loop
4. `grep -n "/api/daemon/" docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md`
   → noms corrects post-S53 namespace migration
5. `grep -n "exemption\|post-plan\|inserted" docs/claude/README.md`
   → §6.9 criteres preflight exemption documentes

## Track C — E2E wire correctness

**Objectif** : verifier que tasks_doc_ticket est cable end-to-end.

1. `grep -n tasks_doc_ticket crates/nexus-coordinator-rs/src/invite.rs`
   → champ dans MintRequest + InviteRecord
2. `grep -n tasks_doc_ticket crates/nexus-shell-daemon/src/invite_api.rs`
   → DocsTicket export dans create_invite
3. `grep -n tasks_doc_ticket crates/nexus-worker-core/src/invite.rs`
   → parsing + DocsTicket from_str
4. Verifier le test invite_worker_requires_project_doc existe et passe
5. Pre-launch policy : pas de INVITE_FORMAT_VERSION bump
   `grep -n FORMAT_VERSION crates/nexus-coordinator-rs/src/invite.rs`
   → version = 1

## Track D — CI infra status

**Objectif** : verifier l'etat de l'infra CI.

1. `grep -c sha256 .woodpecker/ci-linux.yml` → >= 3 images pinnees
2. `grep nexus-core-py .github/workflows/rust-ci.yml` → absent (supprime)
3. Verifier SELF_HOSTED_BUILD.md documente l'etat VPS
4. Si possible, declencher un run GHA et documenter le run ID
   (P2-REVIEW-B-2-S52 3/3 MANDATORY carry)

## Track E — Scope cuts compliance

**Objectif** : verifier qu'aucun scope cut n'a ete viole.

1. `git diff 2f5d76c..5e12d14 --stat` → pas de fichiers dans les
   zones scope-cut (outbox persistant, rate-limit, VPS TLS, systemd, etc.)
2. Verifier que verification.md documente 12/12 scope cuts

## Track F — Test delta verification

**Objectif** : verifier le delta cumule annonce vs reel.

1. `cargo nextest run --workspace --locked 2>&1 | tail -5` → 1207
2. `cd web && npm run test:unit 2>&1 | tail -5` → 250
3. `git log --oneline 2f5d76c..5e12d14 --grep="test\|Test"` →
   identifier les commits ajoutant des tests
4. Comparer avec le delta annonce (+1 Rust, +0 Vitest)

## Track G — Carry-over accountability

**Objectif** : verifier les compteurs carries et les escalades.

1. Verifier que P2-REVIEW-B-1-S52 (Woodpecker) est documente 3/3
   MANDATORY dans verification.md + CLAUDE.md
2. Verifier que P2-REVIEW-B-2-S52 (GHA) est documente 3/3 MANDATORY
3. Verifier que P2-S53-outbox et P2-S53-browse_request sont a 2/3
4. Verifier que 7 nouveaux P2 S54 sont documentes avec source
5. Verifier que les items CLOSED ont bien ete resolus (cross-ref
   Track A-D ci-dessus)

---

## Severite attendue

- 0 P0 attendu (sprint docs + migration mecanique + wire simple)
- 0 P1 attendu (pas de changement architectural)
- 3+ P2 attendus (rigor signal G4)
- Dimensions a couvrir : edition integrity, wire correctness, CI
  status, carry-over accuracy, scope compliance, test delta

---

## Notes pour l'auditeur

- Sprint 54 pair → phase dette obligatoire Phase B (§6.2.1 Regle 1)
  — verifier qu'elle est bien presente et substantive
- Edition 2024 est la migration la plus lourde en fichiers touches
  (110 fichiers Phase A) mais mecanique — focus sur l'integrite
  des unsafe wrapping et le downgrade forbid→deny
- 2 items escalades 3/3 MANDATORY S55 : l'auditeur doit confirmer
  que l'escalade est justifiee (infra prete mais serveur manquant
  pour Woodpecker, fix committe mais non valide GHA pour la CI)
- GAP E2E tasks_doc_ticket est le livrable fonctionnel principal :
  verifier le wire end-to-end (coord → invite → worker)
