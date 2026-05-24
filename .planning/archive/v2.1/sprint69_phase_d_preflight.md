# Sprint 69 Phase D — preflight G8 (agent deep)

Date : 2026-05-22 | HEAD : `9e8deb5` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- feedback_approach.md : "pick deepest, research before code" + "documenter
  AVANT de coder" — Phase D est 100% documentaire, conforme au principe.
  "G8 = mecanisme procedural" — ce preflight.
- vision_model.md : "NO funding / NO fondation / solo maintainer / pattern
  OpenBSD" — le pilote ferme est 2-3 personnes max, feedback structure
  texte (pas telemetrie automatisee, pas de plateforme beta enrollment).
  Conforme a D5 kickoff.
- feedback_context7_systematic.md : pas de lib/dep/API tierce touchee
  par Phase D (docs only). N/A.
- feedback_kudos_non_monetary.md : Phase D ne touche pas les kudos. N/A.
- fairness_vision.md : N/A (pas de formule kudos dans le test protocol).
- Tensions plan vs memory : aucune.

## Scans (all clean)

- S1a OSS prior art : 5 projets recherches (IPFS Kubo, Syncthing,
  Briar, F-Droid, Centercode/UAT guides), APPROACH-ALIGNED — clean
- S1b deps : 0 libs ajoutees/bumpees (phase docs-only) — clean
- S2 historiques : 6 commits main.rs + 14 commits docs/release + 3
  bodies lus in extenso (Phase B, kickoff, S60 wrap-up), 0 decision
  historique contredite — clean
- S3 threat model : FULL, 4 vecteurs analyses — clean (1 gap Low)
- S4 wire format : FULL / VERSION=1 toutes, Day 0 preserved — clean

---

## S1a — OSS prior art deep analysis

### Probleme fonctionnel

"How do mature P2P/OSS desktop projects document their release
acceptance test protocols for closed pilot testing with 2-3 testers?"

### Projets analyses en profondeur

#### [IPFS Kubo] — kubo (https://github.com/ipfs/kubo)

- Fichiers source lus : `docs/releases.md` (~200 LOC via WebFetch raw)
- Pattern architectural extrait : 5-stage release framework (Stage 0 =
  automated tests, Stage 1 = internal infra + IPFS apps rollout,
  Stage 2 = community beta non-prod, Stage 3 = community prod opt-in
  early testers via EARLY_TESTERS.md, Stage 4 = wide release).
  RC promotion cycle 6 semaines. Early testers self-volunteer via PR.
- Edge cases geres : rollback capability a chaque stage, monitoring
  entre stages, breadth of environment coverage, distinction non-prod
  vs prod testing.
- Patterns abandonnes : aucun visible dans docs actuels.
- Verdict : ALIGNED — notre Gate 1 est equivalent a Stage 2-3 (beta
  non-prod → community prod). Notre pilote 2-3 personnes est plus
  restreint (closed vs open beta), coherent avec modele solo maintainer.

#### [Syncthing] — syncthing (https://github.com/syncthing/syncthing)

- Fichiers source lus : `docs/releases.md` (documentation site)
- Pattern extrait : 2 release channels (stable / candidate). Candidate
  = community beta. Promotion apres periode de test sans regression.
  Pas de checklist publique formelle — la stabilite est validee par
  temps-en-candidate.
- Verdict : ALIGNED — approche plus informelle que la notre (pas de
  procedures pas-a-pas), mais meme philosophie : petit groupe teste
  avant release large.

#### [Briar] — briar (https://briarproject.org/)

- Fichiers source lus : News page, OTF security audit report summary
- Pattern extrait : Security audit independant (Radically Open Security
  via Open Tech Fund, 2024) avant beta publique. Desktop beta 0.6.0
  (2023). 6 issues trouvees, 4 resolues au retest fev-mars 2024.
  Penetration testing du client Android + desktop + review protocoles
  cryptographiques.
- Verdict : ALIGNED — Briar fait un audit de securite formel avant
  beta. Notre THREAT_MODEL.md + HARDENING_ROADMAP.md jouent ce role.
  Le pilote ferme est un complement (validation fonctionnelle, pas
  securite pure).

#### [F-Droid] — f-droid (https://f-droid.org)

- Fichiers source lus : "F-Droid in 2025" article, reproducible builds
  page
- Pattern extrait : Rebuilder pattern (tiers verifient ce que le builder
  produit). 21% des 4061 apps reproductibles en 2025. Badges visuels
  par app (check/cross). Pas de "beta test protocol" formalise pour
  testeurs humains — le processus est inherent au cycle de build
  reproductible.
- Verdict : N/A pour le test protocol specifiquement. Inspire notre FG8
  (Factory verifie ce que le daemon signe).

#### [Centercode/UAT best practices] — (https://www.centercode.com/guides)

- Fichiers source lus : "Ultimate Guide to Beta Testing", TestRail UAT
  guide, BrowserStack UAT checklist
- Pattern extrait : Beta testing framework standard : define objectives
  → recruit testers → create test plan (step-by-step test cases +
  expected results) → execute → collect feedback (defect template :
  titre, steps to reproduce, expected vs actual, screenshots,
  severity) → analyze → iterate. Exit criteria definis avant testing.
  Execution 3-12 semaines.
- Verdict : ALIGNED — notre Gate 1 test protocol suit ce pattern (9
  procedures = test plan, formulaire feedback = collect, verdict
  go/no-go = analyze).

### Tableau comparatif

| Aspect | Plan Phase D | IPFS Kubo | Syncthing | Briar | Centercode UAT |
|--------|-------------|-----------|-----------|-------|----------------|
| Format | .md 9 procedures pas-a-pas | RELEASE_ISSUE_TEMPLATE + EARLY_TESTERS | 2 channels sans checklist | Audit securite + release notes | Test plan + feedback form |
| Testeurs | 2-3 ferme (closed) | Open self-volunteer (9+ orgs) | Open via candidate channel | Security auditors + open beta | Recruited panel N config |
| Criteres | 9 binaires go/no-go | Par stage (implicit) | Time-in-candidate | 0 high-severity open | N test cases definis |
| Feedback | Table critere/resultat/notes/bloqueur | GitHub issues | GitHub issues | Audit report formal | Structured form |
| Infra | Aucune (envoi direct binaire) | RC downloads | Apt repo + GitHub releases | F-Droid + direct APK | Platform SaaS |

### Finding S1a

- Classification : APPROACH-ALIGNED
- Evidence : IPFS Kubo 5-stage (releases.md via WebFetch), Centercode
  UAT guide, Syncthing dual-channel, Briar security audit — tous
  valident l'approche "document statique + criteres binaires + petit
  groupe" pour un projet pre-launch solo maintainer.
- Impact sur le plan : aucun, EXECUTE plan-as-is.

---

## S1b — Deps/libs versions + CVE

Phase D ne touche aucune dep. Pas de lib ajoutee, pas de bump, pas de
spec. Le seul fichier code potentiellement touche (`main.rs`) est deja
complet — les subcommands `Sandbox` et `PreviewCheck` sont deja exposes
(Phase B, commit `aec036b`). `#[allow(dead_code)]` dans gates.rs = 0.

**Verification etat CLI** : `main.rs:80-91` expose deja les subcommands
Sandbox et PreviewCheck. `gates.rs` a 0 `#[allow(dead_code)]`. Phase D
n'a pas de travail CLI a faire.

Finding S1b : 0 delta — clean.

---

## S2 — Decision chain reconstruction

### Fichiers scannes

- `crates/sbfb-factory/src/main.rs` : 6 commits lus
  (`aec036b`, `c92e656`, `a201b3e`, `1d53f18`, `a4cc0ae`, `49d6bcd`)
  Bodies complets pour S69 : `aec036b`, `c92e656`, `faf4952`
- `docs/release/` : 14 commits lus (bodies complets pour les 3
  pertinents — `e14a131` S60 tag v1.0, `374bf59` S52 CI, `814e485`
  S27 Gate 3)
- `.planning/active/sprint69_kickoff.md` : D5 lu in extenso
- `.planning/archive/` : grep "pilot|Gate 1|test protocol" — 30+
  resultats scannes, sections pertinentes S17/S18 lues

### Decisions historiques trouvees

#### Decision 1 : CLI subcommands Sandbox + PreviewCheck absorbes Phase B

```
Decision : subcommands CLI Sandbox/PreviewCheck absorbes Phase B (prevu Phase D)
|- Sprint 69, sha `aec036b` : Phase B a absorbe les subcommands
|  Body extrait : "Bonus : subcommands CLI Sandbox et PreviewCheck
|  (prevu Phase D, absorbe ici pour resoudre #[allow(dead_code)]
|  proprement)."
|
`-> Status actuel : active (main.rs:80-91, gates.rs 0 dead_code)
    Impact phase : le plan Phase D §7.1 dit "si necessaire" pour
    les subcommands — ils sont deja faits. Phase D = docs only.
```

- Reverse-commit check : N/A (pas un rejet, une absorption anticipee)

#### Decision 2 : P3-I-2 dead_code CLOSED par Phase B

```
Decision : retrait #[allow(dead_code)] de gates.rs
|- Sprint 69, sha `aec036b` : P3-I-2 CLOSED
|  Body extrait : "P3-I-2 dead_code gates : CLOSED (retrait des 3
|  #[allow(dead_code)] via wiring pipeline + CLI subcommands)"
|
`-> Status actuel : CLOSED (grep -c = 0)
    Impact phase : aucun item carry restant pour Phase D.
```

#### Decision 3 : Gate 1 conditions — roadmap v4

```
Decision : 9 criteres Gate 1 go/no-go
|- Roadmap v4 (2026-05-19) : 9 criteres documentes §Gate 1
|  Criteres : installation, connexion P2P, deploy app, Babel Factory,
|  feed sync, restart, stabilite 24h, search, Proof Card
|
|- Sprint 69 kickoff (b930c34) D5 : "Gate 1 test protocol, 9
|  procedures pas-a-pas"
|
`-> Status actuel : active, non contredite
    Impact phase : Phase D documente ces 9 criteres. Alignement.
```

#### Decision 4 : pas de telemetrie / pas d'infra institutionnelle

```
Decision : pilote ferme sans telemetrie ni plateforme beta
|- memory vision_model.md : "Pattern OpenBSD solo maintainer"
|- kickoff S69 D5 rejected : "Telemetrie automatisee" + "Infrastructure
|  pilote (serveur distribution, update channel)"
|
`-> Status actuel : active (Day 0 transversale)
    Impact phase : aucun — Phase D livre un document statique.
```

### Memory constraints

- feedback_approach.md : "research/doc BEFORE code" — Phase D est
  documentation pure, conforme.
- vision_model.md : "NO telemetrie, NO infrastructure pilote" — le
  test protocol est un document texte distribue manuellement. Conforme.

---

## S3 — Threat model analysis

### Primitive analysee : Gate 1 test protocol (document de test)

Phase D ne cree pas de primitive code. Elle produit un document qui
reference des primitives existantes. L'analyse porte sur le risque que
le test protocol lui-meme introduise un vecteur.

### Assets en jeu

- A1 Binaires distribues aux testeurs (high) : integrite du binaire
  que les testeurs installent
- A2 Keypair Ed25519 testeur (high) : le protocole ne doit pas exposer
  les cles des testeurs ni demander de les partager
- A3 Feedback testeurs (medium) : PII potentiel dans les notes
- A4 Documentation test protocol (low) : document public sans secret

### Threat actors

- TA1 Attaquant supply chain / MITM : capacite interception distribution
  binaire, motivation compromission testeurs. Hors scope direct Phase D
  (distribution manuelle, pas automatisee).
- TA2 Social engineering : instructions ambigues pourraient amener un
  testeur a desactiver des protections.

### Attack vectors identifies

1. V1 Binary integrity (A1) : le test protocol pourrait omettre les
   instructions de verification du hash BLAKE3/SHA256 du binaire.
   Couverture existante : FG8 provenance Ed25519 post-publish couvre
   les apps, pas le binaire installeur lui-meme.
   Recommendation : le protocole DOIT inclure un placeholder pour
   le hash de l'installeur (rempli au moment de la distribution).

2. V2 Key exposure (A2) : le protocole pourrait demander au testeur de
   partager son daemon.key pour debugging.
   Mitigation : le protocole ne doit jamais mentionner daemon.key.
   node_id (public key hex) est public par nature et peut etre partage.

3. V3 Malicious peer connection (A1) : instructions de connexion a un
   noeud non-controle par FlowUP.
   Mitigation : le pilote ferme utilise exclusivement des noeuds
   controles par FlowUP. Le protocole specifie les noeuds bootstrap.

4. V4 Preview abuse DoS (A1) : le protocole encourage le test de
   preview avec des fichiers volumineux.
   Mitigation : MAX_PREVIEW_ENTRIES=10 + MAX_PREVIEW_BYTES=10MB
   (Phase A S69, `c92e656`). T-PREVIEW-EXHAUSTION THREAT_MODEL §13.

### Mitigations existantes

- T0 loopback auth (bearer + Host + Origin) couvre V3 partiellement
- T-PREVIEW-EXHAUSTION (§13) couvre V4
- T-PROOFCARD-FORMULA-GAME (§12) couvre les manipulations score
- FG8 provenance Ed25519 couvre V1 pour les apps (pas l'installeur)

### Gaps identifies

- GAP1 V1 (binaire installeur sans hash automatique) severity LOW :
  le test protocol DEVRAIT inclure un placeholder hash. Pas de
  regression sur T0-T5 existants. Carry post-v1.0 (installer signing).

### Regression check

- La phase ne modifie aucun code → aucune regression possible.
- Pas de nouveau vecteur : le document reference des primitives
  existantes, il n'en cree pas.

### Verdict S3 : clean (1 gap Low non-bloquant)

---

## S4 — Wire format deep audit

### canonical.rs lu integralement : oui (296 lignes)

Phase D ne touche aucune struct dans canonical.rs ni dans aucun autre
module. Les seuls fichiers modifies/crees sont :

- `docs/release/GATE1_TEST_PROTOCOL.md` (NEW — documentation pure)
- `crates/sbfb-factory/src/main.rs` — PAS MODIFIE (subcommands deja
  presents Phase B `aec036b`)

### Structs verifiees : N/A

Aucune struct touchee par Phase D. Verification exhaustive :

- FeedEntry : non touchee
- ProjectAnnouncement : non touchee
- CuratorList (version=1) : non touchee
- KeyRotationAnnouncement (version=1) : non touchee
- HashcashChallenge (version=1) : non touchee
- ProofCard : non touchee
- Provenance : non touchee

### Day 0 check

- D1 FG8 : non touche (Phase B done `aec036b`)
- D2 Babel template : non touche (Phase C done `faf4952`)
- D3 FG9 pipeline : non touche (Phase B done `aec036b`)
- D4 audit log + P2 : non touche (Phase A done `c92e656`)
- D5 Gate 1 test protocol : **CIBLE DE PHASE D** — le document
  GATE1_TEST_PROTOCOL.md implemente D5. Aucune contradiction.

### Decisions actees pivot.md : aucune contredite

Les 12 decisions actees + extensions S12-S14 sont toutes non impactees
par une phase documentaire.

### Pre-launch policy

- *_VERSION = 1 : OK (aucun bump, pas de code)
- Pas de tolerant decoder multi-version : OK (pas de code)
- Pas de tests "legacy decode" zombie : OK (pas de code)

---

## Observation supplementaire : scope Phase D reduit

Le plan §7.1 prevoyait 2 livrables :
1. `docs/release/GATE1_TEST_PROTOCOL.md` (documentation)
2. CLI subcommands Sandbox/PreviewCheck "si necessaire"

Le point 2 est deja fait (Phase B commit `aec036b`). Verification :
- `main.rs:80-91` : subcommands Sandbox + PreviewCheck presents
- `gates.rs` : 0 `#[allow(dead_code)]`

Phase D se reduit a un livrable unique : GATE1_TEST_PROTOCOL.md.
Ce n'est pas un scope cut — c'est un prerequis deja satisfait.
Le plan §7.2 est explicite : "Si necessaire" — ce n'est plus
necessaire.

---

## Findings

Aucun finding bloquant. 1 finding non-bloquant :

- S3 GAP1 : le test protocol GATE1_TEST_PROTOCOL.md devrait inclure
  un placeholder pour les hash des binaires distribues aux testeurs
  pour verification d'integrite. Non-bloquant (Low severity,
  documentation guidance). Le hash sera renseigne quand les binaires
  seront effectivement construits pour le pilote.

---

## Telemetrie preflight (agent deep)

- Duree totale : ~10m
- S1a : 5 projets OSS analyses (IPFS Kubo, Syncthing, Briar, F-Droid,
  Centercode) / 1 fichier source lu en detail (Kubo releases.md
  ~200 LOC via WebFetch) / 0 context7 queries (docs/UAT seulement) /
  5 WebSearch queries / finding : APPROACH-ALIGNED
- S1b : 0 libs scannees (phase docs-only, 0 dep ajoutee) / 0 CVE
  searches / finding : clean (N/A)
- S2 : 6 commits main.rs + 14 commits docs/release + 3 bodies lus
  in extenso (Phase B `aec036b`, kickoff `b930c34`, S60 `e14a131`) /
  30+ archive grep results / 5 memory files lus in extenso /
  finding : clean (subcommands absorbes Phase B confirme)
- S3 : FULL / 4 vectors analyses / 1 gap Low (hash placeholder)
- S4 : FULL / 0 structs verifiees (docs only) / canonical.rs lu
  integralement : oui (296 LOC) / 15 domain tags + 5 format versions
  checkes tous = 1

## Action

Proceder code phase D. Le plan Phase D §7.1 confirme que les subcommands
CLI sont deja faits (Phase B commit `aec036b`). Phase D est purement
documentaire : produire `docs/release/GATE1_TEST_PROTOCOL.md` avec les
9 procedures pas-a-pas correspondant aux 9 criteres Gate 1 de la
roadmap v4.

Notes pour l'agent principal :
1. Inclure dans le document un placeholder pour les hash des binaires
   (GAP1 S3 Low).
2. Les instructions installation doivent viser un utilisateur
   non-technique (memory feedback_v1_prod_ready.md).
3. Ne pas modifier `main.rs` — les subcommands sont deja presents.
