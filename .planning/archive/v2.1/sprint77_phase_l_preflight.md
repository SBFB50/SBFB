# Sprint 77 Phase L — Preflight G8

## Verdict: PLAN-ADAPT

## Resume (3-5 lignes)
Phase L (spec wire machine-lisible + JSON Schemas generes drift-gated) s'implemente dans un
precedent etabli : `schemars 1.2` est deja dep workspace (consommee par `nexus-core-rs`), le
mecanisme anti-drift `schema_snapshot_matches_struct` existe byte-pour-byte
(`schemas/task_response.rs` + `.schema.json`), et tous les types wire shard *payload* sont
all-integer / `[u8;N]` / primitives / enums fermes -> `#[derive(JsonSchema)]` est purement additif,
0 nouvelle dep, 0 float, 0 type iroh. Le verdict n'est PAS EXECUTE : deux points du plan §20.1
exigent une adaptation d'implementation **verifiee par lecture du code** — (A) les DTO
`ShardSessionView`/`ShardSessionStatusResponse` sont PRIVES dans `nexus-shell-daemon/src/http.rs`
et ce crate n'a pas `schemars`, alors que le plan place leur `schema_for!` dans
`nexus-core-rs/src/schemas/shard.rs` (contrainte de **direction de dependance** : core ne peut pas
voir un type du daemon) ; (B) le banner spec liste `ComputeGroup(+Entry)` comme type documente
mais la liste `schema_for!` du plan ne l'inclut pas (incoherence banner<->schemas). Aucune
decision Day-0 figee n'est touchee.

## Validite de ce preflight (transparence orchestration)
Le Workflow preflight (5 scans Opus 4.8 1M en fan-out) a vu **3 scans echouer** (`S1a`, `S3`, `S4`)
sur `StructuredOutput retry cap (5) exceeded` (schema de sortie trop strict — lecon process pour
les prochains Workflows : assouplir `additionalProperties`/`required`). Seuls **S1b** (deps) et
**S2** (decisions) ont rendu un verdict structure. Le synthetiseur a comble S1a/S3/S4 par sa
propre lecture. **Conformement a la regle ultracode (verifier adversarialement)**, chaque fait
des scans manquants a ete **RE-VERIFIE manuellement par lecture directe du code** (greps/reads
ci-dessous) AVANT promotion du verdict — tous confirmes, 0 hallucination detectee.

## S1a OSS prior-art — RE-VERIFIE manuellement (scan workflow echoue)
- **Pattern transposable directement.** `crates/nexus-core-rs/src/schemas/task_response.rs:42-153`
  definit le type wire `TaskResponse` (PAS un miroir : `TaskResponse::new`/`validate_identity`
  l.110-136 sont le vrai type runtime), derive `JsonSchema` (l.66/96), `schema_for!(TaskResponse)`
  -> `serde_json::to_value` (l.150-152), snapshot `task_response.schema.json` (draft 2020-12,
  `$defs`), drift-test `schema_snapshot_matches_struct` rafraichi via `UPDATE_SNAPSHOTS=1`.
- **`[u8;32]` derivable** : `schemars 1.2` impl `JsonSchema` pour `[T; N]` via const-generics
  (genere un array borne). Les types shard payload n'utilisent que `[u8;32]` / `[u8;PUBLIC_KEY_LENGTH]`
  (= 32) — derivables nativement. Seuls les `*Entry` portent `[u8;64]` via `serde_big_array`
  (cf. S4) et sont exclus.
- **Signal : EXECUTE** (pattern reutilisable tel quel).

## S1b deps — signal EXECUTE (workflow, recoupe)
- `schemars 1.2.1` est DEJA dep resolue : `Cargo.toml:350` `schemars = { version = "1.2",
  features = ["derive"] }` + `crates/nexus-core-rs/Cargo.toml:120` `schemars = { workspace = true }`.
  Ajouter `#[derive(JsonSchema)]` = 0 nouvelle ligne lockfile.
- Champs des types shard payload (`shard_plan.rs`) = u8/u16/u32/u64/String/bool + `[u8;32]`
  + `Vec<[u8;32]>` + `Option<[u8;32]>` + 2 enums unit (`ShardRole`/`KvCachePolicy`,
  `#[serde(rename_all="snake_case")]`). AUCUN champ iroh / `VerifyingKey` / `blake3::Hash`.
  **Invariant 0-dep tenu.**
- `compute_group.rs:87` `ComputeGroup` = version u16, group_id String, initiator `[u8;32]`,
  revision u64, members `Vec<[u8;32]>` -> aussi derivable trivialement.
- **CONCERN (-> Adaptation A)** : DTO `ShardSessionView`/`ShardSessionStatusResponse` PRIVES
  dans `nexus-shell-daemon/src/http.rs:2104/2119` ; `nexus-shell-daemon/Cargo.toml` n'a PAS
  `schemars` (verifie : grep = 0 match).
- **CONCERN (-> Adaptation B)** : `ComputeGroup` figure dans le banner doc mais hors liste
  `schema_for!` du plan.

## S2 decisions historiques — signal SCOPE-CUT-CONSISTENT (workflow, recoupe)
- Aucune Day-0 figee n'interdit le derive. S72-C (`3c9ea1b`) a deja bump `schemars 0.8 -> 1.2`
  (DESIGN-CONFLICT resolu PO Option A, `ollama-rs 0.3.4` l'exige) -> Phase L = 0 nouvelle dep.
- Decision PO #2 explicite : `.planning/active/sprint77_plan.md:718-720` « `#[derive(JsonSchema)]`
  additif sur les types wire shard -> OUI ... precedent `TaskResponse` ; mecanisme anti-drift de L
  (pas un type-miroir fragile) ».
- Regime pre-launch raw-op additif confirme : `shard_plan.rs:76,80` `SHARD_PLAN_FORMAT_VERSION:u16=1`
  / `RUN_PROOF_FORMAT_VERSION:u16=1` ; `compute_group.rs:66` `COMPUTE_GROUP_FORMAT_VERSION:u16=1`.
  Le derive JsonSchema est inerte a `serde::Serialize` -> 0-bump, signatures Ed25519 inchangees.
- No-float-signe : `shard_plan.rs:46-53` documente l'invariant ; `RunMetrics` est all-integer
  (`shard_plan.rs:378-399` : `ttft_ms`/`decode_milli_tokens_per_sec`/`network_rx_bytes`/
  `network_tx_bytes` u64, `worker_drop_count` u32). Phase L ne touche pas l'encodage.

## S3 threat — RE-VERIFIE manuellement (scan workflow echoue)
- Whitelist DTO confirmee a la lecture `http.rs:2095-2135` : `ShardSessionView`
  = `{ session_id: String, member_count: usize }` UNIQUEMENT ; doc verbatim « An aggregate count,
  never the member identities » et « Exposes ONLY the aggregate `member_count`, never any
  `worker_pubkey` / `initiator` (the private-group composition, SI-3/SI-4) ». `project_shard_session`
  retourne `plan.assignments.len()` — jamais une identite.
- Phase L **documente** cette whitelist via le schema, ne l'**elargit pas**. Le schema du DTO
  doit publier exactement 2 champs.
- `docs/security/THREAT_MODEL.md` §16 existe (SI-1..SI-11). La spec renvoie a §16, ne duplique pas.
- **Note (hors scope L)** : le doc-comment du DTO mentionne « Sprint 77 Phase K » pour des champs
  futurs — c'est le carry `STALE-PHASE-K-COMMENTS` deja liste `sprint78_audit_plan.md`. Ne pas
  toucher en Phase L.
- **Signal : EXECUTE** (documenter est sur avec le caveat admission != confidentialite).

## S4 wire invariants — RE-VERIFIE manuellement (scan workflow echoue)
- **Derive additif, 0-bump** : `task_response.rs` prouve que `#[derive(JsonSchema)]` coexiste avec
  `Serialize/Deserialize` sans rien changer a la representation serde -> 0 changement wire, aucun
  `*_FORMAT_VERSION` ne bouge, bytes canonical JCS Ed25519 inchanges.
- **5 DOMAIN_*_V1** verifies `canonical.rs` : `DOMAIN_COMPUTE_GROUP_V1` (l.258,
  `nexus-compute-group-v1`), `DOMAIN_SHARD_PLAN_V1` (l.276, `nexus-shard-plan-v1`),
  `DOMAIN_RUN_PROOF_V1` (l.290, `nexus-run-proof-v1`), `DOMAIN_VRF_DRAW_V1` (l.310,
  `nexus-vrf-draw-v1`), `DOMAIN_ACTIVATION_COMMIT_V1` (l.332, `nexus-activation-commit-v1`).
  La spec doit les citer verbatim (cible du `spec_consts_exist`).
- **Contrat ALPN** verifie `shard.rs` : `SHARD_ALPN` (defini dans `crate::node`, importe l.68),
  `MAX_SHARD_FRAME_BYTES = 256*1024*1024` (l.85), `MAX_SHARD_N_CTX: u32 = 8192` (l.97),
  `is_member` (l.304) AVANT `accept_bi` (l.314), framing length-prefixed BE, caps DoS write (l.108)
  ET read-declared (l.121). Verdict hors-bande : la spec le rappelle.
- **`*Entry` exclus du derive** : `ShardedSessionManifestEntry`/`RunProofEntry`/`ComputeGroupEntry`
  portent `signature: [u8; SIGNATURE_BYTES]` (=64, `crypto.rs:49`) via `#[serde(with = "BigArray")]`
  -> deriver JsonSchema y produirait un schema array de 64 items verbeux non desire. Coherent avec
  le precedent `TaskResponse` (on derive le TYPE payload, pas l'enveloppe signee).
- PATTERNS `docs/rust/PATTERNS.md` §P64-69 + §P39 existent -> renvois valides.
- **Signal : EXECUTE** (derive purement additif).

## Approche retenue pour le code (livrables + ordre + decisions A/B tranchees)

Ordre : (1) derives -> (2) `schemas/shard.rs` + DTO deplaces + snapshots -> (3) spec doc -> (4) tests.

1. **Derives JsonSchema additifs** sur les types wire payload :
   - `shard_plan.rs` : `ShardAssignment`, `ShardPlan`, `ShardedSessionManifest`, `RunProof`,
     `RunMetrics`, `ShardRole`, `KvCachePolicy` (+`use schemars::JsonSchema;`).
   - `compute_group.rs` : `ComputeGroup` (decision B).
   - **NE PAS** deriver sur les `*Entry` signes (`[u8;64]` BigArray).

2. **ADAPTATION A — DTO definis cote core (tranche).** Contrainte dure : `nexus-shell-daemon`
   depend de `nexus-core-rs`, donc core ne peut PAS voir un type du daemon ; pour que
   `schema_for!(ShardSessionView)` compile dans `schemas/shard.rs`, le type DOIT vivre dans core.
   **Decision** : DEPLACER la definition de `ShardSessionView` + `ShardSessionStatusResponse` de
   `http.rs` vers `nexus-core-rs/src/schemas/shard.rs` (rendus `pub`, derive
   `Debug, Clone, Serialize, JsonSchema` — pas besoin de `Deserialize`), re-export via
   `schemas/mod.rs` (`pub use`) + racine `lib.rs` (pattern `TaskResponse`). `http.rs` les importe
   (`use nexus_core_rs::{ShardSessionView, ShardSessionStatusResponse};`). `project_shard_session`
   + `shard_session_response` RESTENT dans `http.rs` (logique de projection a partir de l'etat
   runtime). Rayon d'impact = `http.rs` seul (verifie : usages aux l.2104/2119/2124/2131/2155-2161
   + test l.5294). Whitelist PRESERVEE (le type garde ses 2 champs). Choix vs alternative
   « ajouter schemars au daemon » : core-centralise = conforme plan §20.1, single source des
   `.schema.json`, daemon non alourdi, respecte PO #2 (pas un type-miroir — le MEME type, deplace).

3. **ADAPTATION B — ComputeGroup schematise (tranche).** Le banner liste `ComputeGroup` comme type
   documente ; pour eviter une section doc sans contrat machine et honorer « ultra-complet »,
   **inclure `ComputeGroup` dans `schema_for!`** (genere `compute_group.schema.json`). +1 schema /
   +1 entree drift vs la liste litterale du plan. `ComputeGroupEntry` reste hors derive.

4. **`docs/protocol/SHARD_PROTOCOL_SPEC.md`** (anglais, style `docs/protocol/PUBLIC_FEED_SPEC.md`) :
   banner regime pre-v1.0 raw-op additif (`*_FORMAT_VERSION=1`, 5 `DOMAIN_*_V1`, 0-bump/0-dep) ;
   section par type (`ComputeGroup`, `ShardAssignment`, `ShardPlan`, `ShardedSessionManifest(+Entry)`,
   `RunProof(+Entry)`, `RunMetrics`, + DTO observe) ; section ALPN `sbfb/shard/1` (bi-stream QUIC
   long-lived, framing length-prefixed BE, `MAX_SHARD_FRAME_BYTES=256MiB`, `MAX_SHARD_N_CTX=8192`,
   `is_member` crypto-before-`accept_bi`, caps DoS sign ET verify, verdict hors-bande) ; renvois
   THREAT_MODEL §16 + PATTERNS §P64-69. Caveat cardinal admission != confidentialite.

5. **`crates/nexus-core-rs/src/schemas/shard.rs`** : `use` des types depuis `crate::shard_plan` /
   `crate::compute_group` + fonctions `*_schema()` (`schema_for!`) + DEFINITION des 2 DTO +
   snapshots `*.schema.json` (draft 2020-12) generes via `UPDATE_SNAPSHOTS=1`. Enregistrer
   `pub mod shard;` dans `schemas/mod.rs` + `pub use`.

6. **Tests** : `shard_schema_snapshot_matches_struct` (drift, miroir l.277-308) ;
   `schema_parses_as_valid_json_object` + required-fields par type (miroir l.159-187) ;
   `spec_consts_exist` (const-check doc<->code : chaque `DOMAIN_*_V1` + cap citee dans la spec
   existe comme const Rust). Compte revise : plan annoncait +2/+3 ; avec B (+ComputeGroup) viser
   ~+4/+6 selon granularite (un test parametrise par type ou un test par type).

## Invariants a respecter
- **0-bump** : aucun `*_FORMAT_VERSION` ne change ; derive inerte a la serialisation canonique JCS.
- **0-dep** : `schemars 1.2.1` deja resolu ; AUCUN `Cargo.toml` modifie (ni core ni daemon —
  l'adaptation A evite justement d'ajouter schemars au daemon).
- **no-float-signe** : ne pas introduire de float ; `RunMetrics` reste all-integer.
- **whitelist DTO** : le schema de `ShardSessionView` publie EXACTEMENT `session_id` + `member_count` ;
  jamais `worker_pubkey`/`initiator` (SI-3/SI-4).
- **DOMAIN tags exacts** cites verbatim dans la spec (les 5 ci-dessus).
- **NE PAS deriver** sur les `*Entry` signes (`[u8;64]` BigArray).

## Risques residuels / points de vigilance review
- **Deplacement DTO (A)** : verifier que `http.rs` compile apres import (visibilite, `project_shard_session`
  retourne le type core), que la reponse HTTP serialise a l'identique (DTO garde `Serialize` + memes
  champs/ordre), et que le test http.rs:5294 passe. Surface publique de `nexus-core-rs` elargie de 2
  types — justifie : `ShardSessionView` est la projection whitelisted de `ShardedSessionManifest`
  (deja dans core).
- **ComputeGroup (B)** : verifier que deriver sur `ComputeGroup` ne tire pas accidentellement l'`Entry`.
- **Drift cross-toolchain** : regenerer snapshots et confirmer `cargo nextest -p nexus-core-rs` vert
  sous Windows ET Docker rust:1.94 (pretty-print serde stable, comme `task_response.schema.json`).
- **`serde_json::Value` libre** : aucun type shard en scope n'en a (verifie) -> schema strict,
  drift-test verrouille la forme.

## Gate d'acceptation (drift-test + parse + const-check doc<->code)
1. **drift-test** : `cargo nextest run -p nexus-core-rs` -> `shard_schema_snapshot_matches_struct`
   FAIL loud si un `.schema.json` commite != `schema_for!(T)` regenere (assert_eq! sur
   `serde_json::Value`, miroir l.304-307).
2. **parse + required-fields** : chaque schema est un objet JSON valide draft 2020-12 publiant le
   `required` array attendu par type.
3. **const-check doc<->code** : `spec_consts_exist` verifie que chaque `DOMAIN_*_V1` (5) + chaque cap
   citee (`MAX_SHARD_FRAME_BYTES`, `MAX_SHARD_N_CTX`, ...) presente dans `SHARD_PROTOCOL_SPEC.md`
   existe comme const Rust.
