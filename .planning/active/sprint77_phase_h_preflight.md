# Sprint 77 Phase H — Preflight G8 (N1 VRF spot-check + incentive curator-reputation + mapping criticite->niveau)

## Verdict

**PLAN-ADAPT**

Le design de fond de la Phase H est valide par la SOTA 2025 (VeriLLM, DiFR) et ne touche AUCUNE decision Day-0 (D1-D5 gelees, invariant kudos non-monetaire PO-12 tenu) — donc EXECUTE sur l'intention. Mais 4 adaptations d'implementation sont requises, toutes evidence-backed et verifiees in-repo : (1) drift fichier `rerun.rs` ; (2) VRF build-from-Ed25519 (pas de crate ECVRF) ; (3) reconciliation du taux `spot_check_rate` pre-existant ; (4) mapping criticite NET-NEW derive de champs signes existants. 0 bump wire atteignable, 0 dep nouvelle.

Signaux des 5 scans : S1a PLAN-ADAPT, S1b EXECUTE (deps seul), S2 PLAN-ADAPT, S3 PLAN-ADAPT, S4 EXECUTE (wire seul). Les deux EXECUTE sont scopes etroitement (deps / wire) et flaggent EUX-MEMES le drift `rerun.rs` et le choix VRF-from-Ed25519 -> le verdict global est PLAN-ADAPT.

## Resume des 5 scans

### S1a — Delta OSS / SOTA (PLAN-ADAPT)
Le design (VRF-select + prefill ~1% + one-honest-verifier + randomiser temp&seed) est l'implementation directe de **VeriLLM** (arXiv:2509.24257 §2.2 G3 "correctness guaranteed as long as at least one verifier executes honestly" ; §4 prefill = single parallel pass ; Eq.2 Cverify/Cinfer = Tprompt/(Tprompt+Toutput) ≈ 1%) et **DiFR** (arXiv:2511.20621 "Inference Verification Despite Nondeterminism", Karvonen et al. ; "with a fixed sampling seed, over 98% of tokens match exactly"). Trois drifts : (a) `rerun.rs` existe dans nexus-coordinator-rs (pas core-rs, pas net-new), dead-code, selection `simple_hash(task_id)` non-verifiable -> UPGRADER ; (b) `verification.rs::spot_check_rate` (1/5/20%) chevauche le N1 1-5% + mapping criticite -> reconcilier ; (c) aucune crate ECVRF ne fitte ed25519-dalek (vrf=OpenSSL/secp256k1, vrf-r255=ristretto255, ark-vrf=arkworks) -> build-from-Ed25519. Note honnete sur l'incentive explicitement OUVERTE dans addendum §8.1 (VeriLLM s'appuie sur stake-slashing, interdit par kudos non-monetaire). **Activation-DiFR ne verifie pas le sampling** (worker forge des tokens puis calcule le fingerprint post-hoc) -> comparer aussi les tokens sous seed partage.

### S1b — Dependances (EXECUTE, scope deps)
0 nouvelle dep. VRF construit sur `ed25519-dalek` 2.1 deja present (`crypto.rs:96` sign deterministe RFC 8032, verifie ; `curve25519-dalek` 4.1.3 transitif). seed/temp derives deterministes du VRF (BLAKE3/sha2 presents). Incentive via `kudos_ledger::credit()` existant (`kudos_ledger.rs:76-124`, non-monetaire grave dans le module doc:2-4). Crate ECVRF dediee DECONSEILLEE (arbre crypto concurrent + maintenance + conflit dalek). `rand` 0.8 present mais le hasard du tirage vient du VRF (deterministe-verifiable), PAS de rand. Doublons dalek/rand du lock = 100% iroh 0.98 (Day-0 gele), hors-perimetre. Note : `rerun.rs` annonce net-new par le plan mais existe dans coordinator-rs.

### S2 — Historique decisionnel (PLAN-ADAPT)
Ancrage solide : kickoff D4 (scope MAXIMAL override PO 2026-06-20), design_review D4, addendum §89/§8, THREAT_MODEL §16. Aucun DESIGN-CONFLICT de fond. Drift fichier confirme (`ls crates/nexus-core-rs/src/rerun.rs` = absent ; present dans coordinator avec `sample_rate: f64` + `DivergenceScorer` byte-equality). Precedent direct VRF = Phase D `placement.rs:313 sampling_key = blake3(session_id||pubkey)` deterministe non-lexicographique (closure SYBIL-SEEDER-TAIL). Invariant kudos HARD-confirme code+docs+scope-cut #11. Carry MEDIAN-DE-GROUPE = DOC-P2 DEFERRED (§15.3, infaisable sans casser "validator INCHANGE"). verification.rs Layer-3 N0 cable par egalite (Phase G) ; `RunProof.activation_fingerprint` reel mais self-claim tant qu'un verifieur ne RECOMPUTE pas -> Phase H = exactement ce recompute via `ToplocFingerprint::compare`.

### S3 — Threat model (PLAN-ADAPT)
La row N0 §15.2 (ligne 918, verifiee) est deja conditionnee a "emission + recompute N1/N2 = Phase H/I" -> Phase H MET A JOUR cette row (M->L sur swap-detection pour l'echantillon tire, caveat 1-5%/prefill-only maintenu), ne cree pas une row ex-nihilo. La sous-section §16 "Incentive a verifier" EXISTE deja (ligne 1086, Sev M, verifiee). 3 surfaces N1 a ajouter : (1) predictibilite/grinding du tirage (le `simple_hash` public est l'anti-pattern a fermer) ; (2) farming kudos + collusion/Sybil worker<->verifieur ; (3) criticite auto-declaree pour echapper a N2. Drifts : pas de `curator.rs` (ls confirme, l'incentive plugge sur `kudos_ledger`) ; `Task` n'a que `priority`/`verifiable`/`redundancy_factor`, pas de champ criticite (grep=0). Nuance crypto : Ed25519 n'est PAS une VRF unique RFC9381 -> cadrer "tirage verifiable Ed25519", documenter grinding/self-selection. Comparaison TOLERANTE (jamais egalite stricte a temp>0).

### S4 — Wire invariants (EXECUTE, scope wire)
0-bump ATTEIGNABLE. Le N1 est local/coordinateur : tirage recalculable depuis donnees deja signees ; re-execution prefill-only produit un `ToplocFingerprint` compare LOCALEMENT via `ToplocFingerprint::compare` (toploc.rs:281, verifie : signature `compare(&self, replay) -> ToplocComparison`) contre le commitment N0 deja transporte dans `RunProof.activation_fingerprint` ([u8;32], shard_plan.rs:446) / `ResultPayload.logprobs_hash`. Incentive via op `DOMAIN_KUDOS_V1` existante (canonical.rs:89). Mapping criticite derive de champs Task signes (verifiable/redundancy_factor/priority), 0 wire. **Nuance critique** : le commitment N0 est binding-only (compare par EGALITE) ; le `compare()` tolerant exige le full sketch des DEUX cotes -> le verifieur N1 RECALCULE son propre sketch (mode (a), 0 transport). NE PAS transporter le full sketch du prover. Drift `rerun.rs` re-confirme (coordinator, f64+hash-equality).

## Approche validee (corrigee — PLAN-ADAPT)

### A. Selection VRF (tirage du verifieur)
- **Emplacement** : `crates/nexus-coordinator-rs/src/rerun.rs` (UPGRADE in-place, PAS de fichier dans core-rs). Remplacer `should_rerun()->simple_hash(task_id)` (rerun.rs:39-45) — `BLAKE3(task_id)` est publiquement predictible (un worker sait ex-ante s'il sera spot-checke).
- **Primitive** (dans `crypto.rs` ou `nexus-core-rs/src/verifiable_draw.rs` re-exporte) :
  - `vrf_draw(signer: &KeyPair, seed: &[u8]) -> ([u8;64], [u8;32])` — `proof = KeyPair::sign(DOMAIN_VRF_DRAW_V1 || seed)` (crypto.rs:96, RFC 8032), `output = BLAKE3(DOMAIN_VRF_DRAW_V1 || proof)`.
  - `vrf_verify(pubkey: &[u8;32], seed: &[u8], proof: &[u8;64]) -> Result<[u8;32]>` — `crypto::verify` puis recompute output.
  - `seed` = octets DEJA SIGNES non-controlables par le worker verifie : `session_id || epoch || result_commitment` (depuis `ShardedSessionManifest`/`RunProof`). JAMAIS un input choisi par le worker (surface grinding).
  - selection = `output` mappe modulo `|pool honnete|` (index dans `worker_pubkeys`), comparaison ENTIERE (no-float), modele = `placement.rs:313 blake3(session_id||pubkey)` (Phase D).
  - **INTERDICTION** : aucune source `rand::thread_rng`/`OsRng` pour le tirage NI le temp/seed du spot-check (cantonner rand a `KeyPair::generate`, exemption P2-A-1).
- Si `DOMAIN_VRF_DRAW_V1` ajoute : constante domain-separation (canonical.rs), PAS un `*_FORMAT_VERSION`, PAS un bump wire. Une eventuelle proof transportee = raw-op extensible `serde_json::Value` (0 bump).

### B. Recompute + compare (verification reelle)
- Re-execution prefill-only -> `ToplocFingerprint` cote verifieur (modele+prompt via `prompt_profile_hash`). Comparaison via `crates/nexus-core-rs/src/toploc.rs::ToplocFingerprint::compare(&self, replay)` (toploc.rs:281) — JAMAIS commitment-equality (toploc.rs:31-40 ; un BLAKE3 ne tolere rien).
- **Mode a ancrer dans le commit body** : N1 = "le verifieur recompute SON sketch et compare" ; le slot `activation_fingerprint`=[u8;32] ne porte que le commitment du prover (binding-only) ; le verifieur recalcule son sketch localement -> le commitment N0 detecte un prover INCOHERENT avec son propre claim. NE PAS transporter le full sketch du prover (~768 B/32 tok).
- Anti-DiFR : temp ET seed du spot-check derives DETERMINISTES du vrf output (ex. `BLAKE3(vrf_output||"temp")`, `BLAKE3(vrf_output||"seed")`), reproductibles par auditeur, imprevisibles par le worker avant selection. Comparer aussi les TOKENS sous seed partage (Token-DiFR), pas seulement l'activation-fingerprint.

### C. Incentive reputationnel
- Plugger sur `kudos_ledger::credit` (kudos_ledger.rs:76-124) — AUCUN `curator.rs` n'existe (ls confirme). Credit via `task_id`/`project_id` existants.
- NE PAS ajouter de champ `reason=spotcheck` a `HashableKudosEntry` (kudos_ledger.rs:50-58) -> changerait le pre-image `DOMAIN_KUDOS_V1` = bump implicite interdit. Forme on-wire `KudosEntry` INCHANGEE.
- Credit CONDITIONNE a un spot-check VERIFIABLE (RunProof N1 du verifieur signe + `compare()` pass), jamais a une auto-declaration.
- Sanction faux/lazy verifieur = STRICTEMENT non-economique (trust-delta -5 / non-credit, jamais slash/bond/burn — invariant kudos FIGE). Carry honnete : pas de defense game-theoretique anti-lazy-verifier (VeriLLM la tire du slashing, interdit ici).

### D. Mapping criticite -> niveau
- Fonction PURE Rust `criticality_maps_to_verification_level` sur champs Task SIGNES existants (`verifiable: bool`, `redundancy_factor: u8`, `priority: u8` — task.rs:183/247/280) ; AUCUN champ `criticality` au wire (grep criticit=0). Placement : verification.rs ou module mapping co-localise.
- Regle (addendum §3) en NAMED CONSTS, RECONCILIANT `spot_check_rate` (verification.rs:274) en une seule table : haute (`verifiable && redundancy_factor>1`) = N2 obligatoire ; faible = N0 seul ; N1 = echantillonnage VRF 1-5% ; N3 sur litige.
- Regle non-falsifiable : niveau MINIMAL impose par la policy du ComputeGroup / le consommateur, PAS auto-declare par l'initiateur. N1 (tirage VRF) s'applique INDEPENDAMMENT du tag de criticite.

### E. Carry MEDIAN-DE-GROUPE
DOC-P2 DEFERRED (§15.3). L'incentive touche le scoring -> candidat d'absorption. NE PAS toucher `validate_quorum_pre_guardrail` (validator INCHANGE, invariant fige). Soit durcir doc/scoring, soit reconduire honnetement le carry. Commit body §11.5 prevoit deja "Carry MEDIAN-DE-GROUPE".

### F. THREAT_MODEL
MAJ §15.2 row I (ligne 918/1187-1189 : M->L swap-detection POUR L'ECHANTILLON, caveat 1-5%/prefill-only maintenu) + etendre §16 "Incentive a verifier" (ligne 1086 : "concu"->"cable") + 3 surfaces N1 Sev M (predictibilite-tirage/grinding ; farming-kudos+Sybil-verifieur ; criticite-auto-declaree-echappe-N2) + reaffirmer SI-1/SI-4 High INCHANGES (N1 ne chiffre rien) + note DiFR randomise-temp+seed-cote-verifieur, comparaison TOLERANTE jamais egalite stricte a temp>0.

## Garde-fous & invariants

- **Kudos non-monetaire/non-transferable** : 0 occurrence cost/deposit/stake/burn/refund/achat dans le code et la doc Phase H (invariant fige PO-12, hook scan). Sanction = trust-delta/non-credit, jamais economique.
- **Note honnete obligatoire** (verbatim) : "mitigation reputationnelle, PAS garantie economique" + 4 NON-garanties (prefill-only, echantillon 1-5%, pas de confidentialite, reputationnel non-economique) + "PAS un ECVRF RFC 9381 : uniqueness/unpredictability non prouvees, Ed25519 malleable" + "pas de defense game-theoretique anti-lazy-verifier".
- **0 bump wire** : slot `[u8;32]` deja reel (Phase G), op `DOMAIN_KUDOS_V1` existante, mapping derive de champs signes ; aucun `*_FORMAT_VERSION`, aucun nouveau slot. Une eventuelle proof N1 = raw-op extensible.
- **0 dep nouvelle** : ed25519-dalek 2.1 + blake3 suffisent. Crate ECVRF DECONSEILLEE.
- **Named constants** (regle S76) : une seule table de taux N0/N1/N2/N3 + criticite (pas de magic numbers, pas de 2e table divergente).
- **no-float core/JCS** : la primitive de tirage et le mapping sont all-integer ; ne pas re-importer le `f64`/`DivergenceScorer` du coordinator dans le chemin de comparaison (le `compare()` toploc est all-integer).
- **rand interdit** dans le tirage/seed (verifiabilite).
- **validator INCHANGE** (`validate_quorum_pre_guardrail`) : invariant fige Phase D/I, ne pas toucher.

## Plan de tests

Les 4 tests du plan §11.3 (obligatoires) :
1. `n1_vrf_selects_deterministic_verifier` — meme (signer, seed) -> meme verifieur tire ; `vrf_verify` accepte la proof ; tirage different pour seed different. Hermetique, sans GPU.
2. `n1_spot_check_randomizes_temp_and_seed` — temp ET seed derives du vrf output, deterministes cote verifieur, distincts d'un run a l'autre (seed different).
3. `incentive_credits_reputation_on_honest_spotcheck` — `credit()` appele apres `compare()` pass ; assert credit reputationnel ; assert 0 terme monetaire ; pas de nouveau champ dans `HashableKudosEntry`.
4. `criticality_maps_to_verification_level` — haute (verifiable+redundancy>1)=N2 ; faible=N0 ; N1 1-5% ; une seule table named-const.

Tests adversariaux suggeres :
5. `vrf_seed_not_worker_controlled` — un seed force par le worker ne biaise pas la selection (grinding).
6. `lazy_verifier_no_credit_without_proof` — credit refuse si le spot-check N1 n'est pas verifiable (RunProof manquant/invalide).
7. `false_verifier_sanction_is_non_monetary` — sanction = trust-delta/non-credit, assert absence de slash/bond.
8. `criticality_downgrade_does_not_escape_vrf_sampling` — N1 s'applique meme tag faible-criticite.
9. `forged_tokens_caught_by_token_difr` — un prover qui forge des tokens puis calcule le fingerprint echoue la comparaison de tokens sous seed partage.

Acceptation §11.4 : `cargo nextest -p nexus-core-rs -p nexus-coordinator-rs` verts ; VRF deterministe ET verifiable ; incentive credite reputationnel (jamais monetaire). Re-exec prefill-only GGUF = `#[ignore]`-gated rig (precedent F2) ; la primitive (vrf_draw/verify, derivation temp/seed, compare via toploc, mapping, credit) doit etre 100% hermetique sous CI sans GPU.

## Risques residuels

1. **VRF non-RFC9381** : determinisme+verifiabilite OK, mais pas d'uniqueness/unpredictability prouvee (Ed25519 malleable). Acceptable pour spot-check 1-5% one-honest-verifier (mitigation). Si la review exige un ECVRF formel = new dep lourde -> arbitrage PO (hors §11).
2. **Grinding/self-selection** : seed derive de donnees signees non-controlables par le worker ; documenter §16 + test adversarial.
3. **Activation-DiFR ne verifie pas le sampling** : comparer aussi les tokens (Token-DiFR) sinon randomisation temp+seed sans effet -> faux-vert.
4. **Sur-promesse / faux-vert review** : ecrire les 4 NON-garanties verbatim (recurrent S77 : SI-3 overstate Phase E, doc-honnetete Phase G).
5. **MEDIAN-DE-GROUPE** : ne pas toucher le quorum/validator (invariant fige) ; reconduire le carry si l'absorption casse l'invariant.
6. **Tests GPU-gated** : la primitive doit etre hermetique sans GPU (les 4 tests §11.3 doivent couvrir en CI ; le prefill reel reste `#[ignore]`).
7. **Chevauchement `spot_check_rate`** : reconcilier en une source named-const (sinon violation regle named-constants S76).
8. **rand non-deterministe** : interdire toute source rand pour tirage/seed (a flaguer en review).
