# T5 — State actor targeted (regime hostile)

**Tier** : T5 (targeted, individualise, coercion physique possible)
**Budget** : illimite pour cibles prioritaires
**Timeline** : jours a mois pour une cible specifique
**Skill** : full spectrum — cyber, humint, physical, legal
**Ecrit** : Sprint 17 Phase A (2026-04-14)
**Tier DECISIF pour LibanLive / Gate 4**

---

## 1. Profil

Regime autoritaire ou democratique-fragilise qui mene des
operations ciblees contre des populations specifiques :
dissidents, journalistes, opposants, minorites ethniques,
contributeurs identifies. Exemples publics : Chine contre
Ouighours, Russie contre journalistes independants + LGBTQ,
Iran contre feminists Mahsa Amini movement, Arabie Saoudite
(Khashoggi), Emirats (Pegasus deployments documentes), Israel
contre activistes pro-Palestine, Liban sous influence Hezbollah
contre sources LibanLive type app.

**Caracteristique clef** : **coercion physique = scope in**. Un
contributeur SBFB peut etre arrete a un checkpoint, son device
saisi, forensics complet, famille menacee. Aucun degre de
chiffrement du trafic ne mitige une cle Ed25519 extraite d'un
phone desverrouille sous contrainte.

Ce tier est **fondamentalement different** de T1-T4 : le threat
model classique (chiffrement, authentification, sandbox) est
necessaire mais **loin d'etre suffisant**. Il faut ajouter :
duress protocols, panic wipe, deniable encryption, plausible
deniability, multi-juridictions hosting.

**Le cas LibanLive qui a declenche ce sprint** : une app qui
serve aux journalistes citoyens de documenter des crimes de
guerre en zone active (sud-Liban, bande de Gaza). Les
contributeurs sont cibles par les parties au conflit. Une
erreur de design = prisons, mort.

## 2. Capabilites techniques + humaines

**Cyber (equivalent T4+ mais targeted)** :

- Pegasus (NSO Group) : zero-click iPhone / Android, 8 vendors
  actifs 2024
- Predator (Intellexa) — successeur Pegasus avec Cytrox group
- FinSpy, Quadream, Paragon — concurrents actifs
- Cellebrite UFED — forensics device physique (phone unlocked ou
  acces via bypass vendor)
- GrayKey — concurrent Cellebrite
- IMSI catchers (StingRay) — capture IMEI/IMSI mobile proches
  + MitM optionnel si coop operator
- Deep packet inspection nationale (Great Firewall China, Iran
  national DPI, Russie SORM)
- BGP control national level (juridiction sur IXP local)

**HUMINT (humain)** :

- Informants dans communaute cible
- Contributeurs retournes (arrete + coercion famille)
- Infiltration organisations societe civile
- Corruption officials / journalists etrangers

**LEGAL / PHYSICAL** :

- Interpol Red Notices abusives contre dissidents exilies
- Extradition demandes massives
- Pression familles residantes (hostage diplomatique)
- Arrestation checkpoint + device seize immediat
- Interrogation coercive ("pour le suspect", torture psychologique
  ou physique selon regime)
- Assassinations ciblees (Khashoggi, Navalny, Kirtan Ayurvedics)

**ECONOMIC / DIPLOMATIC** :

- Sanctions secondaires sur hosts / relays qui ne cooperent pas
- Blocage banquer des ONGs soutenant l'app
- Pressure vendors upstream (GitHub a retire des accounts russes
  post-2022 sous pressure US, mais l'inverse = Chine a force
  retrait apps VPN 2020)

## 3. Budget & timeline

- Pegasus license : 1.5M-5M$ par pays, unlimited targets
- Predator license : ~2-3M$ equivalent
- Operation cost per target : ~50-500k$ (full spectrum)
- Cellebrite : ~10-20k$ hardware + licence annuelle
- IMSI catcher : 10-200k$ hardware
- Coercion d'une famille : free (levers of power existants)
- Timeline per target : **jours** (Pegasus zero-click < 1h
  installation, forensics immediate sur device saisi)

## 4. Motivations

- Controle political absolute
- Suppression de documentation crimes guerre / droits humains
- Decouragement structurel de l'opposition
- Signaler aux potentiels dissidents le cout personnel
- Parfois : exemple public (execution publique pour effet
  dissuasif)

## 5. Tactiques typiques contre SBFB + LibanLive

**Scenarios obligatoirement couverts** :

| Attaque | Mecanique | Probabilite pour LibanLive |
|---|---|---|
| Checkpoint seize device | Phone unlocked extract all : keypair, consent, iroh-docs cache, app data | Haute |
| Pegasus zero-click installe avant contribution | Install via iMessage / WhatsApp, read all, logger keys | Haute |
| IMSI catcher sur zone de couverture Gaza/Liban sud | Correlate IMEI ↔ node_id via timing + traffic | Haute |
| Informant dans cellule contributeurs | Leak identites reelles, passwords, routine d'usage | Haute |
| ISP national block relais n0 | SBFB network fragmente, contribution impossible | Haute |
| Hack GitHub org qui heberge app repo | Push version backdoored, disable Keyoxide check via fake SBFB.json | Moyenne |
| Arrest contributeur + force continue comme "honeypot" | Contribue depuis prison, poisons queries adversarial | Moyenne |
| Attaque famille contributeur residant | Leverage pour obtenir cooperation | Haute |
| Fake curator liste signee avec cle volee journaliste detenu | Pousse "curateur officiel" empoisonne | Moyenne |

**Pattern dominant** : **arrest + device seize + coercion**.
Aucune mitigation software seule ne resout ca. Il faut des
mesures physiques : duress PIN, panic wipe, plausible deniability
volumes (VeraCrypt hidden volumes pattern), plausible usage
story.

## 6. Observable indicators

T5 actif = **catastrophe si pas detecte avant impact** :

- Arrestations non-expliquees de contributeurs (revelateur
  post-fait)
- Accuse comportement atypique sur son account (contributions
  inversees, curateurs changes)
- Devices saisis formellement (par ordre judiciaire dans
  juridictions autoritaires)
- Compte du contributeur revient actif apres detention = peut-etre
  compromis sous coercion
- Infrastructure "disparait" d'un pays (relais bloques)
- Partenaires ONG recoivent threats formelles ou informelles

**Pattern defense** :
- Deadman switch : contributeur doit signer heartbeat hebdo ; si
  3 semaines sans signal, auto-disabled.
- Safe-check code word via canal independant (Signal secondary
  account, face-to-face).

## 7. Mitigations SBFB actuelles

**Livre (minimal)** :

- Ed25519 node_id non lie directement a identite reele
- iroh QUIC chiffre (empeche traffic analysis "in-flight" passive)
- Consent + caps GPU (empeche device utilise sans owner consent)

**Tout le reste est ABSENT**. SBFB tel que Sprint 16 N'EST PAS
deployable contre T5 pour LibanLive. Le gap est reel et
documente (`sprint17_kickoff.md` §1.2 cas LibanLive, 8 items
critique manquants).

**Items ABSENTS critique-T5** :

- Encryption at rest du keypair (Sprint 18-19 required)
- Duress PIN (user enter code X → wipe silencieux + fake data)
- Panic wipe button (5-taps emergency wipe)
- Deniable encryption (VeraCrypt hidden volume pattern)
- Plausible deniability app routine (app look like calculator)
- Multi-relay federation (Sprint 18-19)
- Tor / Nym transport (Sprint 20-22)
- Bridges + pluggable transports (obfs4, meek) contre DPI
  (Sprint 20-22)
- Deadman switch / heartbeat (Sprint 22+)
- Warrant canary per-contributor (Sprint 22+)
- Panic procedure documentation (Sprint 20+, partenariat EFF)
- Secure boot integration (Sprint 25+, hardware-rooted)
- Multi-sig releases (Sprint 20+)
- Emergency revocation protocol (Sprint 22+)

## 8. Priorisation

T5 defense = **scope integral de Sprint 18-30+**. Un app Gate 4
(LibanLive) ne sort pas avant que **tous** les items ci-dessus
soient landed + audit externe complet (~50-100k$ Cure53/ToB) +
formation OpSec (EFF template) + partenariat multi-ONG signe
+ beta ferme 18 mois avec population formee.

Avant la release Gate 4 : **il faut dire non**. Une app pour
population cible T5 deploye trop tot = deaths. Responsabilite
ecrasante.

## 9. Mitigations obligatoires par Gate

| Gate | Requirement T5 |
|---|---|
| 1 (DnD Forge) | N/A — pas concerne T5 (pas de donnees sensibles) |
| 2 (TransLingua) | N/A — population non-cible T5 typiquement |
| 3 (PolitiScan) | partielle : encryption at rest + Tor optionnel + warrant canary |
| 4 (LibanLive) | **TOUS les items §7 + audit + ONG + formation + beta 18m+ + ethics board + plan rollback** |

## 10. Fork "Profile B" et sister-project

Le kickoff Sprint 17 (§6 scope cuts) mentionne explicitement que
LibanLive necessitera probablement un **sister-project distinct**
— un fork hardening-first qui va plus loin que SBFB mainline.
Raisons :
- Overhead UX acceptable pour contributeur forme, inacceptable pour
  DnD Forge.
- Risque legal different (LibanLive = conflict zone, SBFB = hobby).
- Governance differente (ethics review board, ONG partners).

Decision formelle Sprint 30+.

## 11. Ethique

**T5 impose une responsabilite morale** qu'aucun autre tier n'a.
Deployer un outil pour population cible sans le durcissement
requis n'est pas "MVP" — c'est complicite de harm. Sprint 17
Phase E + Phase D codifient ce point dans le gate system pour
que cette responsabilite soit structurelle, pas optionnelle.

## 12. References

- Amnesty International Security Lab (technical reports Pegasus)
- Citizen Lab Toronto (Predator, Quadream documentation)
- EFF Surveillance Self-Defense — High-Risk Users
- Bruce Schneier "Click Here to Kill Everybody" (2018)
- Jenny Fan & Roger Dingledine — "Tor Against Nation-State
  Adversaries" (2024 USENIX)
- Human Rights Data Analysis Group — documenting risks
- Signal Blog — "Protecting journalists in conflict zones" (2024)
- NSO Group transparency reports (internal, partiellement leaked)
- Access Now Digital Security Helpline — case studies 2020-2025
