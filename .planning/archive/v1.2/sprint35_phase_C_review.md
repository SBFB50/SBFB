# Phase Review — Sprint 35 Phase C

## Verdict : PASS (2 P2 + 1 P3)

Rigor signal : 3 findings documentes (>=1 P2+ requis pour PASS rigoureux).

## Memory consultation
- feedback_approach.md : pick deepest — validator appelle verify_signature() natif, conforme
- Violations memory : 0

## Staging check (Step 1bis)
- Phase fichiers : validator.rs (NEW), lib.rs (+1 mod), preflight
- Planning/docs split : preflight + review seront chore(planning) separe
- Untracked accidentels : 0

## Suites
- En attente resultats background (5 jobs lances)
- A verifier avant commit

## Commit body validation
- Format titre : ✅
- Delta tests : +5 validator (accept valid, reject bad sig, reject unknown, reject completed, accept dispatched)
- Scope cuts honoured : ✅
- Co-Authored-By : ✅

## Modified-file branch coverage (Step 2bis, G9)
- `validator.rs` : `validate()` NEW — 5 tests couvrent accept, bad sig, unknown task, not pending, dispatched ✅
- `lib.rs` : +1 `pub mod validator;` — N/A

## Scope cuts verification
- "Migration complete coordinator" §7.1 : validator only ✅
- "Suppression coordinator Python" §7.2 : 0 suppression ✅
- validator_loop.rs §plan C.2 : scope-cut coherent (voir P2 ci-dessous)

## Findings

### P2-REVIEW-C-1 : validator_loop.rs et runtime.rs wire differes

Le plan §Phase C demandait `validator_loop.rs` (tokio subscription
loop sur iroh LiveEvents) et le wire dans `runtime.rs`. Ce scope
est differe car le validator a besoin du `Doc` handle iroh qui
vit dans DaemonHttpState — or le coordinator-rs n'est pas encore
integre dans le state du daemon (P2-REVIEW-B-1 carry).
La boucle de validation sera wired quand le dispatcher+validator
auront un lifecycle persistant dans le daemon (S36).
**Scope-cut coherent** : le validator core est livre et teste,
la subscription loop est du wiring infrastructure.

### P2-REVIEW-C-2 : validator n'appelle pas encore KudosLedger

Le validate() accepte le result et met a jour le status en
completed, mais ne credite pas les kudos. Le credit kudos est
prevu S36 quand le KudosLedger sera porte en Rust.
**Scope-cut coherent** : kickoff §7.3 differe explicitement
KudosLedger a S36.

### P3-REVIEW-C-1 : model_digest et logprobs_hash non verifies

Le validator verifie la signature Ed25519 et le status task,
mais ne verifie pas le model_digest (layer 2) ni le logprobs_hash
(layer 3). Ces verifications necessitent un registre de digests
connus (whitelist) qui vit dans le coordinator Python. Migration
S36+.

## Recommendation
- Ready to commit : **oui** (apres verification suites)
- Carry-overs S36 : P2-REVIEW-C-1 validator loop, P2-REVIEW-C-2 kudos credit
