# Analyse de Repositionnement — Gouvernance De Confiance

**Date :** 2026-05-18
**Confiance globale :** HIGH (code source lu exhaustivement, recherches
anterieures croisees, dependances verifiees)
**Declencheur :** Pivot PO S67 Factory Foundation vs Gouvernance De
Confiance — ou placer la gouvernance si Factory prend la place de S67 ?

---

## Table des matieres

1. [Classification des livrables Gouvernance](#1-classification-des-livrables-gouvernance)
2. [Analyse des 5 options de placement](#2-analyse-des-5-options-de-placement)
3. [Impact de la strategie raw-op sur chaque option](#3-impact-de-la-strategie-raw-op)
4. [Impact sur le pilote S69](#4-impact-sur-le-pilote-s69)
5. [Impact sur les apps Factory](#5-impact-sur-les-apps-factory)
6. [Recommandation finale](#6-recommandation-finale)

---

## 1. Classification des livrables Gouvernance

La recherche `s67_gouvernance_confiance_research.md` definit 5 phases
(A-E) pour un sprint Gouvernance complet. Voici la classification de
chaque livrable en fonction de son urgence par rapport au pilote S69.

### 1.1 ESSENTIEL pre-pilote (doit etre fait avant S69)

| Livrable | Raison | Complexite | Sprint naturel |
|----------|--------|------------|----------------|
| **Feed raw-op migration** (`FeedEntry.op` -> `serde_json::Value`) | Pre-requis technique absolu. Sans cette migration, tout ajout d'operation au feed exige un version bump. Le pilote S69 ne pourra pas recevoir de futures ops sans casser les noeuds existants. | ~150 LOC refacto | S65 Phase A |
| **`verify_entry()` version guard** | Carry P2-VERIFY-ENTRY-VERSION-GUARD. Sans ce guard, un attaquant peut injecter des entries avec `version: 99` acceptees sans erreur. | 5 LOC | S65 Phase A |
| **`feed_insert` auth tier** | Carry P2-FEED-INSERT-NO-AUTH-TIER (3/3 MANDATORY). N'importe quel process avec le bearer token peut injecter dans le feed. | ~30-50 LOC | S65 Phase A |

**Verdict :** Ces 3 items sont des prerequis de securite/architecture du
feed, pas de la gouvernance au sens produit. Ils doivent etre dans S65
quel que soit le placement de la gouvernance.

### 1.2 SOUHAITABLE pre-pilote (ameliore le pilote mais pas bloquant)

| Livrable | Raison | Complexite | Sprint naturel |
|----------|--------|------------|----------------|
| **`CuratorVouched` feed op** (enum variant + validation) | Permet aux curators de signer un endorsement horodate et scopee dans le feed. Sans ca, les endorsements restent implicites (presence dans la liste = approbation). Le pilote fonctionne sans, mais la confiance est binaire. | ~80-100 LOC Rust + 8-10 tests | S67 ou S68 |
| **`CuratorDisendorsed` feed op** (enum variant + validation) | Permet le dissent public. Sans ca, un curator ne peut que retirer silencieusement un projet de sa liste. Le pilote fonctionne sans, mais le modele de confiance est unidirectionnel. | ~60-80 LOC Rust + 6-8 tests | S67 ou S68 |
| **Browse multi-curator aggregation basique** | Actuellement, si 2 curators vouche pour le meme projet, on voit 2 entrees distinctes. L'aggregation afficherait "Approuve par 2 curators" sur une seule entree. Le pilote est fonctionnel sans, mais l'UX est confuse. | ~100-150 LOC dans `browse.rs` | S67 ou S70 |
| **Freshness curator** ("derniere mise a jour il y a X jours") | Indicateur simple base sur `CuratorList.created_at` vs `now()`. Le pilote fonctionne sans, mais les testeurs ne savent pas si une liste curator est fraiche ou perimee. | ~30-50 LOC frontend | S67 ou S68 |

**Verdict :** Ces items renforcent la credibilite du pilote. Un pilote
sans CuratorVouched fonctionne (les curators vouche en incluant des
projets dans leur liste, comme aujourd'hui), mais le pilote avec
CuratorVouched est plus convaincant car les endorsements sont
horodates, scopees, et signes dans le feed.

### 1.3 POST-PILOTE (peut attendre apres S69)

| Livrable | Raison | Complexite | Sprint naturel |
|----------|--------|------------|----------------|
| **Multi-curator scope** (`CuratorScope` enum : Security, Quality, License) | Le scope est un raffinement de l'endorsement. Il n'est pas necessaire pour prouver que la gouvernance fonctionne. Les testeurs du pilote seront 2-3 personnes, pas un ecosysteme avec des specialistes. | ~40-60 LOC + UI | S70+ |
| **Trust overlay agrege** (badges "3 curators (1x securite, 1x general)") | Depend du multi-curator scope. Inutile avec 2-3 testeurs. | ~80-120 LOC + UI | S70+ |
| **Dissent visible UI** (badge "Avis partage" avec detail) | Sophistication UI. Le pilote ne genere pas assez de dissent pour justifier le UI. Le CuratorDisendorsed dans le feed suffit ; la surface UI peut venir apres. | ~100-150 LOC frontend | S70+ |
| **Stale detection automatique** (timer re-verification des repos) | Necessite un cron/timer dans le coordinator. Le pilote est trop court (2-3 semaines) pour que la staleness soit un probleme. | ~80-120 LOC Rust | S70+ |
| **`SourceRecovered` feed op** | Complement de `SourceBecameStale`. Ne sert que si la stale detection est active. | ~40-60 LOC | S70+ |
| **RevocationCache persistence SQLite** (carry S26) | Le carry est ancien mais le pilote ferme est sur un reseau si petit que les restarts ne perdent pas de revocations significatives. | ~60-80 LOC | S70+ |
| **Tests adversariaux gouvernance** (5 scenarios : curator malveillant, split-brain, stale replay, forgery, flood) | Patterns bien etablis par S64. Peuvent etre ajoutes a n'importe quel sprint post-gouvernance. | ~80-120 LOC | Sprint suivant le code gouvernance |

**Verdict :** Tout ce qui concerne le multi-curator, le scope, le
dissent UI, et la detection automatique est de la sophistication qui
n'ajoute pas de valeur au pilote ferme avec 2-3 testeurs.

---

## 2. Analyse des 5 options de placement

### 2.1 Option A — Gouvernance complete avant Factory (garder l'ordre original)

**Sequence :**
```
S65 Contrat Public -> S66 Durabilite -> S67 GOUVERNANCE -> S68 Proof Pack
-> S69 Pilote -> S70+ Factory ...
```

**Ce que le pilote S69 voit :**
- CuratorVouched + CuratorDisendorsed dans le feed : OUI
- Multi-curator scope + trust overlay : OUI
- Dissent visible : OUI
- Freshness : OUI
- Stale detection auto : OUI
- Gouvernance complete : OUI

**Avantages :**
1. Sequence logique : le vocabulaire de confiance (S65) nourrit la
   gouvernance (S67) qui nourrit le proof pack (S68) qui nourrit le
   pilote (S69). Chaque sprint construit sur le precedent.
2. Le pilote S69 a le meilleur ensemble de fonctionnalites possibles
   pour tester la confiance.
3. Le bump feed (si on reste en enum ferme) ou l'ajout d'ops (si on
   passe en raw-op) est fait avant le pilote, donc les noeuds pilotes
   ont un feed complet.

**Inconvenients :**
1. **Factory reste theorique trop longtemps.** C'est le probleme PO
   central. Factory est la raison d'etre de la plateforme ("n'importe
   qui publie une app"), et elle est repoussee a S73+ soit ~16-20
   semaines apres S65.
2. RRV n'a pas d'objet reel a indexer (pas d'app Factory generee).
3. Babel reste un concept, pas un artefact.

**Risque :** La gouvernance S67 complete prend 4-5 phases. Avec les
stale detection, les tests adversariaux, et la UI dissent, c'est un
sprint technique lourd (3/5 risque). Si des imprevus surgissent, le
pilote S69 est retarde.

**Verdict : SOUS-OPTIMAL.** Le pilote S69 serait excellent, mais le
cout d'opportunite (Factory retardee de 4+ mois) est trop eleve.
La gouvernance complete pour un pilote ferme a 2-3 personnes est du
sur-engineering.

### 2.2 Option B — Gouvernance minimale en S65, full en S70

**Sequence :**
```
S65 (raw-op + version guard + auth tier) -> S66 Durabilite
-> S67 FACTORY FOUNDATION -> S68 Broker/Preview/Publish
-> S69 Pilote (Babel canari) -> S70 GOUVERNANCE COMPLETE -> ...
```

**Ce que le pilote S69 voit :**
- CuratorVouched dans le feed : NON (pas implemente)
- Multi-curator scope : NON
- Dissent visible : NON
- Freshness : NON (mais derivable du timestamp CuratorList)
- Stale detection auto : NON
- Gouvernance visible : AUCUNE au-dela du systeme curator actuel
  (subscribe, liste, presence = vouch)

**Avantages :**
1. Factory est livree en S67-S68, soit ~8 semaines apres S65. Le
   probleme PO est resolu.
2. Babel canari en S69 force Factory a prouver son utilite.
3. La gouvernance en S70 est informee par le feedback du pilote
   ("les testeurs veulent-ils vraiment du multi-curator scope, ou
   est-ce que le vouch implicite suffit ?").
4. L'architecture du feed est prete (raw-op en S65) donc ajouter
   CuratorVouched en S70 est non-breaking.

**Inconvenients :**
1. **Le pilote S69 n'a AUCUNE gouvernance visible au-dela de
   l'existant.** Les testeurs voient la meme chose qu'aujourd'hui :
   un curator qui liste des projets, point. Pas d'endorsement signe,
   pas de dissent, pas de freshness.
2. Le proof pack S68 ne peut pas inclure d'endorsements signes
   (CuratorVouched n'existe pas encore).
3. Si les testeurs du pilote jugent la confiance insuffisante, le
   sprint S70 Gouvernance est un "rattrapage" reactif au lieu d'etre
   une construction proactive.

**Risque :** Le pilote S69 manque de credibilite sur l'axe
gouvernance. Les testeurs voient Factory + Babel (utile) mais le
systeme de confiance est rudimentaire. Pour un pilote ferme entre
amis, c'est acceptable. Pour une demo publique, non.

**Verdict : VIABLE mais le pilote est nu cote confiance.**

### 2.3 Option C — Gouvernance split en quatre sprints

**Sequence :**
```
S65 Phase A : feed raw-op + version guard + auth tier
S67 Phase A : CuratorVouched minimal (juste l'operation feed)
S68 : Factory + broker + CuratorVouched UI combines
S70 : Multi-curator, dissent, stale detection
```

**Ce que le pilote S69 voit :**
- CuratorVouched dans le feed : OUI (depuis S67)
- CuratorVouched dans le UI : OUI (depuis S68)
- Multi-curator scope : NON
- Dissent visible (CuratorDisendorsed) : NON (ou minimal si S67 l'inclut)
- Freshness : PARTIELLE (timestamp du CuratorVouched)
- Stale detection : NON

**Avantages :**
1. Les blocs techniques sont places au "bon moment" : la plomberie
   feed (S65), l'operation feed (S67 Phase A), la surface UI (S68),
   la sophistication (S70).
2. Factory (S67-S68) et gouvernance minimale coexistent dans les
   memes sprints. Le proof pack S68 peut inclure des CuratorVouched.
3. Le pilote voit des endorsements signes et horodates.

**Inconvenients :**
1. **Fragmentation.** La gouvernance est repartie sur 4 sprints (S65,
   S67, S68, S70). Chaque sprint a un contexte different. Le risque
   de bugs d'integration entre les morceaux est reel.
2. **S68 devient lourd.** Factory broker + preview + publish gate +
   CuratorVouched UI + proof pack = 6+ phases potentielles. C'est le
   sprint le plus charge de la roadmap.
3. **Charge cognitive.** Chaque sprint fait un peu de gouvernance et
   un peu d'autre chose. Le narratif du sprint est confus.

**Risque :** S68 est un megatron. Si S67 livre CuratorVouched en Phase
A mais que S68 ne peut pas absorber le UI a cause de la charge Factory,
le CuratorVouched est un bout de code mort pendant tout le pilote.

**Verdict : TROP FRAGMENTE.** La gouvernance n'est pas un sujet qu'on
saupoudre en Phase A de 4 sprints differents. C'est soit un sujet
concentre, soit de la dette diffuse.

### 2.4 Option D — Gouvernance en S73 (dans le hardening)

**Sequence :**
```
S65 -> S66 -> S67 Factory -> S68 Broker -> S69 Pilote
-> S70 RRV -> ... -> S73 GOUVERNANCE + hardening Factory
```

**Ce que le pilote S69 voit :**
- CuratorVouched : NON
- Multi-curator : NON
- Dissent : NON
- Freshness : NON
- Gouvernance visible : AUCUNE

**Avantages :**
1. Factory/Babel sont le focus exclusif de S67-S69. Pas de distraction.
2. La gouvernance en S73 est informee par le pilote ET par l'usage
   reel de Factory (quelles apps ont besoin d'endorsement ?).
3. Simple a planifier : un bloc = un sujet.

**Inconvenients :**
1. **Le pilote S69 n'a AUCUNE gouvernance.** Pire que l'option B
   (meme constat) mais plus tard (S73 au lieu de S70).
2. **S73 est tres loin.** ~16-20 semaines apres S65. Les decisions
   de design gouvernance prises maintenant (CuratorVouched payload,
   scope) devront etre revisitees car le contexte aura change.
3. **Le proof pack S68 est orphelin de gouvernance.** Comment prouver
   la confiance dans une app sans endorsement signe ?

**Risque :** Le pilote devalue la credibilite du protocole. Les
testeurs demandent "comment je sais que cette app est fiable ?" et
la reponse est "un curator l'a mise dans sa liste". C'est le systeme
actuel, pas une avancee.

**Verdict : TROP TARD.** La gouvernance est le coeur de la proposition
de valeur SBFB. La repousser a S73 c'est accepter 4 mois sans
progression sur l'axe confiance.

### 2.5 Option E — Feed raw-op en S65, CuratorVouched en S68 avec le proof pack

**Sequence :**
```
S65 Phase A : feed raw-op + version guard + auth tier
S66 : Durabilite
S67 : Factory Foundation
S68 : Proof Pack + CuratorVouched (endorsements signes = partie du proof)
S69 : Pilote (Babel canari avec endorsements)
S70 : Multi-curator UI + dissent + stale
```

**Ce que le pilote S69 voit :**
- CuratorVouched dans le feed : OUI (depuis S68)
- CuratorVouched dans le UI Browse : OUI (badge "Endorse par curator X")
- CuratorDisendorsed : OUI (si S68 l'inclut, sinon S70)
- Multi-curator scope : NON
- Dissent visible : MINIMAL (le CuratorDisendorsed est dans le feed,
  un badge simple peut l'afficher)
- Freshness : OUI (timestamp du CuratorVouched entry)
- Stale detection : NON

**Avantages :**
1. **Coherence produit.** Le proof pack S68 inclut naturellement les
   endorsements signes. Un "proof pack" qui ne montre pas qui
   approuve l'app est incomplet. CuratorVouched est une evidence de
   confiance.
2. **Factory en S67.** Le probleme PO est resolu.
3. **Le pilote a de la gouvernance visible.** Les testeurs voient des
   endorsements horodates, pas juste une liste implicite.
4. **S68 est naturellement le bon sprint.** Le proof pack assemble :
   provenance, feed entry, deploy E2E, **et** endorsement signe.
   C'est un sprint d'assemblage de preuves — l'endorsement en fait
   partie.
5. **Feed raw-op en S65** rend l'ajout de CuratorVouched en S68
   non-breaking. Pas de version bump. Pas de coordination de
   mise a jour des noeuds.
6. **S70 Gouvernance complete** est informe par le pilote. Le
   multi-curator scope, le dissent UI, et la stale detection sont
   designes avec le feedback reel des testeurs.

**Inconvenients :**
1. **S68 est charge.** Proof pack + CuratorVouched + deploy E2E.
   C'est 5 phases minimum.
2. **Le CuratorVouched n'a pas de sprint dedie.** Il est "fondu"
   dans le proof pack. Le risque est que le code gouvernance soit
   traite comme secondaire par rapport au proof pack.

**Mitigation du poids de S68 :**
- Le CuratorVouched est ~80-100 LOC Rust (ajout de 2 variants a
  l'enum, validation, 8-10 tests). C'est une Phase A naturelle.
- Le proof pack est un assemblage de briques existantes (provenance,
  feed, deploy). C'est des Phase B-D.
- Total : 5 phases (A-E), dans la norme d'un sprint SBFB.

**Risque :** S68 deborde si le proof pack est plus complexe que prevu.
Mitigation : CuratorVouched est en Phase A, donc meme si le proof pack
derape, le CuratorVouched est livre.

**Verdict : OPTIMAL.** C'est l'option qui maximise la valeur du pilote
tout en preservant Factory en S67.

---

## 3. Impact de la strategie raw-op

### 3.1 Recap de la strategie raw-op (Option E du feed bump research)

La migration `FeedEntry.op : PublicFeedOperation` -> `FeedEntry.op :
serde_json::Value` rend l'ajout de nouvelles operations feed non-breaking.
Les noeuds anciens stockent, verifient (hash + signature), et propagent
des operations inconnues sans les interpreter.

### 3.2 Impact sur chaque option de placement

| Option | Avec enum ferme (actuel) | Avec raw-op (propose) |
|--------|--------------------------|----------------------|
| **A — Gouvernance S67** | Bump `FEED_FORMAT_VERSION` v1->v2 en S67 avant pilote. Les noeuds pilotes sont tous v2. Pas de probleme de compat (pas de noeuds v1 externes). | Pas de bump. CuratorVouched ajoute au enum. Meme resultat, moins de ceremonie. |
| **B — Gouvernance S70** | CuratorVouched ajoute en S70. Si des noeuds pilotes S69 existent encore en v1, ils ne peuvent pas lire les entries CuratorVouched. Le feed a des trous. | Pas de bump. Les noeuds pilotes (S69) recoivent les entries CuratorVouched, les stockent et propagent sans les comprendre. Pas de trous dans le feed. **Gain net.** |
| **C — Split 4 sprints** | Bump v2 en S67, CuratorDisendorsed ajoute apres le bump. Si le Disendorsed est en S70, il faut un nouveau bump v3 ou il est inclus dans le bump initial (freeze premature). | Pas de bump. Chaque ajout est non-breaking. **L'option C devient techniquement triviale** (mais reste fragmente conceptuellement). |
| **D — Gouvernance S73** | Bump v2 en S73. Les noeuds pilotes fonctionnent longtemps sans gouvernance feed. | Meme resultat mais sans bump. La gouvernance est juste "ajout de variants". |
| **E — CuratorVouched S68** | Bump v2 en S68 (ou en S67 si on anticipe). Les noeuds S69 pilotes sont v2. | **Pas de bump. CuratorVouched ajoute en S68 comme simple variant.** Le pilote S69 lit et affiche les endorsements. Les noeuds futurs (S70+) ajoutent d'autres ops sans rien casser. |

### 3.3 Conclusion raw-op

**La strategie raw-op rend le timing de la gouvernance independant du
versioning.** C'est l'avantage strategique majeur : on peut placer
CuratorVouched au sprint qui a le plus de sens produit (S68 proof pack)
sans subir de contrainte technique de version.

Sans raw-op, le placement de CuratorVouched est contraint par "quand
fait-on le bump ?" et "quels types batcher dans le bump ?". Avec
raw-op, la question est purement produit : "quand CuratorVouched
apporte-t-il le plus de valeur ?".

---

## 4. Impact sur le pilote S69

### 4.1 Matrice experience testeur par option

| Critere | A (Gouv S67) | B (Gouv S70) | C (Split) | D (Gouv S73) | E (Vouched S68) |
|---------|:---:|:---:|:---:|:---:|:---:|
| Endorsements signes dans le feed | OUI | NON | OUI (partiel) | NON | OUI |
| Endorsements visibles dans Browse | OUI (complet) | NON | OUI (si S68 UI fait) | NON | OUI (badge) |
| Dissent visible | OUI | NON | NON | NON | MINIMAL (badge) |
| Freshness endorsement | OUI | NON | OUI | NON | OUI |
| Multi-curator scope | OUI | NON | NON | NON | NON |
| Stale detection auto | OUI | NON | NON | NON | NON |
| **Score confiance pilote** | **5/5** | **1/5** | **3/5** | **1/5** | **4/5** |
| **Factory livree** | NON | OUI | OUI | OUI | OUI |
| **Babel canari** | NON | OUI | OUI | OUI | OUI |
| **Score valeur produit** | **2/5** | **4/5** | **3/5** | **4/5** | **5/5** |

### 4.2 Que voient concretement les testeurs du pilote ?

**Option A (Gouvernance S67) :**
Les testeurs voient un systeme de confiance complet mais pas d'app
generee par Factory. Ils testent le protocole feed + gouvernance mais
pas la proposition de valeur "n'importe qui publie une app". Le pilote
est une demo technique, pas une demo produit.

**Option B / D (Gouvernance post-pilote) :**
Les testeurs voient Babel Reader genere par Factory, deploy verifie,
provenance visible, mais la confiance est le systeme actuel :
- Un curator X a liste ce projet dans sa CuratorList.
- Badge "Verifie" (provenance_hash existe).
- Aucune information sur quand le curator a endorse, pourquoi, ou
  si un autre curator objecte.

C'est suffisant pour un pilote ferme entre amis (la confiance est
interpersonnelle, pas protocolaire). C'est insuffisant pour une demo
publique.

**Option E (CuratorVouched S68) :**
Les testeurs voient :
- Babel Reader genere par Factory.
- Deploy verifie avec provenance SLSA L1.
- Public feed avec ReleasePublished + CuratorVouched.
- Badge "Endorse par curator X le [date]" dans Browse.
- Proof pack montrant provenance + endorsement signe.
- Si CuratorDisendorsed est inclus : badge "Objecte par curator Y"
  (dissent minimal).

C'est un pilote qui montre les deux axes : la valeur produit (Factory
cree une app) et la confiance protocolaire (les endorsements sont signes,
horodates, et verifiables dans le feed).

### 4.3 Credibilite du pilote sans gouvernance

**Question : le pilote S69 est-il credible sans gouvernance visible ?**

**Reponse : OUI pour un pilote ferme, NON pour une demo publique.**

Le pilote S69 est un pilote ferme entre 2-3 amis (cf. cross-cutting
research : "le pilote doit etre ferme pour limiter l'exposition"). Pour
ces testeurs, la confiance est personnelle — ils connaissent le
developpeur, ils savent que les apps sont legit. La gouvernance
protocolaire n'est pas le facteur limitant de leur confiance.

MAIS : si le pilote est utilise comme vitrine pour NLnet/Newby/Kahle
(cf. memory `babel_post_v1_app.md`), les endorsements signes dans le
feed ajoutent un element de preuve tangible. Un observateur externe
peut verifier que "curator X a endorse cette app a la date T, voici
la signature Ed25519".

**Conclusion :** CuratorVouched dans le proof pack (option E) est la
difference entre "regardez notre app" et "regardez notre app avec son
chainon de preuves d'approbation". Pour un pilote ferme c'est optionnel ;
pour du signaling externe c'est un multiplicateur de credibilite.

---

## 5. Impact sur les apps Factory

### 5.1 Sans gouvernance (Options B/D)

Une app generee par Factory en S67-S68 est deploye avec :
- `ProvenanceRecord` signe (SLSA L1)
- `factory.provenance.json` (provenance Factory locale)
- `ReleasePublished` dans le feed
- Presence dans la CuratorList du deployeur (vouch implicite)

Pas de badge d'endorsement signe. Pas de CuratorVouched. La confiance
est la meme que pour n'importe quelle app deploye avant S67 : le
deployeur la declare comme sienne, un curator la liste ou non.

### 5.2 Avec CuratorVouched en S68 (Option E)

La meme app Factory a en plus :
- `CuratorVouched` entry dans le feed, signee par un curator
- Badge "Endorse par [curator] le [date]" dans Browse
- Le proof pack inclut l'endorsement comme evidence supplementaire

**Difference concrte pour le testeur :**
Quand un testeur ouvre Babel Reader dans Browse, il voit :
- Badge "Provenance" (hash + signature)
- Badge "Endorse" (curator X a signe un vouch le 2026-07-01)
- Source visible (lien vers le repo Git)
- Si le testeur clique "Details de verification", il voit la timeline :
  ReleasePublished -> CuratorVouched -> (eventuellement CuratorDisendorsed)

Sans CuratorVouched, le meme testeur voit :
- Badge "Provenance"
- Source visible
- Pas de notion d'approbation au-dela de "c'est dans la liste du curator"

### 5.3 Multi-curator pour les apps Factory

Les apps Factory n'ont PAS besoin de multi-curator pour le pilote.
Le multi-curator scope est utile quand un ecosysteme a 10+ curators
avec des specialites differentes. Avec 2-3 testeurs, un seul curator
suffit. Le multi-curator scope est clairement post-pilote (S70+).

---

## 6. Recommandation finale

### 6.1 Sequence recommandee : Option E enrichie

```
S65  Contrat Public
     Phase A : feed raw-op migration + version guard + auth tier
     Phase B : taxonomie de confiance (badges UI)
     Phase C : wording corrections + spec §9 update
     Phase D : dette pair carry items

S66  Durabilite
     (inchange — blob persistence, restart E2E, republish)

S67  Factory Foundation
     (inchange — module/broker, template, SBFB.json v2,
      factory.provenance.json, sprint skeleton)

S68  Proof Pack + CuratorVouched
     Phase A : CuratorVouched + CuratorDisendorsed dans le feed
               (enum variants + validation + 8-10 tests)
     Phase B : Browse aggregation — badges endorsement dans Browse
               (frontend consomme les CuratorVouched du feed)
     Phase C : Deploy E2E roundtrip test + feed ReleasePublished
               auto-insert (carry P2-COVERAGE-DEPLOY-E2E)
     Phase D : Proof pack assemblage (provenance + feed + endorsement
               + evidence pack)
     Phase E : Tests adversariaux CuratorVouched
               (forgery, flood, stale replay — 5-8 tests)

S69  Pilote Ferme (Babel canari)
     (inchange — domain pack, Babel Reader, fixtures, storage,
      pilote 2-3 personnes)

S70  Gouvernance Complete
     Phase A : Multi-curator aggregation dans BrowseAggregator
               (dedup par project_id, scope breakdown)
     Phase B : Freshness curator + stale detection timer
     Phase C : Dissent UI (badge "Avis partage", detail dialog)
     Phase D : RevocationCache persistence SQLite (carry S26)
     Phase E : Tests adversariaux gouvernance complete
```

### 6.2 Justification

**Pourquoi pas l'Option A (Gouvernance complete en S67) :**
Repousse Factory de 4+ mois. Le probleme PO est reel : la
proposition de valeur SBFB est "n'importe qui publie une app",
pas "regardez notre systeme de confiance". La gouvernance complete
pour un pilote a 2-3 testeurs est du sur-engineering.

**Pourquoi pas l'Option B (Gouvernance en S70 sans rien avant) :**
Le pilote S69 est nu cote confiance. CuratorVouched est un ajout
de ~80-100 LOC Rust qui transforme le proof pack de "attestation
unilaterale" en "chaine de preuves avec approbation tiers". Le
rapport effort/valeur est excellent.

**Pourquoi pas l'Option C (Split 4 sprints) :**
Fragmentation. La gouvernance repartie sur 4 sprints produit du code
sans narratif clair. Chaque sprint fait "un peu de gouvernance" au
lieu de faire "un sprint produit coherent".

**Pourquoi pas l'Option D (Gouvernance en S73) :**
Trop tard. La confiance est le coeur de SBFB. Repousser toute
gouvernance a S73 c'est dire "on construit une plateforme de confiance
mais on ne travaille pas sur la confiance pendant 4 mois".

**Pourquoi l'Option E :**
1. **Factory en S67** — le probleme PO est resolu.
2. **CuratorVouched en S68** — placement naturel dans le proof pack
   (un pack de preuves qui inclut l'approbation signee).
3. **Le pilote S69 a de la gouvernance visible** — pas complete, mais
   suffisante pour montrer la proposition de confiance.
4. **Raw-op en S65 rend l'ajout non-breaking** — pas de version bump,
   pas de coordination de mise a jour.
5. **Gouvernance complete en S70 informee par le pilote** — le
   multi-curator scope, le dissent UI, la stale detection sont
   designes avec le feedback reel.
6. **S68 est charge mais gerablee** — CuratorVouched Phase A (100
   LOC), proof pack Phase B-D (assemblage), tests Phase E. 5 phases
   dans la norme.

### 6.3 Ce qui change dans le graphe de dependances

**Ancien graphe (roadmap V2 canon) :**
```
S65 -> S66 -> S67 Gouvernance -> S68 Proof Pack -> S69 Pilote
```

**Nouveau graphe (Option E) :**
```
S65 -> S66 -> S67 Factory -> S68 Proof Pack + CuratorVouched -> S69 Pilote
                                        |
                                        v
                                  S70 Gouvernance Complete
```

**Dependances preservees :**
- S65 (raw-op) -> S68 (CuratorVouched) : OK, raw-op est le pre-requis
  technique de tout ajout d'op feed.
- S66 (durabilite) -> S69 (pilote) : OK, inchange.
- S67 (Factory) -> S69 (Babel canari) : OK, inchange du pivot PO.
- S68 (proof pack) -> S69 (pilote) : OK, le proof pack EST le
  livrable du pilote.

**Dependance ajoutee :**
- S68 (CuratorVouched) -> S70 (Gouvernance Complete) : S70 construit
  sur les ops feed de S68.

### 6.4 Impact sur FEED_FORMAT_VERSION

Avec la strategie raw-op adoptee en S65 :
- S65 : migration `FeedEntry.op` -> `serde_json::Value`. Version reste 1.
- S68 : ajout CuratorVouched + CuratorDisendorsed. Version reste 1.
- S70 : pas de nouvelle op feed. Version reste 1.
- S72 : ajout SearchManifestPublished. Version reste 1.

**Le premier bump de version** sera quand la structure de `FeedEntry`
elle-meme change (ajout de champ obligatoire, changement de hash algo).
Ce n'est prevu dans aucun sprint de la roadmap S65-S75.

### 6.5 Impact sur la spec PUBLIC_FEED_SPEC.md

Section §2.2 "Future operations" liste deja `CuratorVouched` comme
"Sprint 2+". Avec l'option E, cette section est mise a jour en S68 :
- `CuratorVouched` passe de "future" a "implemented" avec payload defini
- `CuratorDisendorsed` est ajoute
- Section §9 "Versioning policy" est reecrite en S65 (raw-op strategy)

### 6.6 Risques de la recommandation

| Risque | Probabilite | Impact | Mitigation |
|--------|-------------|--------|------------|
| S68 deborde (trop charge) | MOYEN (30%) | Proof pack incomplet au pilote | CuratorVouched en Phase A, isole du reste. Meme si proof pack derape, le vouch est livre. |
| Gouvernance S70 est "trop tard" pour le signaling externe | FAIBLE (15%) | Les observateurs NLnet/Kahle ne voient pas de gouvernance au pilote | Le CuratorVouched S68 + proof pack montre deja la chaine de preuves. Le multi-curator scope est un raffinement, pas la fondation. |
| Le pilote S69 revele que CuratorVouched est insuffisant | MOYEN (25%) | Sprint fix entre S69 et S70 | S70 est deja dedie a la gouvernance complete. Le feedback du pilote l'informe directement. |
| La migration raw-op S65 a des bugs JCS/Value | FAIBLE (10%) | Hash-chain cassee silencieusement | Tests deterministes obligatoires : "canonical bytes identiques pour Value vs typed struct" (cf. feed_version_bump_strategy.md §5.7). |

### 6.7 Livrables cles par sprint (vue synthetique)

| Sprint | Theme | Livrables gouvernance | Livrables Factory |
|--------|-------|----------------------|-------------------|
| **S65** | Contrat Public | Feed raw-op, auth tier, version guard, badges UI | -- |
| **S66** | Durabilite | -- | -- |
| **S67** | Factory Foundation | -- | Module broker, template, SBFB.json v2 |
| **S68** | Proof Pack | **CuratorVouched + CuratorDisendorsed** (feed ops + Browse badge + tests adversariaux) | Deploy E2E, proof pack assembly |
| **S69** | Pilote | -- (consomme CuratorVouched) | Babel Reader canari |
| **S70** | Gouvernance Complete | Multi-curator, scope, dissent UI, stale detection, freshness, RevocationCache | -- |

---

## 7. Sources

### Code analyse
- `crates/nexus-coordinator-rs/src/public_feed.rs` — FeedEntry, PublicFeedOperation, verify_chain, validate_feed_operation
- `crates/nexus-core-rs/src/curator.rs` — CuratorList, CuratorListEntry, signing, verification
- `crates/nexus-shell-daemon-core/src/browse.rs` — BrowseAggregator, BrowseEntry, aggregate()
- `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` — CuratorRuntime, attention set
- `crates/nexus-coordinator-rs/src/quarantine_queue.rs` — QuarantineQueue
- `docs/protocol/PUBLIC_FEED_SPEC.md` — spec complete §1-12

### Recherches anterieures croisees
- `.planning/research/s67_gouvernance_confiance_research.md` — design complet gouvernance (5 phases, 7 decisions)
- `.planning/research/feed_version_bump_strategy.md` — strategie raw-op (Option E recommandee)
- `.planning/research/s65_s75_factory_babel_canary_research.md` — pivot PO Factory/Babel
- `.planning/research/s65_contrat_public_research.md` — inventaire badges, taxonomie confiance
- `.planning/research/s65_s75_cross_cutting_research.md` — dependances, carry items, sequencage
- `.planning/research/s68_s69_preuves_pilote_research.md` — proof pack design, pilote checklist
