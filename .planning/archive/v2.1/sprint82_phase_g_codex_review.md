Verdict global : 9 livrables confirmés, 1 partiel. Le code, les snapshots et les gates sont verts ; l’unique défaut est documentaire dans `PATTERNS.md`.

### Livrable 1 : déplacement des request DTOs

- Statut : CONFIRMÉ
- Fichier(s) : [shard.rs:225](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/shard.rs:225), [schemas/mod.rs:45](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/mod.rs:45), [lib.rs:182](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/lib.rs:182), [http.rs:2144](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:2144), [http.rs:2316](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:2316)
- Evidence :

  ```text
  238:#[derive(Debug, Deserialize, JsonSchema)]
  239:pub struct ShardGroupMintRequest {
  246:    #[serde(default)]
  259:#[derive(Debug, Deserialize, JsonSchema)]
  260:pub struct ShardGenerateRequest {
  ```

`git grep` trouve les anciennes définitions dans `21674f5:http.rs` aux lignes 2195 et 2308 ; dans le working tree, les deux seules définitions sont désormais dans `shard.rs`. Les trois defaults `revision`, `session_id`, `max_tokens` sont conservés sans `#[schemars(required)]`. Les re-exports et imports daemon sont présents. Le handler conserve le rejet `400` lorsque `body.session_id != path session_id` aux lignes 2316-2325.

### Livrable 2 : fonctions, registre et snapshots

- Statut : CONFIRMÉ
- Fichier(s) : [shard.rs:330](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/shard.rs:330), [shard.rs:348](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/shard.rs:348), [mint snapshot:16](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/shard_group_mint_request.schema.json:16), [generate snapshot:19](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/shard_generate_request.schema.json:19), [shard.rs:629](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/shard.rs:629)
- Evidence :

  ```text
  332:pub fn shard_group_mint_request_schema() -> serde_json::Value {
  333:    to_value(schema_for!(ShardGroupMintRequest))
  338:pub fn shard_generate_request_schema() -> serde_json::Value {
  339:    to_value(schema_for!(ShardGenerateRequest))
  ```

Les entrées sont enregistrées aux lignes 378-385. Les snapshots, inclus comme fichiers untracked dans le diff de phase, ont respectivement `required=["group_id","members"]` et `required=["prompt"]`. Les trois champs optionnels sont nullables et absents de `required`. Le drift-test compare réellement chaque snapshot au schéma généré aux lignes 640-662.

### Livrable 3 : assertions du contrat inverse

- Statut : CONFIRMÉ
- Fichier(s) : [shard.rs:525](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/shard.rs:525)
- Evidence :

  ```text
  530:let mint = required(&shard_group_mint_request_schema());
  531:for key in ["group_id", "members"] {
  538:    !mint.contains(&"revision".to_string()),
  543:    generate.contains(&"prompt".to_string()),
  546:for optional in ["session_id", "max_tokens"] {
  ```

Les assertions testent bien les inclusions et exclusions demandées ; elles ne sont ni vacantes ni limitées à l’existence du schéma. Le test ciblé a passé.

### Livrable 4 : `MountSessionRequest` non schématisée

- Statut : CONFIRMÉ
- Fichier(s) : [shard_session.rs:210](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:210), [compute_group.rs:160](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/compute_group.rs:160)
- Evidence :

  ```text
  219:// embeds the signed `ComputeGroupEntry` ENVELOPE — the exact class the
  220:// schema doctrine excludes (`[u8; 64]` signature via `serde_big_array`) —
  221:// and `ShardWorkerSpec.addr` is `iroh::EndpointAddr`, an upstream type with
  222:// no `JsonSchema` impl whose JSON shape iroh owns
  224:// Request-body table in `docs/protocol/SHARD_PROTOCOL_SPEC.md` §6.1,
  ```

La derive reste `Debug, Clone, Deserialize` sans `JsonSchema` à la ligne 227. Aucun `schema_for!(MountSessionRequest)` n’existe. Le scan des tags ne trouve qu’un vrai tag registre, `ShardPlan` ; `MountSessionRequest` n’en porte pas.

### Livrable 5 : spécification protocolaire

- Statut : CONFIRMÉ
- Fichier(s) : [SHARD_PROTOCOL_SPEC.md:91](/C:/Users/FlowUP/Documents/Code/nexus/docs/protocol/SHARD_PROTOCOL_SPEC.md:91), [SHARD_PROTOCOL_SPEC.md:112](/C:/Users/FlowUP/Documents/Code/nexus/docs/protocol/SHARD_PROTOCOL_SPEC.md:112), [SHARD_PROTOCOL_SPEC.md:303](/C:/Users/FlowUP/Documents/Code/nexus/docs/protocol/SHARD_PROTOCOL_SPEC.md:303)
- Evidence :

  ```text
  303:### 6.1 Request bodies (Sprint 82 Phase G)
  309:**`POST /api/daemon/shard-session/group`** — `ShardGroupMintRequest`
  319:**`POST /api/daemon/shard-session/mount`** — `MountSessionRequest`
  331:**`POST /api/daemon/shard-session/{id}/generate`** —
  341:**The PATH is authoritative** (a runtime contract no JSON Schema can
  ```

La table §3 contient exactement 12 lignes de types aux lignes 93-104. Les deux résultats sont correctement annotés `S81 I`, et les deux requests `S82 G`. Les trois tables §6.1 reproduisent tous les champs, types, optionalités et defaults des structs. La note `PATH` documente explicitement le `400`.

### Livrable 6 : références dans `WIRING_SPEC`

- Statut : CONFIRMÉ
- Fichier(s) : [WIRING_SPEC.md:179](/C:/Users/FlowUP/Documents/Code/nexus/docs/sharding/WIRING_SPEC.md:179)
- Evidence :

  ```text
  179:- **Request bodies (S82 G)** — the three POST bodies are documented
  181:  `crates/nexus-core-rs/src/schemas/shard.rs:ShardGroupMintRequest`
  182:  ...`crates/nexus-core-rs/src/schemas/shard.rs:ShardGenerateRequest`
  185:  `ComputeGroupEntry` envelope + `iroh::EndpointAddr`, SPEC §3/§6.1):
  186:  `crates/nexus-shell-daemon/src/shard_session.rs:MountSessionRequest`.
  ```

Les trois références `path:Symbol` sont résolues par le source-ref gate.

### Livrable 7 : anchors du gate sharding

- Statut : CONFIRMÉ
- Fichier(s) : [check-sharding-docs.sh:92](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-sharding-docs.sh:92), [check-sharding-docs.sh:208](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-sharding-docs.sh:208)
- Evidence :

  ```text
  95:anchor_present "docs/protocol/SHARD_PROTOCOL_SPEC.md" "ShardGroupMintRequest"
  96:anchor_present "docs/protocol/SHARD_PROTOCOL_SPEC.md" "MountSessionRequest"
  97:anchor_present "docs/protocol/SHARD_PROTOCOL_SPEC.md" "ShardGenerateRequest"
  214:REQUIRED_ANCHORS="is_pipeline_contiguous ...
  217:ShardGroupMintRequest MountSessionRequest ShardGenerateRequest"
  ```

Les trois anchors simples et les trois anchors obligatoires sont effectifs. Le script passe sous l’environnement local et dans l’image `bash:5` utilisant les utilitaires BusyBox.

### Livrable 8 : census FRONTIER figé

- Statut : CONFIRMÉ
- Fichier(s) : [check-frontier-contracts.sh:37](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:37), [check-frontier-contracts.sh:180](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:180), [check-frontier-contracts.sh:277](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:277)
- Evidence :

  ```text
  190:DOMAIN_CENSUS_FROZEN=25
  191:domain_census="$({ find crates ... \
  192:  -exec grep -hoE 'const DOMAIN_[A-Z0-9_]+_V[0-9]+' {} + ... || true; }
  196:  echo "     then refresh ... (header (2), comment (2b))"
  197:  echo "     + docs/rust/PATTERNS.md §P70."
  ```

Le header décrit désormais l’`accept-and-close`, le guard `|| true` est présent et le message désigne les surfaces à rafraîchir. La sortie finale contient bien `DOMAIN census [$domain_census frozen]`. Le recensement indépendant retourne 25.

### Livrable 9 : doctrine §P70 et clôture S80-G-1

- Statut : PARTIEL
- Fichier(s) : [PATTERNS.md:3944](/C:/Users/FlowUP/Documents/Code/nexus/docs/rust/PATTERNS.md:3944), [PATTERNS.md:3969](/C:/Users/FlowUP/Documents/Code/nexus/docs/rust/PATTERNS.md:3969), [PATTERNS.md:3982](/C:/Users/FlowUP/Documents/Code/nexus/docs/rust/PATTERNS.md:3982), [shard.rs:130](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/shard.rs:130)
- Evidence :

  ```text
  3953:= **25** distinct const families across all crates
  3955:...of which **3** carry a generated schema
  3956:...RUN_PROOF) → **22**
  3973:`ShardSessionResultView` S81 K; joined S82 G...
  3982:**Doc-lint stays existence-only by design (S80-G-1, CLOSED).**
  ```

Le census 25/3/22, la borne honnête net-zero/rename, la convention DTO sans tag et la clôture « 3rd and FINAL mention » sont présents.

Écarts :

- `PATTERNS.md:3973` date le précédent `ShardSessionResultView` de `S81 K`. C’est factuellement faux : le code indique `Sprint 81 Phase I` à la ligne 130, `git log -S` attribue son introduction au commit `bb6c4f9` Phase I, et la SPEC le classe correctement `S81 I`.
- [PATTERNS.md:3891](/C:/Users/FlowUP/Documents/Code/nexus/docs/rust/PATTERNS.md:3891) conserve le compteur périmé « 8 sharding types + TaskResponse », alors que `schema_snapshots()` contient désormais 12 schémas sharding, soit 13 fichiers avec `TaskResponse`.

Corrections factuelles requises : `S81 K → S81 I` et `8 → 12 sharding types`.

### Livrable 10 : invariants transverses

- Statut : CONFIRMÉ
- Fichier(s) : [shard.rs:238](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/shard.rs:238), [http.rs:2316](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:2316), [check-frontier-contracts.sh:89](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:89)
- Evidence :

  ```text
  2316:if let Some(body_id) = &req.session_id
  2317:    && body_id != &session_id
  2319:    return (
  2320:        StatusCode::BAD_REQUEST,
  2322:        "error": "body session_id disagrees with the path session id"
  ```

Le diff ne touche aucun `Cargo.toml`, `Cargo.lock`, `canonical.rs`, `compute_group.rs` ou `shard_plan.rs`. Aucun attribut `#[test]`/`#[tokio::test]` n’est ajouté. Les DTOs restent `Deserialize`-only. Le scan du texte ajouté avec le `PROMISE_RE` exact retourne zéro correspondance, snapshots compris.

### Vérifications exécutées

- `cargo nextest run -p nexus-core-rs schemas::shard::tests --locked` : 6/6 PASS.
- `cargo check -p nexus-shell-daemon --locked` : PASS.
- `cargo fmt --all --check` et `git diff --check 21674f5` : PASS.
- `bash scripts/check-frontier-contracts.sh` : PASS, census `25 frozen`.
- `bash scripts/check-sharding-docs.sh` : PASS.
- Les deux scripts sous Docker `bash:5`/BusyBox : PASS.
- HEAD final inchangé : `master` sur `21674f59172e26a57bb96161d83d1370533c9856`.

## Résumé final

- Total livrables : 10
- Confirmés : 9
- Gaps : 0
- Partiels : 1

La phase n’est donc pas strictement totalement conforme avant correction des deux affirmations de `PATTERNS.md`; aucun défaut fonctionnel ou de gate n’a été trouvé.

