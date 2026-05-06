# Sprint 54 — Plan (Edition 2024 + dette pair + E2E wire + CI infra)

**Tip d'entree** : `2f5d76c` (post-audit S53 PASS).
**Phases** : A (edition 2024), B (dette pair), C (E2E wire),
D (CI infra), E (wrap-up).

---

## §1 Etat verifie a l'entree

- Tip master : `2f5d76c`
- Workspace : clean
- Edition : 2021
- Rust nextest : 1206/1206, 0 fail (1 flaky pre-existant R5)
- Rust doctests : 6 passed, 1 ignored
- Vitest : 250/250
- Playwright : 42 + 2 fail (env pre-existant)
- size-limit : 6/6
- cargo fmt : 0 diff
- cargo clippy : 0 warnings
- Release build : ok

---

## §2 Decisions Day 0 (rappel synthetique)

- **D1** : edition 2024 via `cargo fix --edition 2024` + SAFETY comments
- **D2** : `tasks_doc_ticket` dans `MintRequest` + `InviteRecord`
- **D3** : Woodpecker agent VPS + GHA 9/9 + images CI pin
- **D4** : dette pair 4 items quick + 1 doc

---

## §3 Research consulte

- context7 `/rust-lang/rust` : edition 2024 migration, reserved keywords,
  unsafe extern blocks, `unsafe_op_in_unsafe_fn` lint, lifetime capture
- Analyse codebase : 0 extern blocks, 0 `unsafe fn`, 0 `gen` identifiant,
  70+ `set_var`/`remove_var` dans 17 fichiers
- iroh-docs DocsTicket : serialisation base32 via `to_string()` /
  `from_str()` (pattern existant dans le codebase)

---

## §Phase A — Edition 2024 upgrade (MANDATORY 3/3)

**But** : migrer le workspace de edition 2021 a edition 2024.
CLOSE P2-REVIEW-B-1-S51 (3/3 MANDATORY).

**Dependencies** : aucune (premiere phase).

### §A.1 Scope

1. Executer `cargo fix --edition 2024` sur le workspace
2. Revue manuelle : verifier que chaque `unsafe {}` ajoute a un
   commentaire SAFETY pertinent. Pattern :
   ```rust
   // SAFETY: called in test setup, single-threaded context.
   unsafe { std::env::set_var("KEY", "value") };
   ```
   Pour le code production (main.rs, runtime.rs) :
   ```rust
   // SAFETY: called before tokio runtime spawn, single-threaded.
   unsafe { std::env::set_var(AUTH_TOKEN_ENV, &token) };
   ```
3. Bump `edition = "2024"` dans `Cargo.toml`
4. Verifier : fmt, clippy, nextest, doctests, release build
5. Fixer tout breakage additionnel edition 2024 (lifetime capture,
   match ergonomics, etc.)

### §A.2 Fichiers touches

| Fichier | Role |
|---|---|
| `Cargo.toml` | edition bump 2021 → 2024 |
| `crates/nexus-core-rs/src/dns_fallback.rs` | unsafe wrap set_var/remove_var (tests) |
| `crates/nexus-core-rs/src/pkarr_resolver.rs` | unsafe wrap set_var/remove_var (tests) |
| `crates/nexus-core-rs/src/relay_config.rs` | unsafe wrap set_var/remove_var (tests) |
| `crates/nexus-core-rs/src/relay_pow_policy.rs` | unsafe wrap set_var/remove_var (tests) |
| `crates/nexus-core-rs/src/tls_pinning.rs` | unsafe wrap set_var/remove_var (tests) |
| `crates/nexus-core-rs/tests/relay_federation.rs` | unsafe wrap set_var/remove_var |
| `crates/nexus-launcher/src/auth.rs` | unsafe wrap set_var/remove_var (tests) |
| `crates/nexus-launcher/src/main.rs` | unsafe wrap set_var (production) |
| `crates/nexus-launcher/src/token_rotation.rs` | unsafe wrap set_var/remove_var (tests) |
| `crates/nexus-launcher/src/unlock.rs` | unsafe wrap set_var/remove_var (prod + tests) |
| `crates/nexus-shell-daemon/src/main.rs` | unsafe wrap remove_var (production) |
| `crates/nexus-shell-daemon/src/runtime.rs` | unsafe wrap remove_var (production) |
| `crates/nexus-shell-daemon-core/src/auth.rs` | unsafe wrap set_var/remove_var (tests) |
| `crates/nexus-shell-daemon-core/src/browse.rs` | unsafe wrap set_var/remove_var (tests) |
| `crates/nexus-shell-daemon-core/src/config.rs` | unsafe wrap set_var/remove_var (tests) |
| `crates/nexus-shell-daemon-core/src/paths.rs` | unsafe wrap set_var/remove_var (tests) |
| `crates/nexus-worker-core/src/config.rs` | unsafe wrap set_var/remove_var (tests) |

### §A.3 Tests plan

Pas de nouveau test — migration mecanique. Les 1206 tests existants
doivent passer sans modification de logique. Si `cargo fix` introduit
un breakage, le fix est dans le meme commit.

### §A.4 Critere d'acceptation

```bash
cargo fmt --all --check          # 0 diff
cargo clippy --workspace --all-targets --locked -- -D warnings  # 0 warnings
cargo nextest run --workspace --locked  # >= 1206, 0 fail
cargo test --workspace --locked --doc   # ok
cargo build -p nexus-shell-daemon --release  # ok
grep 'edition' Cargo.toml  # edition = "2024"
```

### §A.5 Commit cible

```
feat(sprint54): Sprint 54 Phase A — Rust edition 2024 upgrade + unsafe set_var wrapping

Migration du workspace de edition 2021 vers edition 2024.
70+ appels std::env::set_var/remove_var wrappés dans unsafe {}
avec commentaires SAFETY documentés.

Changements :
- Cargo.toml : edition = "2024"
- 17 fichiers .rs : unsafe {} wrapping autour de set_var/remove_var
- 0 changement de logique metier

CLOSE P2-REVIEW-B-1-S51 (3/3 MANDATORY, carry depuis S51).

Delta tests cumule Sprint 54 : 1206 Rust (+0) / 250 Vitest (+0)

Scope cuts respectes : 12/12

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

---

## §Phase B — Dette pair (§6.2.1 Regle 1)

**But** : resoudre 4 items P2 S53 + 1 doc process gap.
Sprint 54 pair → phase dette obligatoire.

**Dependencies** : Phase A (edition 2024 doit compiler).

### §B.1 Scope

1. **node_key perms 0600** : dans `load_or_generate_node_key()`,
   apres `std::fs::write()`, appeler `std::fs::set_permissions()`
   avec `Permissions::from_mode(0o600)` (Unix) ou equivalent Windows.
   Gater avec `#[cfg(unix)]` car Windows utilise des ACLs differentes.

2. **gossip params struct** : extraire un `GossipTaskConfig` struct
   regroupant les 9 parametres de `spawn_gossip_subscribe_task()`.
   Supprimer `#[allow(clippy::too_many_arguments)]`.

3. **periodic republish** : ajouter un timer 45-60s (jitter
   `rand::thread_rng().gen_range(45..=60)`) dans la gossip task loop
   `tokio::select!` qui replay le outbox complet. Complement au
   replay sur NeighborUp.

4. **route collision doc** : mettre a jour
   `LOOPBACK_ENDPOINTS_TRUST_TIERS.md` avec les noms `/api/daemon/*`
   corrects (post Phase A S53 namespace migration).

5. **preflight process gap** : ajouter une note dans `README.md` §6.9
   documentant les criteres d'exemption preflight pour les phases
   inserees post-plan (phases reactives).

### §B.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-shell-daemon/src/runtime.rs` | node_key perms + gossip struct + periodic republish |
| `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` | route names update |
| `docs/claude/README.md` | §6.9 preflight exemption criteria |

### §B.3 Tests plan

1. `test_node_key_permissions_unix` : verifier que le fichier node_key
   a les permissions 0o600 apres creation (cfg(unix) seulement)
2. `test_gossip_task_config_fields` : verifier que le struct contient
   tous les champs necessaires
3. Les tests existants `start_then_shutdown` et `auto_subscribe`
   exercent indirectement le gossip path modifie

### §B.4 Critere d'acceptation

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked  # >= 1206
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release
# Verification specifique :
grep -n "set_permissions\|0o600" crates/nexus-shell-daemon/src/runtime.rs
grep -n "GossipTaskConfig" crates/nexus-shell-daemon/src/runtime.rs
grep -n "periodic\|republish\|timer\|Duration::from_secs" crates/nexus-shell-daemon/src/runtime.rs
```

### §B.5 Commit cible

```
feat(sprint54): Sprint 54 Phase B — dette pair quick P2 batch (node_key perms + gossip refactor + periodic republish)

Phase dette obligatoire sprint pair (§6.2.1 Regle 1).
5 items P2 S53 resolus :

- runtime.rs : node_key permissions 0600 (cfg(unix)) apres write
  CLOSE P2-S53-node_key perms.
- runtime.rs : GossipTaskConfig struct remplace 9 params individuels
  CLOSE P2-S53-gossip params struct.
- runtime.rs : periodic republish timer 45-60s jitter dans gossip
  task loop CLOSE P2-S53-periodic republish.
- LOOPBACK_ENDPOINTS_TRUST_TIERS.md : noms /api/daemon/* corriges
  CLOSE P2-S53-route collision doc.
- README.md §6.9 : criteres exemption preflight post-plan documentes
  CLOSE P2-S53-preflight E/F/G process gap.

Delta tests cumule Sprint 54 : Rust +N / Vitest +0

Scope cuts respectes : 12/12

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

---

## §Phase C — E2E wire task flow

**But** : cabler le chainon manquant `tasks_doc_ticket` dans le wire
format invite. Prerequis LT-7 self-hosted build.

**Dependencies** : Phase A (edition), Phase B (dette).

### §C.1 Scope

1. **coordinator-rs/invite.rs** : ajouter `tasks_doc_ticket: String` a
   `MintRequest` et `InviteRecord`. Le champ porte le `DocsTicket`
   serialise (base32 via `to_string()`). Mettre a jour
   `MintRequest::new()` et les validations.

2. **shell-daemon/invite_api.rs** : dans `create_invite()`, generer le
   `DocsTicket` depuis `state.project_doc` (via
   `iroh_docs::DocTicket::new()` ou equivalent). Passer le ticket
   serialise a `MintRequest::new()`.

3. **worker-core/invite.rs** : parser `tasks_doc_ticket` depuis le
   payload invite. Appeler `iroh_docs::DocTicket::from_str()` pour
   deserialiser. Stocker le ticket pour utilisation par le scan task.

4. **Suppression tests legacy** : si des tests simulent l'ancien
   format invite sans `tasks_doc_ticket`, les supprimer ou les
   adapter (pre-launch policy).

### §C.2 Fichiers touches

| Fichier | Role |
|---|---|
| `crates/nexus-coordinator-rs/src/invite.rs` | MintRequest + InviteRecord + tasks_doc_ticket |
| `crates/nexus-shell-daemon/src/invite_api.rs` | DocsTicket generation dans create_invite |
| `crates/nexus-worker-core/src/invite.rs` | tasks_doc_ticket parsing + DocsTicket |

### §C.3 Tests plan

1. `test_mint_request_with_tasks_doc_ticket` : serialisation/deserialisation
   roundtrip avec le champ ticket
2. `test_invite_record_canonical_with_ticket` : le canonical bytes
   JCS inclut le champ tasks_doc_ticket
3. `test_create_invite_exports_docs_ticket` : le handler HTTP produit
   un invite avec un ticket non-vide
4. `test_worker_invite_parse_ticket` : le worker extrait et valide
   le DocsTicket depuis un invite payload

### §C.4 Critere d'acceptation

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked  # >= 1206 + delta tests
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release
# Verification specifique :
grep -n "tasks_doc_ticket" crates/nexus-coordinator-rs/src/invite.rs
grep -n "tasks_doc_ticket" crates/nexus-shell-daemon/src/invite_api.rs
grep -n "tasks_doc_ticket" crates/nexus-worker-core/src/invite.rs
```

### §C.5 Commit cible

```
feat(sprint54): Sprint 54 Phase C — E2E wire tasks_doc_ticket in invite format

Cable le chainon manquant du chemin task→worker→result : le
tasks_doc_ticket est maintenant inclus dans MintRequest et
InviteRecord. Le coordinateur exporte le DocsTicket de son
project_doc lors de la creation d'invite. Le worker parse le
ticket pour se synchroniser sur le document de taches.

Changements :
- coordinator-rs/invite.rs : +tasks_doc_ticket dans MintRequest
  et InviteRecord, validation, canonical bytes JCS
- invite_api.rs : DocsTicket export depuis project_doc
- worker-core/invite.rs : tasks_doc_ticket parsing + DocsTicket

Pre-launch policy : wire format v1 redefini (pas de bump).

Delta tests cumule Sprint 54 : Rust +N / Vitest +0

Scope cuts respectes : 12/12

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

---

## §Phase D — CI infra (4 items 2/3)

**But** : consolider l'infra CI. Prevention escalation 4 items 2/3
vers 3/3 MANDATORY S55.

**Dependencies** : Phase A (edition 2024 compile sur CI).

### §D.1 Scope

1. **Woodpecker agent VPS** : installer l'agent Woodpecker CI sur
   sbfb-eu (`135.181.42.188`). Configurer la connexion au serveur
   Woodpecker. Valider que le pipeline `.woodpecker/ci-linux.yml`
   tourne sur l'agent.
   CLOSE P2-REVIEW-B-1-S52 (2/3).

2. **GHA 9/9 re-run** : declencher un run GHA depuis la branche
   master post-Phase A (edition 2024). Confirmer que les 9 jobs
   (3 OS × 3 targets) passent. Documenter le run ID et les resultats.
   CLOSE P2-REVIEW-B-2-S52 (2/3).

3. **CI images pin** : pinner les images Docker dans
   `.woodpecker/ci-linux.yml` avec des SHA256 digests au lieu de
   tags flottants (`:latest`, `:22.04`).
   CLOSE P2-AUDIT-1-S52 (2/3).

4. **nextest timeout profiling** : investiguer pourquoi certains tests
   sont lents sur Windows. Profiler avec `--profile ci` et documenter
   les resultats. Si root cause trouvee, fixer. Sinon, documenter
   l'investigation pour S55.
   CLOSE P2-REVIEW-A-1-S52 (2/3).

### §D.2 Fichiers touches

| Fichier | Role |
|---|---|
| `.woodpecker/ci-linux.yml` | images pin SHA256 |
| `docs/architecture/SELF_HOSTED_BUILD.md` | Woodpecker status update |

### §D.3 Tests plan

Pas de nouveau test code — validation infra. Les 9 jobs GHA et le
pipeline Woodpecker sont les tests eux-memes.

### §D.4 Critere d'acceptation

```bash
# Woodpecker pipeline executed sur VPS agent
# GHA run 9/9 jobs pass (documenter run ID)
# Images pinnees dans .woodpecker/ci-linux.yml
grep -n "sha256\|@sha256" .woodpecker/ci-linux.yml
# nextest profiling documente
```

### §D.5 Commit cible

```
feat(sprint54): Sprint 54 Phase D — CI infra Woodpecker agent + GHA validation + images pin

Consolidation infra CI pour prevenir l'escalation de 4 items
a 3/3 MANDATORY S55.

- Woodpecker agent deploye sur VPS sbfb-eu (135.181.42.188)
  CLOSE P2-REVIEW-B-1-S52.
- GHA run ID XXXXX : 9/9 jobs pass (edition 2024 valide CI)
  CLOSE P2-REVIEW-B-2-S52.
- .woodpecker/ci-linux.yml : images pinnees SHA256
  CLOSE P2-AUDIT-1-S52.
- nextest timeout profiling : [resultats]
  CLOSE P2-REVIEW-A-1-S52.

Delta tests cumule Sprint 54 : Rust +N / Vitest +0

Scope cuts respectes : 12/12

Co-Authored-By: Claude Opus 4.6 (1M context) <noreply@anthropic.com>
```

---

## §Phase E — Wrap-up + verification + audit plan S55

**But** : cloturer Sprint 54.

### §E.1 Scope

1. CLAUDE.md : update "Sprints 0-54 CLOSED", carries S55, compteurs
2. HARDENING_ROADMAP : update last_validated S54
3. SPRINT_LOG.md : row S54
4. verification.md : 24+ fail-fast rows
5. sprint55_audit_plan.md : 7+ tracks audit

### §E.2 Commit cible

```
chore(sprint54): Phase E — wrap-up + verification + audit plan S55
```

---

## §9 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1206, 0 fail | |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | |
| 6 | npm lint | `npm run lint` (web/) | 0 error | |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error | |
| 8 | Vitest | `npm run test:unit` (web/) | >= 250 | |
| 9 | build | `npm run build` (web/) | ok | |
| 10 | size-limit | `npm run size` (web/) | 6/6 | |
| 11 | edition | `grep 'edition' Cargo.toml` | "2024" | |
| 12 | Phase A preflight G8 | verdict | EXECUTE | |
| 13 | Phase A review | verdict | PASS | |
| 14 | Phase B preflight G8 | verdict | EXECUTE | |
| 15 | Phase B review | verdict | PASS | |
| 16 | Phase C preflight G8 | verdict | EXECUTE | |
| 17 | Phase C review | verdict | PASS | |
| 18 | Phase D preflight G8 | verdict | EXECUTE | |
| 19 | Phase D review | verdict | PASS | |
| 20 | tasks_doc_ticket | `grep tasks_doc_ticket crates/nexus-coordinator-rs/src/invite.rs` | present | |
| 21 | node_key perms | `grep set_permissions crates/nexus-shell-daemon/src/runtime.rs` | present | |
| 22 | GossipTaskConfig | `grep GossipTaskConfig crates/nexus-shell-daemon/src/runtime.rs` | present | |
| 23 | periodic republish | `grep -E 'Duration::from_secs\|republish' crates/nexus-shell-daemon/src/runtime.rs` | present | |
| 24 | Woodpecker pipeline | pipeline run sur VPS agent | ok | |
| 25 | GHA 9/9 | run ID documente | ok | |
| 26 | CI images pin | `grep sha256 .woodpecker/ci-linux.yml` | present | |
| 27 | Scope cuts | 12/12 respectes | all checked | |
| 28 | Delta tests | cumule documente | documented | |

---

## §10 Git plan

```
1. chore(planning): sprint 54 kickoff + plan + design review G1 + migration S53 archive
2. feat(sprint54): Sprint 54 Phase A — Rust edition 2024 upgrade + unsafe set_var wrapping
3. chore(planning): sprint 54 Phase A review file
4. feat(sprint54): Sprint 54 Phase B — dette pair quick P2 batch
5. chore(planning): sprint 54 Phase B review file
6. feat(sprint54): Sprint 54 Phase C — E2E wire tasks_doc_ticket in invite format
7. chore(planning): sprint 54 Phase C review file
8. feat(sprint54): Sprint 54 Phase D — CI infra Woodpecker agent + GHA validation + images pin
9. chore(planning): sprint 54 Phase D review file
10. chore(sprint54): Phase E — wrap-up + verification + audit plan S55
```

---

## §11 Scope cuts

(Copie §7 kickoff — reference locale pour l'agent executeur)

1. LT-7 self-hosted build foundation — S55
2. Test E2E multi-noeuds automatise — S55
3. Outbox persistant fichier — S55
4. Browse_request rate-limit per-peer — S55
5. VPS TLS + nginx — S55
6. VPS monitoring + alerting — S55+
7. systemd service VPS — S55
8. LT-1 Kudos-v2 fairness reform — sprint dedie (S56+)
9. Events SSE daemon-native — post-v1.0
10. MCP server Rust — post-v1.0
11. Pagination SQL-side LIMIT/OFFSET — S55+
12. Test infra mk_state() refactoring — S55+

---

## §12 Risks

(Copie §9 kickoff)

| # | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Edition 2024 breakage non-anticipe | Low | High | `cargo fix` + 1206 tests |
| R2 | E2E wire > prevu | Medium | Medium | Scope cut > wire S55 |
| R3 | Woodpecker agent incompatible | Low | Low | Docker fallback |
| R4 | GHA flaky | Medium | Low | Re-run 2-3x |
| R5 | nextest timeout sans root cause | Medium | Low | Doc investigation, carry |
