# T3 — Entreprise hostile / concurrent

**Tier** : T3
**Budget** : 100k-1M$ par ops
**Timeline** : mois a annees
**Skill** : pentesters legaux + legal + PR machine
**Ecrit** : Sprint 17 Phase A (2026-04-14)

---

## 1. Profil

Entite legale (societe commerciale, fondation rivale, consulting
firm) avec interet explicite contre SBFB ou une app dessus.
Motivations variees :

- **Concurrent direct** : voit SBFB eroder son marche (ex : un
  cloud LLM providers qui n'aime pas qu'un reseau P2P donne le
  meme service gratuit).
- **Concurrent d'une app** : PolitiScan deplait a un lobby
  politique dote (thinktank, fondation partisane).
- **IP holder** : societe qui pense que SBFB enfreint des brevets
  software (ex : Intellectual Ventures, Ericsson IP division).
- **Reputation management** : agence PR mandat par un tier qui
  veut discrediter avant adoption massive.

T3 agit **dans les limites de la legalite apparente**. Pas de
0-day achete sur darknet (risque legal) mais pentesters sur
contrat legal, lawyers agressifs, PR campagnes. La "hack" est
rarement technique — c'est juridique, economique, social.

## 2. Capabilites techniques

**Techniques "legales" / gris** :

- Pentesters professionnels (ex : ex-NCC Group, IOActive) payes
  100k-500k pour une engagement
- Bug bounty vendors qui vendent les findings a T3 plutot que
  disclosure responsible (rare mais arrive)
- Reverse engineering public software (legal sous Copyright Act
  clean-room) pour extraire IP
- Audits de code public + social analysis
- Infiltration communaute OSS (hire un contributeur qui pousse
  des "improvements" partisans)
- Trademark challenges, cease & desist, DMCA
- Depots de brevets prealables bloquants

**Techniques illegales mais plausibles** (pour les acteurs less
ethics) :

- Hire T2 prestataires pour le "dirty work" (plausible
  deniability)
- Phishing cible contre devs cles
- Donnees corporate hack-and-leak via pigeons

**Ce qu'ils NE peuvent PAS faire "proprement"** :

- Compromettre cles Ed25519 sans complicite interne
- DDoS majeure (signature trop visible)
- Physical coercion (T5 only)
- IMSI catchers, Pegasus (T4+ only)

## 3. Budget & timeline

- Engagement pentest : 100-500k$ pour 3-6 mois
- Legal machine : 50-200k$ par campagne C&D
- PR agency : 50-150k$ pour smear campaign
- Brevets defensifs : 20-50k$ par brevet depose
- Total campagne : 200k-1M$ sur 1-3 ans

Timeline : deliberee, longue. T3 peut attendre 6 mois que SBFB
decolle avant d'agir. Ne rush pas.

## 4. Motivations

- Proteger la business line existante
- Degrader la reputation avant adoption massive
- Creer un precedent legal qui bloque des features
- Forcer SBFB a depenser en defense legale (sapping ressources)
- Recuperer IP utile via reverse engineering

## 5. Tactiques typiques contre SBFB

**Plausibles prioritaires** :

| Attaque | Cout estime | Probabilite |
|---|---|---|
| Pentest complet engagement 6 mois, vendre findings ou exploiter | 300k$ | Moyenne si SBFB > 100k users |
| Infiltration maintainer : hire un dev pour pousser des "improvements" qui dilutent la vision | 50k$/an | Moyenne |
| PR campagne : "SBFB est une plateforme pour criminels / Stormfront" | 150k$ | Moyenne post-release Gate 3 |
| C&D spam au sujet de trademark / patent putatif | 50k$/campagne | Haute si SBFB nom prote commercial |
| Reverse engineering de nexus-core-rs pour integrer equivalent proprietaire | 200k$ | Haute apres adoption 1M users |
| Fake vulnerability report pour discrediter (rejected puis leak selective) | 20k$ | Basse (reputation vendor) |
| Harcelement legal via DMCA / GDPR sur contenus publics | 30k$ | Moyenne |

**Pattern dominant** : **discredit via PR + legal pressure**.
T3 ne cherche pas a exploiter techniquement. Il cherche a faire
peur aux users et aux partners (Amnesty, EFF) avant que
l'ecosysteme soit solide.

## 6. Observable indicators

T3 laisse des traces principalement non-techniques :

- Tweets coordonnes par accounts journalistes-lobby
- Demande de commentaire de media etablis sur "le cote sombre
  de SBFB" (le pitch fourni par PR agency)
- Commits dans le repo avec pseudo + emails VPN, pattern de
  contribution atypique (beaucoup de refactor, peu de fix)
- Filing de brevets software dans les 12 mois post-release
  de feature SBFB
- Trademark challenges sur noms proches (nexus-grid → Nexus
  Corporation, Cisco Nexus, etc.)
- Apparition de tool equivalent "enterprise" avec pricing 99$/mois
  post-adoption SBFB

## 7. Mitigations SBFB actuelles

**Livre** :

- AGPL-3.0 license : bloque l'appropriation proprietaire sans
  contribution back (cf. decision Day 0 Sprint 0).
- Open source full + commits publics + contributeur verifie :
  infiltration maintainer detectable via git blame + review.
- Threat model public (Sprint 16 Phase E) : T3 ne peut pas
  pretendre "SBFB cache ses risques".
- Verified deploy + provenance : T3 ne peut pas publier fake
  exploit / defacement app sans laisser trace Keyoxide.

**Partiel** :

- Pas de multi-maintainer pour blocker infiltration (projet
  solo Sprint 17 : single point of failure).
- Pas de policy ecrite sur code of conduct / governance.
- Pas de bounty program officiel (report vendor peut choisir
  SBFB ou T3 pour selling).

**Absent** :

- Pas de legal defense fund (LF Security, SFLC partner).
- Pas de trademark registre (nom "SBFB" non proteger).
- Pas de patent defense strategy (OIN membership Sprint 22+).
- Pas d'audit externe qui pourrait contrer une PR campagne avec
  un rapport ChainExchange.

## 8. Priorisation

T3 devient preoccupant **a partir de Gate 2-3 (PolitiScan /
TransLingua adoption ≥ 100k users)**. Avant : cible trop petite.

Actions Sprint 18-30 :

- Governance writeup + CoC (Sprint 18)
- Bounty program via HackerOne ou direct (Sprint 19)
- Trademark registration SBFB (Sprint 20, budget ~5k$)
- Open Invention Network membership (Sprint 22, gratuit pour OSS)
- Engage 1+ partenariat FSF / SFLC pour legal defense (Sprint 24)
- Audit Cure53 / ToB publique pre-Gate 3 (Sprint 25, ~15-50k$)

## 9. Mitigations obligatoires par Gate

| Gate | Requirement T3 |
|---|---|
| 1 (DnD Forge) | AGPL + open source public suffit |
| 2 (TransLingua) | + CoC + bounty program + trademark registre |
| 3 (PolitiScan) | + audit externe publique + partenariat ONG + OIN membership |
| 4 (LibanLive) | + all above + legal defense fund + multi-country counsel |

## 10. References

- "The Corporate Patent Arsenal" (Harvard Law 2022)
- Bruce Sterling, "The Hacker Crackdown" (revisite 2024)
- OIN (Open Invention Network) member benefits
- Cases historiques : SCO vs IBM, Oracle vs Google (API copyright)
