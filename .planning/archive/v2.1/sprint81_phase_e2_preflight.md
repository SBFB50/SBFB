# Sprint 81 Phase E2 — Préflight G8 (Workflow ultracode) — PLAN B C8 « zéro-n0 »

> **Verdict : PLAN-ADAPT.** La Phase E2 (nom canonique — regex README §4 `Phase [A-Z]+[0-9]?`,
> précédent A3/A4 ; **« E' » = alias prose** des artefacts, E-core commité `efb9667` « Phase E ») livre
> le **PLAN B C8 intégral** (préflight E §7.3 a-e, split acté par E-core). Six scans + vérifications
> adversariales convergent : **le code neuf est 100 % dans la surface API par défaut d'iroh 1.0.1,
> 0 dep runtime neuve, 0 bump wire, aucun Day-0 touché, aucun arbitrage PO** → **pas de
> DESIGN-CONFLICT**. Mais ce n'est **pas un simple EXECUTE** : les scans remontent **six adaptations
> matérielles** à la lettre §7.3 qui DOIVENT atterrir dans le code/runbook sous peine de régression ou
> d'échec de DoD :
>
> 1. **[CODE — piège load-bearing] Préservation du `MemoryLookup`.** `clear_address_lookup`
>    (`endpoint.rs:585`) vide **TOUT** le vecteur `address_lookup`, **y compris** le `memory_lookup`
>    poussé additivement en `node.rs:318`. Ce lookup est **porteur** : il résout les endpoint-ids
>    depuis les adresses hors-bande des tickets blob/doc (chemin seed/shard S75, `sans` pkarr). Le
>    code neuf zéro-n0 **DOIT re-pousser `memory_lookup`** après tout `clear`, sinon le dial
>    ticket-based casse **silencieusement**. Le §7.3(b) de la lettre ne le mentionne pas — surfacé par
>    4 scans (S1a/S1a2/S3/S4-7).
> 2. **[CODE — fail-loud coupling] Le mode zéro-n0 doit COUPLER discovery-override ET relais custom.**
>    Gate ON mais `SBFB_CUSTOM_RELAYS`/`relays.json` vide → l'endpoint **home toujours sur les relais
>    n0** (zéro-n0 partiel qui défait l'objectif EOL 30/09). Le gate doit **refuser de booter**
>    (fail-loud, root-cause), jamais dégrader en silence. Lie le verrou-3 (ancre VPS `[seed]` vide ne
>    doit pas booter mal-provisionnée).
> 3. **[TEST] L'assert-config pur ne suffit PAS à prouver hermétiquement « n0 retiré ».** Le `Builder`
>    d'iroh expose `address_lookup` comme `Vec` **privé sans getter pré-bind** (correction S4-9) →
>    aucun test hermétique ne peut inspecter le vecteur assemblé. **Résolution** : factoriser la
>    décision zéro-n0 dans une **fonction pure** (`env → DiscoveryPlan | erreur`, miroir de
>    `relay_config::load_relay_map` + pattern `EnvSnapshot`), unit-testée hermétiquement (parse,
>    validation, fail-loud). La preuve E2E « l'endpoint omet réellement n0 » n'est hermétique **QUE**
>    via `DnsPkarrServer` (dev-dep `test-utils`) — décision à trancher (Tier A vs A+B, §5).
> 4. **[INFRA] Le relais d'acceptance doit être un relais PROD TLS sur host/IP DÉDIÉ, pas `--dev`.**
>    Trust runtime = WebPKI-only (Phase E) → le relais exige un A-record public + ACME-joignable `:80`
>    pour un cert valide. `iroh-relay --dev` (HTTP nu `localhost:3340`) est insuffisant cross-machine ;
>    et `validate_relay_url` **rejette le loopback hors `SBFB_DEV_MODE=1`** + **impose https**
>    (`relay_config.rs:198-226`). De plus l'ACME du relais veut `:80/:443` + QUIC UDP `7842` brut →
>    **conflit avec le nginx VPS mono-IP** (`:80`) → **host/IP dédié** (aligne « basculer la flotte »
>    25/08). Corrections S1a/S1a2 intégrées.
> 5. **[INFRA — clarification #1 en impact] Le serveur pkarr HTTP est un service EXTERNE, NON-locké,
>    OBLIGATOIRE dans les DEUX options (d).** `iroh-dns-server` (= ce qu'EST `dns.iroh.link`) est
>    **absent du `Cargo.lock`** ; le crate `iroh-relay` locké couvre le plan **relais/connectivité**,
>    **PAS** le plan **pkarr/discovery**. `PkarrPublisher` PUT et `PkarrResolver` GET frappent tous
>    deux ce serveur pkarr. « HTTP-only » n'évite **pas** un serveur — il évite seulement la moitié
>    **autorité DNS**. Le provisioning (d) est **mandatory quelle que soit** la décision résolveur.
> 6. **[DOC] Runbook NEUF, jamais réutiliser `PKARR_RELAY_OPS.md` verbatim.** Celui-ci (S19) vise un
>    **autre outil** (image pubky `pkarr-relay`, DHT Mainline `6881`), topologie/ports/archi
>    différents. Écrire `docs/release/IROH_SELFHOST_OPS.md` neuf + note supersede/cross-ref.
>
> **Décision design (d) instruite → Option B (PkarrResolver HTTP)** pour le chemin de résolution de
> l'endpoint (argumentée §3 ; les DEUX options font tourner `iroh-dns-server` et publient par HTTP
> PUT — seul le chemin RÉSOLUTION diffère). Ce n'est **pas** un DESIGN-CONFLICT : les deux briques
> existent en 1.0.1, aucun Day-0, aucun arbitrage PO (précédent : le préflight E §6 a explicitement
> délégué cette décision à E'). `presets::N0` **reste le défaut** ; C8 **ajoute** un mode zéro-n0
> gated-env, ne retire rien. 0 bump wire (23 `DOMAIN_*_V1`, `sbfb/seed/0` + `sbfb/shard/1` verbatim,
> tous `*_FORMAT_VERSION=1`), iroh strictement seul (`test-utils` = dev-dep envisageable seulement),
> toolchain 1.94, verrous S74/S75 intacts, duress **non re-gaté** (§4).
>
> G8 : 6 scans (S1a API transport / S1a2 discovery+infra+PLAN B / S1b deps-CVE-lock / S2 décisions
> historiques / S3 threat model / S4 wire+call-sites) + 6 vérifications adversariales. Bilan :
> **5 EXECUTE-local + 1 PLAN-ADAPT (S1a2) ; 1 REFUTED matériel (S1b-6, faux-négatif RUSTSEC hickory) ;
> corrections de sources intégrées comme faits ; manques absorbés** (memory_lookup, fail-loud, T1
> pure-fn, PROD-TLS-host-dédié, serveur pkarr externe-mandatory, hickory-0119).

---

## 1. Rappel de la lettre + gates calendaires

**Nom canonique = « Phase E2 »** (E-core `efb9667` = « Phase E » → le split-off C8 = E2, suffixe chiffre
regex README §4, précédent direct A→A3/A4 canonisés « pour le regex de phase » `git show 7d6b9ea`).
« E' » reste l'alias de prose dans les artefacts.

**Périmètre (préflight E §7.3 a-e — la source contractuelle)** :

- **(a)** Runbook committé + templates config (`relays.json` / `SBFB_CUSTOM_RELAYS`).
- **(b)** CODE NEUF override discovery `node.rs` gated par env (`clear_address_lookup` +
  `PkarrPublisher::builder(self_url)` + `PkarrResolver::builder(self_url)` + `RelayMode::Custom`).
- **(c)** Relais iroh self-hosted (`iroh-relay 1.0.1` feature `server`, MÊME crate lockée, OU Docker)
  sur l'**ancre VPS Hetzner** existante.
- **(d)** pkarr/DNS self-hosted (`iroh-dns-server`) OU bascule `PkarrResolver` HTTP — **DÉCISION
  DESIGN À TRANCHER** (§3).
- **(e)** Acceptance zéro-n0 LIVE 2-nœuds = artefact **T2** JSON machine-lisible
  (`PASS`/`BLOCK{diagnosis}`/`RIG-ABSENT`), RIG-gated, RIG-ABSENT traçable acceptable.
- **(+)** Éventuel dev-dep `iroh features=["test-utils"]` pour `DnsPkarrServer` — **PAS code-free, à
  peser** (§5).

**C8 — RATIFIÉ PO 2026-07-02 (`sprint81_kickoff.md:76-82`)** : plan B relais/discovery self-hosted
**PRÉ-PROVISIONNÉ** + 3 gates calendaires : **01/08** corps S81 pas ouvert → provisionner
immédiatement ; **25/08** Phase F pas PASS → basculer la flotte sur le plan B ; **15/09** Phase H pas
faite → plan B **ACTIF** (2 sem. de vérif zéro-n0 avant l'**EOL n0 30/09**). Aujourd'hui **2026-07-05
< 01/08** → fenêtre de provisionnement **ouverte**, provisioning live infra = fenêtre avant 25/08.

**D2-vs-C8 réconcilié** (§7.2 préflight E) : D2 = relais self-hosted **OPTIONNEL** (résilience) ; C8 =
hedge **OBLIGATOIRE** pré-provisionné face à l'EOL. `presets::N0` conservé par défaut, C8 **ajoute** un
mode activable. Aucun Day-0 touché.

---

## 2. Le vrai périmètre, item par item (evidence-adossé, corrections adversariales appliquées)

### (b) CODE NEUF override discovery — surface 100 % API-défaut, entièrement neuve, localisée

Toute la brique existe dans la surface **par défaut** d'iroh 1.0.1 (aucun `#[cfg(test)]`/`#[cfg(feature)]`
sur les symboles) et est **génuinement neuve** dans `crates/` :

| Primitive | Preuve vendored 1.0.1 | Occurrences prod SBFB |
|---|---|---|
| `Endpoint::builder(preset)` (preset appliqué à la CONSTRUCTION, avant tout `.method()` chaîné) | `endpoint.rs:180-188` (`Builder::new`→`empty().preset().apply`) ; `presets.rs` `N0::apply` = `Minimal.apply` + `address_lookup(PkarrPublisher::n0_dns())` **inconditionnel** + `[not(wasm)] DnsAddressLookup::n0_dns()` / `[wasm] PkarrResolver::n0_dns()` + `relay_mode(default_relay_mode())` (vérifié source) | `node.rs:318` (**seul builder du workspace**) |
| `clear_address_lookup(self)` → `self.address_lookup.clear()` (**vide TOUT**) | `endpoint.rs:585-588` | **0** |
| `address_lookup(...)` → `self.address_lookup.push(...)` (**additif, jamais retire**) | `endpoint.rs:605-608` | 1 (`node.rs:318` `memory_lookup`) |
| `presets::Minimal` (crypto-provider seul, `RelayMode::Disabled`, 0 lookup) | `presets.rs:57-79` (partage `#[cfg(with_crypto_provider)]` avec N0 → atteignable) | **0** |
| `PkarrPublisher::builder(Url)` → `AddressLookupBuilder` (auto-tire `secret_key` + `tls_config` + `dns_resolver`) | `pkarr.rs:290` + `:239-251` | **0** |
| `PkarrResolver::builder(Url)` → `AddressLookupBuilder` (auto-tire `tls_config` + `dns_resolver`, **PAS** `secret_key` — un résolveur ne signe pas ; correction S1a-7) | `pkarr.rs:507` + `:472-484` | **0** |
| `RelayMode::Custom(RelayMap)` | `endpoint.rs:1922/1933` | `node.rs:350` (via `SBFB_CUSTOM_RELAYS`, **réutilisé**) |

**Base recommandée = `presets::Minimal`** plutôt que `N0 + clear_address_lookup` (S1a-3 CONFIRMED) :
Minimal ne câble **rien** de n0, donc **rien à oublier de clear** — strictement moins de footguns. Mais
Minimal met `RelayMode::Disabled` → le mode zéro-n0 **DOIT** aussi poser `RelayMode::Custom` (dimension
relais **déjà résolue** par `SBFB_CUSTOM_RELAYS`, `node.rs:343-351`) **ET re-pousser `memory_lookup`**
(adaptation #1, §4). Sur base Minimal les deux branches (N0 défaut / Minimal zéro-n0) retournent le
`Builder` concret → le branchement compile (S1a-3 vérifié).

**Publish-side = dépendance n0 égale** (nuance sous-articulée S2/S3, vérifiée source) : `N0::apply`
ajoute `PkarrPublisher::n0_dns()` **inconditionnellement** (les 2 branches wasm/non-wasm) → le nœud
**publie** son adresse au relais pkarr n0, pas seulement il résout. Un vrai zéro-n0 doit **aussi cesser
de publier** vers n0 : `clear_address_lookup` retire publisher **ET** lookup (additif-push) — d'où la
base Minimal + re-attache explicite du publisher self-hosted.

### (a) Runbook + templates — 0 format persisté neuf

- `relays.json` : **schéma pré-existant depuis S18** (`RelayListFile{relays:[{url}]}`,
  `RELAYS_FILE_NAME="relays.json"`, `relay_config.rs:78-103`) ; `load_relay_map` lit env (précédence)
  puis `~/.sbfb/relays.json`. E2 **documente** ce contrat, ne fige aucun format.
- Templates à livrer : `SBFB_CUSTOM_RELAYS` (réutilisé, relais self) ; `relays.json` (existant) ; le
  **nouveau flag env zéro-n0** + URL pkarr self (à nommer en `pub const *_ENV: &str = "SBFB_*"`,
  README §6.9 — **ne PAS** surcharger `SBFB_PKARR_RELAYS` [faux ami canari, ci-dessous] ni
  `SBFB_CUSTOM_RELAYS` [relais seul]). Tous **additifs, 0 format persisté, 0 bump wire** (URL `String`
  + env).
- **Faux ami confirmé** : `SBFB_PKARR_RELAYS` (`pkarr_resolver.rs:69`) ne nourrit **QUE** le canari
  quorum anti-eclipse browse (`load_quorum_resolvers_from_env` → `runtime.rs:468`
  `with_quorum_resolvers`), **jamais** la discovery de l'endpoint (`node.rs:318` = seul site). De
  même `dns_fallback.rs` ne nourrit que `BrowseAggregator`. Les régler ne retire **aucune** dépendance
  n0 → le CODE NEUF (b) est **réellement requis** (provisioning + env existants insuffisants).

### (c) Relais self-hosted — brique lockée, mais PROD TLS sur host dédié

- `iroh-relay 1.0.1` = **MÊME crate déjà lockée** (`Cargo.lock:4160`), feature `server`
  (`clap+rcgen+tokio-rustls-acme+toml+tracing-subscriber`, `iroh-relay-1.0.1/Cargo.toml:71-92`), bin
  `iroh-relay` (`required-features=["server"]`, `:119-122`). Wire-compat 1.0.1 par construction.
- Ports défaut (`iroh-relay defaults.rs`) : HTTP `80`, HTTPS `443`, QUIC addr-discovery `7842/UDP`,
  metrics `9090`. `--dev` = HTTP nu `localhost:3340` (ignore TLS). Prod = TOML `--config-path` avec
  `CertMode` Manual|LetsEncrypt|Reloading ; ACME HTTP-01 (contact email, `prod_tls` défaut true).
- **Contrainte réseau (adaptation #4)** : l'ACME veut `:80/:443` + QUIC UDP `7842` brut → **conflit avec
  le nginx VPS mono-IP** (`:80`, `deploy/nginx-nexus.conf:10`). Le relais fait son propre TLS et QUIC
  exige de l'UDP direct → **host/IP dédié** (le plus propre, aligne le gate 25/08 « basculer la
  flotte » = flotte relais séparée). Nuance S1a2-corr : le trafic HTTP/WS du relais **peut** être
  TLS-terminé en amont (`Manual` cert / `dangerous_http_only`), les blockers **durs** sont (i) QUIC
  UDP `7842` exposition directe + (ii) ACME `:80/:443`.
- **Contrainte SBFB** : `validate_relay_url` (`relay_config.rs:198-226`) impose **https** + **rejette
  loopback** hors `SBFB_DEV_MODE=1` → le relais VPS doit servir **HTTPS non-loopback (cert réel)** ;
  `--dev localhost:3340` n'est utilisable qu'en smoke local avec `SBFB_DEV_MODE=1`. Le schéma
  `relays.json` est minimal (url seule, pas d'override port QUIC, `relay_config.rs:80-88`) → le relais
  self doit tourner sur `DEFAULT_RELAY_QUIC_PORT=7842` (vérifié `iroh-relay defaults.rs:7`).

### (d) pkarr/DNS self-hosted — décision instruite §3

Voir §3. **Clarification #5 (le plus fort impact)** : `iroh-dns-server` (crates.io max_stable=1.0.1,
même batch de release `2026-06-29` que iroh 1.0.1, desc « A pkarr relay and DNS server ») est **absent
du `Cargo.lock`** (grep vide, S1b-4 CONFIRMED). C'est un **binaire externe** obligatoire dans les DEUX
options — le crate `iroh-relay` locké ne le couvre PAS.

### (e) Acceptance — T2 LIVE RIG-gated, scoping load-bearing

- Artefact **T2** JSON, **MÊME shape** que `sprint81_t2_e_discovery_survival.json` (`kind:
  "named-live-check"`, `paliers.<x>.verdict ∈ {PASS/BLOCK{diagnosis}/RIG-ABSENT/NOT-RUN}` + `observed`).
  Nom : `sprint81_t2_e2_zero_n0*.json` (suite A3/A4/E).
- **Scoping (S3-6, CONFIRMED)** : l'artefact **prouve** que le réseau converge+sert **sans aucun
  service n0** = résilience à l'**EOL n0 30/09** (le hedge C8 marche). Il **ne prouve PAS** la
  résilience à la **mort du VPS opérateur** — au contraire zéro-n0 **concentre** relais+pkarr+ancre sur
  un VPS. **COMPLÉMENTAIRE** et **non redondant** avec l'acceptance S75 « survives-VPS-death » (qui a
  tourné **sous `presets::N0`**, n0 vivant pour la discovery). Aucune des deux ne teste « le VPS
  opérateur meurt en mode zéro-n0 » = le vrai SPOF résiduel. **Énoncer précisément** pour ne pas
  sur-lire l'artefact.
- **Cadrage convergence-vs-delivery (S1a2-ACCEPT-01)** : cadrer l'acceptance sur la **CONVERGENCE
  DISCOVERY** (endpoint A résout B via pkarr self-hosted + dial via relais self-hosted), **distincte**
  du blocker carry S77 WAN task-delivery (`SeedAnnounced peer_count:0`) qui peut brouiller un run live.
- **RIG** : VPS Hetzner (relais sur host/IP dédié PROD TLS + `iroh-dns-server`) + 2 nœuds SBFB (PC
  Windows + VPS/Mac, cf. `live_acceptance_setup`). **RIG-ABSENT traçable acceptable** si matériel
  absent.

---

## 3. Décision design (d) instruite → **Option B (PkarrResolver HTTP)** pour le chemin de résolution

**AXE clarifié par les presets (S1a2-DESIGN-D-01, vérifié source `presets.rs`)** : le **PUBLISHER est
identique** dans les 2 options (`PkarrPublisher` fait TOUJOURS un HTTP PUT vers le pkarr relay). Seul le
**RESOLVER** diffère : non-browser N0 = `DnsAddressLookup::n0_dns()` (requête DNS, `presets.rs:131-134
#[cfg(not(wasm_browser))]`) ; browser N0 = `PkarrResolver::n0_dns()` (HTTP GET `/pkarr`). Donc **Option A
et Option B font toutes deux tourner `iroh-dns-server` et publient par HTTP PUT** ; l'**unique delta =
chemin de résolution** (HTTP GET [B] vs DNS query [A]).

**Recommandation = Option B (`PkarrResolver::builder(self_url)` HTTP)**, raisons evidence-adossées
(l'argument « délégation NS » du scan S1a2 est **RETIRÉ** — `DnsResolver::with_nameserver` peut pointer
directement l'IP du serveur DNS self, sans délégation NS publique, correction S1a2) :

1. **Transport symétrique** : PUT HTTP + GET HTTP sur le **même** endpoint `/pkarr` → **un seul port
   (443), un cert, une règle reverse-proxy** (co-loge derrière le proxy VPS existant). Option A ajoute
   un chemin DNS query côté résolveur non-browser.
2. **Parité browser/wasm** : `PkarrResolver` **EST** le chemin browser (`DnsAddressLookup` est
   `#[cfg(not(wasm_browser))]`) → Option B fait résoudre **identiquement** le daemon natif ET un futur
   client wasm/shell contre la **même** infra self-hosted ; Option A laisse les browsers incapables de
   résoudre le self-hosted.
3. **Surface ops/attaque minimisée** : pas de serveur DNS **autoritaire** (`:53`) à opérer/durcir.
   (Nuance S3-corr : `iroh-dns-server` est **autoritaire**, pas récursif ouvert → risque
   d'amplification matériellement plus bas mais non nul via réflexion.)
4. **Empreinte CVE (linkage S1b, §9)** : le chemin **DNS résolveur** non-browser passe par
   `hickory-proto 0.24.4` (**RUSTSEC-2026-0119** affecté, non-patché dans notre lock) ; **Option B**
   (HTTP `PkarrResolver` → `reqwest`/`PkarrRelayClient`) **ne touche AUCUN** chemin hickory DNS →
   **sidestep** l'exposition entièrement.

**Clarification NON négociable à ne PAS perdre (le plus haut impact des 6 scans)** : **les DEUX options
exigent le serveur pkarr HTTP** (`iroh-dns-server`, externe + non-locké) **pour publish ET resolve**.
« HTTP-only » **n'élimine pas** un serveur — il élimine la moitié **autorité DNS**. Le provisioning (d) =
faire tourner `iroh-dns-server` sur le VPS, **mandatory** quelle que soit l'option.

**`DnsAddressLookup` reste ADDITIF** (empilable plus tard sans bump wire, `endpoint.rs:605` push) : si
un jour le cache DNS/TTL importe à l'échelle, on l'ajoute — pas maintenant (single-anchor pré-launch ;
TTL « ignored by iroh-dns-server » `pkarr.rs:136-141`).

**Statut** : **décision de design de phase, PAS un DESIGN-CONFLICT** — les 2 briques existent en 1.0.1,
aucun Day-0, aucun arbitrage PO. Le main thread **ratifie Option B** (pré-tranchée ici).

---

## 4. Contraintes code load-bearing (à honorer dans (b))

1. **Re-pousser `memory_lookup` (adaptation #1)** — `clear_address_lookup` vide tout ; le chemin
   ticket-based (`shard.rs:465` `memory_lookup().add_endpoint_info`, seed S75) casse sinon. Sur base
   Minimal : `Endpoint::builder(presets::Minimal).address_lookup(memory_lookup.clone())` **PUIS**
   `PkarrPublisher::builder(self)` + `PkarrResolver::builder(self)` + `relay_mode(Custom)`.
2. **Fail-loud coupling (adaptation #2)** — gate zéro-n0 ON **sans** relais custom → **refuser de
   booter** (jamais home-sur-n0 silencieux). `RelayMode::Custom` sur une map vide erre au bind
   (`endpoint.rs:550-552`) ; préférer une **erreur explicite** au niveau de la fonction de décision.
   Lie verrou-3 (ancre mal-provisionnée refuse, ne dégrade pas).
3. **Fonction de décision PURE (adaptation #3, pour la testabilité)** — factoriser `env →
   DiscoveryPlan | erreur` (miroir `load_relay_map`), consommée par `node.rs`, unit-testée
   hermétiquement (§5). Le `Builder` iroh n'a **pas de getter pré-bind** (`address_lookup` `Vec` privé,
   S4-9) → c'est le **seul** moyen dep-free de prouver hermétiquement la logique de décision.
4. **Garde de validation d'URL parité (S3-PLANB-3, requalifié M)** — appliquer la politique
   `validate_relay_url`-équivalente (https-only, reject loopback hors `SBFB_DEV_MODE`, fail-loud
   malformé) au **NOUVEAU** self-URL pkarr. **Correction adversariale** : c'est de la **défense en
   profondeur** (misconfig opérateur), **même classe** que la garde relais existante (~M, **pas H**) ;
   **ne PAS** prétendre fermer un gap sur `load_quorum_resolvers_from_env` (surface **différente** — le
   canari, fail-safe : une URL invalide **abort le boot**, ne redirige pas). La garde cible le
   **self-URL neuf**, pas le loader canari.
5. **Log LOUD LOCAL au boot (S3-PLANB-4)** — logger la posture discovery custom/dégradée sur le **MÊME
   canal local** que `node.rs:345` (`info! "using custom relay map"`) / `runtime.rs:481`, **jamais une
   émission wire**. Local stdout/journal = **duress-safe par construction** (le gating §15.1 concerne
   l'émission WIRE, pas les logs locaux).
6. **Chokepoint unique (S4-5)** — l'override posé dans `create_node_with_protocols` (`node.rs:306+`,
   seul `Endpoint::builder` prod/test) couvre **automatiquement** worker (`worker
   engine/runtime.rs:270`), daemon (`runtime.rs:357/380`), coordinateur — 0 câblage séparé. Seule
   exception `examples/two_nodes_docs_sync.rs:67` (`Endpoint::bind` direct, démo, acceptable).
7. **NON duress-gaté (S2-4/S3-7, décision contraignante)** — le mode zéro-n0 est un **substrat de
   connectivité de la MÊME classe que `SBFB_CUSTOM_RELAYS`** (lu au boot, appliqué **identiquement** en
   duress et hors-duress ; `nexus-core-rs` ne connaît **pas** le duress, l'endpoint est byte-identique
   par conception d'indistinguabilité). Le **gater créerait un FINGERPRINT transport observable** sous
   duress → viole l'anti-goal. **Aucun nouveau gate duress en E2.** Le dial des vrais pairs qui
   fuiterait est **déjà** bloqué par le gate Phase C `sync_set_entry_in_duress` **quel que soit** le
   backend discovery.

---

## 5. Plan de tests concret (delta chiffré honnête)

**Greffe** : la fonction de décision pure + ses tests vivent dans `crates/nexus-core-rs/src/` (module
override neuf OU `relay_config.rs`), pattern `ENV_GUARD: Mutex` + `EnvSnapshot::capture`
(`relay_config.rs:245`) pour sérialiser la mutation d'env process-globale — **obligatoire** sinon flake
cross-test (correction S4-8). **0 helper à hoister** (pas de dette WS-3/PD-5).

| Test | Type | Assertion | Garde | Statut |
|---|---|---|---|---|
| Décision zéro-n0 : parse env → `DiscoveryPlan` | GREEN | env valide → plan self-hosted (Minimal + relais custom + pkarr self URL) | hermétique, 0 réseau, `EnvSnapshot` | **Tier A — MANDATORY** |
| Validation self-URL pkarr | GREEN | https-only + reject loopback hors `SBFB_DEV_MODE` + fail-loud malformé (parité `validate_relay_url`) | hermétique | **Tier A — MANDATORY** |
| Fail-loud coupling | GREEN | gate ON + relais custom vide → **erreur explicite** (refuse boot) | hermétique | **Tier A — MANDATORY** |
| Tripwire pkarr (E-core, existant) | GREEN | `DEFAULT_PKARR_RELAY_URL == iroh N0_DNS_PKARR_RELAY_PROD` | hermétique | **déjà vert (E-core `+1`)** |
| `DnsPkarrServer` : 2 nœuds convergent zéro-n0 in-process | GREEN | endpoint override omet n0, résout via pkarr+relais in-process fake, **sans** `.preset()` (hand-build `PkarrResolver::builder(server.pkarr_url())`, miroir du chemin PROD B, S1a2-missed) | **exige dev-dep `iroh features=["test-utils"]`** | **Tier B — À TRANCHER** |
| Zéro-n0 2-nœuds LIVE | **T2 artefact** | 2 nœuds convergent via relais+pkarr self-hosted, 0 service n0 ; `b3` PASS | **RIG-gated → `RIG-ABSENT` traçable** | E2 LIVE |

**Explicitement NE PAS ajouter (zombies)** : handshake seed 2-nœuds, handshake shard admits/rejects,
pkarr parse/label/env → **déjà couverts et verts** (E-core §3 ; `pkarr_resolver.rs:204-318`,
`seed_protocol.rs:459+`, `shard.rs:515/537`).

**Tension testabilité (adaptation #3, à trancher)** :
- **Tier A** (dep-free) prouve la **logique de décision** (parse/validation/fail-loud) + le dial
  in-process reste couvert par les handshakes seed/shard existants. Il **ne prouve PAS** que
  l'endpoint **omet réellement n0** end-to-end (Builder opaque, pas de getter).
- **Tier B** (`DnsPkarrServer` via dev-dep `test-utils`) est le **SEUL** test hermétique qui prouve
  l'**override E2E** (2 nœuds convergent sans n0 via un pkarr+relais fake in-process). **Coût** : le
  dev-dep tire `iroh-relay/server` = **4+ crates NEUVES** dans le graphe de **TEST** (dont
  `tokio-rustls-acme`, ACME) ; **ne fuit PAS** en release (resolver v2, dev-dep d'une lib, S1b-3
  CONFIRMED) → invariant runtime « 0 dep neuve » **tenu**, mais coût compile test réel + unification
  feature workspace-wide sous `nextest --workspace`/`clippy --all-targets`.
- **Réassurance vérifiée source (dissout la peur du flip staging)** : `force_staging_infra()`
  (`endpoint.rs:1970`) est **env-only** (`IROH_FORCE_STAGING_RELAYS`), **PAS** `cfg(feature="test-utils")`
  — le doc-comment du preset N0 (« when the test-utils feature is enabled, this will use STAGING »)
  **surestime** la réalité du code. Ajouter le dev-dep `test-utils` **ne flippe PAS** le chemin N0
  défaut en STAGING ; le tripwire PROD (`const == const`, statique) **reste valide** quoi qu'il arrive.

**Recommandation** : Tier A **MANDATORY** (cheap, dep-free, mirror `relay_config`). **Tier B RECOMMANDÉ**
pour honorer le gate DoD « T1 E2E hermétique BLOQUANT » **sur le comportement cœur de l'override**
(dev-dep explicitement **envisagé** par la mission ; dev-only, pas un Day-0). **Si Tier B décliné** :
E2 ship Tier A + T2 LIVE et **ne DOIT PAS** claimer que l'override est hermétiquement E2E-prouvé
(honnêteté). ↔ Décision main thread.

**Delta tests réaliste** : **E2 code = +3 Rust hermétiques (Tier A)** + **(optionnel +1 Tier B)** +
**1 artefact T2 LIVE** + runbook + templates. **Delta handshake réel = 0** (déjà vert, E-core). −0
zombies.

---

## 6. Provisioning / infra (a)(c)(d) — runbook neuf + route version-exacte

- **Runbook NEUF** `docs/release/IROH_SELFHOST_OPS.md` (ou `ZERO_N0_OPS.md`), front-matter SPDX +
  `written`/`last_validated`/`triggers_revalidate` (pattern `PKARR_RELAY_OPS.md:1-13`), charpente :
  prérequis / provisioning Hetzner host-dédié / **2 units systemd durcies distinctes** (relais +
  `iroh-dns-server`, user système non-root propre + `StateDirectory` propre, pattern
  `nexus-shell-daemon.service` S75 `systemd-analyze security 1.7`) / smoke test / monitoring / tear-down.
- **PIÈGE (adaptation #6)** : **NE PAS** réutiliser `PKARR_RELAY_OPS.md` (S19) verbatim — il vise
  l'image pubky `ghcr.io/sbfb50/pkarr-relay` (DHT **Mainline** `6881`), **outil/topologie différents**.
  Écrire neuf + note supersede/cross-ref (« pour zéro-n0 iroh 1.0.1 → `IROH_SELFHOST_OPS.md` »).
  **Correction adversariale** : la raison « wire-incompatible avec pubky » est **NON vérifiée** — les
  raisons **solides** = topologie de résolution différente (Mainline DHT vs `iroh-dns-server` DNS/HTTP),
  ports/archi différents, `PkarrRelayClient` HTTP propre à iroh, résolveur non-browser DEFAULT DNS-based.
- **Route version-exacte** : `cargo install iroh-relay --features server` (feasibility **confirmée** :
  edition 2024 / rust-version 1.91, toolchain 1.94 OK) — binaire 1.0.1 par construction ;
  `iroh-dns-server` cargo-install feasibility **NON indépendamment confirmée** (son `Cargo.toml` non
  vendored → **re-vérifier** avant de s'y fier, correction S1a2). Docker `n0computer/iroh-relay` (tags
  1.0.1) OK ; `n0computer/iroh-dns-server` tag **1.0.1 non prouvé** (tags visibles v0.35/v0.91). →
  **Recommander cargo-install pour la garantie de version**, Docker en commodité.
- **Templates** : `SBFB_CUSTOM_RELAYS` (env, relais self, `relay_config.rs:62`) + `~/.sbfb/relays.json`
  (S18) + nouveau flag env zéro-n0 + pkarr self-URL. Tous additifs, 0 bump wire.

---

## 7. Invariants & Day-0 (tenus)

- **`presets::N0` reste le DÉFAUT** — C8 **ajoute** un mode zéro-n0 gated-env, **ne retire rien**
  (`node.rs:318` inchangé par défaut). Verrou-3 : `[seed] keep_online_projects` **vide** par défaut sur
  l'ancre VPS **non touché** (le zéro-n0 câble discovery/relais, pas la seed-list ;
  `config.rs:280`/`:643`). Verrou-4 (seeder≠auteur) + invariant cardinal héberger≠publier
  **orthogonaux** au transport. **Verrous S74/S75 5/5** (verrou-1 write-side selector, verrou-2 honest
  count, verrou-5 subscribed-only) : le code zéro-n0 touche **uniquement** relais + `address_lookup`,
  jamais la seed accept-list / subscription-gating / directory ingest / app count.
- **iroh STRICTEMENT SEUL** — **0 dep runtime neuve**. `iroh-relay` = MÊME crate lockée ;
  `iroh-dns-server` = **binaire externe** (pas une dep). L'éventuel `test-utils` (Tier B) = **dev-dep**,
  ne fuit pas en release (resolver v2). **Précondition à écrire** : le dev-dep DOIT être en
  `[dev-dependencies]` — s'il migre en `[dependencies]` avec `features=["test-utils"]` il unifierait en
  release et casserait l'invariant (correction S1b-3).
- **0 bump wire SBFB (par construction)** — `sbfb/seed/0` (`node.rs:68`) + `sbfb/shard/1`
  (`node.rs:80`) verbatim ; **23** tags `DOMAIN_*_V1` (`canonical.rs`) inchangés ; tous
  `*_FORMAT_VERSION=1`. L'override ne modifie que des **URL `String` + env**. Le paquet pkarr est
  **FORMAT-invariant** vs `n0_dns()` — l'URL décide **où** le paquet est servi, jamais son schéma (le
  publish PUT / resolve GET encode une URL relais **valeur** différente à l'intérieur du `SignedPacket`,
  mais le **format** est identique → 0 touche `DOMAIN_*_V1` ; **dire « format-invariant », pas
  « byte-identique »**, correction S4-3).
- **Duress** : **non re-gaté** (§4-7) ; la publication d'adresse tourne déjà la clé chargée
  (leurre-sous-duress) exactement comme `presets::N0` aujourd'hui — zéro-n0 change seulement QUEL relais
  pkarr reçoit l'enregistrement (déjà-leurre). **Aucun nouveau gate.**
- Toolchain **1.94** ; tests hermétiques (Tier A/B) uniquement ; la sonde live = **acceptance T2**,
  **jamais un unit test**.

---

## 8. Risques résiduels

- **Régression auto-infligée si `memory_lookup` non re-poussé (P1, §4-1)** — dial ticket-based casse en
  **silence** ET la mitigation finding-5 de discovery silencieuse disparaît. **Bloquant à honorer.**
- **Zéro-n0 partiel si relais custom absent (P1, §4-2)** — home-sur-n0 silencieux qui défait l'EOL.
  Résolu par le **fail-loud coupling**.
- **T1 hermétique faible sans Tier B (P1, §5)** — le gate DoD « E2E hermétique BLOQUANT » sur le cœur
  de l'override n'est satisfait que par `DnsPkarrServer` (dev-dep). Décliner Tier B = ship sans preuve
  hermétique E2E du comportement + **interdit de le claimer**. ↔ Décision main thread.
- **SPOF opérateur + jointure métadonnées (P1 → CARRY G)** — relais+pkarr+ancre sur 1 VPS = SPOF +
  **jointure « pair Y connecté via mon relais à T » × « pair Y a fetch le hash X de mon ancre »**
  (S3-missed). Mitigation : **≥2 relais pkarr DISTINCTS** (qui doivent être **non-n0** dans le scénario
  EOL — `DEFAULT_PKARR_RELAY_URL` est **mort** après 30/09, correction S3-PLANB-2) + relais sur host
  distinct. **NB** : `SBFB_PKARR_RELAYS` (canari) ≠ redondance du **chemin résolveur** (2 code paths
  séparés, chacun mérite ≥2).
- **Casse SILENCIEUSE de discovery élargie (P1 → CARRY G, S3-PLANB-5)** — self-hosted (1 VPS) plus
  fragile que la flotte n0 4-régions → fenêtre morte silencieuse (warn-only iroh `pkarr.rs:386-397`,
  `grep discovery_health=0`) **s'élargit**. La sonde T2 couvre le **steady-state** (liveness), pas la
  **mort en vol** (pas de fault-injection). Sonde boot bruyante = remédiation tracée G.
- **`hickory-proto 0.24.4` RUSTSEC-2026-0119 (P2 → CARRY G)** — faux-négatif du scan S1b (refuted) :
  copie SBFB-directe (`hickory-resolver 0.24` `DnsFallbackResolver`) **affectée + non-patchée** (DoS
  encode O(n²)) ; copie iroh transitive 0.26.1 patchée. **Pas un bloqueur E2** (pré-existant depuis B,
  faible exploitabilité stub-resolver, E2 ne bump pas hickory) mais à **dispositionner**. Remédiation =
  bump `nexus-core-rs hickory-resolver 0.24→0.26` (aligne iroh, clôt 0119, collapse le double) — **bump
  cassant** (signature `PkarrRelayClient::new`/`DnsResolver`), **hors scope E2**. Option B (§3-4)
  **sidestep** l'exposition côté résolveur endpoint.
- **`quinn-proto 0.11.14` = RÉSOLU** RUSTSEC-2026-0037 (via `reqwest`, hors iroh/`noq`) — reformuler le
  carry mémoire « RÉSOLU ».
- **Piège `PKARR_RELAY_OPS.md` réutilisé (P2)** — outil différent (pubky/Mainline). Runbook neuf
  obligatoire (§6).
- **Env session (05/07)** : reboots tuent les shells ; `cmd | tail && ...` masque les exit codes.
  Non pertinent E2 hermétique ; pertinent pour l'acceptance E2 LIVE.

---

## 9. Carries sortants (E2 → G, K, I-J)

1. **G (THREAT_MODEL / doc, §15.x « Surface zéro-n0 self-hosted discovery, Sprint 81 Phase E2 »)** :
   (a) trust+availability relocation n0→opérateur (relais voit métadonnées connexion, jamais plaintext
   QUIC E2E-chiffré ; pkarr = autorité résolution n0→opérateur **bornée par paquet Ed25519-signé** →
   censor/stale/eclipse possible, **forge impossible**, « no central server » + BLAKE3 préservés) ;
   (b) **SPOF + jointure relais-métadonnées × ancre-contenu** (S3-missed) → **≥2 relais pkarr distincts
   non-n0** + host distinct ; (c) **silent-loss discovery élargie** (S3-PLANB-5) + sonde boot bruyante ;
   (d) **hickory-proto 0.24.4 RUSTSEC-2026-0119** faux-négatif à dispositionner + remédiation bump
   0.24→0.26 (hors E2) ; (e) `quinn-proto 0.11.14` = RÉSOLU (reformuler) ; (f) EOL n0 30/09 (remédié
   par upgrade) ; (g) « operator VPS meurt en zéro-n0 » = SPOF résiduel non testé (ni S75 ni E2).
2. **K (dette / doc)** : `age_witness.rs:6/21` doc-stale 0.98 ; repères plan périmés
   (`node.rs:348`→`:350` RelayMode::Custom, drift +2) ; convention env : chaque nouvelle var = `pub
   const *_ENV` nommée.
3. **I-J** : RTT multipath LIVE cross-machine (`shard.rs` UNVERIFIED-high-risk, PO C1) — **jamais
   claimer « shard SAUVÉ »**. E2 ne touche pas le shard.
4. **Veille continue** : re-jouer crates.io (1.0.2/0.102 ?) + RustSEC avant push live (05/07 : iroh
   1.0.1 / docs+gossip 0.101 / blobs 0.103 = plafonds stables ; `iroh-relay`/`iroh-dns-server` 1.0.1).

---

## 10. Restitution des scans (fan-out 6 + adversarial)

| Scan | Verdict-local | Findings clés retenus (après adversarial) | Adversarial |
|---|---|---|---|
| **S1a** API discovery-override | **EXECUTE** | brique 100 % API-défaut, 0 dep ; Minimal = base la plus sûre ; publisher n0 inconditionnel = dépendance n0 égale ; multi-relais natif (push N) ; **DnsPkarrServer = dev-dep test-utils** | 13 CONFIRMED. Corr : **S1a-7** resolver ne tire PAS `secret_key` ; « HTTP-only évite un serveur » **oversell** (le pkarr server externe reste requis) ; loopback-reject contraint le rig. **Missed clé** : serveur pkarr externe non-locké ; `.preset()` ≠ chemin prod ; **memory_lookup trap** |
| **S1a2** infra self-hosted + PLAN B | **PLAN-ADAPT** | iroh-relay feature server + iroh-dns-server 1.0.1 existent ; **contention ports VPS mono-IP → host dédié** ; runbook neuf ; option (d) instruite Option B | 15 CONFIRMED. Corr : arg **NS-delegation RETIRÉ** ; « wire-incompat pubky » NON vérifié ; « relais jamais derrière nginx » adouci (QUIC:7842 + ACME:80/443 = vrais blockers) ; iroh-dns-server cargo-install à re-vérifier. **Missed** : memory_lookup ; T1 hermétique ; relais **PROD TLS pas --dev** |
| **S1b** deps / features / CVE / lock | **EXECUTE (défect confiné S1b-6)** | code (b) 100 % API-défaut 0 feature ; test-utils = 4+ crates de TEST, **0 fuite release** ; iroh-dns-server hors lock ; pins au plafond ; **quinn-proto 0.11.14 = RÉSOLU** | 7 CONFIRMED, **1 REFUTED (S1b-6** hickory) : `hickory-proto 0.24.4` **PRÉSENT + SBFB-direct + AFFECTÉ** RUSTSEC-2026-0119 non-patché (le scan l'avait déclaré absent/non-applicable). Corr : test-utils **>4** crates transitives ; précondition dev-dep placement |
| **S2** décisions historiques | **EXECUTE** | verrou-3/4 non touchés ; **nexus-core NE connaît PAS le duress** → zéro-n0 **NON duress-gaté** (même classe que SBFB_CUSTOM_RELAYS) ; nom canonique **E2** ; convention env `*_ENV` | 11 CONFIRMED. Corr : citations wrong-crate (`node_directory.rs`/`dns_fallback.rs` = `nexus-core-rs`) ; `http.rs:6038` = **test** pas le gate ; grep DEVIATION garbled. **Missed** : 5 verrous (1/2/5 aussi orthogonaux) ; publish-side n0 sous-articulé |
| **S3** threat model | **EXECUTE** | delta = **relocation trust+availability** n0→opérateur, bornée cryptographiquement ; garde URL + log local loud = seuls codes ; acceptance ≠ survives-VPS-death (complémentaire S75) ; **rien à sur-coder** | 9 CONFIRMED. Corr : **S3-3 sévérité H→M** (défense-en-profondeur, pas anti-attaquant ; ne touche pas le loader canari) ; ≥2 relais **non-n0** ; DoS DNS autoritaire pas amplification ouverte. **Missed** : **memory_lookup trap** ; jointure métadonnées×contenu |
| **S4** wire + call-sites + greffe | **EXECUTE** | 0 bump wire par construction ; **1 seul Endpoint::builder** couvre worker/daemon/coord ; paquet pkarr FORMAT-invariant ; relays.json = format S18 ; T2 = planning pas wire ; **memory_lookup à re-pousser (A-CODER)** | 9 CONFIRMED. Corr : `RelayMode::Custom` = `node.rs:350` pas `:52`/`:348` ; **« format-invariant » pas « byte-identique »** ; fixture privée → réutiliser `create_node_with_protocols` public + `ENV_GUARD`. **Missed** : blast-radius feature test-utils workspace-wide ; **T1 hermétique inatteignable sans test-utils** ; coupling relais |

**Convergence** : 6 scans → **PLAN-ADAPT global** (5 EXECUTE-local + S1a2 PLAN-ADAPT). Le seul REFUTED
(S1b-6 hickory) **ne renverse pas** le verdict — il **ajoute un carry G** (dette CVE pré-existante). Les
manques **convergents** (memory_lookup ×4, T1 hermétique ×2, serveur pkarr externe ×3, fail-loud
coupling ×2) sont les **6 adaptations** du blockquote.

---

## 11. Commit shape (indicatif — E2)

`feat(core): Sprint 81 Phase E2 — mode zéro-n0 discovery-override gated-env + PLAN B C8 self-hosted
(presets::N0 conservé défaut, 0 bump wire)` (ex-« E' » en prose) — body : **CODE NEUF override discovery**
sur base `presets::Minimal` + re-push `memory_lookup` + `PkarrPublisher::builder(self)` +
`PkarrResolver::builder(self)` HTTP **[Option B tranchée]** + `RelayMode::Custom`, **gated par nouveau
`SBFB_*_ENV`** (jamais `SBFB_PKARR_RELAYS` faux-ami canari), **fail-loud si relais custom absent**,
**log local loud** (jamais wire), **non duress-gaté** (substrat connectivité, indistinguabilité) +
fonction de décision **pure** unit-testée hermétiquement (+3 Rust : parse/validation-parité/fail-loud ;
`EnvSnapshot`) **[+1 optionnel `DnsPkarrServer` si dev-dep `test-utils` adopté]** + **runbook neuf
`IROH_SELFHOST_OPS.md`** (supersede `PKARR_RELAY_OPS.md` pubky/Mainline) + templates
(`SBFB_CUSTOM_RELAYS`/`relays.json`/self-URL) + **infra VPS Hetzner** (`iroh-relay` feature `server`
PROD TLS **host/IP dédié** + `iroh-dns-server` externe pour publish+resolve) + **acceptance zéro-n0 LIVE
= artefact T2** (`sprint81_t2_e2_zero_n0.json`, convergence-discovery, RIG-gated, `RIG-ABSENT` traçable ;
**≠ survives-VPS-death** S75, complémentaire) + carries G(threat relocation + SPOF/jointure + silent-loss
+ hickory-0119 + quinn-proto RÉSOLU)/K(age_witness + `:350`)/I-J(RTT — **jamais « shard SAUVÉ »**) +
**0 bump wire** (23 `DOMAIN_*_V1`, ALPN verbatim, paquet pkarr FORMAT-invariant) + **iroh strictement
seul** (`iroh-dns-server` externe, `test-utils` dev-dep) + verrous S74/S75 5/5 + toolchain 1.94.

**Note ex-E'** : « E' » = alias prose des artefacts ; le nom de commit canonique est **Phase E2** (regex
README §4, précédent A3/A4).
