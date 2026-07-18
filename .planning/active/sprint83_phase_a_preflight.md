# Sprint 83 — Phase A — Preflight G8

## Verdict: EXECUTE

Les 5 scans (S1a OSS, S1b deps/CVE, S2 décisions historiques, S3 threat model,
S4 wire+split) convergent sur **EXECUTE** après réfutation adversariale (S3
rejoué main-thread après échec technique de l'agent Workflow — voir §S3).
Aucun fait matériel n'a été réfuté (tous CONFIRMED, 2 IMPRECISE cosmétiques
sans impact verdict, 1 IMPRECISE — la thèse « chore bypass Codex » — corrigée
par vérification directe du hook). Aucune décision gelée (D1..D5, Day-0,
politique pre-launch) n'est ré-ouverte. Phase A = artefacts de cadrage + stubs
schemars VIDES + test grammaire : 0 wire bump, 0 nouveau DOMAIN, 0 route réseau,
crate PUR. **Preuves d'enforcement vérifiées** : golden schemars 7/7 draft
2020-12, `check-frontier-contracts` clean (census 25), fmt/clippy verts, test
grammaire 23/23.

## Contexte (phase, HEAD, gates ouverts)

- Phase : Sprint 83 Phase A — cadrage, matrice de gates et grammaire de phase.
- HEAD : `803a451` (S83 kickoff + ratifications ; local ahead origin `aa9db47`).
- S82 clos = CONDITIONAL PASS `74531d4`.
- Gates ouverts/joués en Phase A (plan §Phase A l.226-228) :
  - **G-SCOPE-FLOOR** : OUVRE en Phase A, fermeture AJ1 (cohérent, aucun gate
    fermé prématurément).
  - **G-PHASE-GRAMMAR** : joué (preuve test grammaire bi-lettre).
  - **G1** (design review) + **G9** (préflight présent) : mécaniques hook.
- Livrables Phase A : `sprint83_gate_matrix.md`, `sprint83_t2_spec.md`, test
  grammaire `scripts/agent/test_agentctl.py`, crate NEUF
  `crates/sbfb-workflow-core` (7 stubs schemars + golden `schema_snapshot_stub`),
  `docs/factory/WORKFLOW_AGENTS_OVERVIEW.md`.

## S1a OSS prior-art — EXECUTE — faits confirmés + findings

- **schemars 1.2.1** résolu à `Cargo.lock:7530-7531`, consommé par 3 crates
  workspace. Défaut schemars 1.x = JSON Schema **draft 2020-12** ; les schémas
  générés existants déclarent `$schema` draft/2020-12 avec `$defs`
  (`crates/nexus-core-rs/src/schemas/task_response.schema.json`). ⇒ le golden
  `schema_snapshot_stub` (draft 2020-12 non-vide) est factuellement réalisable.
  [CONFIRMED]
- **Précédent matrice/tracks** interne = pattern S82 audit_plan ;
  `.planning/active/sprint83_audit_plan.md` porte déjà 15 références track/gate.
  [CONFIRMED]
- IMPRECISE cosmétique (n'affecte pas le verdict) : la formule « in-toto subject
  blake3 **standard** » sur-affirme — les digests standardisés in-toto sont
  sha256/sha512 ; blake3 est le choix propre de SBFB « à la in-toto » (D2 le
  gèle explicitement). Ne pas écrire « standard in-toto » dans le commit body.

## S1b Deps/CVE — EXECUTE — liste deps+versions, 0 dep neuve prouvée

Le crate NEUF `crates/sbfb-workflow-core` (PUR) n'a besoin que de dépendances
**déjà pinnées au workspace** :

- `Cargo.toml:57` serde 1.0 (derive) — **dep normale** (7 types portent
  `#[derive(JsonSchema)]` + serde).
- `Cargo.toml:360` schemars 1.2 (derive) — **dep normale** ; déjà consommée par
  `nexus-core-rs/Cargo.toml:122` + `nexus-worker-core/Cargo.toml:146`. 0 dep neuve.
- `Cargo.toml:58` serde_json 1.0 — **dev-dep** suffisante pour le golden test.
- Différés aux phases de remplissage (D2/D3, PAS Phase A) : serde_jcs 0.2
  (`Cargo.toml:65`), ed25519-dalek 2.1 (`:68`), blake3 1.5 (`:69`), rusqlite 0.36
  (`:135`), rusqlite_migration 2.2 (`:136`).

Faits :
- **0 dépendance runtime neuve, 0 nouveau nœud de lock, 0 duplicate introduit.**
  Le seul doublon schemars (0.9.0 `Cargo.lock:7518` + 1.2.1 `:7530`) est
  PRÉ-EXISTANT via serde_with 3.20.0 (`Cargo.lock:7796-7808`). `deny.toml:137`
  `multiple-versions = "warn"` (pas deny). [CONFIRMED]
- **0 advisory** ouvert sur schemars/serde. Seuls ignores actifs = 2 quick-xml
  iroh-internes (`deny.toml:84-85`, RUSTSEC-2026-0194/0195). [CONFIRMED]
- **CI-1 (supply-chain rouge)** = Unlicense/MIT-0 absents de l'allowlist
  `deny.toml:103-119` (pré-existant S20/S32, routé S83 Phase A1). Un crate PUR
  serde+schemars ne tire AUCUN crate Unlicense/MIT-0 ⇒ Phase A **n'aggrave ni
  ne résout** CI-1. Ne PAS coupler la création du crate à ce ticket. [CONFIRMED]

## S2 Décisions historiques — EXECUTE — grammaire prouvée, 0 décision gelée ré-ouverte

- **Grammaire Phase `[A-Z]+[0-9]?`** confirmée aux 4 sites, bi-lettre identique :
  `scripts/agent/agentctl.py:28` (PHASE_TITLE_RE) + `:32` (FALLBACK) + `:566`
  (re.search) ; `.claude/hooks/phase-precommit-lightcheck.sh:88` ;
  `.claude/hooks/phase-auditor-gate.sh:54`. Labels S83 A1/W1/AA/AI2/AJ1 parsent
  tous (`[A-Z]+` exige ≥1 lettre + `[0-9]?` optionnel) ; « Phase 3 » (0 lettre)
  correctement refusé. [CONFIRMED]
- **`test_agentctl.py:46`** teste DÉJÀ A1, AA, AB1 ; **`:68-70`** refuse DÉJÀ les
  nombres nus '0'/'8'. Le fichier actuel = un seul `main()` + `_title`/`_check`,
  **aucune fonction `test_*` nommée**. [CONFIRMED]
- **`canonical.rs` = 23 `pub const DOMAIN_*_V1`** ; Phase A n'en ajoute AUCUN
  (stubs vides). Les DOMAIN_* dédiés Workflow (D2) sont déclarés aux phases de
  remplissage. [CONFIRMED] **PIÈGE** : le chiffre « DOMAIN 25 frozen » de
  l'en-tête orchestrateur = census schema S82-G (concept DISTINCT), PAS les 23
  constantes de `canonical.rs`. Ne pas confondre les deux compteurs dans la matrice.
- **§P75** (`docs/rust/PATTERNS.md:4314`, « golden HTTP characterization ») =
  pattern golden committé + test de validation, transposable au golden schemars
  `schema_snapshot_stub`. [CONFIRMED]
- **`verification_blueprint.md:3-24`** porte DOUBLE bannière SUPERSEDED
  (ARCHITECTURE 2026-07-15 + S82 Phase E PO-9) ; ses décisions (moteur-dans-Factory,
  interprètes Rust+JS, 8 nodes figés, task_submit-suffit) sont NON-activables,
  remplacées par D3/D4 gelées. [CONFIRMED]
- **Aucune décision gelée D1..D5 / Day-0 / pre-launch violée.** [CONFIRMED]

Findings honnêteté (P3, adoptés) :
- **S2-1** : NE PAS revendiquer « premier exercice bi-lettre » dans le commit
  body — A1/AA/AB1 sont déjà testés (`:46`). Seuls les *strings* W1/AI2/AJ1 sont
  non-exercés (leurs FORMES le sont). Valeur réelle du nouveau test = fonctions
  `test_*` nommées + assert que les 2 hooks bash matchent la même grammaire
  (non testé aujourd'hui) + cas '3' explicite.
- **S2-2** : delta « +2 agentctl » honnête si compté comme 2 fonctions `test_*`
  nouvelles, PAS 2 comportements neufs protégés.

## S3 Threat model — EXECUTE — 0 surface neuve, décision `// FRONTIER` CORRIGÉE

> **Incident** : l'agent S3 du Workflow preflight a échoué techniquement
> (StructuredOutput retry cap dépassé — `pipeline[3]`). Cette section a été
> **rejouée et vérifiée main-thread** contre le code au HEAD ; elle **corrige**
> la recommandation `// FRONTIER` inférée par la synthèse (qui aurait cassé le
> gate). Vérification directe de `scripts/check-frontier-contracts.sh`.

- Phase A = cadrage + stubs schemars VIDES + test grammaire : **aucune surface
  d'attaque neuve** (0 route réseau, 0 verbe bridge, 0 runtime, crate PUR sans
  Tokio/réseau/DB — conforme D3).
- Les 7 types stub sont des **frontières nommées vides** — les schémas sont
  remplis (et leur menace analysée) aux phases dédiées.
- **DÉCISION `// FRONTIER` CORRIGÉE (ne PAS annoter en Phase A)** : le check (2)
  de `check-frontier-contracts.sh:131-170` **exige** qu'un type porteur d'un
  `// FRONTIER: <name> domain=DOMAIN_X_V1 version=X_FORMAT_VERSION` **résolve**
  ses consts `const DOMAIN_*` + `const *_VERSION` sous `crates/` ET un
  `schema_for!` ou une exemption. Les stubs Phase A n'ont **pas** ces consts (ils
  arrivent au remplissage). Annoter maintenant **ferait échouer le gate**. Les
  types **non annotés ne sont PAS des violations** (registre opt-in, header
  check (2) l.37-47). ⇒ Phase A laisse les stubs non annotés ; l'étiquette
  `// FRONTIER` est posée au commit de la primitive **remplie** (AM-3).
  **Vérifié** : `bash scripts/check-frontier-contracts.sh` = `clean` avec le
  crate présent (census `DOMAIN_*_V1` = **25 frozen** intact, 0 STALE-PHASE-K).
- **RATIFICATION PO REQUISE (tension AM-3, relevée Codex round 1)** : AM-3
  (kickoff:536) nomme littéralement PackV1/GraphV1/ProfileV1/ProjectionV1 comme
  primitives portant `// FRONTIER:` **dans le commit qui les introduit**. Or ce
  commit n'introduit que des **identités vides** (stubs) : annoter maintenant
  exigerait de déclarer les consts `DOMAIN_*`/`*_VERSION` — ce qui **casse** le
  census gelé 25 + le gate + D2 (« domaines ajoutés au remplissage »). La
  résolution est donc FORCÉE : `// FRONTIER:` est posé au commit de
  **remplissage** de chaque contrat (F/G/H/D/X). Exception à ratifier :
  **« identité vide réservée ≠ primitive de frontière remplie »**. Sans autre
  voie technique cohérente. **RATIFIÉE PO (Option A, 2026-07-18)** : exception
  actée, `// FRONTIER:` posé au commit de remplissage de chaque contrat
  (F/G/H/D/X), kickoff AM-3 inchangé.
- **Anti STALE-PHASE-K** : la doc in-code du crate (`crates/sbfb-workflow-core`,
  scanné par le check (1)) est au **présent** (« fields authored in its dedicated
  phase »), sans jeton `Phase X will` / `Sprint N will` — vérifié gate clean.
- L'invariant Plan-Then-Execute (D4) et l'intersection 7-couches (Phase C avant
  tout pouvoir bridge) ne sont pas touchés : aucun verbe Workflow n'est câblé
  en Phase A. Aucun bump `THREAT_MODEL` requis (route AJ1, AM-3).

## S4 Wire format + split commit — EXECUTE — 0 wire bump prouvé, DÉCISION SPLIT

Wire (pre-launch) :
- **Tous les `*_VERSION` wire existants restent à 1** (`public_feed.rs:20`
  FEED_FORMAT_VERSION=1, `task.rs:61` TASK_FORMAT_VERSION=1,
  `publish.rs:24`, `iroh_runtime.rs:93`, node_directory/seed/shard/pow/curator/
  compute_group/activation_commit/canary/key_rotation tous =1). Seuls
  `invite.rs:73` INVITE_FORMAT_VERSION=2 et `keystore.rs:108` BLOB_VERSION=0x01.
  Phase A ne modifie AUCUN de ces fichiers. [CONFIRMED]
- **Aucun format Workflow n'existe** (grep RUN_EVENT/WORKFLOW/PROMPT_AGENT_PACK/
  AGENT_DEFINITION/COMPILED_PLAN dans `crates/` = 0 hit ; crate absent). Les 7
  stubs .v1 sont des **contrats NEUFS vides**, pas des bumps. **0 bump
  FEED_FORMAT_VERSION** (les stubs ne sont pas encore une raw-op). [CONFIRMED]
- Recommandation : **ne déclarer AUCUN const `*_VERSION`** dans les stubs Phase A
  (garder vides ; versions/DOMAIN aux phases de remplissage). Note corrigée :
  le Check 4 (WARN wire-format, non-bloquant) du hook est **filename-based**
  (`lightcheck.sh:218` `git diff --name-only | grep -E canonical.rs|schemas/|_VERSION`),
  pas const-based — un fichier sous `schemas/` déclencherait un WARN non-bloquant.
  Sans impact verdict.

## Findings consolidés (P0..P3) après réfutation adversariale

- **P0/P1** : aucun.
- **P2 — S2-3 (REQUALIFIÉ)** : la thèse « `chore(planning)` contourne la gate
  Codex » est **fausse au niveau du hook** (backstop réel). Vérification directe :
  `.claude/hooks/phase-precommit-lightcheck.sh:294` Check 7 matche
  `^(feat|fix|docs|chore|test|refactor)\(` — **`chore` EST inclus**. Un commit
  `chore(...): Sprint N Phase X` déclenche donc Codex (Check 7 l.294), préflight
  (Check 8 l.398) ET 9-sections (Check 9 l.418). L'exemption `chore` n'existe que
  dans la couche Python `agentctl.py:564` (`^(feat|fix|docs|test|refactor)\(`,
  chore absent), qui n'est PAS le backstop de commit. ⇒ le split N'EST PAS requis
  pour obtenir Codex. Reste le point de **labeling** (voir décision ci-dessous).
- **P3 — S2-1/S2-2** : honnêteté du commit body (pas de « premier exercice
  bi-lettre » ; delta test compté en fonctions `test_*`).
- **P3 — S4-WIRE-3 (IMPRECISE)** : mécanisme Check 4 décrit const-based au lieu
  de filename-based ; WARN non-bloquant, sans impact.
- **INFO — S1a** : « in-toto standard » à ne pas écrire (blake3 = choix SBFB).
- **MÉCANIQUE (bloquant si oublié)** : ajouter `crates/sbfb-workflow-core` aux
  workspace members `Cargo.toml:3-18` (sinon `cargo nextest run -p sbfb-workflow-core`
  échoue). Modèle d'ajout : `sbfb-manifest` (`Cargo.toml:15`) déjà crate+member.
- **RAPPEL** : golden `schema_snapshot_stub` doit suivre §P75 (`PATTERNS.md:4314`).

## Décision de séquencement de commit (actionnable)

**DÉCISION : UN SEUL commit.**

Titre exact :
`feat(workflow-core): Sprint 83 Phase A — cadrage, matrice de gates et grammaire de phase`

Périmètre du commit unique (11 fichiers de livrable + 3 artefacts process) :
- `crates/sbfb-workflow-core/` (Cargo.toml + src/lib.rs + src/contracts.rs +
  src/contracts.schema.json golden ; 7 stubs schemars + `schema_snapshot_stub`)
- édition `Cargo.toml` (ajout aux `[workspace].members`) + **`Cargo.lock`**
  (entrée du nouveau membre — régénérée par cargo, staging obligatoire)
- `scripts/agent/test_agentctl.py` (test grammaire + anti-régression '3')
- `.planning/active/sprint83_gate_matrix.md`
- `.planning/active/sprint83_t2_spec.md`
- `docs/factory/WORKFLOW_AGENTS_OVERVIEW.md`
- Artefacts process (Check 7/8) : `sprint83_phase_a_preflight.md` (ce fichier),
  `sprint83_phase_a_review.md` (PASS-PENDING→PASS), `sprint83_phase_a_codex_review.md`
- Body : 9 sections `##` obligatoires (Check 9).

Justification (résolution du conflit inter-scans) :
1. **Le hook n'exige pas de split.** Le plan A.5 conditionne le scindage à
   « si le hook lightcheck l'exige » — or vérification directe : le hook
   applique Codex/préflight/9-sections à TOUS les types y compris `chore`
   (`lightcheck.sh:294/398/418`). La condition de scindage n'est PAS déclenchée.
2. **Un split DOUBLERAIT la charge Codex** : les DEUX commits (feat + chore)
   déclencheraient chacun Check 7 ⇒ deux artefacts Codex, deux préflights, deux
   review.md — pour zéro bénéfice mécanique. La justification originale du split
   (S2-3 : « chore bypass Codex ») est réfutée.
3. **`feat` est le type CORRECT** : le crate introduit une API publique
   fonctionnelle + test golden ⇒ `feat`, pas `chore` (mislabel). `feat` honore
   aussi la rigueur per-phase (Codex + review.md déclenchés).
4. **Préserve « un commit par phase »** (discipline cardinale CLAUDE.md).

Alternative (2 commits `feat(workflow-core)` puis `chore(planning)`) : hook-valide
mais NON recommandée (double Codex, rompt un-commit-par-phase, sans nécessité).

## Questions ouvertes PO (le cas échéant)

- **Aucune bloquante — Phase A peut démarrer immédiatement.**
- Info PO (non-bloquant) : le plan A.5 nommait par défaut `chore(planning)` ; le
  préflight retient `feat(workflow-core)` en commit unique (le plan délègue
  explicitement « DÉCISION AU PREFLIGHT »). Motif : le hook enforce Codex sur
  `chore` (l.294), donc le split par défaut n'apporte rien et le crate
  fonctionnel impose `feat`. Ratification implicite suffisante ; à signaler au
  wrap-up si le PO préfère la trace `chore(planning)` séparée.
