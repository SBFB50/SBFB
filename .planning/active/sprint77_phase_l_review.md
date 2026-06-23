# Sprint 77 Phase L — Review

## Verdict: PASS

## Résumé

Phase L = `JsonSchema` derive purement additif sur les 8 payloads shard + génération de 8 (+1 enveloppe) fichiers `*.schema.json` + spec wire publique `SHARD_PROTOCOL_SPEC.md` (NET-NEW) + relocalisation des DTO `ShardSessionView`/`ShardSessionStatusResponse` de `http.rs` vers `nexus-core-rs::schemas::shard` (adaptation A du PLAN-ADAPT). Les 5 dimensions de review convergent : la sécurité (whitelist SI-3/SI-4), l'honnêteté de la spec (PROVISIONAL / RIG-ABSENT / carry S78 / admission ≠ confidentialité / self-claim), et tous les invariants (0-bump, 0-dep, no-float, derive additif, `*Entry` signés exclus) sont CLEAN et vérifiés ligne par ligne.

**Un seul finding bloquant a été rapporté (D3 : BLOCKER BOM UTF-8 sur `run_metrics.schema.json`) — il est REFUTÉ par re-vérification on-disk.** Le fichier ne porte AUCUN BOM dans l'état soumis (head `7b 0a` = `{` + LF, tail `}\n`, JSON valide ; les 9 `.schema.json` sont propres). D2 et D4 ont tous deux documenté que leur incident (parse error sur `run_metrics.schema.json`) était un artefact transitoire — file-lock race d'un cargo concurrent (D2) / tamper-test de drift restauré via `UPDATE_SNAPSHOTS=1` (D4) — et que le working tree a été remis byte-identique (`}\n`). L'état committable est donc vert. Le BLOCKER D3 décrit un état transitoire, pas l'arbre soumis : il dégrade en P3 process (recommander un check no-BOM lightcheck pour les JSON/MD versionnés, piège déjà rencontré `d6dea45`).

Restent uniquement des findings P2/P3 de doc-honnêteté, non bloquants, à consigner dans le body de commit. **PASS-PENDING n'est PAS un verdict committable final** : il valide la review humaine/agent ; le gate Codex reste à passer avant commit.

## Dimensions

### D1 — Derives (corrections dérives + génération schémas) : PASS
- 8 types EXACTEMENT reçoivent `JsonSchema`, ajouté en DERNIÈRE position du `#[derive(...)]` (vérifié : `shard_plan.rs:116,129,143,188,234,379,408` + `compute_group.rs:87` + `schemas/shard.rs:72,87`). Position de `Serialize`/`Deserialize` et attributs `#[serde(...)]` inchangés → 0 changement de sérialisation wire (test de drift `shard_schema_snapshot_matches_struct` vert).
- Constat mineur D1 (drift d'horizon « Phase K » dans `http.rs` vs « Sprint 78 » dans la spec/DTO) = RÉEL mais P3, et c'est le même carry pré-existant `STALE-PHASE-K-COMMENTS` confirmé ci-dessous (D5). Non bloquant.

### D2 — DTO-move (adaptation A) + impact daemon : PASS
- DTO `ShardSessionView`/`ShardSessionStatusResponse` supprimés de `http.rs`, importés depuis core (`http.rs:2094`). Consommateurs (`project_shard_session`, `live_shard_session` stub `None`, `shard_session_response`, route async, enregistrement `/api/daemon/shard-session/{id}`) byte-identiques vs HEAD. Réponse HTTP préservée (`Serialize` conservé, mêmes champs même ordre).
- DTO en core `pub` (requis cross-crate, sinon E0616), `#[derive(Debug, Clone, Serialize, JsonSchema)]` sans `Deserialize` (read-only sortant — correct). Surface publique core +2 types JUSTIFIÉE (contrainte daemon→core : `schema_for!` doit vivre où vit le type ; centralisation = single source des `.schema.json`, cohérent avec le précédent `TaskResponse`, conforme plan §20.1 + PO #2).
- Note process D2 (file-lock race d'un cargo concurrent → faux parse error) = artefact de harness, PAS un défaut code. Confirmé : fichier valide on-disk. Recommandation lancer les suites cargo en série RETENUE.

### D3 — Sécurité (whitelist + honnêteté) : PASS (le BLOCK rapporté est REFUTÉ)
- Whitelist `shard_session_view.schema.json` : `properties` = EXACTEMENT `{member_count, session_id}`, `required: ["session_id","member_count"]`, zéro champ identité en propriété (vérifié on-disk). Le `worker_pubkey` n'apparaît QUE dans la prose `description` — le test `shard_session_view_schema_is_whitelisted` (`schemas/shard.rs:251-277`) inspecte `/properties` (pas le blob brut) avec `assert_eq!` sur le set exact + garde négatif sur `worker_pubkey/initiator/members/assignments`. VRAIE garantie, pas un faux-positif. Runtime intact (`project_shard_session` ne pose que `session_id`+`member_count` ; `live_shard_session` = stub `None`).
- Types wire complets (`ShardAssignment`/`ShardedSessionManifest`/`RunProof`/`ComputeGroup`) publient `worker_pubkey`/`initiator` dans LEURS schémas : CORRECT, pas une fuite — ce sont les structures qui circulent entre membres authentifiés, distinctes du DTO observé. Spec §1 (admission ≠ confidentialité, honest-but-curious) + §5 (`is_member` AVANT `accept_bi`) + §4.7 cadrent honnêtement.
- **BLOCKER D3 (BOM `EF BB BF` en tête de `run_metrics.schema.json`) : REFUTÉ.** Re-vérification on-disk des 9 schémas : tous commencent par `7b 0a` (`{`+LF), `run_metrics.schema.json` finit par `}\n`, aucun BOM nulle part. Le drift canary passe sur l'arbre soumis. D3 a observé un état transitoire (le même incident que D2/D4 ont restauré). Le risque réel décrit n'existe pas dans l'arbre committable. Dégradé en P3 process.

### D4 — Tests (qualité sémantique + couverture) : PASS (2 P2 + 3 P3)
- 5 tests net-new évalués sémantiquement, drift-test prouvé fonctionnel par tamper-test (rougit → restauré byte-identique). Tous testent réellement ce qu'ils prétendent. `shard_session_view_schema_is_whitelisted` = le plus rigoureux (égalité exacte triée + boucle négative).
- 2 P2 doc-honnêteté/robustesse RÉELS (voir section P2/P3), 3 P3 mineurs. Aucun trou de couverture critique : 8 types schématisés couverts par drift, whitelist verrouillée exactement, caps/tags/ALPN const-checkés.

### D5 — Spec-scope (doc + scope-cut + grounding) : PASS
- `SHARD_PROTOCOL_SPEC.md` (241 lignes, NET-NEW) lu en entier, 7 points doc↔code groundés : 5 DOMAIN tags = valeurs réelles `canonical.rs` (verbatim), table des caps = vraies valeurs/noms (« enforced at BOTH sign and verify » confirmé), contrat ALPN fidèle (`is_member` avant `accept_bi`, length-prefixed big-endian, caps DoS), renvois THREAT §16 + PATTERNS §P64-69/§P39 tous résolvent, adaptation B (`ComputeGroup` schématisé + documenté), TUTORIAL Diataxis différé S78 (scope-cut consistent), PROVISIONAL honnête. 0 invention, 0 sur-promesse.
- Note D5 (commentaire `http.rs` périmé « Phase K ») = carry pré-existant `STALE-PHASE-K-COMMENTS` tracké `sprint78_audit_plan.md`, explicitement hors-scope L. Pas une régression de cette phase.

## Findings P0/P1 (bloquants — à corriger avant commit)

**Aucun.** Le seul finding bloquant rapporté (D3 BLOCKER BOM) est REFUTÉ par re-vérification : `crates/nexus-core-rs/src/schemas/run_metrics.schema.json` ne porte aucun BOM dans l'état soumis (head `7b 0a`, tail `}\n`, JSON valide). L'incident D3 (et le parse error analogue de D2) était un artefact transitoire (lock-race / tamper restauré). Le drift canary `schemas::shard::tests::shard_schema_snapshot_matches_struct` est vert sur l'arbre committable.

## Findings P2/P3 (à documenter dans le body de commit)

- **P2-1 (D4) — doc-overstate de `spec_consts_exist`** (`crates/nexus-core-rs/src/schemas/shard.rs:349-362`) : le doc-comment affirme « renaming a const reds the test ». FAUX pour les 7 caps (`MAX_SHARD_FRAME_BYTES`, `MAX_SHARD_N_CTX`, `SHARD_PLAN_MAX_ASSIGNMENTS`, `RUN_PROOF_MAX_PARTICIPANTS`, `SESSION_ID_MAX`, `SHARD_HASHES_MAX`, `COMPUTE_GROUP_MAX_MEMBERS`) qui sont des littéraux string, pas des refs const (contrairement aux 5 DOMAIN tags qui SONT de vraies refs compile-time). Renommer une const ne rougit pas le test. Fix : corriger la prose, OU transformer les caps en vraies refs (`const _: usize = MAX_SHARD_FRAME_BYTES;`). Valeur protégée (doc↔wire mentionne ces noms) reste utile. Non bloquant.

- **P2-2 (D4) — `schemas_publish_required_fields` n'assert jamais l'EXCLUSION des `#[serde(default)]`** (`crates/nexus-core-rs/src/schemas/shard.rs:192-245`) : le test vérifie l'inclusion des champs requis mais pas l'absence d'`activation_fingerprint` (RunProof) / `fallback_node` (ShardAssignment) de `required`. Les schémas sont corrects (vérifié), mais une régression (retrait accidentel de `#[serde(default)]`) ne serait captée que par le drift-test. Ajouter `assert!(!p.contains("activation_fingerprint"))` rendrait l'intention explicite et défendrait la tolérance runtime documentée. Non bloquant.

- **P3 (D1/D5) — commentaires `http.rs` périmés « Phase K »** (`crates/nexus-shell-daemon/src/http.rs:2109,2111,2120,2137,2142-2144,5209-5211`) : référencent encore « Phase K » / « Phase K+ » comme seam du live store, alors que Phase K (wrap-up S77 `0f597cf`) est livrée sans store et l'horizon réel = Sprint 78 (cohérent dans la spec `:201` + DTO core `:64`). Carry pré-existant `STALE-PHASE-K-COMMENTS` tracké `sprint78_audit_plan.md`, hors-scope Phase L. À re-libeller en S78. Aucun impact code/wire/test.

- **P3 (D3) — process : check no-BOM lightcheck** : recommander un garde-fou d'encodage no-BOM dans le hook lightcheck pour les `.json`/`.md` versionnés (le piège `Out-File`/`Set-Content` PowerShell a déjà cassé un frontmatter d'agent `d6dea45`). Hors-scope L, suggestion process.

- **P3 (D4) — couverture indirecte** : enums `ShardRole`/`KvCachePolicy` + `RunMetrics` non racines de `schema_snapshots()` mais couverts via `$defs` des parents (pas un gap) ; `schema_parses_as_valid_json_object` ne vérifie pas la valeur exacte de l'URL draft (couverte par drift) ; aucun test ne « lock » l'exclusion des `*Entry` du derive (garanti par compilation). Renforcements optionnels.

## Invariants confirmés

- **0-bump** : aucun `*_FORMAT_VERSION` ne change de valeur — les hits grep sont des doc-comments / re-export context shifté. `SHARD_PLAN`/`RUN_PROOF`/`COMPUTE_GROUP` = 1. Derive `JsonSchema` inerte à `Serialize` → JCS/Ed25519 inchangés.
- **0-dep** : `git diff --name-only HEAD | grep -i cargo` = NONE. `schemars 1.2` + `serde_json` déjà deps workspace de `nexus-core-rs` (bump S72-C). Le daemon n'ajoute rien (import de type seul, `cargo check -p nexus-shell-daemon` compile).
- **no-float-signé** : `run_metrics.schema.json` = 6 propriétés `"type": "integer"` (uint64/uint32), 1 `object` ; les 2 occurrences « number/double/float » sont de la prose `description` expliquant le rationale all-integer, pas des types JSON. `RunMetrics` all-integer confirmé.
- **whitelist SI-3/SI-4** : `shard_session_view.schema.json` `properties` = EXACTEMENT `{member_count, session_id}`, `required` = ces 2 seuls, zéro identité en propriété. Test `shard_session_view_schema_is_whitelisted` verrouille au niveau machine-lisible (égalité exacte + garde négatif).
- **derive additif** : `JsonSchema` en dernière position sur les 8 `#[derive(...)]`, ordre `Serialize/Deserialize` + attributs serde inchangés.
- **`*Entry` signés exclus** : `ShardedSessionManifestEntry` (`shard_plan.rs:306`), `RunProofEntry` (`shard_plan.rs:487`), `ComputeGroupEntry` (`compute_group.rs:160`) gardent `#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]` SANS `JsonSchema`. Aucun schéma `[u8;64]` BigArray généré ; 9 fichiers = 8 payloads + 1 enveloppe `shard_session_status_response`, 0 Entry/signature.

## Gate suites §7.4

Rappel des suites BLOQUANTES à exécuter et valider verts avant commit phase (cf. README §7.4, dual-platform memory) :
- `cargo nextest run -p nexus-core-rs --locked` (cible ~444 ; drift canary `shard_schema_snapshot_matches_struct` vert sur l'arbre soumis, BOM REFUTÉ).
- `cargo nextest run --workspace --locked` (Win natif + Docker canonique rust:1.94 `sbfb-ci`).
- `cargo clippy --workspace --all-targets --locked -- -D warnings` (0 warning hors artefact externe).
- `cargo fmt --all --check` (vert sous les 2 toolchains — convergence Win/Linux).
- `cargo test --workspace --locked --doc`.
- `cargo build -p nexus-shell-daemon --release`.

Le verdict PASS-PENDING suppose ces suites vertes au moment du commit. **Étape suivante obligatoire : gate Codex (review→commit bloquante)** ; PASS-PENDING n'autorise pas le commit tel quel.

---

## Traitement post-review (corrections appliquees, pas documentees)
Les 2 findings P2 de la review ont ete CORRIGES dans le code (no band-aid) :
- **P2-1** (`spec_consts_exist` doc-overstate) : les 7 caps sont desormais LIES a leurs vraies
  consts Rust (`crate::MAX_SHARD_FRAME_BYTES`, etc.) via `assert!(cap_value > 0, ...)` -> un
  renommage/suppression devient une erreur de COMPILATION, plus un drift silencieux ; prose corrigee.
- **P2-2** (required-fields n'assertait pas l'exclusion `#[serde(default)]`) : ajout d'assertions que
  `RunProof.activation_fingerprint` et `ShardAssignment.fallback_node` sont ABSENTS de `required`.

## Codex reconciliation
- **Codex run 1** (`codex exec`, GPT 5.5) : 6 livrables -> 4 CONFIRME, 0 GAP, **2 PARTIEL** :
  - L3 : `shard_session_status_response.schema.json` rendait `session` optionnel alors que le
    contrat envelope (S73-E/S75-D) garantit `session` TOUJOURS serialise (`null` en absence).
  - L4 : `lib.rs` ne re-exportait que les 2 DTO, pas les 8 fonctions schema (incoherent vs
    `task_response_schema`).
- **Corrections** (no band-aid) : `#[schemars(required)]` sur `session` (schema fidele :
  `required: [found, session]`) + assertion de garde dans `schemas_publish_required_fields` ;
  re-export des 8 fonctions schema a la racine `lib.rs`.
- **BOUCLE COMPLETE relancee** : regen snapshots (`UPDATE_SNAPSHOTS`) + Windows
  fmt/clippy --all-targets/nextest core **444/444** + Docker rust:1.94 drift-test core (snapshot
  envelope byte-identique cross-platform) + **Codex run 2 = 6/6 CONFIRME, 0 GAP, 0 PARTIEL**,
  6 invariants CONFIRME.
- Artefact Codex brut (dernier run) : `sprint77_phase_l_codex_review.md` (non reecrit par Claude).

## Note environnementale (hors Phase L)
`operator_sprint_history_endpoint` (sbfb-factory) echouait en workspace tant que le package
`sprint79_factory_*` (artefact session decouverte hors-process) etait dans `.planning/active/`
(`detect_history_sprint` prend le max sprint avec kickoff/plan -> 79 sans phases -> 404). Resolu
HORS-Phase-L par decision PO : package S79 relocalise a la racine `.planning/` (untracked, intact) ;
driver d'acceptance `shard_node.rs` (cassait `--all-targets`) gate via
`required-features=["llm_llama_cpp"]`, commit chore `6e07182`. Avec ces deux assainissements,
**nextest workspace = 1954/1954, 0 skip** (Windows) + Docker core vert.
