# Sprint 77 Phase B Preflight — Shard data plane ALPN + private ComputeGroup

> Preflight G8 orchestré en **Workflow ultracode** (fan-out 5 scans factuels S1a/S1b/S2/S3/S4
> + synthèse adversariale, run `wf_4a584340-e54`, 6 agents, 503k tokens). Transcrit par
> l'exécuteur. Verdict porté par l'orchestration Workflow elle-même (pas de superviseur).

## Verdict: PLAN-ADAPT

0 DESIGN-CONFLICT crédible. Les 5 scans convergent EXECUTE sur le fond ; le scope Phase B
(plan §5) est confirmé **intact et borné** contre le code réel et les décisions gelées. Une
seule famille de CONCERN réelle (API RTT iroh 0.98) impose une **approche corrigée** → le code
suit l'adaptation, pas le libellé original du plan. PLAN-ADAPT avec évidence OSS concrète, ne
touche **aucune** décision Day-0.

## Adaptations (le code suit CECI, pas le plan original)

### A1 — API RTT : `conn.rtt(PathId::ZERO)`, PAS `conn.stats().path.rtt`

Le plan (§5.2/§5.3 test #6, kickoff D2) parle de « `conn.stats()` expose RTT ». Le Workflow a
d'abord proposé `conn.stats().path.rtt` **en scannant `quinn-proto-0.11.14`** (un dup transitif).
**Correction exécuteur (vérif source réelle installée)** : la dép réelle est **`noq-proto 0.17.0`**
(fork quinn d'iroh, ré-exporté `noq`). Dans noq-proto 0.17.0, `ConnectionStats` n'a **AUCUN champ
`path`** (`udp_tx/udp_rx/frame_tx/frame_rx/lost_packets/lost_bytes` seulement) et son
`impl Add<PathStats>` documente explicitement que `Connection::stats()` **ignore les champs
`rtt`/`cwnd`/`current_mtu`**. Donc `conn.stats().path.rtt` **ne compile pas**.

L'accès RTT réel et correct :
`iroh::endpoint::Connection::rtt(&self, path_id: PathId) -> Option<Duration>`
(« Current best estimate of this connection's latency (round-trip-time) »,
`iroh-0.98.2/src/endpoint/connection.rs:970`). `PathId::ZERO` (`noq-proto-0.17.0/src/connection/
paths.rs:56 pub const ZERO: Self = Self(0)`, ré-exporté `iroh::endpoint::PathId`) est le **chemin
primaire par défaut** (noq dial via `network_path(PathId::ZERO)`) — c'est une CONSTANTE, pas une
résolution multipath. La crainte du Workflow (« ne pas utiliser conn.rtt(path_id), complexité
multipath ») est donc infondée pour `PathId::ZERO`. Phase B expose un helper
`conn_rtt(&Connection) -> Option<Duration>` = `conn.rtt(PathId::ZERO)`.

### A2 — Framing = code NEUF (pas un copier-coller du seed one-shot)

`seed_protocol.rs:265-269` fait un `read_to_end(MAX)` one-shot (req/resp unique). Le plan §5.2
promet un `open_bi` **long-vécu multi-frame**. Le framing longueur-préfixe est un **AJOUT** :
boucle `read_exact(4-byte BE len)` + `read_exact(payload)`, clean-EOF entre frames détecté via
`ReadExactError::FinishedEarly` → `None`. Cap anti-DoS par frame `MAX_SHARD_FRAME_BYTES`
(miroir `MAX_SEED_MSG_BYTES seed_protocol.rs:61`, dimensionné pour les activations). API confirmée
réelle : `iroh::endpoint::RecvStream::read_exact` (noq-0.18.0/src/recv_stream.rs:89),
`SendStream::write_all`/`finish` (send_stream.rs:74/189).

### A3 — Rejet handshake AVANT tout frame

Point d'insertion exact `seed_protocol.rs:264` : `let dialer = *conn.remote_id().as_bytes()`
extrait AVANT `accept_bi()` (l.265). `ShardProtocol::accept` compare `conn.remote_id()`
(identité QUIC Ed25519, non-spoofable, THREAT_MODEL:127) à l'allowlist `ComputeGroup` ; non-membre
→ `conn.close(code, reason)` SANS jamais lire un frame.

## Scope confirmation (borné, 0 débordement C/D)

Livrables Phase B = (1) `SHARD_ALPN` via `extra_protocols` miroir SEED_ALPN, (2) `ShardProtocol`
handler open_bi long-vécu + framing, (3) module `shard.rs` primitive connexion + `conn_rtt`,
(4) `compute_group.rs` `ComputeGroup` Ed25519+JCS `DOMAIN_COMPUTE_GROUP_V1` additif, (5) rejet
handshake non-membre AVANT frame, (6) exposition RTT pour la perf-map (Phase D). **AUCUN
débordement** vers C (primitives `ShardPlan`/`RunProof` absentes des livrables §5.2) ni D
(scheduler/perf-map : Phase B EXPOSE le RTT, ne le CONSOMME pas). ComputeGroup = **contrôle
d'admission** (qui participe), **PAS** chiffrement des activations (scope cut #4 — activations en
clair assumées, SI-1/SI-4) ni découverte ouverte (scope cut #8, R-iroh-audit P0).

## Vigilance sécurité (S3 CONCERN, non-bloquante)

SI-4 collusion inter-workers : l'allowlist borne le pool aux membres invités mais ne garantit pas
≥1 membre honnête. **La doc du code ne doit PAS sur-promettre** que l'allowlist apporte la
confidentialité — elle apporte le contrôle d'admission. (§16 THREAT_MODEL = Phase K wrap-up.)

## Invariants confirmés (0-bump wire)

8 `*_FORMAT_VERSION` tous à 1, aucun touché (S4). `DOMAIN_COMPUTE_GROUP_V1` purement additif
(canonical.rs après :239, rationale 0-bump :236-238). `SHARD_ALPN = b"sbfb/shard/1"` = string ALPN
transport, ne touche aucun canonical. `schema_version:1` net-new pré-launch. `canonical_bytes<T>`
(canonical.rs:260) générique réutilisable verbatim. **0 dép nouvelle** : iroh 0.98.2, ed25519-dalek
2.2.0, serde_jcs 0.2.0, blake3 1.8.5 (existants). Day-0 honorées : iroh 0.98 pinné, groupe privé
jamais public, kudos non-monétaire (pas de stake dans ComputeGroup), llama.cpp RPC reste REJETÉ.

## Code anchors confirmés

- `SHARD_ALPN` : `node.rs:68` (à côté de `SEED_ALPN`)
- Enregistrement : `node.rs:294-297` + `:395-398` (boucle `extra_protocols` AVANT `spawn()` :400)
- `ExtraProtocolFactory` : `node.rs:82-83`
- Handler mirror : `seed_protocol.rs:262-283` (`let dialer` :264 AVANT `accept_bi` :265)
- `ComputeGroup` mirror : `node_directory.rs:201-294` (NodeDirectoryEntry sign/verify + caps
  sign-AND-verify + attribution split-brain) ; `seed.rs:275-282` (roundtrip test template)
- `DOMAIN_COMPUTE_GROUP_V1` : `canonical.rs:239` (après DOMAIN_NODE_DIRECTORY_V1)
- RTT : `conn.rtt(PathId::ZERO)` (`connection.rs:970` + `noq-proto paths.rs:56`)
- Framing : `RecvStream::read_exact` (noq recv_stream.rs:89), `ReadExactError::FinishedEarly`

## Test plan (§5.3) + rigueur crypto R5

Les 6 tests nommés sont hermétiques-testables in-process (pas de WAN, pas de GPU). Surface crypto
Ed25519+JCS (R5) → tests crypto approfondis ajoutés (tamper, attribution, cap, domain-sep,
membership) au-delà des 6 nommés ; **delta réel annoncé au commit** (l'estimation plan §15 = +6
est indicative ; honnêteté annoncé==réel = invariant audit). P3-D-3 (test #7) reste conditionnel :
Phase B n'ajoute AUCUN chemin result-sync `seen.remove` → **doc-note** (non déclenché).

## Scans (signaux bruts)

| Scan | Signal | CONCERN |
|---|---|---|
| S1a OSS prior-art | EXECUTE | RTT API PathId (corrigée A1) |
| S1b deps/CVE | EXECUTE | RTT path-based (corrigée A1) ; R6 tuning `set_max_concurrent_bi_streams` existe mais 0 usage SBFB — ne pas promettre |
| S2 décisions historiques | EXECUTE | — (0 DESIGN-CONFLICT) |
| S3 threat model | EXECUTE | SI-4 collusion : ne pas sur-promettre confidentialité (vigilance doc) |
| S4 wire invariants | EXECUTE | `#[serde(default)]` runtime-tolerance à documenter |
