# Trust Taxonomy — SBFB Network

Six niveaux cumulatifs de confiance pour les apps deployees sur le
reseau SBFB. Chaque niveau porte une assertion positive (ce qu'il
garantit) et une non-assertion explicite (ce qu'il ne garantit PAS).

## Niveaux

### N0 — Upload direct

- **Label** : Upload direct
- **Assertion** : l'archive zip a ete publiee sur le reseau.
- **Non-assertion** : aucune information sur l'origine du code.
  L'archive peut contenir n'importe quoi.
- **Condition technique** : `POST /api/v1/deploy` (deploy prive).
- **Verification** : aucune verification possible par un tiers.

### N1 — Source lisible

- **Label** : Source lisible
- **Assertion** : un depot source public est reference.
- **Non-assertion** : le lien entre le depot et l'archive n'est
  PAS garanti. Le depot peut avoir diverge ou etre un leurre.
- **Condition technique** : `repo_url` present dans
  `ProjectAnnouncement` et accessible (HTTP 200).
- **Verification** : un tiers peut ouvrir le lien et lire le code.

### N2 — Provenance auto-attestee

- **Label** : Provenance
- **Assertion** : l'archive a ete construite depuis le depot source
  par le noeud local qui l'a deployee. Une attestation SLSA L1
  (`provenance.json`) lie le commit source au hash de l'archive
  via une signature Ed25519 du noeud.
- **Non-assertion** : c'est une **auto-attestation**. Le noeud
  atteste lui-meme — aucun tiers independant n'a reproduit le
  build ou verifie la correspondance.
- **Condition technique** : `deploy-from-repo` pipeline
  (clone → Ed25519 → zip → `provenance.json` SLSA L1).
- **Verification** : un tiers peut verifier la signature Ed25519
  de `provenance.json` et confirmer que le hash de l'archive
  correspond au hash signe.

### N3 — Signature verifiee live

- **Label** : Signature verifiee
- **Assertion** : le daemon local du visiteur a verifie live la
  signature Ed25519 de l'attestation de provenance. Le resultat
  est affiche dans l'UI.
- **Non-assertion** : la verification porte sur la signature, pas
  sur la correspondance source↔archive (pas de build reproductible).
- **Condition technique** : appel `provenance_verify` via le
  daemon local, verification Ed25519 + BLAKE3.
- **Verification** : le resultat est visible dans le badge
  dynamique de la page projet.

### N4 — Build reproductible (futur)

- **Label** : Build reproductible
- **Assertion** : un tiers independant a reconstruit l'archive
  depuis le depot source et obtenu le meme hash.
- **Non-assertion** : pas encore implemente. Necessite une
  infrastructure de build tiers.
- **Condition technique** : `BuildQuorumReached` feed operation
  (Sprint 67+).
- **Verification** : le tiers publie son attestation de build
  avec le hash resultant.

### N5 — Feed verifie hash-chain

- **Label** : Feed verifie
- **Assertion** : l'historique complet des operations du projet
  dans le feed public est integre (hash-chain BLAKE3 + Ed25519
  par entree, depuis le genesis).
- **Non-assertion** : l'integrite du feed ne garantit PAS que les
  operations sont correctes — seulement qu'elles n'ont pas ete
  alterees apres insertion.
- **Condition technique** : `verify_chain()` valide sur le feed
  local.
- **Verification** : n'importe quel noeud peut rejouer le feed
  et verifier la chaine.

## Dimensions transversales

Ces dimensions s'appliquent independamment des niveaux N0-N5.

### Licence AGPL-3.0

Le code du protocole SBFB lui-meme est sous AGPL-3.0 (OSI-approved,
copyleft reseau). Les apps deployees sur le reseau ne sont PAS
automatiquement AGPL — chaque app a sa propre licence.

### Curator vouch

Un curator humain (identifie par sa cle Ed25519 publique) peut
endorser une app via une `CuratorList`. C'est un avis humain, pas
une garantie technique. Un curator peut retirer son endorsement.

### Sandbox iframe

Toutes les apps sont rendues dans un iframe sandbox
(`sandbox="allow-scripts"` sans `allow-same-origin`, CSP
`connect-src 'none'` pour contenu untrusted). Le sandbox est
une protection du client, pas une propriete de l'app.

## Why not OpenSSF Scorecard

OpenSSF Scorecard (30+ checks) assume une infrastructure CI/CD
centralisee : branch protection, binary artifact hosting, automated
dependency updates, SAST/DAST pipelines. SBFB est un reseau P2P
sans CI centralisee — les apps sont deployees depuis le repo source
par le noeud local. Les checks Scorecard (e.g. "Branch-Protection",
"CI-Tests", "Signed-Releases") n'ont pas de sens dans ce contexte.
La taxonomie SBFB est adaptee au modele P2P : elle mesure ce que
le protocole peut verifier, pas ce qu'un service centralise impose.

## Evolution

Les niveaux sont cumulatifs : N2 implique N1 implique N0. Ajouter
un niveau ne retire pas les niveaux inferieurs. Les niveaux futurs
(N4, et au-dela) seront ajoutes quand l'infrastructure necessaire
sera implementee.
