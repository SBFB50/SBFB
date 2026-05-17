# Iroh, Babel et diffusion anti-censure sans Internet

Date: 2026-05-16
Statut: recherche factuelle, basee sur le repo actuel et la documentation Iroh officielle.

## Verdict court

La phrase correcte n'est pas "Iroh marche sans Internet" au sens absolu. La phrase correcte est:

> Le web public et les serveurs centraux ne sont qu'une couche de diffusion. Avec Iroh/SBFB, Babel peut aussi circuler par relais autogeres, reseau local, hotspot, Wi-Fi direct, USB, tickets hors-bande et caches locaux. Mais Iroh reste un protocole reseau: sans aucun lien IP, radio, cable, support physique ou canal de bootstrap, il ne peut pas joindre magiquement deux machines.

Donc:

- Oui: credible pour eviter une dependance exclusive au web public, a GitHub, a un serveur central ou a une plateforme de distribution.
- Oui: credible pour des pays censures si au moins un canal reste disponible: Internet degrade, relais amis, LAN, hotspot, Wi-Fi local, USB, Bluetooth ou autre passage hors-bande.
- Non: pas credible de promettre une communication "sans Internet" si on entend "sans aucun reseau, sans aucun support physique, sans aucun canal initial".

## Ce que dit Iroh officiellement

Iroh est une pile reseau modulaire en Rust pour construire des apps peer-to-peer. Les connexions sont chiffrees et authentifiees de bout en bout via QUIC/TLS, et Iroh cherche a etablir des connexions directes entre appareils quand c'est possible.

Sources officielles:

- https://docs.iroh.computer/what-is-iroh
- https://docs.iroh.computer/concepts/discovery
- https://docs.iroh.computer/concepts/tickets
- https://docs.iroh.computer/concepts/relays
- https://docs.iroh.computer/concepts/nat-traversal
- https://docs.iroh.computer/connecting/local-discovery

Points techniques importants:

- Les relais Iroh servent de point de rendez-vous et de fallback chiffre, mais ne lisent pas le contenu applicatif.
- Le NAT traversal permet souvent un chemin direct apres coordination initiale via relais.
- Les tickets Iroh embarquent les informations necessaires pour joindre un endpoint ou recuperer un contenu. Ils peuvent etre transmis hors-bande: QR code, message, fichier, cle USB, etc.
- La decouverte globale par defaut depend de DNS/Pkarr et donc d'une infrastructure Internet ou de resolvers accessibles.
- La decouverte locale mDNS existe cote Iroh, mais elle n'est pas activee par defaut et doit etre configuree explicitement.

Conclusion Iroh: Iroh reduit fortement la dependance a un serveur central, mais il ne supprime pas le besoin d'un moyen de transport ou de bootstrap.

## Ce qui est deja en place dans Nexus/SBFB

### Stack Iroh presente

Le workspace utilise Iroh comme couche P2P principale:

- `Cargo.toml:36-39`: `iroh = "0.98"`, `iroh-docs = "0.98"`, `iroh-gossip = "0.98"`, `iroh-blobs = "0.100"`.
- `crates/nexus-core-rs/src/node.rs:243`: endpoint construit avec `Endpoint::builder(presets::N0)`.
- `crates/nexus-core-rs/src/node.rs:311-313`: un meme endpoint route `iroh-blobs`, `iroh-gossip` et `iroh-docs` via ALPN.

Impact: la base protocolaire est bien P2P/Iroh, pas seulement une API web classique.

### Relais custom et fallback n0

Le projet a deja une surface de configuration pour eviter de dependre uniquement des relais publics n0:

- `crates/nexus-core-rs/src/relay_config.rs:11-17`: ordre de priorite `SBFB_CUSTOM_RELAYS`, `~/.sbfb/relays.json`, puis fallback vers `iroh::defaults::prod::default_relay_map()`.
- `crates/nexus-core-rs/src/relay_config.rs:181-187`: les relais doivent etre en HTTPS, et les loopbacks sont refuses hors mode dev.
- `crates/nexus-core-rs/src/node.rs:273`: injection de `RelayMode::Custom(map)` si la config custom existe.

Impact: pour l'anti-censure, SBFB peut viser des relais autogeres ou communautaires. Mais tant que la config n'est pas fournie, le chemin par defaut reste les relais/discovery Iroh standards.

### Tickets et MemoryLookup

Le repo a deja une logique utile pour bootstrap hors-bande:

- `crates/nexus-core-rs/src/node.rs:237-243`: chaque node attache un `MemoryLookup`.
- `crates/nexus-core-rs/src/blobs.rs:112-120`: `fetch_ticket` parse un `BlobTicket` et seed le `MemoryLookup` avec l'`EndpointAddr` contenu dans le ticket.
- `crates/nexus-core-rs/src/blobs.rs:221-255`: test local prouvant qu'un node B recupere un blob via ticket.
- `crates/nexus-core-rs/src/discovery.rs:296-321`: test prouvant qu'un peer local seed dans `MemoryLookup` est joignable sans pkarr.

Impact: c'est le socle le plus important pour dire que le web public n'est pas obligatoire. Un ticket transporte l'adresse et peut circuler via un autre canal.

### Iroh-docs et replication

Le projet a deja des tickets de documents et de la replication:

- `crates/nexus-core-rs/src/docs.rs:385-398`: les tickets docs incluent `RelayAndAddresses`.
- `crates/nexus-core-rs/src/docs.rs:524-535`: test de sync entre deux nodes via import de ticket.
- `crates/nexus-shell-daemon/src/storage_api.rs:29`: replication hardcodee pour `sbfb-ideas`.
- `crates/nexus-shell-daemon/src/storage_api.rs:478-505`: routes `storage_ticket` et `storage_join`.
- `crates/nexus-shell-daemon/src/storage_api.rs:597-611`: subscription live avec increment de version sur insertion distante.

Impact: le modele est coherent pour Babel: corpus, metadonnees, validation humaine et registres de provenance peuvent vivre dans des namespaces iroh-docs.

### Feed sync et catch-up

La couche feed recente est plus solide:

- `crates/nexus-shell-daemon/src/feed_sync.rs:513-535`: `feed_join` utilise `import_and_subscribe` sans fenetre entre import et subscribe.
- `crates/nexus-shell-daemon/src/feed_sync.rs:581-597`: backfill des entrees deja presentes avant le live stream.
- `crates/nexus-shell-daemon/src/feed_sync.rs:173-207`: dedup avant rate-limit, puis controle de debit.
- `crates/nexus-test-harness/tests/multi_daemon.rs:338-480`: tests E2E prevus pour feed sync et catch-up offline, mais marques integration.

Impact: bon alignement avec un usage Babel: un node peut rattraper l'historique quand il revient en ligne.

### Gossip non bloquant au boot

Le daemon a ete durci contre le deadlock du premier noeud:

- `crates/nexus-core-rs/src/gossip.rs:390-405`: `join_topic` reste bloquant car il attend `NeighborUp`.
- `crates/nexus-core-rs/src/gossip.rs:412-424`: `subscribe_topic` est la variante non bloquante.
- `crates/nexus-shell-daemon/src/runtime.rs:1053-1110`: outbox + subscribe non bloquant.
- `crates/nexus-shell-daemon/src/runtime.rs:1083-1197`: replay de l'outbox sur `NeighborUp`.

Impact: c'est essentiel pour un reseau anti-censure. Le premier node d'un groupe local ne doit pas rester bloque parce qu'aucun voisin n'est encore visible.

## Ce qui n'est pas encore vrai

### "Sans Internet" local automatique n'est pas encore cable au runtime

Iroh supporte une decouverte locale type mDNS, mais le repo ne montre pas de wiring runtime actuel pour `MdnsAddressLookup` ou `address-lookup-mdns` dans les crates actives. La recherche locale a trouve des mentions docs, mais pas de configuration Cargo/runtime active.

Impact: sur un LAN ou hotspot local, le chemin le plus credible aujourd'hui passe par tickets, adresses seedees, relai local configure ou canal manuel. La decouverte locale automatique est un bon prochain chantier.

### Les blobs sont encore en memoire

`crates/nexus-core-rs/src/node.rs:72` et `node.rs:294` indiquent que les blobs utilisent encore `MemStore`.

Impact: pour Babel, ce n'est pas suffisant. Les livres, traductions, OCR, artefacts et manifests doivent survivre au redemarrage. Il faut un store blob persistant avant de promettre une bibliotheque durable.

### DNS fallback existe, mais pas comme chemin boot principal

Le code DNS fallback existe:

- `crates/nexus-core-rs/src/dns_fallback.rs:67-70`: variables `SBFB_DNS_FALLBACK_ENABLED` et `SBFB_DNS_FALLBACK_DOMAIN`.
- `crates/nexus-core-rs/src/dns_fallback.rs:319-322`: loader env.
- `crates/nexus-shell-daemon-core/src/browse.rs:347-349`: `with_dns_fallback`.

Mais la recherche runtime n'a pas trouve de wiring direct de `load_dns_fallback_from_env()` dans `crates/nexus-shell-daemon/src/runtime.rs`.

Impact: ce n'est pas encore un chemin operationnel global pour le daemon.

### Tor n'est pas un transport Iroh actif ici

`crates/nexus-core-rs/src/tor_transport.rs:4-6` et `tor_transport.rs:124-125` disent explicitement que la Phase 1 charge la config et bootstrap Tor, mais que la Phase 2 devra garder le handle pour les connexions reelles.

Impact: ne pas promettre aujourd'hui que SBFB/Iroh passe par Tor dans ce repo. C'est une piste, pas un acquis.

### Tests multi-daemon non lances ici

Les tests E2E reseau existent, mais exigent `SBFB_INTEGRATION=1`:

- `crates/nexus-test-harness/tests/multi_daemon.rs:8-14`
- `crates/nexus-test-harness/tests/multi_daemon.rs:130-198`
- `crates/nexus-test-harness/tests/multi_daemon.rs:338-596`

Impact: pour fermer une promesse publique, il faut une matrice runtime: LAN sans Internet, hotspot local, relai custom, ticket USB, redemarrage, catch-up.

## Babel: coherence avec le projet

Les docs Babel existantes cadrent deja le bon positionnement:

- `docs/affine-sbfb/04_BABEL_SUR_SBFB.md:5-15`: Babel comme app vitrine SBFB avec corpus, traduction, provenance, validation humaine, lecture offline, Babel Shelf.
- `docs/affine-sbfb/04_BABEL_SUR_SBFB.md:47-67`: chaque texte doit garder un graphe de provenance complet; Gutenberg est un corpus de demarrage, pas une exception au droit.
- `docs/affine-sbfb/02_PROTOCOLE_DATA_FLOW.md:47-61`: Babel peut devenir une app SBFB et recevoir ses contenus via USB, Wi-Fi local, serveur local, noeud SBFB leger ou firmware Babel.
- `.planning/research/rrv_scoped_search_compute_groups.md:41-69`: si Babel est la premiere app publique, RRV devient un moteur de preuve et d'exploration de Babel.

Conclusion produit:

Babel est un bon cas d'usage anti-censure parce qu'il combine:

- corpus libre ou legalement redistribuable;
- traduction IA locale ou distribuee;
- validation humaine par traducteurs;
- provenance publique des sources, traductions et validations;
- lecture offline;
- diffusion P2P ou hors-bande.

La traductrice professionnelle peut donc contribuer utilement a deux endroits:

- validation linguistique: qualite, registre, fidelite, erreurs d'IA, style cible;
- UX produit: comprehension du workflow de validation, lisibilite des preuves, ergonomie pour un vrai traducteur.

## Formulation publique recommandee

Formulation forte mais exacte:

> Babel est pense comme une bibliotheque et un atelier de traduction distribue. L'objectif est de rendre des textes libres ou legalement redistribuables accessibles dans le plus de langues possible, avec traduction IA, validation humaine et provenance verifiable. Le projet s'appuie sur SBFB, le protocole P2P que je construis: Internet peut servir de couche de diffusion, mais ce n'est pas la seule. Les contenus peuvent aussi circuler via reseaux locaux, relais communautaires, tickets, caches offline ou supports physiques selon le contexte.

Formulation a eviter:

> Ca marche sans Internet partout.

Correction:

> Ca peut fonctionner sans web public et sans serveur central. Sans Internet global, il faut quand meme un autre canal: reseau local, hotspot, relai local, Bluetooth, USB ou autre moyen de bootstrap.

## Priorites techniques avant promesse publique forte

1. Activer et tester une decouverte locale explicite pour LAN/hotspot.
2. Ajouter un store blob persistant pour livres et artefacts Babel.
3. Documenter un bootstrap Babel par ticket: QR, fichier, USB, message, hotspot.
4. Tester relai custom SBFB sans relais publics n0.
5. Tester une matrice "pays censure": DNS bloque, GitHub bloque, web bloque, relai custom OK, LAN OK, USB OK.
6. Ajouter un runbook utilisateur non technique: recevoir Babel, verifier provenance, lire offline, partager a quelqu'un d'autre.
7. Separarer clairement "Internet", "web public", "relai", "LAN", "support physique" dans toute communication.

## Conclusion

Le projet est techniquement coherent avec une ambition anti-censure P2P mondiale, mais la promesse doit rester precise.

Nexus/SBFB a deja des briques solides: Iroh, docs, gossip, blobs, tickets, relays custom, feed sync, replication, catch-up, provenance et orientation Babel. Ce qui manque pour dire "ca marche sans Internet" de facon robuste, c'est surtout la partie operationnelle: decouverte locale active, blob persistence, runbook de bootstrap hors-bande, tests LAN/hotspot/USB, et relays communautaires testes hors n0.

La meilleure promesse aujourd'hui:

> Babel ne depend pas d'une plateforme centrale unique. Il peut utiliser Internet quand c'est disponible, mais il est concu pour pouvoir aussi circuler par des chemins locaux, P2P et hors-bande, avec validation humaine et provenance verifiable.
