<!--
SPDX-License-Identifier: AGPL-3.0-or-later
written: 2026-07-05  # Sprint 81 Phase E2 (PLAN B C8)
last_validated: 2026-07-05  # initial write — versions crates.io re-vérifiées le jour même (iroh-relay 1.0.1, iroh-dns-server 1.0.1)
triggers_revalidate:
  - "bump du pin iroh workspace (=1.0.1) → réinstaller relais + dns-server à la MÊME version"
  - "EOL réel des services n0 (annoncé 2026-09-30) → ce runbook devient le chemin PRIMAIRE"
  - "gate calendaire C8 25/08 (bascule flotte) ou 15/09 (plan B ACTIF) atteint"
  - "changement de schéma config iroh-relay / iroh-dns-server upstream"
  - "provisionnement réel du premier host dédié → passer ce doc en last_validated live"
audited_findings: []
-->

# IROH_SELFHOST_OPS — runbook zéro-n0 (relais iroh + pkarr self-hosted)

Cible : opérateur d'un nœud SBFB (ancre VPS ou mainteneur), ~1 h pour un
premier déploiement fonctionnel. Sprint 81 Phase E2, décision PO **C8** :
plan B pré-provisionné face à l'**EOL des services n0 le 2026-09-30**
(gates calendaires : 01/08 provisionner ; 25/08 basculer la flotte si la
Phase F n'est pas PASS ; 15/09 plan B ACTIF).

**Objectif** : le réseau SBFB tient — publication d'adresse, résolution de
pairs, connectivité relayée — **sans aucun service n0 vivant**. Deux
services self-hosted remplacent la flotte n0 :

| Plan | Service n0 remplacé | Binaire self-hosted | Rôle |
|---|---|---|---|
| Connectivité | `*.relay.n0.iroh.link` (4 relais) | `iroh-relay` (feature `server`) | relais QUIC/WSS + address discovery |
| Discovery | `dns.iroh.link/pkarr` | `iroh-dns-server` | pkarr relay HTTP : `PUT /pkarr` (publish) + `GET /pkarr` (resolve) |

> **Ne PAS confondre avec [`PKARR_RELAY_OPS.md`](PKARR_RELAY_OPS.md)**
> (Sprint 19) : ce dernier documente l'image pubky `pkarr-relay` adossée
> au DHT **Mainline** (port 6881) — un autre outil, une autre topologie de
> résolution, qui alimente le **canari quorum anti-eclipse**
> (`SBFB_PKARR_RELAYS`), jamais la discovery de l'endpoint. Pour le mode
> zéro-n0 iroh 1.0.1, c'est CE document qui fait foi.

> **Statut 2026-07-05 (soir)** : code client livré et testé (mode
> `SBFB_ZERO_N0`, cf. §5). **Décision PO 2026-07-05 : Topologie B**
> (§4.4) — les deux services co-logés sur l'ancre VPS existante
> (`sbfb-eu`, 135.181.42.188) derrière **Caddy** (qui y tient déjà
> :80/:443 pour `ci.sbfb.world` ; nginx présent mais inactif ; UDP 7842
> et :53 public libres — inspection 2026-07-05), coût 0 €. Action
> pendante : **2 A-records chez Porkbun** (zone `sbfb.world`) —
> `relay1` et `pkarr1` → `135.181.42.188` — puis déploiement §4.4 et
> replay du palier T2 (`.planning/active/sprint81_t2_e2_zero_n0.json`,
> RIG-gated). La Topologie A (host dédié) reste la cible « propre »
> post-répétition (QUIC addr discovery + répartition SPOF), à
> re-décider avant le gate 25/08.

---

## 1. Architecture cible

```
   Nœud A (Windows dev)                    Nœud B (Mac / VPS ancre)
   SBFB_ZERO_N0=1                          SBFB_ZERO_N0=1
        │  publish PUT /pkarr/{z32}             │
        ▼                                       ▼
   ┌─────────────────────────────────────────────────┐
   │  HOST DÉDIÉ (nouveau, IP publique propre)       │
   │  ├── iroh-relay      :80/:443 + UDP :7842       │
   │  └── iroh-dns-server :8443 https /pkarr         │
   └─────────────────────────────────────────────────┘
        ▲                                       ▲
        └── resolve GET /pkarr/{z32} ───────────┘
             puis dial via le relais self-hosted
```

- Le **client** (chaque nœud SBFB) publie ET résout via le pkarr
  self-hosted (`PkarrPublisher` + `PkarrResolver` HTTP — décision design
  Option B du préflight E2 : un seul port, un seul cert, parité
  browser/wasm, aucun chemin DNS résolveur côté endpoint). Le serveur DNS
  autoritaire d'`iroh-dns-server` tourne mais n'est **pas** requis par les
  clients SBFB ; `DnsAddressLookup` reste empilable plus tard (additif).
- Le publisher ne publie **que la relay URL** (filtre par défaut iroh) —
  aucune IP directe ne fuit vers le serveur pkarr.
- **≥ 2 relais pkarr distincts recommandés** (résilience + le canari
  quorum du browse mérite ses propres cibles non-n0 après l'EOL) ; un
  seul suffit pour démarrer.

## 2. Prérequis

| Ressource | Spécification | Note |
|---|---|---|
| Host — **Topologie A (dédiée)** | 1 vCPU / 2 GB (Hetzner CX22 ou équivalent) | **IP publique propre, PAS l'ancre existante** : le relais veut `:443` en direct (TLS + ACME TLS-ALPN-01) et `:80` (portail/redirect), le QUIC address-discovery veut l'UDP `7842` en direct — Caddy occupe déjà `:80`/`:443` sur l'ancre ; et co-loger relais + pkarr + ancre sur une seule machine aggrave le SPOF et la jointure de métadonnées (THREAT_MODEL) |
| Host — **Topologie B (co-logée, 0 €)** | l'ancre VPS existante | Les deux services en loopback DERRIÈRE le Caddy déjà en place (§4.4) : viable car le data-plane relais = WSS sur :443 (proxifiable) ; **trade-off : QUIC address-discovery désactivé** (exige l'UDP `7842` + TLS terminé par le relais lui-même) + SPOF concentré. Retenue par décision PO 2026-07-05 pour la répétition générale |
| DNS | 2 A-records, ex. `relay1.sbfb.world` + `pkarr1.sbfb.world` | même IP acceptable pour les deux services (ports distincts) |
| OS | Debian 12 / Ubuntu 24.04 LTS | pattern ancre S75 |
| Rust | toolchain stable ≥ 1.91 (1.94 recommandé, parité repo) | build des binaires — OU utiliser l'image Docker `n0computer/iroh-relay` (tag 1.0.1) ; **cargo-install recommandé pour la garantie de version exacte** |
| Ports entrants | TCP 80, 443, 8443 ; UDP 7842 | 9090 (metrics) reste loopback |

## 3. Installation des binaires (version-exacte)

Le pin workspace SBFB est `iroh =1.0.1` : installer les services à la
**même version** garantit la wire-compat par construction.

```bash
# Relais (même crate que celle déjà lockée par le workspace, feature server)
cargo install iroh-relay --version 1.0.1 --features server

# Serveur pkarr + DNS (ce que dns.iroh.link EST ; binaire externe,
# volontairement PAS une dépendance du workspace — invariant
# « iroh strictement seul »)
cargo install iroh-dns-server --version 1.0.1
```

Vérifié 2026-07-05 : les deux crates existent sur crates.io en 1.0.1
(même batch de release que iroh 1.0.1, 2026-06-29) ; `iroh-dns-server`
expose bien `GET` **et** `PUT` sur `/pkarr` (README upstream + routes).
`iroh-relay --dev` (HTTP nu `localhost:3340`) ne sert qu'au smoke local
avec `SBFB_DEV_MODE=1` — jamais cross-machine (la policy SBFB
`validate_relay_url` impose https + rejette le loopback hors dev).

## 4. Configuration serveur

### 4.1 `iroh-relay` — `/etc/iroh-relay/config.toml`

```toml
# Relais SBFB self-hosted (zéro-n0). Schéma : struct Config de
# iroh-relay 1.0.1 src/main.rs.
enable_relay = true
http_bind_addr = "[::]:80"            # captive portal / redirect HTTP
                                      # (l'ACME passe en TLS-ALPN-01 sur
                                      # :443 via tokio-rustls-acme)
enable_quic_addr_discovery = true     # QUIC UDP 7842 (address discovery)
metrics_bind_addr = "127.0.0.1:9090"  # loopback ONLY — le défaut iroh-relay
                                      # est [::]:9090 (toutes interfaces)

[tls]
https_bind_addr = "[::]:443"
hostname = ["relay1.sbfb.world"]
cert_mode = "LetsEncrypt"
contact = "ops@sbfb.world"
prod_tls = true
cert_dir = "/var/lib/iroh-relay/certs"

# access = "everyone" (défaut). Un allowlist/denylist/shared_token existe
# si un jour le relais doit être restreint — le laisser ouvert est le
# comportement n0 que l'on remplace.
```

### 4.2 `iroh-dns-server` — `/etc/iroh-dns-server/config.toml`

Adapté du `config.prod.toml` upstream. Seul le bloc `[https]` (endpoint
`/pkarr`) est requis par les clients SBFB ; le DNS autoritaire est un
bonus (DoH + futur `DnsAddressLookup`).

```toml
pkarr_put_rate_limit = "smart"

[https]
port = 8443
domains = ["pkarr1.sbfb.world"]
cert_mode = "lets_encrypt"
letsencrypt_prod = true

[dns]
port = 53
default_soa = "dns1.pkarr1.sbfb.world hostmaster.sbfb.world 0 10800 3600 604800 3600"
default_ttl = 30
origins = ["pkarr1.sbfb.world", "."]
rr_a = "<IP-du-host>"
rr_ns = "ns1.pkarr1.sbfb.world."

[mainline]
enabled = false
```

Topologie alternative (sans host dédié pour le pkarr SEUL) : le bloc
`[http]` d'`iroh-dns-server` peut binder `127.0.0.1:8080` derrière un
reverse-proxy TLS existant (le trafic `/pkarr` est de l'HTTP pur) — c'est
le RELAIS qui exige le host dédié (QUIC UDP direct + ACME), pas le
serveur pkarr.

### 4.3 Units systemd durcies (pattern ancre S75)

Deux units **complètes et installables** sont livrées dans le repo —
copier telles quelles, les headers contiennent la séquence d'install :

- [`deploy/iroh-relay.service`](../../deploy/iroh-relay.service)
- [`deploy/iroh-dns-server.service`](../../deploy/iroh-dns-server.service)

Chacune : user système dédié non-root, `StateDirectory=` propre, et le
bloc hardening canonique de
[`deploy/nexus-shell-daemon.service`](../../deploy/nexus-shell-daemon.service)
(`NoNewPrivileges`, `ProtectSystem=strict`, `RestrictAddressFamilies=AF_UNIX
AF_NETLINK AF_INET AF_INET6`, `SystemCallFilter=@system-service`,
`UMask=0077`). Unique delta vs l'ancre : les deux services binden des
ports < 1024 (`:80/:443` relais, `:53` DNS) →
`AmbientCapabilities=CAP_NET_BIND_SERVICE` +
`CapabilityBoundingSet=CAP_NET_BIND_SERVICE` (au lieu de vide). Viser
`systemd-analyze security` ≤ 2.0 (l'ancre S75 tient 1.7).

### 4.4 Topologie B — co-logée derrière Caddy sur l'ancre (décision PO 2026-07-05, 0 €)

État réel de l'ancre (`sbfb-eu`, inspection 2026-07-05) : Caddy actif
sur `:80`/`:443` (sert `ci.sbfb.world` → Woodpecker Docker loopback
`:8000`), nginx installé mais **inactif**, UDP `7842` et `:53` public
**libres**. DNS : zone `sbfb.world` gérée chez **Porkbun**
(`*.ns.porkbun.com`).

**Prérequis DNS (action opérateur, une fois)** — chez Porkbun, zone
`sbfb.world`, 2 enregistrements A : `relay1` → `135.181.42.188` et
`pkarr1` → `135.181.42.188`.

**Relais — config sans bloc `[tls]`** (voie supportée upstream : « TLS
is disabled if not present and the Relay server will serve all
services over plain HTTP » ; JAMAIS poser `dangerous_http_only` à la
main). `enable_quic_addr_discovery` DOIT rester `false` (il exige un
`[tls]`) — c'est le trade-off documenté §2 : les nœuds découvrent
leurs adresses via le protocole relais classique (mécanisme pré-1.0,
toujours supporté), pas via la sonde QUIC. Le port UDP `7842` du VPS
restera muet : le client tentera la sonde best-effort et
timeout-era en silence, non bloquant.

```toml
# /etc/iroh-relay/config.toml — Topologie B (loopback, TLS terminé par Caddy)
enable_relay = true
http_bind_addr = "127.0.0.1:3340"
enable_quic_addr_discovery = false
metrics_bind_addr = "127.0.0.1:9090"
```

**iroh-dns-server — bloc `[http]` loopback seul** (le `/pkarr` est du
HTTP pur ; ni `[https]` ni ACME côté service) :

```toml
# /etc/iroh-dns-server/config.toml — Topologie B
pkarr_put_rate_limit = "smart"

[http]
port = 8080
bind_addr = "127.0.0.1"

[dns]
port = 5353
bind_addr = "127.0.0.1"
default_soa = "dns1.pkarr1.sbfb.world hostmaster.sbfb.world 0 10800 3600 604800 3600"
default_ttl = 30
origins = ["pkarr1.sbfb.world", "."]
rr_a = "135.181.42.188"
rr_ns = "ns1.pkarr1.sbfb.world."

[mainline]
enabled = false
```

**Caddy — 2 blocs à ajouter** (`/etc/caddy/Caddyfile`, certs
automatiques ; Caddy gère l'upgrade WebSocket nativement pour le
data-plane WSS du relais) :

```caddyfile
relay1.sbfb.world {
    reverse_proxy localhost:3340
}
pkarr1.sbfb.world {
    reverse_proxy localhost:8080
}
```

Puis `systemctl reload caddy`. Les units systemd §4.3 restent
utilisables telles quelles (les binds loopback > 1024 n'utilisent pas
`CAP_NET_BIND_SERVICE`, qui reste sans effet nocif ; en Topologie B le
`[dns]` est déplacé sur `5353` loopback — le DNS autoritaire public
n'est PAS exposé, seul le `/pkarr` HTTP l'est via Caddy, ce qui suffit
à l'Option B client publish+resolve).

**Ce que la Topologie B ne donne PAS** (assumé, re-décision avant le
gate 25/08) : QUIC address-discovery ; répartition du SPOF (relais +
pkarr + ancre + CI sur UNE machine — la mort du VPS emporte tout) ;
la jointure de métadonnées relais×ancre chez le même opérateur reste
entière (THREAT_MODEL §15.x, carry Phase G).

## 5. Configuration client (chaque nœud SBFB)

Le mode zéro-n0 est **opt-in par nœud** et **fail-loud** : toute config
partielle refuse de booter (jamais de retombée silencieuse sur n0).
`presets::N0` reste le défaut quand `SBFB_ZERO_N0` est absent.

```bash
# Gate (1/true = on ; 0/false/absent = off ; toute autre valeur = refus de boot)
SBFB_ZERO_N0=1

# Pkarr self-hosted : publish (PUT) + resolve (GET). ≥ 2 URLs distinctes
# recommandées, séparées par des virgules. Policy : https obligatoire,
# loopback rejeté hors SBFB_DEV_MODE=1.
# (Topologie B : port 443 implicite via Caddy ; en Topologie A directe
# ce serait https://pkarr1.sbfb.world:8443/pkarr.)
SBFB_ZERO_N0_PKARR_RELAYS=https://pkarr1.sbfb.world/pkarr

# Relais self-hosted (knob S18 réutilisé — env OU ~/.sbfb/relays.json).
# OBLIGATOIRE quand SBFB_ZERO_N0=1 : sans relais custom le nœud
# refuserait de booter (sinon il resterait home sur la flotte n0).
SBFB_CUSTOM_RELAYS=https://relay1.sbfb.world
```

Équivalent fichier pour le relais (`~/.sbfb/relays.json`, schéma S18) :

```json
{ "relays": [ { "url": "https://relay1.sbfb.world" } ] }
```

**À ne PAS confondre** : `SBFB_PKARR_RELAYS` (sans `ZERO_N0`) alimente le
canari quorum anti-eclipse du browse, pas la discovery de l'endpoint —
le régler ne retire aucune dépendance n0.

Au boot, le nœud loggue sa posture (log local, jamais une émission
réseau) :

```
zero-n0 discovery override active: relays + pkarr self-hosted, no n0 service wired
```

## 6. Smoke test

```bash
# 1. Relais vivant (WSS handshake endpoint) :
curl -sI https://relay1.sbfb.world/ | head -1          # HTTP/2 200 (captive page)

# 2. Handler pkarr vivant (GET nu = 404/405, jamais un timeout — même
#    sémantique que la sonde T2 Phase E sur dns.iroh.link) :
curl -s -o /dev/null -w '%{http_code}\n' https://pkarr1.sbfb.world/pkarr

# 3. Boot d'un nœud SBFB avec les env §5 → chercher le log
#    "zero-n0 discovery override active" puis vérifier qu'un
#    PUT /pkarr/{z32} arrive dans le journal d'iroh-dns-server.

# 4. Convergence 2 nœuds : lancer un 2e nœud zéro-n0, publier une app,
#    vérifier le Browse croisé — c'est l'acceptance T2
#    (sprint81_t2_e2_zero_n0.json, RIG-gated).
```

## 7. Monitoring / tear-down

- Metrics relais : `curl -s 127.0.0.1:9090/metrics` (loopback only).
- La mort du pkarr relay est **silencieuse côté client** (warn-only dans
  iroh, aucun signal santé SBFB — carry THREAT_MODEL Phase G « sonde boot
  bruyante ») : surveiller les DEUX services côté serveur, pas côté
  client.
- Tear-down : `systemctl disable --now iroh-relay iroh-dns-server` ;
  retirer les env `SBFB_ZERO_N0*` des nœuds → retour au défaut
  `presets::N0` au boot suivant (tant que la flotte n0 vit encore).

## 8. Threat model (résumé — détail : THREAT_MODEL Phase G)

Le zéro-n0 **déplace** la confiance disponibilité/métadonnées de n0 vers
l'opérateur SBFB, il ne l'élimine pas : l'opérateur du relais voit les
métadonnées de connexion (jamais le contenu — QUIC chiffré de bout en
bout) ; l'opérateur du pkarr est l'autorité de résolution, **bornée par
la signature Ed25519** des paquets pkarr (censure/stale possibles, forge
impossible). Mitigations : host relais ≠ host ancre (jointure
métadonnées × contenu), ≥ 2 relais pkarr distincts, les verrous
anti-recentralisation S74/S75 ne sont pas touchés (le mode ne change ni
la seed-list, ni le directory ingest, ni le comptage).
