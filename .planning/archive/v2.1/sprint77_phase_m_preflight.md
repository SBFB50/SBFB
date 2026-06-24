# Sprint 77 Phase M — Preflight G8 (docs humaines Diataxis + doc-lint)

**VERDICT : PLAN-ADAPT** — Le coeur est EXECUTE (plan cohérent Diataxis + style maison, tous faits HOW_TO_WIRE confirmés contre code réel, 0 REFUTED, 0 DESIGN-CONFLICT). 3 corrections opérationnelles code-ancrées sont OBLIGATOIRES (scan-en-strings non réutilisable verbatim ; portabilité BusyBox Woodpecker ; chemin daemon.ts). Suivre ce document verbatim.

---

## Synthèse des 5 scans

### S1a — Précédent style Diataxis + maison
- **Diataxis = principe organisateur** : hub README labellisé « hub Diataxis + table des 4 quadrants » (plan:771). Phase M livre le hub + 3 quadrants (Explanation=EXPLANATION.md, How-to=HOW_TO_WIRE.md, Reference=REFERENCE.md) ; le 4e (TUTORIAL) est DIFFÉRÉ S78 (plan:715) → **SCOPE-CUT-CONSISTENT** : le tutorial est le quadrant le plus couplé à un happy-path runnable, exactement ce qui est RIG-ABSENT. Preuves-de-vie de substitution EXISTENT : `scripts/acceptance/b3_shard_pipeline.sh` + `web/e2e/compute-shard.spec.ts`.
- **Style maison REFERENCE à mirrorer** (depuis docs/protocol/SHARD_PROTOCOL_SPEC.md) : banner H1 + Status/Versioning-regime (:1-14), régime pre-v1.0 raw-op additif + flip post-v1.0 (:8-14), table domain-tag Family/tag/Signer (:55-61), table « Signed payload? » par type (:75-84), no-floats/units convention (:92-98), section-par-type ### 4.1-4.7, table « Caps & limits (named constants) » Constant/Value/Bounds.
- **Précédent doc FR** : docs/protocol/SBFB_JSON_V2.md (headers FR + table Champ/Type/Description, :11/:30/:33-37). **Précédent JSON-example** : PUBLIC_FEED_SPEC.md:40-50.
- **docs/sharding/ n'existe PAS** (greenfield, vérifié).

### S1b — Tooling doc-lint + portabilité CI
- **Pattern à mirrorer (honesty-gate)** : `scripts/scan-trust-wording.sh` (multi-règle grep + compteur VIOLATIONS + echo par règle + exit unique).
- **Pattern à mirrorer (link/anchor + PORTABLE)** : `scripts/check-spdx.sh` — `#!/usr/bin/env bash` + `set -euo pipefail` + SCRIPT_DIR/REPO_ROOT via `BASH_SOURCE` (:13-16) + `find ... -name '*.md'` + boucle `while IFS= read -r` + `head -n N | tr -d '\r' | grep -qF` (:23-36) + tableau `missing[]` + exit 1. **N'utilise QUE grep -qF (PAS -P, PAS --include) → tourne sur Woodpecker bash:5 BusyBox.**
- **PIÈGE scan-en-strings** : `web/scripts/scan-en-strings.sh` fait `cd web/` (:20) puis grep `src` avec `--include='*.tsx' --include='*.ts'` (:28) — AUCUN .md, AUCUN chemin docs → réutilisé tel quel = **faux-vert** (clean car rien matché). Régex EN_WORDS à répliquer : `\b(Welcome|Dashboard|Sign in|Log in|Sign up|Please|Click here|Coming soon|Loading...)` (:26).
- **Câblage CI (3 points)** : (1) `.github/workflows/ci.yml` step après `[13] SPDX check` (:113-114 `run: bash scripts/check-spdx.sh`) ; (2) `.woodpecker/ci-linux.yml` step après `spdx-check` (:69-72, image bash:5 → idiomes portables OU préférer rust:1.94 déjà épinglé) ; (3) `scripts/verify.sh` à côté des steps 16/18. **shellcheck.yml : 0 edit** (path-filter `scripts/**/*.sh` auto-couvre → le script doit passer `shellcheck --severity=warning`, non installé localement, écrire propre en mirrorant check-spdx.sh).
- **Cibles link-check toutes présentes** : SHARD_PROTOCOL_SPEC.md, THREAT_MODEL §16 (@1010), PATTERNS §P39 (@2181), §P64-P69 (@3552/3605/3672/3748/3785/3812), 8 schemas/*.schema.json. `docs/sharding/` + `scripts/check-sharding-docs.sh` net-new (vérifié absents).

### S2 — Décisions PO traversées
- Les 5 décisions PO (plan:714-726) restent COHÉRENTES avec le code. #1 TUTORIAL différé S78 (preuves-de-vie présentes). #5 seuils dans REFERENCE = valeurs réelles + marqueur « S78-pending tuning ».
- **Constantes réelles citables (décision #5)** : TOPLOC `TOP_K=128`/`EXP_MISMATCH=38`/`MANT_MEAN=10`/`MANT_MEDIAN=8` (toploc.rs:66-81) ; SENTINEL `ALPHA_BP=9000`/`DEVIATION_THRESH_BP=5000`/`BP_DENOMINATOR=10000` (sentinel.rs:48-59) ; SPOT_CHECK `TRUSTED=100bp`/`STANDARD=500bp`/`SUSPECT=2000bp`/`DENOM=10000` (verification.rs:285-291).
- **Politique langue** (CLAUDE.md:568-574) : FR pour README/EXPLANATION/HOW_TO_WIRE + EN-body REFERENCE (matériel de référence agent/contributeur externe) — exactement le split plan:771-786.
- **Watch-out** : commentaires in-code « Phase K » périmés (toploc.rs:72, http.rs:2111) — les docs disent **S78**, jamais « Phase K ».

### S3 — Invariants menace (honnêteté)
- **Phrase caveat admission≠confidentialité VERBATIM** (THREAT_MODEL.md:1036-1044) — à citer en gras dans README+EXPLANATION+HOW_TO_WIRE.
- **Échelle N0-N3** (1-line chacune) : N0 TOPLOC = détecteur de SWAP modèle/quant par inégalité d'un commitment BLAKE3 (1066/1083) ; N1 VRF spot-check = tirage Ed25519 déterministe (PAS ECVRF) re-rejoue ~1% prefill, mitigation one-honest-verifier (1119/1132) ; N2 quorum tolérant = CLIQUE mutual-agreement sur RunProof SIGNÉ, quorum exact result_text INCHANGÉ (1189/1212) ; N3 commit-reveal opML-style + SENTINEL EMA forward-only O(1), escalade de litige seulement, PAS soundness fraud-proof (1233/1244). **N4 zkML hors-scope S77** (1138/1247/1272).
- **TRAPS overstatement (interdits)** : ne pas dire « le réseau vérifie chaque shard » (primitives câblées+testées, émission in-vivo = carry S78, live path = quorum exact INCHANGÉ, 1075-1078/1104-1105/1140-1144) ; ne pas dire « groupe privé ⇒ données protégées » (activations en clair, SI-1/SI-4 High résiduel ASSUME) ; ne pas dire « VRF garantit » (pas ECVRF) ; ne pas dire « bisection O(1) » ; incentive = kudos non-monétaire, 0 anti-lazy-verifier (1173-1187) ; SI-5 padding = carry S78 (1288-1292).
- **Anchors à LINKER, jamais dupliquer** : THREAT §16 (@1010) + §5.9 (@245) ; PATTERNS §P64-P69. **§P39 N'EST PAS un anchor sharding-sécurité** (= DaemonHttpState DB singleton, @2181) — ne pas le citer comme source d'échelle de vérification (le plan:791 le liste pour link-check ; le citer SEULEMENT comme hôte de la route read-only, pas comme source N0-N3).
- **Banner PROVISIONAL/RIG-ABSENT** (sprint77_verification.md:126-133/163) : feature shard = PROVISIONAL, T2 RIG-ABSENT carry P1 S78, cause STRUCTURELLE (pas d'orchestrateur session in-vivo + rig 2-machines absent), status machine-lisible {PASS|BLOCK{diag}|RIG-ABSENT}, jamais prose DIFFERE-materiel.

### S4 — Vérité terrain wire (HOW_TO_WIRE/REFERENCE)
- **Route OBSERVE** : `GET /api/daemon/shard-session/{id}` retourne un STUB — `live_shard_session()` retourne `None` inconditionnellement (http.rs:2115-2117) → 200 `{found:false, session:null}` (PAS 404) pour tout id. Registre = carry S78. **NE PAS écrire « observer une session active » au présent.**
- **Whitelist** : `ShardSessionView` = exactement 2 champs `session_id` + `member_count` (schemas/shard.rs:72-79) ; `member_count = plan.assignments.len()` AGRÉGAT (http.rs:2100-2105) ; JAMAIS worker_pubkey/initiator (SI-3/SI-4).
- **Bridge** : `web/public/sbfb-bridge.js` = AUCUNE méthode shard (3 whitelist task_submit/storage_get/storage_set). Entrée = panel shell `/compute`, pas appel bridge. Nom/forme méthode bridge figés S78 (décision PO #4) — marquer PROPOSED, ne rien inventer.
- **Contrainte** : llama-arch only (refus arch≠"llama" au backend llm/shard.rs:296, documenté au claim gate shard_claim.rs:28) + même-GGUF homogène. **PAS** de symbole `required_runtime`/`RuntimeTuple` dans shard_claim/shard_plan (= mécanisme cohorte GPU S76 ailleurs).
- **Chemin front** : `web/src/api/daemon.ts` (getShardSession ~:575) — **`web/src/lib/daemon.ts` N'EXISTE PAS** (corriger le plan).

---

## Faits ancrés OBLIGATOIRES pour le coder

### A. Phrase caveat admission≠confidentialité (à citer VERBATIM, en gras, dans README + EXPLANATION + HOW_TO_WIRE)
> L'admission `ComputeGroup` (allowlist Ed25519 signée, `compute_group.rs`) est un contrôle d'**ADMISSION** (qui peut participer), **PAS** de la confidentialité des activations : celles-ci circulent **en clair** (aucun TEE GPU grand public en 2026, scope cut #4) et l'allowlist ne garantit pas une majorité honnête (SI-4 résiduel). En conséquence : **aucun secret applicatif ne doit transiter par les prompts d'une session shardée** — un membre admis mais curieux voit les activations de son segment. Le sharding sert à exécuter un GROS modèle public éclaté, pas à traiter des entrées confidentielles.

Source : `docs/security/THREAT_MODEL.md:1036-1044`.

### B. Table type REFERENCE.md (name | type | units | signed? | DOMAIN | cap) — copier verbatim des consts
| Type | Champs clés (type Rust) | signed? | DOMAIN tag | cap |
|---|---|---|---|---|
| ComputeGroup | group_id String, members Vec<[u8;32]> | **oui** | `nexus-compute-group-v1` | id COMPUTE_GROUP_ID_MAX=128, members COMPUTE_GROUP_MAX_MEMBERS=256 |
| ShardAssignment | layer_start/layer_end u32 (demi-ouvert), role ShardRole=layer_worker, shard_hashes Vec<[u8;32]>, kv_cache_policy=local_ephemeral, fallback_node Option<[u8;32]> serde-default, launch_profile_hash [u8;32] | **non** (dans manifest) | — | shard_hashes SHARD_HASHES_MAX=64 |
| ShardPlan | assignments Vec<ShardAssignment> | **non** (dans manifest) | — | SHARD_PLAN_MAX_ASSIGNMENTS=256 |
| ShardedSessionManifest | version u16, session_id String, group_id String, revision u64, model_digest/tokenizer_hash/chat_template_hash [u8;32], plan ShardPlan | **oui** | `nexus-shard-plan-v1` | session_id SESSION_ID_MAX=128, group_id **SHARD_GROUP_ID_MAX=128** |
| RunMetrics | ttft_ms u64, decode_milli_tokens_per_sec u64 (**tok/s ×1000, ENTIER**), p95_token_latency_ms u64, network_rx_bytes/tx_bytes u64, worker_drop_count u32 | **non** (dans RunProof) | — | — |
| RunProof | session_id String, activation_fingerprint [u8;32] serde-default (N0 TOPLOC), model_digest [u8;32], metrics RunMetrics, participants Vec | **oui** | `nexus-run-proof-v1` | session_id SESSION_ID_MAX=128, participants RUN_PROOF_MAX_PARTICIPANTS=256 |
| ShardSessionView | session_id String, member_count usize | non (DTO observé) | — | — |
| ShardSessionStatusResponse | found bool, session Option<ShardSessionView> | non (DTO observé) | — | — |

Frame ALPN `sbfb/shard/1` : length-prefixed big-endian (header u32 4-octets), `MAX_SHARD_FRAME_BYTES=256 MiB` (256*1024*1024) enforced write ET read, `MAX_SHARD_N_CTX=8192` (policy const, JAMAIS sérialisé wire), `is_member` crypto-AVANT-`accept_bi`, close code SHARD_REJECT_NOT_MEMBER=1. **5 DOMAIN tags on-wire exacts** : nexus-compute-group-v1 / nexus-shard-plan-v1 / nexus-run-proof-v1 / nexus-vrf-draw-v1 / nexus-activation-commit-v1. **Caps enforced BOTH sign AND verify** (cap-check AVANT crypto). **Ne pas confondre SHARD_GROUP_ID_MAX (manifest) et COMPUTE_GROUP_ID_MAX (ComputeGroup), deux consts distinctes valeur 128.** Enums fermés snake_case 1 variante : ShardRole::LayerWorker, KvCachePolicy::LocalEphemeral. REFERENCE doit énoncer la relation **single-source-of-truth → SHARD_PROTOCOL_SPEC.md (et via lui les structs Rust)**, ne jamais redéfinir une forme.

### C. Claims HOW_TO_WIRE vérifiées (toutes TRUE dans l'arbre actuel)
- (a) `GET /api/daemon/shard-session/{id}` = STUB, retourne `{found:false,session:null}` 200 (http.rs:2115-2117/2147-2150). OBSERVE expose `member_count` agrégat seulement.
- (b) `sbfb-bridge.js` = 0 méthode shard ; entrée = panel `/compute`, pas bridge.
- (c) llama-arch only + même-GGUF homogène (SHARD_PROTOCOL_SPEC.md:192-193 ; llm/shard.rs:296 backend ; shard_claim.rs:28 claim gate). Ne pas nommer une fonction d'enforcement précise (assess_capacity ne check PAS l'arch).
- START via `/compute` « Lancer un gros modèle en réseau » = texte explicatif seul aujourd'hui ; JOIN « Rejoindre un groupe de calcul » = lookup read-only id hors-bande ; bannière honnête : pas de store live, **orchestrateur = carry S78**.
- Chemin front si cité = `web/src/api/daemon.ts` (PAS lib/).

### D. Anchors à LINKER (jamais dupliquer le contenu)
- THREAT_MODEL §16 « Surface sharding inference » (@1010) + §5.9 (@245) — catalogue SI-1..SI-11 + N0-N3 + incentive.
- PATTERNS rust §P64(@3552)/§P65(@3605)/§P66(@3672)/§P67(@3748)/§P68(@3785)/§P69(@3812). PATTERNS shell §P39(@2181) = DB singleton (PAS sécurité sharding ; n'apparaît que pour link-check route read-only).

### E. Script check-sharding-docs.sh — pattern + câblage
- **Modèle** : `scripts/check-spdx.sh` pour link/anchor (portable BusyBox) + `scripts/scan-trust-wording.sh` pour honesty-gate multi-règle. Header `#!/usr/bin/env bash` + `set -euo pipefail` + SCRIPT_DIR/REPO_ROOT via `BASH_SOURCE`.
- **Idiomes PORTABLES OBLIGATOIRES** (Woodpecker bash:5 BusyBox) : `find docs/sharding -name '*.md'` + boucle `while IFS= read -r` + `head | tr -d '\r' | grep -qF` / `grep -E`. **INTERDIT** : `grep -P`, `--include`, `--exclude-dir`. (Alternative : faire tourner la step Woodpecker sur image rust:1.94 déjà épinglée.)
- **3 devoirs (plan:790-794)** :
  1. **link-check** — chaque lien repo-relatif + ancre §-citée (THREAT §16, PATTERNS §P64-69/§P39, routes, sources) résout. Vérifier FICHIER + ancre (grep de la section, pas juste existence fichier).
  2. **honesty-gate** — grep que README+EXPLANATION+HOW_TO_WIRE contiennent le marqueur `PROVISIONAL` + la phrase caveat `admission ≠ confidentialité` (forme FR, accentuée), et que HOW_TO_WIRE contient `S78` sur le bloc orchestrateur.
  3. **scan-en-strings FR** — répliquer le régex EN_WORDS DANS le script contre docs/sharding/README.md + EXPLANATION.md + HOW_TO_WIRE.md SEULEMENT ; EXEMPTER REFERENCE.md (EN-body intentionnel).
- **Câblage 3 points** : ci.yml (après step [13], :113-114) ; ci-linux.yml (après spdx-check, :69-72) ; verify.sh (à côté steps 16/18). shellcheck.yml = 0 edit (auto-couvert, doit passer --severity=warning).

### F. Marqueurs honnêteté grep (que le gate vérifie + que les docs doivent contenir)
- `PROVISIONAL` (banner README).
- caveat `admission ≠ confidentialité` en gras (README+EXPLANATION+HOW_TO_WIRE).
- `S78` sur le bloc orchestrateur de HOW_TO_WIRE (jamais « Phase K »).
- REFERENCE : seuils + marqueur `S78-pending tuning`.

---

## Rationale verdict + adaptations
**PLAN-ADAPT** (pas pure EXECUTE) car le plan contient 3 hypothèses opérationnelles fausses vis-à-vis du code, chacune avec preuve concrète : (1) « scan-en-strings réutilisé » (plan:794) donnerait un faux-vert car le script est codé en dur sur web/src/*.ts (web/scripts/scan-en-strings.sh:20,28) ; (2) « en CI » non pinné alors que la step Woodpecker tourne sur bash:5 BusyBox (.woodpecker/ci-linux.yml:70) qui ne supporte pas garantiment grep -P/--include ; (3) le chemin front cité dans la tâche/risques est web/src/lib/daemon.ts qui n'existe pas (réel = web/src/api/daemon.ts). Aucune de ces corrections ne touche une décision Day-0 gelée. **Pas de DESIGN-CONFLICT** : tous les invariants menace (admission≠confidentialité, N0-N3 primitives-pas-enforced-live, VRF≠ECVRF, kudos non-monétaire, SI-5 carry) sont citables verbatim depuis THREAT §16, et le code fait littéralement ce que les docs doivent affirmer (stub None, whitelist 2 champs, 0 méthode bridge shard). Le quadrant TUTORIAL différé S78 reste **SCOPE-CUT-CONSISTENT**. Suivre les `plan_adaptations` ci-dessus verbatim.
