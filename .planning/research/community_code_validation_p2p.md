# Community Code Validation in P2P/Decentralized Contexts

**Date** : 2026-05-10
**Contexte** : SBFB Ideas Hub — apres qu'un worker AI genere du
code pour une idee votee par la communaute, comment le reseau
valide ce code avant deployment, sans autorite centrale.
**Confiance globale** : MEDIUM-HIGH (sources primaires verifiees
pour la majorite des mecanismes).

---

## 1. Revue de code decentralisee

### 1.1 Radicle — le modele de reference

Radicle (https://radicle.dev) est le projet le plus avance sur la
revue de code P2P. Son architecture repose sur trois primitives :

**Delegates et threshold** : Chaque depot Radicle possede un
*identity document* (JSON canonique) qui definit :
- Les DIDs (`did:key`) des *delegates* (mainteneurs)
- Un *threshold* : nombre minimum de signatures delegate requises
  pour qu'un commit sur la branche par defaut devienne canonique

Exemple : si threshold=2 et 3 delegates existent, il faut que 2
des 3 delegates poussent le meme commit sur `master` pour qu'il
devienne l'etat canonique du depot. Aucun serveur central ne
decide — la convergence est deterministe.

**Source** : https://radicle.xyz/2025/08/12/canonical-references
(Radicle 1.3.0, aout 2025)

**Canonical References (v1.3.0+)** : Extension du modele delegate
qui permet des regles par reference Git. Le document identite
contient des regles `xyz.radicle.crefs` qui specifient :
- Un pattern de reference Git (ex: `refs/heads/release/*`)
- Un ensemble de DIDs autorises
- Un threshold par pattern

Cela permet des politiques differenciees : threshold 2/3 pour
`master`, threshold 1/1 pour `feature/*`, threshold 3/3 pour
`release/*`.

**Patches (PRs decentralisees)** : Les patches Radicle sont des
*Collaborative Objects* (CRDTs) stockees dans le depot Git
lui-meme. Un contributeur soumet un patch, les delegates le
reviewent, et le merge ne se produit que quand le threshold est
atteint. La review est asynchrone et fonctionne offline.

**Pertinence SBFB** : Le modele delegates + threshold de Radicle
est directement transposable au systeme curator SBFB. Les curator
lists Ed25519 signees sont l'equivalent des delegates Radicle, et
un threshold configurable par projet est le chemin naturel.

### 1.2 ForgeFed — federation inter-forges

ForgeFed (https://forgefed.org) definit un protocole ActivityPub
etendu pour la federation de forges (Forgejo, Gitea, GitLab). Le
vocabulaire couvre `Repository`, `Commit`, `Branch`, `Ticket`,
`Review` comme des objets ActivityPub.

**Etat 2026** : La federation Forgejo fonctionne pour les
"stars" inter-instances. Le support des pull requests et code
reviews federees est en cours mais pas encore production-ready.
Les mecanismes de moderation et controle d'acces sont absents.

**Source** : https://codeberg.org/ForgeFed/forgefed

**Pertinence SBFB** : Faible a court terme. ForgeFed vise
l'interoperabilite entre forges centralisees, pas un modele P2P
pur. SBFB n'a pas besoin de federer avec Gitea/GitLab — le
modele verified deploy depuis repo Git + curator validation est
plus simple et plus sur.

### 1.3 Mecanisme recommande pour SBFB

Adapter le modele Radicle : chaque projet deploye sur SBFB
definit dans son `SBFB.json` un ensemble de curators autorises
(Ed25519 pubkeys) et un threshold. Le code AI-genere suit le
chemin :

```
Worker AI genere le code
  |
  v
Code pousse sur un repo public (verified deploy prerequis)
  |
  v
N curators reviewent + signent une approbation
  |
  v
Quand threshold atteint, le coordinateur autorise le deploy
```

---

## 2. Validation par curators de confiance (package managers)

### 2.1 F-Droid — build reproductible + review manuelle

F-Droid (https://f-droid.org) est le modele le plus proche de SBFB
pour la validation d'apps open source.

**Inclusion Policy** : Chaque app doit etre :
- 100% FLOSS (licence reconnue DFSG/FSF/GNU/OSI)
- Buildable avec une toolchain 100% FLOSS (Debian-packaged tools)
- Sans tracking/advertising proprietaire
- Sans telechargement de binaires additionnels sans consentement
- Fonctionnelle et maintenue activement

**Source** : https://f-droid.org/en/docs/Inclusion_Policy/

**Process de review** : Les changements de metadata passent par
des merge requests reviewees manuellement par des contributors
F-Droid. Les reviewers verifient :
- La coherence entre metadata et code source
- La licence (copyright notices, README)
- Les anti-features (tracking, ads, dependances non-free)
- La buildabilite

**Signing** : Deux niveaux distincts :
1. **Index du repo** : signe par une cle RSA 2048-bit du repo
2. **APK individuels** : signes par des cles per-app generees
   automatiquement + option PGP via `fdroid gpgsign`

**Source** : https://f-droid.org/en/docs/Signing_Process/

**Builds reproductibles** : F-Droid peut verifier qu'un APK
rebuild localement est bit-for-bit identique a celui du
developpeur upstream. Cela utilise le modele "Diverse Double-
Compiling" — deux toolchains independantes produisant des
binaires identiques.

**Source** : https://f-droid.org/en/docs/Reproducible_Builds/

**Limite** : F-Droid n'a PAS de multi-signature pour les apps.
Un seul maintainer signe l'index. Le trust repose sur la
confiance dans le process de review + la reproductibilite des
builds, pas sur un quorum cryptographique.

### 2.2 Debian — chaine de confiance GPG + NEW queue

Debian utilise un modele de confiance en couches :

**SecureApt** : La chaine de verification va du package installe
jusqu'au fournisseur :
- Le fichier `Release` du repo est signe GPG (InRelease = inline,
  Release.gpg = detache)
- `Release` contient les checksums de `Packages`
- `Packages` contient les checksums des packages individuels
- Les cles de confiance sont dans `/etc/apt/trusted.gpg.d/`

**Source** : https://wiki.debian.org/SecureApt

**Point cle** : Debian signe le REPO, pas les packages
individuels. Le trust est delegue a l'infrastructure du repo.

**NEW queue** : Les nouveaux packages passent par la queue NEW
reviewee par l'equipe FTP Master (separee de l'equipe Archive
Operations depuis 2026). Les criteres :
- Conformite DFSG (peut etre distribue par Debian)
- Correction basique (pas de test complet)
- Licence appropriee
- Qualite minimale

**Source** : https://wiki.debian.org/NewQueue

**Sponsorship** : Les contributeurs sans droits d'upload passent
par un sponsor (Debian Developer) qui review, build, teste et
uploade. Le sponsor est le garant humain de la qualite.

**Source** : https://mentors.debian.net/intro-maintainers/

**Pertinence SBFB** : Le modele sponsor de Debian mappe
directement sur le role curator SBFB. Un curator = un sponsor qui
vouch pour un projet. La difference : SBFB peut exiger un quorum
de curators (N sponsors), la ou Debian n'en exige qu'un.

### 2.3 Flathub — review humaine + sandbox + verification

Flathub (https://flathub.org) combine :
- Review humaine par des reviewers volontaires (merge requests)
- Sandbox Flatpak (permissions explicites)
- Verification de l'identite du developpeur
- Rejet possible a tout moment (pre ou post-merge)

**Source** : https://docs.flathub.org/docs/for-app-authors/submission

**Pertinence SBFB** : Le modele Flathub de "verification +
sandbox + review" est exactement le triangle de confiance SBFB :
verified deploy (verification) + iframe sandbox (isolation) +
curator validation (review).

### 2.4 Homebrew — review centralisee avec trust tiers

Homebrew separe formulae (compile sur infra Homebrew, verifie par
checksums) et casks (binaires pre-compiles, verification limitee).
Les maintainers utilisent "approve", "approve with comments",
"request changes" sur les PRs.

**Point notable** : Les casks SHA256 ne sont PAS consideres comme
une mesure de securite fiable par Homebrew eux-memes — si un
attaquant compromet l'URL, il change aussi le hash. C'est un
avertissement pertinent pour SBFB : le hash seul ne suffit pas,
la provenance (d'ou vient le code) est critique.

**Source** : https://docs.brew.sh/Maintainer-Guidelines

---

## 3. Quality gates automatisees en contexte P2P

### 3.1 Radicle CI — le premier CI decentralise

Radicle CI (https://radicle-ci.liw.fi) est le systeme le plus
avance pour du CI decentralise sur un reseau P2P.

**Architecture broker/adapter** : Un broker ecoute les evenements
du noeud Radicle et lance un adapter pour executer le CI. Les
adapters existants :
- **Native** : execute un script shell localement
- **Ambient** (janvier 2025) : CI standalone
- **Concourse/Kraken** : integration CI tiers
- **Webhook generique** : integration avec tout CI externe

**Source** : https://blog.liw.fi/posts/2025/radicle-ci-status-quo/

**Reporting via COBs** : Les resultats CI sont stockes comme des
*Collaborative Objects* (CRDTs) dans le depot. Chaque resultat
porte : quel noeud a run le CI, pour quel commit, succes/echec.
Le desktop Radicle affiche le statut CI directement.

**Decentralisation** : "Any Radicle node can choose to run CI for
any repository it has access to. Any project using Radicle can
choose which CI nodes it trusts." Chaque noeud decide
independamment quels projets il CI-teste.

**Source** : https://radicle.dev/2025/07/23/using-radicle-ci-for-development

**`rad ci`** : Outil CLI pour reproduire le CI localement. Le
developpeur n'attend pas un noeud CI distant — il peut runner
localement et poster le resultat.

**Pertinence SBFB** : Le modele Radicle CI est directement
applicable. Les workers SBFB qui re-buildent et testent le code
AI-genere sont l'equivalent des noeuds CI Radicle. Le resultat
signe (build OK/KO + hash artefact) est un attestation
distribuee.

### 3.2 Builds reproductibles — Guix `challenge` + Lila

**GNU Guix `guix challenge`** : Commande qui compare les resultats
de build locaux contre ceux des build farms distantes. Si les
binaires ne sont pas bit-for-bit identiques, le build n'est pas
reproductible — signal d'alerte.

Guix maintient plusieurs build farms independantes (hardware
different, kernel different) qui ne telechargent pas de binaires
entre elles. La commande `guix challenge --diff=diffoscope`
automatise la comparaison.

**Source** : https://guix.gnu.org/manual/en/html_node/Invoking-guix-challenge.html

**Lila (2026)** : Systeme decentralise de monitoring de
reproductibilite pour Nix/Guix. Papier presente a MSR'26 (Mining
Software Repositories, avril 2026).

Mecanisme :
1. Chaque builder rapporte automatiquement via un post-build hook
2. L'attestation contient : hash de la recette + hashes des
   outputs, signee par la cle du builder
3. Les attestations sont aggregees dans une base de reproductibilite
4. N'importe qui peut comparer les attestations de multiples
   builders independants

**Source** : https://arxiv.org/html/2601.20662v1

**Modele de confiance** : L'authentification (token API) est
separee de la verification (signature cryptographique des
attestations). Un builder malveillant ne peut pas falsifier les
attestations d'un autre builder.

**Resultats** : Nix atteint >90% de reproductibilite sur 80 000+
packages, prouvant que la reproductibilite a grande echelle est
techniquement realisable.

**Pertinence SBFB** : Le modele Lila est le plus pertinent pour
SBFB. Chaque worker qui re-build une app AI-generee produit une
attestation signee. Si N workers independants produisent le meme
hash, le code est valide. C'est exactement le pattern
`PUBLISH_MODEL.md §6` ("SBFB self-build quorum — N builders
independants, SHA256 consensus, attestation signee").

### 3.3 Checks automatises realisables sans CI central

| Check | Faisabilite P2P | Comment |
|---|---|---|
| **Lint (eslint, clippy)** | Haute | Deterministe, chaque worker peut runner |
| **Tests unitaires** | Haute | Deterministe si les tests sont inclus |
| **Build reproductible** | Haute | Hash du zip = consensus multi-worker |
| **Security scan (npm audit, cargo audit)** | Moyenne | Base de donnees advisories requise (cacheable) |
| **SAST (semgrep, CodeQL)** | Moyenne | Patterns locaux, pas de cloud requis |
| **Taille artefact** | Haute | Verification triviale per-worker |
| **Signature provenance** | Haute | SLSA L1 deja en place SBFB |
| **Dependencies check** | Moyenne | Lockfile + hashes verifiables localement |

---

## 4. Quorum et modeles de threshold

### 4.1 Modeles existants dans l'ecosysteme

| Systeme | Modele | Threshold | Notes |
|---|---|---|---|
| **Radicle** | K-of-N delegates | Configurable par projet | Minimum = 1, recommande >= 2 |
| **TUF** | Threshold per role | Configurable, multi-niveau | Root, Targets, Timestamp, Snapshot |
| **FROST** | K-of-N threshold sig | Cryptographique | Compatible Ed25519, sig = standard |
| **Debian** | 1 sponsor + FTP Master | Effectivement 2 approvals | Non-cryptographique |
| **F-Droid** | 1 reviewer + build bot | Effectivement 1.5 | Semi-automatise |
| **BFT consensus** | 2f+1 of 3f+1 | Tolere f noeuds malveillants | Trop lourd pour du code review |

### 4.2 The Update Framework (TUF) — le standard

TUF (https://theupdateframework.io) definit un framework de
securite pour les mises a jour logicielles avec un systeme de
roles et delegations :

**Root** : Delegue la confiance aux roles de premier niveau,
specifie les cles et thresholds pour chaque role.

**Targets** : Signe les metadata des fichiers distribues (hashes,
tailles). Peut deleguer a des sous-roles avec leurs propres cles
et thresholds + patterns glob sur les fichiers.

**Expiry** : Tous les metadata ont une date d'expiration.
Les clients refusent les metadata plus vieux que ceux deja vus
(rollback protection). Pas besoin de revocation explicite — les
metadata expirent naturellement.

**Source** : https://theupdateframework.io/docs/metadata/

**Pertinence SBFB** : Le modele TUF est le plus adapte pour
structurer la confiance curator SBFB. Un curator "root" delegue
a des sous-curators par categorie d'apps, chacun avec son
threshold. L'expiry native evite le probleme de revocation (voir
section 5).

### 4.3 FROST — threshold cryptographique pour Ed25519

FROST (RFC 9591, IETF janvier 2025) permet de splitter une cle
Ed25519 en N shares avec un threshold K. K participants
collaborent pour produire une signature standard Ed25519 —
indistinguable d'une signature single-key.

**Compatibilite SBFB** : SBFB utilise deja FROST pour le warrant
canary (`crates/nexus-shell-daemon-core/src/canary/frost.rs`).
La meme primitive peut signer les approbations de code.

**iroh + FROST** : Le blog iroh (https://www.iroh.computer/blog/
frost-threshold-signatures) documente l'integration FROST avec
Ed25519 pour le key management iroh, incluant le split de cles
existantes.

**Source** : https://github.com/ZcashFoundation/frost

### 4.4 Recommandation threshold pour SBFB

Trois niveaux adaptes a la maturite du reseau :

| Phase reseau | Threshold | Justification |
|---|---|---|
| **Pre-v1.0** (maintenant) | 1/1 | Un seul mainteneur, pas de quorum possible |
| **Post-v1.0 early** (<50 curators) | 2/3 | Quorum simple, tolere 1 curator indisponible |
| **Mature** (>50 curators) | ceil(N/2)+1 ou ponderation Kudos | Majorite simple + reputation |

**Modele recommande** : Threshold configurable par projet (comme
Radicle), pas global. Un projet critique (infrastructure SBFB)
exige 3/5. Une app ludique exige 1/2. Le createur du projet
definit le threshold dans `SBFB.json`.

**Ponderation par reputation** : Les votes Kudos du systeme
existant peuvent ponderer les approbations curator. Un curator
avec Kudos 100 pese plus qu'un curator avec Kudos 5. Cela evite
la Sybil attack "creer 10 curators vides pour atteindre le
threshold".

Format suggere dans `SBFB.json` :

```json
{
  "validation": {
    "curators_required": ["did:key:z6Mk...", "did:key:z6Mn..."],
    "threshold": 2,
    "weighted": false
  }
}
```

---

## 5. Rollback et revocation

### 5.1 Le probleme fondamental en P2P

La revocation est le probleme le plus dur en P2P. Sans autorite
centrale, il n'y a pas de "bouton supprimer" universel.
Trois approches existent :

### 5.2 Expiry naturelle (TUF model)

**Mecanisme** : Tous les metadata ont une date d'expiration. Si
le curator ne re-signe pas, le metadata expire et les clients
refusent de l'accepter. Pas besoin de revocation explicite.

**Avantage** : Zero infrastructure de revocation. Le silence est
la revocation.

**Inconvenient** : Latence. Un package malveillant reste valide
jusqu'a expiry (typiquement 24h-30j selon le role TUF).

**Pertinence SBFB** : Directement applicable. Les curator lists
SBFB ont deja un `revision` monotone (rollback protection,
Sprint 7). Ajouter un `expires_at` ferait expirer naturellement
les approbations.

### 5.3 Curator blacklist (revocation active)

**Mecanisme** : Un curator publie une liste signee de hashes
d'artefacts revoques. Les noeuds du reseau refusent de servir
les artefacts blacklistes.

Equivalent dans l'ecosysteme :
- **IPNS** : Pointeur mutable vers un CID. Pour "revoquer",
  on met a jour le pointeur IPNS vers un contenu vide ou un
  avertissement. Mais IPNS n'a PAS de mecanisme de revocation
  de cle natif (issue ouverte : https://github.com/ipfs/specs/issues/219).
- **Certificate Revocation Lists (CRL)** : Modele PKI classique
  transpose au P2P. Chaque curator maintient une CRL signee.

**Implementation SBFB** : Le systeme de quarantine existant
(`crates/nexus-coordinator-rs/src/quarantine_queue.rs`) peut etre
etendu. Un curator qui detecte du code malveillant publie un
`CuratorRevocation` signe sur le gossip topic dedie. Les noeuds
qui font confiance a ce curator retirent l'app de leur browse
local.

**Format suggere** :

```rust
pub struct CuratorRevocation {
    pub curator_pubkey: [u8; 32],
    pub artifact_hash: Vec<u8>,     // hash iroh-blobs de l'artefact
    pub reason: RevocationReason,   // enum: malicious, vulnerable, stale
    pub revoked_at: i64,
    pub evidence_url: Option<String>, // lien vers le report/CVE
    pub signature: [u8; 64],
}
```

### 5.4 Warrant canary pour projets (dead-man switch)

SBFB a deja un warrant canary sophistique (FROST K-of-N,
`WARRANT_CANARY_HARDENING.md`). Le meme pattern peut s'appliquer
au niveau projet :

- Un mainteneur de projet publie un "canary projet" mensuel
  (signe, avec headline du jour)
- Si le canary expire (>45 jours), les curators retirent
  automatiquement le projet de leurs listes

Ce pattern adresse le cas "mainteneur compromis silencieusement"
— si le mainteneur ne peut plus publier librement, le projet
est automatiquement degrade.

### 5.5 Modele recommande pour SBFB

Combiner les trois couches :

```
Couche 1 : Expiry (TUF-like)
  - Approbations curator expirent apres T jours (defaut 90)
  - Le deploiement reste actif tant qu'au moins 1 curator
    re-approuve avant expiry

Couche 2 : Revocation active (CuratorRevocation)
  - N'importe quel curator peut publier une revocation signee
  - Les noeuds qui font confiance a ce curator retirent l'app
  - Threshold configurable : 1 revocation = warning,
    quorum revocations = retrait automatique

Couche 3 : Dead-man switch (optionnel)
  - Le createur du projet publie un canary periodique
  - Expiry = degradation automatique de confiance
```

---

## 6. Synthese — pipeline de validation recommande pour SBFB

### 6.1 Flux complet : idee votee -> code AI -> deployment

```
1. VOTE (Ideas Hub)
   - Idee proposee, votee par la communaute (Ed25519 + Kudos)

2. GENERATION (Worker AI)
   - Worker AI genere le code
   - Code pousse sur repo public (prerequis verified deploy)

3. AUTOMATED GATES
   - N workers independants re-buildent le code
   - Verification hash consensus (build reproductible)
   - Lint + tests + security scan automatiques
   - Resultats signes (attestations individuelles)
   - Seuil : 100% workers doivent produire meme hash

4. CURATOR REVIEW
   - Curators designes (ou volontaires) reviewent le code
   - Chaque curator signe une approbation (Ed25519)
   - Threshold configurable par projet (defaut 2/3)
   - Approbation = "j'ai review et le code fait ce qu'il dit"

5. DEPLOYMENT
   - Quand threshold atteint + automated gates verts :
     coordinateur autorise le verified deploy
   - Artefact signe, provenance SLSA L1, distribue iroh-blobs

6. MONITORING POST-DEPLOY
   - Curator revocations possibles a tout moment
   - Approbations expirent apres T jours (re-review periodique)
   - Dead-man switch optionnel du createur
```

### 6.2 Ce qui existe deja dans SBFB

| Brique | Status | Sprint |
|---|---|---|
| Curator lists Ed25519 signees | LIVRE | S7-S8 |
| Verified deploy (Git -> Keyoxide -> SLSA L1) | LIVRE | S14 |
| Quarantine queue | LIVRE | S21/S41 |
| FROST threshold signatures | LIVRE (canary) | S20 |
| Kudos reputation | LIVRE | S17 |
| DelegationCert node -> SSH key | DESIGN | S22 RFC |
| Multi-forge cross-validation | DESIGN | S22 RFC |
| Build quorum multi-worker | DESIGN | LT-7 |
| Key rotation + revocation cache | LIVRE | S24 |

### 6.3 Ce qui manque

| Brique manquante | Complexite | Prerequis |
|---|---|---|
| **CuratorApproval wire type** | Medium | Nouveau CRDT/gossip message type |
| **Threshold configurable par projet** | Medium | Extension SBFB.json |
| **CuratorRevocation wire type** | Medium | Nouveau gossip topic |
| **Expiry sur approbations** | Low | Champ `expires_at` dans CuratorApproval |
| **Automated build attestation** | High | LT-7 multi-builder quorum |
| **Ponderation Kudos sur votes** | Medium | Bridge `kudos_score` + algo |
| **Gossip topic revocations** | Low | Topic seed + subscribe |

---

## 7. Risques et mitigations specifiques au code AI-genere

| Risque | Gravite | Mitigation |
|---|---|---|
| **Code AI backdoored** (prompt injection -> code malveillant) | Critique | Automated security scan + curator review obligatoire |
| **Code AI subtilment mauvais** (fonctionne mais vol de donnees via bridge) | Critique | Whitelist bridge stricte (3+5 methodes seulement), sandbox iframe |
| **Sybil curators** (attaquant cree N curators) | Haute | Ponderation Kudos (reputation non-transferable) |
| **Curator fatigue** (trop de code a reviewer) | Moyenne | Automated gates filtrent avant review humaine |
| **Code AI non-reproductible** (meme prompt -> outputs differents) | Basse | Hash consensus multi-worker filtre les non-determinismes |
| **Fork malveillant** (modifie le code apres approval) | Critique | Hash iroh-blobs immutable, provenance lie commit SHA |

---

## 8. Sources

### Sources primaires (HAUTE confiance)

- Radicle Protocol Guide : https://radicle.dev/guides/protocol
- Radicle Canonical References : https://radicle.xyz/2025/08/12/canonical-references
- Radicle CI status : https://blog.liw.fi/posts/2025/radicle-ci-status-quo/
- F-Droid Inclusion Policy : https://f-droid.org/en/docs/Inclusion_Policy/
- F-Droid Signing Process : https://f-droid.org/en/docs/Signing_Process/
- F-Droid Reproducible Builds : https://f-droid.org/en/docs/Reproducible_Builds/
- TUF Roles and Metadata : https://theupdateframework.io/docs/metadata/
- Debian SecureApt : https://wiki.debian.org/SecureApt
- Debian NEW Queue : https://wiki.debian.org/NewQueue
- Debian Sponsorship : https://mentors.debian.net/intro-maintainers/
- FROST RFC 9591 : https://datatracker.ietf.org/doc/rfc9591/
- frost-ed25519 (Zcash Foundation) : https://github.com/ZcashFoundation/frost
- iroh FROST blog : https://www.iroh.computer/blog/frost-threshold-signatures
- GNU Guix challenge : https://guix.gnu.org/manual/en/html_node/Invoking-guix-challenge.html
- Flathub Submission : https://docs.flathub.org/docs/for-app-authors/submission
- IPNS Key Revocation Issue : https://github.com/ipfs/specs/issues/219

### Sources secondaires (MOYENNE confiance)

- Lila paper (MSR'26) : https://arxiv.org/html/2601.20662v1
- ForgeFed spec : https://codeberg.org/ForgeFed/forgefed
- Homebrew Maintainer Guidelines : https://docs.brew.sh/Maintainer-Guidelines
- Sigstore/Cosign : https://github.com/sigstore/cosign
- FOSDEM 2026 Radicle talk : https://fosdem.org/2026/schedule/event/TMQZTP-radicle/

### Sources internes SBFB

- `docs/security/CONTRIBUTOR_ATTESTATION_RFC.md` — Couche 3 multi-forge
- `docs/security/WARRANT_CANARY_HARDENING.md` — FROST canary
- `docs/architecture/PUBLISH_MODEL.md` — 4 etats publication
- `crates/nexus-core-rs/src/curator.rs` — CuratorList wire type
- `crates/nexus-coordinator-rs/src/quarantine_queue.rs` — Quarantine
- `.planning/research/pre_v1_apps_protocol_explorer_ideas_hub.md` — Ideas Hub design
