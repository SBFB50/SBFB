# Sprint 82 Phase E — Preflight (G8)

Date : 2026-07-14. Phase E réconcilie les ledgers de dette
`docs/{rust,shell}/PATTERNS.md`, purge les zombies de l'ère Python
(supprimée S50-S51), résout la double-numérotation `T15`/`T16` (shell),
nettoie les steps Python morts de `scripts/verify.sh`, reconcilie le
décompte docs-contrat, marque le staging `sprint82_workflow_engine/`
**SUPERSEDED** et corrige `sprint82_audit_plan §6` (PO-9). Périmètre
**100 % docs / planning / script** : 0 fichier `crates/`, **0 wire bump**,
**0 dep** touchée (S4 + S1b confirmés). Preflight ultracode = 5 scans
factuels (S1a/S1b/S2/S3/S4) + 4 inventaires exhaustifs + 2 passes
adversariales, **complété par une vérification file:line du synthétiseur**
(comptage des en-têtes `### T`, ancre T49, collision T15/T16, `git ls-files
packages/`, steps verify.sh, état staging, sources du décompte).

## Verdict: PLAN-ADAPT

Le plan est exécutable et adossé à des décisions historiques vérifiées
(aucun DESIGN-CONFLICT : PO-9 autorise le périmètre, CLAUDE.md
« sessions fraîches » + « pré-launch zombies » APPUIENT la purge, aucune
décision protectrice trouvée). Mais **quatre faits du plan sont inexacts
ou trompeurs** et imposent une exécution corrigée par item :

1. **« Purger zombies Python T44-T51 »** — FAUX pour **T49** : ancre RUST
   VIVANTE (`crates/nexus-shell-daemon-core/src/publish.rs`, présent), pas
   un `.py` mort. La plage = 8 tickets = **7 zombies Python** + 1 ticket
   Rust ouvert à re-ancrer. Le supprimer avec le bloc détruirait un ticket
   ouvert réel.
2. **« ~80 tickets T\* »** — SUR-COMPTE (~+33 %). Recompte du synthétiseur :
   **60 en-têtes `### T`** (52 shell + 8 rust). Le « ~80 » ne se reconcilie
   qu'en repliant de la dette **hors-en-tête-T** (prose historique, puces
   mnémoniques `PAT-1`/`CARRY-1`/`HARD-2`, `T-NN`×4, refs inline).
3. **« stale 8 P2/11 P3 S80 → réel 4 P2/10 P3 »** — cadrage TROMPEUR :
   ce sont **DEUX JEUX DE FINDINGS DISTINCTS** (audit **S79** = 8/11 vs
   audit **S80** = 4/10), pas un compteur unique corrigé de −4/−1. Un swap
   numérique à l'aveugle orphelinerait les items S79 encore ouverts.
4. **Label « ZOMBIE-supprimé »** — sans précédent canonique (verbes du
   ledger = CLOSED / SUPERSEDED). La suppression d'un carry passe par
   `docs/DEPRECATED.md` ou un tombstone SHA-ancré (README §6.2.1 + en-tête
   shell), jamais un `git rm` silencieux.

Aucune de ces adaptations ne re-débat une décision Day-0/PO → **PLAN-ADAPT**,
pas DESIGN-CONFLICT.

## Scans

### S1a — Prior-art / conventions internes des ledgers — `PLAN-ADAPT`, med
Le format de statut prévu (CLOSED / ZOMBIE-supprimé / OPEN-ancre) est
LARGEMENT compatible, **3 adaptations** à acter :
- **CLOSED** : compatible tel quel via le mécanisme canonique = **suffixe
  inline** sur le header (`— CLOSED Sprint 82 Phase E`) ou **callout
  blockquote** `> **Status update (S82 Phase E, 2026-07-14).** …` avec corps
  historique gardé verbatim. **Ne pas** inventer un champ `Status:` séparé
  (evidence : `docs/shell/PATTERNS.md:760` `— CLOSED`, `:1508` `SUPERSEDED` ;
  `docs/rust/PATTERNS.md:976-998` callout `Status update` verbatim).
- **OPEN-ancre** : NATIF à la convention — l'en-tête `docs/shell/PATTERNS.md:5-7`
  impose déjà de dater + référencer le SHA introducteur (`git show <sha>`) +
  `Audit reference:` / `Cross-ref:`. Aucune contrainte de format nouvelle.
  **Garde-fou README §6.2.1 Règle 2** : un OPEN à **≥3 reports** n'est PAS un
  statut légitime — escalade au plan ou suppression. Croiser chaque OPEN avec
  son compteur (dans `carry_summary`/kickoff, PAS dans PATTERNS) avant de le
  laisser OPEN.
- **ZOMBIE** : SEULE divergence — aucun label « ZOMBIE » n'existe. Purger via
  **tombstone** `— SUPERSEDED (ère Python retirée S50-S51)` **OU** entrée
  `docs/DEPRECATED.md avec rationale` (README:1464/1514), jamais un `git rm`
  muet qui casserait le contrat grep-history.

Collision T15/T16 = **shell-only** (rust n'en a aucune). Résoudre par
**append-sequential au prochain ID libre > max (T51 → T52/T53)**, PAS par
réutilisation de slots purgés ni du trou T19. **Deux schémas d'ID coexistent**
(headers `T{N}` vs puces mnémoniques `PAT-1`/`CARRY-1`/`HARD-2`) — le re-audit
ne doit pas forcer un `T{N}` sur les items mnémoniques.

### S1b — Supply-chain / deps — `EXECUTE`, low
**0 dep touchée, 0 pin changé, 0 lockfile.** Aucun livrable Phase E n'est un
manifeste (`Cargo.toml`/`package.json`/`*.lock`) ; `git ls-files
'packages/**/{Cargo.toml,package.json,*.lock}'` = 0. `git ls-files packages/`
= **0** (vérifié par le synthétiseur). `verify.sh` = script bash hors `crates/`.
Deux pièges d'implémentation (non-bloquants) : (a) cibler `scripts/verify.sh`,
PAS les copies `.claude/worktrees/*/scripts/verify.sh` ; (b) `packages/` existe
**untracked sur cette machine dev** → un run local peut MASQUER l'abort
fresh-checkout — valider le critère « sans abort » sur un checkout réellement
frais.

### S2 — Chaînes de décision historiques — `EXECUTE`, med
Les 4 axes vérifiés, aucun conflit doctrinal :
- **git log** : suppression Python ancrée **S51 Phase A (`49782a9`)** (247
  fichiers, −72k LOC ; `packages/` ajouté au `.gitignore`). Aucun commit
  ultérieur ne réintroduit `packages/`. `git ls-files packages/` = 0 (critère
  machine PASS d'entrée). Les zombies T44-T51 pointent tous des `.py`
  inexistants.
- **PO-9** (kickoff:46/122-123) autorise explicitement le périmètre docs/planning.
- **staging** `.planning/research/sprint82_workflow_engine/` = 2 fichiers,
  **PAS marqué SUPERSEDED** (grep=0, en-tête « STAGING hors sprint »).
- **0 décision protectrice** (`grep DEVIATION/rejected/WONTFIX` = batchs
  d'audit standards + anti-patterns design sans rapport). Purge APPUYÉE par
  CLAUDE.md.

RÉSERVE la plus importante (déjà prévue par le Goal du plan « re-audit COMPLET,
pas confiance au décompte ») : la reconcile du décompte n'est **pas** un swap
`8/11 → 4/10` — ce sont deux audits distincts, re-audit LIVE par item obligatoire.
RÉSERVE 2 : purge = **tickets T\* uniquement** ; la prose Python historique de
`rust/PATTERNS.md` (PyO3, §P40, « Post-S45 removed ») n'est pas numérotée T* —
ne pas la sur-purger.

### S3 — Couverture threat-model — `EXECUTE` avec **1 garde-fou P1**
La purge est **threat-SAFE** : les cibles (zombies Python T44-T48/T50/T51 +
T15a/T16a S9) portent 0 mitigation vivante et 0 cross-ref `docs/security/`
(grep mot-borné = 0 hit ; les tokens T0-T5 saturant `docs/security/` sont la
taxonomie adversaire d'`ADVERSARIES.md`, faux positifs). La préoccupation de
T16a (content_type client-contrôlé) a MIGRÉ en Rust (`blob_serve.rs:215`
`detect_content_type` magic-bytes) — rien de porteur n'est perdu.
**GARDE-FOU P1 — `rust-T20`** (relay cert-pinning / PinValidator) : carry de
sécurité **VIVANT**, cross-référencé par `THREAT_MODEL.md:1169`, `:1738` et
`HARDENING_ROADMAP.md:19`. HORS scope purge ; le re-audit et la résolution de
collision NE DOIVENT PAS le renuméroter ni le confondre avec **`shell-T20`**
(asyncio SSE, zombie Python, namespace indépendant). Recommandation : marquer
les zombies « concern N/A — Python path removed S50-S51 (successeur : <ancre
rust si applicable>) » plutôt que « CLOSED », pour rester honnête migration-vs-résolution.

### S4 — Invariants wire-format — `EXECUTE`, very low
Trivialement hors-wire. Les seules occurrences `_VERSION` dans
`crates/nexus-core-rs/src/` sont 2 doc-comments `//!` (hors scope). Phase E ne
touche AUCUNE struct wire, AUCUN canonical bytes JCS, AUCUN `*_VERSION` ; la
pré-launch protocol policy = N/A (elle encadre les éditions de wire formats).
`git status` clean, aucune modif `crates/` latente. **0 wire bump définitif.**

## Inventaires factuels

### A. `docs/rust/PATTERNS.md` — 8 en-têtes `### T`, 0 collision, 0 T44-T51
Headers T (TOUS OPEN) : **T19, T20, T21, T22, T23, T25, T26, T27** (`T24`
absent-en-tête, existe en réf inline `:1431` = dette ouverte handoff secret ;
`T3 :1390` = closed-ref). Variantes `T-NN`×4 (`:1972`/`:2002` RESOLVED S21 ;
`:2041` iframe PII OPEN ; `:2684` canonical_bytes dup OPEN). Puces mnémoniques
Sprint 77 : `PAT-1`/`CARRY-1`/`HARD-2` (ancres Rust vivantes). **Aucun T44-T51
côté rust.** Ancres à numéro de ligne périmé (grep-résolvables, pas mortes) :
**T23** (`Dockerfile:14` → FROM réel `:19`, aucun `@sha256` → réellement OPEN).
`T20` : hook `ca_tls_config` désormais utilisé mais en `insecure_skip_verify()`
(`node.rs:858`), pin non composé → OPEN (status-update S81 déjà inscrit
`:976-998`). `T19` : test `unsubscribe_persist_failure_rollback` toujours
absent → OPEN réel.

### B. `docs/shell/PATTERNS.md` — 52 en-têtes `### T`, collision T15/T16
**52 blocs `### T` = 50 numéros distincts (T1-T18, T20-T51) + 2 collisions
(T15×2, T16×2). T19 absent (trou).** Répartition : **24 CLOSED / 1 SUPERSEDED
(T41) / 27 OPEN**. Sur les 27 OPEN : **15 zombies à ancre morte** + **12 à
ancre vivante**.
- **Zombies à purger (15)** : T15a(`:1103` SVG BOM), T16a(`:1118` CAS
  content_type), T17(`:1135`), T18(`:1152`), T20(`:1169` asyncio SSE),
  T22(`:1200`) [Python `files.py`/`events.py`/`test_gov`] ; **T44, T45, T46,
  T47, T48, T50, T51** [Python `deploy.py`/`provenance.py`/`test_deploy.py`] ;
  T21(`:1185` `web/src/hooks/useAppEvents.ts` ABSENT) ; T23(`:1322` sujet
  `nexus/` moot, `git ls-files nexus/`=0).
- **OPEN à ancre vivante (à statuer OPEN, pas purger)** : T1, T6, T13, T14,
  T24, T25, T26, T27, **T49**, **T15b**(`:2339` `scripts/verify.sh`),
  **T16b**(`:2350` compute-shard E2E). **T7 MIXTE** (moitié Playwright vivant /
  moitié caplog Python mort) → statuer **stale**.

### T44-T51 (réponse spécifique — CORRECTION DU PLAN)
Bloc = 8 tickets. **7 zombies Python** (T44 `_dir_size`, T45 `_git_rev_parse`,
T46 `startswith("http")`, T47 `provenance.py json.dumps`, T48
`verify_provenance schema_version`, T50 `D4 clone`, T51 `_clone_repo`) — tous
ancre `packages/…/*.py`, `git ls-files packages/`=0 → **purge légitime**.
**T49 = EXCLU de la purge** : header `PA v4 bump breaks forward compat`, ancre
`crates/nexus-shell-daemon-core/src/publish.rs:131` (fichier PRÉSENT). Ticket
**doublement STALE** vérifié par le synthétiseur : (a) le corps parle de
`v > PROJECT_ANNOUNCEMENT_VERSION (v4)` alors que la constante vaut **1**
(`publish.rs:24`, politique pré-launch `*_ANNOUNCEMENT_VERSION=1`) ; (b) le
reject réel est `publish.rs:183` (`from_gossip_bytes`), pas `:131`. → **OPEN à
re-ancrer** (v4→v1, `:131`→`:183`), jamais supprimé avec le bloc.

### Collision T15/T16 (shell-only) — résolution
`T15` = `:1103` (SVG BOM, S9, zombie Python) **vs** `:2339` (verify.sh stale,
S77, VIVANT). `T16` = `:1118` (CAS content_type, S9, zombie Python) **vs**
`:2350` (compute-shard E2E GHA-only, S77, VIVANT). Cause : le bloc « Sprint 77
audit gate » a réutilisé T15/T16 sans consulter le max (T51). rust = 0 collision.
**Résolution préférable** (résout collision + dette en un geste) : **supprimer
T15a/T16a** (eux-mêmes zombies Python, déjà cibles de purge) → laisse T15b/T16b
uniques et vivants. **Fallback** (si on garde les 4) : renuméroter la paire S77
en **T52/T53** (> max T51). Vérifié : **aucune référence cross-body à T15/T16
par numéro hors les 4 en-têtes** → renuméroter/supprimer est sans orphelin.
**Ne JAMAIS** réutiliser le trou T19 ni un slot purgé T44-T51 (casserait le
grep-history des zombies supprimés).

### verify.sh — steps Python morts (L44-57)
Steps **4-8** : step 4 `uv run ruff format --check packages/ examples/`
(L44-45), step 5 `uv run ruff check packages/ examples/` (L47-48), step 6
`uv run pytest packages/nexus-sdk/tests/ -q` (L50-51), step 7
`packages/nexus-coordinator/tests/` (L53-54), step 8
`packages/nexus-app-gov/tests/` (L56-57). Sous `set -euo pipefail` (L19), sur
checkout frais `packages/` est absent → **abort au step 4**. Nettoyer AUSSI le
commentaire d'usage L12 (« Rust + Python + web ») + L15-17 (assume
`.venv/`/`nexus_core` editable). **Décision d'implémentation** : steps 4-5
ciblent aussi `examples/` qui garde **1 `.py` suivi**
(`examples/hello-world-app/src/hello_world_app/__init__.py`) → garder ruff sur
`examples/` seul OU dropper entièrement les steps 4-5 (non-bloquant, pas une
contrainte dep). Cross-link : le fix verify.sh **FERME le ticket vivant T15b**
(`:2339`) — le séquencer (fix verify.sh → marquer T15b CLOSED dans le même commit).

### Décompte docs-contrat — TROIS ledgers, pas un swap
- **« 8 P2/11 P3 »** = tally de l'**audit S79** (`sprint79_audit_findings.md:14`
  « 0 P0 · 1 P1 · 8 P2 · 11 P3 » ; headers `P2 (8)` items P2-1..P2-8 / `P3 (11)`
  P3-1..P3-11). Étiquette « docs-contract S80 » = **mislabel** (audit S79 =
  Phase 0 de S80). Propagé verbatim : `sprint81_audit_findings.md:303`,
  `sprint82_audit_plan.md:172`.
- **« 4 P2/10 P3 »** = **audit S80** (`sprint80_audit_findings.md` verdict `:15`
  CONDITIONAL PASS ; 4 P2 = S80-E-1/F-1/H-1/H-2 ; 10 P3 = A-1/A-2/E-2/E-3/G-1/
  G-2/H-3/H-4/I-1/J-1), corroboré `sprint81_plan.md:33`. **Chiffre EXACT**,
  non aspirationnel (réserve honnête d'un vérificateur : le 4/10 repose sur
  verdict + table de synthèse + corroboration, pas sur un re-comptage
  ligne-à-ligne indépendant).
- **PIÈGE** : swap aveugle `8/11 → 4/10` **orphelinerait** les items S79 encore
  ouverts (ex. **S79-P2-1** `task_response.rs`, re-routé **S82 Phase F**, 2
  reports). Résolution actée `sprint82_design_review.md:186-191` = **D9 / Phase E
  = re-audit COMPLET par item**. Reconcilier **TROIS ledgers** : (a) S79 8/11
  [statut LIVE par item], (b) S80 4/10 [statut LIVE par item], (c) doc-dette S81
  (`sprint82_kickoff.md:357` : C-1/2/3, H-1/2, K-1/2, J-3/4, I-1/2, G-2, F-*).

### Staging workflow-engine — état
`.planning/research/sprint82_workflow_engine/` = **2 fichiers**
(`verification_blueprint.md` + `raw_workflow_verification_wf_23e03df4.json`).
**PAS marqué SUPERSEDED** (grep `SUPERSEDED` = 0 sur les 2 fichiers ; en-tête
`:3` = « STAGING hors sprint », `:71` D6 « SUPERSEDE doctrinal à ratifier PAR
LE PO »). PO-9 (2026-07-11) a DÉCALÉ workflow-engine+Viewer (futurs slots) et
ratifié la supersede D6 → Phase E ajoute la bannière SUPERSEDED.

### audit_plan §6 (L198-203) — mis-scope
Actuel : « L'audit gate S81 est la Phase 0 de S82 — il BLOQUE toute Phase A de
S82 (kickoff staging : `.planning/research/sprint82_workflow_engine/` +
verification_blueprint, décisions D6-supersede à ratifier PO) ». STALE : présente
le workflow-engine comme kickoff bloquant de S82 avec D6-supersede PENDANTE, or
S82 = **sprint dette docs-contrat + refacto** (kickoff réel = `sprint82_kickoff.md`
en `active/`) et workflow-engine est DÉCALÉ post-S82. À corriger : repointer §6
vers le vrai kickoff, workflow-engine DÉCALÉ (futur slot), supersede D6 **RATIFIÉE**.

### 3 gates docs (critère machine)
= `check-sharding-docs.sh` + `check-frontier-contracts.sh` + `check-factory-docs.sh`
(les 3 gates docs-contrat). `check-spdx.sh` = **4e script mais gate de LICENCE
distinct**, PAS un gate docs-contrat. Câblés 3 surfaces : Woodpecker
`.woodpecker/ci-linux.yml:84-97`, GHA `.github/workflows/ci.yml:130-140`,
`verify.sh:108-115` (steps 19-21). **Nuance CI** : GHA « CI » est ROUGE
end-to-end (GTK/glib-sys, cf. CLAUDE.md) → le critère « exit 0 » se valide en
**local / Woodpecker** (vert).

## Vérification adversariale

| Claim du plan | verdict adversarial | disposition |
|---|---|---|
| « purger zombies Python **T44-T51** » | **RÉFUTÉ pour T49** (ancre Rust vivante `publish.rs`, ticket ouvert stale) | purger 7 (T44-48/50/51), **re-ancrer T49 séparément** |
| « **~80 tickets T\*** » | **RÉFUTÉ** (recompte = 60 en-têtes T : 52 shell + 8 rust ; sur-compte ~33 %) | re-audit distingue en-têtes `T{N}` / puces mnémoniques / prose |
| « stale **8/11 S80 → réel 4/10** » | **CORRIGÉ** : 2 jeux distincts (S79 8/11 vs S80 4/10), pas un compteur | re-audit LIVE 3 ledgers (D9), jamais swap aveugle |
| label « **ZOMBIE-supprimé** » | **CORRIGÉ** : sans précédent canonique | tombstone SUPERSEDED **ou** `DEPRECATED.md` (README §6.2.1) |
| collision **T15/T16 shell** | **CONFIRMÉ** exact (×2 chacune, shell-only) | supprimer T15a/T16a (préférable) **ou** renuméroter S77 en T52/T53 |
| décompte **S80 = 4 P2/10 P3** | **CONFIRMÉ** GROUNDED (table synthèse + `sprint81_plan.md:33`) | écrire 4/10 pour S80 APRÈS re-audit par item |
| purge **threat-safe** (0 cross-ref sécurité) | **CONFIRMÉ** (grep mot-borné = 0 ; T16a migré `blob_serve.rs:215`) | marquer « concern N/A — Python removed S50-S51 » |
| garde-fou **rust-T20** vivant (3 cross-refs sécurité) | **CONFIRMÉ P1** (`THREAT_MODEL:1169/:1738`, `HARDENING_ROADMAP:19`) | préserver ID + OPEN ; ne pas confondre avec shell-T20 |
| `git ls-files packages/` = 0 | **CONFIRMÉ** (synthétiseur : 0) | purge Python justifiée sans risque |
| **0 wire / 0 dep** (S4/S1b) | **CONFIRMÉ** (0 fichier `crates/`, 0 manifeste) | aucune considération compat format |

**Réserve honnête** : le « 4/10 » n'a pas été re-compté ligne-à-ligne
indépendamment (repose sur verdict S80 + table `:388-389` + corroboration
`sprint81_plan.md:33`) — le re-audit par item de Phase E (D9) le confirmera.

## Invariants à préserver

- **0 wire bump / 0 dep** : 100 % docs/planning/script ; tous `*_VERSION`
  restent =1 ; `zip` non concerné ; aucun champ de struct canonique touché.
- **Grep-history contract** : toute suppression laisse un tombstone SHA-ancré
  ou une entrée `DEPRECATED.md` (README §6.2.1) ; jamais un `git rm` muet.
- **rust-T20 intangible** : ID + OPEN préservés (carry sécurité vivant,
  cross-réf `THREAT_MODEL`/`HARDENING_ROADMAP`) ; namespace `rust` ≠ `shell`.
- **T49 préservé** : ticket Rust ouvert re-ancré (v4→v1, `:131`→`:183`), jamais
  purgé avec le bloc T44-T51.
- **Reconcile honnête** : re-audit LIVE des 3 ledgers (S79/S80/S81), jamais un
  swap `8/11→4/10` orphelinant les items S79 ouverts (S79-P2-1 → Phase F).
- **Frontière blob zip-only + garde feed S65** : rappelées (les préoccupations
  purgées ont migré en Rust) — non ré-ouvertes, non affaiblies.
- **Aucune décision Day-0/PO re-débattue** : suppression Python S50-S51,
  `*_ANNOUNCEMENT_VERSION=1`, PO-9 (workflow-engine décalé, D6 ratifiée) —
  CONFIRMÉES.
