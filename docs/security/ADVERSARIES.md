# Adversaires — taxonomie T0-T5

**Ecrit** : Sprint 17 Phase A (2026-04-14)
**Tip reference** : `f75b2c6` (ouverture Sprint 17 post-gate S16)
**Methodologie** : 6-tier capability model inspire ENISA Threat
Landscape 2024 + EFF Surveillance Self-Defense, adapte au
contexte SBFB (reseau P2P compute + app store OSS).

---

## 1. Rationale du tier system

### 1.1 Pourquoi une taxonomie par tier

Le threat model Sprint 16 Phase E
([`THREAT_MODEL.md`](THREAT_MODEL.md)) structure l'analyse par
**composant** (shell daemon, coordinator, blob-serve, etc.) avec
STRIDE + LINDDUN. Cette vue est utile pour raisonner "quelle
surface offre quel risque", mais elle ne dit rien sur **qui
peut exploiter quoi**.

Un script kiddie et un operateur Pegasus attaquent
fundamentalement differemment :
- Moyens : outils publics vs 0-day targeted
- Timeline : weekends vs mois
- Cibles : opportuniste vs nominativement individuel
- Coercion : zero vs coercion physique possible

Traiter les deux avec la meme list de controles (ex : "TLS
partout", "input validation") produit soit du gold-plating
inutile pour le kiddie, soit un faux sense de securite pour la
Pegasus-class. D'ou l'**echelle monotone de capacites** T0-T5 :
un tier englobe tous les moyens du precedent.

### 1.2 Pourquoi 6 tiers (pas 4, pas 10)

Considerations retenues :

**Moins de 6 tiers (ex : 4)** : on fusionne T0 avec T1 (user
misconfig avec script kiddie) et T4 avec T5 (dragnet avec
targeted). Perdu : pas de distinction "UX defaults" vs
"technical defense", pas de distinction "collecte massive" vs
"physical coercion". Ces distinctions determinent des
mitigations differentes — on perd en actionabilite.

**Plus de 6 tiers (ex : 10)** : granularite type
"script kiddie / skilled amateur / crime lieutenant / crime
boss / ..." — valeur marginale faible, overhead entretien eleve
pour un projet solo.

**6 tiers** est la granularite ou chaque step represente un
saut de capacite **qualitatif**, pas juste quantitatif. Un
mitigation efficace contre T3 l'est mecaniquement contre T0-T2
(par definition monotone). Permet de dire "cette mitigation
protege jusqu'a T3, pas T4+".

### 1.3 Pourquoi pas MITRE ATT&CK, kill-chain, ou taxonomie par motivation

**MITRE ATT&CK** : excellent framework pour **tactics / techniques
classification** (TA0001 Initial Access, T1190 Public-Facing
Application Exploit, etc.) mais **orientation entreprise**.
Categories AD / Windows lateral movement / OAuth abuse sont
surdimensionees pour un stack P2P. ATT&CK sera **re-utilise**
dans `ATTACK_SCENARIOS.md` pour classer les techniques
specifiques a chaque scenario — mais la taxonomie d'adversaire
reste T0-T5.

**Lockheed Kill Chain** : trop simpliste pour attaques modernes
persistent threat. Se prete mal aux attaques P2P non-lineaires
(Sybil, Eclipse).

**Taxonomie par motivation (criminal / state / activism)** :
pedagogique mais ne traduit pas en capabilities techniques.
Deux T2 avec la meme motivation (profit) peuvent avoir des
capacites tres differentes si l'un opere en Russie (protection
informelle) et l'autre en UE (contrainte legale forte).

### 1.4 Limites du modele

- **Monotone stricte imparfaite** : un T2 peut avoir plus de
  skill technique qu'un T3 (qui achete des pentesters sous
  contrat). La monotonie porte sur les **capacites plafond**,
  pas sur l'efficacite moyenne.
- **Frontieres floues** : un operateur T2 peut etre sous-traitant
  d'un T3 (hire ransomware group pour task precise). Le tier
  reflete le **commanditaire** et son budget, pas l'executant.
- **Evolution temporelle** : les tiers bougent. Pegasus etait
  quasi-exclusif en 2018 (T5), accessible a ~20 etats en 2024.
  Traiter T5 comme stable 5 ans = risque.
- **Biais geopolitique** : la distinction T4 vs T5 reflete une
  norme democratique. Un pays qui passe de democratie fragile a
  autoritarisme se promeut de T4 a T5 en mois. Modele a revoir
  tous les 2 ans minimum.

---

## 2. Table synthetique

| Tier | Persona | Capacite max | Budget | Timeline | Motivation | Fiche |
|---|---|---|---|---|---|---|
| **T0** | User legitime mal configure | Misconfig, oubli, partage accidentel | zero | immediat | pas adversaire (frustration, experimentation) | [`T0-curious-user.md`](adversaries/T0-curious-user.md) |
| **T1** | Script kiddie / troll | Outils publics, 0 0-day | <1k$ | jours | clout, vengeance, trolling | [`T1-script-kiddie.md`](adversaries/T1-script-kiddie.md) |
| **T2** | Criminel organise | 1-2 0-days achetes, pentest avance, RaaS | 10-100k$ | semaines-mois | profit financier | [`T2-criminal-organized.md`](adversaries/T2-criminal-organized.md) |
| **T3** | Corp. hostile / concurrent | Pentest legal, legal machine, PR, infiltration | 100k-1M$ | mois-annees | proteger business, discrediter, IP theft | [`T3-corporate.md`](adversaries/T3-corporate.md) |
| **T4** | State mass surveillance | SIGINT national, dragnet, BGP, cryptanalyse legale | ~illimite collectif | decenies | securite nat. anti-terror counter-intel | [`T4-state-dragnet.md`](adversaries/T4-state-dragnet.md) |
| **T5** | State targeted | Pegasus, IMSI, arrest, coercion, full spectrum | illimite par cible | jours-mois | controle politique, suppression dissidence | [`T5-state-targeted.md`](adversaries/T5-state-targeted.md) |

---

## 3. Mapping tier → app risk (gate)

Le systeme de gates Gate 1-4 est defini dans
[`RELEASE_GATES.md`](RELEASE_GATES.md) (Sprint 17 Phase E). Il
conditionne la release d'une app au durcissement suffisant
contre le tier d'adversaire qu'elle affronte realistiquement.

| Tier mitige jusqu'a | Gate min atteignable | Exemples apps |
|---|---|---|
| T0-T1 | Gate 1 (Low stakes) | DnD Forge, hello-world-app, test utilities |
| T0-T2 | Gate 2 (Medium stakes, PII light) | TransLingua, FamilyScan |
| T0-T3 | Gate 3 (High stakes, reputation + legal) | PolitiScan, NEXUS cold-case |
| T0-T5 | Gate 4 (Critical, life-safety) | LibanLive, war-crime documentation apps |

### 3.1 Pourquoi T5 non-atteignable avant Gate 4 complet

Une app qui sert une population cible T5 (contributeurs en zone
de conflit, dissidents sous regime hostile) sans tous les
controles Gate 4 livres = complicite de harm. Cette clause est
**structurelle** dans le gate system : le code ne peut pas
sortir en "beta ouverte" pour LibanLive. Il faut :

- Encryption at rest + duress PIN + panic wipe (infra Sprint 18-20)
- Multi-relai federation (ONGs) + Tor / Nym transport optionnel
  (Sprint 20-22)
- Reproducible builds + Radicle / forge mirror anti-takedown
  (Sprint 18)
- Sybil-resistance kudos-weighted (Sprint 21-22)
- Audit externe Cure53 ou Trail of Bits comprehensive (Sprint 25+,
  budget ~50-100k$)
- Partenariat multi-ONGs juridictionnellement diverses (Amnesty
  + HRW + CPJ + MSF + Human Rights Data Analysis Group)
- Formation OpSec ouverte pour contributeurs (template EFF)
- Beta ferme 18+ mois avec population formee + ethics review board

Cette liste **n'est pas optionnelle**. Une app avec 80% des items
= pas Gate 4 = non-deployable pour sa population cible.

### 3.2 Escalation de gate (app qui monte)

Une app peut monter de gate quand son usage reel evolue :

- **DnD Forge** (Gate 1) → hub social avec DMs = PII → Gate 2
- **TransLingua** (Gate 2) → utilise en zone crise pour
  traduction actes reels de conflit → Gate 3-4 selon contexte

Le gate est revu annuellement + post-incident. Une montee de
gate = release freeze jusqu'au hardening correspondant livre.

### 3.3 De-escalation (rare)

Exceptionnellement, une app peut descendre de gate si usage reel
diverge de l'intent. **PolitiScan** commercialise par une agence
de com comme outil de recherche academique (perd la dimension
investigation journalistique) = Gate 2. Rare, documente.

---

## 4. Glossaire

**0-day / zero-day** : vulnerabilite inconnue du vendor au
moment d'exploitation. Prix marche gris 2024 : 50k-2.5M$ selon
plateforme (Zerodium bounty public). Les plus chers : iPhone
zero-click Chrome SBX + LPE Windows chainable.

**Dragnet (collection)** : collecte massive non-ciblee de
metadata / trafic sur population generale, avec analyse
retroactive. Terme tire des pratiques peche industrielle
("filet de peche qui prend tout"). Oppose a targeted surveillance.

**IMSI catcher (StingRay)** : appareil simulant une cell tower
mobile pour capturer IMEI/IMSI des phones a proximite, et
optionnellement intercepter trafic si operateur cooperant
(operateur leak cles session). Legal sous mandate US, usage
documente democratique + autoritaire.

**Pegasus** : spyware developpe par NSO Group (Israel), vendu
a gouvernements. Capability zero-click (install via
vulnerabilite WhatsApp / iMessage sans interaction user),
acces complet phone. ~20+ gouvernements clients documentes
2024. Exemples usage : Khashoggi, journalists AP Mexique,
Human Rights Watch mobiles.

**Predator (Intellexa)** : concurrent Pegasus, developpe par
Cytrox group, vendu similarly. Base Israel + Europe Est.

**Cellebrite UFED** : device forensics phone, extract contenu
y compris apps chiffres via bypasses vendor. Used par
law enforcement democratique + autoritaire. ~10-20k$ unite.

**Side-channel** : classe d'attaques qui exploit des fuites
**non-intentional** d'info via canaux physiques (timing, power,
EM, acoustic, thermal). Ex : Spectre/Meltdown (cache timing),
rowhammer (DRAM disturbance), van Eck phreaking (monitor
radiation).

**Supply chain attack** : compromission d'une dependance /
vendor en amont de la cible finale. Ex : SolarWinds 2020,
XZ-utils 2024 (backdoor upstream liblzma), event-stream npm
2018. Difficile a detecter sans reproducible builds + audit
comprehensive.

**DPI (Deep Packet Inspection)** : inspection du contenu des
paquets IP au-dela des headers, utilise pour filtering ISP
(Great Firewall China, Iran national DPI). Combat par
pluggable transports (obfs4, meek, Snowflake).

**Pluggable transports (Tor)** : modules qui obfusquent trafic
Tor pour contourner DPI. Variantes : obfs4 (random-looking),
meek (HTTPS fronting), Snowflake (WebRTC P2P). Deployment
evolutif vs DPI arms race.

**Warrant canary** : statement publie periodiquement par un
service declarant "nous n'avons recu aucune legal order". Si le
statement disparait ou n'est pas renouvelle, signal indirect
qu'une legal order confidentielle a ete recue. Pattern
Signal, Riseup, Canary Mail. Legalite debattue mais non-refutee
US.

**Deadman switch** : mecanisme auto-declenche si heartbeat
contributeur manque N periods. Peut auto-desactiver compte,
wiper donnees, publier document en cas de disparition. Pattern
journalists sensitive publication.

**Duress PIN** : code secret qui, quand entre, declenche une
action coerce-safe (ex : wipe silencieux, unlock vers fake
data, alerte contact externe). Permet a l'user sous coercion
physique de fournir un code "valide" sans livrer le contenu.

**Panic wipe** : action utilisateur emergency (ex : 5 taps sur
un bouton) qui wipe keypair + donnees sensibles immediatement,
sans confirmation. Contre-mesure checkpoint / arrest imminent.

**Plausible deniability (crypto)** : design ou l'existence meme
de donnees chiffrees est niable. Pattern VeraCrypt hidden
volumes : meme avec le password du volume outer, un attaquant
ne peut prouver qu'un volume inner existe.

**Sybil attack** : creation d'un grand nombre de fake
identites pour sur-representer une opinion / manipuler un
vote / saturer un systeme de reputation. Fondamental dans tout
systeme P2P sans cout-par-identite.

**Eclipse attack** : isolement d'un pair dans un sous-graphe
controle par l'attaquant (tous ses peers sont fake). Le pair
voit une realite coherente mais biaisee. Pattern Bitcoin
Eclipse (Heilman et al. 2015) : montrable en 40 minutes sans
mitigations.

**Kudos ledger** : systeme de reputation per-project interne
a SBFB, accumule par contributions verifiees. Design Sprint 6.

**Keyoxide** : standard pour prouver ownership d'une identite
decentralisee via self-certified claims (OpenPGP + notation
standardized). Utilise Sprint 14 pour lier node_id Ed25519 a
repo git (le repo contient SBFB.json signe par la cle du
node).

**SLSA (Supply-chain Levels for Software Artifacts)** :
framework Google pour classifier niveau de securite chaine
approvisionnement software. SBFB implemente Level 1
(Sprint 14) : provenance.json signe, metadata basic. Level 2+
requiert reproducible builds + service hardened (Sprint 18+).

**Transparency report** : rapport periodique (trimestriel/annuel)
sur les legal orders recues, leur juridiction, et la reponse
vendor. Pattern Signal, Cloudflare, Google. Augmente legitimite
vs users.

---

## 5. Comment utiliser ce document

### 5.1 Pour un designer de nouvelle feature

1. Identifier quel tier(s) la feature peut servir (quelle app l'utilise).
2. Verifier la fiche du tier correspondant : quelles mitigations
   sont requises ?
3. Checker [`ATTACK_SCENARIOS.md`](ATTACK_SCENARIOS.md) pour voir
   si un scenario similar est deja modelise.
4. Si mitigation manquante identifiee : ajouter dans
   [`HARDENING_ROADMAP.md`](HARDENING_ROADMAP.md) (Sprint 17 Phase D).

### 5.2 Pour un publisher d'app

1. Identifier population cible (DnD players ? journalists ?
   dissidents ?).
2. Mapper population a tier adversaire probable.
3. Lire la fiche du tier + Gate requirement correspondant dans
   [`RELEASE_GATES.md`](RELEASE_GATES.md).
4. Verifier que l'app (infrastructure SBFB mainline + code
   app-specific) satisfait tous les requirements du Gate.
5. Si gap : ne pas publier en production (beta ferme acceptable
   avec users formes).

### 5.3 Pour un auditeur externe (Cure53, ToB, community)

1. Utiliser T0-T5 comme scope specifier : "cet audit couvre
   T1-T3" / "cet audit couvre T1-T5".
2. Verifier que chaque scenario de `ATTACK_SCENARIOS.md` est
   exerce (au moins review statique + test dynamique possible).
3. Reporter findings mappees au tier (ex : "vulnerability
   exploitable T2+" plutot que "vulnerability high severity")
   pour faciliter prioritisation.

---

## 6. Revision & evolution

Ce document est revu **au minimum a chaque audit externe** ou
en cas de nouveau tier emergent (ex : si Pegasus-class devient
T4 accessible, renommer actuel T5 en T6). Les Sprint 18+ peuvent
ajouter des fiches complementaires (sub-tiers : T5a democratic-
police vs T5b authoritarian-military) si granularite necessaire.

Derniere revision : 2026-04-14 (creation Sprint 17 Phase A).
Prochaine revision programmee : **pre-Gate 3** (estimation
Sprint 25+) pour audit externe.

---

## 7. References

- ENISA Threat Landscape 2024
- EFF Surveillance Self-Defense — Assessing Your Risks
- Citizen Lab (citizenlab.ca) — technical reports state actor
  surveillance 2020-2025
- Amnesty International Security Lab — Pegasus forensic methods
- MITRE ATT&CK Enterprise + ICS
- Access Now Digital Security Helpline — case studies
- NIST SP 800-30 Rev 1 — Guide for Conducting Risk Assessments
- Schneier "Applied Cryptography" 3rd ed (2015) — adversary modeling
- Bruce Schneier "Secrets and Lies" (2000, re-reviewed 2020) —
  real-world threat modeling
