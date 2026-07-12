Audit fait sur `master`, `HEAD=70dd845`, diff non committé.

### Livrable 1 : `deny.toml [bans]` / convergence crypto
- Statut : CONFIRME
- Fichier(s) : `deny.toml:140`, `deny.toml:142`, `deny.toml:150`
- Evidence :
```text
140:[bans]
142:# iroh 1.0.1 (and 1.0.2) pin `ed25519-dalek = "=3.0.0-rc.0"`, so the
143:# lock carries two ed25519-dalek trees (2.2.0 SBFB-stable +
146:# multi-version groups total. Flip deferred as P2-AUDIT-2-RESIDUEL
150:multiple-versions = "warn"
```
- Commandes : `cargo tree -d --locked -i ed25519-dalek` retourne bien `specification 'ed25519-dalek' is ambiguous` avec `ed25519-dalek@2.2.0` et `ed25519-dalek@3.0.0-rc.0`. `cargo deny check bans` sort `bans ok` avec `DUPLICATE_WARNING_COUNT=72`. `git diff HEAD` ne contient pas `P2-AUDIT-2.*CLOSED`.

### Livrable 2 : advisories `deny.toml` / `Cargo.lock`
- Statut : CONFIRME
- Fichier(s) : `Cargo.lock:194`, `Cargo.lock:195`, `Cargo.lock:1370`, `Cargo.lock:1371`, `deny.toml:61`, `deny.toml:83`
- Evidence :
```text
194:name = "anyhow"
195:version = "1.0.103"
1370:name = "crossbeam-epoch"
1371:version = "0.9.20"
```
```text
61:ignore = [
83:{ id = "RUSTSEC-2026-0119", reason = "..."}
84:{ id = "RUSTSEC-2026-0098", reason = "...name-constraint laxity..."}
98:{ id = "RUSTSEC-2026-0195", reason = "...quick-xml...iroh..."}
```
- Commandes : scan du diff `Cargo.lock` = `AddedPackageHeaders=0`, `RemovedPackageHeaders=0`, seulement `anyhow 1.0.102→1.0.103` et `crossbeam-epoch 0.9.18→0.9.20`. `cargo deny check advisories` = `advisories ok`. Le code confirme le claim DNS fallback : `crates/nexus-core-rs/src/dns_fallback.rs:127` `enabled: false`, endpoints Cloudflare/Google à `130-151`, env opt-in à `319-333`. `cargo tree -i quick-xml@0.39.4 --target all` confirme `quick-xml -> plist -> netdev -> netwatch -> iroh`.

### Livrable 3 : commentaire pins iroh / MSRV
- Statut : CONFIRME
- Fichier(s) : `Cargo.toml:24`, `Cargo.toml:33`, `Cargo.toml:36`, `Cargo.toml:48`
- Evidence :
```text
24:rust-version = "1.91"
36:# re-check (crates.io API, 2026-07-08): iroh 1.0.2 exists
39:# ed25519-dalek "=3.0.0-rc.0", so it would not unblock the
48:iroh = "=1.0.1"
```
- Commandes : `git diff HEAD -- Cargo.toml` ne touche pas `rust-version`. `cargo info iroh@1.0.2 -v` confirme `ed25519-dalek@=3.0.0-rc.0`; API crates.io confirme `created_at: 2026-07-06T21:29:52Z`.

### Livrable 4 : `THREAT_MODEL.md`
- Statut : CONFIRME
- Fichier(s) : `docs/security/THREAT_MODEL.md:22`, `:195`, `:197`, `:823`, `:1091`, `:1120`, `:1600`
- Evidence :
```text
22:| **iroh stack** | ... iroh =1.0.1 / docs 0.101 / gossip 0.101 / blobs 0.103
195:| E | RCE via deserialization iroh | ... Version pinnee =1.0.1 ... | **M** |
197:**Note S81 (upgrade ≠ audit)** : l'upgrade iroh 0.98→1.0.1 ne franchit
198:PAS Gate 1 / Gate 3 — **R-iroh-audit reste une zone rouge P0**
```
```text
823:nommage ... le front S80 consomme le SSE via
824:`fetch`+`ReadableStream` (jamais `EventSource`...)
827:TOUT, SSE inclus (`useTokenStream.ts:135` `credentials:
829:lui, ne PEUT pas poser d'en-tete custom). Le header `x-sbfb-token`
```
- Cross-check code : `tools/factory-operator/src/lib/useTokenStream.ts:134-138` utilise `fetch(... credentials: 'same-origin', headers: { accept: 'text/event-stream' })`, aucun `x-sbfb-token`. `crates/sbfb-factory/src/auth.rs:299-305` et `338-352` confirment header d’abord puis cookie gardé par `Sec-Fetch-Site`. §15.4 contient les rows/residuels demandés à `1115-1118`, `1120-1132`, `1136-1167`; entrée v15 à `1600-1619`.

### Livrable 5 : `EXTERNAL_AUDIT_SCOPE.md`
- Statut : CONFIRME
- Fichier(s) : `docs/security/EXTERNAL_AUDIT_SCOPE.md:82`, `:96`, `:134`, `:144`
- Evidence :
```text
82:| iroh (endpoint + pkarr discovery) | =1.0.1 (S81) |
83:| iroh gossip | 0.101.0 (S81) |
84:| iroh docs | 0.101.0 (S81) |
85:| iroh blobs | 0.103.0 (S81) |
```
```text
144:**Replay S81 Phase G (2026-07-08, lock at `70dd845` pre-commit)**
147:- `ed25519-dalek` → **ambiguous** : `2.2.0` ... et `3.0.0-rc.0`
154:- `aes-gcm 0.10.3` ; `frost-ed25519 3.0.0` ; `iroh 1.0.1` ;
155:  `iroh-blobs 0.103.0` ; `iroh-gossip 0.101.0` ; `iroh-docs 0.101.0`
```
- Commandes rejouées : versions conformes, et `cargo tree -p ed25519-dalek --depth 0 --locked` est ambigu.

### Livrable 6 : `HARDENING_ROADMAP.md`
- Statut : PARTIEL
- Fichier(s) : `docs/security/HARDENING_ROADMAP.md:3`, `:5`, `:28`, `deny.toml:71`
- Evidence :
```text
3:last_validated: 2026-07-08  # S81 Phase G ...
5:  - "iroh breaking release > 1.0.x OU iroh-docs 0.102+ ... OU yank ed25519-dalek..."
28:  - "2026-07-08 S81 Phase G supply-chain + docs securite : gate convergence crypto JOUE...
```
- GAP : l’entrée `audited_findings` existe et couvre convergence, advisories, TOOLCHAIN-LABEL et S75, mais `HARDENING_ROADMAP.md:28` regroupe les advisories hickory avec `DoS-class`. Cela contredit la précision correcte de `deny.toml:71-73`, où `0098/0099` sont des laxités de name-constraints, classe authentification, **NOT DoS**.

### Livrable 7 : `LOOPBACK_ENDPOINTS_TRUST_TIERS.md`
- Statut : CONFIRME
- Fichier(s) : `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md:3`, `:110`, `:111`, `:112`, `:123`
- Evidence :
```text
110:| `GET /api/terminal/ws` (**spawn + write stdin**) ... WebSocket **PTY interactif** ...
111:| `GET /api/git/diff` | ... Lecture seule ...
112:| `GET /api/gates` | ... Lecture seule ...
123:**Double transport du bearer ...
```
- Cross-check code : routes présentes dans `crates/sbfb-factory/src/operator_server.rs:198-200`; handlers lecture seule `handle_git_diff`/`handle_gates` à `1841-1856`. PTY interactif confirmé par `terminal.rs:54`, `:81`, `:170`. Cookie `sbfb_operator; HttpOnly; SameSite=Strict` posé à `operator_server.rs:321-324`; garde cookie dans `auth.rs:338-352`.

### Livrable 8 : `STORE_MIGRATION_OPS.md`
- Statut : CONFIRME
- Fichier(s) : `docs/release/STORE_MIGRATION_OPS.md:22`
- Evidence :
```text
22:1. **Snapshot tar AVANT toute migration réelle** (`NEXUS_GRID_ROOT`
24:   Snapshots : Windows PRIS (Phase B) ; **Mac PRIS 2026-07-08**
25:   (`sbfb-snapshots/s81-phase-b/mac-nexus-grid-pre-s81h.tar.gz`,
28:   prérequis snapshot ouvert pour la Phase H.
```

### Livrable 9 : contraintes négatives globales
- Statut : CONFIRME
- Fichier(s) : diff `HEAD`
- Evidence :
```text
CHANGED_COUNT=8
Cargo.lock
Cargo.toml
deny.toml
docs/release/STORE_MIGRATION_OPS.md
docs/security/{EXTERNAL_AUDIT_SCOPE,HARDENING_ROADMAP,LOOPBACK_ENDPOINTS_TRUST_TIERS,THREAT_MODEL}.md
```
- Commandes : `NO_RS_TS_TSX_IN_DIFF`, `NO_RUST_TOOLCHAIN_TOML`, `NO_EXTRA_VERSIONED_FILES`, `NO_BAD_UNTRACKED_FILES`. Untracked autorisés uniquement : `.planning/active/sprint81_phase_g_preflight.md` et `.planning/active/sprint81_phase_g_review.md`.

## Résumé final

- Total livrables : 9
- Confirmés : 8
- Gaps : 0
- Partiels : 1