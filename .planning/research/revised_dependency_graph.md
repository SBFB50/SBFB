# Graphe de Dependances Revise S65-S75 — Arbitrage Doc A / Doc B

**Date :** 2026-05-18
**Objet :** Fusionner la roadmap canon (Doc A) et la recherche Factory-first (Doc B)
**Statut :** ANALYSE — soumis au PO pour decision

---

## 1. Matrice sprint-par-sprint — Doc A vs Doc B

| Sprint | Doc A (roadmap canon) | Doc B (canary pivot) | Conflit ? |
|--------|----------------------|---------------------|-----------|
| **S65** | Contrat Public : taxonomie confiance, badges UI, auth tier feed, version guard, 8 carry items dette | Contrat Public + contrat Factory/Babel : idem + manifest app v2, artefacts sprint app, gates Factory G0-G10 | **OUI** — Doc B ajoute du scope (manifest v2, gates Factory). Doc A est purement confiance/wording. |
| **S66** | Durabilite : iroh data_dir, FsStore, feed republish, RevocationCache persistence, E2E restart | Durabilite (identique) : blob store persistant, restart E2E, republish, feed join, evidence restart | **NON** — Meme contenu, meme ordering. Consensus. |
| **S67** | Gouvernance de Confiance : CuratorVouched/Disendorsed feed ops, multi-curator overlay, UX confiance, stale detection, feed v1->v2 bump | **Factory Foundation / Sprint OS** : module/broker Factory minimal, template statique, SBFB.json v2, node_id deprecation, factory.template.lock, factory.provenance.json, sprint skeleton | **OUI — CONFLIT MAJEUR.** Doc A met la gouvernance ici. Doc B met Factory ici. Sujet completement different. |
| **S68** | Pack de Preuves Release : proof pack CLI, SBOM, canary refresh, verify.sh, absorbe PROVENANCE-404-BRIDGE + COVERAGE-DEPLOY-E2E | **Broker, preview, publish gate** : UI /factory, diff preview, apply, preview sandbox, scan secrets, path traversal deny, publish-check, proof pack Factory, integration deploy-from-repo | **OUI — CONFLIT MAJEUR.** Doc A met le proof pack ici. Doc B met le broker/sandbox Factory ici. |
| **S69** | Pilote Ferme : checklist prereqs, invite mechanism, installeur cross-platform, feedback collector, scenarios test guides, go/no-go, absorbe VERIFY-LOCAL-KEY-ONLY + PLAYWRIGHT-SPECS-STALE | **Babel Reader canari ferme** : domain pack Babel, app generee par Factory, fixtures multilingues, source manifests, storage, reviews, provenance, Browse, pilote ferme | **OUI — CONFLIT DE SCOPE.** Doc A = pilote generique. Doc B = Babel canari + pilote. Doc B subsume le pilote de Doc A dans le contexte Babel. |
| **S70** | RRV LocalOnly : Tantivy (fallback FTS5), index local, API search, bridge method, app sbfb-search MVP | RRV LocalOnly sur corpus reel : FTS5-first (pas Tantivy), index apps Factory/Babel, index manifest/provenance/feed, recherche avec citations, labels separes | **PARTIEL.** Meme theme, mais Doc B choisit FTS5-first et indexe du contenu reel (Babel/Factory). Doc A choisit Tantivy-first avec fallback FTS5. |
| **S71** | RRV Proof Cards : ProofCard model, computation deterministe, API + bridge, integration search, tests adversariaux | Proof Cards : Proof Card Babel, Proof Card Factory artifact, score completude, warnings | **NON** — Meme contenu. Doc B ajoute explicitement Babel/Factory comme sujets de proof cards, ce qui est un enrichissement, pas un conflit. |
| **S72** | SearchManifest Opt-In : format + signing, publication opt-in via iroh, discovery + verification, anti-spam + privacy | SearchManifest opt-in (identique) | **NON** — Consensus. |
| **S73** | Code Factory Templates : SBFB.json v2, template engine + 3 templates, CLI `sbfb create`, template verification + lock | Factory hardening / templates : templates additionnels, template lock stable, deuxieme app, migration v1/v2, docs | **OUI — CONFLIT DE TIMING.** Doc A met la fondation Factory ici. Doc B met le hardening Factory ici (la fondation etant deja en S67-S68). |
| **S74** | Code Factory Broker/Sandbox : broker architecture + routes, diff generation, review UI page /factory, preview sandbox + publish gate | Babel translation beta : task_submit traduction mock/worker, resultat stocke, review utilisateur, provenance draft, fallback fixtures | **OUI — CONFLIT MAJEUR.** Doc A met le broker ici. Doc B met Babel translation beta (le broker etant deja en S68). |
| **S75** | Babel Dogfood / Domain Packs : domain pack format, Babel reader via Factory, bridge integration, deploy verifie, spec domain pack | Pack produit defendable : evidence pack final, Babel/Factory proof cards, RRV local + SearchManifest, release narrative, go/no-go public | **OUI — CONFLIT DE SCOPE.** Doc A : premiere creation Babel. Doc B : consolidation finale (Babel existe depuis S69). |

### Synthese des conflits

| Zone | Nature du conflit | Gravite |
|------|-------------------|---------|
| S67 | Gouvernance vs Factory Foundation | HAUTE — sujet radicalement different |
| S68 | Proof Pack vs Broker/Preview/Publish | HAUTE — sujet radicalement different |
| S69 | Pilote generique vs Babel canari + pilote | MOYENNE — Doc B subsume Doc A |
| S70 | Tantivy-first vs FTS5-first | BASSE — decision technique, pas structurelle |
| S73 | Factory foundation vs Factory hardening | MOYENNE — timing |
| S74 | Broker/Sandbox vs Babel translation beta | HAUTE — sujet different |
| S75 | Babel creation vs Pack produit defendable | MOYENNE — scope |

---

## 2. Nouveau graphe de dependances (si pivot adopte)

### 2.1 Analyse des dependances reelles

**S65 -> S66 : INCHANGE**
Auth tier (S65) doit preceder la persistence (S66). Des operations non-autorisees persistees = corruption permanente. Dependance D-HIDDEN-1 confirmee.

**S66 -> S67 Factory : OUI, la persistence est requise.**
Raison : Factory genere des apps et les publie. Si le blob store est volatil, une app publiee par Factory disparait au restart. Le publish path (`deploy-from-repo`) cree une archive + provenance + Browse entry. Sans persistence (S66), le roundtrip Factory -> deploy -> Browse est ephemere.

Contre-argument : "Factory genere du code, pas des blobs P2P". FAUX. Factory genere un repo, mais le deploy path le transforme en blob P2P (archive zip dans iroh-blobs). Sans FsStore, le blob est perdu au restart. Factory sans persistence = demo jetable, pas un outil.

**S67 Factory -> S68 Broker : OUI, dependance forte.**
Le broker (S68) est l'orchestrateur des operations privilegiees de Factory. Il necessite les templates et le module Factory de base (S67). Le diff/preview/publish gate s'applique aux artefacts que la fondation Factory sait generer.

**S68 Broker -> S69 Babel canari : OUI, dependance forte.**
Babel est genere PAR Factory (decision PO). Si le broker n'est pas pret, Babel ne peut pas etre genere de maniere verifiable. Le publish gate (S68) est le checkpoint que Babel doit passer.

**S69 Babel -> S70 RRV : DEPENDANCE FAIBLE.**
RRV "observe" Babel, mais ne necessite pas Babel pour fonctionner. RRV peut indexer Explorer + Ideas Hub + toute app deployee. Babel est un enrichissement du corpus, pas un prerequis.

Direction inverse : Babel ne necessite pas RRV pour exister. Babel est deployable et visible dans Browse sans recherche.

**S65 -> S67 Factory : OUI, nouvelle dependance.**
Le vocabulaire de confiance (S65) doit etre en place avant que Factory affiche des badges. D-HIDDEN-4 de Doc A s'applique encore plus tot dans Doc B.

**Ou est la Gouvernance ?**
Doc B ne place la gouvernance nulle part explicitement. C'est un GAP. CuratorVouched est mentionne dans la roadmap V2 comme S67, mais Doc B prend ce slot pour Factory. La gouvernance doit etre reinsere.

Options :
- **Option G-1 :** Gouvernance en S67, Factory repoussee a S68-S69. Conflit : retour a la sequence Doc A.
- **Option G-2 :** Gouvernance fusionnee dans S65 Phase E (scope bloat).
- **Option G-3 :** Gouvernance apres Factory, en S70 (apres le pilote). Viable si le pilote ferme n'a pas besoin de CuratorVouched.
- **Option G-4 :** Gouvernance dans un sprint dedie entre Factory et Babel (S68.5 = scope creep).
- **Option G-5 :** Gouvernance compactee en 2 phases integrees a S68 (broker + gouvernance). Le broker genere des apps ; la gouvernance signe les endorsements. Les deux touchent le feed. Synergie naturelle.

**Recommandation : Option G-5.** CuratorVouched est implementable en 2 phases (feed op + validation). Le broker S68 touche deja le feed (ReleasePublished). Fusionner les deux dans un S68 enrichi est faisable si le scope est borne : CuratorVouched/Disendorsed feed ops + validation, PAS l'UX multi-curator complete (reportee a S71-S72).

**Ou est le Proof Pack ?**
Doc A place le proof pack en S68. Doc B le remplace par le broker. Le proof pack est le livrable d'evaluation qui prouve la credibilite du projet.

Options :
- **Option PP-1 :** Proof Pack integre a S69 (Babel canari = le proof pack vivant).
- **Option PP-2 :** Proof Pack comme sprint dedie entre S69 et S70.
- **Option PP-3 :** Proof Pack fusionne dans S75 (pack produit defendable).
- **Option PP-4 :** Proof Pack allege integre a S68 (broker produit un evidence pack).

**Recommandation : Option PP-4 + PP-3.** Le broker S68 produit deja un `factory.provenance.json` et un `factory.audit.jsonl`. Enrichir avec un `proof-pack/` allege en S68. Le proof pack complet (SBOM, canary, feed snapshot, verify.sh) arrive en S75 (pack defendable). Cela distribue l'effort sur deux points au lieu d'un sprint dedie.

### 2.2 Graphe revise

```
S65 Contrat Public
  |
  v
S66 Durabilite
  |
  v
S67 Factory Foundation (templates, SBFB.json v2, module broker base)
  |
  |---> D-HIDDEN-4 : vocabulaire S65 utilise dans Factory
  |
  v
S68 Broker + Gouvernance CuratorVouched (preview, publish gate,
    diff, CuratorVouched/Disendorsed feed ops, evidence pack allege)
  |
  |---> Absorbe carry PROVENANCE-404-BRIDGE, COVERAGE-DEPLOY-E2E
  |
  v
S69 Babel Reader Canari + Pilote Ferme
  |
  |---> Absorbe carry VERIFY-LOCAL-KEY-ONLY, PLAYWRIGHT-SPECS-STALE
  |---> Gate 1 : go/no-go Arc 2
  |
  v
S70 RRV LocalOnly (FTS5-first, corpus reel Factory+Babel+Explorer+Ideas)
  |
  v
S71 Proof Cards (Babel, Factory, toutes apps)
  |
  v
S72 SearchManifest Opt-In + UX Gouvernance complete
  |
  |---> CuratorVouched (implemente en S68) enrichi avec UX multi-curator
  |---> Gate 2 : go/no-go Arc 3
  |
  v
S73 Factory Hardening + Templates additionnels
  |
  v
S74 Babel Translation Beta + Domain Packs
  |
  v
S75 Pack Produit Defendable (proof pack complet, evidence, release narrative)
```

### 2.3 Graphe ASCII avec dependances croisees

```
                S65 Contrat Public
                    |
                    v
                S66 Durabilite
                    |
                    v
            S67 Factory Foundation
               |           \
               |            \---> (vocabulaire S65 herite)
               v
    S68 Broker + Gouvernance (CuratorVouched feed ops)
               |           \
               |            \---> carry PROVENANCE-404, COVERAGE-E2E
               v
    S69 Babel Canari + Pilote Ferme
               |           \
               |            \---> carry VERIFY-LOCAL-KEY, PLAYWRIGHT
               |
          === GATE 1 ===
               |
               v
    S70 RRV LocalOnly (FTS5, corpus reel)
               |
               v
    S71 Proof Cards
               |
               v
    S72 SearchManifest + UX Gouvernance complete
               |
          === GATE 2 ===
               |
               v
    S73 Factory Hardening
               |
               v
    S74 Babel Translation Beta
               |
               v
    S75 Pack Produit Defendable
```

---

## 3. Carry items redistribution

### 3.1 Items dont le sprint cible change

| Item | Sprint Doc A | Sprint Doc B (pivot) | Sprint hybride propose | Raison du changement |
|------|-------------|---------------------|----------------------|---------------------|
| P2-PROVENANCE-404-BRIDGE | S68 proof pack | Non mentionne | **S68 Broker+Gouv** | Le broker integre le deploy roundtrip. Le proof pack allege dans le meme sprint couvre la distinction "projet inexistant" vs "pas de provenance". Sprint identique, contenu legèrement different. |
| P2-COVERAGE-DEPLOY-E2E | S68 proof pack | Non mentionne | **S68 Broker+Gouv** | Le broker S68 integre `deploy-from-repo`. Le test E2E deploy roundtrip EST le test du publish gate Factory. Meme sprint. |
| P2-VERIFY-LOCAL-KEY-ONLY | S69 pilote (Doc A) | Non mentionne | **S69 Babel+Pilote** | Toujours necessaire avant exposition externe. Le pilote est dans S69 dans les deux documents. Inchange. |
| P2-PLAYWRIGHT-SPECS-STALE | S65 suppression + S69 re-ecriture (Doc A) | Non mentionne | **S65 suppression + S69 re-ecriture** | Inchange. La suppression des 12 zombies est en S65 Phase D. La re-ecriture est en S69 Phase D (scenarios test guides). |

### 3.2 Items dont le sprint cible ne change PAS

| Item | Sprint | Raison |
|------|--------|--------|
| P2-FEED-INSERT-NO-AUTH-TIER (3/3) | **S65 Phase A MANDATORY** | Inchange. Prerequis absolu. |
| P2-VERIFY-ENTRY-VERSION-GUARD (1/3) | **S65 Phase A** | Inchange. 5 LOC, meme contexte. |
| P2-BADGE-WORDING-PREMATURE | **S65 Phase B** | Inchange. Coeur du sprint contrat public. |
| P2-COMMIT-TITLE-FORMAT (2/3) | **S65 Phase D** | Inchange. Dette pair. |
| P2-REVIEW-ORDER (2/3) | **S65 Phase D** | Inchange. Dette pair. |
| P2-PYTHON-BLOCK-EXEMPTION (2/3) | **S65 Phase D** | Inchange. Reclassifie resolved. |
| P2-EXPLORER-ESCAPE-SINGLE-QUOTE (2/3) | **S65 Phase D** | Inchange. 1 LOC. |
| P2-FEED-JOIN-HANDLE-LEAK (1/3) | **S66 Phase C** | Inchange. Shutdown lifecycle. |
| P2-ORPHAN-REPUBLISH-RECOVERY (1/3) | **S66 Phase C** | Inchange. Crash recovery. |

### 3.3 Items monitoring/hors scope — inchanges

| Item | Status | Sprint |
|------|--------|--------|
| P2-A-1 rand blocker | UPSTREAM | Monitoring continu |
| P2-AUDIT-2 iroh transitives | UPSTREAM | Decision point Gate 1 |
| P2-G-1 exe lock | INTERMITTENT | Monitoring continu |
| T-NN+2 iframe Rust-wasm | HORS SCOPE | Post-S75 |
| LT-2 Radicle | TRIGGER | Push tag v1.0 -> S66-S67 |
| LT-5 redundancy persistence | LATENT | Post-S75 sauf S69 |
| LT-7 quorum E2E | LATENT | Post-S75 sauf S69 |

### 3.4 Nouveaux items introduits par Doc B

| Item | Sprint | Description |
|------|--------|-------------|
| SBFB.json v2 node_id deprecation | **S67** | `node_id` optionnel/deprecie dans manifest. Requis pour templates portables. |
| feed raw-op migration (Option E) | **S65 ou S67** | Migration FeedEntry.op -> serde_json::Value. Selon feed_version_bump_strategy.md. |
| ReleasePublished auto-insertion | **S68** | deploy-from-repo doit creer automatiquement l'entree feed. Prerequis pour que Babel soit visible dans le feed. |

---

## 4. Chemin critique revise

### 4.1 Chemin critique Doc A

```
S65 -> S66 -> S69 -> S70 -> S71 -> S72
         \-> S67 -> S68 -> S69
```

Chemin le plus long : **S65 -> S66 -> S67 -> S68 -> S69 -> S70 -> S71 -> S72** = 8 sprints = ~16-18 semaines.

(Note : Doc A dit "S65 -> S66 -> S69 -> S70 -> S71 -> S72" mais S67/S68 sont aussi des prereqs de S69 via le proof pack.)

### 4.2 Chemin critique Hybride (pivot adopte)

```
S65 -> S66 -> S67 -> S68 -> S69 -> S70 -> S71 -> S72
```

Chemin le plus long : **8 sprints sequentiels de S65 a S72** = ~16-18 semaines.

Puis S73 -> S74 -> S75 = **3 sprints** = ~6 semaines.

**Total : 11 sprints sequentiels = ~22-24 semaines.**

### 4.3 Le chemin critique change-t-il ?

**NON fondamentalement.** Dans les deux cas, le chemin critique traverse S65 -> S66 -> ... -> S72. La difference est le contenu des sprints intermediaires, pas leur nombre.

Ce qui change :

| Aspect | Doc A | Hybride |
|--------|-------|---------|
| S67-S68 sur le chemin critique | Gouvernance + Proof Pack | Factory + Broker+Gouvernance |
| Premier dogfood reel | S75 (Babel) | S69 (Babel canari) |
| RRV a du contenu reel | Seulement Explorer + Ideas | Explorer + Ideas + Babel + Factory artifacts |
| Arc 3 (S73-S75) | Foundation Factory | Hardening + Babel beta + Pack defendable |

### 4.4 Y a-t-il des sprints qui peuvent etre parallelises ?

En regime solo-maintainer : **NON** pour la chaine principale.

Cependant, si un contributeur externe rejoint :

| Sprint principal | Sprint parallelisable | Condition |
|------------------|-----------------------|-----------|
| S70 RRV | S73 Factory Hardening | Aucune dependance |
| S71 Proof Cards | S74 Babel Translation | Faible dependance |
| S72 SearchManifest | S74 Babel Translation | Aucune dependance directe |

En pratique, le calendrier reste **~24 semaines** sauf contribution externe.

### 4.5 Le total est-il toujours 24 semaines ?

OUI. 11 sprints x ~2 semaines/sprint + marge Gate 1 (1 semaine) + marge Gate 2 (1 semaine) = ~24 semaines.

La difference n'est pas dans la duree mais dans le moment ou le dogfood arrive :
- Doc A : dogfood a S75 (semaine 22-24)
- Hybride : dogfood a S69 (semaine 9-11) puis approfondissement S74 (semaine 20-21)

---

## 5. Gates revisees

### 5.1 Gate 0.5 apres S66 — RECOMMANDEE

**Ajout propose :** Une gate legere apres S66 pour confirmer que la persistence est solide avant de construire Factory dessus.

| Critere | Go | No-Go |
|---------|-----|--------|
| 10 restarts consecutifs sans perte | Daemon survit | Perte de donnees |
| FsStore operationnel | Archives apps persistent | Blobs volatils |
| Feed republish au boot | Entries reinsertees | Orphans non recuperes |
| JoinHandle feed_join | Shutdown propre | Task abandonnee |

**Raison :** Dans Doc A, si S66 echoue, S67 (gouvernance) peut demarrer en mode degrade (ops feed sans persistence). Dans l'hybride, si S66 echoue, Factory (S67) est construite sur du sable — une app publiee par Factory disparait au restart. La gate est donc PLUS critique dans le pivot.

**Decision PO requise :** Formaliser ou rester en gate implicite (criteres de validation S66 existants).

### 5.2 Gate 1 apres S69 — ENRICHIE

La Gate 1 Doc A evaluait le pilote generique. Dans l'hybride, elle evalue aussi Factory + Babel canari.

| Critere | Go (Arc 2 demarre) | No-Go |
|---------|-----|--------|
| Installation | 2/3 testeurs installent sans aide | 0/3 reussit |
| Babel deploye | Babel visible dans Browse | Deploy echoue |
| Provenance | Babel provenance verifiable | Provenance cassee |
| Factory | Une app generee par Factory a deploye | Factory non fonctionnelle |
| Feed sync | Feed synchronise entre 2+ noeuds | Divergence |
| CuratorVouched | Au moins 1 endorsement signe dans le feed | Feed ops manquantes |
| Restart 24h | Daemon + Babel survivent 24h | Crash, perte donnees |

**Decision iroh 0.98 vs 1.0 :** Toujours evaluee a Gate 1. INCHANGEE.

**Si > 5 bugs P0/P1 :** Sprint fix dedie entre S69 et S70. Le sprint reserve (herite de l'ancienne roadmap) est consomme ici.

### 5.3 Gate 2 apres S72 — MODIFIEE

Doc A evaluait si RRV etait fonctionnel pour enrichir Factory. Dans l'hybride, Factory existe deja. La Gate 2 evalue si RRV + SearchManifest sont assez matures pour le hardening (S73) et Babel translation (S74).

| Critere | Go (Arc 3 demarre) | No-Go |
|---------|-----|--------|
| SearchManifest | Manifest sync stable 3 noeuds | Sync instable |
| RRV local | >= 100 entrees indexees, Babel trouvable | Index vide ou casse |
| Proof Cards | Score deterministe, Babel proof card visible | Score non reproductible |
| UX Gouvernance | Multi-curator visible, dissent affiche | Gouvernance invisible |
| Aucun bug P0 ouvert | 0 P0 | >= 1 P0 |

**Contingence :** Si RRV est insuffisant, S73-S75 demarrent quand meme. Factory hardening (S73) et Babel translation beta (S74) n'ont pas besoin de RRV. Le pack defendable S75 sera moins riche (pas de SearchManifest) mais toujours viable.

### 5.4 Pas de Gate 0.5 formelle si PO le refuse

Alternative : les criteres de validation S66 existants (section 2.4 de Doc A) servent de gate implicite. Le sprint S67 ne demarre que si S66 passe ses criteres. C'est le pattern actuel (gate audit entre sprints).

---

## 6. Calendrier revise

| Semaine | Sprint | Arc | Theme | Risque | Carry absorbes |
|---------|--------|-----|-------|--------|----------------|
| S1-S2 (mai-juin 2026) | S65 | 1 | Contrat Public + feed raw-op migration | 2/5 | FEED-INSERT-NO-AUTH-TIER, VERSION-GUARD, BADGE-WORDING, COMMIT-TITLE, REVIEW-ORDER, PYTHON-BLOCK, ESCAPE-QUOTE, PLAYWRIGHT-STALE (suppression) |
| S3-S4 | S66 | 1 | Durabilite (persistence + crash recovery) | 4/5 | FEED-JOIN-HANDLE-LEAK, ORPHAN-REPUBLISH |
| -- | **GATE 0.5** | -- | Persistence OK ? | -- | -- |
| S5-S6 | S67 | 1 | Factory Foundation (templates, SBFB.json v2, module base) | 3/5 | -- |
| S7-S8 | S68 | 1 | Broker + CuratorVouched feed ops + evidence pack allege | 3/5 | PROVENANCE-404-BRIDGE, COVERAGE-DEPLOY-E2E |
| S9-S11 | S69 | 1 | Babel Reader canari + Pilote Ferme | 5/5 | VERIFY-LOCAL-KEY-ONLY, PLAYWRIGHT-STALE (re-ecriture) |
| -- | **GATE 1** | -- | Go/no-go Arc 2 + decision iroh 0.98/1.0 | -- | -- |
| S12-S13 | S70 | 2 | RRV LocalOnly (FTS5-first, corpus reel) | 3/5 | -- |
| S14-S15 | S71 | 2 | Proof Cards (Babel, Factory, toutes apps) | 2/5 | -- |
| S16-S17 | S72 | 2 | SearchManifest opt-in + UX Gouvernance complete | 4/5 | -- |
| -- | **GATE 2** | -- | Go/no-go Arc 3 | -- | -- |
| S18-S19 | S73 | 3 | Factory Hardening + Templates additionnels | 2/5 | -- |
| S20-S21 | S74 | 3 | Babel Translation Beta + Domain Packs | 3/5 | -- |
| S22-S24 | S75 | 3 | Pack Produit Defendable (proof pack complet, evidence, narrative) | 3/5 | -- |

**Total : ~24 semaines = ~6 mois (mai 2026 -> novembre 2026).**

**Contingence : +2-4 semaines** (identique a Doc A) pour fixes pilote, iroh upgrade eventuel, gates echouees.

---

## 7. Risques de sequencage revises

### 7.1 Factory S67 sans gouvernance : les apps n'ont pas de curator endorsement

**Risque REEL mais GERABLE.**

Analyse : Dans la sequence hybride, Factory (S67) produit des apps avant que CuratorVouched (S68) n'existe. Une app generee par Factory en S67 est visible dans Browse avec un badge "Provenance" (defini en S65) mais sans endorsement curator.

**Impact :** L'app est auto-attestee. C'est coherent avec SLSA L1 ("le builder est aussi le publisher"). L'endorsement curator arrive 2 semaines plus tard (S68). Pour un reseau pre-launch sans tiers, c'est acceptable.

**Mitigation :** Le vocabulaire S65 dit deja "provenance auto-attestee". L'absence de CuratorVouched en S67 n'est pas une sur-promesse — c'est une etape.

### 7.2 Babel S69 + Pilote S69 : scope double ?

**Risque MOYEN.**

Analyse : Doc B fusionne Babel canari et pilote ferme dans le meme sprint. C'est ambitieux : generer Babel par Factory, le deployer, recruter 2-3 testeurs, faire tourner 24h, collecter le feedback, et decider go/no-go.

**Decomposition du risque :**
- Babel generation par Factory : mecanique, faible risque (Factory + domain pack sont prets depuis S67-S68)
- Deploy Babel : 1 commande `deploy-from-repo`, teste en E2E depuis S68
- Pilote operationnel : recrutement testeurs, infrastructure, feedback = effort operationnel, pas technique

**Mitigation :** S69 est deja prevu comme 5 phases + 2-3 semaines (le plus long sprint). Le scope Babel canari + pilote est tenable SI Babel est genere mecaniquement par Factory (pas code a la main).

**Plan B :** Si le scope est trop large, decouper : S69a = Babel canari (2 semaines), S69b = pilote (2 semaines). Le calendrier total passe de 24 a 26 semaines. C'est dans la marge de contingence.

### 7.3 Proof Pack absent de la sequence S67-S69 : quand le faire ?

**Risque FAIBLE.**

Analyse : Doc A dedie un sprint entier (S68) au proof pack. Dans l'hybride, le proof pack est distribue :
- S68 : evidence pack allege (`factory.provenance.json`, `factory.audit.jsonl`, checksums)
- S75 : proof pack complet (SBOM, canary, feed snapshot, verify.sh, attestation CI)

**Pourquoi c'est suffisant :**
- Pour le pilote S69, un evidence pack allege suffit. Les 2-3 testeurs n'ont pas besoin d'un `verify.sh` autonome.
- Pour la release publiquement defendable (S75), le proof pack complet est le livrable final. C'est son meilleur placement — apres que toutes les briques existent.

**Ce qui serait insuffisant :** Si le pilote etait PUBLIC, un proof pack complet serait requis AVANT. Mais le pilote est ferme (D-GEL-3). L'evidence pack allege suffit.

### 7.4 CuratorVouched reporte : impact sur SearchManifest S72 ?

**Risque NUL dans la sequence hybride.**

Analyse : CuratorVouched est en S68 (pas reporte, juste deplace de S67 a S68). SearchManifest est en S72. Il y a 4 sprints entre les deux. Le feed v2 bump (ou raw-op migration Option E) est fait avant S68. SearchManifest utilise le meme mecanisme.

La VRAIE question est : l'UX multi-curator complete (agregation, dissent visible, freshness) est reportee de S67 (Doc A) a S72 (hybride). C'est 5 sprints de retard pour l'UX gouvernance.

**Impact :** Pour le pilote ferme (S69), l'absence d'UX multi-curator complete est acceptable. Les 2-3 testeurs voient les endorsements dans le feed mais pas dans une UI agregee. C'est brut mais fonctionnel.

**Pour Gate 2 (apres S72) :** L'UX gouvernance complete est livree dans S72 meme. Les proof cards (S71) montrent deja les endorsements. SearchManifest (S72) les propage. L'UX finale arrive juste a temps.

### 7.5 Feed raw-op migration timing

**Risque MOYEN si fait en S67, FAIBLE si fait en S65.**

Analyse : La recherche `feed_version_bump_strategy.md` recommande Option E (FeedEntry.op = serde_json::Value). Cette migration doit etre faite AVANT le premier ajout d'operation (CuratorVouched en S68).

- Si en S65 : la migration est faite avant que quiconque ajoute des ops. Risque minimal.
- Si en S67 : la migration et Factory foundation sont dans le meme sprint. Risque de surcharge.

**Recommandation :** S65 Phase X (la recherche le dit explicitement). Cela ajoute ~150 LOC de refacto a S65 mais securise tout le reste de la roadmap.

### 7.6 FTS5-first vs Tantivy-first (Doc B vs Doc A)

**Risque FAIBLE.**

Doc B recommande FTS5-first : rustqlite est deja une dependance, FTS5 est mature, pas de nouvelle dep.

Doc A recommande Tantivy-first avec fallback FTS5 : BM25 + fuzzy + stemming 17 langues.

**Pour la sequence hybride :** FTS5-first est le choix pragmatique. RRV S70 a du contenu reel a indexer (Babel, Factory artifacts, Explorer, Ideas Hub). FTS5 couvre le cas nominal (recherche metadata). Tantivy reste un upgrade futur si le volume ou les features l'exigent.

**Decision gelee D-GEL-5 de Doc A reste valide :** Tantivy avec fallback FTS5. La sequence hybride inverse juste la preference initiale (FTS5-first, Tantivy-later).

---

## 8. Proposition de sequence hybride

### 8.1 Principes de la fusion

1. **Confiance exacte de Doc A** : S65 (contrat public, taxonomie, badges) est le socle. Aucun raccourci.
2. **Dogfood precoce de Doc B** : Factory en S67, Babel canari en S69. Le protocole est teste par une app reelle 13 semaines plus tot que dans Doc A.
3. **Gouvernance non sacrifiee** : CuratorVouched feed ops integrees a S68 (pas reporte a post-S72).
4. **Proof Pack distribue** : evidence pack allege S68, proof pack complet S75. Pas de sprint dedie isole.
5. **Pas de scope creep** : chaque sprint a 4-5 phases max. Babel canari est genere par Factory, pas code a la main.
6. **Gates claires** : Gate 0.5 (persistence), Gate 1 (pilote + Babel + Factory), Gate 2 (RRV + SearchManifest).

### 8.2 Sequence proposee

#### Arc 1 — Publiquement Defendable (S65-S69)

**S65 — Contrat Public** (2 semaines)
- Phase A : Securite feed (FEED-INSERT-NO-AUTH-TIER, VERIFY-ENTRY-VERSION-GUARD)
- Phase B : Taxonomie confiance + badges UI migration
- Phase C : Badge dynamique post-verification + feed raw-op migration (Option E)
- Phase D : Non-regression wording + dette pair (7 carry items)
- Gate S65 : Zero sur-promesse dans l'UI. Feed secure.

**S66 — Durabilite** (2 semaines)
- Phase A : iroh data_dir + iroh-docs persistence
- Phase B : iroh-blobs FsStore
- Phase C : Feed republish au boot + feed_join handle (2 carry items)
- Phase D : RevocationCache persistence + SQLite synchronous
- Phase E : Test E2E restart complet
- Gate 0.5 : 10 restarts sans perte. FsStore operationnel.

**S67 — Factory Foundation** (2 semaines)
- Phase A : SBFB.json v2 + validation + node_id deprecation
- Phase B : Module factory_broker base dans nexus-shell-daemon-core
- Phase C : Template engine + template `static-minimal`
- Phase D : CLI `sbfb create` + factory.template.lock + factory.provenance.json
- Gate S67 : `sbfb create` genere une app statique deployable.

**S68 — Broker Verifiable + Gouvernance Feed** (2 semaines)
- Phase A : Diff generation + review API + CuratorVouched/Disendorsed feed ops
- Phase B : Preview sandbox + publish gate (path traversal, secrets, bridge validation)
- Phase C : UI `/factory` (template selector, diff viewer, approve/reject)
- Phase D : Evidence pack allege + deploy roundtrip E2E + carry PROVENANCE-404 + COVERAGE-E2E
- Gate S68 : App Factory deploie -> Browse -> provenance verifiable. CuratorVouched insertable dans le feed.

**S69 — Babel Reader Canari + Pilote Ferme** (3 semaines)
- Phase A : Domain pack Babel + app generee par Factory (fixtures, source manifests)
- Phase B : Babel reader UI (liste textes, vue lecture, toggle langue, storage, identity)
- Phase C : Deploy Babel + invite mechanism + installeurs testes + carry VERIFY-LOCAL-KEY
- Phase D : Scenarios test guides + re-ecriture Playwright (carry PLAYWRIGHT-STALE)
- Phase E : Analyse go/no-go. Decision documentee.
- Gate 1 : 2/3 testeurs installent. Babel deploye, visible, provenance verifiable. Daemon 24h sans crash. Decision iroh 0.98/1.0.

#### Arc 2 — Intelligent et Verifiable (S70-S72)

**S70 — RRV LocalOnly** (2 semaines)
- Phase A : Index FTS5 local + API search
- Phase B : Indexation au boot + incrementale (Babel, Factory artifacts, Explorer, Ideas)
- Phase C : Bridge method `search` + citations
- Phase D : App sbfb-search MVP
- Gate S70 : Babel trouvable localement. < 50ms. Citations exactes.

**S71 — Proof Cards** (2 semaines)
- Phase A : ProofCard data model + computation deterministe
- Phase B : API + bridge proof_card_get
- Phase C : Integration search results + Browse
- Phase D : Tests adversariaux (spoofing, injection, stale, determinism)
- Gate S71 : Score deterministe. Projet sans provenance <= 50.

**S72 — SearchManifest + UX Gouvernance Complete** (2 semaines)
- Phase A : SearchManifest format + signing + feed op
- Phase B : Publication opt-in via iroh + gossip topic
- Phase C : Discovery + verification + cache + UX multi-curator (agregation, dissent, freshness)
- Phase D : Anti-spam + privacy analysis
- Gate 2 : Manifest sync 3 noeuds. Multi-curator visible dans Browse. Aucun P0.

#### Arc 3 — Productif et Defendable (S73-S75)

**S73 — Factory Hardening** (2 semaines)
- Phase A : Templates additionnels (static-storage, react-vite)
- Phase B : Template lock hash stable + verification BLAKE3
- Phase C : Deuxieme app simple (Repair Notebook ou similaire)
- Phase D : Migration v1/v2 + docs create/publish + erreurs UX propres
- Gate S73 : 2+ apps generees sans regression. Hash templates stable.

**S74 — Babel Translation Beta** (2 semaines)
- Phase A : Domain pack format formalise (spec docs/factory/DOMAIN_PACKS.md)
- Phase B : task_submit traduction mock ou worker local feature-gated
- Phase C : Resultat stocke + review utilisateur + provenance draft
- Phase D : Fallback fixtures officiel + documentation
- Gate S74 : Traduction mock fonctionne. Fallback fixtures OK si worker absent.

**S75 — Pack Produit Defendable** (2-3 semaines)
- Phase A : Proof pack complet (SBOM CycloneDX 1.6, cargo-deny, attestation CI)
- Phase B : Feed snapshot + canary refresh + verify.sh portable
- Phase C : CLI `sbfb proof-pack generate` + `sbfb proof-pack verify`
- Phase D : Release narrative + documentation publique
- Phase E : Decision go/no-go publication large
- Gate finale : Proof pack generable < 60s. Verifiable par bash+jq+sha256sum. Babel + Factory proof cards visibles. Release narrative coherente.

### 8.3 Comparaison des moments cles

| Moment | Doc A | Hybride | Gain |
|--------|-------|---------|------|
| Premier dogfood app reel | S75 (sem 22-24) | S69 (sem 9-11) | **-13 semaines** |
| Factory operationnelle | S73 (sem 18-19) | S67 (sem 5-6) | **-13 semaines** |
| CuratorVouched dans le feed | S67 (sem 5-6) | S68 (sem 7-8) | -2 semaines |
| UX gouvernance complete | S67 (sem 5-6) | S72 (sem 16-17) | -11 semaines |
| Proof pack complet | S68 (sem 7-8) | S75 (sem 22-24) | -15 semaines |
| Pilote ferme | S69 (sem 9-11) | S69 (sem 9-11) | 0 |
| RRV local | S70 (sem 12-13) | S70 (sem 12-13) | 0 |
| Calendrier total | ~24 sem | ~24 sem | 0 |

**Conclusion :** Le calendrier total ne change pas. Ce qui change est le moment ou le protocole est teste par une app reelle (13 semaines plus tot). Le prix est un retard de l'UX gouvernance complete (-11 semaines) et du proof pack complet (-15 semaines). Ces deux elements sont moins critiques en pre-launch ferme.

### 8.4 Decisions gelees revisees

Les decisions gelees de Doc A restent valides avec les ajustements suivants :

| Decision | Doc A | Hybride | Changement ? |
|----------|-------|---------|-------------|
| D-GEL-1 iroh 0.98 | Pour S65-S69 | Pour S65-S69 | NON |
| D-GEL-2 OS sandbox | Pas wasmtime | Pas wasmtime | NON |
| D-GEL-3 Pilote ferme | 2-3 personnes | 2-3 personnes | NON |
| D-GEL-4 Sequentiel | Arc 2 avant Arc 3 | Arc 2 avant Arc 3 | NON |
| D-GEL-5 Tantivy/FTS5 | Tantivy-first, fallback FTS5 | **FTS5-first, Tantivy-later** | OUI — inversion |
| D-GEL-6 Babel fixtures | MVPe fixtures | MVP fixtures | NON |
| D-GEL-7 Feed v2 batche | Bump v1->v2 en S67 | **Option E : pas de bump, raw-op** | OUI — elimination |
| D-GEL-8 Vocabulaire | Source verifiable | Source verifiable | NON |

Nouvelle decision gelee :

**D-GEL-9 : Factory avant RRV.** La Factory (S67-S68) precede RRV (S70-S72). Factory n'attend pas RRV. RRV beneficie de Factory (contenu reel a indexer).

**D-GEL-10 : Babel genere par Factory.** Babel n'est pas code a la main. Il est genere par `sbfb create --domain-pack babel` puis personnalise. La generation est un test de la Factory, pas un bypass.

### 8.5 Tests delta projete cumule (hybride)

| Sprint | Arc | Theme | Rust entree | Rust sortie | Vitest sortie | Total sortie |
|--------|-----|-------|-------------|-------------|---------------|-------------|
| S64 (base) | -- | -- | -- | 1326 | 265 | 1597 |
| S65 | 1 | Contrat Public + raw-op migration | 1326 | ~1342 | ~275 | ~1623 |
| S66 | 1 | Durabilite | ~1342 | ~1367 | ~275 | ~1648 |
| S67 | 1 | Factory Foundation | ~1367 | ~1382 | ~285 | ~1673 |
| S68 | 1 | Broker + CuratorVouched | ~1382 | ~1402 | ~293 | ~1701 |
| S69 | 1 | Babel + Pilote | ~1402 | ~1417 | ~313 | ~1736 |
| S70 | 2 | RRV LocalOnly | ~1417 | ~1435 | ~325 | ~1766 |
| S71 | 2 | Proof Cards | ~1435 | ~1443 | ~333 | ~1782 |
| S72 | 2 | SearchManifest + UX Gouv | ~1443 | ~1463 | ~341 | ~1810 |
| S73 | 3 | Factory Hardening | ~1463 | ~1478 | ~353 | ~1837 |
| S74 | 3 | Babel Translation Beta | ~1478 | ~1488 | ~358 | ~1852 |
| S75 | 3 | Pack Defendable | ~1488 | ~1498 | ~366 | ~1870 |

**Projection S75 : ~1870 tests totaux** (vs ~1863 Doc A, +7 net du a la raw-op migration tests supplementaires en S65).

---

## 9. Decisions ouvertes pour le PO

| # | Question | Recommandation | Impact si non tranche |
|---|----------|---------------|---------------------|
| 1 | Gate 0.5 formelle ou implicite ? | Formelle (criteres documentes) | Si S66 echoue, S67 Factory est construite sur du sable |
| 2 | FTS5-first confirme ? | OUI (Doc B est correct) | Tantivy ajoute une dep lourde pour un gain marginal pre-launch |
| 3 | Feed raw-op migration en S65 ou S67 ? | S65 Phase C (avant tout ajout d'op) | Si S67, le sprint est surcharge |
| 4 | Factory dans workspace nexus ou repo sibling ? | Workspace nexus pour MVP, extraction post-S75 | Si repo sibling maintenant, overhead CI/build |
| 5 | CuratorVouched feed op complet en S68 ou juste insertion ? | Insertion + validation en S68, UX complete en S72 | Si complet en S68, sprint surcharge |
| 6 | Proof Pack dedie ou distribue ? | Distribue (allege S68, complet S75) | Si dedie S68, Factory est repoussee |
| 7 | Babel + Pilote = 1 sprint ou 2 ? | 1 sprint de 3 semaines | Si 2 sprints, calendrier +2 semaines |
| 8 | `node_id` manifest : deprecation douce ou suppression ? | Deprecation douce (optionnel, warning) | Si suppression, apps existantes cassent |

---

## 10. Matrice de correspondance des carry items — verification croisee finale

| # | Carry Item | Doc A Sprint | Hybride Sprint | Changement | Raison |
|---|-----------|-------------|---------------|------------|--------|
| 1 | P2-FEED-INSERT-NO-AUTH-TIER (3/3) | S65 Ph.A | S65 Ph.A | NON | MANDATORY, identique |
| 2 | P2-VERIFY-ENTRY-VERSION-GUARD (1/3) | S65 Ph.A | S65 Ph.A | NON | Identique |
| 3 | P2-BADGE-WORDING-PREMATURE | S65 Ph.B | S65 Ph.B | NON | Coeur sprint, identique |
| 4 | P2-COMMIT-TITLE-FORMAT (2/3) | S65 Ph.D | S65 Ph.D | NON | Dette pair, identique |
| 5 | P2-REVIEW-ORDER (2/3) | S65 Ph.D | S65 Ph.D | NON | Dette pair, identique |
| 6 | P2-PYTHON-BLOCK-EXEMPTION (2/3) | S65 Ph.D | S65 Ph.D | NON | Reclassifie, identique |
| 7 | P2-EXPLORER-ESCAPE-SINGLE-QUOTE (2/3) | S65 Ph.D | S65 Ph.D | NON | 1 LOC, identique |
| 8 | P2-PLAYWRIGHT-SPECS-STALE pt1 | S65 Ph.D | S65 Ph.D | NON | Suppression zombies, identique |
| 9 | P2-FEED-JOIN-HANDLE-LEAK (1/3) | S66 Ph.C | S66 Ph.C | NON | Shutdown lifecycle, identique |
| 10 | P2-ORPHAN-REPUBLISH-RECOVERY (1/3) | S66 Ph.C | S66 Ph.C | NON | Crash recovery, identique |
| 11 | P2-PROVENANCE-404-BRIDGE (2/3) | S68 Ph.A | S68 Ph.D | **PHASE** | Meme sprint, phase differente (proof pack -> evidence pack) |
| 12 | P2-COVERAGE-DEPLOY-E2E (2/3) | S68 Ph.A | S68 Ph.D | **PHASE** | Meme sprint, phase differente |
| 13 | P2-VERIFY-LOCAL-KEY-ONLY (2/3) | S69 Ph.A | S69 Ph.C | **PHASE** | Meme sprint, phase differente (invite -> deploy) |
| 14 | P2-PLAYWRIGHT-SPECS-STALE pt2 | S69 Ph.D | S69 Ph.D | NON | Re-ecriture, identique |
| 15 | P2-A-1 rand blocker | Monitoring | Monitoring | NON | Upstream |
| 16 | P2-AUDIT-2 iroh transitives | Gate 1 | Gate 1 | NON | Decision point |
| 17 | P2-G-1 exe lock | Monitoring | Monitoring | NON | Intermittent |
| 18 | T-NN+2 iframe Rust-wasm | Post-S75 | Post-S75 | NON | Hors scope |
| 19 | LT-2 Radicle | S66-S67 | S66-S67 | NON | Trigger tag push |
| 20 | LT-5 redundancy persistence | Post-S75 | Post-S75 | NON | Latent |

**Conclusion :** Aucun carry item ne change de sprint. 3 items changent de phase a l'interieur du meme sprint. La redistribution est transparente.

---

## 11. Synthese

### Ce que le pivot apporte

1. **Dogfood 13 semaines plus tot.** Babel canari en S69 au lieu de S75. Le protocole est teste par une app reelle avant l'arc 2 (RRV), ce qui informe le design RRV avec des donnees reelles.

2. **Factory prouvee, pas theorique.** Factory genere une app reelle (Babel) des S69. Les bugs Factory sont decouverts AVANT de promettre un ecosysteme d'apps.

3. **RRV a du contenu reel a indexer.** Quand RRV demarre en S70, il indexe Explorer + Ideas Hub + Babel + Factory artifacts. Pas un index vide.

4. **Proof Pack au bon moment.** Le proof pack complet arrive en S75 (pack defendable) quand TOUTES les briques existent. Un proof pack en S68 (Doc A) serait incomplet (pas de Babel, pas de RRV, pas de SearchManifest).

### Ce que le pivot coute

1. **UX gouvernance retardee.** L'UX multi-curator complete (agregation, dissent, freshness) passe de S67 a S72 (-11 semaines). En pilote ferme, c'est acceptable. En release publique, ce serait un probleme.

2. **CuratorVouched feed ops sans UX.** Les ops sont dans le feed (S68) mais l'UI ne les montre pas completement avant S72. Les testeurs du pilote voient les endorsements bruts.

3. **Proof Pack complet retarde.** De S68 a S75 (-15 semaines). Compense par l'evidence pack allege en S68 qui suffit pour le pilote ferme.

4. **Complexite S68 accrue.** S68 fusionne broker + CuratorVouched feed ops + evidence pack. C'est le sprint le plus charge de l'arc 1. Risque de surcharge si mal borne.

### Verdict

**Le pivot est net-positif.** Le gain de dogfood precoce (-13 semaines) depasse largement le cout de l'UX gouvernance retardee. La sequence hybride ne change pas le calendrier total (24 semaines) mais change radicalement le moment ou le protocole est valide par l'usage.

**La sequence hybride est celle qui presente le meilleur ratio risque/valeur pour un projet solo-maintainer pre-launch.**

---

*Document d'analyse. La decision finale revient au PO. Ce document sera reference dans le kickoff du sprint qui l'adopte.*
