# Sprint 77 — Phase I preflight (G8)

> Produit par Workflow ultracode (5 scans factuels Opus-4.8 + synthèse adversariale Opus-4.8).
> Re-run Opus-grade après détection que les scans Explore/general-purpose héritaient de Haiku 4.5
> (le 1er run rendait EXECUTE ; le run Opus rend PLAN-ADAPT — corrections N3 load-bearing).

## Verdict: PLAN-ADAPT

La partie **N2 (redondance tolérante M-of-N)** est EXECUTE-able telle quelle : elle réutilise `ToplocFingerprint::compare` déjà éprouvé par N1, et le chemin additif dans `validator.rs` laisse `validate_quorum_pre_guardrail` (validator.rs:219) strictement inchangé — vérifié byte-pour-byte (le dispatch unique est `validator.rs:117` sur `task.redundancy_factor > 1`, disjoint des fingerprints). La partie **N3 impose un PLAN-ADAPT** appuyé sur des évidences OSS concrètes (arXiv 2401.17555 opML, arXiv 2603.03592 SENTINEL, arXiv 2501.16007 TOPLOC) : le plan §12.2 fusionne en un seul mécanisme deux primitives SOTA orthogonales, attribue « O(1 bloc) » à une « bissection » (oxymore), et omet que le reveal opML d'activations cross-GPU NE PEUT PAS être vérifié par égalité de commitment BLAKE3. **Aucune décision Day-0/historique gelée n'est remise en cause** (0-bump wire, 0-dep, no-float-cœur, JCS, `validate_quorum_pre_guardrail` intact, `redundancy_factor` non-signé, PO-12 non-monétaire tous tenus) → ce n'est PAS un DESIGN-CONFLICT. L'adaptation corrige l'APPROCHE de N3, pas le scope.

## Scans (resume)

### S1a OSS prior-art
Signal **PLAN-ADAPT**. N2 solidement aligné SOTA (TOPLOC full-sketch exponent-exact + mantissa-tolerant, déjà câblé par N1 `rerun.rs:110`). N3 = trois écarts d'exécution corrigeables sans toucher Day-0 :
1. Le plan **fusionne opML et SENTINEL** en un seul mécanisme. Ce sont deux primitives orthogonales : opML (2401.17555) = commit-reveal + **bissection interactive O(log L)** réduisant le désaccord à une instruction ; SENTINEL (2603.03592) = monitoring **statistique EMA per-stage, O(1), AUCUN commit-reveal**. Le « O(1) » vient précisément de l'ABSENCE de bissection. → Séparer en deux primitives nommées.
2. **SENTINEL est conçu pour le TRAINING** (forward + backward, gradients β_h=0.9/β_g=0.8). SBFB est inférence forward-only — seule la moitié forward-activation de l'EMA est portable ; la métrique entière (no-float) doit être spécifiée (norme L1/L2 entière de l'activation inter-stage).
3. **opML n'a de sens que sous exécution déterministe bit-exacte** (fixed-point + softfloat + seed fixe). SBFB ne l'a PAS (question ouverte #6 du design gelé ; non-déterminisme GPU = raison d'être de TOPLOC). Donc le reveal doit transporter le **sketch TOPLOC complet (+ nonce)** et le verdict doit être `compare` tolérant, JAMAIS l'égalité de commitment (qui faux-rejette systématiquement cross-GPU, avalanche 1 bit, toploc.rs:31-40).

### S1b Deps/CVE
**EXECUTE.** 0 nouvelle dépendance requise. Tout l'outillage est déjà déclaré dans `nexus-core-rs/Cargo.toml:32-35,30` : ed25519-dalek 2.2.0, blake3 1.8.5, sha2 0.10.9, hmac 0.12.1, serde_jcs 0.2. Le coordinator hérite d'ed25519-dalek + `DOMAIN_*` via la dep path `nexus-core-rs`. La comparaison tolérante (N2) et l'EMA SENTINEL (N3) sont faisables en std + entiers — précédent direct `toploc.rs:297` (`abs_diff` sur mantisses u16, zéro crate). EMA tient en u128 basis-points. CVE : un seul advisory ignoré (RUSTSEC-2026-0097 rand 0.8, non-exploitable car OsRng direct), non aggravé par Phase I. iroh 0.98 hors-scope.

### S2 Decisions historiques
**Aucun DESIGN-CONFLICT.** Trois zones gelées pertinentes, toutes respectées :
- **Invariant cardinal** : `validate_quorum_pre_guardrail` (validator.rs:219-338, dispatch 117 sur `redundancy_factor>1`) = quorum exact-equality sur `result_text` brut. Aucun chemin N2/tolérant n'existe aujourd'hui. Le plan exige un chemin ADDITIF distinct (test #3 git-diff=0). Séparable structurellement (N2 opère sur fingerprints RunProof, disjoints de result_text/sha256).
- **`redundancy_factor` NON-signé** (S23 `34c77ce`, exclu canonical). Phase H a institué le pattern ADVISORY/consumer-enforced (`verification.rs:372-379` : niveau N2 ssi `verifiable && redundancy_factor>1`, mais ADVISORY car MITM peut abaisser). N2 peut être SÉLECTIONNÉ via le champ advisory, mais l'ACCEPT/REJECT de sécurité doit reposer sur des fingerprints dérivés du **RunProof SIGNÉ** (DOMAIN_RUN_PROOF_V1).
- **DOMAIN_KUDOS_V1 figé + PO-12 non-monétaire** : Phase I n'introduit ni incentive ni sanction économique (N3 explicitement « PAS de smart-contract »). Garde-fou : SENTINEL localise un stage corrompu → verdict de correctness/rejet, JAMAIS un slash monétaire.

### S3 Threat model
**EXECUTE/additif** (pattern v11/v12 établi : chaque phase ajoute sa sous-section §16 + MAJ §15.2 row I). Surfaces nouvelles à documenter :
- **N2** : collusion-sur-résultat-proche (instanciation INTÉGRITÉ de SI-4 High) ; calibration du seuil de tolérance (trop large = faux-accept / trop étroit = faux-reject cross-GPU, §15.2:917) ; note cardinale que N2 ne desserre JAMAIS le quorum exact (validator.rs:219 INCHANGÉ).
- **N3** : grinding/refus-de-reveal (withholding = défaut-coupable)/replay du commit-reveal ; gouvernance « qui arbitre » (coordinateur vs vérifieur N1-style, « PAS de smart-contract ») ; classe propre aux détecteurs EMA (évasion lente sous-seuil + empoisonnement de baseline).
- **Carries honnêtes à déclarer** (modèle anti-lazy-verifier Phase H, THREAT §16 N1:1140-1154, sanction strictement non-économique, Sev M) : collusion N2 = SI-4 High ASSUMÉ borné par pilote fermé/anti-Sybil ; lazy-arbitre N3 + heuristique SENTINEL non prouvés (N4 zkML hors-scope) ; faux-reject cross-GPU = coût physique assumé ; confidentialité SI-1/SI-4 INCHANGÉE (N2/N3 recomputent/localisent, ne chiffrent rien).

### S4 Wire format
**EXECUTE** (0 BLOCKER/CONCERN). `DOMAIN_ACTIVATION_COMMIT_V1` confirmé absent du code (uniquement en planning) → ajout purement additif, 0-bump (pattern S74 `DOMAIN_SEED_REQUEST_V1`, canonical.rs:255-289). Idiome canonical strict : const domaine + `*_FORMAT_VERSION:u16=1` + champ `version` SANS `#[serde(default)]` (version manquante = malformed) + struct payload UNSIGNED tout-entiers/[u8;32]/String-borné + struct `*Entry` redondante signée JCS (signature + identité redondante JAMAIS dans canonical_bytes) + ordre `version-gate → caps DoS → attribution → crypto` (cap-AVANT-crypto, shard_plan.rs:355-370). **Le slot RunProof.activation_fingerprint N'EST PAS le point d'ancrage N3** : il porte le commitment N0 self-claim global (comparé par égalité, sans nonce ni frontière) — N3 exige une struct `ActivationCommitEntry` séparée. Export à ajouter dans `lib.rs:80` (entre `DOMAIN_AGE_WITNESS_V1` et `DOMAIN_CLAIM_V1`, ordre alpha).

## Recoupement adversarial

**Recoupement croisé des 5 scans — un scan a-t-il raté un conflit qu'un autre suggère ?**

- **S1a (CONCERN N3 commit cross-GPU) vs S4 (INFO commitment [u8;32])** : tension réelle et load-bearing. S4 décrit l'idiome canonical avec `activation_fingerprint:[u8;32]` comme un commitment BLAKE3, et S1a prouve qu'un commit-reveal vérifié par **égalité** de ce commitment faux-rejette cross-GPU. **Résolution** : le commitment 32B stocké/signé reste `BLAKE3(sketch || nonce)` (idiome S4 respecté), MAIS le **reveal transporte le sketch TOPLOC complet** (hors du slot 32B, via raw-op ou champ reveal séparé) et le verdict est `ToplocFingerprint::compare` (S1a). Les deux scans sont compatibles : le commit est byte-exact (binding/hiding), le verdict est tolérant. Vérifié contre `toploc.rs:261-269` (`commitment()`) + `shard_plan.rs:438-445` (la doc du slot RunProof dit déjà explicitement « tolerant recompute lives in compare … once the full sketch is transported off this 32-byte slot »).

- **S2 (redundancy_factor non-signé advisory) vs S3 (calibration seuil = paramètre de sécurité)** : convergent. N2 sélectionné via champ advisory mais verdict dérivé de RunProof signé. Le seuil `TOPLOC_THRESH_*` est un paramètre de sécurité named-const, pas un détail — vérifié `toploc.rs:316-322`.

- **S1a (« O(1) ≠ bissection ») vs plan §12.2/§12.3 test 5 (« bissection O(1 bloc) »)** : le plan contient une **inexactitude de complexité** (oxymore). C'est exactement le pattern doc-honnêteté que Codex a relevé en G/H. À corriger dans les noms de tests + commentaires : le test 5 teste la voie **SENTINEL/EMA (localisation directe O(1), PAS de bissection)**.

**BLOCKER réel ?** Non. **Faux-positifs ?** Aucun — les 5 scans sont factuellement exacts (vérifiés contre validator.rs:117/219, toploc.rs:281-331, canonical.rs:255-310, shard_plan.rs:407-455/355-370, lib.rs:79-87). Le seul « risque » serait d'implémenter N3 comme le plan le LIT littéralement (commit-reveal vérifié par égalité + « bissection O(1) ») → ce serait une primitive non-falsifiable. Le PLAN-ADAPT le neutralise. **Garde-fou unique à tenir à l'écriture** : ne pas éditer le corps 219-338 ni le dispatch 117 de validator.rs ; ne pas attacher de pénalité monétaire au stage SENTINEL localisé (PO-12).

## Consignes d'implementation

### N2 — `crates/nexus-core-rs/src/redundancy.rs` (NET-NEW) + chemin additif `validator.rs`

**Réutiliser `ToplocFingerprint::compare`** (toploc.rs:281), NE PAS réinventer le seuil. N2 = généralisation M-of-N : accepter ssi au moins M des N fingerprints sont **deux-à-deux tolérants** via `compare(...).accepted`.

```rust
// redundancy.rs — 100% no-float, std seul
/// Minimum agreeing fingerprints for an M-of-N tolerant quorum.
/// Reuses ToplocFingerprint::compare (exponent-exact + mantissa-tolerant)
/// — NEVER hash byte-equality. Selection (which tasks use N2) is advisory
/// via redundancy_factor; the ACCEPT/REJECT verdict here rests on
/// SIGNED RunProof-derived fingerprints, never on the unsigned field.
pub fn tolerant_quorum_accepts(
    fingerprints: &[ToplocFingerprint],
    min_agree: usize,            // named-const M, ex. TOLERANT_QUORUM_MIN_AGREE
) -> bool {
    // Construire le plus gros cluster où tous les membres sont
    // deux-à-deux tolerants (compare().accepted), retourner cluster.len() >= min_agree.
    // O(N^2) sur N petit (redundancy_factor borné) — pas de float.
}
```
- Le **chemin additif dans `validator.rs`** est une NOUVELLE fonction (ex. `validate_tolerant_quorum_shard`) appelée UNIQUEMENT pour les tâches shard, JAMAIS depuis le corps de `validate_quorum_pre_guardrail` (219-338) ni le dispatch ligne 117 (qui reste `redundancy_factor>1 → quorum exact result_text`). Le test #3 (git diff=0 hors N2 additif) doit passer.
- Named-const `TOLERANT_QUORUM_MIN_AGREE` (mirror M). NE PAS introduire de nouveau seuil de tolérance : réutiliser `TOPLOC_THRESH_EXP_MISMATCH`/`TOPLOC_THRESH_MANT_MEAN`/`TOPLOC_THRESH_MANT_MEDIAN`.

### N3 — DEUX primitives SÉPARÉES et nommées

**(A) `activation_commit` (opML-style commit-reveal, audit/dispute a posteriori)** — NET-NEW module (`crates/nexus-core-rs/src/activation_commit.rs`).

- Constante `DOMAIN_ACTIVATION_COMMIT_V1: &[u8] = b"nexus-activation-commit-v1"` dans `canonical.rs` (doc-note « purely additive 0-bump, S74 DOMAIN_SEED_REQUEST_V1 pattern »), + `ACTIVATION_COMMIT_FORMAT_VERSION: u16 = 1`.
- Re-export `lib.rs:80` entre `DOMAIN_AGE_WITNESS_V1` et `DOMAIN_CLAIM_V1` (ordre alpha) + structs/version comme shard_plan.
- Struct payload UNSIGNED tout-entiers/[u8;32]/String-borné :

```rust
pub struct ActivationCommitPayload {
    pub version: u16,                          // PAS de #[serde(default)] — version manquante = malformed
    pub worker_pubkey: [u8; PUBLIC_KEY_LENGTH],// croisé avec l'Entry redondant
    pub session_id: String,                    // borné SESSION_ID_MAX=128, anti-replay cross-session
    pub frontier_index: u32,                   // frontière de shard = layer_end (u32, shard_plan.rs:156)
    pub commitment: [u8; 32],                  // = BLAKE3(sketch_canonical_bytes || nonce) — HIDING
}
pub struct ActivationReveal {                  // hors enveloppe signée — transporte le sketch COMPLET
    pub sketch: ToplocFingerprint,             // PAS le commitment seul — re-jugé tolérant
    pub nonce: [u8; 32],                        // haute entropie OsRng, HIDING/anti-dictionnaire
}
```
- **Anti-grinding** : le `commitment` lie des valeurs non choisies par le prover via le seed canonical (`session_id || frontier_index || amont_commit`), même discipline que le VRF N1 (rerun.rs:51-53). Le nonce `[u8;32]` est haute-entropie (HIDING), pas un compteur.
- **Verdict = `ToplocFingerprint::compare` tolérant**, JAMAIS l'égalité de `commitment` BLAKE3. À la phase reveal : recomputer `BLAKE3(reveal.sketch.to_bytes() || reveal.nonce)` == `payload.commitment` (binding), PUIS `prover_sketch.compare(reveal.sketch).accepted` (correctness tolérante cross-GPU).
- Enveloppe `ActivationCommitEntry` = payload + `worker_pubkey` redondant + `signature:[u8;64]` (`#[serde(with=BigArray)]`), signature/identité JAMAIS dans canonical_bytes. `sign()`/`verify_signature()` via `canonical_bytes(&payload, DOMAIN_ACTIVATION_COMMIT_V1)`.
- **Ordre des checks dans `verify_signature()`** : version-gate → caps DoS (`check_*_caps`, ex. borne longueur sketch, SESSION_ID_MAX) → attribution (`payload.worker_pubkey == envelope.worker_pubkey`) → crypto verify (cap-AVANT-crypto, shard_plan.rs:355-370). Caps enforcés à sign ET verify.

**(B) `SENTINEL` (EMA inter-stages statistique, localisation O(1) SANS bissection)** — module/fonction NET-NEW.

- **Inférence forward-only** : l'EMA suit UNE statistique entière sur l'activation forward à chaque frontière inter-stage (ex. norme L1 entière `Σ|activation_i|` ou L2² entière). PAS de gradient backward (sans objet en inférence).
- **Formule EMA tout-entiers, no-float, u128 basis-points** :

```rust
/// EMA in integer basis-points (alpha_bp ∈ [0,10000]), no float.
/// ema_next = (alpha_bp*x + (10000-alpha_bp)*ema_prev) / 10000
const SENTINEL_ALPHA_BP: u128 = 9000; // β_h forward, named-const
fn ema_step(ema_prev: u128, x: u128, alpha_bp: u128) -> u128 {
    (alpha_bp.saturating_mul(x)
        .saturating_add((10000 - alpha_bp).saturating_mul(ema_prev)))
        / 10000
}
```
- **Localisation O(1)** : chaque frontière compare son signal courant à l'EMA locale du stage ; le stage dont la déviation `|x - ema|` dépasse un seuil entier (fence de Tukey approximée en entiers, ou seuil named-const `SENTINEL_DEVIATION_THRESH_BP`) est flaggé directement → PAS de descente de trace. C'est ce qui donne le O(1).
- Vocabulaire load-bearing : NE PAS écrire « bissection … O(1) ». SENTINEL = localisation directe O(1). Une éventuelle bissection opML serait O(log L) (non livrée Phase I, ou test séparé assumant O(log L)).

## Invariants a ne pas toucher

1. **`validate_quorum_pre_guardrail` (validator.rs:219-338) byte-pour-byte INCHANGÉ** + le dispatch `validator.rs:117` (`redundancy_factor>1 → quorum exact result_text`). N2 est un chemin ADDITIF (nouvelle fonction). Test #3 = git diff 0 hors N2.
2. **0-bump wire** : `FEED_FORMAT_VERSION`=1 (raw-op extensible), tous les `*_ANNOUNCEMENT_VERSION`/`*_FORMAT_VERSION` existants intouchés. `DOMAIN_ACTIVATION_COMMIT_V1` purement additif.
3. **0 nouvelle dépendance** : ed25519-dalek 2.2.0 / blake3 1.8.5 / serde_jcs 0.2 suffisent. NI ndarray NI num.
4. **no-float-cœur** : tout-entiers (u16/u32/u64/u128 basis-points), [u8;32] commitments. JCS sign/verify via `canonical_bytes` (le SEUL chemin autorisé).
5. **`redundancy_factor` reste NON-signé** (S23 34c77ce) — verdict de sécurité N2 dérivé du RunProof SIGNÉ, pas du champ advisory.
6. **Slot `RunProof.activation_fingerprint` reste à son usage N0 self-claim** (binding only, comparé par égalité). N3 = struct `ActivationCommitEntry` SÉPARÉE, ne réutilise PAS ce slot (besoin nonce + frontier_index absents de RunProof).
7. **PO-12 non-monétaire** : SENTINEL localise → verdict de correctness/rejet, JAMAIS slash/bond/burn/stake. DOMAIN_KUDOS_V1 / HashableKudosEntry intouchés.
8. **Confidentialité SI-1/SI-4 INCHANGÉE** : N2/N3 recomputent/localisent, ne chiffrent rien (miroir caveat N0/N1).
9. **THREAT_MODEL** : sous-sections §16 N2 + §16 N3 additives (pattern v11/v12) + MAJ §15.2 row I (de « recompute N1/N2 Phase H/I non livré » à N2 câblé). PATTERNS §P66 + bump version THREAT v13.

## Tests attendus

Les 5 tests du plan §12.3, avec les noms/sémantiques alignés sur le PLAN-ADAPT :

1. **`n2_tolerant_quorum_accepts_close_fingerprints`** — N fingerprints cross-GPU proches (exp égal, mantisse < seuil) → M-of-N accepté via `compare().accepted`.
2. **`n2_tolerant_quorum_rejects_divergent`** — fingerprints divergents (exp mismatch ≥ TOPLOC_THRESH_EXP_MISMATCH ou < M dans le cluster) → rejeté.
3. **`validator_exact_quorum_unchanged`** — quorum result_text exact INCHANGÉ : `git diff` = 0 ligne hors N2 additif (corps 219-338 + dispatch 117 intacts).
4. **`n3_activation_commit_reveal_roundtrip`** — commit `BLAKE3(sketch||nonce)` signé `DOMAIN_ACTIVATION_COMMIT_V1` ; reveal `(sketch complet + nonce)` ; binding (recompute commitment == payload.commitment) + verdict tolérant (`prover.compare(reveal).accepted`) ; canonical stable. + assertion cap-AVANT-crypto (« exceeds … before the signature check », pattern shard_plan.rs:773-781).
5. **`n3_sentinel_localizes_corrupted_stage`** — fixture multi-stages : un stage dont le signal forward dévie de l'EMA au-delà du seuil → SENTINEL flagge **directement ce stage (localisation O(1), PAS de bissection)**. Le commentaire/nom du test NE doit PAS revendiquer « bissection O(1) » (oxymore corrigé).

Tests additionnels recommandés (doc-honnêteté Codex G/H) : `n3_commit_reveal_rejects_wrong_nonce` (binding), `n3_commit_reveal_rejects_divergent_sketch_cross_gpu` (le verdict est tolérant mais rejette un vrai swap), `n2_selection_via_advisory_field_does_not_relax_exact_quorum`. T1 E2E : `N-A-no-frontend-change` (aucun changement front).
