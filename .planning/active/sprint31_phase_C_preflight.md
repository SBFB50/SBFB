# Sprint 31 Phase C — preflight G8

Date : 2026-04-26 | HEAD : `0771dc8` | Verdict : **EXECUTE plan-as-is**

## Memory consultation (Step 1.5)

- feedback_approach.md : pick deepest technical option, context7
  obligatoire avant code touchant lib externe, OSS prior art
  obligatoire (G10). Phase C ajoute arti-client 2.0 = nouvelle dep
  critique → context7 consulte.
- feedback_context7_systematic.md : context7 obligatoire avant
  toute lib/API touchee. arti-client resolve + query-docs effectue.
- Tensions plan vs memory : aucune.

## Scans (all clean)

- S1a OSS prior art : 4 projets recherches (arti official, torpy,
  tun2tor, TorRequest). Plan propose TorClient::create_bootstrapped
  + connect() → DataStream, config opt-in disabled. context7 confirme
  cette API comme pattern principal documente (code snippets 1597,
  reputation High, benchmark 85). APPROACH-ALIGNED — clean
- S1b deps : arti-client 2.0.x LTS stable (crates.io). tor-rtcompat
  2.0.x aligne. RUSTSEC-2024-0339 (tor-circmgr vanguards lite path
  length) affecte onion services uniquement, hors scope Phase C
  (outbound TCP). 0 CVE arti-client 2.0 trouve (RustSec + NVD +
  WebSearch 2026). 0 delta — clean
- S2 historiques : 6 fichiers cibles scannes (lib.rs, Cargo.toml,
  core-py/lib.rs, tasks.py, HARDENING_ROADMAP.md). git log
  DEVIATION|rejected|scope-cut : 1 commit `9c8ffc9` S30 Phase D
  (HARDENING refresh, Tor prescrit S31 — support, pas conflit).
  Archive scan : mentions Tor = scope-cut S25-S30 (arti pre-1.0
  bloquait, maintenant 2.0 LTS stable). Memory feedback : 0
  conflit — clean
- S3 threat model : fast-path verified. Phase C = transport wrapper,
  pas nouveau wire format ni composant securite. HARDENING_ROADMAP
  §3 S31 prescrit "Tor transport phase 1" — aligned. Pas de
  regression T0-T5 — clean
- S4 wire format : fast-path verified. `_VERSION = 1` dans
  schemas/mod.rs, inchange par Phase C. canonical.rs non touche.
  Day 0 D3 (coordinator outbound) preserved. Pre-launch protocol
  policy preserved — clean

## Telemetrie preflight

- Duree totale : ~3m
- S1a : ~2m / 4 projets OSS consultes + context7 arti (1597
  snippets) / finding : APPROACH-ALIGNED (clean)
- S1b : ~1m / 2 libs scannees (arti-client, tor-rtcompat) + 1
  RustSec check / finding : clean
- S2 : ~30s / 6 fichiers + archive scan / finding : clean
- S3 : fast-path / ~15s
- S4 : fast-path / ~15s

## Notes implementation (context7)

- API : `TorClient::create_bootstrapped(config).await?` puis
  `tor_client.connect(("host", port)).await?` retourne DataStream
  (futures::io AsyncRead + AsyncWrite, PAS tokio::io)
- **Flush obligatoire** : Arti bufferise — appeler
  `stream.flush().await?` apres chaque write
- Lazy init : `TorClient::builder().config(c)
  .bootstrap_behavior(BootstrapBehavior::OnDemand)
  .create_unbootstrapped()?` pour opt-in disabled-by-default
- Runtime : `tor-rtcompat` avec feature `tokio` pour le runtime
  partage iroh + arti

## Scope-cut decouvert implementation

**arti-client dep bloquee par conflit rusqlite** : arti-client 0.41
tire `tor-dirmgr` qui depend de `rusqlite >= 0.36`. Le workspace
pin `rusqlite 0.32` (via `nexus-shell-daemon-core`). Conflit
`libsqlite3-sys` links (une seule version native autorisee).

Resolution Phase C : livrer config infrastructure + feature gate +
transport API + fallback + coordinator wire + tests. Le dep
`arti-client` est commente dans workspace Cargo.toml, la feature
`tor = []` est declaree vide. Le code `#[cfg(feature = "tor")]`
avec `use arti_client::...` reste comme dead code prepare.

Carry S32 : rusqlite 0.32→0.36 workspace upgrade + activation dep
arti-client + full Tor bootstrap E2E.

## Action

Proceder code phase C (scope ajuste).
