# Review Phase H S77 — N1 VRF spot-check + incentive reputationnel + mapping criticite

## Verdict

**PASS** (review 5 dimensions PASS → Codex 7/7 CLEAN round 2 → promu PASS ; voir
« Codex reconciliation » + « Verdict: PASS » en fin de fichier). L'etat initial de
la review etait **PASS-PENDING** (review OK, Codex pas encore execute — jamais
committable seul) ; il a ete promu apres reconciliation Codex.

Conforme a la regle §4.5.7 : review propre avec >=1 P2 documente, 0 P0/P1 confirme. Le contexte SBFB exige >=1 P2+ pour un PASS-PENDING : satisfait (1 P2 cross-ref docs + 7 P3). Les 5 dimensions sont PASS, tous les invariants imposes par le preflight PLAN-ADAPT (A-F) sont tenus et verifies sur le code reel du working tree (Phase H non commitee).

## Resume par dimension

| Dimension | Verdict | Synthese |
|---|---|---|
| **correctness** | PASS | Lecture ligne par ligne de `verifiable_draw.rs` (NEW) + diffs `verification.rs`/`canonical.rs`/`lib.rs`/`rerun.rs` (REWRITE). Aucun bug de justesse. Selection u128 `value*10000 < rate_bp*2^64` correcte a toutes les bornes (rate 0 ne selectionne jamais meme value=0 ; rate>=denominateur selectionne toujours ; 50% strict-< a 2^63 ; pas d'overflow u128). Indices `output[0..8]`/`h[0..8]` in-bounds sur `[u8;32]`. Separation de domaine saine (draw_message signe vs draw_output hashe ; perso temp/seed distinctes). `spotcheck_creditable` 3 conditions sans condition morte. Fixtures de test verifiees mathematiquement (bf16 exp identiques, mant diff=2, mean=2<10, median_x2=4<16 -> accepted, commitments differents). Mapping criticite pur, jamais N3, exclut priority. Biais modulo temp = 3e-7 (negligeable). build+clippy clean, 751/751 verts dont 4 tests PLAN. |
| **security** | PASS | 6 invariants de securite tenus. (1) Kudos non-monetaire : 0 occurrence cost/deposit/stake/burn/refund/slash/bond ; les "token" sont des LLM-tokens / Token-DiFR ; sanction non-economique verbatim. (2) Grinding : seed non-controlable par le worker verifie (session_id||epoch||result_commitment signes), `vrf_verify` rejette tampering (teste 3 modes). (3) Auto-declaration : `spotcheck_creditable` exige tirage re-verifie + RunProof signe FROM verifier (cross-check pubkey + signature Ed25519) ; cas impostor teste. (4) Honnetete crypto verbatim en code+doc. (5) Token-DiFR ferme la faille "forge tokens puis recalcule fingerprint" (teste). (6) 0 bump wire : DOMAIN_VRF_DRAW_V1 additif, HashableKudosEntry inchange, 0 dep. 1 seul P3 (cast `as u32` non-exploitable). |
| **tests** | PASS | 4 tests §11.3 obligatoires presents ET prouvent leur propriete (determinisme+verifiabilite+redraw ; temp ET seed varient/deterministes/independants ; credit reputationnel reel `db total>0` ; mapping N0/N1/N2 + jamais N3). Couverture adversariale forte : tolerant-vs-byte-equality avec commitments verifies DIFFERENTS, impostor/unselected non-creditable, `vrf_verify` rejette 3 modes, bornes de taux (0/denom/>denom/seuil 50% strict), Token-DiFR forge. ZERO zombie : ancien `rerun.rs` (RerunSampler/DivergenceScorer/simple_hash + 6 tests) entierement supprime, 0 caller orphelin. Caveats d'honnetete presents verbatim. Findings restants = P3 mineurs (completude, pas correctness). |
| **scope-research** | PASS | Conforme integral A-F. (A) `rerun.rs` upgrade in-place coordinator (pas core-rs), VRF sur ed25519-dalek existant via crypto.rs (aucune crate ECVRF), INTERDICTION rand respectee (0 thread_rng/OsRng dans draw/seed/temp ; seul KeyPair::generate en tests). (B) recompute delegue a `ToplocFingerprint::compare` tolerant, pas de transport full sketch, Token-DiFR cable, temp+seed derives deterministes. (C) incentive `kudos_ledger::credit` existant (aucun curator.rs), HashableKudosEntry inchange, gate par spotcheck_creditable. (D) mapping pur sur champs Task signes, 0 champ wire, N3 jamais auto-assigne, N1 independant du tag. (E) carry MEDIAN-DE-GROUPE non absorbe, validator.rs absent du diff. (F) note honnetete verbatim. Research grounding fidele (VeriLLM 2509.24257, DiFR 2511.20621) et NON overstated (reconnait ne PAS repliquer le slashing economique, PO-12). |
| **patterns-docs** | PASS | Documentation fidele au code reel, honnete sur les non-garanties, conforme au verdict preflight. Tous invariants tenus : 0 bump wire (DOMAIN_VRF_DRAW_V1 additif, HashableKudosEntry inchange confirme diff vide), 0 dep (rand absent du tirage), no-float dans le coeur, une seule table de taux named-const (spot_check_rate_bp source entiere, spot_check_rate f64 derivee). Symboles re-exportes existent et compilent (build 2 crates exit 0, 0 warning). 4 tests §11.3 + 8 adversariaux. Primitives N1 sans call-site in-vivo (grep confirme), exactement ce que les docs claiment. Un seul P2 (cross-ref §P64.1/§P64.3 imprecis). |

## Findings confirmes

### P2 — PATTERNS §P65 cite des ancres §P64.1 / §P64.3 inexistantes (dangling cross-ref)
- **Fichier** : `docs/rust/PATTERNS.md:3637, 3643`
- **Severite** : P2 (cosmetique, non-bloquant, reference recuperable par contexte)
- **Constat** : ligne 3637 « (§P64.1, §P60.2) » et ligne 3643 « (cf. §P64.3) ». §P64 commence a L3552 et est structure en **liste numerotee a plat** (items 1.–4. a L3559/3571/3580/3587), PAS en sous-sections a en-tete. Grep confirme : les seules sous-sections numerotees de cette zone sont §P60.1/§P60.2/§P60.3 ; aucune ancre §P64.1/§P64.2/§P64.3 n'existe (grep des litteraux ne renvoie QUE les 2 lignes citantes 3637/3643). Les autres cross-refs de §P65 (§P60.2 L3382, §P61 L3422) sont valides. Le contenu vise est correct (§P64 item 1 = decision commitment-equality/false-reject ; item 3 = all-integer bf16 « worker floats only at the GPU boundary »).
- **Correction** : remplacer « §P64.1 » par « §P64 (item 1) » et « §P64.3 » par « §P64 (item 3) », ou simplement « §P64 ».

### P3 — Biais modulo dans derive_spotcheck_temp_milli (~3e-7, negligeable, non-securitaire)
- **Fichier** : `crates/nexus-core-rs/src/verifiable_draw.rs:143-147`
- **Severite** : P3 (note de tracabilite, aucun fix requis)
- **Constat** : `raw % VRF_MAX_TEMP_MILLI` avec VRF_MAX_TEMP_MILLI=2000, raw u32 BLAKE3. 2^32 mod 2000 = 1296 -> non-uniformite absolue ~3.0e-7. Non-securitaire : l'imprevisibilite crypto vit dans la proof Ed25519 + l'output BLAKE3 ; un attaquant devrait predire `output` AVANT de produire la proof (precisement ce que le VRF empeche), donc le biais ne confere aucun avantage. **Nuance** : le rationale original disait le biais « deja documente acceptable » — FAUX (grep modulo/bias dans verifiable_draw.rs = 0 match) ; la seule correction honnete serait au wording du finding, pas au code.

### P3 — token_agreement `as u32` cast tronque les compteurs au-dela de u32::MAX
- **Fichier** : `crates/nexus-coordinator-rs/src/rerun.rs:119, 124`
- **Severite** : P3 (footgun latent microscopique, non-bloquant)
- **Constat** : `prover_tokens.len().max(replay_tokens.len()) as u32` (L119) et `.count() as u32` (L124) — narrowing usize->u32 non gardes. Irrealiste (>4.29G tokens = ~17 Go contigus, hors de toute context window). Pas de surface : (a) `pub` mais aucun caller in-vivo (grep confirme rerun.rs seul referent) ; (b) la consommatrice `tokens_agree` elargit en u64 avant la multiplication (0 overflow arithmetique) ; (c) symetrie .max() + seuil 95% : une troncature ne permet pas de passer la verif.
- **Correction (optionnelle, forward Phase I/K)** : passer total/matching en u64 OU cap longueur a la TOPLOC_TOP_K quand le transport sera cable (precedent toploc.rs:65-66,224-235).

### P3 — assert_ne!(t1, t2) sur temp dans [0,2000) flaky ~1/2000 (deja flaky via keypair aleatoire)
- **Fichier** : `crates/nexus-core-rs/src/verifiable_draw.rs:265` (test `n1_spot_check_randomizes_temp_and_seed`, L252-279)
- **Severite** : P3 (qualite de test, impact faible)
- **Constat** : `assert_ne!(t1, t2)` sur deux valeurs dans [0,2000) -> collision 1/2000. **Cause corrigee vs rationale original** : les seeds sont hardcodes MAIS `kp = KeyPair::generate()` (L253) tire de OsRng (verifie crypto.rs:66-71), donc d1.output/d2.output sont deux valeurs aleatoires independantes a CHAQUE run -> le test est DEJA flaky ~0.05%/run, pas conditionne a un futur refactor. Le sibling `assert_ne!(s1, s2)` (seed u64, 2^64) est OK.
- **Correction (optionnelle)** : asserter borne+determinisme (deja couverts L262-264, 270) et comparer les outputs complets (32 octets) plutot que la valeur mod-2000 pour l'inegalite cross-draw.

### P3 — Non-monetarite non assertee positivement dans incentive_credits_reputation_on_honest_spotcheck
- **Fichier** : `crates/nexus-coordinator-rs/src/rerun.rs:221-247`
- **Severite** : P3 (ecart plan-vs-implementation, sans impact correctness/securite/wire)
- **Constat** : le preflight §11.3 test #3 demandait verbatim « assert 0 terme monetaire ». Le test livre n'assert que `spotcheck_creditable == true` et `get_project_kudos_total > 0`. Refutation tentee (echouee) : il n'existe AUCUN type/champ monetaire dans le systeme contre lequel asserter « = 0 » ; la non-monetarite repose structurellement sur (a) reuse de kudos_ledger::credit non-transferable, (b) 0 champ ajoute a HashableKudosEntry (pre-image DOMAIN_KUDOS_V1 figee, confirme kudos_ledger.rs hors diff), (c) 0 stake/burn/bond/slash (grep = 0 match). L'invariant PO-12 est tenu par construction.

### P3 — tokens_agree : cas verdict-level "replay-longer" et "one-empty/other-non-empty" non testes
- **Fichier** : `crates/nexus-coordinator-rs/src/rerun.rs:118-134, 323-329`
- **Severite** : P3 (completude de test, fonction structurellement symetrique)
- **Constat** : les tests booleens `tokens_agree` (L308-320) couvrent exact/forged/PROVER-longer-truncation/both-empty ; la direction replay-longer n'est exercee qu'au niveau tuple `token_agreement` (L327), jamais propagee a travers le bool. Le cas asymetrique `tokens_agree(&[], &[1,2], pct)` (total=2, matching=0 -> false) est une combinaison de branches non testee. Aucun defaut de code (`total = len().max(len())` + zip symetrique -> replay-longer == prover-longer, one-empty(A) == one-empty(B)) : pur gap mirror-inputs.

### P3 — Cast as u32 sur len()/count() dans token_agreement (doublon dimension securite)
- **Fichier** : `crates/nexus-coordinator-rs/src/rerun.rs:119, 124`
- **Severite** : P3 — meme defaut que ci-dessus, signale aussi cote securite ; confirme benin (tokens_agree elargit en u64 avant multiplication ; pas de cablage in-vivo ; symetrie .max()).

## Findings refutes

Aucun finding refute. Les 8 findings remontes ont tous ete confirmes factuellement sur le code reel apres tentative de refutation adversariale. Les seules « refutations » sont des corrections de rationale (P3 biais-modulo : pas « deja documente » ; P3 flaky-assert : flaky DEJA via keypair aleatoire, pas via futur refactor) qui ne changent pas la severite ni la realite du defaut.

## Invariants verifies

| Invariant | Statut | Preuve |
|---|---|---|
| **Kudos non-monetaire** (PO-12, Day-0 #7) | TENU | 0 occurrence cost/deposit/stake/burn/refund/slash/bond dans le code Phase H ; reuse `kudos_ledger::credit` existant ; sanction non-economique verbatim (rerun.rs:29-32, THREAT §16, PATTERNS §P65 pt 5) ; aucun module curator cree. |
| **0 bump wire** | TENU | `DOMAIN_VRF_DRAW_V1` purement additif (pas un *_FORMAT_VERSION), canonical.rs:289+. HashableKudosEntry INCHANGE (kudos_ledger.rs hors diff). Aucun slot/struct wire modifie. THREAT §15.2 v12 + §16 confirment 0-bump. |
| **0 dep nouvelle** | TENU | `git diff --name-only` ne liste AUCUN Cargo.toml/Cargo.lock. VRF construit sur ed25519-dalek existant via crypto.rs ; rand absent du chemin de tirage (seul KeyPair::generate -> OsRng en tests). |
| **Named constants (1 seule table de taux)** | TENU | SPOT_CHECK_RATE_*_BP + TRUST_TIER_* = source de verite entiere ; `spot_check_rate` f64 derive via `spot_check_rate_bp(...)/DENOMINATOR` (test `spot_check_rate_bp_is_the_integer_source_of_truth` prouve l'unicite de table). VRF_RATE_DENOMINATOR/VRF_MAX_TEMP_MILLI named. |
| **No-float dans le coeur** | TENU | verifiable_draw.rs : 0 f32/f64 (draw/compare/select all-integer u64/u128/u32). Seul `spot_check_rate` f64 = vue humaine documentee ; `from_topk` f32 = frontiere GPU (mirror toploc bf16_bits). |
| **Honnetete verbatim** | TENU | « NOT an ECVRF (RFC 9381) » + « Ed25519 is malleable » + « uniqueness ... not cryptographically proven » + « unpredictability is not proven » + « mitigation, not a guarantee » + « no anti-lazy-verifier defense » + « re-exec prefill GPU + transport sketch gated Phase I/K » presents en CODE (verifiable_draw.rs:27-37, rerun.rs:26-32) ET DOC (THREAT §16 N1, PATTERNS §P65 pts 2/5). |
| **Carry MEDIAN-DE-GROUPE non absorbe (E)** | TENU | validator.rs / validate_quorum_pre_guardrail ABSENT du diff (`git diff --name-only` : seuls rerun.rs/canonical.rs/lib.rs/verification.rs/PATTERNS.md/THREAT_MODEL.md). |
| **Primitives sans cablage in-vivo (gate Phase I/K)** | TENU | grep : seuls callers de vrf_*/spotcheck_*/token*/criticality_* = leurs definitions + tests #[cfg(test)] + re-exports lib.rs ; 0 caller de prod. Zombies RerunSampler/DivergenceScorer/simple_hash supprimes (0 code, seulement doc-comments de remplacement). |

## Suites (a remplir par le main thread)

Verifications locales executees pendant cette review (crates touches uniquement) :
- **nextest (Win, p=core+coordinator)** : 751/751 PASS, 0 skip — dont les 4 tests §11.3 (`n1_vrf_selects_deterministic_verifier`, `n1_spot_check_randomizes_temp_and_seed`, `incentive_credits_reputation_on_honest_spotcheck`, `criticality_maps_to_verification_level`).
- **fmt** : `cargo fmt --all --check` exit 0.
- **clippy** : `cargo clippy -p nexus-core-rs -p nexus-coordinator-rs --all-targets --locked -- -D warnings` exit 0.

A completer par le main thread AVANT commit (gate dual-platform §7.4) :
- [ ] **Win nextest --workspace --locked** (delta cumule attendu ~+16 depuis 1900 = ~1916, a confirmer)
- [ ] **Docker canonique rust:1.94** (`sbfb-ci`) nextest workspace Linux
- [ ] **clippy --workspace --all-targets** (workspace complet, pas seulement les 2 crates)
- [ ] **cargo test --workspace --locked --doc** (doctests)
- [ ] **cargo build -p nexus-shell-daemon --release**
- [ ] **fmt --all --check sous les 2 toolchains** (Win + Docker 1.94)

## Reste pour commit (Codex)

PASS-PENDING n'est **jamais committable seul**. Avant le commit atomique Phase H :
1. **Codex GPT5.5** (`codex exec`, gate BLOQUANTE review->commit) — output brut dans `codex_review.md`, jamais reecrit/condense. Resoudre tout GAP avant commit OU le tracker explicitement (META-1).
2. Optionnel non-bloquant : corriger le P2 cross-ref §P64.1/§P64.3 dans PATTERNS §P65 (1 edit doc) — recommande avant commit pour eviter un carry cosmetique.
3. Les 7 P3 sont des notes de tracabilite / completude / forward-Phase-I-K ; aucun ne bloque le commit (a router en carry si non corriges).
4. Suites dual-platform vertes AVANT push (cf. section Suites).

## Corrections in-phase (post-review, pre-Codex)

Le main thread a corrige le P2 et 4 des 7 P3 EN-PHASE (avant Codex) ; tests
re-verts apres (core 751/751, coordinator 329/329, clippy/fmt 0) :

- **P2 cross-ref docs** (CORRIGE) — PATTERNS §P65 : « §P64.1 » → « §P64 (item 1) »,
  « §P64.3 » → « §P64 (item 3) » (§P64 est une liste numerotee a plat, sans
  sous-ancres).
- **P3 flaky temp** (CORRIGE) — `verifiable_draw.rs` test
  `n1_spot_check_randomizes_temp_and_seed` : le `assert_ne!(t1, t2)` (~1/2000
  flaky via keypair OsRng) est remplace par un echantillon de 8 tirages distincts
  asserte non-constant (`HashSet.len() > 1`, faux-negatif ~(1/2000)^7).
- **P3 cast u32** (CORRIGE) — `rerun.rs` `token_agreement` retourne `(u64, u64)`
  (lossless depuis `usize`), supprimant la troncature theorique >u32::MAX ; ferme
  les 2 findings doublons (correctness + securite).
- **P3 non-monetarite non assertee** (CORRIGE) — `rerun.rs`
  `incentive_credits_reputation_on_honest_spotcheck` : ajout d'assertions positives
  (exactement 1 ligne de reputation creditee depuis un solde initial nul, montant
  > 0, rien n'est depense/debite — preuve structurelle non-monetaire PO-12).
- **P3 cas tokens non testes** (CORRIGE) — `rerun.rs` `token_difr_catches_forged_tokens`
  : ajout des cas replay-plus-long, une-seule-vide(A), une-seule-vide(B).

Carries P3 NON corriges (notes de tracabilite, aucun fix code requis, a mentionner
en commit body) :

- **P3 biais modulo temp ~3e-7** — non-securitaire (l'imprevisibilite vit dans la
  proof Ed25519 + output BLAKE3 ; le biais ne confere aucun avantage a un attaquant
  qui devrait predire l'output AVANT de produire la proof). Documente, aucun fix.

## Codex reconciliation

Codex GPT5.5 (`codex exec`, output brut `sprint77_phase_h_codex_review.md`, non
reecrit) execute en 2 rounds :

- **Round 1** : 3 CONFIRME + 4 PARTIEL + 0 GAP. Triage :
  - PARTIEL L2 (zombies textuels `RerunSampler`/`simple_hash`/`DivergenceScorer`)
    et PARTIEL L4 (mots `slash/bond/burn/stake`) = **faux-positifs grep-litteral** :
    les symboles n'existent plus qu'en doc-comments expliquant le remplacement ; les
    termes monetaires n'apparaissent que dans la prose d'honnetete qui les INTERDIT.
    Aucun mecanisme reel. (La review l'avait deja valide.)
  - PARTIEL L5 + L7 (`redundancy_factor` qualifie de « signe ») = **VRAI finding
    doc-accuracy**. `redundancy_factor` est exclu des bytes canoniques signes
    (Sprint 23 `34c77ce`) ; seul `verifiable` est signe. **CORRIGE EN-PHASE** :
    verification.rs (doc enum + fn), THREAT_MODEL §16, PATTERNS §P65 reformules —
    le niveau est ADVISORY vis-a-vis de la redondance, le minimum LIANT est impose
    par la policy consommateur/groupe (jamais fait confiance au hint non-signe).
- **Round 2** (apres fix + re-run suites) : **7/7 CONFIRME, 0 GAP, 0 PARTIEL —
  CLEAN**. Provenance honnete `redundancy_factor` non-signe confirmee par Codex
  (`task.rs:39-43` retire le champ des canonical bytes ; `verifiable` signe
  `task.rs:232-239`). Tous les livrables 1-7 confirmes avec evidence fichier:ligne.

## Verdict: PASS

Review 5 dimensions PASS + Codex 7/7 CLEAN (round 2). 0 P0/P1. P2 + 4/5 P3 corriges
en-phase ; 1 P3 (biais modulo ~3e-7) documente non-securitaire, aucun fix. Pret au
commit apres gate dual-platform vert.