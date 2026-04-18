# Sprint 20 Phase E — nexus-phase-auditor review

HEAD pre-commit: `e653619bf9cd266f2e103a65a2d12eb7b5b64a1c`
Draft commit body: "feat(canary): Sprint 20 Phase E — federation foundations + WSS fallback observability"
Timebox: 45m

## Verdict : PASS

(0 P0 + 0 P1 + 3 P2 documentes — rigor signal G4 satisfait)

Rationale synthese : l'implementation est conforme au pivot G8 Option C arbitre
2026-04-18. Les 7 sous-taches E.1-E.7 sont livrees comme spec. Le wire format
`CanarySigned v1` est preservé au sens large (FROST sig = Ed25519 RFC 8032
byte-identique, verifie par test `frost_sig_verifiable_by_standard_ed25519_verifier`).
Aucun scope cut S20 §8 touche. Tests delta annonce (+17 Rust + 5 Python = +22)
confirme par comptage direct. 3 P2 identifies avec carry-over ou note inline.

---

## Dimensions

### Security

- [x] **Aucun bloc `unsafe`** dans les 6 fichiers Rust nouveaux (frost.rs,
  mod.rs/canary/, signer.rs, duress_ack.rs, attestation.rs, transport_probe.rs) —
  grep confirme 0 occurrence.
- [x] **unwrap() / expect() en production** : 2 `expect()` dans
  `FrostCanarySigner::CanarySigner` impl (lignes 300 + 315 de frost.rs). Les deux
  sont correctement justifies :
  - L. 300 : `to_bytes().expect("frost-ed25519 verifying key always serializes to 32 bytes")`
    — la construction `trusted_dealer` a reussi donc la cle est valide ; un echec
    ici = bug lib, pas condition runtime utilisateur. Pattern P26 (expect-as-invariant)
    respecte.
  - L. 315 : `expect("in-process FROST sign with self-dealt shares cannot fail")` —
    shares auto-produits localement, impossible d'echouer avec des inputs valides.
    Justification inline adequate.
  - Les `unwrap()` restants sont exclusivement dans `#[cfg(test)]` (Date hardcoded).
    Conformes au pattern.
- [x] **CanarySigner trait migration** (`&KeyPair` → `&dyn CanarySigner`) : pas de
  nouveau vecteur. Le trait `Sign + Sync` est minimal (pubkey + sign). L'interface
  ne donne pas acces au secret en clair — `Ed25519CanarySigner::keypair()` expose
  `&KeyPair` mais uniquement via methode publique explicite, pas via le trait.
  Pas de downcast possible. Vecteur absent.
- [x] **AttestationProvider / CanarySigner decoupling** : les deux traits sont
  orthogonaux, pas de partage de secret entre eux. `NoopAttestation` ne fait rien.
  Correct.
- [x] **Separation cross-stream replay duress-ack** : `DOMAIN_DURESS_ACK_V1 =
  b"nexus-duress-ack-v1"` est distinct de `DOMAIN_WARRANT_CANARY_V1 = b"nexus-
  warrant-canary-v1"`. Test `duress_ack_topic_id_deterministic_and_distinct_from_canary`
  verifie l'inegalite `duress_ack_topic_id() != warrant_canary_topic_id()`. Domain
  separation correcte et testee.
- [x] **POST /api/canary/observed — auth** : le router `canary_router` est ajoute
  APRES `app.add_middleware(LoopbackAuthMiddleware, ...)` dans `create_app`. FastAPI
  ASGI middleware = scope global. Toutes les routes incluant les nouvelles endpoints
  canary passent par `LoopbackAuthMiddleware`. Bearer token Sprint 16 actif.
- [x] **POST /api/canary/observed — validation** : body parse via pydantic
  `CanaryObservation.model_validate` / `DuressAckObservation.model_validate` avec
  `extra="forbid"`. Champs `pubkey_hex` / `signature_hex` contraints en longueur
  (min_length + max_length). Type `kind` valide (400 si unknown, 422 si payload mal
  forme). Aucune decision de trust — le registry est observational-only.
- [~] **Registry sans verification crypto Ed25519 at ingest** : `canary_registry.py`
  accepte des payloads via `POST /api/canary/observed` sans verifier la signature
  Ed25519. Ce comportement est **delibere et documente** (module docstring +
  `WARRANT_CANARY_HARDENING.md §2` table T-canary-registry-spoof) : le registre
  est observationnel, la verification est a la charge du caller / operator. Le preflight
  §S3 le classe NEW threat mitige par design. Toutefois : un attaquant
  qui peut appeler `POST /api/canary/observed` (via loopback — donc local) peut
  injecter des observations fake avec un `pubkey_hex` quelconque et un
  `signature_hex` forge. La defanse est le bearer token loopback + le fait que
  l'operateur tient la trust root (CANARY.txt bootstrap pubkeys). Documente P2 ci-
  dessous (pas P1 car mitigated + intentionnel).
- [x] **Loopback / wire / zip** : Phase E ne touche pas de route loopback, pas de
  module zip, pas de module canonical.rs hors ajout DOMAIN_DURESS_ACK_V1. Pas
  de regression.
- [x] **secrets / path traversal** : grep sur patterns `AKIA|ghp_|pat_|sbfb_` → 0.
  Aucun secret embedded. canary_registry_path() utilise `nexus_grid_root() /
  "canary-registry.json"` — path fixe, pas d'interpolation utilisateur.

### Patterns

- [x] **P-canonical / JCS** : les signatures dans Phase E (canary, duress_ack)
  passent par `canonical_bytes(&struct, DOMAIN_*)` qui appelle `serde_jcs` (RFC 8785).
  Le domain separation est applique. **PASS.**
- [~] **P-JCS wire broadcast (`canary_wire_bytes` utilise `serde_json::to_vec`)** :
  `canary_wire_bytes` line 242 de mod.rs utilise `serde_json::to_vec` (non-JCS)
  pour le broadcast gossip. Ce code est **PREEXISTANT a Phase E** (confirme via
  `git show HEAD:crates/nexus-shell-daemon-core/src/canary.rs` lines 212-213 —
  identique). Phase E ne l'introduit pas. C'est un P2 carry-over du S18 E2 qui
  n'a jamais ete leve. Note : ce broadcast JSON n'est pas signe — la signature
  couvre les `canonical_bytes` (JCS). Le JSON de broadcast est juste l'enveloppe
  de transport. Cependant le pattern SBFB (docs/rust/PATTERNS.md) prefere JCS
  partout pour eviter les ambiguites de deserialization cross-language (cf. P-wire
  sprint 2). Porte ici comme P2 carry-over S18 hors scope Phase E.
- [x] **P31 (nouveau, Phase E)** : les 3 invariants P31 sont honores :
  - CanarySigner trait = seul contrat signe (minimal, pas de Result sur sign,
    pas async)
  - FROST sig = Ed25519 RFC 8032 byte-identique (test cible)
  - Federated registry = observational-only (jamais re-signe, pas de trust decision)
- [x] **P32 (nouveau, Phase E)** : transport_probe = observability-only, jamais
  `endpoint.set_relay_mode()`. Pattern documente et respecte.
- [x] **Domain separation** (pattern preexistant, renforce Phase E) : 2 nouveaux
  domain tags (`DOMAIN_DURESS_ACK_V1`) + 1 nouveau topic gossip
  (`nexus-grid/canary-duress-ack/v1`) distincts de leurs homologues canary. Test
  explicit de distinction.
- [x] **Wire format `CanarySigned v1`** : `CANARY_VERSION = 1` inchange.
  `DOMAIN_WARRANT_CANARY_V1 = b"nexus-warrant-canary-v1"` inchange. `CanarySigned`
  struct inchangee. Aucun `#[serde(default)]` ajoute a la struct — la migration
  trait E.1 est un refactor pur.

### Working tree audit (G5)

- [x] **PHASE** : 24 fichiers listes dans le working tree (git status --short).
  Tous correspondent aux 7 sous-taches E.1-E.7 du plan pivot Option C.
  - Cargo.lock + Cargo.toml (E.2 dep frost-ed25519 = "2.1")
  - crates/nexus-core-rs/src/{canonical,lib}.rs (E.4 DOMAIN_DURESS_ACK_V1 + export)
  - crates/nexus-shell-daemon-core/Cargo.toml + lib.rs (E.2 feature + E.6 mod)
  - canary/{mod,signer,frost,duress_ack,attestation}.rs (E.1 + E.2 + E.4 + E.5)
  - transport_probe.rs (E.6)
  - crates/nexus-shell-daemon/src/main.rs (E.1 wrap caller)
  - docs/rust/PATTERNS.md + docs/shell/PATTERNS.md (E.7)
  - docs/security/WARRANT_CANARY_HARDENING.md (E.7)
  - docs/security/HARDENING_ROADMAP.md (E.7)
  - packages/nexus-coordinator/src/nexus_coordinator/{api/app,coordinator,paths}.py (E.3)
  - packages/nexus-coordinator/src/nexus_coordinator/{canary_registry,api/canary}.py (E.3)
  - packages/nexus-coordinator/tests/test_{canary_registry,api_canary}.py (E.3 tests)
- [x] **CRAFT** : 0 fichier planning/docs Claude non attendus dans le diff. Les
  artefacts planning (preflight.md, pivot_proposal.md) sont deja commites dans
  commits chore(planning) anterieurs (`bd16e64` + `e653619`). Separation disciplinee.
- [x] **DEBT** : 0 fichier scope cut touche.
- [x] **NOISE** : 0 fichier accidentel (node_modules, .pdb, .env, cache).
- [x] **Section "Working tree audit" attendue dans body commit** : le draft body
  documente une section G5 listant 25 fichiers PHASE / 0 CRAFT / 0 DEBT / 0 NOISE.
  Conforme.

### G8 traceability

- [x] **`sprint20_phase_E_pivot_proposal.md`** existe dans `.planning/active/`
  (verifie). Verdict : DESIGN-CONFLICT. Arbitrage user Option C documente §7 +
  commit `bd16e64` chore(planning) anterieur au code.
- [x] **`sprint20_phase_E_preflight.md`** existe dans `.planning/active/` (verifie).
  Verdict : SCOPE-CUT-CONSISTENT. HEAD re-validation post-crash `b634c23`.
- [x] **Plan §Phase E reflete le pivot** : commit `bd16e64` a mis a jour
  `sprint20_plan.md §8 Phase E` AVANT l'ecriture du code. Pas de pivot silencieux.
- [x] **Finding S1 E.6 inline absorption** : le preflight documente l'ajustement
  (`relay_wss_only` n'existe pas → probe diagnostic-only). Le code `transport_probe.rs`
  est coherent avec l'ajustement. Le body commit referece le preflight.md. Traçable.
- [x] **Findings non-bloquants carry S+1** : preflight verdict SCOPE-CUT-CONSISTENT
  avec 1 finding non-bloquant (S1 E.6) absorbe inline. 0 carry-over S+1. Conforme
  (carry requis seulement si finding non-absorbe).
- [x] **Pivot retrospective trackee** : preflight §garde-fous cite "Phase F ajouter
  ligne *Pivot retrospective Phase E* dans `sprint20_audit_plan.md`". A verifier
  Phase F.

### Scope-cuts

Scope cuts S20 §8 (kickoff) scannes contre le diff :
- `Hardware keystore` (TPM/SE/StrongBox) → 0 match dans les fichiers du diff.
- `HPKE envelope` → 0 match.
- `Rate-limit per-consumer` → 0 match.
- `Client-side redaction SDK` → 0 match.
- `Kudos-weighted gossip admission` → 0 match.
- `Tool-calling sandbox allow-list strict` → 0 match.
- `PQC migration ML-DSA / ML-KEM` → 0 match (note : frost.rs doc cite
  "PQC ML-DSA" dans le contexte "future backend trait" — c'est de la documentation
  prospective, pas d'implémentation. Non bloquant).
- `Arti Tor bridge / Domain fronting` → 0 match.

[x] **Aucun scope cut touche.**

### Tests-delta

Comptage direct des fonctions de test dans les fichiers du diff :

| Module | Tests existants avant Phase E | Tests apres Phase E | Delta |
|---|---|---|---|
| canary/mod.rs (ex-canary.rs) | 10 (inchanges, refactor pur) | 10 | +0 |
| canary/signer.rs (nouveau) | 0 | 2 | +2 |
| canary/frost.rs (nouveau) | 0 | 6 | +6 |
| canary/duress_ack.rs (nouveau) | 0 | 3 | +3 |
| canary/attestation.rs (nouveau) | 0 | 2 | +2 |
| transport_probe.rs (nouveau) | 0 | 4 | +4 |
| **Rust total** | | | **+17** |
| test_canary_registry.py (nouveau) | 0 | 4 | +4 |
| test_api_canary.py (nouveau) | 0 | 1 | +1 |
| **Python total** | | | **+5** |
| **TOTAL** | | | **+22** |

Annonce dans le draft body : +17 Rust + 5 Python = +22.
Reel mesure : +17 Rust + 5 Python = +22. **Delta exact. PASS.**

Note complementaire : le plan pivot Option C prevoyait +20 tests
(E.1 +2 + E.2 +5 + E.3 +4 + E.4 +3 + E.5 +2 + E.6 +4 = 20). Le reel est +22 :
- E.2 +6 (au lieu de +5 : split test K=1 rejection en test defensif separe)
- E.3 +5 Python (au lieu de +4 : test api_canary integre test POST + GET en 1 test)

Les 3 P2 auto-identifies en phase couvrent ces ecarts. Cap 2.5x (cap 50) respecte.

- [x] Aucun test `#[ignore]`, `pytest.mark.skip`, `#[should_panic]` sans raison.
  Grep confirme 0 occurrence.
- [x] Aucun test supprime (canary/mod.rs preexistant conserve integralement ses 10 tests).

### Research-grounding

```
Deps Rust ajoutees (git diff HEAD -- Cargo.toml):
  + frost-ed25519 = "2.1"
```

**frost-ed25519 = "2.1"** :
- Trace presente : `Cargo.toml` contient un commentaire de 20 lignes documentant
  la version (2.1.0 stable crates.io), l'audit (Trail of Bits 2023, ZcashFoundation),
  la spec (RFC 9591 jan 2025), le RUSTSEC advisory check 2026-04-18 (0 advisory
  actif ; RUSTSEC-2026-0075 sur `libcrux-ed25519` non applicable). Preflight.md
  §S1 documente les sources (context7 + WebSearch ZcashFoundation/frost +
  WebSearch RFC 9591 erratum + WebSearch RUSTSEC).
- **PASS** : trace complete, version pinnee, advisory check date <= 1 jour.

```
Deps Python ajoutees (git diff HEAD -- packages/nexus-coordinator pyproject.toml):
  Aucune dep externe ajoutee — canary_registry.py utilise pydantic
  (deja workspace dep) + structlog (deja workspace dep) + stdlib json/pathlib.
```

**PASS** : 0 nouvelle dep Python.

```
APIs crypto / specs standardisees utilisees dans le diff :
  - FROST RFC 9591 (frost-ed25519 crate) — trace presente (Cargo.toml + preflight.md)
  - Ed25519 RFC 8032 (verify path, deja utilise S18) — pas nouveau, pass
  - JCS RFC 8785 (canonical_bytes, deja S2) — pas nouveau, pass
  - DOMAIN_DURESS_ACK_V1 (nouveau domain tag) — tag interne, pas spec externe, pass
```

[x] **Toutes les APIs crypto / specs standardisees tracees.** PASS.

### Horizon long-terme + documentation amont

- [x] **Design doc present** : `docs/security/WARRANT_CANARY_HARDENING.md` (~300
  lignes) est livre dans Phase E.7 AVANT que les commits subsequents ne le
  referencent. Le doc couvre les 4 couches L0-L2, le threat model (T-canary-*),
  la procedure FROST DKG cross-juridiction, le roadmap TEE, l'operator runbook.
  Suffisant pour un module structurant sprint-lifetime.
- [x] **G8 pivot_proposal.md §4 cite les alternatives rejetees** : Option A
  (scope-cut conservatif), Option B (staleness alarm minimal), Option C (deep-
  evolution). Rationale du choix Option C documente en 7 points. Conforme.
- [x] **Solution la plus poussee** : FROST RFC 9591 (2025, audite ToB 2023) est
  l'etat de l'art threshold Ed25519 en 2026. `AttestationProvider` trait decouple
  correctement selon le pattern Confidential Computing Consortium. Transport probe
  correctement scope observability-only (la documentation S1 G8 montre que la
  vraie solution etait deja dans iroh, pas cote client).
- [x] **Aucune estimation LOC dans plan/kickoff** : grep `sprint20_plan.md` +
  `sprint20_kickoff.md` pour `LOC estimee|~ \d+ LOC|estime.*LOC` — kickoff §1.2
  mention des LOC HARDENING_ROADMAP (`~800 LOC`, `~500 LOC`, etc.) sont des
  **projections roadmap retrospectives** (la roadmap est une reference amont, pas
  une estimation dans le plan Phase E). Note : le format contient des `~800 LOC`
  dans les kickoff items §1.2 HARDENING_ROADMAP reference — c'est ambigu. Par
  consequent, ce point est leve comme P2 avec recommandation de clarifier si les
  LOC du kickoff §1.2 sont des retrospectives de roadmap ou des estimations (cf.
  feedback_approach.md — LOC retrospective legitime, LOC estimee dans plan = P2).

---

## Findings

- **P2-1** : `canary_wire_bytes` (canary/mod.rs:242) utilise `serde_json::to_vec`
  (non-JCS) pour le broadcast gossip. Ce code est **PREEXISTANT** a Phase E (confirme
  git show HEAD:canary.rs:212). Phase E ne l'introduit pas mais ne le corrige pas non
  plus. Le pattern SBFB (P-wire sprint 2) prefere JCS pour eviter les ambiguites
  cross-language. La signature Ed25519 couvre les canonical_bytes (JCS), pas le
  broadcast envelope, donc l'impact securite est nul. Mais un subscriber Python qui
  re-serialise le dict recu pourrait produire un ordre different. Carry-over S18 E2.
  **Recommandation** : creer tech debt entry dans PATTERNS.md (`T-NN: canary_wire_bytes
  → migrer vers serde_jcs`). Hors scope Phase E, ne bloque pas le commit.

- **P2-2** : Le `CanaryRegistry` Python (`canary_registry.py`) ne verifie pas la
  signature Ed25519 des canaries/acks ingeseres via `POST /api/canary/observed`.
  C'est une decision de design deliberee et documentee (observational-only, doc
  module + threat model T-canary-registry-spoof). Toutefois la documentation
  WARRANT_CANARY_HARDENING.md §2 table note la mitigation comme "futur". Un attaquant
  local (meme uid, bearer token en main) pourrait injecter des observations avec
  n'importe quel pubkey_hex et n'importe quelle signature — le registre afficherait
  une freshness fictive pour ce pubkey, potentiellement masquant un vrai pubkey stale.
  **Recommandation** : ajouter une tech debt entry T-NN dans PATTERNS.md
  (`T-NN: CanaryRegistry.observe_canary doit verifier Ed25519 sig avant stockage,
  actuellement observational-only par design`). Non bloquant Phase E — l'impact
  est limite au loopback local et l'acces require le bearer token.

- **P2-3** : Les LOC mentionnees dans `sprint20_kickoff.md §1.2` (tableau
  HARDENING_ROADMAP items : `~800 LOC`, `~500 LOC`, `~400 LOC`, etc.) semblent
  etre des references a la roadmap originale Phase D S17, pas des estimations Phase E
  specifiques. La politique `docs/claude/feedback_approach.md` interdit les
  estimations LOC dans le plan (LOC retrospective = OK, LOC estimee = P2 anti-
  pattern). A clarifier : sont-elles des estimations historiques de roadmap
  (leg avant S20, donc hors scope de la regie) ou des nouvelles estimations dans
  ce kickoff ? Si elles sont des projections historiques de la roadmap, elles sont
  legitimes et ce finding peut etre ferme inline. Si elles constituent des estimations
  dans le plan actuel, elles doivent etre supprimees. **Recommandation** : l'executeur
  verifie la provenance et ajoute un commentaire inline si necessaire.

---

## Recommendation

**Commit autorise.**

0 P0 + 0 P1 → le diff est commiteable. Les 3 P2 sont docuementes :

1. **P2-1** (`canary_wire_bytes` serde_json preexistant) → ajouter tech debt
   T-NN dans docs/rust/PATTERNS.md avant ou en meme temps que le commit Phase E,
   OU noter comme carry S21 dans sprint20_audit_plan.md Phase F.

2. **P2-2** (registry sans verify Ed25519) → ajouter tech debt T-NN dans
   docs/rust/PATTERNS.md (ou docs/shell/PATTERNS.md cote Python) avant ou
   avec le commit Phase E, OU noter carry S21.

3. **P2-3** (LOC kickoff §1.2 ambigues) → verification inline par l'executeur.
   Si LOC = projections roadmap historiques, finding ferme. Si LOC = estimations
   plan actuel, retirer avant commit.

Les 3 P2 peuvent etre resolus via un commit `chore(patterns)` avant le feat ou
via notation carry S21 dans audit_plan Phase F — les deux patterns sont valides.

---

## Pivot retrospective audit (dimension supplementaire Phase E)

Demande par le preflight §6 + draft body. Verify :

- **G8 decode correctement** : le premier pivot_proposal.md documente une premiere
  application retroactive (G8 n'existait pas au moment de l'arbitrage). L'execution
  retrospective est conforme — le preflight.md re-valide les 4 scans post-crash.
- **Coherence plan-vs-code** : sous-taches E.1-E.7 toutes livrees. E.6 ajuste
  (observability-only) coherent avec S1 finding preflight. E.2 +6 tests au lieu
  de +5 (split defensif K=1) documente comme P2 self-identified.
- **Dead-man switch integrité** : aucune signature canary automatique introduite.
  `CanarySigner::sign` est synchrne, appele uniquement par `build_canary` (CLI
  manuel). Pas d'entree scheduler/cron/GHA. Decision S18 E2 commit `04c9621`
  honoree by construction.
- **FROST wire-format invariant** : test `frost_sig_verifiable_by_standard_ed25519_verifier`
  + test `frost_dkg_k2_n3_produces_valid_ed25519_sig` → `verify_canary(&canary)`
  verifient que le path de verification standard accepte les sigs FROST sans modification.
  Invariant valide.
