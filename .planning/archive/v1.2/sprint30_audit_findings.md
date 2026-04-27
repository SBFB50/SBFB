# Sprint 30 — Audit findings (S31 Phase 0)

**Auditeur** : session fraiche S31 Phase 0
**Date** : 2026-04-26
**Tip audite** : `0afd21b` (chore(sprint30): Phase E)
**Audit plan** : `.planning/active/sprint31_audit_plan.md`

---

## Verdict global : PASS

| Severite | Count |
|----------|-------|
| P0       | 0     |
| P1       | 0     |
| P2       | 0 new (6 carry confirmations) |
| P3       | 1 new |

**G4 rigor signal** : 6 P2 carry confirmations + 1 P3 nouveau =
7 findings documentes. Signal >= 1 satisfait.

S31 Phase A peut demarrer sans fix prealable.

---

## Track A — P2 batch S29 audit (Phase A `a731811`)

### CONSENT-1 pure function — PASS

`_populate_threat_fields` renommee `_threat_fields_for_level`
(consent.py:126). Retourne un `dict` sans mutation. Les callers
(consent.py:193, consent.py:206) appellent
`cfg.model_copy(update=_threat_fields_for_level(cfg.level))` —
creation d'une copie Pydantic, l'input n'est jamais mute.

### CONSENT-2 test coverage — PASS

`test_consent_threat_fields_pure` (test_consent.py:162) valide
que la raw cfg ne contient pas `level_threat_note` et que le
retour enrichi du endpoint le contient. Regression tests
existants (test_consent.py:139, 154, 159) passent sans
modification.

### OTEL-1 doc fix — PASS

0 matches "0.27" dans HARDENING_ROADMAP.md. 0 matches "0.28"
dans les lib.rs des crates. Nettoyage effectue.

### EXECUTOR-1 trace path — PASS

Commentaire inline main.rs:34-36 : "Intentional relative path:
the executor runs as a child process of the broker and must NOT
share the daemon's trace directory. Process isolation requires
each binary to own its trace output." Rationale claire.

### TASK-RUNNER-1 defense-in-depth — PASS

Commentaire task_runner.rs:5-8 : "Stub: returns an empty result.
The real Ollama/llama.cpp dispatch is gated on the executor IPC
stabilisation (carry S31). This stub cannot execute arbitrary
code — it touches no prompt, no model, no subprocess, no
network." Contrat explicite.

### THREAT-MODEL-1 §9.5 gap note — PASS

THREAT_MODEL.md §9.5 contient : "output filter designed Sprint
23, wire end-to-end target Sprint 31. PII filter + watermark +
rate limit sont wired (S21-S24). Output filter reste design-only
(carry P2-REVIEW-B-2)." Gap documente.

**Track A findings** : 0.

---

## Track B — blob-serve COOP/COEP (Phase B `a63562e`)

### COOP-1 header present — PASS

Constante `BLOB_SERVE_COOP = "same-origin"` (blob_serve.rs:274).
Middleware `blob_serve_csp_middleware` (http.rs:263) applique le
header sur toutes les reponses blob-serve (http.rs:273). Test
assertion http.rs:2052-2058 verifie la valeur exacte.

### COEP-1 header present — PASS

Constante `BLOB_SERVE_COEP = "require-corp"` (blob_serve.rs:276).
Meme middleware (http.rs:277). Test http.rs:2060-2066 verifie.

### NOSNIFF-1 pre-existing — PASS

Test http.rs:2042-2048 verifie `X-Content-Type-Options: nosniff`
sur reponse 200. Test supplementaire http.rs:2220 verifie la
presence sur 404. Couverture sur les deux chemins.

### SANDBOX-1 iframe — PASS (production) + P3 (orphan)

`BrowsedProject.tsx:369` : `sandbox="allow-scripts"` sans
`allow-same-origin` — **correct**.

`WebAppFrame.tsx:29` : `sandbox="allow-scripts allow-same-origin"`
— **incorrect** mais composant Sprint 11 orphelin. Grep confirme
qu'il n'est importe nulle part dans le code de production (seul
import : `WebAppFrame.test.tsx:12`). Pas de risque securite
en l'etat. Cf. P3-AUDIT-1 ci-dessous.

### CSP-1 unchanged — PASS

CSP string blob_serve.rs:272 : `"default-src 'self'
'unsafe-inline' 'unsafe-eval' data: blob:; connect-src 'none';
frame-ancestors *"`. `connect-src 'none'` toujours present.
Test http.rs:2039 verifie.

### CI-1 rust-ci.yml coverage — PASS

rust-ci.yml matrice 3 OS : ubuntu-latest, windows-latest,
macos-14 (Apple Silicon). `nexus-events-core` est dans le
workspace (Cargo.toml:11) et inclus via `--workspace
--exclude nexus-core-py` (rust-ci.yml:162). `ci-cross-platform.yml`
n'existe pas (redondance evitee — conforme Phase B review P3).

### P2 carry confirmation

**P2-REVIEW-B-1-S30** (Playwright COEP iframe test, 1/3) :
confirme absent. Aucun test Playwright existant ne charge une
app iframe et verifie les COEP headers. Carry S31.

**Track B findings** : 1 P3 (P3-AUDIT-1).

---

## Track C — Warrant canary FROST DKG (Phase C `387b6b9`)

### DKG-1 generate_with_dealer — PASS

`frost.rs:163` : `frost::keys::generate_with_dealer(max_signers,
min_signers, IdentifierList::Default, rng)`. Validation params :
`dkg.rs:186-188` teste K=1, K=0, K>N (tous rejetes).
`frost.rs:403-412` teste specifiquement K=1 per RFC 9591 avec
assertion `FrostError::Keygen`.

### DKG-2 serialization — PASS

Test `dkg_generate_serialize_roundtrip` (dkg.rs:127) : genere
K=2/N=3, serialise share en JSON, deserialise, `load_share`
roundtrip, `load_pubkey` roundtrip, verifying_key hex match.

### CEREMONY-1 full roundtrip — PASS

Test `ceremony_full_roundtrip_3_participants` (ceremony.rs:215)
exerce round1 → build_signing_package → round2 → aggregate avec
K=2 signers sur N=3. Signature finale verifiee par
`nexus_core_rs::crypto::verify`.

### CEREMONY-2 insufficient signers — PASS

Test `ceremony_insufficient_signers_rejected` (ceremony.rs:237)
envoie 1 signer sur K=2 — erreur confirmee.

### CEREMONY-3 tamper detect — PASS

Test `frost_tampered_share_rejected` (frost.rs:440) : hybrid
shares (1 legit + 1 d'un autre keygen) → FROST aggregate ou
round2 detecte la tampering et rejette. Assertion couvre
`FrostError::Round2 | FrostError::Aggregate`.

### COMPAT-1 Niveau 0/1 — PASS

Test `frost_sig_verifiable_by_standard_ed25519_verifier`
(frost.rs:478) : signature FROST aggregee verifiee par
`nexus_core_rs::crypto::verify` (ed25519-dalek sous le capot).
Cross-check negatif : FROST sig ne passe pas avec pubkey
differente. Test supplementaire `dkg_roundtrip_produces_valid_signature`
(dkg.rs:151) exerce le chemin DKG → sign → Ed25519 verify.
Invariant wire-format confirme.

### CONFIG-1 canary.toml.sample — PASS

`configs/canary.toml.sample` : TOML valide, documente
`min_signers = 2`, `max_signers = 3`, `share`, `pubkey_package`,
`ceremony_dir`. Reference vers `WARRANT_CANARY_HARDENING.md §4`.

### RUNBOOK-1 ops — PASS

WARRANT_CANARY_HARDENING.md §4 contient des commandes CLI reelles
matchant les 5 sous-commandes wired Phase C :
- `nexus-shell-daemon canary frost trusted-dealer --k 2 --n 3`
- `nexus-shell-daemon canary frost round1 --share ... --commitment ... --nonces ...`
- `nexus-shell-daemon canary frost build-signing-package --commitments ... --headline ...`
- `nexus-shell-daemon canary frost round2 --share ... --nonces ... --signing-package ...`
- `nexus-shell-daemon canary frost aggregate` (section §4.3 step 7)

Workflow air-gapped complet avec distribution shares, destruction
nonces, BLAKE3 verification. Pas de pseudo-commandes.

### P2 carry confirmation

**P2-REVIEW-C-1-S30** (HTTP integration tests FROST, 1/3) :
confirme absent. Endpoints declares dans le routeur HTTP
(`POST /api/canary/frost/{trusted-dealer,round1,round2,aggregate}`)
mais non testes en integration HTTP. Tests unitaires couvrent la
logique core. Carry S31.

**Track C findings** : 0.

---

## Track D — G2 HARDENING + split inference (Phase D `9c8ffc9`)

### G2-1 last_validated — PASS

HARDENING_ROADMAP.md frontmatter : `last_validated: 2026-04-26`
avec mention S30 et 3 triggers scannes.

### G2-2 triggers documentes — PASS

`audited_findings` derniere entree (HARDENING_ROADMAP.md:26) :
3 triggers ACTIFS documentes avec dates et sources :
- iroh 0.98.0 publie 2026-04-17 (trigger 'iroh > 0.97' ACTIF)
- arti-client 2.0.0 stable 2026-02-07 (trigger 'arti > 1.x' ACTIF)
- openai-agents-python 0.14.6 publie 2026-04-25 (trigger '> 0.7.0' ACTIF)
Triggers INACTIFS egalement documentes.

### G2-3 S31 entry — PASS

HARDENING_ROADMAP.md:827 : "Sprint 31 — Tor transport phase 1 +
carries S30". Mentionne arti-client 2.0.0 LTS, feature principale
Tor transport, carries S30 resolus. S31 prescriptions coherentes.

### G2-4 S30 statut reel — PASS

HARDENING_ROADMAP.md:766-825 documente les 4 items prescrits :
- Nym : RE-DEFERRED S32+ (SDK paused crates.io)
- TEE H100 : SCOPE-CUT (pas de hardware partenaire)
- Split inference : LIVRE Phase D (SPLIT_INFERENCE_DESIGN.md)
- Warrant canary N1 : PARTIELLEMENT LIVRE Phase C (code wiring
  sans recrutement ni TEE)

### SPLIT-1 structure — PASS

5 sections : 1. Contexte SBFB, 2. Patterns existants,
3. Implications threat model, 4. Recommendations, 5. References.

### SPLIT-2 grounding — PASS

Patterns cites avec references verifiables :
- BOINC : Anderson 2019 paper + boinc.berkeley.edu
- Truebit : whitepaper + truebit.io
- Golem : golem.network
- Split learning : references academiques

Pas de hallucination detectee. Patterns reels et documentes.

### BLUEPRINT-1 coherence — PASS (no-op confirme)

Les 3 triggers (iroh, arti, openai-agents) sont des deps, pas
des menaces. VALIDATED_BLUEPRINT.md n'est pas en conflit avec
ces triggers. Coherence confirmee.

Note : VALIDATED_BLUEPRINT.md:275 "Kirchenbauer 2023" et
:278 "spaCy NER wasm" sont stale (cf. P2-REVIEW-D-1-S30
carry 1/3) mais ceci ne constitue pas un conflit avec les
triggers — c'est un probleme de fraicheur documentaire.

### P2 carry confirmation

**P2-REVIEW-D-1-S30** (VALIDATED_BLUEPRINT Couche 6 stale, 1/3) :
confirme. Kirchenbauer → SynthID (S27 Phase D), spaCy → GLiNER
(S21 Phase B). Carry S31.

**Track D findings** : 0.

---

## Track E — Process compliance

### G8-1 preflights — PASS

4 fichiers preflight existent dans archive/v1.2/ :
`sprint30_phase_{A,B,C,D}_preflight.md`. Verdicts EXECUTE
confirmes par commits `chore(planning)` correspondants.

### G8-2 reviews — PASS

4 fichiers review existent avec verdicts PASS :
- Phase A : PASS (1 P2 + 1 P3) — rigor signal ✓
- Phase B : PASS (1 P2 + 1 P3) — rigor signal ✓
- Phase C : PASS (1 P2 + 1 P3) — rigor signal ✓
- Phase D : PASS (1 P2 + 1 P3) — rigor signal ✓

Chaque review a >= 1 P2 (G4 satisfait).

### COMMIT-1 format — PASS

4 commits suivent le pattern `feat|docs(sprint30): Sprint 30
Phase X — titre` :
- `a731811 feat(sprint30): Sprint 30 Phase A — P2 batch S29 audit (7 items)` ✓
- `a63562e feat(sprint30): Sprint 30 Phase B — dette pair blob-serve COOP/COEP isolation` ✓
- `387b6b9 feat(sprint30): Sprint 30 Phase C — warrant canary Niveau 1 FROST DKG code wiring` ✓
- `9c8ffc9 docs(sprint30): Sprint 30 Phase D — G2 HARDENING refresh + split inference research` ✓

Phase C body verifie : sections Scope, Nouveaux modules, CLI,
HTTP endpoints, tests, fichiers touches. Body riche conforme §4.

### G1-1 design review — PASS

`sprint30_design_review.md` existe dans archive/v1.2/ avec
scoring D1..D5 : 3 ✅ (D1 D2 D3) + 2 ⚠️ (D4 D5). Kickoff §4
acknowledge les 2 ⚠️ avec ajustements.

### SCOPE-1 cuts respected — PASS

13 scope cuts du kickoff §7 verifies contre le diff S30 :
1. Tor transport → aucun fichier arti touche ✓
2. Nym mixnet → aucun nym-sdk ajoute ✓
3. TEE H100 → aucun module TEE ✓
4. DKG distribue → trusted dealer uniquement (DKG file-based, pas inter-process) ✓
5. Recrutement mainteneurs → 0 ops infra ✓
6. iroh 0.98 → Cargo.lock iroh 0.97 inchange ✓
7. openai-agents → aucune dep ajoutee ✓
8. task_runner impl → stub conserve (task_runner.rs:9 retourne resultat vide) ✓
9. §9.5 output filter wire → doc seulement (THREAT_MODEL §9.5 note) ✓
10. Full process isolation blob-serve → aucun rewrite architectural ✓
11. Tor PoW → trigger inactif, rien touche ✓
12. MCP spec → trigger inactif, rien touche ✓
13. CI full workspace → scope nexus-events-core uniquement (rust-ci.yml:162) ✓

### PRE-LAUNCH-1 — PASS

Grep `_VERSION` dans `crates/nexus-core-rs/src/` :
- `CURATOR_LIST_FORMAT_VERSION: u16 = 1`
- `KEY_ROTATION_FORMAT_VERSION: u16 = 1`
- `BLOB_VERSION: u8 = 0x01`
- `POW_FORMAT_VERSION: u16 = 1`
- `TASK_FORMAT_VERSION: u16 = 1`
- `PIN_FILE_FORMAT_VERSION: u8 = 1`
- `TASK_RESPONSE_VERSION: u8 = 1`

Tous = 1. Aucun bump. Aucun tolerant decoder. Aucun test
"legacy decode" introduit. Pre-launch protocol respecte.

### ROADMAP-1 coherence — PASS

Commit `c50976a` cree `roadmap_v1.0_alexandria.md` (252 LOC).
Mapping S31-S35 : S31 task_runner + carries + Tor, S32-S33
Alexandria, S34 polish, S35 tag v1.0. Pas de promesse pour
des livrables scope-cut (Nym, TEE, DKG distribue absents de
S31-S35 direct scope). Coherent avec kickoff §7.

**Track E findings** : 0.

---

## Track F — Meta-track carries

### CARRY-1 compteurs — PASS

`sprint30_carry_summary.md` documente :
- P2-REVIEW-B-2 (§9.5 output filter) : **2/3** reports ✓
- P2-REVIEW-C-1 (task_runner stub) : **2/3** reports ✓

Si non resolus S31, ils passent 3/3 = MANDATORY S32
(§6.2.1 Regle 2).

### CARRY-2 LT-6 — PASS

LT-6 documente carry_summary.md:49 : "Trigger met (iroh
0.98.0 2026-04-17) — bloque par Day 0 #3 pin, reste latent."
Day 0 #3 (iroh 0.97 pinne) empeche l'upgrade. Awareness only.

### CARRY-3 new carries — PASS

4 nouveaux carries documentes avec source commit :
- P2-REVIEW-B-1-S30 : Playwright COEP iframe (1/3, Phase B review)
- P2-REVIEW-D-1-S30 : VALIDATED_BLUEPRINT stale (1/3, Phase D review)
- P3-REVIEW-D-1-S30 : confidence_score field (1/3, Phase D review)
- P2-REVIEW-C-1-S30 : HTTP FROST tests (1/3, Phase C review)

**Track F findings** : 0.

---

## Track G1 presence — PASS

`sprint30_design_review.md` present dans archive/v1.2/ avec
scoring 5/5 decisions (3 ✅ + 2 ⚠️). G1 Design Review Board
execute.

---

## Track HARDENING drift — PASS

| Item prescrit | Livre ? | Justification |
|---|---|---|
| Nym mixnet phase 1 | Non | Scope-cut kickoff §7.2 (SDK paused crates.io) |
| TEE H100 attestation | Non | Scope-cut kickoff §7.3 (pas hardware partenaire) |
| Split inference research | Oui | Phase D `9c8ffc9` SPLIT_INFERENCE_DESIGN.md |
| Warrant canary Niveau 1 | Oui | Phase C `387b6b9` DKG + ceremony code wiring |

2 items non-livres avec scope-cut justifie dans le kickoff.
0 drift non-justifie.

---

## Findings consolides

### P3

**P3-AUDIT-1** : `web/src/components/app/WebAppFrame.tsx:29` a
`sandbox="allow-scripts allow-same-origin"` — inconsistant avec
la politique securite SBFB (`allow-scripts` SANS `allow-same-origin`).
Composant Sprint 11 Phase C orphelin : non importe dans aucun
code de production (seul import : test unitaire
`WebAppFrame.test.tsx:12`). Le composant de production
`BrowsedProject.tsx:369` est correct. Pas de risque securite
en l'etat. Nettoyage recommande (suppression ou alignement
sandbox attribute).

### Carry confirmations (P2, non-nouveaux)

| ID | Description | Reports | Source originale |
|---|---|---|---|
| P2-REVIEW-B-2 | §9.5 output filter not wired end-to-end | 2/3 | S29 review → S30 doc |
| P2-REVIEW-C-1 | task_runner stub (impl reelle) | 2/3 | S29 review → S30 doc |
| P2-REVIEW-B-1-S30 | Playwright COEP iframe test | 1/3 | S30 Phase B review |
| P2-REVIEW-D-1-S30 | VALIDATED_BLUEPRINT Couche 6 stale | 1/3 | S30 Phase D review |
| P2-REVIEW-C-1-S30 | HTTP integration tests FROST endpoints | 1/3 | S30 Phase C review |
| P3-REVIEW-D-1-S30 | confidence_score field | 1/3 | S30 Phase D review |

---

## Carry-overs pour S31

### MANDATORY si non resolus S31 (§6.2.1 Regle 2, 3/3 S32)

| ID | Reports apres S31 si non resolu |
|---|---|
| P2-REVIEW-B-2 | 3/3 → MANDATORY S32 |
| P2-REVIEW-C-1 | 3/3 → MANDATORY S32 |

### P2 carries actifs

| ID | Reports |
|---|---|
| P2-REVIEW-B-1-S30 | 2/3 apres S31 si non resolu |
| P2-REVIEW-D-1-S30 | 2/3 apres S31 si non resolu |
| P2-REVIEW-C-1-S30 | 2/3 apres S31 si non resolu |

### P3

| ID | Reports |
|---|---|
| P3-AUDIT-1 | 1/3 (nouveau) |
| P3-REVIEW-D-1-S30 | 2/3 apres S31 si non resolu |
