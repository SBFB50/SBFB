# Runtime isolation — roadmap VM invisible

**Ecrit** : Sprint 16 Phase E (2026-04-14)
**Cible** : Sprint 17+
**Status** : **roadmap uniquement** — aucune ligne implementee ce
sprint. Les fondations compatibles sont posees par Sprint 16 A-B
(bearer token + UDS + paths `~/.sbfb/`).

---

## 1. Rationale

Le threat model [§5.7](THREAT_MODEL.md#57-key-storage) identifie
la keypair `~/.sbfb/daemon.key` + `~/.sbfb/auth_token` comme le
risque residuel #1 : un process user-mode local peut les lire
(perm 0600 suffit pas face a un malware sur le meme compte). Les
mitigations Sprint 16 **ne ferment pas** ce trou : elles
protegent la couche reseau, pas le FS.

L'approche **VM invisible** le ferme proprement :

- Le daemon + coordinator + keypair vivent dans un runtime
  isole (WSL2 sur Windows, Virtualization.framework sur Mac,
  systemd-nspawn sur Linux).
- Le shell React (browser hote) reste sur le host, parle au
  guest via un port forward + bearer token.
- Un malware user-mode sur le host ne peut plus lire la keypair
  : c'est un filesystem different, mappe a un autre namespace.

Ce qu'on gagne (estimation) :

| Categorie | Avant Sprint 17+ VM | Apres VM |
|---|:---:|:---:|
| Extension navigateur avec loopback perms | Bloque S16 (bearer + Host + Origin) | Bloque |
| Malware user-mode lit keypair | **Possible** (perm 0600) | **Impossible** (guest FS isole) |
| Malware user-mode pivote via `/project/deploy-from-repo` | Bloque S16 (bearer) | Bloque |
| Autre user sur la meme machine | Bloque S16 (SO_PEERCRED / DACL) | Bloque |
| 0-day exploit iroh / axum | RCE user-mode | RCE **dans la VM** (blast radius reduit) |
| Supply chain dep Rust malveillante | RCE user-mode | RCE dans la VM |

On garde les caps consent (Phase C) et la verification deploy
(Sprint 14) : la VM ne remplace pas, elle ajoute une couche.

L'exigence d'**invisibilite** est cruciale : l'utilisateur ne
doit **pas** avoir a apprendre WSL, installer VirtualBox, ou
deboguer des network bridges. Le launcher gere tout.

---

## 2. Technologies cibles

### 2.1 Windows — WSL2 (Windows Subsystem for Linux 2)

- **Disponibilite** : natif Windows 10 2004+ / 11, preinstalle
  sur 11 depuis 22H2.
- **Installation** : `wsl --install --no-distribution` puis
  `wsl --install -d Ubuntu-24.04` (ou image custom SBFB
  minimaliste).
- **CUDA passthrough** : natif depuis WSL2 2022 (kernel 5.10.43+).
  RTX 5080 accessible via `/dev/nvidia*` sans aucune config.
- **Network** : bridge NAT par defaut, `localhost` du host ping
  le guest (WSL2 mirrored networking mode depuis Windows 11 22H2
  simplifie encore).
- **Cout UX** : premier lancement ~2 min (download image). Les
  suivants ~3s (image deja hydratee).

### 2.2 macOS — Virtualization.framework

- **Disponibilite** : natif macOS 11 Big Sur+ (Apple Silicon et
  Intel).
- **Installation** : le launcher utilise la crate
  [`vz`](https://crates.io/crates/vz) ou appel FFI direct vers
  `Virtualization.framework`. Image custom Linux ARM64.
- **CUDA passthrough** : **N/A** (Apple Silicon n'a pas de GPU
  NVIDIA). Les workers Mac restent CPU-only ou Metal via un
  chemin separe (scope Sprint 18+).
- **Network** : `virtio-net` bridge, port forward geres par le
  framework.
- **Cout UX** : premier lancement ~30s (image ~200 MB), suivants
  ~2s.

### 2.3 Linux — systemd-nspawn

- **Disponibilite** : natif systemd 219+ (distros modernes
  depuis ~2017).
- **Installation** : container root ext4 extrait depuis un
  tarball signe. Pas d'image disk complete.
- **CUDA passthrough** : via `--bind=/dev/nvidia*` +
  `nvidia-container-toolkit` installe cote host. Fonctionne mais
  depend de la config NVIDIA.
- **Network** : `veth` pair + NAT, geres par
  `systemd-networkd`.
- **Cout UX** : premier lancement ~10s (extract tarball), suivants
  ~1s.
- **Alternatives considerees** : Docker (trop lourd UX, requiere
  daemon), LXC (config plus complexe), Podman (bon, mais moins
  universel), Firecracker (trop bas niveau).

### 2.4 Tableau comparatif

| Critere | WSL2 | Virtualization.framework | systemd-nspawn |
|---|:---:|:---:|:---:|
| Installation silencieuse | OK (wsl --install) | OK (framework systeme) | OK (systemd integre) |
| CUDA NVIDIA passthrough | **Natif** | N/A (Apple Silicon) | Via nvidia-container-toolkit |
| Cold start | ~2 min | ~30 s | ~10 s |
| Warm start | ~3 s | ~2 s | ~1 s |
| Disk image | VHDX ~500 MB | RAW ~200 MB | Tarball ~150 MB |
| Memoire idle | ~150 MB | ~80 MB | ~40 MB |
| Complexite launcher | Moyen | Moyen (FFI) | Bas |

---

## 3. Phasage Sprint 17+

Decoupe proposee en **4 phases** sur Sprint 17 (potentiellement
etalees sur Sprint 17 + 18 si scope trop large) :

### Phase A (Sprint 17) — Detection environnement

**Scope** :
- `crates/nexus-launcher/src/runtime.rs` nouveau module
- Fonction `detect_runtime() -> RuntimeKind` :
  - Windows : probe `wsl --status` exit code, version kernel
  - Mac : probe `/System/Library/Frameworks/Virtualization.framework`
  - Linux : probe `systemd-run --version` + `/proc/1/comm ==
    systemd`
- Fallback `RuntimeKind::HostNative` si aucun runtime detectable
- Endpoint launcher `GET /runtime/status` expose la detection
  pour le shell React

**Livrable** : `sbfb-launcher` sait dire a l'UI si WSL2/VM est
disponible. Aucun guest n'est encore demarre.

### Phase B (Sprint 17) — Bootstrap image signee

**Scope** :
- Image SBFB minimaliste (Alpine + Rust binaires + Python) :
  ~200 MB
- Signature Ed25519 par la keypair de release
- Hash BLAKE3 pinne dans le launcher
- Download depuis release GitHub (ou mirror IPFS)
- Extraction dans :
  - Windows : `%LOCALAPPDATA%\SBFB\wsl-image.vhdx` + `wsl --import`
  - Mac : `~/Library/Application Support/SBFB/guest.raw`
  - Linux : `~/.local/share/SBFB/guest/` (ext4 extrait)

**Livrable** : le launcher telecharge et installe l'image en
background au 1er lancement. Barre de progression UI.

### Phase C (Sprint 17) — Migration daemon + coord dans VM

**Scope** :
- Le launcher spawn le daemon + coordinator **dans le guest**
  plutot que sur le host.
- Port forward `host:8080 -> guest:8080` (coord TCP),
  `host:7777 -> guest:7777` (daemon TCP), `host:7000 ->
  guest:7000` (blob-serve).
- Le bearer token vit dans le guest (`~/.sbfb/auth_token` cote
  guest) — le shell React le fetche via le port forward
  `/auth/token` et l'injecte normalement.
- La keypair Ed25519 ne sort plus du guest FS.
- UDS/Named Pipe : desactives sur le host, actifs dans le guest
  entre daemon et coord (meme chemin `~/.sbfb/run/*.sock`).

**Livrable** : l'utilisateur voit zero changement UX. La keypair
est invisible au host.

### Phase D (Sprint 17 ou 18) — Fallback sans virtualisation

**Scope** :
- Si la detection Phase A retourne `HostNative`, on garde le
  mode Sprint 16 (daemon + coord sur host) mais on ajoute :
  - **Encryption at rest** via Keychain (macOS) / DPAPI
    (Windows) / libsecret (Linux) pour `daemon.key` et
    `auth_token`.
  - Avertissement UI "Mode isole non disponible sur ta
    machine" avec lien vers doc.
- Pattern : `keyring` crate Rust (ecosysteme mature, wraps les
  3 APIs OS).

**Livrable** : l'utilisateur sans WSL2/framework dispo a quand
meme la keypair protegee au repos (equivalent 80% de la VM pour
le residual #1).

### Compteur tests estime Sprint 17+

- Phase A : ~80 tests (detection par OS, mocks)
- Phase B : ~60 tests (image verify, download resume, path
  perm)
- Phase C : ~100 tests (spawn guest, port forward, bearer
  round-trip)
- Phase D : ~80 tests (keyring roundtrip, fallback transparent)

Total : ~320 tests, ~3000 LOC.

---

## 4. Strategie backward-compat

### 4.1 Upgrade path pour utilisateurs v1.2

- Au 1er boot Sprint 17 : launcher detecte WSL2/VM disponible.
- Si **oui** : propose migration. Dialog React :
  > "SBFB peut maintenant proteger ta keypair via un
  > environnement isole (WSL2). Ton identite P2P reste la meme.
  > [Activer maintenant] [Plus tard] [Ne jamais]"
- Si **accepte** :
  - La keypair est copiee du host vers le guest (UDS ephemere)
  - Le fichier host est chiffre puis supprime (backup
    ChaCha20-Poly1305 cle derivee keypair)
- Si **plus tard** : re-prompt au prochain boot, state persist
  dans `~/.sbfb/runtime_prompt.json`.
- Si **jamais** : flag permanent, mode Phase D (keyring OS).

### 4.2 Downgrade v1.3 -> v1.2

- Si l'utilisateur downgrade (pas recommande mais doit marcher) :
  - v1.2 ne connait pas le guest, ne trouve pas la keypair.
  - Launcher v1.2 regenere un `daemon.key` + `auth_token`.
  - **Consequence** : nouveau node_id. Les kudos P2P sont
    perdus — comme avant Sprint 17.
- Documenter dans `CHANGELOG.md` Sprint 17 : "downgrade =
  nouveau node_id".

### 4.3 Multi-host scenario (v2+)

Un utilisateur avec 2 PCs veut-il la **meme** identite sur les 2 ?
Actuellement non (chaque install = un node_id). Sprint 17+ ne
change rien. La portabilite d'identite via cle USB / sync chiffre
est un scope v2+.

---

## 5. Alternative sans virtualisation : process isolation

Pour completude, si la VM n'est pas souhaitable (ops lourd,
maintenance image, support CI), des alternatives **host-only**
existent :

### 5.1 seccomp-bpf (Linux)

- Filtre syscalls au niveau du daemon/coord via
  [`libseccomp`](https://github.com/seccomp/libseccomp).
- Bloque `ptrace`, `process_vm_readv` → un malware meme-user ne
  peut plus lire la memoire du daemon.
- **Limitation** : le FS reste accessible (le malware peut
  toujours `cat daemon.key`). Ne remplace pas la VM.
- Effort estime : ~200 LOC + tests.

### 5.2 AppArmor / SELinux (Linux)

- Profils qui limitent le FS access du daemon a
  `~/.sbfb/` + deny all to other processes du meme user.
- **Limitation** : requiere profil admin-installable, friction
  UX.

### 5.3 Hardened macOS entitlements

- Signature code + hardened runtime + entitlement
  `com.apple.security.get-task-allow = false`.
- Bloque `task_for_pid()` depuis autres process user.
- **Limitation** : requiere Developer ID, cout ~$99/an.

### 5.4 Windows ACL niveau FS

- Posix `0600` → Windows ACL explicite `SE_FILE_OBJECT` +
  `DACL_SECURITY_INFORMATION` avec SID du user uniquement.
- Deja present implicitement via perm 0600 emulation.
- **Limitation** : un malware user-mode a le meme SID, donc peut
  lire.

**Decision** : les alternatives §5.1-5.4 sont des stopgaps. La
vraie solution reste la VM, c'est pourquoi elle est en tete de
la roadmap Sprint 17.

---

## 6. Risques et incertitudes

| # | Risque | Impact | Mitigation |
|---|---|---|---|
| RI1 | WSL2 indisponible sur Windows 10 Home < 2004 | UX degrade ~5% utilisateurs | Phase D fallback keyring |
| RI2 | Apple Silicon n'a pas de GPU NVIDIA — workers Mac sont CPU-only ou MPS | Contribution Mac limitee | Documenter dans README, prioriser workers Linux/Win |
| RI3 | Image bootstrap 200 MB downloade au 1er run | Friction UX / bande passante | Barre progression + download en background post-premiere-UI |
| RI4 | WSL2 + VPN corpo pete souvent le bridge | Bug reports | Timeout + fallback TCP sur host + warning UI |
| RI5 | CUDA WSL2 requiere driver NVIDIA host >= 470.xx | Workers bloques sur vieux drivers | Check driver version dans detection Phase A, warn si trop vieux |
| RI6 | systemd-nspawn pas dispo sur Gentoo/Alpine host | Linux minority cassent | Phase D fallback keyring + suggerer Docker opt-in |
| RI7 | Launcher doit survivre a une VM cassee (image corrompue) | Blocage total | Phase A detecte + propose re-install, fallback host |
| RI8 | Reviewers externes (future audit) peuvent challenger l'ajout de complexite | Debat arch | Threat model chiffre le gain (95% residual #1) pour justifier |

---

## 7. Decisions differees a ce stade

- **Choix Linux : systemd-nspawn vs Docker vs Podman** — a
  trancher en Sprint 17 kickoff apres benchmark cold-start.
- **Image base : Alpine vs Ubuntu minimal vs Debian slim** —
  idem, benchmark.
- **Canal de distribution image : GitHub Releases vs CDN
  self-hosted vs IPFS** — GitHub par defaut, IPFS reflechi v2.
- **Rotation image** : freq minor (security patches) — mensuel ?
  A definir Sprint 17.
- **Portabilite keypair cross-host** — explicitement **v2+**.

---

## 8. Pointeurs

- [`THREAT_MODEL.md`](THREAT_MODEL.md) §5.7 + §8 R1 — les
  risques que cette roadmap ferme.
- [`.planning/active/sprint16_kickoff.md`](../../.planning/active/sprint16_kickoff.md)
  §1.5 — origine de la decision de differer l'implementation.
- [Microsoft WSL2 docs](https://learn.microsoft.com/en-us/windows/wsl/install)
  (consulte 2026-04-14).
- [Apple Virtualization.framework](https://developer.apple.com/documentation/virtualization)
  (consulte 2026-04-14).
- [systemd-nspawn manpage](https://man7.org/linux/man-pages/man1/systemd-nspawn.1.html).
- [`vz` crate](https://crates.io/crates/vz) — Rust bindings
  Virtualization.framework.
- [`keyring` crate](https://crates.io/crates/keyring) — OS
  secret store pour Phase D fallback.

---

## 9. Revue et evolution

- **v1 (Sprint 16 Phase E, 2026-04-14)** : version initiale.
  Roadmap proposee, non implementee.
- Sprint 17 kickoff challengera §2 choix tech via research
  phase. Si une techno est retiree du tableau, mise a jour ici
  avec raison.
- Chaque phase S17 (A, B, C, D) qui livre rafraichit cette doc :
  section §3 devient "LIVRE Sprint 17 Phase X" + commit hash.
