# Sprint 82 Phase E — Réconciliation des ledgers de dette (D9)

Date : 2026-07-14. HEAD à l'entrée : `c7b6790` (Phases A-D + T2 DONE).
Méthode : re-audit LIVE **par item** (jamais un swap de compteurs) via
Workflow ultracode (4 re-audits parallèles + 2 vérificateurs adversariaux +
synthèse — **0 statut réfuté**, 6 corrections cosmétiques appliquées),
consolidé et vérifié main-thread. Chaque CLOSED cite le commit/état de code
qui ferme (`git merge-base --is-ancestor` vérifié) ; chaque ROUTED cite la
phase S82 du plan qui couvre l'item ; OPEN = dette réelle non fermée.

Vocabulaire des statuts : `CLOSED` (fermé, preuve commit/code) · `ROUTED`
(couvert nommément par une phase S82 A-T non encore jouée) · `OPEN` (dette
réelle, ancre grep-résolvable) · `ACCEPT-DOCUMENTED` (décision fermante
accept-and-document actée) · `STALE` (constat périmé/mixte, requalifié) ·
`SUPERSEDED` (dépassé par une décision/architecture ultérieure).

---

## 1. Correction du décompte — TROIS ledgers, pas un compteur

Le « **8 P2 / 11 P3 docs-contract S80** » propagé
(`sprint81_audit_findings.md:303`, `sprint82_audit_plan.md:172`) est un
**mislabel** : 8 P2/11 P3 = tally de l'**audit S79**
(`sprint79_audit_findings.md:14`, joué en Phase 0 de S80 — d'où la
confusion). L'audit **S80** compte **4 P2 / 10 P3**
(`sprint80_audit_findings.md`, re-compté ligne à ligne indépendamment par
le re-audit — levant la réserve honnête du préflight qui notait ce
re-comptage comme restant à faire). Un swap aveugle `8/11 → 4/10` aurait orpheliné les items S79
encore ouverts (ex. S79-P2-1 → Phase F). D'où le re-audit par item ci-dessous
sur les TROIS ledgers : S79 (20 items), S80 (14 items), S81 doc-dette.

## 2. Ledger audit S79 (1 P1 · 8 P2 · 11 P3 = 20 items)

| Item | Sév. | Statut LIVE | Evidence / route |
|---|---|---|---|
| P1-1 gate CSP redeploy | P1 | CLOSED | `c0a2ffe` — `atelier.rs:95` gate avant zip+upload + test `redeploy_blocks_on_csp_violation` |
| P2-1 promesses task_response.rs + PROMISE_RE | P2 | ROUTED | Phase F (fusion S80-G-2, compteur 2 reports — toucher fermant) |
| P2-2 classification Vendored par suffixe | P2 | OPEN | `gates.rs:497-498/:588` ; mitigé par la CSP navigateur (défense en profondeur) ; non routé S82 |
| P2-3 scanner CSP statique évadable | P2 | ACCEPT-DOCUMENTED | limite inhérente disclosée `THREAT_MODEL.md:770` §13.1 |
| P2-4 source-ref check = substring | P2 | OPEN | `check-factory-docs.sh:180` grep -qF ; limite de profondeur documentée ; non routé |
| P2-5 footgun 16-hex digest | P2 | ACCEPT-DOCUMENTED | documenté in-code + PATTERNS ~3900 |
| P2-6 test runnable ancre le contrat pas le gate | P2 | ACCEPT-DOCUMENTED | contrainte inhérente (crate binaire-pur) |
| P2-7 codex_review S79-B committé STALE | P2 | STALE | incident de discipline historique — l'artefact archivé (`archive/v2.1/sprint79_phase_b_codex_review.md`, commit `b27079c`) est immuable, non-actionable rétroactivement ; la classe est couverte par le hook lightcheck Check 7 actuel |
| P2-8 6 PLAN-ADAPT consécutifs | P2 | ACCEPT-DOCUMENTED | signal méta adressé par le process 5-scans S80+ |
| P3-1 baseline nextest non re-jouée | P3 | CLOSED | findings l.119 re-run 1994/1994 |
| P3-2 §P70 « F to come » stale | P3 | OPEN | `rust/PATTERNS.md:3878` — hors scope nommé Phase E (tickets T*), candidat balayage Phase H |
| P3-3 drift familles DOMAIN_*_V1 | P3 | ROUTED | Phase G (métrique figée + grep déterministe) |
| P3-4 verification.md omet 2 carries | P3 | STALE | les 2 carries tracés `sprint80_audit_plan §3` (moot) |
| P3-5 « line-semantic » sur-vendu | P3 | STALE | caveat déjà posé `sprint80_audit_plan §3` |
| P3-6 T2 app-authoring JSON gitignored | P3 | OPEN | `.gitignore:157-159` ; politique T2-JSON s'applique aux features cross-machine |
| P3-7 HARDENING_ROADMAP sans app-authoring | P3 | CLOSED | couverture satisfaite sur doc vivant `THREAT_MODEL.md:749` §13.1 |
| P3-8 volet french-body quasi-vacant | P3 | OPEN | ancre `check-factory-docs.sh` volet (3) `EN_WORDS` ; limitation « narrow subset » documentée |
| P3-9 honesty-gate présence-seule | P3 | OPEN | ancre `check-factory-docs.sh` `require_marker` (grep -qF) ; limitation documentée |
| P3-10 PACK_DIR figé animejs | P3 | OPEN | ancre `check-factory-docs.sh:244` `PACK_DIR` ; limitation single-pack documentée |
| P3-11 disclosure volet4 partielle | P3 | ACCEPT-DOCUMENTED | recoupe le footgun 16-hex P2-5 (lui-même ACCEPT-DOCUMENTED) ; l'artefact archivé est immuable |

**Tally S79 : 3 CLOSED · 2 ROUTED (P2-1→F, P3-3→G) · 3 STALE (P3-4,
P3-5, P2-7) · 5 ACCEPT-DOCUMENTED (P2-3, P2-5, P2-6, P2-8, P3-11) ·
7 OPEN (P2-2, P2-4, P3-2, P3-6, P3-8, P3-9, P3-10).** Les 7 OPEN se
décomposent honnêtement : 2 limites de gate sécurité (P2-2/P2-4,
mitigées par la frontière CSP navigateur inchangée) + 5 dettes
doc/lint/artefact mineures, chacune à ancre grep-résolvable ; aucun
n'accumule de compteur §6.2.1.

## 3. Ledger audit S80 (4 P2 · 10 P3 = 14 items) — recompte CONFIRMÉ

| Item | Sév. | Statut LIVE | Evidence |
|---|---|---|---|
| S80-E-1 inflation delta E2E | P2 | ACCEPT-DOCUMENTED | body immuable ; exit vigilance « 0 récurrence S81 » MET |
| S80-F-1 3 headers Verdict | P2 | CLOSED | `dcc3eea` (in-gate Phase 0 S81) |
| S80-H-1 routes diff/gates absentes docs | P2 | CLOSED | `50f05c1` (S81 Phase G) `LOOPBACK:117-118` |
| S80-H-2 asymétrie cookie 2 docs | P2 | CLOSED | `50f05c1` revalidation |
| S80-A-1 sous-rapport budgets operator | P3 | CLOSED | `dcc3eea` |
| S80-A-2 étiquette toolchain Win 1.94/1.95 | P3 | ACCEPT-DOCUMENTED | statu quo Docker-canonique acté S81 G préflight |
| S80-E-2 baseline E2E body Phase D | P3 | ACCEPT-DOCUMENTED | même classe que E-1, exit MET |
| S80-E-3 jalon « E 92 » faux | P3 | CLOSED | `dcc3eea` |
| S80-G-1 doc-lint 3 reports | P3 | ACCEPT-DOCUMENTED | **accept-and-close DÉJÀ acté au kickoff S81** (`sprint81_kickoff.md:95-96`, archive v2.1 : « DOC-LINT-SEMANTIC (S80-G-1) → ACCEPT-AND-CLOSE acté ») — cf. §6 compteurs |
| S80-G-2 S79-P2-1 bucket-route | P3 | ROUTED | Phase F (fusion S79-P2-1) |
| S80-H-3 terminal/ws « lecture cast » | P3 | CLOSED | `50f05c1` |
| S80-H-4 « SSE (EventSource) » | P3 | CLOSED | `50f05c1` fetch+ReadableStream |
| S80-I-1 trailer Co-Authored-By | P3 | ACCEPT-DOCUMENTED | décision S81 kickoff (cosmétique, pas d'enforcement) |
| S80-J-1 canon Track J web/e2e en dur | P3 | CLOSED | `dcc3eea` glob généralisé |

**Tally S80 : 8 CLOSED · 5 ACCEPT-DOCUMENTED · 1 ROUTED (G-2→F) · 0 OPEN.**
Constat central : **13/14 items S80 étaient déjà résolus AVANT S82** (4
CLOSED in-gate `dcc3eea`, 4 CLOSED par S81 Phase G `50f05c1`, 5
ACCEPT-DOCUMENTED par décisions S81) — le routage
`kickoff:357` « 4 P2/10 P3 S80 → Phases E-J » était largement stale. Seul
S80-G-2 est genuinement routé (Phase F). Disambiguation load-bearing :
S80-H-1/2/3/4 (CLOSED S81 G) ≠ S81-H-1/2/3 (findings NEUFS, routés Phase I).
(Le P1 S80-K-1, hors-périmètre docs-contrat de ce tableau, est CLOSED
`2c85b28` — symétrie avec le P1-1 listé au §2.)

## 4. Ledger S81 (34 findings audit + carries) — re-audit par item

| Item | Sév. | Statut LIVE | Evidence / route |
|---|---|---|---|
| S81-A3-1 E2E web rouge 3/45 | P1 | CLOSED | `ad53940` (split 2 projets Playwright, re-run GREEN 44/2skip) |
| S81-A3-2 GHA rouge GTK + claim CI | P1 | CLOSED | claim : `ad53940` ; infra : Phase C `2931b82` (GTK câblé 3 surfaces) |
| S81-G-ESC-1 boot-SEED OVERDUE 3/3 | P1 | CLOSED | Phase A `19b92e6` + T2 live `34550c1` (`sprint82_t2_bootseed.json` PASS 18 s) |
| S81-J-1 gate T1 non consigné | P1 | CLOSED | `ad53940` (section Acceptance machine-lisible) |
| S81-A3-3 compte E2E stale | P2 | CLOSED | `ad53940` (44/2skip CLAUDE.md) |
| S81-C-1 / C-2 doc-dette patterns Track C | P2 | ROUTED | Phase H (prose exacte à ré-extraire des reviews S81 — jamais fabriquée) |
| S81-C-3 pointeur rust-T20 faux | P2 | ROUTED | Phase H (re-ancrage ; le carry sécurité T20 lui-même reste OPEN, cf. §5.2) |
| S81-F-1 / F-2 / F-3 hygiène fichiers-review | P2 | ROUTED | Phase J (ré-extraction Track F) |
| S81-G-1 migration stores worker redb2→4 | P2 | ROUTED | Phase T (D12, artefact `sprint82_t2_store_migration.json`) |
| S81-H-1 / H-2 doc-dette hardening Track H | P2 | ROUTED | Phase I (ré-extraction ; ≠ S80-H-* CLOSED S81 G) |
| S81-I-1 ligne G8 Phase J inexacte | P2 | CLOSED | `ad53940` (verdict corrigé au plan) |
| S81-I-2 méta-process Track I | P2 | ROUTED | Phase J |
| S81-J-2 integration-nightly NEVER-RAN | P2 | ROUTED (PARTIAL) | calibré Phase C `2931b82` (K-R-13/K-R-14) ; run réel = gate push Phase T — seul PARTIAL du ledger |
| S81-J-3 / J-4 consignation testabilité + vocab T2 | P2 | ROUTED | Phase J (ratification README §4 ACTED/MIXED/NOT-RUN) |
| S81-K-1 drift prose ShardSessionView | P2 | CLOSED | `ad53940` (6 sites, gates docs verts) |
| S81-A-1 deny.toml 6 advisory-ignores | P3 | ROUTED | Phase K (4 hickory `deny.toml:83-86`) ; 2 quick-xml = veille upstream iroh à chaque bump ; quinn/ed25519-dalek rc.0 = `[bans]` multiple-versions (veille, pas advisory) |
| S81-A3-4 hygiène frontend/E2E part 3 | P3 | ROUTED | Phase J (consignation) OU standing — statué à la ré-extraction |
| S81-A3-5 coverage web vert solo | P3 | ROUTED | standing env-variance vitest (memory) ; consignation Phase J si retenu |
| S81-C-4 / C-5 patterns Track C P3 | P3 | ROUTED | Phase H |
| S81-D-1 libellé body Phase I | P3 | ROUTED | Phase J (nit méta) |
| S81-F-4 / F-5 fichiers-review P3 | P3 | ROUTED | Phase J |
| **S81-G-2** carry Track G (veille/supply-chain) | P3 | ROUTED | Phase I OU standing selon nature ré-extraite — c'est le « G-2 » de la carry-line `sprint81_audit_findings.md:303-309` (à ne pas confondre avec S80-G-2→Phase F ni G-D5-1→Phase I) |
| S81-G-3 catalog_len=0 seeder | P3 | ACCEPT-DOCUMENTED | PO-8 ; consignation THREAT/PATTERNS Phase I — décision fermante §6.2.1, sort des carries |
| S81-H-3 hardening Track H P3 | P3 | ROUTED | Phase I |
| S81-I-3 méta-process P3 | P3 | ROUTED | Phase J |
| S81-J-5 testabilité P3 | P3 | ROUTED | Phase J |
| S81-K-2 prose résiduelle Track K | P3 | ROUTED | Phase I |
| G-D5-1 VALIDATED_BLUEPRINT iroh 0.97 stale | P3 | ROUTED | Phase I (`:156` pin + `:157` prose gossip → =1.0.1 / 0.101) |
| K-R-7 qualificatifs sur-larges | P3 | ROUTED | Phase I |
| CARGO-AUDIT-CLAIM-HONESTY | P3 | ROUTED | Phase I (trancher câbler vs subsomption cargo-deny) |
| PIP-AUDIT-JOB-INOPERANT *(ajout daté S82 Phase I, découverte Codex round 1)* | P2 | ROUTED | Slot CI/hardening : le job `pip-audit` de `supply-chain.yml` cible 3 packages Python purgés S50-S51 (`uv export` exit 2) — réparer ou supprimer le job ; les docs sécurité le qualifient INOPÉRANTE depuis Phase I |
| Carries : K-R-13 slow-timeout / K-R-14 save-if | P2 | CLOSED | Phase C `2931b82` (nextest override 180 s ; save-if mainline-only) |
| Carry : RELAY-GATED-MULTI-DAEMON 4/10 | P1 | CLOSED | Phase D `c7b6790` (6 réparations / 0 requalif, 10/10) |
| Carry : BENCHMARKS-STANDARDS (PO-2) | P2 | CLOSED | Phase B `1670251` (harness + T3 canon + artefact BLOCK{rig}) |
| Carry : SCHEMAS-SHARD-REQ | P2 | ROUTED | Phase G (schémas) + Phase T (indexation) |
| Carry : HICKORY-024-RUSTSEC | P2 | ROUTED | Phase K (bump 0.24→0.26, 4 ignores retirés, 4 RUSTSEC clos) |
| Carries S79-P2-1 / S80-G-2 / S80-G-1 | P2/P3 | cf. §2, §3, §6 | Phase F (toucher fermant) / Phase F / déjà-accept-closed (Phase G = formalisation) |

La **prose exacte** des P2/P3 doc-dette (Tracks C/F/H/I/J/K) doit être
**ré-extraite des phase-reviews S81 archivées** en Phases H/I/J —
interdiction de la fabriquer.

## 5. Ledger PATTERNS (60 en-têtes `### T` : 52 shell + 8 rust) — re-audit complet

Le « ~80 tickets » du plan était un sur-compte (~+33 %) : **60 en-têtes**
réels ; le reste = puces mnémoniques (`PAT-1`/`CARRY-1`/`HARD-2`), variantes
`T-NN`×4 et prose historique — schémas d'ID distincts, non forcés en `T{N}`.

### 5.1 Shell (52 en-têtes → 50 numéros + 2 collisions)

- **CLOSED antérieurs (24)** : T2-T5, T8-T12, T28-T40, T42, T43 — inchangés.
  **SUPERSEDED antérieur (1)** : T41.
- **PURGÉS ce commit (15 zombies, ancre morte ère Python/pré-pivot)** :
  T15a, T16a, T17, T18, T20 (asyncio SSE — ≠ rust-T20), T21
  (`useAppEvents.ts` absent), T22, T23 (`nexus/` : `git ls-files` = 0),
  T44, T45, T46, T47, T48, T50, T51 (`packages/**.py` : `git ls-files
  packages/` = 0, purge S51 Phase A `49782a9`). Mécanisme canonique README
  §6.2.1 : blocs retirés du ledger, **tombstones dans `docs/DEPRECATED.md`**
  avec rationale + SHA — jamais un `git rm` muet. Concerns marqués « N/A —
  Python path removed S50-S51 » (honnêteté migration-vs-résolution : T16a a
  migré en Rust `blob_serve.rs:215` magic-bytes).
- **Collision T15/T16 RÉSOLUE** par suppression de T15a/T16a (eux-mêmes
  zombies) → T15b (verify.sh) et T16b (compute-shard E2E) restent uniques.
  Aucune référence cross-body par numéro (vérifié) — 0 orphelin. Le trou T19
  et les slots purgés ne seront JAMAIS réutilisés (grep-history).
- **CLOSED ce commit (1)** : **T15b** — fermé par le fix `scripts/verify.sh`
  (steps Python 4-8 retirés, même commit).
- **Réconciliés ce commit (4)** : **T14** suffixé CLOSED Sprint 74 Phase G
  (travail fait S74 : coverage vert enforced + FileUploadBlock +11 tests —
  header jamais suffixé) ; **T49** re-ancré (corps parlait de v4 alors que
  `PROJECT_ANNOUNCEMENT_VERSION = 1` `publish.rs:24` ; reject réel
  `publish.rs:183`, pas `:131`) — reste OPEN, EXCLU de la purge (ancre Rust
  vivante) ; **T6** re-scopé (référence « Sprint 8 Phase C » pré-pivot moot ;
  reste un advisory sur le renderer React actuel) ; **T7** splitté (volet
  caplog Python mort N/A ; volet Playwright anchors vivant reste OPEN).
- **OPEN à ancre vivante (10)** : T1 (accept-and-document inline déjà acté),
  T6 (re-scopé), T7 (volet Playwright), T13 (advisory ANALYZE, chiffres
  S9-era re-datés), T24-T27 (deploy VPS, trigger-driven, hors-thème D10 —
  note : le trigger de T25 « as real VPS ops begin » a FIRÉ depuis S75,
  candidat re-évaluation au prochain slot deploy), T16b (E2E compute-shard
  absent du miroir Woodpecker — GHA restaurée Phase C `2931b82`, le miroir
  reste sans Playwright), T49 (re-ancré).

### 5.2 Rust (8 en-têtes, tous OPEN — 0 collision, 0 zombie Python)

- **T19** (test rollback unsubscribe absent — vérifié toujours absent),
  **T22** (bench PoW non archivé), **T23** (**re-ancré ce commit** :
  `Dockerfile:14` → `:19` ; toujours aucun `@sha256` → réellement OPEN),
  **T25** (FIPS), **T26** (Argon2id), **T27** (`--pin` argv) : OPEN,
  trigger-driven, hors-thème D10 — statués, non codés.
- **T20 (relay cert-pinning) — GARDE-FOU P1, préservé ID+OPEN.** Carry
  sécurité VIVANT (cross-réf `THREAT_MODEL:1169/:1738`,
  `HARDENING_ROADMAP:19`) ; ≠ shell-T20 (asyncio, zombie purgé — namespaces
  indépendants). **Décision fermante consignée (pas un report sec)** : son
  exemption external-blocker a LAPSÉ (iroh 1.0.1 expose
  `CaTlsConfig::custom_server_cert_verifier`) → le **defer est RÉAFFIRMÉ
  MOTIVÉ** : le câblage SBFB-side est un chantier sécurité délicat (le
  verifier custom REMPLACE WebPKI entièrement et gate aussi pkarr/DoH — un
  scoping raté casse la discovery, cf. status update S81 in-ledger), routé
  **slot hardening dédié** ; le re-ancrage du pointeur (S81-C-3) reste Phase
  H. Phase E docs-only ne câble pas de TLS.
- **T21** (pins bootstrap n0) : OPEN external — note ajoutée : croise la
  re-décision Topologie A/B due avant le 25/08 (PO-5) ; si zéro-n0 confirmé
  après l'EOL 2026-09-30, T21 devient moot (candidat SUPERSEDED à cette
  échéance).
- **Hors en-tête** (schéma d'ID distinct, statués tels quels) : T24 inline
  (`:1431` handoff env var, atténué) ; T-NN/T-NN+1 RESOLVED S21 ; T-NN+2
  (iframe PII, external-blocker upstream valide §P34) ; T-NN+3
  (canonical_bytes dup, dépendance séquentielle interne nexus-core-rs).

### 5.3 Scripts zombies purgés (extension D9 constatée au codage)

- `scripts/verify.sh` : steps 4-8 Python retirés (`uv run ruff`/`pytest
  packages/` — abort garanti au step 4 sur checkout frais sous `set -euo
  pipefail`), commentaire d'usage et hypothèses `.venv/`/éditable retirés,
  steps renumérotés 1-16. **Ferme T15b.** Décision d'implémentation : steps
  ruff droppés entièrement (le seul `.py` restant est un exemple
  d'app-archive, pas du code projet lint-é).
- `scripts/setup.sh` : **zombie Python intégral** (uv venv + maturin +
  `nexus-core-py`, ère retirée S50-S51) découvert via la référence
  verify.sh L15-17 — purgé (même classe exacte que les steps 4-8, tombstone
  `docs/DEPRECATED.md`).
- `.githooks/post-merge` : hook opt-in S9 rappelant `setup.sh` pour le
  wheel `nexus_core` (`crates/nexus-core-py/` supprimé) — purgé. Les hooks
  `.githooks/{pre-commit,commit-msg}` (agentctl lightcheck/auditor-gate)
  sont VIVANTS, non concernés.
- `CONTRIBUTING.md` : réconcilié (l'arborescence affichait encore
  `packages/ # Python workspace (uv)` + `setup.sh` + section standards
  Python ruff/pytest + « 18 steps » — remplacés par la réalité Rust + web +
  examples, verify.sh 16 steps, pointeur DEPRECATED.md).

## 6. Compteurs §6.2.1 (Règle 2) — état après re-audit

1. **boot-SEED S81-G-ESC-1 (3/3)** : escalade SATISFAITE-CLOSED — phase
   obligatoire jouée (Phase A `19b92e6` + T2 live `34550c1`,
   `sprint82_t2_bootseed.json` PASS delay 18 s < 30 s). Compteur soldé.
2. **doc-lint S80-G-1 (3)** : DÉJÀ accept-and-closed au kickoff S81
   (`sprint81_kickoff.md:95-96`, archive v2.1 — « ACCEPT-AND-CLOSE acté »,
   l'item sort des carries). Le re-listage kickoff
   S82:89-90 comme « compteur dur » + le re-routage plan Phase G sont une
   **contradiction de framing** : Phase G = formalisation documentaire
   redondante, PAS une escalade fraîche ni un 4e report. **À qualifier
   explicitement au préflight Phase G.**
3. **S79-P2-1 (2 reports)** : ROUTED Phase F = le toucher fermant (trancher
   réparé/accept/statué — jamais un 3e report sec).
4. **catalog_len=0 (design, depuis S75)** : décision fermante PO-8
   accept-and-document, consignation Phase I (S81-G-3) — sort des carries.
5. **Tickets PATTERNS : AUCUN n'accumule de compteur §6.2.1 dur.** Le
   cluster rust T19-T27 a été reclassifié « tech debt long-terme » dès S20
   (soupape external-blocker/>500 LOC) ; les deploy shell T24-T27 n'ont
   jamais été dans le carry cycle. Seule décision fermante due hors-registre
   était rust-T20 — consignée §5.2. Réserve honnête : aucun
   `carry_summary.md` n'existe ; la source des compteurs = kickoff S82:89-92.

## 7. Tickets hors-thème (D10) — statués, routés, NON codés

| Ticket | Sév. | Route |
|---|---|---|
| R-J-6 RunProofs per-worker + binding N0-N3 | feature-gap | slot rig-chaud S83+ |
| F2 KV-cache cross-step | known-limitation | slot rig-chaud S83+ (optimisation) |
| SI-12 TOCTOU load↔hash | P2 | slot rig-chaud S83+ ; disposition THREAT v17 §16 déjà écrite |
| SHARD-TRUST-RECALIB (umbrella kickoff, pas un ID findings) | bundle | slot rig-chaud S83+ |
| J1b-3 cap participants decode | P3 | dette audit hors-thème S83+ |
| D3-2 charset piece | P3 | dette audit hors-thème S83+ |
| D4-2 préfixe 16-hex dans Err churn | P3 | dette audit hors-thème S83+ |
| J-D5-1 assertion conn_type==direct | P2 | dette audit hors-thème / slot rig-chaud |

## 8. Staging workflow-engine + audit_plan §6 (PO-9)

- `.planning/research/sprint82_workflow_engine/verification_blueprint.md` :
  bannière **SUPERSEDED** ajoutée (PO-9 2026-07-11 : workflow-engine +
  Viewer décalés vers de futurs slots tracés ; supersede D6 RATIFIÉE). Le
  JSON brut compagnon n'est pas modifié (donnée immuable du run).
- `sprint82_audit_plan.md §6` : repointé vers le vrai kickoff (sprint dette
  docs-contrat + refacto), workflow-engine DÉCALÉ, D6 RATIFIÉE.
- `sprint82_audit_plan.md:172` + `sprint81_audit_findings.md:303` :
  mislabel « 8 P2/11 P3 S80 » corrigé avec note datée (cf. §1).

## 9. Critères machine (état à l'issue de la phase)

- `git ls-files packages/` = **0** (et `nexus/` = 0).
- Tout `### T{N}` OPEN pointe un fichier existant (T21 shell/T23 shell
  purgés étaient les dernières ancres mortes ; T49/T23-rust re-ancrés).
- **0 collision d'ID** (T15/T16 uniques post-suppression T15a/T16a).
- `verify.sh` : plus aucune référence `packages/`/`uv`/`.venv` ; `bash -n`
  OK ; abort fresh-checkout step 4 éliminé par construction.
- 3 gates docs (`check-sharding-docs.sh`, `check-frontier-contracts.sh`,
  `check-factory-docs.sh`) + `check-spdx.sh` : exit 0 — validés en local
  (GHA rouge = env GTK, hors-scope ; surface verte = Woodpecker/local,
  cf. Phase C).
