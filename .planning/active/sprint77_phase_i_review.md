# Sprint 77 — Phase I review

## Verdict: PASS

Review independante profonde (6 dimensions, code lu reellement via git diff + Read,
recoupement adversarial) : **0 P0, 0 P1, 0 P2/P3 confirme**. Le PLAN-ADAPT du
preflight est applique correctement et integralement. Tous les invariants Day-0
sont tenus, verifies mecaniquement (git diff). Gate Codex GPT 5.5 passe **7/7
CONFIRME, 0 GAP** (cf. `## Codex reconciliation` en fin de document) — promu de
PASS-PENDING a PASS, committable.

## Resume par dimension

| # | Dimension | Verdict | Findings |
|---|---|---|---|
| 1 | CORRECTNESS (N2/N3, diff line-by-line) | PASS | 12 (tous OK) |
| 2 | SECURITE PROFONDE (N2 redondance + N3 commit/sentinel) | PASS | 10 (tous OK) |
| 3 | SCOPE CUTS & INVARIANTS DAY-0 | PASS | 6 (tous OK) |
| 4 | RESEARCH GROUNDING (PLAN-ADAPT applique) | PASS | 6 (tous OK) |
| 5 | TESTS & COUVERTURE SEMANTIQUE | PASS | 12 (tous OK) |
| 6 | PATTERNS & DOC-HONNETETE | PASS | 7 (tous OK) |

Points saillants verifies sains :

- **PLAN-ADAPT N3 honore** : N3 est livre comme **deux primitives separees et
  nommees** — `activation_commit.rs` (commit-reveal opML-style, verdict TOLERANT
  `verify_reveal` jamais egalite-commitment) et `sentinel.rs` (EMA forward-only
  entiere, `localize_corrupted_frontier` localisation **directe O(1)**, pas de
  bissection). L'oxymore « bissection O(1) » du plan §12.2 est explicitement
  corrige en erreur-de-categorie dans le module-doc (sentinel.rs:14-22), THREAT
  §16 (1217-1218) et PATTERNS §P66.
- **N2 = clique d'accord mutuel** (anti-straddle/transitivite) : `fingerprints_agree`
  exige les **deux** directions de `compare` (redundancy.rs:74-76, le compare
  toploc est directionnel) ; `largest_agreeing_cluster` calcule la clique max
  exacte par branch-and-bound borne (`N2_MAX_FINGERPRINTS=32`). Le test
  `n2_clique_defeats_transitivity_straddle` (204-229) prouve qu'un straddler
  n'inflate pas un quorum 3-way inexistant.
- **Chemin additif validator** : `validate_tolerant_quorum_shard` + enum
  `ShardQuorumOutcome` sont **net-new** ; le verdict ne touche ni DB ni etat tache
  (fonction pure). `validate_quorum_pre_guardrail` (corps 220-339) et le dispatch
  (validator.rs:118 sur `redundancy_factor>1`) sont **byte-pour-byte inchanges** —
  confirme : ZERO ligne supprimee dans le diff de validator.rs.
- **Verdict N2 ancre sur du SIGNE** : le filtre (validator.rs:383-386) ne retient
  une soumission que si `entry.verify_signature().is_ok()` (DOMAIN_RUN_PROOF_V1)
  **ET** `sketch.commitment() == entry.proof.activation_fingerprint` (le sketch
  off-envelope ouvre le commitment N0 signe). Une preuve forgee ou un carrier
  altere est ecarte AVANT le vote. `redundancy_factor` (non-signe S23) ne sert
  qu'a la **selection** advisory, jamais a l'accept/reject — verifie par
  `n2_shard_quorum_rests_on_signed_carrier_consistent_inputs` (1099-1140).
- **N3 commit-reveal correct** : binding d'abord (`reveal.opens` = recompute
  `BLAKE3(sketch||nonce)` == commitment), puis correctness **tolerante**
  (`verifier_recompute.compare(reveal.sketch).accepted`), jamais egalite de
  commitment 32B (qui faux-rejetterait tout cross-GPU honnete — avalanche BLAKE3,
  documente activation_commit.rs:28-31). Ordre des checks verify =
  version-gate → caps DoS → attribution → crypto (cap-AVANT-crypto, 206-222),
  enforce a sign ET verify ; teste y compris la signature-forgee-zero
  (n3_commit_caps_reject_oversized_session_id_before_crypto, 491-517).
- **SENTINEL anti-poisoning** : un frontier flagge ne met PAS a jour l'EMA
  (sentinel.rs:138-140), donc un spike transitoire ne tire pas la baseline —
  teste (sentinel_warmup_and_outlier_does_not_poison_baseline, 202-221). EMA
  entiere basis-points u128 saturating, seuil relatif scale-invariant (`|x-ema|*DENOM
  >= thresh_bp*ema`), borne inclusive testee (231-241). Limite honnete (slow-drift
  sous-seuil SI-11 + seuil statique) declaree module-doc + THREAT.

## Findings confirmes (avec severity + fichier:ligne + action)

**Aucun finding P0/P1/P2/P3 confirme** apres verification adversariale.

Tous les findings emis par les 6 dimensions sont des **points verifies sains (OK)**
— ils documentent que les invariants tiennent, pas des defauts. Aucune action
corrective requise avant commit.

## Faux-positifs ecartes (bref)

1. **P2 « pas de test direct du seuil exp_mismatch>=38 dans N2 »** — ECARTE
   (severite surevaluee → faux-positif).
   - Fait exact : aucun test N2 ne pilote `exp_mismatches` jusqu'a la frontiere 38
     (les rejets N2 passent par top-k disjoint → mantissa-stats vides → reject,
     redundancy.rs:175-229).
   - Pourquoi ce n'est PAS une vraie dette : N2 n'ajoute **aucune arithmetique de
     seuil** — `fingerprints_agree` est un wrapper pur sur `toploc::compare`, et la
     frontiere `exp_mismatches < TOPLOC_THRESH_EXP_MISMATCH` (toploc.rs:316) est
     **deja testee a EXACTEMENT 38 (reject) et 37 (accept)** dans
     `compare_threshold_boundaries` (toploc.rs:619-628). Un test N2 dedie
     re-exercerait la meme frontiere deja couverte a travers un wrapper delegant,
     sans toucher la logique propre a N2 (symetrie + clique max), elle directement
     et adversariale-ment testee (straddle, swap, all-divergent, edge-cases). La
     reutilisation du seuil calibre est l'architecture DELIBEREE et DOCUMENTEE
     (redundancy.rs:44-46), pas un trou de couverture. Au plus P3 nice-to-have
     dupliquant une couverture existante.

## Invariants verifies (0 bump wire / 0 dep / no-float / quorum exact inchange / PO-12)

| Invariant | Etat | Evidence |
|---|---|---|
| **0 bump wire** | TENU | `git diff` : aucun `*_FORMAT_VERSION` / `*_ANNOUNCEMENT_VERSION` existant change ; seul ajout = `DOMAIN_ACTIVATION_COMMIT_V1` (additif, canonical.rs:332) + `ACTIVATION_COMMIT_FORMAT_VERSION:u16=1` neuf (activation_commit.rs:69). Pattern S74 DOMAIN_SEED_REQUEST_V1. |
| **0 nouvelle dep** | TENU | `git diff` sur Cargo.toml/Cargo.lock = VIDE. redundancy/sentinel = std seul ; activation_commit reutilise serde + serde_big_array + blake3 + ed25519 deja declares. |
| **no-float-coeur** | TENU | `grep f32/f64` sur les 3 modules core = ZERO hors fixtures de test (`from_topk(f32)` = la frontiere GPU bf16 deja existante, autorisee). EMA = u128 basis-points saturating ; sketch = entiers + [u8;32] ; commitment = BLAKE3 sur bytes entiers. |
| **JCS sign/verify** | TENU | `ActivationCommitEntry::sign`/`verify_signature` via `canonical_bytes(&payload, DOMAIN_ACTIVATION_COMMIT_V1)` (activation_commit.rs:188,220) ; identite redondante + signature JAMAIS dans canonical_bytes ; domaine separe teste (n3_domain_separated_from_run_proof, 519-532). |
| **redundancy_factor NON-signe → verdict sur RunProof SIGNES** | TENU | N2 ne lit jamais le champ pour accept/reject : il filtre sur signature RunProof verifiee + binding commitment N0 (validator.rs:383-386). Selection advisory seulement (doc 364-371). |
| **quorum exact validate_quorum_pre_guardrail INCHANGE** | TENU | ZERO deletion dans le diff validator.rs ; corps 220-339 + dispatch 118 intacts. Sentinel-test `validator_exact_quorum_unchanged` (998+) assere le comportement identique. |
| **PO-12 non-monetaire** | TENU | `grep slash/bond/burn/stake/deposit` = uniquement la **prose d'interdiction** (redundancy.rs:43 « never by an economic stake (PO-12) »), aucun mecanisme. SENTINEL localise → verdict de correctness/rejet, jamais sanction economique. DOMAIN_KUDOS_V1 / HashableKudosEntry intouches. |
| **Slot RunProof.activation_fingerprint reste N0 self-claim** | TENU | N3 utilise une struct `ActivationCommitEntry` SEPAREE (nonce + frontier_index absents de RunProof) ; le slot 32B n'est pas detourne. N2 ne fait que le LIRE pour le binding. |
| **Confidentialite SI-1/SI-4 inchangee** | TENU | N2/N3 recomputent/localisent, ne chiffrent rien (caveat miroir N0/N1, THREAT 1197). |

## Tests (delta + couverture)

- **Suites Windows VERTES (annonce session)** : fmt + clippy --all-targets +
  nextest workspace **1947** + doctests + release. Review n'a pas relance (mandat).
- **Tests net-new Phase I** (lus, couverture semantique verifiee) :
  - `redundancy.rs` (4) : accept clique-3 close (non byte-identique), reject
    divergent + all-divergent, **straddle anti-transitivite**, edge-cases
    (empty/singleton/vacuous min_agree=0).
  - `activation_commit.rs` (8) : roundtrip sign+verify+reveal tolerant +
    JSON re-verify ; reject wrong-nonce/wrong-sketch (binding) ; divergent
    cross-GPU localise ; anti-replay session+frontier (canonical pre-image) ;
    reject wrong-signer + attribution tamper ; reject wrong-version + commitment
    tamper + sig-flip ; **cap-AVANT-crypto** (signature zero forgee → erreur
    « exceeds » et non sig-error) ; domaine-separe vs RunProof.
  - `sentinel.rs` (5) : ema_step entier (fixed-point, alpha 0/10000, saturating
    u128::MAX) ; localise corrupted-stage exact (ni voisin) + healthy=None ;
    warmup + outlier-no-poison ; short-inputs flag-nothing ; frontiere inclusive
    (>=, +50% flagge / +49.x non).
  - `validator.rs` (3 net-new + 1 sentinel-invariant) : `validator_exact_quorum_unchanged`
    (la voie N2 est separee, n'influence pas le quorum exact) ; accept
    close-signed ; **rests-on-signed-carrier-consistent** (carrier-mismatch +
    sig-forgee droppes avant le vote, 2-of-3 accept / 3-of-3 reject).
- **Couverture des branches load-bearing** : binding-fail, tolerant-divergent,
  symetrie clique, anti-straddle, cap-before-crypto, attribution split-brain,
  anti-replay session/frontier, no-poison, frontiere de seuil inclusive,
  drop-pre-vote sur signe-invalide/carrier-altere — toutes couvertes
  adversarial-ement (chaque rejet a une raison distincte testee, pas un
  rouge-trivial).
- **Pas de test legacy-decode zombie** introduit (politique pre-launch respectee).
- **T1 E2E front** : aucun changement frontend (N-A-no-frontend-change) — Phase I
  est 100% Rust core + coordinator + docs.

## Codex reconciliation

Gate Codex GPT 5.5 (CLI externe `codex exec`, cross-model, output brut dans
`sprint77_phase_i_codex_review.md` — non reecrit) execute apres review PASS-PENDING.

- **Verdict Codex : 7/7 livrables CONFIRME, 0 GAP, 0 PARTIEL** au 1er round (CLEAN).
- Codex a verifie chaque livrable avec evidence file:line et a **lance lui-meme les
  tests cibles** (`n2_`, `n3_`, `validator_exact_quorum_unchanged`, `n2_shard_quorum`
  sur core-rs + coordinator-rs) → tous PASS.
- Points prioritaires confirmes par Codex independamment : (a) `validate_quorum_pre_guardrail`
  inchange (« git diff -U0 montre uniquement import + nouvelle fonction/tests, aucune
  hunk dans `validate_quorum_pre_guardrail` ni le dispatch ») ; (b) verdict N2/N3
  TOLERANT (`compare`), jamais egalite de hash/commitment ; (c) « bissection O(1) »
  evite (vocabulaire honnete) ; (d) 0 bump wire / 0 dep (Cargo.toml/lock intacts) ;
  (e) PO-12 non-monetaire.
- **0 GAP P0/P1 a corriger** → pas de boucle de re-verification. Review promu PASS.
