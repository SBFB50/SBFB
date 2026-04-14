# T4 — State actor mass surveillance

**Tier** : T4 (dragnet / collective, non-targeted)
**Budget** : effectivement illimite mais operations collectives
**Timeline** : annees (programmes decenniaux)
**Skill** : equipes R&D integres, SIGINT, cryptanalyse legale
**Ecrit** : Sprint 17 Phase A (2026-04-14)

---

## 1. Profil

Agence gouvernementale de pays democratique ou semi-democratique :
NSA pre-Snowden, GCHQ, BND, DGSE, ASD, CSE, frontex-like data
fusion centers. Operations legales sous leur propre droit (FISA,
IPA 2016 UK, Loi Renseignement 2015 FR). Mandat officiel :
securite nationale, lutte anti-terrorisme, crime organise,
renseignement etranger.

**Caracteristique clef** : **ciblage non-individuel**. T4 collecte
a l'echelle (dragnet : "filet de peche"), stocke, analyse
retroactivement. Un individu non suspect se retrouve dans les
datasets parce que metadata de son trafic touche un cable
international. Ce n'est pas Pegasus (T5) — c'est XKeyscore, PRISM,
Tempora.

Ce tier **respecte en general des limites formelles** (minimisation,
suppression donnees U.S. persons pour NSA, etc.), imparfaitement
executees mais non-arbitraires. Ne torture pas, ne saisit pas
de device individuel sans mandate. Leak d'outils est un risque
(Shadow Brokers 2016 : outils NSA fuites = utilises par T1-T2).

## 2. Capabilites techniques

**SIGINT passive** :

- Tap cables sous-marins (Tempora UK, Upstream US)
- Taps points d'echange (NetNod SE, LINX UK)
- Acces legal data retention ISP (6 mois UE, plus long US)
- DNS/BGP monitoring a grande echelle
- Passive traffic analysis (timing, size, metadata)
- SSL/TLS downgrade / interception via backbone (partiellement)

**SIGINT active** :

- BGP hijacks (sous couvert diplomatique dans certains cas)
- DNS poisoning a echelle nationale (Chine, Russie explicite ;
  democraties plus limitees)
- Man-in-the-middle CA (possible via CA coopte, ex :
  CNNIC 2015, WoSign)
- Cryptanalyse symetrique legale (AES 128-256 encore safe,
  pre-quantum)
- Compromission vendors (RSA 2010, Solarwinds 2020)

**COMPUTE** :

- Cluster cryptanalyse RSA-1024 possible selon budget (~200M$)
- Cracking MD5, SHA-1 collisions (trivial)
- Farms GPU pour AI / rainbow tables

**CE QU'ILS NE FONT GENERALEMENT PAS** :

- Ciblage individu lambda (T5 only, sous mandat)
- Physical coercion (T5 only)
- Compromission vendor OSS a petite echelle (pas economique)
- Deploy Pegasus sur 1M users (T5 only, par licence NSO)

## 3. Budget & timeline

- Budget NSA annuel (public) : ~11B$ (2023)
- Black budget US community IC : ~90B$/an
- Programmes multi-decennies (TEMPORA depuis 2011)
- Skilled personnel : 30-40k combined staff ecosysteme

## 4. Motivations

- Mandate national security : anti-terror, anti-proliferation
- Counter-intelligence (identifier espions adverses)
- Diplomatic intelligence (intercepter com. ambassades)
- Commercial intelligence (rare, varie par pays)

**Non-motive par** :

- Harassment individuel (sauf mandate specifique)
- Profit financier direct
- Sabotage (sauf operations cyber ciblees : T5 fait ca)

## 5. Tactiques typiques contre SBFB (si SBFB devient adopte)

| Attaque | Mecanique | Probabilite |
|---|---|---|
| Dragnet metadata iroh-blobs | Cable tap → correlation node_id ↔ IP ↔ identite | Haute si > 1M users |
| BGP-level monitoring relais n0 | Observer qui parle a qui | Haute si n0 sous juridiction |
| DNS pkarr monitoring passive | Lister tous les node_id actifs | Haute |
| Legal order vers n0 SA / relay providers | "Donnez logs d'acces" | Haute |
| Injection de relay "ami" sous couverture | SA fait un relay public patriotique | Moyenne |
| Compromission vendor PyO3 / iroh upstream | Insere backdoor telemetry | Basse (detectable source) |
| Backdoor algo crypto | Cherche a influencer NIST PQ choices | Moyenne (post-Snowden prudent) |
| Cracking crypto classique | AES-128 Ed25519 BLAKE3 = safe pre-quantum | Basse |

**Pattern dominant** : **metadata correlation massive**. T4 n'a
pas besoin de lire les tasks pour identifier qui participe a
quoi. Il correlate node_id ↔ IP ↔ abonnement ISP ↔ identite
reele. Si SBFB ne minimise pas le metadata, T4 construit un
graphe social de tous les contributeurs sans jamais casser le
chiffrement.

## 6. Observable indicators

T4 est **tres difficile a detecter** :

- Tres peu de traces dans les logs (passive monitoring upstream)
- Legal requests vers vendors rares mais documentes (transparency
  reports)
- Leaks occasionnels (Snowden 2013, Shadow Brokers 2016) :
  indicateur retrospectif
- Certains patterns BGP anomalous detectables avec RIPE RIS
- CT logs revelent parfois des CA cooptees
- Queries DNS pattern aberrants sur endpoints non-publies

**Ce qui doit alerter** :
- Subpoena / National Security Letter (US) recue pour donnees
  SBFB users — mais gag order empeche disclosure publique direct.

## 7. Mitigations SBFB actuelles

**Livre (partiel)** :

- iroh QUIC avec ChaCha20-Poly1305 : traffic chiffre, mais
  metadata (timing, size, endpoints) reste visible.
- Ed25519 node_id non lie a email / phone / identite reelle
  directement (si user ne leak pas).
- Verified deploy : empeche supply chain trivial mais pas vendor
  PyO3 upstream compromission.

**Partiel** :

- Pas de mixing / cover traffic (Sprint 20+ Tor/Nym).
- Pas d'anonymisation metadata cote iroh-blobs (fetch = visible).
- Pas de multi-path / multi-relay obfuscation.
- Pas de reproducible builds (Sprint 18) pour detecter vendor backdoor.

**Absent** :

- Pas de warrant canary ("a la date X nous n'avons recu aucun
  NSL") — pattern Signal, Riseup.
- Pas de post-quantum crypto migration path (Sprint 26+,
  FIPS 203/204 encore frais).
- Pas de traffic padding / timing randomization (Sprint 22+).
- Pas d'opt-in relai alternatif (Tor, Nym, Yggdrasil) : Sprint 20-22.

## 8. Priorisation

T4 est **preoccupation a moyen-terme** pour apps Gate 3+
(PolitiScan) et **critique** pour Gate 4 (LibanLive). Meme si
le ciblage n'est pas individuel, la collecte metadata suffit pour
identifier des contributeurs a regimes T5.

Actions Sprint 20-30 :

- Reproducible builds (Sprint 18 Phase D)
- Warrant canary pattern ajoute (Sprint 19)
- Tor/Nym transport optionnel (Sprint 20-22)
- Traffic padding cover traffic (Sprint 22+)
- Post-quantum crypto migration plan ecrit (Sprint 26+)

## 9. Mitigations obligatoires par Gate

| Gate | Requirement T4 |
|---|---|
| 1 (DnD Forge) | iroh crypto basique suffit (pas de sensibilite metadata) |
| 2 (TransLingua) | + warrant canary + transparency report annuel |
| 3 (PolitiScan) | + all above + Tor transport optionnel + reproducible builds |
| 4 (LibanLive) | + all above + traffic padding + metadata minimization stricte + PQ-ready |

## 10. References

- Snowden disclosures 2013 (Greenwald "No Place to Hide")
- Schneier "Data and Goliath" (2015, encore pertinent)
- EFF Surveillance Self-Defense
- "The Post-Snowden Landscape" (Lawfare Blog 2023)
- RFC 9420 (MLS protocol) — modern group messaging vs dragnet
