# Sprint 53 — Plan (P2P smoke test multi-plateforme + VPS bootstrap)

**Tip d'entree** : `b85a3a1` (post-audit S52 PASS).
**Phases** : A (build + smoke test LAN), B (VPS + smoke test WAN),
C (unsafe set_var + verification + wrap-up),
D (gossip bootstrap from attention set — ajoutee post-smoke finding).

---

## §Phase A — Build cross-platform + smoke test LAN (Windows ↔ Mac)

**But** : builder le daemon sur macOS ARM et valider le P2P
entre la machine dev Windows et le MacBook Air 15 sur le LAN.

### Prerequis

MacBook Air 15 accessible sur le meme reseau local que la
machine dev Windows. Connexion internet pour rustup + git clone.

### Etapes

1. **Setup Mac** (guide utilisateur) :
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   git clone <repo-url> nexus-grid
   cd nexus-grid
   cargo build --release -p nexus-shell-daemon -p nexus-launcher
   ```
   Si erreur de compilation : documenter, fixer inline.

2. **Verification binaires** :
   - Windows : `nexus-shell-daemon.exe --version` (deja operationnel)
   - Mac : `./target/release/nexus-shell-daemon --version`
   - Verifier que la version cargo match

3. **Test Niveau 1 — daemon start/stop** :
   - Windows : `nexus-shell-daemon init && nexus-shell-daemon start`
   - Mac : `./target/release/nexus-shell-daemon init && ./target/release/nexus-shell-daemon start`
   - Verifier : `running.json` ecrit, HTTP port bound, logs propres
   - Stop : ctrl+c, verifier shutdown propre (pas de crash)

4. **Test Niveau 2 — P2P discovery LAN** :
   - Les 2 daemons tournent simultanement
   - Observer les logs iroh : connection au relay EU, peer discovery
   - Si le frontend React est accessible : ouvrir Browse sur les
     2 machines, verifier si les noeuds se voient
   - Documenter ce qui fonctionne et ce qui ne fonctionne pas

5. **Fix bugs bloquants** (si trouves) :
   - Bugs runtime macOS (chemins, permissions, cfg(unix) paths)
   - Bugs P2P (discovery, relay connection)
   - PAS de fix wire format (pre-launch policy : redefine v1)

### Criteres d'acceptation

- Binaire `nexus-shell-daemon` compile sur macOS ARM (aarch64-apple-darwin)
- Daemon demarre et s'arrete proprement sur macOS
- Niveau 1 atteint minimum sur les 2 OS
- Resultats P2P documentes (succes ou bugs trouves)
- cargo nextest --workspace inchange (>= 1199)
- Vitest inchange (250)

### Commit

```
feat(sprint53): Sprint 53 Phase A — cross-platform build + P2P smoke test LAN Windows-macOS
```

---

## §Phase B — VPS deployment + smoke test WAN (dev ↔ VPS)

**But** : builder le daemon sur le VPS Linux et valider le P2P
a travers internet (Windows dev ↔ VPS Helsinki).

### Prerequis

Phase A commitee. VPS accessible via SSH
(`ssh -i ~/.ssh/sbfb_hetzner root@135.181.42.188`).

### Etapes

1. **Setup VPS** :
   ```bash
   ssh -i ~/.ssh/sbfb_hetzner root@135.181.42.188
   # Sur le VPS :
   apt update && apt install -y build-essential pkg-config libssl-dev git
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   git clone <repo-url> /opt/nexus-grid
   cd /opt/nexus-grid
   cargo build --release -p nexus-shell-daemon
   ```

2. **Firewall** :
   ```bash
   ufw allow 22/tcp          # SSH (deja ouvert)
   ufw allow proto udp to any port 1024:65535  # QUIC iroh
   ufw --force enable
   ```
   Note : iroh utilise un port ephemere UDP pour QUIC. Ouvrir
   la plage haute est necessaire pour les connexions directes
   (pas seulement relay).

3. **Test Niveau 1 — daemon start Linux** :
   ```bash
   ./target/release/nexus-shell-daemon init
   ./target/release/nexus-shell-daemon start -v
   ```
   Verifier : pas de crash, `running.json` ecrit, HTTP port bound.

4. **Test Niveau 2/3 — P2P cross-network** :
   - Daemon tourne sur VPS + machine dev Windows (ou Mac)
   - Observer les logs des 2 cotes : relay connection, peer discovery
   - Si Niveau 3 possible : publier un projet sur une machine,
     verifier la decouverte sur l'autre
   - Documenter latence, timeouts, erreurs

5. **Documenter resultats** :
   - Quel niveau atteint (1, 2, ou 3)
   - Bugs trouves et leur nature (runtime, network, protocol)
   - Performance observee (temps de decouverte, taille binaire, RAM usage)

### Criteres d'acceptation

- Binaire `nexus-shell-daemon` compile sur Ubuntu 24.04 x86_64
- Daemon demarre sur le VPS (Niveau 1 minimum)
- Resultats P2P WAN documentes
- Pas de regression tests

### Commit

```
feat(sprint53): Sprint 53 Phase B — VPS bootstrap + P2P smoke test WAN dev-Helsinki
```

---

## §Phase C — unsafe set_var fix + verification + wrap-up

**But** : resoudre le carry 2/3 unsafe set_var, executer la
verification fail-fast, rediger l'audit plan S54.

### Etapes

1. **unsafe set_var fix** :
   - `grep -rn "set_var\|remove_var" crates/ --include="*.rs"`
   - Wrapper chaque appel dans `unsafe {}` avec commentaire :
     ```rust
     // SAFETY: called before tokio runtime spawn, single-threaded.
     unsafe { std::env::set_var("KEY", "value") };
     ```
   - Verifier compilation + tests

2. **CLAUDE.md** :
   - Mettre a jour "Sprints 0-53 CLOSED"
   - Carries S54
   - Compteurs tests

3. **HARDENING_ROADMAP.md** :
   - Mettre a jour last_validated S53

4. **Verification fail-fast** :
   - cargo fmt --all --check
   - cargo clippy --workspace --all-targets --locked -- -D warnings
   - cargo nextest run --workspace --locked
   - cargo test --workspace --locked --doc
   - cargo build -p nexus-shell-daemon --release
   - Frontend : lint, tsc, vitest, build, size
   - G8 preflights (A + B)
   - Phase reviews (A + B)
   - Scope cuts respectes
   - Delta tests cumule
   - Smoke test resultats
   - unsafe set_var CLOSED

5. **sprint54_audit_plan.md** :
   - Track A : smoke test LAN resultats
   - Track B : VPS deployment resultats
   - Track C : unsafe set_var fix
   - Track D : process meta

### Criteres d'acceptation

- 0 appel `set_var` / `remove_var` non-unsafe dans le workspace
- verification.md 20+ rows fail-fast verts
- CLAUDE.md a jour
- sprint54_audit_plan.md present
- P2-REVIEW-B-1-S51 CLOSED

### Commit

```
chore(sprint53): Phase C — unsafe set_var fix + wrap-up + verification + audit plan S54
```

---

## §Phase D — Gossip bootstrap from attention set

**But** : debloquer la propagation gossip inter-noeuds. Le smoke
test Phase A/B a revele que `join_topic(topic_id, vec![])` sans
bootstrap peers bloque indefiniment — le gossip_sender reste None
et `POST /publish` ne broadcast jamais. Fix : passer les peer_ids
de l'attention set comme bootstrap au `join_topic`.

### Etapes

1. **`spawn_gossip_subscribe_task`** (runtime.rs) : accepter un
   parametre `bootstrap_peers: Vec<String>` et le passer au
   `gossip.join_topic(topic_id, bootstrap_peers)`.

2. **`DaemonRuntime::start`** (runtime.rs) : au moment du spawn,
   lire `curator_runtime.subscribed_pubkeys_hex()` et le passer
   comme bootstrap_peers.

3. **Tests** :
   - Test unitaire : verify gossip_sender is Some after boot
     with at least 1 subscribed curator (mock ou integration
     selon faisabilite)

### Criteres d'acceptation

- `join_topic` recoit les peers connus comme bootstrap
- gossip_sender devient Some quand >= 1 peer est dans l'attention set
- cargo nextest inchange (>= 1203)

### Commit

```
feat(sprint53): Sprint 53 Phase D — gossip bootstrap from curator attention set
```

---

## §Phase G — Browse pull via gossip request + bouton Rafraichir

**But** : permettre au bouton "Rafraichir" de la page Browse de
soliciter activement les browse entries des peers connectes. Sans
ce mecanisme, un noeud qui arrive apres la publication d'une app
ne la voit jamais (le push via outbox/NeighborUp est unidirectionnel).

### Etapes

1. **Discriminant gossip** : ajouter `is_browse_request()` dans
   publish.rs (type "browse_request", payload minimal `{"type":"browse_request"}`).

2. **Runtime gossip task** : quand un message `browse_request`
   est recu, replayer l'outbox vers le gossip (meme que NeighborUp).

3. **GossipCmd::RequestBrowse** : nouvelle commande pour que le
   handler HTTP puisse declencher un broadcast de browse_request.

4. **Endpoint `POST /api/daemon/browse/pull`** : envoie un
   browse_request via gossip, retourne immediatement. Le refetch
   cote React montre les resultats quand les reponses arrivent.

5. **Frontend Browse.tsx** : le bouton "Rafraichir" appelle
   d'abord POST /browse/pull, attend 2s, puis refetch GET /browse.

### Criteres d'acceptation

- Un noeud arrivant apres publication voit l'app apres "Rafraichir"
- cargo nextest inchange (>= 1203)
- Vitest inchange (>= 250)

### Commit

```
feat(sprint53): Sprint 53 Phase G — browse pull via gossip request
```

---

## §4 Fail-fast checklist

| # | Check | Commande | Critere | Observed |
|---|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff | |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings | |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1199, 0 fail | |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok | |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok | |
| 6 | npm lint | `npm run lint` (web/) | 0 error | |
| 7 | tsc | `npx tsc --noEmit -p tsconfig.app.json` | 0 error | |
| 8 | Vitest | `npm run test:unit` (web/) | >= 250 | |
| 9 | build | `npm run build` (web/) | ok | |
| 10 | size-limit | `npm run size` (web/) | 6/6 | |
| 11 | Phase A preflight G8 | verdict | | |
| 12 | Phase A review | verdict | | |
| 13 | Phase B preflight G8 | verdict | | |
| 14 | Phase B review | verdict | | |
| 14b | Phase D preflight G8 | verdict | | |
| 14c | Phase D review | verdict | | |
| 15 | macOS build | `cargo build --release -p nexus-shell-daemon` | ok | |
| 16 | macOS daemon start | `nexus-shell-daemon start` | running.json | |
| 17 | Linux VPS build | `cargo build --release -p nexus-shell-daemon` | ok | |
| 18 | Linux daemon start | `nexus-shell-daemon start` | running.json | |
| 19 | P2P Niveau 1 | daemon start 2+ OS | ok | |
| 20 | P2P Niveau 2 | LAN discovery | resultat documente | |
| 21 | P2P Niveau 3 | WAN discovery | resultat documente | |
| 22 | unsafe set_var | 0 non-unsafe calls | grep clean | |
| 23 | Scope cuts | 12/12 respectes | | |
| 24 | Delta tests | cumule documente | | |

---

## §5 Scope cuts (rappel kickoff §7)

1. Woodpecker agent VPS — S54
2. systemd service VPS — S54
3. VPS TLS + nginx — S54
4. VPS monitoring + alerting — S54+
5. LT-1 Kudos-v2 — sprint dedie (S55+)
6. LT-7 self-hosted build — S54-S55
7. Events SSE daemon-native — post-v1.0
8. MCP server Rust — post-v1.0
9. Pagination SQL-side — S54+
10. Test infra mk_state() — S54+
11. Deploy scripts rewrite — S54
12. Load testing / benchmark P2P — post smoke test

---

## §6 Risks (rappel kickoff §9)

| # | Risque | Mitigation |
|---|---|---|
| R1 | Build macOS ARM echoue | Fix inline, deps systeme iteratives |
| R2 | Build VPS echoue (RAM) | CX33 = 8GB, swap si besoin |
| R3 | iroh P2P NAT traversal echoue | Relay EU n0, VPS IP publique |
| R4 | Daemon crash Linux runtime | Debug SSH, logs -vv |
| R5 | Flaky test pre-existant | Monitorer, non imputable S53 |
| R6 | unsafe set_var complexe | Grep exhaustif, wrapping mecanique |
