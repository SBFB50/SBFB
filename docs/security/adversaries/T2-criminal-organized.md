# T2 — Criminel organise / ransomware group

**Tier** : T2
**Budget** : 10k-100k$ par ops
**Timeline** : semaines a mois
**Skill** : operateurs formes, peut acheter 1-2 0-days gris
**Ecrit** : Sprint 17 Phase A (2026-04-14)

---

## 1. Profil

Groupe structure : leaders (operateurs), developers payes, money
mules, affiliates. Modele business : ransomware-as-a-service,
crypto-jacking, fraude bancaire, marketplaces gris. Exemples
publics : LockBit (avant takedown 2024), Conti alumni, BlackCat,
Clop, FIN7, Evilnum.

**Objectif principal = retour financier**. Pas de politique, pas
de trolling. Si le ROI d'une attaque est inferieur au cout
operationnel, ils passent. Ce qui les rend **predictibles** :
attaquent les surfaces que la massi des users a (Windows entreprise,
Office365, VPN Fortinet, etc.). SBFB = cible marginale sauf si
atteint 100k+ users actifs.

Variante "access broker" : vend l'acces a un T3/T4, pas
d'exploitation directe.

## 2. Capabilites techniques

**Outils utilises** :

- Cobalt Strike licensed (leaked ou acheta)
- Ransomware toolkit custom (affilie RaaS)
- Phishing infrastructure industrielle (Evilginx avance, MitM O365)
- Loader custom, anti-EDR bypass a jour
- Acces a 1-2 0-days achetes sur Zerodium / Anon Dealer (Windows
  LPE, Chrome RCE, rare)
- Initial access brokers (credentials pre-achetees 100-5000$)
- Money laundering via mixers, OTC desks

**Ce qu'ils peuvent faire** :

- RCE chainee sur infrastructure entreprise
- Development de malware multi-stage signe
- Abuse legit toolchain (MSIX, ClickOnce, Electron packagers)
- Lateral movement AD / Entra ID en heures
- Negotiation extortion (calls avec victimes)

**Ce qu'ils NE peuvent PAS faire** :

- Compromettre un vendor majeur sans insider (rare)
- Reverse protocol militaire (hors cibles)
- Developper un exploit Ed25519 (cryptanalyse = budget T4+)
- Compromettre CI GitHub a grande echelle (supply chain : T3-T4)

## 3. Budget & timeline

- Budget ops : 10k-100k$ par campagne
- 0-days : 1-2 maximum, acheta 50-200k$ piece
- Operateurs : 5-50 personnes payees mensuellement
- Timeline attack : 2 semaines planning + jours d'exec
- Persistence infrastructure : mois a annees (jusqu'a takedown)

## 4. Motivations

- Profit (seul critere)
- ROI minimum acceptable : 100k$ par ops reussie
- Evitent les cibles avec risque legal personnel (hopitaux
  post-2021 politique evite, infrastructure OTAN)

## 5. Tactiques typiques contre SBFB

**Scenarios plausibles** :

| Attaque | ROI estime | Probabilite |
|---|---|---|
| Crypto-miner publie via deploy-from-repo, hit N workers | 100$/mois/worker × 1000 = 100k$/mois | Moyenne si SBFB > 10k users |
| Ransomware deploy-from-repo qui chiffre fichiers user via worker | Cher (faut LPE Windows), ROI tournant | Basse |
| Fake "AI image gen" app qui exfil prompts (donnees sensibles revendables) | 1$/prompt × millions | Moyenne |
| Compromission repo contributeur populaire → push backdoor | Depend du nombre d'users | Moyenne |
| Fraude kudos (fake workers accumulent kudos et pay-out) | Dependant si kudos devient monetisable | Haute si kudos-to-fiat |

**Pattern dominant** : **supply chain via deploy-from-repo**.
T2 paye 5k$ pour compromettre le repo d'un contributeur populaire,
pousse un commit signe avec la cle volee, SBFB clone et deploy
la version backdoored. Victimes = tous les users qui font
download apres publication.

## 6. Observable indicators

T2 laisse moins de traces que T1 mais des patterns reconnaissables :

- Bursts d'activite aligned avec bus. hours ouest-europeens
  (Amsterdam / Londres ops centers)
- Infrastructure C2 sur bulletproof hosting connus (FlokiNET,
  Yalishanda, Njalla)
- Malware avec strings russes/ukrainiens/chinois (attribution
  partielle)
- Binaires signes avec certs voles stolen (reporter via
  CRLite)
- Pattern kudos anormal : accumulation rapide, zero contributions
  humaines reelles
- Forks de repos populaires avec commits timing suspicieux

## 7. Mitigations SBFB actuelles

**Livre** :

- Verified deploy (Sprint 14) : force public repo + Keyoxide
  proof — eleve le cost-of-attack significativement (T2 doit
  hack le repo, pas juste publier un zip).
- Provenance signee (Sprint 14) : difficile de pousser sans la
  cle du contributeur. T2 doit compromettre la cle locale ou la
  forge (token CI).
- Consent GPU 4 niveaux + caps W/VRAM/h (Sprint 16 C) : worker
  ne peut pas etre silencieusement detourne mining en continu
  — le cap heures/jour bloque.
- `is_open_source` flag + chain de provenance (Sprint 16 D) : une
  app qui pretend etre open source sans chain provenance est
  rejected cote daemon (`87cae71`).

**Partiel** :

- Pas de monitoring anomaly cote coordinator (detect mass-deploy
  rapide d'un meme signer).
- Pas de multi-party signing pour releases critiques (Sprint 20+).
- Audit de chaine supply manuel, pas CI (reproducible builds
  Sprint 18 Phase D).

**Absent** :

- Pas de signing multi-sig obligatoire pour apps Gate 3+ (Sprint 22+).
- Pas de sandbox runtime worker-side (le cap hours mitige le
  mining prolonge mais pas un steal one-shot).
- Pas de detect fraud kudos (Sybil-resistance Sprint 21-22).

## 8. Priorisation

T2 est le **premier tier qui justifie des investissements
proactifs**. Sprint 18-22 contient :

- Reproducible builds + CI signing (Sprint 18 Phase D)
- Multi-sig release pour projets Gate 3+ (Sprint 20+)
- Kudos Sybil-resistance (Sprint 21-22)
- Monitoring anomaly coordinator-side (Sprint 19+)

## 9. Mitigations obligatoires par Gate

| Gate | Requirement T2 |
|---|---|
| 1 (DnD Forge) | Sprint 14 + 16 suffisent (low-stakes) |
| 2 (TransLingua) | + reproducible builds + multi-sig pour release |
| 3 (PolitiScan) | + all above + independent SBOM audit annuel |
| 4 (LibanLive) | + all above + hardware-rooted signing (YubiKey / HSM) |

## 10. References

- Verizon DBIR 2024 (section Ransomware)
- Chainalysis Crypto Crime Report 2024
- CrowdStrike Global Threat Report 2025
- "The Ransomware Payments Problem" (Brookings 2023)
