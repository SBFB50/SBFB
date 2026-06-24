# Sprint 77 Phase G — Preflight G8 (N0 TOPLOC fingerprint)

> Orchestration **Workflow ultracode** (`wf_98896c26-090`, 6 agents, 5 scans
> paralleles + synthese adversariale). Phase G = §10 du plan. Cas B.

## Verdict : **PLAN-ADAPT**

Aucune Day-0 violee (canonical no-float tenu, 0 bump wire tenu, named-consts
applicables). Le plan §10 est applicable a ~90% mais une formulation load-bearing
est impossible telle quelle : **« comparaison seuil global » in-vivo on-wire en
Phase G**. Resolution ci-dessous. La note `verification.rs:42-47` « v1.2 concern »
est un pointeur-avant **perime** (arbitrage PO D4 reclasse N0=Phase G, N2=Phase I
S77) a corriger en Phase G — doc-honnetete, pas conflit.

## Algorithme TOPLOC reconstitue (scan S1a, code source PrimeIntellect-ai/toploc + arXiv 2501.16007v2)

- **Materiel** : pour une fenetre de tokens, top-k=128 (par valeur absolue) du
  **dernier hidden state** ; x = indices, y = bits BRUTS bf16 reinterpretes en
  uint16. Encodage natif TOPLOC = polynome de Newton sur GF(65497) → 258 B/32 tok
  (header u16 modulus injectif + 128 coeffs u16 BE). **Tout-entier dans la preuve**
  (les seuls floats sont les stats de verif COTE VALIDATEUR, jamais dans le proof).
- **Comparaison TOLERANTE** (2 composantes sur les bits bf16 aux indices top-k) :
  (a) `exp_mismatches` = nb d'indices ou les 8 bits d'exposant different (mask
  `0x7F80>>7`) ; (b) pour les indices ou l'exposant matche, `abs(mant_proof -
  mant_replay)` (7 bits, mask `0x007F`) → mean + median. Accept SSI
  `exp_mismatches < T_EXP ET mant_err_mean < T_MEAN ET mant_err_median < T_MEDIAN`.
- **Seuils bf16** (du papier, pas du code OSS) : T_EXP=38, T_MEAN=10, T_MEDIAN=8.
  fp32 : 8/256/128. Validation empirique tests OSS : jitter *1.01 (meme modele) →
  exp_mismatches=0, mant_mean<=2 (PASS) ; *1.10 → mant_mean>10 (au seuil) ; *4
  (modele different) → exp_mismatches=k, mant>2^32 (REJET franc).
- **Propriete coeur** : l'exposant bf16 est ROBUSTE au non-determinisme GPU
  (reordering/FMA) ; un modele/precision DIFFERENT change radicalement quels
  indices sont top-k ET leurs exposants → explosion exp_mismatches.

## Resolution commitment-vs-payload (TRANCHE)

Le slot wire est `[u8;32]` (= `BLAKE3_BYTES`) : `RunProof.activation_fingerprint`
(`shard_plan.rs:439`, `DOMAIN_RUN_PROOF_V1`) et `ResultPayload.logprobs_hash`
(`task.rs:511`, `DOMAIN_RESULT_V1`). L'encodage TOPLOC (258 B/32 tok) **ne loge
pas dans 32 octets**. Le plan §10.2 exige « slot TOPLOC reel (0 bump wire) » + la
memory S77 impose « 0 bump wire ». Donc :

- **Le slot porte un COMMITMENT BLAKE3 32B** de l'encodage canonique tout-entier.
  0 nouveau `DOMAIN_*`, 0 nouveau champ, 0 bump version (slot reserve deja present
  depuis Phase C). §19 (compute/shard domains) reste clos a 4.
- **La comparaison TOLERANTE opere sur le payload complet** (indices + bits bf16),
  qui n'a **aucun porteur on-wire en Phase G**. Transport du payload + comparaison
  in-vivo cross-worker = **H/I** (candidat recommande : blob iroh-blobs ancre dont
  BLAKE3 == le commitment, content-addressing — a trancher en H/I, PAS en G).
- **BLOCKER de design (resolu, a documenter)** : un hash BLAKE3 DETRUIT la localite
  (1 bit flip → avalanche). Le commitment binde le model-swap par egalite ; il ne
  porte AUCUNE tolerance. La tolerance reelle vit dans la primitive exposee +
  payload off-slot (exactement le « separate off-canonical payload » anticipe
  `verification.rs:46-47`). Phase G NE pretend PAS resoudre la verif cross-GPU
  tolerante in-vivo.

## Decision d'encodage (latitude laissee par le verdict : « coeffs poly GF ou sketch entier »)

→ **Sketch entier direct** : `ToplocFingerprint { indices: Vec<u32>, value_bits:
Vec<u16> }` (value_bits = troncature bf16 = top 16 bits de `f32::to_bits`). PAS le
polynome GF(65497). Justification (sanctionnee par le verdict PLAN-ADAPT) :
1. **Auditabilite** d'une primitive de securite **always-compiled** (core-rs) : le
   polynome introduit l'aliasing `y mod 65497` sur activations negatives (bits bf16
   65497..65535 → [0,39)), l'evaluation a indices non-vus, et un `mod_inverse`
   (Euclide etendu) — surface de bug subtile pour zero benefice fonctionnel ici.
2. La **compression GF (258B)** n'est PAS realisee on-wire en Phase G (slot =
   commitment 32B). Le sketch direct (indices u32 + value_bits u16) sert
   identiquement le commitment, le roundtrip, la detection de swap et la
   comparaison tolerante exp/mantisse FIDELE a TOPLOC.
3. Le polynome GF reste une **optimisation de compression deferable a H/I**
   (transport on-wire du payload), si la taille importe alors.
   Note honnete : taille sketch SBFB = ~768 B/32 tok (vs 258B GF) — negligeable,
   non on-wire en G.

## no-float compliance (BLOCKER, non negociable)

La pre-image hashee DOIT etre tout-entier. Source = `top_k_by_magnitude`
(`shard.rs:142`) → `Vec<(u32,f32)>` ; le f32 ne round-trip PAS bit-identique
cross-plateforme. Donc l'encodage quantifie le f32 en bits bf16 (`(to_bits>>16) as
u16`) AVANT le hash. dtype = bf16/fp16 (seuils bf16). Le commitment etant un hash,
JCS no-float ne s'y applique pas, mais la pre-image entiere est la condition de
round-trip Rust-signer / verifieur Python.

## Scope Phase G (livrables a coder)

1. **Module `crates/nexus-core-rs/src/toploc.rs`** (net-new, always-compiled,
   0-dep, tout-entier) : `ToplocFingerprint` (indices + value_bits, cap K_TOPLOC),
   `bf16_bits`, `from_topk(&[(u32,f32)])`, `to_bytes`/`from_bytes` (roundtrip),
   `commitment() -> [u8;32]` (BLAKE3 de la pre-image canonique), primitive
   tolerante `compare(&self, &Self) -> ToplocComparison { exp_mismatches,
   mant_err_*, accepted }` (distance exp/mantisse, seuils named-consts, integer-pure
   — compare `sum < T*count` pour eviter le float meme local).
2. **`lib.rs`** : `pub mod toploc` + re-exports.
3. **`verification.rs`** : Layer3 re-cable doc → egalite-sur-commitment TOPLOC
   (mecanique d'egalite-hash INCHANGEE : `*expected == reported_lp`) ; corriger la
   note module « v1.2 concern » → N0=Phase G / N2=Phase I S77 ; reecrire (PAS
   supprimer) les 2 tests `logprob_hash_match/mismatch` sur un commitment TOPLOC
   reel ; les 6 autres tests INCHANGES.
4. **`task.rs`** : doc-note `logprobs_hash` → commitment TOPLOC reel + caveat
   auto-attestation.
5. **`shard_plan.rs`** : doc-note `activation_fingerprint` → encodage reel livre G,
   recompute independant H/I, caveat.
6. **worker-core** : helper pur `toploc_commitment(hidden: &[f32]) -> [u8;32]`
   (top_k_by_magnitude → `nexus_core_rs::ToplocFingerprint` → commitment),
   CI-testable ; cable dans le chemin gated last-shard (post-norm hidden state).
   Sur Ollama/HTTP : slot reste `[0u8;32]` (N0 infaisable sans fork) — documente.
7. **Named-consts** : `TOPLOC_TOP_K=128`, `TOPLOC_THRESH_EXP_MISMATCH=38`,
   `TOPLOC_THRESH_MANT_MEAN=10`, `TOPLOC_THRESH_MANT_MEDIAN=8`, masks bf16. Verifies
   G-REVIEW.
8. **THREAT_MODEL** : sous-section §16 N0 + MAJ §15.2 row I + bump v10→v11.

### Reste H/I (NE PAS coder en G)
- **H** : N1 VRF spot-check (`rerun.rs`) qui RECOMPUTE le fingerprint + compare via
  la primitive tolerante ; incentive curator-reputation ; mapping criticite→niveau.
- **I** : N2 redondance tolerante (`redundancy.rs`+`validator.rs` additif, quorum
  exact INCHANGE) consommant la primitive tolerante in-vivo ; N3 bissection
  opML/SENTINEL ; **TRANSPORT du payload complet on-wire** (blob/doc/champ additif).

## Threat model actions
- MAJ §15.2 row I (`THREAT_MODEL:918`) : residuel « detection swap cablee N0 (Phase
  G, encode-only) ; recompute independant N1/N2 = Phase H/I ». Residuel « Worker
  menteur GGUF different » : M tant que H/I non livres (renvoi explicite).
- Sous-section §16.x « N0 TOPLOC fingerprint (Phase G) » : DETECTE ~100% le swap
  modele/precision ; ne DETECTE PAS (a) honnete-mais-curieux confidentialite
  (SI-1/SI-4 High INCHANGES), (b) activation forge coherente, (c) la correction du
  calcul en general.
- **Caveat auto-attestation OBLIGATOIRE** (mirror `task.rs:483-511` +
  `shard_plan.rs:34-44`) : self-claim tant que N1/N2 (H/I) ne recomputent pas ; live
  result path = quorum exact-match `result_text` INCHANGE.
- Caveat backend Ollama (slot `[0u8;32]`, sharding impose `llm_llama_cpp`).
- Citer SI-3 : le fingerprint est lui-meme derive du hidden state (correle au TYPE
  de prompt), borne par le groupe prive.
- Caveat retention/GC (addendum §10 q.210, OUVERT) : persiste tant que le RunProof
  persiste, GC non cable (renvoi N3 H/I).
- Bump §17 v10→v11 (MAJ §7/§8, pas de bump wire).

## Tests plan
1. `toploc_fingerprint_encode_decode_roundtrip` (hermetique CI).
2. `toploc_detects_model_swap` (hermetique CI) — commitment different.
3. `toploc_accepts_same_model_within_threshold` (hermetique CI) — primitive
   tolerante, fixture *1.01 PASS / *1.10 au seuil / *4 REJET.
4. `commitment_*` : BLAKE3(encodage) ecrit dans les 2 slots ; pre-image tout-entier.
5. `verifier_layer3_commitment_*` : 2 tests logprob reecrits (match→passed,
   mismatch→trust_delta -5 sans ban) ; empty/tampered/digest_* + spot_check_rate
   INCHANGES.
6. worker-core : helper pur testable CI (top-k → commitment deterministe).
7. T1 E2E : `N-A-no-frontend-change`. Acceptation §10.4 :
   `cargo nextest run -p nexus-core-rs --locked` verts ; primitive detecte le swap.

## Blockers (tous resolus par ce verdict, A DOCUMENTER)
1. Commitment BLAKE3 DETRUIT la localite → slot = binding seul, tolerance off-slot
   (H/I). G ne pretend pas resoudre la verif cross-GPU tolerante in-vivo.
2. Caveat auto-attestation preserve mot-pour-mot (sinon regression d'honnetete).
3. Pre-image hashee tout-entier (jamais f32 serialise).
4. NON-BLOCKER a confirmer (hors-G, conditionne l'encodage cote worker) : tenseur
   ggml extrait F1/F2 = post-final-norm/pre-logits ; dtype bf16/fp16 ; k=128 assez
   large pour intersection top-k cross-backend CUDA(5080)/Metal(M2) — calibre sur
   rig en Phase K.
