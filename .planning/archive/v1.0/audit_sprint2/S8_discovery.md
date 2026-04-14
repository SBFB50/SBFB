# Audit S8 — discovery.rs

**Auditeur**: Agent S8 | **Date**: 2026-04-10 | **Fichier**: `crates/nexus-core-rs/src/discovery.rs`

---

## Conforme

- `DiscoveryClient::my_addr()` utilise `Endpoint::watch_addr()` → `impl Watcher<Value = EndpointAddr>` (confirmé registry iroh-0.97.0 `endpoint.rs:1134`). API correcte.
- `EndpointAddr { id: EndpointId, addrs: BTreeSet<TransportAddr> }` — noms de champs exacts (iroh-base-0.97.0 `endpoint_addr.rs:42-46`). Code accède `.id` et `.addrs` correctement.
- `TransportAddr::Relay(RelayUrl)` et `TransportAddr::Ip(SocketAddr)` — deux seules variantes de l'enum (iroh-base-0.97.0 `endpoint_addr.rs:54-59`). Match exhaustif avec `_ => {}` techniquement redondant mais non-bugué (enum non-exhaustive marquée `#[non_exhaustive]` possible en amont — clause défensive justifiée).
- `Watcher::get(&mut self) -> T` et `Watcher::updated(&mut self) -> NextFut` produisant `Result<T, Disconnected>` — usage correct avec `.map_err(...)` (n0-watcher-0.6.1 `lib.rs:255,292`).
- Polling loop cap à 20 itérations avec erreur explicite si jamais peuplé (`discovery.rs:90-105`) — protège contre boucle infinie.
- `NodeAddrInfo` sérialisable, champs human-readable (hex node_id, String relay_url, Vec<String> addrs).

## Manquant

- Plan S8 (`magical-marinating-phoenix.md:666`) spécifie "publish pkarr record, resolve node_id → endpoints, périodique refresh" — **aucune de ces trois fonctionnalités n'est implémentée**. Le module ne fait que lire l'adresse locale. Le commentaire doc reconnaît ce report au Sprint 4, mais la spec S8 n'est pas satisfaite à ce niveau.

## Déviations

- `updated().await` retourne `Result<EndpointAddr, Disconnected>` (NextFut::Output) mais le code **ignore la valeur retournée** et appelle `watcher.get()` ensuite (`discovery.rs:94-98`). Correct fonctionnellement (double-fetch atomique sous le même lock), mais légèrement redondant — `ep_addr = watcher.updated().await.map_err(...)?;` suffirait.

## Qualité

- Absence de timeout configurable : le test impose 15 s via `tokio::time::timeout` extérieur, mais `my_addr()` lui-même n'a aucun paramètre timeout. Un appelant Python sans timeout risque un hang réseau si le relay n0 est inaccessible. Concern qualité (non-bug).
- `DiscoveryClient` est `Copy` + `Clone` (struct contenant `&'a Endpoint`) — bon pour les appels fréquents.

## Tests

```
running 3 tests
test discovery::tests::node_addr_info_serde_roundtrip ... ok
test discovery::tests::my_node_id_is_stable           ... ok
test discovery::tests::my_addr_returns_address_info_within_reasonable_time ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; finished in 0.19s
```
Tous verts. Couverture correcte pour l'implémentation existante.

## Bugs (DO NOT FIX)

- **BUG P1 — Scope incomplet vs plan** (`discovery.rs:1-134`): `publish pkarr`, `resolve(node_id)`, et `périodique refresh` du plan S8 sont absents. Le module est un sous-ensemble de ce qui était spécifié. Risque: les sprints suivants qui supposent `resolve(node_id)` disponible seront bloqués.
- **Concern qualité — Pas de timeout interne** (`discovery.rs:80-133`): `my_addr()` peut boucler indéfiniment si le relay n0 est inaccessible et le watcher reste connecté. Les appelants PyO3 sans `asyncio.wait_for` seront bloqués sans signal. Ajouter un `tokio::time::timeout` interne avec durée configurable.
- **Deviation mineure — updated() value ignorée** (`discovery.rs:94-98`): valeur `Ok(EndpointAddr)` de `NextFut` silencieusement jetée, suivi d'un second `watcher.get()`. Non-bugué mais sous-optimal.
