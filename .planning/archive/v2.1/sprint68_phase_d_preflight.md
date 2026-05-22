# Sprint 68 Phase D — preflight G8

Date : 2026-05-21 | HEAD : `a201b3e` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- `feedback_approach.md` : pick deepest, research before code, OSS prior art obligatoire (G10). Phase D est UI frontend — pas de lib externe nouvelle, pas de primitive crypto. Le principe s'applique au pattern UI (expandable card, score display).
- `feedback_context7_systematic.md` : context7 avant tout code touchant lib/API. Phase D ne touche aucune nouvelle lib (React, lucide-react, shadcn/radix deja en place). N/A pour queries context7 supplementaires.
- `vision_model.md` : aucune tension — Phase D ne touche ni funding, ni gouvernance, ni institutionnalisation.
- `feedback_kudos_non_monetary.md` : la ProofCard affiche un "confidence score" 0-100, PAS un kudos score. Aucun champ kudos dans ProofCard. N/A.
- `fairness_vision.md` : N/A — ProofCard mesure la completude de preuve d'un projet (provenance, license, curation), pas la contribution worker. Aucune tension.
- `nexus_grid_pivot.md` : D5 gelee "Proof Card UI composant shell Browse" confirmee. D1 formule gelee (FORMULA_VERSION=1). D16 formula_version gelee roadmap v4. Aucune tension avec Phase D.

Tensions plan vs memory : **aucune**.

## Scans (all clean)

- S1a OSS prior art : 5 projets recherches (OpenSSF Scorecard Visualizer, F-Droid Verification, Sigstore/cosign, VerificationDetail SBFB, npms.io), APPROACH-ALIGNED — clean
- S1b deps : 0 nouvelle dep Phase D, 0 CVE critique sur deps existantes — clean
- S2 historiques : 16 commits BrowsedProject.tsx + 5 commits THREAT_MODEL.md, bodies lus — clean
- S3 threat model : FULL, 5 vectors analyses — clean
- S4 wire format : FULL / canonical.rs lu integralement / VERSION=1, Day 0 preserved — clean

---

## S1a — OSS prior art deep analysis

### Projets analyses en profondeur

#### [OpenSSF Scorecard Visualizer] — ossf/scorecard-visualizer (https://github.com/ossf/scorecard-visualizer)
- Type : React app (create-react-app) pour afficher les donnees OpenSSF Scorecard API
- Pattern architectural : score 0-10 par check, chaque check expandable avec reasoning/details. Vue comparateur entre versions. Donnees structurees checks[]→{score, reason, documentation_url}.
- Pattern UI retenu : score global proéminent + checks individuels detailles = aligne avec ProofCard (confidence 0-100 + layers). Le pattern "score global + evidence layers expandable" est le consensus dans l'ecosysteme scoring securite.
- Verdict : ALIGNED

#### [F-Droid Verification] — verification.f-droid.org (https://verification.f-droid.org/)
- Type : service web de verification de builds reproductibles
- Pattern : status cards par app, badges rebuilder (verified/unsigned). IzzyOnDroid 5-level graph. NLnet funding overhaul 2025.
- Pattern UI retenu : badges par couche de verification (pas juste un badge unique). Le pattern "checklist visuelle de preuves" est utilise par F-Droid pour montrer QUOI est verifie (build, signature, source), pas juste SI c'est verifie.
- Verdict : ALIGNED

#### [Sigstore/Cosign] — sigstore.dev (https://docs.sigstore.dev/)
- Type : outils CLI de signature/verification, pas de composant UI standard
- Pattern : cosign v2.6.0 bundles, in-toto attestations. Pas de composant React UI dans l'ecosysteme Sigstore — chaque projet construit le sien.
- Finding : espace design ouvert pour les composants d'affichage de preuve. SBFB definit son propre composant (ProofCard.tsx) — pas de lib existante a adopter.
- Verdict : N/A (pas de prior art UI a comparer)

#### [VerificationDetail SBFB] — composant existant dans le projet (web/src/components/VerificationDetail.tsx)
- Fichiers source lus : VerificationDetail.tsx (261 LOC), BrowsedProject.tsx (693 LOC)
- Pattern architectural : Dialog modal shadcn, fetch lazy au clic, 7 champs provenance, badge vert/rouge verification Ed25519, bouton reverify. Grid layout dl/dt/dd.
- Edge cases geres : hash mismatch warning, loading state, empty state (404), error state.
- Pattern retenu pour ProofCard : meme architecture de composant React (fetch on mount, etat loading/loaded/error), mais en carte expandable inline au lieu de modal dialog. Coherence visuelle avec le glassmorphism existant.
- Verdict : ALIGNED (ProofCard suit les patterns UI etablis dans le projet)

#### [npms.io / npm quality score] — npm registry scoring
- Type : score qualite 0-1 par package npm, 3 metriques (quality, popularity, maintenance)
- Pattern : demande communaute pour "expand quality et voir pourquoi le score" (GitHub Discussion #128411). Confirme le besoin d'un score decompose en facteurs visibles.
- Verdict : ALIGNED (score decompose = pattern attendu par les utilisateurs)

### Tableau comparatif

| Aspect | Plan Phase D | OpenSSF Scorecard | F-Droid | VerificationDetail SBFB |
|--------|-------------|-------------------|---------|------------------------|
| Score global + details | confidence 0-100 + layers expandable | score 0-10 + checks expandable | badges par couche + graph | verified/failed badge + 7 champs detail |
| Composant UI | carte inline expandable | page dediee | page web status | modal Dialog |
| Fetch pattern | bridge proof_card_get au mount | API call au chargement | static JSON | lazy fetch au clic |
| Evidence layers | provenance, license, freshness, curation, hash | 18+ checks securite | build repro, signature, source | provenance seulement |
| Risk factors visibles | oui (risk_factors array) | oui (reasoning per check) | non (badge binaire) | oui (hash mismatch) |

### Finding S1a

- Classification : **APPROACH-ALIGNED**
- Evidence : OpenSSF Scorecard et npms.io utilisent le meme pattern "score global decompose en facteurs visibles". F-Droid utilise le pattern "checklist par couche de verification". Le plan ProofCard combine les deux : score numerique + layers + risk factors. VerificationDetail SBFB existant fournit le template de composant React coherent avec le projet.
- Impact sur le plan : aucun — l'approche est alignee avec l'etat de l'art.

---

## S1b — Deps/libs versions + CVE

### Deps perimetre Phase D

Phase D ne touche que le frontend. Aucune nouvelle dep ajoutee. Deps existantes utilisees :

| Dep | Version pinned | CVE check | Status |
|-----|---------------|-----------|--------|
| react | ^19.2.4 | CVE-2025-55182 (React Server Components RCE CVSS 10.0) — **N/A** : SBFB est une SPA Vite, pas de React Server Components | clean |
| react-dom | ^19.2.4 | idem | clean |
| lucide-react | ^1.7.0 | Aucun CVE connu 2025-2026. Socket.dev : pas de vulnerabilite signalee | clean |
| @radix-ui/react-dialog | ^1.1.15 | Utilise par VerificationDetail, pas par ProofCard (expandable inline, pas modal) | clean |
| @tanstack/react-query | ^5.96.2 | Pas de CVE critique 2025-2026 | clean |
| zod | ^3.25.76 | Pas de CVE critique | clean |
| tailwindcss | ^4.2.2 | CSS only, pas de surface d'attaque | clean |

### Finding S1b

0 delta deps. 0 CVE critique affectant le perimetre Phase D. La CVE-2025-55182 React Server Components est non applicable (SPA Vite). Clean.

---

## S2 — Decision chain reconstruction

### Fichiers scannes

- `web/src/pages/BrowsedProject.tsx` : 16 commits lus (bodies complets des 6 plus pertinents)
- `docs/security/THREAT_MODEL.md` : 5 commits lus (bodies complets)
- `web/src/bridge/protocol.ts` : commit f9d722e (Phase A S68) lu
- `web/src/bridge/useBridge.ts` : commit f9d722e (Phase A S68) lu

### Decisions historiques trouvees

#### Decision 1 : Vocabulaire badges provenance (Sprint 65)

- Sprint 65, sha `de9d55f` : migration vocabulaire badges UI. "Verifie"→"Provenance", "Auto-publie"→"Upload direct", title="Provenance auto-attestee (SLSA L1)".
  Body extrait : "Aligne tous les badges et labels UI avec la nomenclature TRUST_TAXONOMY.md"
- Sprint 65, sha `54f13eb` : badge provenance dynamique dans BrowsedProject. 4 etats visuels (loading/verified/failed/absent).
  Body extrait : "Badge provenance dynamique dans BrowsedProject : appel automatique GET /api/v1/project/{id}/provenance"
- Reverse-commit check :
  1. `git log --all --oneline "de9d55f..HEAD" -- web/src/pages/BrowsedProject.tsx` → 2 commits (S66 C + S68 A), aucun ne contient revert/undo/unblock
  2. `git log --all --grep="de9d55f" --oneline` → 0 hits
  3. Status : **active** — aucune reversion
- Impact phase D : **aucun** — la ProofCard est un composant additionnel, elle ne remplace pas le badge provenance existant dans la top bar. Les deux coexistent.

#### Decision 2 : VerificationDetail modal (Sprint 63)

- Sprint 63, sha `272523c` : composant modal shadcn Dialog pour afficher les details de verification provenance.
  Body extrait : "Nouveau composant modal shadcn Dialog [...] Badge ShieldCheck existant devient button cliquable"
- Reverse-commit check :
  1. `git log --all --oneline "272523c..HEAD" -- web/src/components/VerificationDetail.tsx` → 1 commit (S66 fix provenance hash), pas de revert
  2. Status : **active**
- Impact phase D : **aucun** — ProofCard est une carte inline expandable dans le body de la page, pas un remplacement du modal VerificationDetail. Les deux sont complementaires (VerificationDetail = provenance seulement, ProofCard = score composite + toutes les couches).

#### Decision 3 : THREAT_MODEL structure d'ajout (Sprint 66-67)

- Sprint 66, sha `ea87547` : ajout §10 Feed surface (T-FEED-1..4). Pattern : section numerotee avec threats tabulaires (Dimension/Valeur).
  Body extrait : "docs/security/THREAT_MODEL.md | Section §10 Feed surface (T-FEED-INTEGRITY..CLOCK-SKEW)"
- Sprint 67, sha `f46bc66` : ajout §11 Search surface (T-SEARCH-INJECTION, T-CURATOR-VOUCH, T-SEARCH-DOS). Renommage §11→§12 (revue et evolution).
  Body extrait : "THREAT_MODEL.md — §11 Search surface [...] closure P2-THREAT-MODEL-FEED-SURFACE 3/3, renommage §11→§12"
- Reverse-commit check :
  1. Aucun revert sur THREAT_MODEL.md depuis S66
  2. Status : **active** — chaque sprint qui ajoute une surface d'attaque ajoute une section §N+1 et renumerote §Revue
- Impact phase D : **respect du pattern** — Phase D doit ajouter §12 ProofCard surface et renommer l'actuel §12 (Revue) en §13. Meme format tabulaire Dimension/Valeur.

### Memory constraints

- `feedback_approach.md` : pick deepest, pas de band-aid. Phase D est un composant UI — la profondeur est dans l'affichage exhaustif des couches (pas juste le score), ce qui est le plan.
- `feedback_kudos_non_monetary.md` : ProofCard n'utilise pas les kudos. Le score "confidence" mesure la completude de preuve, pas la reputation. Aucune tension.

---

## S3 — Threat model analysis

### Primitive analysee : ProofCard UI + T-PROOFCARD-FORMULA-GAME

### Assets en jeu

- A1 ProofCard score (confidence 0-100) : criticite **medium**. Le score influence la perception utilisateur de la fiabilite d'un projet. Un score trompeur pourrait conduire a installer un projet malveillant.
- A2 Evidence layers (provenance, license, freshness, curation, hash) : criticite **low**. Donnees derivees des donnees daemon existantes (browse entry, feed, provenance). Pas de donnee confidentielle.

### Threat actors

- TA1 Fournisseur d'app malveillant (AD5) : optimise les facteurs mesurables pour obtenir un score eleve sans substance (score gaming). Capacite : controle le repo source, peut fournir provenance valide, commit recent, license SPDX. Motivation : gagner la confiance utilisateur.
- TA2 Attaquant reseau (AD3) : tente de forger une ProofCard avec un score different de celui calcule par le daemon local. Capacite : publie des messages gossip crafted. Motivation : manipulation reputation.

### Attack vectors identifies

1. **V1 Formula gaming (T-PROOFCARD-FORMULA-GAME)** : un attaquant optimise toutes les couches mesurables (provenance verifiee, repo public, commit frais, license SPDX, curator vouch) pour obtenir un score 100 tout en livrant un payload malveillant. Le score mesure la **completude de preuve**, pas la **qualite du code**.
   - Asset vise : A1 (perception utilisateur)
   - Couverture : **nouveau** — c'est la menace planifiee dans le plan §7.2 (T-PROOFCARD-FORMULA-GAME dans THREAT_MODEL.md §12)
   - Severity : M (le score est explicitement labelle "completude de preuve", pas "securite")

2. **V2 Forge ProofCard reseau** : un noeud byzantin envoie une ProofCard fabriquee pour un projet distant.
   - Asset vise : A1
   - Couverture : **deja couvert** — la ProofCard est un compute local. Le daemon compute la carte a la volee depuis ses donnees locales (browse, feed, provenance). Il n'y a pas de ProofCard en transit sur le reseau. Un noeud distant ne peut pas forger la ProofCard d'un autre daemon.
   - Severity : N/A (attaque impossible par design)

3. **V3 XSS via risk_factors ou layer names** : le composant React affiche des strings provenant du daemon. Si un attaquant controle le contenu de `risk_factors[]` ou `curator_names[]`, il pourrait tenter une injection XSS.
   - Asset vise : browser session utilisateur
   - Couverture : **deja couvert** — React echappe automatiquement les strings dans JSX. Pas de `dangerouslySetInnerHTML` dans le projet frontend (verifie grep). Les strings sont rendues via `{variable}` JSX.
   - Severity : L (mitigue par React auto-escaping)

4. **V4 DoS via requetes proof_card_get repetees** : flood du endpoint proof_card_get pour epuiser les ressources daemon.
   - Asset vise : daemon availability
   - Couverture : **deja couvert** par le bearer auth + small corpus + compute local O(1). Meme profil que T-SEARCH-DOS (§11) — bearer auth protege l'endpoint, le compute est local et rapide (~1ms).
   - Severity : L

5. **V5 Score confusion utilisateur** : l'utilisateur interprete le score comme une garantie de securite alors qu'il mesure la completude de preuve.
   - Asset vise : A1 (perception)
   - Couverture : **mitigue par design** — le composant affiche les couches detaillees (pas juste le chiffre), les risk factors sont visibles, et le label est "completude de preuve" (pas "securite"). T-PROOFCARD-FORMULA-GAME documente cette limitation.
   - Severity : M (residuel acceptable — le score est transparently decompose)

### Mitigations existantes

- V2 couvert : ProofCard = compute local (pas de wire format reseau)
- V3 couvert : React JSX auto-escaping + pas de dangerouslySetInnerHTML
- V4 couvert : bearer auth + compute local O(1)

### Gaps identifies

- GAP1 V1 T-PROOFCARD-FORMULA-GAME : severity M — **planifie Phase D** (ajout THREAT_MODEL.md §12). Le score ne mesure que la completude, pas la qualite. Mitigation : label explicite + couches detaillees + formula_version pour iterer.

### Regression check

- La ProofCard UI ne diminue l'efficacite d'aucune mitigation T0-T5 existante.
- La ProofCard ne cree aucun nouveau vecteur reseau (compute local seulement).
- L'endpoint proof_card_get suit le meme pattern auth que les endpoints existants.

### Verdict S3 : clean (1 gap M planifie dans le livrable Phase D lui-meme)

---

## S4 — Wire format deep audit

### canonical.rs lu integralement : oui (296 lignes)

### Structs verifiees

Phase D ne touche PAS canonical.rs ni aucune struct wire format. La ProofCard est un artefact local compute (proof_card.rs dans nexus-coordinator-rs) qui :
- N'a PAS de DOMAIN_*_V1 tag (pas dans la surface de signature)
- N'utilise PAS canonical_bytes() (pas de JCS)
- N'est PAS transmis sur le reseau (compute local, HTTP local response)
- A `#[derive(Serialize)]` uniquement pour la reponse HTTP JSON (axum JSON response)
- N'a PAS de `#[serde(default)]` (struct jamais deserialisee depuis l'exterieur)
- `FORMULA_VERSION = 1` est une constante locale (pas *_FORMAT_VERSION protocolaire)

#### ProofCard (proof_card.rs:40-51)
- version = FORMULA_VERSION = 1 (local, pas wire) : OK
- serde derives : Serialize seulement (pas Deserialize — never read from wire) : OK
- serde(default) : absent : OK
- DOMAIN signature : N/A (pas dans la surface canonical_bytes) : OK
- JCS serialization : N/A : OK
- Option<T> usage : OK (ProofCardHash.archive_hash, provenance_hash, etc.)

### Day 0 check

- D1 ProofCard struct Rust + formule score deterministe : **respectee** — struct livree Phase A, UI Phase D la consomme via l'endpoint existant. Aucune modification de la struct ou de la formule.
- D5 Proof Card UI composant shell Browse : **c'est Phase D** — le plan implemente exactement cette D5.
- D2, D3, D4 : non touchees par Phase D.
- D1..D5 sprint courant : aucune contredite.

### Decisions actees pivot.md

Verification des 12 decisions actees + extensions :
- #12 Archive zip format universel, daemon blob-serve : non impacte
- #13 postMessage bridge seul canal : proof_card_get est deja dans le bridge (Phase A). Coherent.
- #14 Deploy verifie from source : ProofCard consomme les donnees provenance existantes. Coherent.
- D16 formula_version gelee : FORMULA_VERSION = 1 dans proof_card.rs:12. Non bumpee. Coherent.
- Aucune decision actee contredite.

### Pre-launch policy

- `*_FORMAT_VERSION` / `*_ANNOUNCEMENT_VERSION` : non touchees
- Feed extensible raw-op : ProofCard comme feed op → scope cut S70+ (respecte)
- `#[serde(default)]` : non ajoute
- Tests "legacy decode" : aucun (ProofCard n'est jamais deserialisee depuis le reseau)

### Version constants grep

```
FORMULA_VERSION: u32 = 1      (proof_card.rs:12, local compute)
```
Aucune constante *_FORMAT_VERSION ou *_ANNOUNCEMENT_VERSION modifiee.

---

## Telemetrie preflight (agent deep)

- Duree totale : ~8m
- S1a : ~3m / 5 projets OSS analyses / 2 fichiers source code projet lus (VerificationDetail.tsx 261 LOC + BrowsedProject.tsx 693 LOC) / 954 LOC reviewees / 0 context7 queries (aucune nouvelle lib) / 6 WebSearch queries / finding : APPROACH-ALIGNED
- S1b : ~1m / 7 deps scannees / 2 CVE searches / finding : clean (0 delta)
- S2 : ~2m / 21 commit bodies lus (16 BrowsedProject + 5 THREAT_MODEL) / 0 archive files (sprint actif) / 6 memory files lus / finding : clean
- S3 : FULL / ~1m / 5 vectors analyses / 1 gap M (planifie Phase D)
- S4 : FULL / ~1m / 1 struct verifiee (ProofCard) / canonical.rs lu integralement : oui

## Action

Proceder code phase D.
