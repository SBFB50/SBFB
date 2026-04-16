# Sprint 19 — Audit plan pour Sprint 20 Phase 0

**Ecrit** : 2026-04-16 (Phase F wrap-up)
**Commit stack a auditer** : range `1a606a3..<wrap-up>` — 8 feat/chore direct
S19 + 2 chore(planning) + 2 chore(claude) tooling hors-sprint (G4 auditor
Write enforce + TOOLING.md doc). Phases livrees :

- Phase A `ab6985c` DHT quorum runtime wire (carry S18 C-1)
- Phase B `edfc51b` PoW Hashcash primitive + gossip subscribe integration
- Phase B follow-up `08f4e41` wire Cargo deps + canonical + lib + PATTERNS
- Phase C `540bb51` TLS cert pinning relays (SPKI hash validate)
- Phase D `f238d31` delayed upload queue (0-5min exponential jitter)
- Phase D wrap `2fd6c60` chore(planning) review artefact + workflow autonomy
- Phase E `2fd4d72` pkarr relay self-hosted docker image + ops doc
- Phase F `<wrap-up>` verification + audit plan + migrations + updates

Chore hors-sprint inclus dans le range chronologique :
- `fe0a8fd` chore(planning) guardrails G1..G7
- `4216436` chore(skill) nexus-phase-auditor enforce Write on review file G4
- `c609a03` chore(docs) TOOLING.md Write-obligatoire auditor G4

---

## Mode d'emploi pour la session fraiche

1. Lire dans l'ordre :
   - memory (`MEMORY.md`, `nexus_grid_pivot.md`, `sprint_audit_gate.md`,
     `feedback_approach.md`)
   - `git log --oneline 1a606a3..HEAD` (range Sprint 19 complet)
   - `.planning/archive/v1.2/sprint19_kickoff.md` (D1..D5 gelees, **NE PAS
     rebattre** : D2 PoW 2^18, D3 SPKI pin, D4 queue 0-5min, D5 pkarr
     docker-only)
   - `.planning/archive/v1.2/sprint19_plan.md`
   - `.planning/archive/v1.2/sprint19_verification.md`
   - **ce document**
   - Phase reviews `.planning/archive/v1.2/sprint19_phase_{A,B,C,D,E}_
     review.md` (lus **apres** formation opinion own-track)
2. **NE PAS lire** les phase reviews en entier avant d'avoir forme une
   opinion track par track. Ces docs captent la narration livreur +
   auditor inline — l'audit gate doit **challenger** pas confirmer.
3. Timebox suggere : **2-3h**.
4. Livrable : `.planning/active/sprint19_audit_findings.md` (meme layout
   que `sprint18_audit_findings.md` archive).
5. Commits fix eventuels (P0/P1) doivent atterrir avant le premier commit
   Sprint 20 Phase A. Format `fix(sprint19): <track>-P<n> — <short>`.

---

## Scope auditable

Sprint 19 livre **~3500+ LOC code + ~850 LOC tests + ~1100 LOC docs/config**
reparti sur 5 phases + 1 wrap-up :

| Track | Phase | Commit | Livrable principal |
|---|---|---|---|
| A | DHT quorum runtime wire | `ab6985c` | `PkarrQuorumResolver` + `PkarrRelayClient` wrap + wiring browse aggregator + curator runtime + 7 tests |
| B | PoW Hashcash gossip subscribe | `edfc51b` + `08f4e41` | `pow.rs` primitive + `relay_pow_policy.toml` loader + `subscribe_with_pow` wrap + 29 tests |
| C | TLS cert pinning relays | `540bb51` | `tls_pinning.rs` SPKI extract + `PinValidator` + fixture PEM + PATTERNS.md section + 9 tests |
| D | Delayed upload queue | `f238d31` + `2fd6c60` | `upload_queue.py` async queue + scheduler + integration `api/tasks.py` + 21 tests coord + 2 tests SDK helper |
| E | pkarr relay self-hosted | `2fd4d72` | `docker/pkarr-relay/Dockerfile` + `build-pkarr-image.yml` + `PKARR_RELAY_OPS.md` §1-§7 + smoke test CI + 12 tests integration |
| F | Wrap-up | `<this>` | verification + audit plan + migrations + updates CLAUDE/SPRINT_LOG/memory + flip S18 `[~]→[x]` |

Meta-track : **Radicle-v1.0 activation tracking** (P2-2 S18 re-carry S19
re-carry S20).

---

## Track A — DHT quorum runtime wire (carry S18 C-1)

**Question centrale** : la primitive `redundant_resolve` S18 est-elle
**reellement** cablee dans le browse aggregator + curator runtime (pas juste
un export public inutilise), le fallback 2/3 quorum est-il exerce par les
tests (pas juste la primitive unit-test), le degraded mode est-il observable
(log, metric, breaker) ?

**Methodes** :

1. Grep `redundant_resolve` + `PkarrQuorumResolver` dans `crates/nexus-shell-
   daemon-core/` + `crates/nexus-core-rs/`. Chaque call site doit passer
   par la primitive S18, pas un single-node lookup. Aucun `pkarr::Client::
   resolve` direct ne doit rester dans les paths browse/curator.
2. Lire `sprint19_phase_A_review.md` section Dimensions apres avoir forme
   opinion own-track.
3. Test wire : fail-inject 1 pkarr relay sur 3 au niveau mock `PkarrRelay
   Client` → browse aggregator continue retourner resultat (quorum 2/3
   satisfait) ? 2 relais fail → degraded mode flag ? 3 relais fail →
   aggregator retourne erreur + log WARN ?
4. Eclipse-by-DHT verification : le flip S18 verification `[~]→[x]` est-il
   **reellement** justifie ? La primitive refusait 1/3 minoritaire S18, est-
   ce le cas apres wire (le call path traverse bien le quorum, pas
   shortcutte) ?
5. Signature `PkarrRelayClient::new` : verifier coherence 2-arg post fix
   P3 #2 Phase A review (annotation doc uniquement) vs le code actuel.

**Trigger audit P0** : wire bypass (un call site browse/curator a shortcut
single-node non visible au skim code review).

**Trigger audit P1** : degraded mode silencieux (2 relais fail → pas de log
/ metric → ops aveugle a la degradation DHT).

---

## Track B — PoW Hashcash gossip subscribe

**Question centrale** : la primitive `pow.rs` est-elle **crypto-sound**
(pas d'attaque shortcut trouvable type timestamp cache hit), la difficulty
2^18 tient-elle (~100ms CPU moderne bench), l'integration `subscribe_with_
pow` est-elle **fail-closed** (publisher sans proof valide = reject
subscribe, pas soft-warn) ?

**Methodes** :

1. Lire `crates/nexus-core-rs/src/pow.rs` entier. Algorithme : solve =
   trouver nonce tel que `hash(msg || nonce) < target` ou `leading_zeros >=
   difficulty` ? Which hash (SHA256 / BLAKE3) ? Domain separation present
   (prefix bytes domain tag `DOMAIN_POW_V1` avant hash) ?
2. `cargo bench --bench pow` ou equivalent : reproduire bench Phase B.
   Difficulty 2^18 doit tourner < 500ms CPU moderne (si > 500ms → UX
   degrade trop, trigger ajust difficulty S21+). Documenter 1 run local.
3. Integration path : grep `subscribe_with_pow` call sites. Quel est le
   comportement si relay policy `relay_pow_policy.toml` omet un relai ?
   Default fail-open (soft-warn, accept sans PoW) ou fail-close (reject
   subscribe) ?
4. Test fail-path : un publisher qui envoie proof avec difficulty < policy
   exigee → subscribe reject cote listener ? Un publisher qui envoie
   proof pour un different topic (replay cross-topic) → reject ? Domain
   separation protect cross-topic replay ?
5. `relay_pow_policy.toml` schema : format stable (versionne ou flat), un
   fichier malformed → daemon panic ou degraded fallback ?

**Trigger audit P0** : shortcut crypto (proof reutilisable cross-topic, ou
timestamp cache permettant attack par rainbow table).

**Trigger audit P1** : fail-open par defaut sur policy omission (un
attaquant qui publie un `relay_pow_policy.toml` omit ses propres relais
contourne le gate).

---

## Track C — TLS cert pinning relays

**Question centrale** : le SPKI hash extract est-il conforme RFC 7469 HPKP
(SHA256 SubjectPublicKeyInfo encode DER), le `PinValidator` est-il
**fail-closed** sur pinset empty (refuse tout, au lieu de accept tout par
default), la rotation n0 est-elle documentee (quand n0 roll cert, quel est
le runbook user-side) ?

**Methodes** :

1. Lire `crates/nexus-core-rs/src/tls_pinning.rs` entier. SPKI extract :
   utilise `x509-parser` / `rustls-pki-types` / autre ? Hash SHA256 applique
   bien sur SPKI DER (pas cert DER complet) ?
2. `relay_test_cert.pem` fixture : gen local ou stable ? Si gen dynamique,
   les tests sont-ils deterministes (same SPKI hash run-to-run) ?
3. Pinset empty test : `PinValidator::new(vec![]).validate(cert)` →
   **refuse** (fail-closed) ou **accept all** (fail-open) ? Default fail-
   closed attendu.
4. Integration iroh : lire le call path `RelayClient::builder()` wrap. Si
   iroh 0.97 n'expose pas de TLS validator custom hook, quelle est la
   workaround (fork connect path ? TODO upstream PR ?) ? Grep `TODO(upstream)`.
5. Doc `PATTERNS.md §TLS cert pinning` : rotation procedure complete
   (obtain new SPKI hash via `openssl s_client` → edit `~/.sbfb/relay-
   pins.json` → restart daemon) ? Un ops sans context peut-il l'executer ?

**Trigger audit P0** : SPKI extract mis sur cert-complet au lieu de SPKI
subfield (empeche rotation cross-CA, impose rotation par cert complet).

**Trigger audit P1** : `PinValidator` fail-open sur pinset empty (casse la
propriete "pinning actif" si config absent).

---

## Track D — Delayed upload queue

**Question centrale** : le jitter exponential 0-5min est-il **reellement
anti-correlation** (distribution uniforme pas exponential biaise vers 0),
la queue survit-elle a un restart coord (persistence disque ou accept
data loss), le throughput ne reverte-t-il pas sur un burst (N tasks
simultanes ne cassent pas le flush 30s granularity) ?

**Methodes** :

1. Lire `packages/nexus-coordinator/src/nexus_coordinator/upload_queue.py`
   entier. Distribution jitter : `random.uniform(0, 300)` OK, mais "exponen
   tial" dans le commit title suggere `random.expovariate(1.0/150)` — quel
   est vraiment implemente ? Une exponential biaise vers 0 = moins
   anti-correlation (50% des tasks passent dans 100s).
2. Lire `packages/nexus-coordinator/tests/test_upload_queue*.py` (si
   existe). Le range 0-5min est-il exerce (distribution verification N=100
   samples) ?
3. Persistence : queue in-memory seule (accept data-loss on coord restart)
   ou serialisee disque (SQLite / JSON) ? Quel trade-off acte plan §D ?
4. Concurrent submit : N=10 submit() simultanes → ordre preserve ? Un
   scheduler single-thread 30s suffit pour flush ? Pas de thundering herd
   a chaque tick ?
5. Integration `api/tasks.py` : le endpoint `POST /project/task/submit`
   passe bien par queue (pas un direct gossip emit residuel). Grep
   `gossip.publish` dans `api/` — doit etre uniquement dans le scheduler
   flush, pas dans l'endpoint.

**Trigger audit P0** : bypass (un code path submit direct non-piped dans
queue = fuite correlation pas delayed).

**Trigger audit P1** : distribution biaise exponential vers 0 alors que
commit title / doc annonce uniform 0-5min (promesse anti-correlation
affaiblie).

---

## Track E — pkarr relay self-hosted image

**Question centrale** : le Dockerfile est-il **reproducible** (pin version
base + pin dependencies), le workflow `build-pkarr-image.yml` est-il
safe (scan Trivy effectif, permissions minimales, no secret leak), la doc
`PKARR_RELAY_OPS.md §1-§7` est-elle **executable** sans re-research
(provisioning Hetzner commands copy-paste + systemd unit + nginx + Let's
Encrypt + smoke test) ?

**Methodes** :

1. Lire `docker/pkarr-relay/Dockerfile` entier. `FROM rust:1.94-slim`
   ou `FROM rust:1.94-slim@sha256:...` (pin SHA recommande pour repro) ?
   USER non-root, tini present, healthcheck expose ?
2. Lire `.github/workflows/build-pkarr-image.yml`. `permissions: contents:
   read, packages: write` uniquement ? Scan Trivy `aquasecurity/trivy-
   action@<pin>` inline avec fail-closed sur CVE Critical ? Tag SHA pin.
3. Lire `docs/release/PKARR_RELAY_OPS.md §2 provisioning` : commandes
   copy-paste complete (apt update, install docker, pull image, systemd
   unit, nginx proxy, certbot Let's Encrypt, smoke test) ? Un SRE sans
   context peut-il l'executer en ~30 min ?
4. `PKARR_RELAY_OPS.md §5 smoke test` : publish + resolve en E2E
   reellement testable local avec `docker-compose up` + `pkarr-cli` ?
5. `PKARR_RELAY_OPS.md §7 rotation SPKI cert` : cross-ref Phase C TLS
   pinning existant ? Un user qui deploie un pkarr self-hosted sait-il
   comment faire connaitre son SPKI hash pour pin cote clients ?

**Trigger audit P0** : leak token `GHCR_TOKEN` dans logs workflow GHA
(scan logs apres premier run reel).

**Trigger audit P1** : `PKARR_RELAY_OPS.md §3 systemd unit` incomplete (un
user qui copy-paste ne boot pas le service = friction + reporting probable
vers le team SBFB).

---

## Meta-track — Radicle-v1.0 activation tracking (re-carry S18 → S19 → S20)

**Origine** : finding P2-2 du review E3 S18 (`sprint18_phase_E3_review.md`).
Re-carry S18 → S19 cette session. Re-carry **S19 → S20** maintenant.

**Question centrale** : l'activation Radicle au v1.0 go-live a-t-elle
toujours un landing spot concret (deadline, owner, runbook) qui resistera
a la cloture du sprint 19 + changements de session ?

**Item** :

```
[Radicle-v1.0 activation]
- Owner : maintainer (FlowUP)
- Deadline : jour du tag v1.0 (probablement sprint release v1.0)
- Blocker : tag v1.0 prerequisite — flip GitHub+Codeberg public d'abord
- Runbook : docs/release/MIRROR_FALLBACK.md §3 (8 sous-sections 3.1-3.8,
  self-contained)
- Resources :
  * VM Linux disponible (testee Phase E3 S18)
  * Action pinned : gsaslis/mirror-to-radicle@514707f3 (v0.2.0, avril 2026)
  * 5 secrets GHA a creer : RADICLE_IDENTITY_{ALIAS,PASSPHRASE,PRIVATE_KEY,
    PUBLIC_KEY}_KEY + RADICLE_REPOSITORY_ID
- Check post-activation :
  * Workflow mirror-radicle.yml run verte
  * rad clone rad:z<RID> depuis machine tierce recupere master tip
  * Radicle Explorer app.radicle.xyz montre le projet
  * CANARY.txt §mirror_urls extend avec 3 sources (GitHub / Codeberg /
    Radicle)
  * MIRROR_FALLBACK.md §1 status update → "v1.0 flipped, anti-subpoena
    public active"
```

**Trigger audit S20 Phase 0** : verifier que cet item existe dans le sprint
dans lequel v1.0 tag est prevu. Si v1.0 != S20, re-report ce meta-track au
sprint release. Pas de perte si Radicle jamais active, **mais** la promesse
"pattern pret en 15 min" du commit E3 S18 doit etre tenable — donc
periodiquement `rad` CLI + action doivent etre testees (stale rot
prevention).

**Fix si omis** : ajouter au kickoff S20 (ou sprint release v1.0) `§Items
carry/dette` cet item avec deadline + owner.

---

## Track F — Wrap-up coherence

**Question centrale** : les docs Phase F (CLAUDE.md `§Etat actuel`,
SPRINT_LOG.md row S19, memory `nexus_grid_pivot.md` frontmatter) sont-ils
coherents entre eux et avec le tip final, la migration PARA est-elle
complete (10 files `.planning/active/sprint19_*.md` → `archive/v1.2/`),
le flip S18 verification `[~]→[x]` est-il execute et correctement
annote ?

**Methodes** :

1. `git log --oneline 1a606a3..HEAD` : doit contenir au moins 5 feat S19
   directs (A/B/C/D/E) + 1 Phase B follow-up fix + 2 chore planning + 2
   chore tooling G4 + 1 wrap-up F. Total attendu : ~11 commits.
2. `CLAUDE.md §Etat actuel` : mentionne tip final + compteurs tests finals
   + mention "Eclipse-by-DHT defense pleinement active runtime" +
   "Sprint 19 CLOSED".
3. `SPRINT_LOG.md` : row S19 v1.2 avec Phase stack + theme + etat DONE +
   mention re-carry Meta-1.
4. Memory `nexus_grid_pivot.md` frontmatter description : tip sync +
   compteurs post-S19 + etat Sprint 20 OPEN ou READY (selon quand la
   session audit gate demarre).
5. `ls .planning/active/` : vide (tous les sprint19_*.md archives).
6. `ls .planning/archive/v1.2/sprint19_*.md` : **10 files attendus**
   (kickoff, plan, verification, audit_plan, supervision_log, 5 phase
   reviews A/B/C/D/E).
7. Flip S18 verification : grep `DHT redundant lookup` dans
   `archive/v1.2/sprint18_verification.md` → doit contenir `[x]` (pas
   `[~]`) + annotation `wire Phase A S19 ab6985c`.

---

## Criteres verdict

- **PASS** : 0 finding P0, 0 finding P1.
- **CONDITIONAL PASS** : 0 P0, P1 trouve MAIS fix commits livres avant
  `feat(sprint20): Phase A` = gate leve.
- **FAIL** : P0 non-resolu au demarrage S20 Phase A. Sprint 20 doit
  attendre fix.
- **Rigor signal G4** : verdict PASS exige >=1 P2+ documente (pas de 0
  finding systematique = signal auditeur insuffisant — re-auditer
  dimension manquee). Applicable au verdict final, pas aux verdicts par
  track.

Pattern permanent depuis Sprint 7.
