# Self-Distribution — le protocole est son propre premier contenu

**Ecrit** : Sprint 27 Phase D (2026-04-25)
**Type** : design doc (spec consumee par sprint d'implem ~S30)
**Statut** : draft, zero code

---

## 1. Principe

Tout est un blob : apps, donnees, et le protocole lui-meme. Les
binaires SBFB (nexus-launcher, nexus-shell-daemon, nexus-coordinator
wheel, nexus-worker) sont des blobs distribuables par le meme reseau
iroh qu'ils creent. Pas d'exception, pas de CDN, pas de store
applicatif, pas de serveur de telechargement centralise.

Un noeud SBFB existant peut exporter un bundle installe contenant
tout ce qu'il faut pour creer un nouveau noeud. Le nouveau noeud
rejoint le reseau et peut a son tour exporter le bundle. Le reseau
est auto-suffisant des le deuxieme noeud.

---

## 2. Format bundle

Archive zip signee Ed25519 (reutilise le pipeline verified deploy
S14 : Keyoxide Ed25519 + provenance.json SLSA L1).

Contenu du bundle :

```
sbfb-bundle-<version>-<platform>.zip
├── bin/
│   ├── nexus-launcher[.exe]
│   ├── nexus-shell-daemon[.exe]
│   └── nexus-worker[.exe]
├── wheels/
│   └── nexus_coordinator-<version>-py3-none-any.whl
├── configs/
│   ├── bootstrap_relays.toml    # URLs relay iroh pour premier contact
│   ├── trust_web_seeds.toml     # trust anchors (ONG Ed25519 keys)
│   └── worker.toml.sample
├── provenance.json              # SLSA L1, meme format que apps S14
└── MANIFEST.blake3              # hash BLAKE3 de chaque fichier
```

**Platforms cibles** : `x86_64-unknown-linux-gnu`,
`x86_64-pc-windows-msvc`, `aarch64-apple-darwin`.

**Signature** : la cle Ed25519 du mainteneur (FlowUP bootstrap,
ONG post-S28) signe `MANIFEST.blake3`. La verification utilise
le meme `nexus_core_rs::verify_signature` que le verified deploy
S14 — zero code crypto supplementaire.

**Taille estimee** : ~30-50 MB compresse (daemon ~15 MB release,
worker ~10 MB, launcher ~3 MB, wheel ~5 MB, configs ~10 KB).

---

## 3. Canaux de distribution

### 3.1 iroh-blobs P2P (canal primaire)

Si le destinataire a deja un noeud SBFB (meme minimal, meme un
ancien bundle), il peut recevoir le nouveau bundle via
`iroh-blobs` comme n'importe quel autre blob. Le hash du bundle
est publie sur le gossip topic `nexus-grid/protocol-update/v1`.

### 3.2 Bluetooth (~30-50 MB, ~2 min)

Pour le premier noeud dans un reseau isole (pas de connexion
Internet, pas de noeud SBFB existant). Un noeud existant envoie
le bundle via Bluetooth PAN ou OBEX. Le recepteur verifie la
signature Ed25519 avant installation.

Cas d'usage : distribution en zone de conflit, camp de refugies,
zone de catastrophe naturelle (cf. `docs/apps/CATASTROPHE_HUMANITAIRE.md`).

### 3.3 WiFi Direct (~3 sec)

Meme scenario que Bluetooth mais plus rapide sur courte distance.
Le noeud emetteur demarre un hotspot WiFi Direct temporaire,
sert le bundle via HTTP local. Le recepteur verifie la signature.

### 3.4 Carte SD / USB

Distribution physique. Le bundle est copie sur un support
amovible. Le recepteur branche le support, lance le launcher
qui detecte automatiquement le bundle et propose l'installation.

### 3.5 HTTP fallback (download classique)

Pour les utilisateurs avec acces Internet mais sans noeud SBFB.
Le bundle est disponible sur un miroir statique (GitHub Releases,
miroir Radicle, cf. `docs/release/MIRROR_FALLBACK.md`). Ce canal
est le seul centralise — il existe uniquement comme bootstrap
initial et devient obsolete des que le premier noeud local est
operationnel.

---

## 4. Bootstrap problem

Le tout premier noeud d'un reseau isole vient forcement d'un
canal externe (Bluetooth, WiFi Direct, USB, HTTP). C'est une
contrainte fondamentale de tout reseau P2P : le premier pair ne
peut pas se decouvrir lui-meme.

Une fois le premier noeud installe et connecte (meme a un seul
relay), le reseau est self-sustaining : chaque noeud peut exporter
le bundle et le distribuer a de nouveaux noeuds via n'importe quel
canal §3.

**Resilience** : si tous les relays iroh sont bloques (scenario
ISP national T5), les canaux hors-ligne (Bluetooth, WiFi Direct,
USB) permettent de creer des reseaux mesh locaux autonomes. Le
bundle contient la config relay mais aussi les cles trust anchor
— un reseau local peut fonctionner sans jamais contacter un relay
Internet.

---

## 5. Lien verified deploy S14

Meme signature Ed25519, meme `provenance.json` SLSA L1, meme
verification via `nexus_core_rs::verify_provenance`. La seule
difference : le payload est les binaires SBFB au lieu d'une app
tierce.

La chaine de confiance est identique :
1. Le mainteneur clone le repo source (GitHub/GitLab/Codeberg)
2. Build reproductible (`--locked`, `SOURCE_DATE_EPOCH`)
3. Signature Ed25519 du manifeste BLAKE3
4. Publication provenance.json avec repo URL + commit SHA + build
   env
5. Le recepteur verifie : signature valide + provenance coherente
   + hash BLAKE3 match

Le code S14 (`crates/nexus-core-rs/src/provenance.rs` +
`packages/nexus-coordinator/src/nexus_coordinator/verified_deploy.py`)
est reutilise tel quel. Zero duplication crypto.

---

## 6. Endpoint daemon — `GET /export-bundle`

Nouvel endpoint loopback sur le shell daemon :

```
GET /daemon/export-bundle
Authorization: Bearer <X-SBFB-Token>
```

Reponse : le bundle zip du noeud en cours (binaires installes +
config active + provenance). Le frontend peut proposer un bouton
"Exporter pour un ami" qui declenche le telechargement.

**Securite** : endpoint protege par le bearer token loopback S16.
Pas d'exposition externe. Le bundle exporte contient les binaires
publics et la config bootstrap — pas de cle privee, pas de state
SQLite, pas de donnees utilisateur.

---

## 7. Update P2P

Un noeud existant peut recevoir une nouvelle version du bundle
via le gossip topic `nexus-grid/protocol-update/v1`.

Protocole :
1. Le mainteneur publie le nouveau bundle hash sur le gossip topic
2. Les noeuds abonnes recoivent l'annonce
3. Chaque noeud telecharge le bundle via iroh-blobs
4. Verification : signature Ed25519 mainteneur + hash BLAKE3
   + version > version installee
5. Si verification OK : notification utilisateur "nouvelle version
   disponible"
6. L'utilisateur accepte → le launcher remplace les binaires et
   redemarre

**Pas d'auto-update silencieux** : l'utilisateur controle
l'installation. Le reseau notifie, l'utilisateur decide. Pattern
Firefox/Chromium, pas Windows Update.

**Rollback** : le bundle precedent est conserve dans
`~/.sbfb/bundles/previous/`. Si la nouvelle version crashe, le
launcher peut restaurer automatiquement (detection : daemon ne
repond pas au health-check apres 30s post-restart).

---

## 8. Implementation target

Sprint ~S30 (release prep pre-v1.0). Estimations :

- `crates/nexus-shell-daemon-core/src/export_bundle.rs` — endpoint
  + bundle assembly (~150 LOC Rust)
- `crates/nexus-launcher/src/update.rs` — gossip subscribe +
  download + verify + replace (~150 LOC Rust)
- CI cross-compile matrix GitHub Actions (3 targets) — ~100 LOC
  YAML
- Total : ~400 LOC + CI config

**Pre-requis** :
- Binaires stables (hardening S18-S29 complete)
- Verified deploy pipeline S14 live
- Au moins 2 relays iroh operationnels
- Trust anchors ONG S28 en place (au minimum FlowUP bootstrap)
