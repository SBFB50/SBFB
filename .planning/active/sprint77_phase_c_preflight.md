# Sprint 77 — Preflight Phase C (G8) : primitives wire shard + RunProof

## Verdict: PLAN-ADAPT

Le plan §6 est exécutable tel quel sur sa structure (4 primitives signées Ed25519+JCS, 2 DOMAIN_* additifs, 0 bump wire) — c'est un mirror direct du patron `compute_group.rs` (Phase B), aucune Day-0 contredite, aucun pivot. Le verdict est **PLAN-ADAPT** et non EXECUTE pour une seule raison de conformité repo : le draft RunProof (`remote_user_sharded_llm_rnd.md` §10.4) porte des **métriques flottantes** (`decode_tokens_per_sec`, `p95_token_latency_ms`, `network_rx_mb`…) qui **violent la politique canonical no-float du repo** (un f64 dans un payload signé JCS ne round-trip pas bit-identiquement cross-langue → signatures non reproductibles + `Eq` non-dérivable). L'approche corrigée (toutes les métriques en entiers `u64`/`u32`) est précisée plus bas. Aucune Day-0 n'est touchée : on conserve Ed25519+JCS, iroh 0.98, 0 nouveau dep, le draft JSON R&D n'est pas le canonical SBFB. Deux ajustements de forme additionnels (digests en `[u8;32]` au lieu de `String "blake3:..."`, terminologie `version: u16` plutôt que `schema_version` à plat) relèvent du mirror du code, pas du plan.

## Résumé exécutif (3-5 lignes)

Phase C ajoute 4 structures wire (`ShardAssignment`, `ShardPlan`, `ShardedSessionManifest`, `RunProof`) dans `nexus-core-rs`, deux signées Ed25519+JCS (`ShardedSessionManifest` sous `DOMAIN_SHARD_PLAN_V1`, `RunProof` sous `DOMAIN_RUN_PROOF_V1`), `ShardPlan`/`ShardAssignment` étant des sous-structs non signées portées par le manifeste. Le patron exact à reproduire est `compute_group.rs` (Phase B, le plus récent : payload non-signé + enveloppe `Entry` avec attribution redondante + signature `BigArray` + `sign()`/`verify_signature()` enforçant caps DoS + version + attribution aux DEUX bouts). Les 5 scans convergent vers EXECUTE-en-mirror ; la seule correction bloquant le fait de figer le struct tel quel est le no-float sur les métriques RunProof. 0 bump wire confirmé (11 `*_FORMAT_VERSION` restent = 1), 2 domaines net-new confirmés (0 match dans `crates/`), 0 nouveau dep.

## S1a OSS prior-art (findings + ce qui valide/invalide les champs)

**Le design Phase C est ALIGNÉ SOTA et déjà figé au niveau champ** dans `remote_user_sharded_llm_rnd.md` §10.1-10.4 (+ addendum SOTA 2026-05-30 §0/§1/§3). Le kickoff exécute, ne re-conçoit pas (memory `sharding_design_frozen.md`). Confrontation à l'état de l'art :

- **ShardAssignment — champs load-bearing tous présents (Petals / Parallax).** Petals = blocs Transformer **consécutifs** par hôte, chaîne pipeline-parallel (arXiv 2209.01188) ; Parallax = Model Shard Holder + chaîne GPU layer-indexée (arXiv 2509.26182). Les 4 champs load-bearing de l'état de l'art sont VALIDÉS : (a) plage de couches **contiguë** `[layer_start, layer_end]` (le plan §6.2 les nomme exactement, reformulation du `layers[]` du draft §10.3 en bloc contigu = D3 pipeline-parallel) ; (b) worker id Ed25519 (`worker_pubkey`) ; (c) shard/model digest BLAKE3 pour hash-pin des poids (`shard_hashes` §10.3) ; (d) ordre pipeline. Les ajouts §10.3 (`role: layer_worker`, `kv_cache_policy: local_ephemeral`, `fallback_node`, `launch_profile_hash`) sont justifiés (churn Petals → fallback, addendum §2) et **figés** (addendum §1 confirme `kv_cache_policy` local éphémère).

- **RunProof — aligner sur TOPLOC (slot réservé, encodage = Phase G).** TOPLOC (arXiv 2501.16007, PrimeIntellect) : preuve = LSH top-k du DERNIER hidden state, 258 octets/32 tokens, binding modèle+prompt **implicite** (le vérifieur fournit même modèle + prompt + précision + topk pour recomputer). Conséquence : le slot fingerprint N0 doit pouvoir porter (i) les bytes de preuve TOPLOC (encodage **Phase G, PAS C**), (ii) le `prompt_profile_hash` BLAKE3 (= le binding externe que TOPLOC exige), (iii) le `model_digest` BLAKE3. Les slots miroirs existent déjà : `ResultPayload::logprobs_hash: [u8;32]` (verification.rs:223 / task.rs:511, le « slot l.383 » du brief) et `model_digest: [u8;32]` (task.rs:502).

- **Ordre pipeline = champ sémantiquement load-bearing.** JCS préserve l'ordre des éléments d'un array (il ne trie que les clés d'objet, canonical.rs:318-333) : un `ShardPlan = Vec<ShardAssignment>` signé garde son ordre dans les bytes canoniques → **pas de casse de signature**. Mais l'inférence exige que la couche N précède N+1 (Petals chaîne ordonnée, Parallax DAG layer-indexé). S'appuyer sur la seule position du Vec est fragile (un consommateur qui retrie casse l'inférence). L'invariant aligné SOTA : `layer_start` strictement croissant **et contigu** entre assignments consécutifs EST l'ordre pipeline — Phase C réserve la capacité de le vérifier (fonction check : pas de trou de couche, pas de chevauchement, `[0..L)` complet).

**Ce qui invalide le draft JSON (à corriger en codant, pas un blocage du plan)** :
1. **No-float** : `metrics.decode_tokens_per_sec` / `metrics.p95_token_latency_ms` etc. → entiers (cf. S1a finding blocker, détaillé en spec).
2. **Format digest** : `String "blake3:..."` → `[u8;32]` (le repo n'utilise JAMAIS ce préfixe dans les types signés ; `CatalogApp.archive_hash` est validé en 64 hex via `is_valid_archive_hash`, node_directory.rs:308-312).

## S1b Deps/CVE (0 nouveau dep confirmé)

**0 nouveau dep — CONFIRMÉ.** Tout est couvert par les deps existantes de `nexus-core-rs` (Cargo.toml:27-35) : `serde` (derive), `serde-big-array` 0.5 (`BigArray` pour `[u8;64]`, déjà utilisé par `ComputeGroupEntry`/`NodeDirectoryEntry`), `serde_json` (roundtrip tests), `serde_jcs` 0.2 (RFC 8785 via `canonical_bytes`), `ed25519-dalek` (via `crate::crypto::KeyPair`/`verify`), `blake3` (fingerprint N0). Aucune des 4 structures n'exige un crate absent. Day-0 « pas de nouveau dep crypto » respectée.

**Chemin Ed25519+JCS = mirror exact** : `canonical_bytes(&payload, DOMAIN_X)` (canonical.rs:276) → `keypair.sign(&bytes)` (crypto.rs:96) → `crate::crypto::verify(&pubkey, &bytes, &sig)` (crypto.rs:164). Mêmes imports que compute_group.rs:56-57.

**Slot N0 = `[u8;32]` BLAKE3 direct** : serde dérive nativement pour `[u8; N<=32]`, donc PAS de `serde-big-array` (réservé aux `[u8;64]` de signature), PAS d'encodage binaire nouveau. Si un futur fingerprint dépasse 32 octets → `Vec<u8>` borné par cap (déjà dispo), mais le design TOPLOC actuel = BLAKE3 32 octets.

**CVE** : aucune advisory active pertinente. `ed25519-dalek` lié = 2.2.0 (Cargo.lock:2086, la ligne 3.0.0-pre.6 est transitive frost/iroh seulement) — très au-dessus de RUSTSEC-2022-0093 (corrigé 1.0.1+). `serde_json` 1.0.149, `serde_jcs` 0.2.0, `blake3` 1.8.5 : 0 advisory active à la cutoff. Phase C n'ajoute aucune surface de dep nouvelle à auditer.

## S2 Décisions historiques + INVARIANTS de forme à respecter

Aucune DEVIATION historique ni DESIGN-CONFLICT bloquant : `git log -i --grep` DEVIATION/ShardPlan/RunProof ne renvoie que le planning S77 (`a1bbf00` kickoff) et Phase B (`81d667c`). La correction A1 de Phase B (`conn.rtt(PathId::ZERO)`) concerne le data plane `shard.rs`, PAS les primitives wire de C. **Invariants de forme que Phase C DOIT suivre (mirror `compute_group.rs`, le patron le plus récent)** :

1. **Deux types par primitive signée** : un struct PAYLOAD non-signé + un struct ENVELOPE (`*Entry`). PAYLOAD = `derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)`, contient — dans cet ordre — `version: u16`, l'identité du signataire (`[u8; PUBLIC_KEY_LENGTH]`), puis les champs métier. ENVELOPE = `payload` + identité **redondante** (MUST == payload.identité) + `signature: [u8; SIGNATURE_BYTES]` avec `#[serde(with = "BigArray")]`.

2. **Invariant cardinal — la signature couvre le PAYLOAD seul.** La `signature` et l'identité redondante de l'enveloppe ne sont JAMAIS dans les bytes canoniques (compute_group.rs:83-85). `canonical_bytes(&payload, DOMAIN_X)` ne voit que le payload.

3. **`version: u16` SANS `#[serde(default)]`** : un `version` manquant = payload malformé, PAS une omission tolérée (compute_group.rs:88-91). Const dédiée `SHARD_PLAN_FORMAT_VERSION = 1` / `RUN_PROOF_FORMAT_VERSION = 1` (ou une const partagée), vérifiée au verify.

4. **`sign()` ordre des checks** : (1) payload.identité == `keypair.public_bytes()` sinon `Err Crypto` ; (2) `check_caps(&payload)` ; (3) `bytes = canonical_bytes(&payload, DOMAIN_X)?` ; (4) `signature = keypair.sign(&bytes)` ; → `Entry { payload, identité: keypair.public_bytes(), signature }`.

5. **`verify_signature()` ordre des checks** : (1) `payload.version == X_FORMAT_VERSION` sinon `Err` ; (2) `check_caps` (AVANT de hasher) ; (3) `payload.identité == self.identité_redondante` sinon `Err` (anti split-brain) ; (4) `crate::crypto::verify(&identité, &canonical_bytes(&payload, DOMAIN_X)?, &signature)`.

6. **Caps DoS = fn libre appelée par `sign` ET `verify`** : un nœud ne peut JAMAIS produire un payload que ses propres pairs rejetteraient (mirror `check_group_caps` compute_group.rs:243-262, appelé sign:195 + verify:227). Référence chiffrée : `COMPUTE_GROUP_MAX_MEMBERS = 256`, `COMPUTE_GROUP_ID_MAX = 128`.

7. **Attribution asymétrique** : `ShardedSessionManifest` signé par l'INITIATEUR (même owner que `ComputeGroup.initiator`) ; `RunProof` signé par le WORKER qui a exécuté. Champ d'attribution redondant adapté par enveloppe.

8. **`group_id` corrélation** : `ShardedSessionManifest.group_id` doit matcher le `group_id` du `ComputeGroup` de Phase B (handle stable choisi par l'initiateur, compute_group.rs:94-97).

**FRICTION terminologie (concern, pas blocker)** : le draft §10 + plan §6.5 disent `schema_version: 1` et `signature` à plat DANS la struct. Aucune primitive signée du code (compute_group, node_directory, curator, invite, seed) n'utilise ce nom ni une signature in-struct : la convention établie = `version: u16` + enveloppe `Entry`. **Mirror le CODE, pas le JSON R&D.** Lire le critère pre-launch `schema_version: 1` comme « version=1, pas de bump », PAS comme une obligation de nommer le champ `schema_version`. Ne pas créer une 3e convention.

**SOUS-SPÉCIFICATION du plan (concern)** : le plan §6.3 ne liste que 3 tests. Le patron compute_group/node_directory impose un set plus large (~9-11) sinon une régression échappe au G-REVIEW — voir « Spécification concrète » → Tests.

## S3 Threat model (menaces + couverture + doc-notes phases ultérieures)

Les 4 primitives n'introduisent **aucune frontière d'admission nouvelle** (admission = ComputeGroup+ALPN de Phase B) ni aucun exec : ce sont des structures de données signées. Menaces classiques d'une primitive Ed25519+JCS, toutes couvertes par le mirror :

- **Confusion de domaine (blocker si non couvert)** : `DOMAIN_SHARD_PLAN_V1 != DOMAIN_RUN_PROOF_V1 != DOMAIN_COMPUTE_GROUP_V1`. Sans séparation, une signature de manifeste (« voici le plan que J'AUTORISE ») pourrait être rejouée comme RunProof (« voici ce que J'AI EXÉCUTÉ ») — les deux partagent la même clé Ed25519 du nœud. **Couvert** par 2 DOMAIN_* distincts + doc-comment énumérant la disjonction cross-famille + **test cross-domain** (mirror compute_group.rs:390-402 : bytes distincts ET signature mintée sous le mauvais domaine rejetée au verify).

- **Split-brain d'attribution (blocker si non couvert)** : un forwarder agrafe un pubkey différent sur l'enveloppe. **Couvert** par le champ redondant vérifié (`Entry.initiator == payload.initiator` / `Entry.worker_pubkey == payload.worker_pubkey`), test `verify_rejects_attribution_mismatch`.

- **DoS (blocker si non couvert)** : `ShardPlan = Vec<ShardAssignment>` et `RunProof.participants = Vec` déserialisés depuis le réseau. **Couvert** par caps nommés enforced sign-AND-verify AVANT hash. Fingerprint N0 = type fixe `[u8;32]` → borné par construction, élimine la classe DoS (préférer `[u8;32]` à un `Vec<u8>` cappé).

- **Replay temporel / rollback de révision (concern, doc-note)** : une primitive crypto pure ne protège PAS contre le rejeu d'un manifeste plus ancien légitime (propriété stateful). **Doc-note** : anti-rollback = concern ingest/scheduler (Phase J), pas cette couche (mirror compute_group.rs:105-110). **Cross-session replay** : lier `RunProof` à `(session_id, assignment)` DANS les canonical-bytes pour qu'une preuve honnête d'une session passée ne soit pas rejouée dans une session courante = mitigation par construction.

- **Worker malveillant / fingerprint N0 attaquant-contrôlée (concern, doc-note phases G/H/I)** : en Phase C la fingerprint est PUREMENT attaquant-contrôlée (un worker signe un RunProof valide sur sa propre clé portant une fingerprint arbitraire). C'est ATTENDU : la signature prouve seulement WHO (non-répudiation), PAS la correction du calcul. **Doc-note obligatoire** sur RunProof : « auto-attestation signée ; une signature valide n'atteste QUE l'identité du worker, jamais la correction du calcul ; fingerprint N0 NON vérifiée tant que G/H/I ne sont pas livrées ; un consommateur ne doit pas la traiter comme une preuve de calcul » (mirror honnête task.rs:485-497 model_digest). Carry candidat P-NN : « RunProof verification not wired » → Phase G.

- **Privacy SI-1/SI-3 de la fingerprint d'activation (concern, doc-note Phase K)** : la fingerprint TOPLOC est un LSH top-k du dernier hidden state — surface de SI-1 (reconstruction d'input, High) / SI-3 (fingerprinting de prompt, Medium). En C la valeur n'est PAS encore produite (slot réservé) → 0 fuite réelle. **Doc-note** : analyse privacy SI-1/SI-3 de la fingerprint = THREAT_MODEL §16, écrit en Phase K (plan §14.2). Ne PAS sur-promettre la confidentialité (scope cut #4 : activations en clair, limite physique).

**Aucune row STRIDE due en Phase C** ; §16 sharding (SI-1..SI-5) est écrite en Phase K.

## S4 Wire format (verdict 0-bump + additivité)

**Verdict S4 binaire : 0-bump CONFIRMÉ + additivité des 2 domaines CONFIRMÉE. Aucune violation.**

- **Les 11 `*_FORMAT_VERSION` restent = 1, Phase C n'en touche aucun** : `COMPUTE_GROUP_FORMAT_VERSION` (compute_group.rs:66), `CURATOR_LIST_FORMAT_VERSION` (curator.rs:61), `KEY_ROTATION_FORMAT_VERSION` (key_rotation.rs:32), `NODE_DIRECTORY_FORMAT_VERSION` (node_directory.rs:84), `POW_FORMAT_VERSION` (pow.rs:85), `SEED_FORMAT_VERSION` (seed.rs:51), `TASK_FORMAT_VERSION` (task.rs:61), `PIN_FILE_FORMAT_VERSION` (tls_pinning.rs:102). Phase C ajoute des structs net-new, ne réécrit aucun canonical existant.
- **`DOMAIN_SHARD_PLAN_V1` + `DOMAIN_RUN_PROOF_V1` strictement net-new** : `grep DOMAIN_SHARD_PLAN|DOMAIN_RUN_PROOF` sur tout `crates/` = 0 match. Ajout pur, 0 réécriture d'un domaine en place (22 DOMAIN_* existants intacts). Pattern = `DOMAIN_COMPUTE_GROUP_V1` (canonical.rs:241-255).
- **Les 4 structs n'existent pas encore** : `grep ShardPlan|ShardAssignment|ShardedSessionManifest|RunProof` sur `crates/` = 0 match. `shard.rs` (Phase B) ne contient QUE le data plane ALPN. Aucune collision de type.
- **`FEED_FORMAT_VERSION` / `DOMAIN_FEED_V1` non touchés** : Phase C définit des structs crypto autonomes, PAS des opérations de feed.
- **2 domaines DISTINCTS obligatoires** : `canonical_bytes` (prefix `<domain><0x00><jcs>`) garantit l'isolation tant que les const diffèrent. Test domain-separation explicite à ajouter (mirror compute_group.rs:391).

## Spécification concrète recommandée pour Phase C

**Où vit le code** : module dédié `crates/nexus-core-rs/src/shard_plan.rs` (le plan §6.2 autorise « shard.rs ou module dédié » ; séparer le data plane ALPN — `shard.rs` — des primitives wire signées est plus lisible et évite de mélanger framing et crypto). Déclarer `pub mod shard_plan;` dans `lib.rs` près de ses voisins (`shard` est en lib.rs:61). Imports : `use crate::canonical::{DOMAIN_SHARD_PLAN_V1, DOMAIN_RUN_PROOF_V1, canonical_bytes}; use crate::crypto::{KeyPair, PUBLIC_KEY_LENGTH, SIGNATURE_BYTES};` + `serde_big_array::BigArray`.

**Constantes (mirror compute_group.rs:66-79)** :
```
SHARD_PLAN_FORMAT_VERSION: u16 = 1;
RUN_PROOF_FORMAT_VERSION: u16  = 1;
SHARD_PLAN_MAX_ASSIGNMENTS: usize = 256;   // mirror COMPUTE_GROUP_MAX_MEMBERS ; fan-out réel 3-5 (addendum §1), 256 généreux < seuil RAM
RUN_PROOF_MAX_PARTICIPANTS: usize = 256;   // mirror idem
SESSION_ID_MAX: usize = 128;               // mirror COMPUTE_GROUP_ID_MAX
SHARD_GROUP_ID_MAX: usize = 128;           // mirror COMPUTE_GROUP_ID_MAX (corrélation ComputeGroup)
SHARD_HASHES_MAX: usize = 64;              // cap sur Vec<[u8;32]> shard_hashes par assignment
```
**Constantes DOMAIN_* (additives, canonical.rs, mirror DOMAIN_COMPUTE_GROUP_V1:241-255 avec doc-comment énumérant la disjonction anti-replay cross-famille + phrase canon « purely additive, 0-bump… S74 DOMAIN_SEED_REQUEST_V1 pattern »)** :
```
DOMAIN_SHARD_PLAN_V1: &[u8] = b"nexus-shard-plan-v1";
DOMAIN_RUN_PROOF_V1:  &[u8] = b"nexus-run-proof-v1";
```

### `ShardAssignment` (sous-struct NON signée, testée serde-seul)
| champ | type | rationale |
|---|---|---|
| `worker_pubkey` | `[u8; PUBLIC_KEY_LENGTH]` | id Ed25519 du worker (mirror compute_group.members ; PAS String — cohérent avec l'allowlist au handshake shard.rs `conn.remote_id`) |
| `layer_start` | `u32` | borne basse contiguë du bloc (reformule `layers[]` §10.3 ; D3 pipeline-parallel) |
| `layer_end` | `u32` | borne haute ; invariant `layer_start <= layer_end` (check caps) |
| `role` | `String` (cappé) ou enum | `layer_worker` (§10.3) ; si String, cap nommé |
| `shard_hashes` | `Vec<[u8; 32]>` (cap `SHARD_HASHES_MAX`) | BLAKE3 hash-pin des poids du shard (§10.3 ; PAS `String "blake3:..."`) |
| `kv_cache_policy` | `String` (cappé) ou enum | `local_ephemeral` (figé addendum §1) |
| `fallback_node` | `Option<[u8; PUBLIC_KEY_LENGTH]>` `#[serde(default)]` | nœud de secours (churn Petals, addendum §2) ; `default` runtime-tolerance car optionnel |
| `launch_profile_hash` | `[u8; 32]` | BLAKE3 (§10.3 ; PAS String) |

### `ShardPlan` (sous-struct NON signée)
| champ | type | rationale |
|---|---|---|
| `assignments` | `Vec<ShardAssignment>` (cap `SHARD_PLAN_MAX_ASSIGNMENTS`) | liste **ordonnée** ; ordre pipeline = `layer_start` strictement croissant + contigu (invariant vérifiable par fonction check : pas de trou, pas de chevauchement, `[0..L)` complet) |

### `ShardedSessionManifest` (PAYLOAD signé) + `ShardedSessionManifestEntry` (enveloppe)
PAYLOAD (signé sous `DOMAIN_SHARD_PLAN_V1`, par l'INITIATEUR) :
| champ | type | rationale |
|---|---|---|
| `version` | `u16` | == `SHARD_PLAN_FORMAT_VERSION` ; **PAS** `serde(default)` |
| `initiator` | `[u8; PUBLIC_KEY_LENGTH]` | signataire = group owner (mirror ComputeGroup.initiator) |
| `session_id` | `String` (cap `SESSION_ID_MAX`) | handle de session (uuid §10.1) |
| `group_id` | `String` (cap `SHARD_GROUP_ID_MAX`) | **doit matcher** `ComputeGroup.group_id` (corrélation pipeline↔allowlist) |
| `revision` | `u64` | monotone ; anti-rollback = ingest (doc-note) |
| `plan` | `ShardPlan` | le plan de shards autorisé |
| `model_digest` | `[u8; 32]` | BLAKE3 du modèle (§10.1 model.name/format → digest) |
| `tokenizer_hash` | `[u8; 32]` | BLAKE3 (§10.1) |
| `chat_template_hash` | `[u8; 32]` | BLAKE3 (§10.1) |

ENVELOPE : `manifest: ShardedSessionManifest` + `initiator: [u8; PUBLIC_KEY_LENGTH]` (redondant, MUST ==) + `signature: [u8; SIGNATURE_BYTES]` `#[serde(with = "BigArray")]`.

> Note : `network_profile` / `security` du draft §10.1 sont des sous-objets de politique. S'ils sont inclus en C, les structurer en sous-structs à champs entiers/bool (`max_rtt_ms: u32`, `min_uplink_mbps: u32`, `relay_allowed: bool`, `private_group_only: bool`…) — JAMAIS de float. S'ils ne sont pas load-bearing pour le round-trip de signature en C, les différer est un scope-cut cohérent (le plan §6.2 ne les exige pas explicitement).

### `RunProof` (PAYLOAD signé) + `RunProofEntry` (enveloppe)
PAYLOAD (signé sous `DOMAIN_RUN_PROOF_V1`, par le WORKER) :
| champ | type | rationale |
|---|---|---|
| `version` | `u16` | == `RUN_PROOF_FORMAT_VERSION` ; **PAS** `serde(default)` |
| `worker_pubkey` | `[u8; PUBLIC_KEY_LENGTH]` | signataire = worker (auto-attestation) |
| `session_id` | `String` (cap `SESSION_ID_MAX`) | **lie** la preuve à la session (anti cross-session replay) |
| `model_digest` | `[u8; 32]` | BLAKE3 (§10.4 ; PAS String) |
| `prompt_profile_hash` | `[u8; 32]` | BLAKE3 = binding externe que TOPLOC exige (§10.4) |
| `activation_fingerprint` | `[u8; 32]` `#[serde(default)]` | **slot N0 TOPLOC réservé** ; `[0u8;32]` = vide en C, encodage réel = **Phase G** ; doc-comment « slot réservé, encodage TOPLOC Phase G » (mirror logprobs_hash task.rs:508-511) |
| `metrics` | `RunMetrics` (sous-struct) | voir ci-dessous — **TOUT EN ENTIERS** |
| `participants` | `Vec<[u8; PUBLIC_KEY_LENGTH]>` (cap `RUN_PROOF_MAX_PARTICIPANTS`) | nœuds du pipeline (§10.4 ; pubkeys, PAS String "node-a") |

`RunMetrics` (**correction no-float — c'est l'ajustement PLAN-ADAPT central**) :
| champ | type draft (float, INTERDIT) | type corrigé | rationale |
|---|---|---|---|
| `ttft_ms` | `u64` | `u64` | déjà entier |
| `decode_milli_tokens_per_sec` | `decode_tokens_per_sec` (float) | `u64` (milli-tok/s) | éviter le ratio flottant ; alternative : `tokens_generated: u64` + `duration_ms: u64`, ratio calculé côté lecture |
| `p95_token_latency_ms` | float | `u64` | ms entiers |
| `network_rx_bytes` | `network_rx_mb` (float) | `u64` | octets entiers (mirror ResultPayload.*_bytes:u64) |
| `network_tx_bytes` | `network_tx_mb` (float) | `u64` | octets entiers |
| `worker_drop_count` | int | `u32` | déjà entier |

> Justification no-float : verification.rs:43-47 documente que le format canonical **interdit les floats** (ne round-trippent pas bit-identiquement cross-plateforme) ; `Task` ne dérive PAS `Eq` à cause d'un unique f64 ; tous les métriques signés existants sont des entiers (`ResultPayload.generation_time_ms:u64`, `tokens_generated:u64`). Un f64 dans un payload signé Ed25519+JCS → signatures divergentes entre signataire Rust et vérifieur Python (footgun JCS d'origine canonical.rs) + `Eq` non-dérivable. **En entiers, RunMetrics et tous les payloads dérivent `Eq`.**

ENVELOPE : `proof: RunProof` + `worker_pubkey: [u8; PUBLIC_KEY_LENGTH]` (redondant, MUST ==) + `signature: [u8; SIGNATURE_BYTES]` `#[serde(with = "BigArray")]`.

### Rationale `#[serde(default)]`
Appliquer UNIQUEMENT aux champs **optionnels/additifs de robustesse runtime** (`fallback_node`, `activation_fingerprint` vide), JAMAIS à `version` ni aux champs identitaires (`worker_pubkey`, `initiator`, `session_id`, `model_digest`, `layer_start/end`) ni à la signature. Doc-comment par champ : « runtime tolerance: un client Python envoyant un JSON minimal à l'API daemon désérialise ce champ omis à zéro/false plutôt que 422 » (pas historical-compat, pre-launch policy CLAUDE.md). Risque sinon : un champ signé omis → désérialise à zéro → canonical-bytes divergents → signature casse silencieusement.

### Tests (élargir les 3 du plan au set complet du patron, ~9-11)
Les 3 du plan : `shard_plan_signature_roundtrip` (= manifest signature roundtrip), `shard_assignment_serde_roundtrip` (serde-seul, PAS signé), `run_proof_signature_roundtrip`. **Ajouter (mirror compute_group.rs:264-414)** : `verify_rejects_tampered_payload`, `verify_rejects_tampered_signature`, `verify_rejects_attribution_mismatch`, `sign_rejects_wrong_signer`, `rejects_oversized_*` (caps DoS sign ET verify, boundary pass + over-cap fail mirror node_directory.rs:445-461), `domain_separated_*` (bytes diffèrent), `cross_domain_signature_rejected` (signature mintée sous le mauvais domaine rejetée — LE test anti-replay critique), `json_roundtrips` + re-verify. Plus un test de l'invariant pipeline (layer contiguïté : trou/chevauchement rejeté).

### Acceptation
`cargo nextest run -p nexus-core-rs --locked` vert ; round-trip canonical-bytes stable sous `DOMAIN_SHARD_PLAN_V1` et `DOMAIN_RUN_PROOF_V1` ; `cargo fmt --check` + `cargo clippy --workspace --all-targets --locked -- -D warnings` ; dual-platform (Windows nextest + Docker `sbfb-ci` rust:1.94).

## Carry T-NN+3 (absorption JCS)

**NON absorbé en Phase C → carry maintenu.** Phase C touche `canonical.rs` UNIQUEMENT pour AJOUTER 2 constantes `DOMAIN_*` (additif pur, ~12 lignes de doc chacune) — elle ne touche PAS la primitive JCS. La fonction `canonical_bytes` (canonical.rs:276) est DÉJÀ l'unique point de factorisation JCS ; il n'y a PAS de duplication de la logique JCS dans le crate. Le « JCS dup » réel est le **boilerplate sign/verify/check_caps** répété par module (compute_group, node_directory, curator, seed) — un trait générique `SignedEnvelope` touchant 4+ modules crypto stables = refactor transverse qui élargit le blast radius d'une phase wire-primitive (risque de régression sur des chemins crypto stables) et mérite son propre cycle review+Codex (pas de band-aid). Phase C porte ce boilerplate à une **5e/6e copie** (`ShardedSessionManifestEntry` + `RunProofEntry`) — c'est le seuil où un trait commun devient justifié. **Recommandation** : doc-note T-NN+3 « 5e+ copie atteinte en S77-C, candidat à un sprint de dette dédié », router en carry P2.

## Risques résiduels

1. **No-float (PLAN-ADAPT, traité)** : si l'implémenteur copie le draft JSON verbatim avec des floats, les signatures ne round-trippent pas et `Eq` casse — la spec ci-dessus le prévient (RunMetrics tout-entiers). À re-vérifier au G-REVIEW : `grep f32|f64` dans `shard_plan.rs` doit être vide.
2. **Format digest** : risque de coder `String "blake3:..."` par mimétisme du draft → cohérence rompue avec le repo. Spec impose `[u8;32]` partout.
3. **Sous-spécification tests du plan** : ne livrer que les 3 tests du §6.3 laisserait passer une régression (attribution mismatch, cross-domain) — le set élargi est obligatoire au G-REVIEW.
4. **Couplage Phase B/C non vérifié** : la corrélation `group_id` manifest↔ComputeGroup n'est qu'un invariant documenté en C (le check stateful est Phase J/scheduler) ; ne pas le câbler en C est cohérent mais doit être doc-noté.
5. **Slot N0 inerte** : `activation_fingerprint` reste `[0u8;32]` jusqu'à Phase G ; risque qu'un consommateur intermédiaire le croie — mitigé par doc-comment « auto-attestation, fingerprint NON vérifiée jusqu'à G/H/I » (mirror honnête task.rs model_digest).
