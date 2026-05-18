# S65 Contrat Public — Recherche exhaustive

**Date** : 2026-05-18
**Mode** : Ecosystem + Feasibility
**Confiance globale** : HIGH (base de code lue integraleme, sources externes verifiees)

---

## 1. Resume executif

SBFB utilise actuellement un vocabulaire de confiance imprecis
qui sur-promet par rapport a ce que le code garantit reellement.
Le badge "Verifie" (Browse, BrowsedProject) s'affiche des qu'un
`provenance_hash` **existe** dans les donnees reseau, sans
verification live de la signature Ed25519. Le label "open source
verifie" (GpuConsentDialog L2, Protocol Explorer, PUBLISH_MODEL)
melange trois concepts distincts : la lisibilite du code source,
la verification de build, et la provenance cryptographique. Le
mot "confiance" apparait dans l'UI Curators sans qualifier ce
qu'un curator vouch represente. Les implications AGPL sont
absentes de l'UI publique.

Ce sprint doit construire un **contrat public** : une taxonomie
claire des niveaux de confiance SBFB, et migrer tous les textes
UI pour que chaque badge/label exprime exactement ce que le code
prouve.

---

## 2. Inventaire exhaustif des textes de confiance

### 2.1 Browse.tsx — Page explorer

| Emplacement | Wording exact | Condition d'affichage | Promesse percue | Garantie reelle | Gap |
|---|---|---|---|---|---|
| HeroSection pill | `"Archive P2P"` | `entry.archive_hash` existe | App distribuee P2P | Le hash d'archive existe dans les donnees browse | **MINIMAL** — texte factuel |
| AppCard badge | `"Verifie"` + icone ShieldCheck | `entry.provenance_hash` existe | L'app a ete verifiee cryptographiquement | Un hash de provenance est **annonce** dans le reseau — pas de verification live | **CRITIQUE** — promesse >> garantie |
| AppCard badge | `"P2P"` | `entry.archive_hash` existe | App distribuee sur le reseau P2P | Hash archive present dans donnees browse | **MINIMAL** |
| AppCard badge | `"Source"` + lien | `entry.repo_url` existe | Code source lisible | Un URL de repo est annonce — pas de verification d'accessibilite | **FAIBLE** — URL pourrait etre mort |
| AppCard badge | `"Auto-publie"` | `entry.source === "direct"` | Publie par le noeud lui-meme | Publie sans passer par un curator | **MINIMAL** |

### 2.2 BrowsedProject.tsx — Vue immersive

| Emplacement | Wording exact | Condition d'affichage | Promesse percue | Garantie reelle | Gap |
|---|---|---|---|---|---|
| Top bar badge | `"Verifie"` + ShieldCheck | `entry.provenance_hash` existe | Signature Ed25519 validee | Provenance hash **existe** — verification live non faite | **CRITIQUE** |
| Top bar badge | `"blob:{hash}"` | `entry.archive_hash` existe | Hash de l'archive iroh | Hash affiche — factuel | **AUCUN** |
| Top bar badge | `"Source"` + lien | `entry.repo_url` existe | Code lisible | URL annonce | **FAIBLE** |
| Top bar badge | `"sandbox"` + Shield | toujours | Protection sandbox | Vrai — sandbox="allow-scripts" sans allow-same-origin | **AUCUN** |
| Top bar badge | `"Auto-publie"` | `entry.source === "direct"` | Publie directement | Factuel | **AUCUN** |
| GPU button | `"Contribuer mon GPU"` | consent L3 | Contribution GPU manuelle | Factuel | **AUCUN** |

### 2.3 VerificationDetail.tsx — Dialog de verification

| Emplacement | Wording exact | Condition d'affichage | Promesse percue | Garantie reelle | Gap |
|---|---|---|---|---|---|
| Dialog title | `"Details de verification"` | ouverture dialog | Verification en cours | API appelee, verification reelle | **AUCUN** |
| Dialog desc | `"Provenance et integrite du deploiement verifie"` | toujours | Deploy verifie = integrite confirmee | Verification API **reelle** (Ed25519) | **FAIBLE** — wording pre-suppose le resultat |
| Badge resultat | `"Signature valide"` + ShieldCheck vert | `verified && !hashMismatch` | Signature cryptographique validee | Ed25519 verifie par le coordinator | **AUCUN** — c'est correct |
| Badge resultat | `"Signature invalide"` + ShieldX rouge | `!verified` | Signature echouee | Verification Ed25519 echouee | **AUCUN** |
| Badge resultat | `"Hash de provenance incoherent"` + ShieldX | `hashMismatch` | Hash annonce != hash retourne | Comparaison provenance_hash | **AUCUN** |
| Warning | `"Le hash de provenance retourne ne correspond pas au hash annonce dans le reseau"` | hashMismatch | Integrite douteuse | Comparison technique exacte | **AUCUN** |

**Constat** : le dialog de verification est **bien fait** — il
affiche le resultat reel. Le probleme est que le badge "Verifie"
affiche **avant** d'ouvrir ce dialog, sans avoir fait la
verification.

### 2.4 GpuConsentDialog.tsx — Consent GPU

| Emplacement | Wording exact | Condition d'affichage | Promesse percue | Garantie reelle | Gap |
|---|---|---|---|---|---|
| L2 title | `"Projets open source verifies"` | option L2 | Apps verifiees open source | Worker accepte si `is_open_source=true` dans la task | **MODERE** — "verifie" ici veut dire "deploy-from-repo" mais ca n'est pas explicite |
| L2 hint | `"Accepte les apps publiees depuis un depot Git public et signees"` | option L2 | Signature validee | Le flag `is_open_source` est set par le coordinator au deploy | **FAIBLE** — techniquement exact mais simplifie |
| L2 threat | `"Apps open source verifiees (SLSA L1). Exposition Sybil si contributeur malveillant."` | tooltip L2 | SLSA L1 verifie | Provenance SLSA L1 existe — pas SLSA L2/L3 | **MODERE** — SLSA L1 signifie "provenance exists" et "trivial to bypass or forge" selon la spec officielle. L'utilisateur comprend plus de securite que ce qui est garanti. |

### 2.5 Network.tsx — Consent badges

| Emplacement | Wording exact | Condition | Promesse | Garantie | Gap |
|---|---|---|---|---|---|
| Consent badge | `"L2 — Open source"` | consent level 2 | Open source verifie | Flag `is_open_source` | **MODERE** — raccourci |

### 2.6 Deploy.tsx — Page deploiement

| Emplacement | Wording exact | Condition | Promesse | Garantie | Gap |
|---|---|---|---|---|---|
| Subtitle | `"Clone un depot Git public, verifie l'identite, et publie l'app sur le reseau P2P"` | toujours | Verification d'identite | Verifie que SBFB.json node_id match le daemon local | **FAIBLE** — "identite" est vague, mais techniquement correct |
| Success | `"App deployee"` + hash + provenance + commit | deploy reussi | Deploy verifie | Provenance SLSA L1 generee | **AUCUN** |

### 2.7 Curators.tsx — Page curators

| Emplacement | Wording exact | Condition | Promesse | Garantie | Gap |
|---|---|---|---|---|---|
| Page desc | `"Abonne-toi a des listes signees Ed25519"` | toujours | Listes cryptographiquement signees | Ed25519 verifie cote daemon | **AUCUN** |
| Add form | `"Colle la cle publique Ed25519 d'un curator de confiance"` | toujours | Curator de confiance | L'utilisateur decide de faire confiance — le systeme ne valide rien de plus que la signature | **MODERE** — "de confiance" sur-promet |
| Status | `"X projet(s) vouche(s)"` | curator actif | Projets cautionnes | Le curator les a listes | **FAIBLE** — "vouche" est correct mais pas defini formellement |

### 2.8 KudosTab.tsx — Integrite hash-chain

| Emplacement | Wording exact | Condition | Promesse | Garantie | Gap |
|---|---|---|---|---|---|
| Badge valide | `"Hash chain valide"` + `"Toutes les entrees sont signees et liees sans modification detectable"` | hash-chain valide | Integrite verifiee | BLAKE3 hash-chain recalculee et validee | **AUCUN** — excellent wording |
| Badge invalide | `"Hash chain corrompue"` + `"Ce registre ne doit pas etre considere comme de confiance"` | hash-chain invalide | Registre corrompu | Hash-chain broken | **AUCUN** — excellent wording |

### 2.9 ProjectDetail.tsx — Detail projet

| Emplacement | Wording exact | Condition | Promesse | Garantie | Gap |
|---|---|---|---|---|---|
| Badge | `"Public"` / `"Prive"` | visibility field | Visibilite du projet | Flag visibility | **AUCUN** |

### 2.10 Protocol Explorer (sbfb-explorer)

| Emplacement | Wording exact | Promesse percue | Garantie reelle | Gap |
|---|---|---|---|---|
| App lifecycle §2 | `"Le code sur le reseau = le code du depot"` | Egalite bit-a-bit | Le coordinator clone, build, et signe — mais c'est le **meme noeud** qui clone et signe. Pas de build reproductible multi-party. | **CRITIQUE** — phrase trop forte |
| Security §4 | `"Deploy verifie"` + `"Le coordinator verifie l'identite du noeud et signe un enregistrement de provenance SLSA L1 Ed25519"` | Verification d'identite forte | node_id match + Ed25519 self-sign | **MODERE** — "verifie l'identite" est vague |
| Security §4 | `"Le code sur le reseau correspond au code du depot"` | Egalite garantie | Meme chose que ci-dessus — self-attestation | **CRITIQUE** — repete la sur-promesse |
| Security §4 | `"Resistance Sybil"` + `"Un noeud sans historique de contribution verifie ne peut pas influencer le reseau"` | Protection Sybil forte | Age witness + PoW + contributor attestation existent — mais Age witness fallback PoW-only pre-v1.0, pas encore en prod | **MODERE** |
| Philosophy §5 | `"Open source par construction"` + `"Le modele F-Droid/Linux applique aux apps web P2P"` | F-Droid-level verification | SBFB n'a pas de builds reproductibles multi-builder. F-Droid a des rebuilders independants. | **CRITIQUE** — comparaison sur-promettante |
| Philosophy §5 | `"Decentralisation reelle"` + `"Pas de noeud bootstrap privilegie"` | Aucun noeud privilegie | Pre-v1.0 : bootstrap allowlist existe pour l'admission age witness | **MODERE** |
| Verification §6 | `"Chaine de preuve"` + diagramme | Chaine de preuve complete | Self-attestation par le coordinator local | **MODERE** — "preuve" implique tiers independant |
| Footer | `"une app open source deployee sur le reseau SBFB"` | App open source | Factuel — le code est sur GitHub | **AUCUN** |

### 2.11 Ideas Hub (sbfb-ideas)

| Emplacement | Wording exact | Promesse | Garantie | Gap |
|---|---|---|---|---|
| Footer | `"une app open source deployee sur le reseau SBFB"` | App open source | Code sur GitHub — factuel | **AUCUN** |

---

## 3. Analyse des gaps — synthese par severite

### 3.1 CRITIQUE — doit etre corrige S65

| # | Gap | Localisation | Probleme |
|---|---|---|---|
| G1 | Badge "Verifie" sans verification | Browse.tsx:259, BrowsedProject.tsx:281 | Le badge s'affiche a la **presence** d'un hash, pas apres verification Ed25519. L'utilisateur croit que la signature a ete validee. C'est le carry **P2-BADGE-WORDING-PREMATURE** depuis S14. |
| G2 | "Le code sur le reseau = le code du depot" | sbfb-explorer index.html:144-145, :301-302 | C'est une **self-attestation** : le meme noeud clone et signe. Pas de build reproductible. Pas de verification tiers. Pas d'egalite bit-a-bit garantie. |
| G3 | Comparaison F-Droid/Linux | sbfb-explorer index.html:418 | F-Droid a des rebuilders independants. SBFB a un self-build single-node. Le modele n'est pas le meme. |

### 3.2 MODERE — devrait etre corrige S65

| # | Gap | Localisation | Probleme |
|---|---|---|---|
| G4 | "SLSA L1" utilise comme signal de securite forte | GpuConsentDialog.tsx:70 | SLSA L1 signifie officiellement "provenance exists, trivial to bypass or forge". C'est un niveau de documentation, pas de securite. |
| G5 | "curator de confiance" | Curators.tsx:144 | Le systeme ne valide rien au-dela de la signature Ed25519 du curator. "de confiance" est un jugement que l'UI ne devrait pas pre-supposer. |
| G6 | "verifie l'identite" deploy | sbfb-explorer index.html, Deploy.tsx | Verifie que node_id match — c'est une auto-declaration, pas une verification d'identite au sens PKI/WebPKI. |
| G7 | "Resistance Sybil" sans nuance | sbfb-explorer index.html:289-295 | Les mecanismes existent mais ne sont pas tous actifs en prod. |
| G8 | "Chaine de preuve" | sbfb-explorer index.html:338 | "Preuve" implique verification independante. C'est une self-attestation signee. |
| G9 | "Decentralisation reelle" / "pas de noeud privilegie" | sbfb-explorer index.html:431 | Bootstrap allowlist existe. |

### 3.3 FAIBLE — a clarifier si possible

| # | Gap | Localisation | Probleme |
|---|---|---|---|
| G10 | "Source" link sans check accessibilite | Browse.tsx:263 | URL pourrait etre mort |
| G11 | "Provenance et integrite du deploiement verifie" pre-suppose | VerificationDetail.tsx:129 | Wording assume succes avant resultat |
| G12 | "L2 — Open source" raccourci | Network.tsx:355 | "Open source" != "open source verifie" |

---

## 4. Recherche externe — vocabulaire de confiance ecosysteme

### 4.1 F-Droid (reference directe — le projet cite F-Droid)

F-Droid a publie en mai 2025 "Making Reproducible Builds Visible"
qui aborde exactement le probleme SBFB :

**Enseignements cles :**
- Les badges sont "une expression de quiconque controle le site web" — ils necessitent de **faire confiance a une personne**, ce qui contredit le principe des builds reproductibles.
- F-Droid recommande de presenter la verification comme **une action que l'utilisateur peut faire** plutot qu'un fait accompli (passer de "quelqu'un a verifie ceci" a "vous pouvez verifier ceci").
- F-Droid utilise deux symboles simples : un "check vert" signifie "notre rebuilder a reproduit le meme APK" et un "coeur brise" signifie "n'a pas reproduit". Pas de badge generique "verifie".
- Distinction nette entre : (1) source lisible, (2) build F-Droid depuis source, (3) build reproductible confirme par rebuilder independant.

**Application SBFB** : le badge "Verifie" de Browse devrait dire
"Provenance disponible" et le click devrait mener a l'action de
verification (ce que VerificationDetail fait deja bien). Le badge
ne devrait afficher "Signature validee" **qu'apres** verification
reelle.

Sources :
- [Making reproducible builds visible](https://f-droid.org/2025/05/21/making-reproducible-builds-visible.html)
- [F-Droid Verified](https://verification.f-droid.org/verified.html)

### 4.2 SLSA (Supply-chain Levels for Software Artifacts)

La spec SLSA v1.0 definit 4 niveaux :

| Niveau | Signification officielle | Securite reelle |
|---|---|---|
| **L0** | Aucune exigence | Neant |
| **L1** | La provenance **existe**. Peut prevenir les erreurs mais est **triviale a contourner ou falsifier**. | Documentation, pas securite |
| **L2** | Falsifier la provenance necessite une "attaque" explicite (mais peut etre facile). Build hosted + provenance signee. | Dissuade adversaires non-sophistiques |
| **L3** | Falsifier necessite d'exploiter une vulnerabilite au-dela des capacites de la plupart des adversaires. Build isole. | Securite forte |

**SBFB est a SLSA L1** : le coordinator genere la provenance et la
signe. C'est le meme acteur qui build et qui atteste. Il n'y a pas
de plateforme de build hosted separee (L2), ni d'isolation de build
(L3).

**Implication** : ecrire "SLSA L1" dans un tooltip de securite est
**techniquement exact** mais donne une fausse impression de
securite. Le label devrait dire "provenance auto-attestee" ou
"provenance SLSA L1 (auto-attestation)".

Source : [SLSA Security Levels](https://slsa.dev/spec/v1.0/levels)

### 4.3 Sigstore / in-toto

Sigstore distingue clairement :
- **Signing** = signature cryptographique (qui a signe)
- **Attestation** = declaration structuree (quoi a ete fait)
- **Provenance** = evidence de comment et ou le logiciel a ete build
- **Verification** = action de recalculer et confirmer

SBFB utilise "verification" pour deux choses distinctes :
(1) le process de deploy (le coordinator "verifie" le node_id) et
(2) la verification live de la signature Ed25519 par l'utilisateur.
Il faut distinguer ces deux sens.

Sources :
- [Sigstore overview](https://docs.sigstore.dev/cosign/signing/overview/)
- [Sigstore In-Toto Attestations](https://docs.sigstore.dev/cosign/verifying/attestation/)

### 4.4 AGPL-3.0 / copyleft

L'AGPL-3.0 garantit que :
- Le code source **doit** etre disponible pour quiconque interagit avec le logiciel via le reseau.
- Les modifications **doivent** etre partagees sous la meme licence.

Ce que l'AGPL ne garantit **pas** :
- Le code ne sera pas utilise commercialement (l'utilisation commerciale est autorisee).
- Le code ne sera pas forke (forker est explicitement permis).
- Le projet restera maintenu.
- La qualite ou la securite du code.

Le terme "anti-extraction" n'existe pas dans le vocabulaire AGPL.
C'est un concept informel qui signifie approximativement "empecher
l'appropriation proprietaire sans reciprocite". L'AGPL l'approche
via la clause reseau, mais ne le garantit pas completement (ex :
un fork proprietaire qui n'offre pas de service reseau echappe a
la clause).

Source : [GNU AGPL-3.0](https://www.gnu.org/licenses/agpl-3.0.en.html)

### 4.5 Builds reproductibles (Debian, NixOS)

La communaute Reproducible Builds distingue :
- **Source disponible** : le code est lisible (condition minimale)
- **Build deterministe** : meme source → meme binaire (condition technique)
- **Build reproductible** : un tiers independant peut re-builder et obtenir le meme resultat (verification)
- **Build verifie** : un rebuilder independant a effectivement confirme la reproduction

SBFB est au niveau "source disponible" + "build single-node
auto-signe". Il n'y a pas de build reproductible au sens de la
communaute (NixOS est a ~91% de reproductibilite, Debian a un
reseau de rebuilders independants).

Le roadmap LT-7 (self-hosted build) prevoit un quorum
multi-builder, ce qui rapprocherait SBFB du niveau "build
reproductible" mais n'y est pas encore.

Source : [Reproducible Builds](https://reproducible-builds.org/reports/2025-05/)

---

## 5. Taxonomie proposee — niveaux de confiance SBFB

Basee sur l'analyse du code et l'ecosysteme externe, voici une
taxonomie en 6 niveaux, du plus faible au plus fort :

### Niveau 0 — Upload direct (pas de source)
- **Badge propose** : aucun badge, ou "non verifie"
- **Ce qui est garanti** : l'archive existe dans iroh-blobs avec un hash BLAKE3
- **Ce qui n'est PAS garanti** : aucune correspondance code source, aucune provenance
- **Etat publish** : Unverified Build

### Niveau 1 — Source lisible
- **Badge propose** : "Source" + lien vers repo
- **Ce qui est garanti** : une URL de depot est declaree dans les metadonnees reseau
- **Ce qui n'est PAS garanti** : le code du depot correspond a l'archive, le depot est accessible, le code est audite
- **Condition code** : `entry.repo_url` existe

### Niveau 2 — Provenance auto-attestee (SLSA L1)
- **Badge propose** : "Provenance disponible" (icone document/certificat)
- **Ce qui est garanti** : le coordinator qui a deploye a genere un enregistrement de provenance lie a un commit+repo+artifact_hash, signe Ed25519
- **Ce qui n'est PAS garanti** : la provenance a ete verifiee par un tiers, le build est reproductible, la signature est valide (non encore verifiee)
- **Condition code** : `entry.provenance_hash` existe
- **Action utilisateur** : cliquer pour verifier la signature (VerificationDetail)

### Niveau 3 — Signature verifiee live
- **Badge propose** : "Signature verifiee" + ShieldCheck vert (UNIQUEMENT apres verification API)
- **Ce qui est garanti** : la signature Ed25519 de la provenance a ete recalculee et validee par le coordinator, et le hash annonce correspond au hash retourne
- **Ce qui n'est PAS garanti** : le build est reproductible par un tiers, le code est audite
- **Condition code** : appel API `provenance_verify` retourne `verified: true` ET pas de hashMismatch

### Niveau 4 — Build reproductible (futur, post-LT-7)
- **Badge propose** : "Build reproductible" + nombre de rebuilders
- **Ce qui est garanti** : N builders independants ont obtenu le meme artifact_hash depuis le meme commit
- **Condition** : LT-7 quorum multi-builder implemente
- **Status** : non implemente

### Niveau 5 — Feed verifie (hash-chain publique)
- **Badge propose** : "Hash-chain valide" (deja bien fait dans KudosTab)
- **Ce qui est garanti** : la sequence d'entrees du feed est integre (BLAKE3 hash-chain + Ed25519 per-entry)
- **Note** : ce niveau est orthogonal aux niveaux 0-4 (il concerne le feed, pas les apps individuelles)

### Niveaux transversaux

| Concept | Badge propose | Signification precise |
|---|---|---|
| Licence AGPL-3.0 | "AGPL-3.0" ou "Copyleft" | Le code source des modifications doit etre partage si utilise en reseau |
| Curator vouch | "Referenc par {curator_name}" | Un curator a inclus ce projet dans sa liste signee Ed25519. Le curator est un humain qui a fait un choix — pas une verification technique. |
| Sandbox | "Sandbox" (deja present) | L'app tourne dans un iframe sandbox="allow-scripts" sans allow-same-origin, CSP connect-src 'none' |

---

## 6. Corrections specifiques proposees

### 6.1 Browse.tsx — Badge "Verifie" → "Provenance"

**Avant** :
```tsx
{entry.provenance_hash && (
  <span data-testid="verified-badge">
    <ShieldCheck /> Verifie
  </span>
)}
```

**Apres** :
```tsx
{entry.provenance_hash && (
  <span data-testid="provenance-badge">
    <FileCheck /> Provenance
  </span>
)}
```

Remplacer `ShieldCheck` par `FileCheck` ou `ScrollText`
(icone document, pas icone bouclier). Le mot "Provenance" est
factuellement exact sans sur-promettre.

### 6.2 BrowsedProject.tsx — Badge "Verifie" → "Provenance" + action

**Avant** : badge "Verifie" statique.
**Apres** : badge "Provenance" qui indique qu'on peut verifier.
Apres click et verification reussie : badge dynamique
"Signature valide" (vert) ou "Verification echouee" (rouge).

### 6.3 GpuConsentDialog.tsx — L2 wording

**Avant** : "Projets open source verifies (SLSA L1)"
**Apres** : "Apps deployees depuis un depot public (provenance auto-attestee)"
ou "Apps avec provenance source (deploy verifie, auto-atteste)"

Le tooltip pourrait dire : "Le coordinator a clone le depot Git
public, construit l'archive, et signe la provenance. C'est une
auto-attestation — le meme noeud build et signe. Ce n'est pas un
build reproductible par des tiers independants."

### 6.4 Curators.tsx — "de confiance" → neutre

**Avant** : "Colle la cle publique Ed25519 d'un curator de confiance"
**Apres** : "Colle la cle publique Ed25519 d'un curator"

Le choix de confiance appartient a l'utilisateur — l'UI ne
devrait pas qualifier le curator de "de confiance".

### 6.5 Protocol Explorer — corrections majeures

| Section | Avant | Apres |
|---|---|---|
| App lifecycle §2 | "Le code sur le reseau = le code du depot" | "L'archive reseau est construite depuis le depot source par le noeud local. C'est une auto-attestation — un tiers peut verifier la provenance mais pas encore reproduire le build independamment." |
| Security §4 | "Le coordinator verifie l'identite" | "Le coordinator verifie que le SBFB.json du depot declare le meme node_id que le daemon local" |
| Security §4 | "Le code sur le reseau correspond au code du depot" | "La provenance lie un commit source au hash de l'archive via une signature Ed25519 du noeud qui a deploye" |
| Philosophy §5 | "Le modele F-Droid/Linux" | "Inspire par F-Droid — les apps publiques sont deployees depuis leur code source. A terme, des builds reproductibles multi-builder renforceront cette garantie." |
| Philosophy §5 | "Pas de noeud bootstrap privilegie" | "Le protocole converge vers zero noeud privilegie — la phase pre-lancement utilise une allowlist bootstrap temporaire." |
| Verification §6 | "Chaine de preuve" | "Chaine de provenance" (provenance ≠ preuve) |

### 6.6 Network.tsx — L2 label

**Avant** : `"L2 — Open source"`
**Apres** : `"L2 — Depot public"`

### 6.7 PUBLISH_MODEL.md — align wording

Le doc utilise "open source verifie" comme nom d'etat. Renommer
en "Release avec provenance" ou "Verified Release (auto-atteste)"
pour eviter la confusion entre "verifie par un tiers" et
"auto-atteste par le builder".

---

## 7. Plan de phases propose pour S65

### Phase A — Inventaire + taxonomie (documentation)

Delivrable : `TRUST_TAXONOMY.md` dans `docs/protocol/` qui
definit formellement les 6 niveaux, le vocabulaire exact, et les
conditions code pour chaque badge.

- Ecrire le document de taxonomie
- Corriger PUBLISH_MODEL.md (aligner vocabulaire)
- Corriger les textes du Protocol Explorer (index.html)
- Tests : aucun test code — documentation seulement

Scope : ~200 lignes de doc, ~20 lignes de corrections HTML.

### Phase B — Migration badges UI

Delivrable : tous les badges/labels UI migres vers la nouvelle
taxonomie.

- Browse.tsx : "Verifie" → "Provenance"
- BrowsedProject.tsx : "Verifie" → "Provenance" + etat dynamique post-verification
- GpuConsentDialog.tsx : L2 wording corrige
- Network.tsx : L2 label corrige
- Curators.tsx : "de confiance" retire
- Mise a jour des tests existants (BrowsedProject.test.tsx, VerificationDetail.test.tsx, Deploy.test.tsx)
- Nouvelle icone (FileCheck ou ScrollText au lieu de ShieldCheck pour le badge pre-verification)

Scope : ~10 fichiers touches, ~60 lignes modifiees, ~20 lignes de tests ajustees.

### Phase C — Badge dynamique post-verification (optionnel mais recommande)

Delivrable : le badge "Provenance" dans BrowsedProject devient
dynamique — il passe de "Provenance" (neutre) a "Signature
valide" (vert) ou "Echoue" (rouge) apres verification live
automatique a l'ouverture de la page.

- Appel `provenance_verify` automatique a l'ouverture de BrowsedProject
- Etat transitoire : "Verification..." pendant l'appel API
- Cache du resultat pour la session (eviter appels repetitifs)
- Tests : nouveau test Vitest pour l'etat dynamique

Scope : ~50 lignes ajoutees dans BrowsedProject.tsx, ~30 lignes de test.

### Phase D — Tests de non-regression wording

Delivrable : aucun texte public ne sur-promet.

- Script CI `scan-trust-wording.sh` analogue a `scan-en-strings.sh`
  qui grep les termes interdits dans le contexte UI :
  - "verifie" sans qualification (autorise dans VerificationDetail post-check)
  - "de confiance" dans un contexte automatique
  - "Le code sur le reseau = le code" (egalite trop forte)
  - "anti-extraction" (terme non-defini)
- Corrections residuelles trouvees par le script
- Mise a jour CLAUDE.md §Carry avec les items resolus

Scope : ~40 lignes de script, ~10 lignes de corrections residuelles.

### Ordre et rationale

1. **Phase A d'abord** : on ne peut pas corriger l'UI sans avoir defini la taxonomie de reference.
2. **Phase B ensuite** : migration technique des textes existants, avec tests mis a jour.
3. **Phase C en option** : amelioration UX qui n'est pas strictement necessaire pour le contrat public mais rend le badge honnete.
4. **Phase D en dernier** : gate de non-regression pour empecher les futurs sprints de re-introduire des sur-promesses.

### Items MANDATORY carries a traiter en parallele

- **P2-FEED-INSERT-NO-AUTH-TIER (3/3 MANDATORY)** : doit etre traite en Phase A ou Phase B comme fix de securite.
- **P2-BADGE-WORDING-PREMATURE** : directement resolu par Phase B.

---

## 8. Pitfalls potentiels

### P1 — Regression UX par excis de precision

**Risque** : en rendant les badges trop techniques, l'utilisateur
ne comprend plus rien et ignore les informations de confiance.

**Mitigation** : garder un vocabulaire simple ("Provenance" et non
"auto-attestation SLSA L1 Ed25519 JCS RFC 8785"). Les details
techniques restent dans le tooltip ou le dialog.

### P2 — Tests de wording fragiles

**Risque** : un script qui grep "verifie" bloquera des usages
legitimes (ex : "Reverifier maintenant" dans VerificationDetail).

**Mitigation** : le script doit avoir une allowlist de contextes
autorises (VerificationDetail, tests, comments code).

### P3 — Incoherence entre Protocol Explorer et UI React

**Risque** : corriger l'UI React mais oublier le Protocol
Explorer (ou vice versa).

**Mitigation** : Phase A corrige les deux simultanement. Phase D
couvre les deux dans le scan.

### P4 — "Provenance" terme inconnu du grand public

**Risque** : l'utilisateur non-technique ne comprend pas ce que
"Provenance" signifie.

**Mitigation** : tooltip explicatif sur le badge. "La provenance
lie le code source au hash de l'archive deployee via une
signature Ed25519." Alternative : "Origine verifiable" mais c'est
plus long.

### P5 — SLSA L1 comme marqueur marketing

**Risque** : SLSA L1 sonne bien mais garantit peu. Le retirer de
l'UI pourrait sembler anti-marketing.

**Mitigation** : SLSA L1 reste dans la documentation technique
(PUBLISH_MODEL, THREAT_MODEL) mais sort de l'UI grand public.
Les tooltips de la dialog consent peuvent dire "provenance signee
par le noeud builder" sans citer SLSA.

---

## 9. Ce qui est bien fait (ne pas casser)

Certains elements de l'UI sont deja excellents et ne doivent pas
etre touches :

1. **KudosTab integrite badge** : wording precis et honnete, distingue valide/invalide, explique les consequences.
2. **VerificationDetail dialog** : verification reelle via API, affiche le resultat exact, warning sur hashMismatch.
3. **Badge "sandbox"** dans BrowsedProject : factuel, pas de sur-promesse.
4. **GpuConsentDialog threat notes** : bonne pratique de transparence sur les risques.
5. **Deploy.tsx** : affiche les hash/commit/provenance post-deploy sans qualifier de "verifie".
6. **Curators page header** : "listes signees Ed25519" est techniquement exact.

---

## 10. Estimation effort et risque

| Phase | Effort | Risque | Bloquante |
|---|---|---|---|
| A — Taxonomie + docs | 1h travail code, 2h ecriture | Faible | Oui (prerequis B-D) |
| B — Migration badges UI | 2h code + tests | Faible | Oui (coeur du sprint) |
| C — Badge dynamique | 1h code + tests | Faible | Non (amelioration) |
| D — Script non-regression | 1h script + corrections | Moyen (faux positifs) | Non (protection future) |

**Total estime** : 4 phases en 1 sprint de taille normale.

---

## 11. Sources

### Sources code (analysees exhaustivement)
- `web/src/pages/Browse.tsx` (392 lignes)
- `web/src/pages/BrowsedProject.tsx` (633 lignes)
- `web/src/pages/Deploy.tsx` (191 lignes)
- `web/src/pages/Network.tsx` (438 lignes)
- `web/src/pages/Curators.tsx` (327 lignes)
- `web/src/pages/ProjectDetail.tsx` (214 lignes)
- `web/src/pages/OnboardingEmpty.tsx` (111 lignes)
- `web/src/components/VerificationDetail.tsx` (261 lignes)
- `web/src/components/GpuConsentDialog.tsx` (408 lignes)
- `web/src/components/project/KudosTab.tsx` (181 lignes)
- `web/src/components/project/OverviewTab.tsx` (109 lignes)
- `examples/sbfb-explorer/index.html` (461 lignes)
- `examples/sbfb-ideas/index.html` (69 lignes)
- `.planning/codebase/frontend_architecture.md`
- `.planning/codebase/security_posture.md`
- `.planning/codebase/protocol_wire_formats.md`
- `.planning/codebase/APPS_BRIDGE_DOCS.md`
- `docs/architecture/PUBLISH_MODEL.md`
- `docs/protocol/PUBLIC_FEED_SPEC.md`
- `docs/security/THREAT_MODEL.md`
- `docs/security/RUNTIME_ISOLATION.md`

### Sources externes
- [F-Droid — Making Reproducible Builds Visible (mai 2025)](https://f-droid.org/2025/05/21/making-reproducible-builds-visible.html)
- [F-Droid Verified](https://verification.f-droid.org/verified.html)
- [SLSA v1.0 Security Levels](https://slsa.dev/spec/v1.0/levels)
- [SLSA Provenance Spec](https://slsa.dev/spec/v1.0-rc1/provenance)
- [Sigstore Overview](https://docs.sigstore.dev/cosign/signing/overview/)
- [Sigstore In-Toto Attestations](https://docs.sigstore.dev/cosign/verifying/attestation/)
- [GNU AGPL-3.0](https://www.gnu.org/licenses/agpl-3.0.en.html)
- [Reproducible Builds — Mai 2025](https://reproducible-builds.org/reports/2025-05/)
- [NixOS Reproducible Builds](https://reproducible.nixos.org/)

---

*Recherche S65 Contrat Public : 2026-05-18*
