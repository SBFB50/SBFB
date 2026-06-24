Préambule : checkout sur `master`, mais Phase M n’est pas dans `HEAD` : `docs/sharding/` et `scripts/check-sharding-docs.sh` sont non trackés, et les 3 fichiers CI sont modifiés. Les statuts ci-dessous portent donc sur le worktree réel audité, pas sur un commit déjà intégré.

### Livrable 1 : `docs/sharding/README.md`
- Statut : CONFIRME
- Fichier(s) : `docs/sharding/README.md:14`, `docs/sharding/README.md:31`, `docs/sharding/README.md:51`, `docs/sharding/README.md:67`
- Evidence : README porte bien `PROVISIONAL`, `RIG-ABSENT`, carry S78, caveat “admission ≠ confidentialité”, table Diátaxis 4 quadrants, liens spec/schémas/preuves.
- Côté code : `crates/nexus-core-rs/src/compute_group.rs:144-151` confirme que l’admission est seulement `is_member`; `crates/nexus-core-rs/src/shard.rs:300-307` rejette le non-membre avant frame; `docs/security/THREAT_MODEL.md:1012-1037` confirme activations en clair et admission ≠ confidentialité.
- Preuves-de-vie non stubs : `scripts/acceptance/b3_shard_pipeline.sh:13-31` définit PASS/BLOCK/RIG-ABSENT; `web/e2e/compute-shard.spec.ts:26-44` teste le panneau `/compute` hermétique.

### Livrable 2 : `docs/sharding/EXPLANATION.md`
- Statut : CONFIRME
- Fichier(s) : `docs/sharding/EXPLANATION.md:20`, `docs/sharding/EXPLANATION.md:45`, `docs/sharding/EXPLANATION.md:72`
- Evidence : doc dit pipeline-parallel, pas tensor-parallel; blocs `[layer_start, layer_end)`; `is_pipeline_contiguous` = contiguïté structurelle seulement; couverture `[0,L)` séparée par `covers_full_model`.
- Côté code : `crates/nexus-core-rs/src/shard_plan.rs:201-212` confirme que `is_pipeline_contiguous` ne vérifie pas le départ à 0 ni la fin à `total_layers`; `crates/nexus-coordinator-rs/src/placement.rs:294-305` confirme `covers_full_model`.
- Signatures et honnêteté : `crates/nexus-core-rs/src/canonical.rs:276`, `:290`, `:310`, `:332` confirment les DOMAIN tags; `crates/nexus-core-rs/src/shard_plan.rs:374-387` confirme no-floats dans `RunMetrics`; `docs/security/THREAT_MODEL.md:1073-1105` confirme `RunProof` in-vivo carry S78 et live path exact-match `result_text`.

### Livrable 3 : `docs/sharding/HOW_TO_WIRE.md`
- Statut : CONFIRME
- Fichier(s) : `docs/sharding/HOW_TO_WIRE.md:10`, `:31`, `:38`, `:63`
- Evidence : rôles START/JOIN/OBSERVE présents; bannière honnête “pas de store live”, orchestrateur carry S78, route `GET /api/daemon/shard-session/{id}`, stub `{found:false, session:null}`, helper front dans `web/src/api/daemon.ts`.
- Côté code : `crates/nexus-shell-daemon/src/http.rs:2107-2134` retourne `None` puis `{ found:false, session: None }`; `:2137-2149` expose la route; tests `:5208-5228` pin l’enveloppe vide et `:5231-5292` pin le whitelist `session_id/member_count`.
- Front/bridge : `web/src/api/daemon.ts:531-582` contient `ShardSessionViewSchema` + `getShardSession`; `web/src/lib/daemon.ts` est absent; `web/public/sbfb-bridge.js` n’a aucun match `shard`, et ses `_call(...)` publics listés `:170-388` ne contiennent pas de méthode shard.
- Llama/GGUF : `crates/nexus-worker-core/src/llm/shard.rs:288-301` fail-close si `general.architecture != "llama"`; `crates/nexus-core-rs/src/shard_plan.rs:262-272` signe `model_digest/tokenizer_hash/chat_template_hash`.

### Livrable 4 : `docs/sharding/REFERENCE.md`
- Statut : CONFIRME
- Fichier(s) : `docs/sharding/REFERENCE.md:12`, `:34`, `:50`, `:75`, `:95`
- Evidence : single-source-of-truth clair; corps anglais assumé; tables types/domain/caps/seuils présentes; seuils marqués `S78-pending tuning`.
- Types/domaines : `crates/nexus-core-rs/src/compute_group.rs:69-118`, `crates/nexus-core-rs/src/shard_plan.rs:88-108`, `:144-179`, `:235-272`, `:374-455` confirment champs/caps; `crates/nexus-core-rs/src/canonical.rs:258`, `:276`, `:290`, `:310`, `:332` confirment les 5 tags.
- Schémas/spec : `docs/protocol/SHARD_PROTOCOL_SPEC.md:55-83`, `:109-158`, `:197-222` concordent; drift tests `crates/nexus-core-rs/src/schemas/shard.rs:314-341` et const/spec test `:346-410` empêchent la dérive.
- Seuils : `toploc.rs:64-81`, `verification.rs:280-317`, `sentinel.rs:47-59` confirment TOPLOC, spot-check et SENTINEL.

### Livrable 5 : `scripts/check-sharding-docs.sh`
- Statut : CONFIRME
- Fichier(s) : `scripts/check-sharding-docs.sh:35`, `:51`, `:69`, `:88`, `:106`, `:115`
- Evidence : les 4 docs sont obligatoires (`:35-45`), liens relatifs vérifiés (`:51-65`), ancres exigées (`:69-85`), honesty markers (`:88-99`), scan EN sur docs FR (`:106-113`), sortie `exit 1` si `fail != 0` (`:115-117`).
- Portabilité : pas de `grep -P`, pas de `--include`; le script utilise `grep -oE/-qF/-qE`. Il nécessite Bash, mais les 3 surfaces CI l’appellent avec `bash`.
- Exécution : `C:\Program Files\Git\usr\bin\bash.exe --noprofile --norc ... scripts/check-sharding-docs.sh` => `check-sharding-docs: clean`.
- Probes négatifs en copie minimale : baseline `BASE_EXIT=0`; lien cassé `LINK_EXIT=1`; retrait `PROVISIONAL` `HONESTY_EXIT=1`; injection `Welcome` `FRENCH_EXIT=1`.

### Livrable 6 : câblage CI
- Statut : CONFIRME
- Fichier(s) : `.github/workflows/ci.yml:116`, `.woodpecker/ci-linux.yml:74`, `scripts/verify.sh:19`, `scripts/verify.sh:108`
- Evidence : GitHub step `[14] sharding docs check` exécute `bash scripts/check-sharding-docs.sh` sans `|| true` ni `continue-on-error`; Woodpecker step `sharding-docs-check` exécute le même script; `scripts/verify.sh` a `set -euo pipefail` puis step 19 appelle le gate.
- Bloquant : confirmé par structure fail-fast et par le script lui-même qui sort `1` sur violation.

### Résumé final
- Total livrables : 6
- Confirmés : 6
- Gaps : 0
- Partiels : 0

Caveat versionnement : si “sur master” signifie “déjà commité dans `HEAD`”, alors ce n’est pas vrai pour Phase M : les docs et le script sont encore non trackés, et le câblage CI est en modifications locales.

