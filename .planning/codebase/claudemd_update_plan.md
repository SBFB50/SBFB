# Plan de mise a jour CLAUDE.md + .planning/ pour roadmap S65-S75

**Date :** 2026-05-18
**Contexte :** S64 CLOSED, roadmap S65-S75 (3 arcs, 11 sprints) recherchee et validee.
**Sources :** `s65_s75_cross_cutting_research.md`, `s65_contrat_public_research.md`, CLAUDE.md actuel, SPRINT_LOG.md, .planning/ layout.

---

## 1. CLAUDE.md — analyse section par section

### 1.1 `## Projet` — MISE A JOUR

**Probleme :** Le paragraphe "App store open source par construction" sur-promet
(cf. S65 contrat public research §3.1 gaps G2/G3). Le vocabulaire "open source"
doit devenir "source verifiable" conformement a la decision roadmap S65-S75.

**Ancien texte (lignes 11-16) :**
```
**App store open source par construction** : chaque app publique
est deployee depuis un repo Git verifie. Les utilisateurs
peuvent en 1 clic voir le code source, signaler un bug, proposer
une feature, contribuer via PR, ou forker l'app et deployer leur
propre version sur le reseau. Le modele F-Droid/Linux applique
aux apps web P2P.
```

**Nouveau texte :**
```
**Plateforme a source verifiable** : chaque app publique est
deployee depuis un repo Git avec provenance auto-attestee
(Ed25519 + SLSA L1). Les utilisateurs peuvent verifier la
signature, voir le code source, signaler un bug, contribuer via
PR, ou forker l'app et deployer leur propre version sur le
reseau. Inspire par F-Droid — a terme, des builds reproductibles
multi-builder renforceront cette garantie (cf. LT-7 quorum).
```

**Rationale :** Aligne sur la taxonomie de confiance S65 (niveaux 0-5). Remplace
"open source par construction" (sur-promesse) par "source verifiable" (factuel).
Qualifie la comparaison F-Droid.

---

### 1.2 `## Source de verite pour le workflow Claude` — INCHANGE

Aucun changement necessaire. Le pointeur vers `docs/claude/README.md` reste valide.

---

### 1.3 `## Modele de rendu` — INCHANGE

Aucun changement technique au modele de rendu. Le bridge reste a 3 methodes
(Sprint 13+). Factory (S73-S75) n'ajoute pas de methodes bridge — c'est un
module daemon, pas une app iframe.

---

### 1.4 `## Deploy verifie` — MISE A JOUR MINEURE

**Ancien texte (lignes 48-51) :**
```
Apps publiques deployees **depuis le repo source** par le
coordinateur (clone → Keyoxide Ed25519 → zip → provenance.json
SLSA L1). Code sur le reseau = code du repo. Multi-forge, zero
OAuth. Cf. `sprint14_keyoxide_decision.md` (memory).
```

**Nouveau texte :**
```
Apps publiques deployees **depuis le repo source** par le
coordinateur (clone → Keyoxide Ed25519 → zip → provenance.json
SLSA L1). Provenance auto-attestee : le meme noeud clone, build
et signe. Multi-forge, zero OAuth.
Cf. `sprint14_keyoxide_decision.md` (memory).
```

**Rationale :** "Code sur le reseau = code du repo" est la sur-promesse
identifiee comme gap G2 CRITIQUE. Remplace par la description factuelle
"provenance auto-attestee".

---

### 1.5 `## Architecture` — INCHANGE

Aucun nouveau crate dans S65. Les futurs crates (nexus-factory-broker potentiel
en S74) seront ajoutes a leur sprint respectif. Pas d'anticipation.

---

### 1.6 `## Stack` — INCHANGE

Stack identique. iroh 0.98 maintenu pour Arc 1 (S65-S69). L'upgrade iroh 1.0
est un decision point S66 (cf. cross-cutting §8.1).

---

### 1.7 `## Structure des crates / packages` — MISE A JOUR MINEURE

**Ajout apres la ligne `archive/v1.2/` (ligne 99) :**
```
│   └── archive/v2.0/              # S61-64 (public verifiable feed: spec executable + feed local + sync P2P + verification tiers + hardening public)
```

**Rationale :** v2.0 existe deja dans le filesystem mais n'est pas documente dans
CLAUDE.md.

---

### 1.8 `## Securite` — INCHANGE

Le threat model et le runtime isolation roadmap restent les references.

---

### 1.9 `## Etat actuel` — MISE A JOUR MAJEURE

C'est la section qui change le plus. Remplacement complet du contenu.

**Ancien texte (lignes 117-188) :** tout le bloc actuel.

**Nouveau texte :**

```
## Etat actuel
- **Sprints 0-64 CLOSED**, v1.2 livree. **Tag v1.0 pose.**
  Projet Rust+Frontend pur depuis S50-S51.
  P2P valide cross-machine : LAN Win↔Mac, WAN dev↔VPS Helsinki.
  CI operationnel : Woodpecker ci.sbfb.world + GHA.
- **~1597 tests total** (1326 Rust / 265 Vitest / 6/6 size-limit)
  — tous verts code.
- **Roadmap v1.0 livree** :
  S59 = early adopter ready. S60 = end user ready → tag v1.0.
- **Roadmap S65-S75 — 3 arcs, 11 sprints** :
  Decision PO 2026-05-18. Detail :
  `.planning/research/s65_s75_cross_cutting_research.md`.
  - **Arc 1 — Credibilite publique (S65-S69)** :
    S65 contrat public (taxonomie confiance + badges UI + 7 carry) →
    S66 durabilite (persistence iroh-docs + crash recovery) →
    S67 gouvernance (CuratorVouched + UI curators) →
    S68 proof pack (release pipeline + evidence deploy E2E) →
    S69 pilote ferme (2-3 testeurs, premier contact utilisateurs).
    Gate 1 apres S69.
  - **Arc 2 — Recherche, Reputation, Verification (S70-S72)** :
    S70 RRV local-only (FTS5 index) →
    S71 proof cards (resultats enrichis) →
    S72 SearchManifest opt-in (P2P discovery).
    Gate 2 apres S72.
  - **Arc 3 — Code Factory (S73-S75)** :
    S73 templates (scaffolding) →
    S74 broker/sandbox (isolation OS, pas wasmtime) →
    S75 Babel dogfood (premiere app SBFB reelle).
  Estimation totale : ~24 semaines (~6 mois).
- Carry items distribues par sprint :
  **S65 MANDATORY :**
  P2-FEED-INSERT-NO-AUTH-TIER (3/3 — feed_insert auth tier).
  P2-VERIFY-ENTRY-VERSION-GUARD (1/3 → S65 — 5 LOC).
  P2-BADGE-WORDING-PREMATURE (pre-S14 → S65 — coeur sprint).
  **S65 dette pair :**
  P2-COMMIT-TITLE-FORMAT, P2-REVIEW-ORDER,
  P2-PYTHON-BLOCK-EXEMPTION (reclassification resolved),
  P2-EXPLORER-ESCAPE-SINGLE-QUOTE (1 LOC),
  P2-PLAYWRIGHT-SPECS-STALE (suppression zombies).
  **S66 :** P2-FEED-JOIN-HANDLE-LEAK, P2-ORPHAN-REPUBLISH-RECOVERY.
  **S68 :** P2-PROVENANCE-404-BRIDGE, P2-COVERAGE-DEPLOY-E2E.
  **S69 :** P2-VERIFY-LOCAL-KEY-ONLY, P2-PLAYWRIGHT-SPECS-STALE
  (re-ecriture specs pages actuelles).
  **Monitoring continu (pas de sprint specifique) :**
  P2-A-1 rand blocker upstream (exemption externe).
  P2-AUDIT-2 pre-release transitives iroh (decision point S66).
  P2-G-1 exe lock intermittent (non reproductible).
  **Hors scope S65-S75 :**
  T-NN+2 iframe Rust-wasm (triggers non actifs).
  LT-5 redundancy persistence (post-S75 sauf si S69 l'exige).
  LT-7 quorum E2E (post-S75 sauf si S69 l'exige).
  **Trigger-dependent :** LT-2 Radicle (trigger = push tag v1.0).
  **RESOLVED :** LT-6 iroh neighborhood (RESOLVED S32 Phase A).
  LT-3/LT-4 hors-sprint (post-v1.0).
- Zones rouges : R-iroh-audit P0 / R-wasmtime-cve P0 /
  R-libcrux-hax P2 / R-pyodide-escape (inchangees).
  R-wasmtime-cve : exclu pour Factory S74 (OS sandbox instead).
- Historique sprint-par-sprint → `docs/claude/SPRINT_LOG.md`.
```

**Changements majeurs :**
1. Suppression du resume detaille S64 (archives, pas besoin de le repeter).
2. Remplacement de "Roadmap post-v1.0 — Public Verifiable Protocol Feed 6 sprints"
   par la nouvelle roadmap 3 arcs / 11 sprints S65-S75.
3. Carry items redistribues par sprint au lieu d'une liste plate.
4. Items resolus retires (LT-6 -> note RESOLVED).
5. Ajout note wasmtime exclu pour Factory.

---

### 1.10 `## Commandes cles` — INCHANGE

Les commandes de build/test restent identiques.

---

### 1.11 `## Decisions architecturales gelees` — MISE A JOUR

**Ajout apres la derniere ligne actuelle (ligne 221) :**

```
- iroh 0.98 pour Arc 1 (S65-S69), evaluer upgrade 1.0 pour Arc 2+
- wasmtime exclu pour Factory (OS sandbox processus+filesystem,
  pas WASM isolation — 12 CVEs avril 2026 dont 2 Critical)
- Pilote S69 ferme (2-3 testeurs, pas public — R-iroh-audit P0)
- Vocabulaire "source verifiable" pas "open source" pour badges UI
- Factory = module daemon/broker local, pas app iframe
- Sequencage Arc 1 → Arc 2 → Arc 3 sauf feedback S69 contraire
```

**Rationale :** 6 decisions identifiees par la recherche cross-cutting (§8)
et le feedback GPT 5.5. Gelees pour eviter de les re-debattre a chaque sprint.

---

### 1.12 `## Principe de conception` — INCHANGE

Le principe reste valide et s'applique directement a la transition S64→S65.

---

### 1.13 `## Pre-launch protocol policy` — MISE A JOUR

**Ajout d'un paragraphe entre l'ancien "Apres le tag v1.0" et la fin :**

**Ancien texte (lignes 253-256) :**
```
Apres le tag `v1.0`, la politique bascule : chaque break bump la
version, chaque decoder accepte un range, chaque ajout de champ
carry un `#[serde(default)]` assume pour la compat ascendante.
Jusque-la, on edite le canonical librement.
```

**Nouveau texte :**
```
Le tag `v1.0` est pose localement mais **pas pousse vers origin**.
Le projet est toujours en regime pre-launch : aucun noeud tiers
ne consomme les wire formats en production. La politique pre-launch
reste active (versions a 1, pas de tolerant decoder multi-version).

**Feed protocol (S61+)** : `FEED_FORMAT_VERSION = 1`. Le
`PUBLIC_FEED_SPEC.md` definit le canonical. `verify_entry()` doit
checker `entry.version == FEED_FORMAT_VERSION` (carry
P2-VERIFY-ENTRY-VERSION-GUARD, cible S65).

Apres le **go-live** (pilote S69 ou push tag public), la politique
bascule : chaque break bump la version, chaque decoder accepte un
range, chaque ajout de champ carry un `#[serde(default)]` assume
pour la compat ascendante. Jusque-la, on edite le canonical
librement.
```

**Rationale :** Le tag v1.0 est pose mais pas pousse. Il faut clarifier que
le regime pre-launch continue. Ajout de la mention explicite du feed protocol
version qui est le sujet de P2-VERIFY-ENTRY-VERSION-GUARD. Le trigger de
bascule passe de "tag v1.0" a "go-live (pilote S69 ou push tag public)" car
c'est l'exposition a des noeuds tiers qui compte, pas le tag local.

---

### 1.14 `## Discipline de travail` — INCHANGE

Aucun changement au workflow.

---

### 1.15 `## Langue` — INCHANGE

Aucun changement aux conventions linguistiques.

---

## 2. `.planning/active/` — nettoyage et preparation S65

### 2.1 Archiver les fichiers S64

Les 15 fichiers S64 dans `.planning/active/` doivent etre deplaces vers
`.planning/archive/v2.0/` :

```bash
git mv .planning/active/sprint64_*.md .planning/archive/v2.0/
```

Fichiers a deplacer :
- `sprint64_audit_plan.md`
- `sprint64_design_review.md`
- `sprint64_kickoff.md`
- `sprint64_phase_A_preflight.md`
- `sprint64_phase_A_review.md`
- `sprint64_phase_B_preflight.md`
- `sprint64_phase_B_review.md`
- `sprint64_phase_C_preflight.md`
- `sprint64_phase_C_review.md`
- `sprint64_phase_D_preflight.md`
- `sprint64_phase_D_review.md`
- `sprint64_phase_E_preflight.md`
- `sprint64_phase_E_review.md`
- `sprint64_plan.md`
- `sprint64_verification.md`

**ATTENTION** : `sprint65_audit_plan.md` reste dans `active/` — c'est le
document produit par S64 Phase E qui sera consomme par S65 Phase 0.

### 2.2 Etat de `.planning/active/` apres archivage

Contenu restant :
```
.planning/active/
└── sprint65_audit_plan.md   # produit S64 Phase E, consomme S65 Phase 0
```

### 2.3 Documents a creer au kickoff S65

Le kickoff S65 creera ses documents standards :
- `sprint65_kickoff.md` — constat d'entree, D1-D5, plan outline
- `sprint65_plan.md` — plan detaille phases A-D
- `sprint64_audit_findings.md` — produit en Phase 0 (gate du sprint precedent)

En fin de sprint :
- `sprint65_verification.md`
- `sprint66_audit_plan.md` (ou `sprint65_audit_plan.md` mis a jour — selon le
  pattern habituel, le sprint N produit l'audit_plan pour le sprint N+1)

### 2.4 Roadmap formelle — placement

La roadmap S65-S75 est deja documentee dans deux fichiers :
- `.planning/research/s65_s75_cross_cutting_research.md` (analyse exhaustive)
- `.planning/research/public_verifiable_feed_roadmap.md` (roadmap originale 6 sprints)

**Recommendation :** Pas de nouveau fichier roadmap a creer. La roadmap
originale `public_verifiable_feed_roadmap.md` couvre S61-S66 (les 6 sprints
de l'ancienne roadmap). La recherche `s65_s75_cross_cutting_research.md`
etend a 11 sprints. Les deux vivent dans `research/` (cross-sprint). Le
pointeur dans CLAUDE.md §Etat actuel suffit.

---

## 3. `.planning/README.md` — MISE A JOUR

### 3.1 Table mapping versions

**Ajouter une ligne dans la table §Regroupement par version :**

**Ancien (ligne 84 environs) :**
```
| **v1.2** | S16+ (en cours) | Security hardening (loopback auth + GPU consent + VM roadmap) | TBD |
```

**Nouveau :**
```
| **v1.2** | S16-60 | Security hardening + carry resolution + Port Rust + installer + tag v1.0 | tag v1.0 |
| **v2.0** | S61-64 | Public Verifiable Protocol Feed (spec + feed local + sync P2P + verification + hardening) | `cf1100b` |
| **v2.1** | S65+ (en cours) | Contrat public + durabilite + gouvernance + pilote + RRV + Factory | TBD |
```

**Rationale :** v1.2 est CLOSED (tag v1.0 pose). v2.0 couvre S61-64 (feed
verifiable). v2.1 s'ouvre avec S65 (3 arcs). L'ouverture d'une nouvelle
version mineure au changement de theme (credibilite publique → pilote → RRV →
Factory) est coherente avec le pattern existant.

**Alternative :** Garder v2.0 pour tout S61-S75 si on considere que c'est la
meme release "Public Verifiable Protocol". Dans ce cas, pas de v2.1. Je
recommande v2.1 car le theme change radicalement (spec+feed → contrat public +
Factory).

**Decision PO requise :** v2.0 continue ou v2.1 s'ouvre ?

---

## 4. SPRINT_LOG.md — mise a jour

### 4.1 Section v2.0 header

**Ancien (ligne 14) :**
```
## v2.0 — Public Verifiable Protocol Feed (OPEN)
```

**Decisions possibles :**

**Option A — v2.0 continue (S61-S75, tout Public Verifiable) :**
Garder le header, ajouter les sprints S65-S75 un par un au fur et mesure.

**Option B — v2.0 CLOSED + v2.1 OPEN :**
```
## v2.1 — Credibilite publique + Factory (OPEN)

| Sprint | Etat | Tip cloture | Nb commits | Docs |
|---|---|---|---|---|
| 65 | PLANNED — Contrat public (taxonomie confiance + badges UI + 7 carry items) | -- | -- | -- |

---

## v2.0 — Public Verifiable Protocol Feed (CLOSED)
```

**Recommendation :** Option B. Les sprints S61-S64 forment un ensemble coherent
(spec→feed→sync→verification→hardening). S65+ est un theme different (contrat
public, pilote, RRV, Factory).

### 4.2 Lignes planifiees S65-S75

Ne PAS pre-remplir les lignes S66-S75. Le SPRINT_LOG documente les sprints
**livres**, pas les sprints planifies. Seul S65 peut avoir une ligne "PLANNED"
quand il s'ouvre.

La roadmap planifiee vit dans `.planning/research/s65_s75_cross_cutting_research.md`,
pas dans le SPRINT_LOG.

---

## 5. Decisions gelees — texte exact

### 5.1 Section a ajouter dans CLAUDE.md §Decisions architecturales gelees

Apres la derniere ligne actuelle ("Launcher Rust minimal (pas Tauri, browser = client)"),
ajouter :

```
- iroh 0.98 pour Arc 1 (S65-S69), evaluer upgrade 1.0 pour Arc 2+
- wasmtime exclu pour Factory (OS sandbox processus+filesystem,
  pas WASM isolation — 12 CVEs avril 2026 dont 2 Critical)
- Pilote S69 ferme (2-3 testeurs, pas public — R-iroh-audit P0)
- Vocabulaire "source verifiable" pas "open source" pour badges UI
- Factory = module daemon/broker local, pas app iframe
- Sequencage Arc 1 → Arc 2 → Arc 3 sauf feedback S69 contraire
```

### 5.2 Rationale par decision

| Decision | Source | Rationale court |
|---|---|---|
| iroh 0.98 Arc 1 | cross-cutting §8.1 | Upgrade 1.0 = 2-3 phases breaking changes, distraction pendant Arc 1 credibilite. Evaluer apres pilote. |
| wasmtime exclu | cross-cutting §8.2 | Factory execute git/npm/cargo (processus OS), pas WASM. 12 CVEs avril 2026. iframe sandbox + CSP suffisent pour apps. |
| Pilote ferme | cross-cutting §8.3 | R-iroh-audit P0 rend pilote public irresponsable. 2-3 amis = feedback suffisant. |
| Source verifiable | S65 research §5 | "Open source" sur-promet (gap G2/G3). "Source verifiable" est factuel. F-Droid best practice. |
| Factory = module | S73-S75 research | Factory est une UI + broker local + workspace sandbox + index @dev. Pas une app iframe. |
| Sequencage Arcs | cross-cutting §5 | Chemin critique S65→S66→S69→S70→S71→S72. Arc 3 independant mais solo-maintainer = sequentiel. |

---

## 6. PATTERNS.md (Rust + Shell) — mise a jour tech debt

### 6.1 `docs/rust/PATTERNS.md`

**T-NN+2 (iframe Rust-wasm)** : INCHANGE. Reste "open, blocked by upstream".
Reclassifie "hors scope S65-S75" dans CLAUDE.md mais le detail technique
reste dans PATTERNS.md.

Aucun autre item tech debt dans PATTERNS.md n'est impacte par la roadmap S65-S75.

### 6.2 `docs/shell/PATTERNS.md`

**Aucun changement requis.** Les items tech debt T1-T7 sont tous soit CLOSED
soit specifiques au shell React. La roadmap S65-S75 n'impacte pas les
patterns shell existants.

**A surveiller S65 :** Si la Phase B (migration badges UI) introduit un
nouveau pattern (TrustBadge composant, taxonomie de confiance), un nouveau §P
devrait etre ajoute a PATTERNS.md shell. Mais c'est le travail du sprint S65,
pas de cette mise a jour.

---

## 7. Carry items — tableau decision final

### 7.1 Items a RETIRER du §Etat actuel

| Item | Raison retrait |
|---|---|
| LT-6 iroh neighborhood | RESOLVED S32 Phase A (deja marque resolved, garder une note) |
| LT-3/LT-4 | Hors-sprint post-v1.0 (garder une note, pas dans la liste principale) |

### 7.2 Items a REDISTRIBUER (ancien → nouveau)

| Item | Ancien placement | Nouveau placement | Raison |
|---|---|---|---|
| P2-FEED-INSERT-NO-AUTH-TIER | "3/3 MANDATORY S65" | S65 MANDATORY | Confirme, inchange |
| P2-VERIFY-ENTRY-VERSION-GUARD | "1/3, before go-live" | S65 MANDATORY (cote a cote FEED-INSERT) | 5 LOC, pre-requis contrat public |
| P2-BADGE-WORDING-PREMATURE | "pre-existant S14" | S65 (coeur sprint) | Litteralement le sujet S65 |
| P2-COMMIT-TITLE-FORMAT | 2/3 | S65 dette pair | Process fix 20 LOC |
| P2-REVIEW-ORDER | 2/3 | S65 dette pair | Process fix 10 LOC |
| P2-PYTHON-BLOCK-EXEMPTION | 2/3 | S65 reclassification resolved | Projet 100% Rust depuis S50 |
| P2-EXPLORER-ESCAPE-SINGLE-QUOTE | 2/3 | S65 dette pair | 1 LOC fix |
| P2-PLAYWRIGHT-SPECS-STALE | 2/3 | S65 (suppression) + S69 (re-ecriture) | 12 fichiers zombies a supprimer |
| P2-FEED-JOIN-HANDLE-LEAK | 1/3 | S66 | Shutdown lifecycle = sujet S66 |
| P2-ORPHAN-REPUBLISH-RECOVERY | 1/3 | S66 | Crash recovery = sujet S66 |
| P2-PROVENANCE-404-BRIDGE | 2/3 | S68 | UX verification = proof pack |
| P2-COVERAGE-DEPLOY-E2E | 2/3 | S68 | Deploy roundtrip E2E = proof pack |
| P2-VERIFY-LOCAL-KEY-ONLY | 2/3 | S69 | Cross-node verification = pilote |
| P2-A-1 rand blocker | exemption externe | Monitoring | Upstream, pas d'action |
| P2-AUDIT-2 iroh transitives | exemption externe | Monitoring (decision point S66) | iroh 1.0-rc.0 sorti |
| P2-G-1 exe lock | monitoring | Monitoring | Non reproductible |
| T-NN+2 iframe Rust-wasm | PATTERNS §P34 | Hors scope S65-S75 | Triggers non actifs |
| LT-2 Radicle | trigger pending | Trigger-dependent (push tag v1.0) | S66-S67 si pousse |
| LT-5 redundancy persistence | reclassifie S26 | Hors scope S65-S75 (sauf S69) | Post-S75 |
| LT-7 quorum E2E | post-tag | Hors scope S65-S75 (sauf S69) | Post-S75 |

### 7.3 Items RESOLUS a confirmer

| Item | Evidence resolution |
|---|---|
| LT-6 iroh neighborhood | "RESOLVED S32 Phase A" — deja marque, garder la note mais plus dans la liste active |

---

## 8. Memory files — mises a jour requises

### 8.1 `nexus_grid_pivot.md` (memory)

**Mise a jour necessaire apres le commit :**
- Changer "S64 CLOSED" → "S64 CLOSED, S65 en cours"
- Remplacer "6 sprints (5+1 reserve)" par "11 sprints, 3 arcs (S65-S75)"
- Ajouter mention des 3 arcs et gates

### 8.2 `MEMORY.md` (memory)

**Ligne a mettre a jour :**
```
- [SBFB pivot 2026-04-10](nexus_grid_pivot.md) — S64 CLOSED (hardening public, Sprint 4/6). 1326 Rust / 265 Vitest. Prochain S65 go-live.
```
→
```
- [SBFB pivot 2026-04-10](nexus_grid_pivot.md) — S64 CLOSED. 1326 Rust / 265 Vitest. Roadmap S65-S75 (3 arcs: credibilite publique, RRV, Factory). Prochain S65 contrat public.
```

**Ligne a mettre a jour :**
```
- [Public Feed roadmap](public_feed_roadmap.md) — Decision PO 2026-05-13 : 6 sprints (5+1 reserve) pour credibilite publique protocole verifiable. Gate scission S2.
```
→
```
- [Public Feed roadmap](public_feed_roadmap.md) — Decision PO 2026-05-18 : 11 sprints S65-S75 (3 arcs). Arc 1 credibilite (S65-S69) + Arc 2 RRV (S70-S72) + Arc 3 Factory (S73-S75). Gates entre arcs.
```

---

## 9. Checklist d'execution

L'execution de ce plan se fait en un seul commit (ou 2 si archivage + edits sont separes) :

### Etape 1 — Archivage S64
```bash
git mv .planning/active/sprint64_*.md .planning/archive/v2.0/
```

### Etape 2 — Edits CLAUDE.md
Appliquer les changements §1.1, §1.4, §1.7, §1.9, §1.11, §1.13.

### Etape 3 — Edit .planning/README.md
Appliquer §3.1 (table versions).

### Etape 4 — Edit SPRINT_LOG.md
Appliquer §4.1-4.2 (header v2.0 CLOSED, v2.1 OPEN).

### Etape 5 — Memory files
Appliquer §8.1-8.2.

### Etape 6 — Verification
- `CLAUDE.md` ne contient plus "App store open source par construction"
- `CLAUDE.md` ne contient plus "Code sur le reseau = code du repo"
- `CLAUDE.md` §Decisions gelees a 15 items (9 anciens + 6 nouveaux)
- `.planning/active/` contient uniquement `sprint65_audit_plan.md`
- `.planning/archive/v2.0/` contient les 15 fichiers S64
- SPRINT_LOG.md a v2.0 CLOSED et v2.1 OPEN

---

## 10. Questions pour decision PO

1. **v2.0 continue vs v2.1 ?** Recommendation : v2.1 (theme change radicalement).
   Impact : header SPRINT_LOG, dossier archive, .planning/README.md table.

2. **Archiver S64 maintenant ou au kickoff S65 Phase 0 ?** Recommendation :
   maintenant (dans ce commit de mise a jour docs). Le Pattern habituel est
   "deplacer a l'ouverture du sprint suivant", mais les fichiers sont dans
   active/ depuis la cloture S64 et polluent le contexte.

3. **Le resume detaille S64 dans §Etat actuel doit-il etre preserve ?**
   Recommendation : NON. Le detail vit dans SPRINT_LOG.md et dans les archives.
   Le §Etat actuel doit refeter l'etat ACTUEL, pas l'historique.

---

*Plan redige 2026-05-18 — pret pour execution.*
