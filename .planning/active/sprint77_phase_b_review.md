# Sprint 77 Phase B Review — Shard data plane ALPN + private ComputeGroup

## Verdict: PASS

0 P0/P1. Couverture des 5 dimensions (correctness, sécurité, scope/research, tests,
patterns/API) + auto-vérification adversariale. 4 P3 de couverture de branches documentés
(non bloquants). **Codex GPT 5.5 (gate d'indépendance externe) : CONFIRMED 8/8 livrables, 0
GAP** → review promue PASS (voir §Codex reconciliation).

> **Note process honnête (529 outage)** : le review Workflow ultracode (fan-out 5
> dimensions + adversarial + synth) a été lancé **deux fois** et un agent unique
> `nexus-phase-review-deep` une fois — **les trois ont échoué intégralement sur `API Error:
> 529 Overloaded`** (outage serveur Anthropic, ~15 min, persistant). Per la chaîne de
> fallback du bootstrap (dernier recours = review main-thread), cette review est conduite
> **inline** (l'exécuteur, avec vérification de l'API iroh contre la source réelle installée).
> **L'indépendance est assurée par Codex GPT 5.5** (infra OpenAI, non affectée par le 529),
> qui est de toute façon le gate de vérification croisée bloquant. Les scripts Workflow sont
> sauvegardés (`sprint77-phaseb-review-wf_c7eb437d-330.js`) et re-jouables quand l'API
> Anthropic récupère, si une trace multi-agents est souhaitée pour les archives.

## Scope & staging

Diff de phase atomique, 5 fichiers cohérents (tous `crates/nexus-core-rs/src/`) :
- NEW `compute_group.rs` (ComputeGroup Ed25519+JCS allowlist) + NEW `shard.rs` (ALPN
  `sbfb/shard/1` : framing + ShardProtocol + conn_rtt)
- MOD `canonical.rs` (+`DOMAIN_COMPUTE_GROUP_V1` additif), `node.rs` (+`SHARD_ALPN`),
  `lib.rs` (`pub mod` + re-exports)

Artefacts planning du commit : `sprint77_phase_b_preflight.md`, ce `review.md`,
`sprint77_phase_b_codex_review.md` (à venir). **Exclus** : `.planning/research/
factory_embedded_ide_study.md` (hors-scope, untracked) + `.target-docker/` (cache build
Docker, untracked — supprimé avant commit). T1 = `N-A-no-frontend-change` respecté (0 `web/`).

## Three-block verification (fail-fast)

- **Windows natif** : `cargo fmt --all --check` 0 diff · `cargo clippy --workspace
  --all-targets --locked -D warnings` 0 · **`cargo nextest run --workspace --locked`
  1828/1828 0-skip** (baseline 1811 **+17 exact**) · `cargo test --workspace --locked --doc`
  ok · `cargo build -p nexus-shell-daemon --release` ok.
- **Docker canonique `sbfb-ci` (rust:1.94 Linux)** : total 1832 (+4 `#[cfg(unix)]`). Un seul
  test a flaké en run concurrent — `nexus-shell-daemon::e2e
  start_writes_running_json_and_responds_to_health` — **qui PASSE en ré-exécution isolée dans
  le MÊME conteneur** (1 passed, 0.37s) ET en Windows natif full (1828/1828). C'est le flake
  env connu Docker-sur-Windows bind-mount (le test joint des relays réseau réels
  `relay.n0.iroh-canary` + bind port + FS sous charge conteneur ; cf. mémoire S74 « Docker
  canonique env-bloqués »). Phase B est additif dans `nexus-core-rs`, ne touche pas le chemin
  startup/health du daemon → pas une régression Phase B.
- Code de prod : **0 `unwrap()`/`panic!`/`unsafe`/`todo!`** hors `#[cfg(test)]` (toutes les
  erreurs via `?`/`map_err`). `#![deny(unsafe_code)]` crate-wide respecté. `#![warn(missing_docs)]`
  satisfait (clippy `-D warnings` vert). Frontend N/A.

## Delta tests

+17 Rust (10 `compute_group` + 7 `shard`). 0 Vitest (cohérent, core-interne). Plan §15
estimait +6 (6 tests nommés §5.3) ; les 11 supplémentaires sont des tests de rigueur crypto
(surface R5 Ed25519+JCS) + codec framing pur — annoncés honnêtement (annoncé==réel, invariant
audit). Tous verts Windows natif. Les 6 nommés présents et sémantiquement corrects (voir
dimension Tests).

## Dimension 1 — Correctness + branches

- `compute_group.rs` : `sign()` valide initiator==keypair → caps → canonical_bytes ;
  `verify_signature()` ordre version → caps → attribution → crypto verify (mirror exact
  `node_directory.rs:272-294`). `is_member` n'inclut PAS l'initiateur (testé). `with_member`/
  `new` corrects.
- `shard.rs` : `frame_len_to_header`/`header_to_frame_len` cappent **avant** alloc (pas de
  `vec![0; len]` avant le check — vérifié `shard.rs` read_frame ordonne `header_to_frame_len`
  AVANT `vec![0u8; len]`). `read_frame` clean-EOF = `ReadExactError::FinishedEarly(0)` → `Ok(None)`,
  et `FinishedEarly(n>0)` (header tronqué mid-lecture) tombe dans le bras `Err(e)` → `Err`
  (une troncature n'est PAS avalée comme EOF). `accept` : `remote_id()` lu, membership-check,
  reject AVANT `accept_bi` ; boucle echo ; `finish`. `conn_rtt` = `conn.rtt(PathId::ZERO)`.
- **Branches non couvertes par un test réseau (P3)** : (a) `header_to_frame_len` over-cap sur
  un frame entrant forgé (la logique de cap EST couverte par le test pur `frame_header_rejects_oversize`) ;
  (b) `payload read_exact` erreur mid-frame ; (c) `write_frame` early-return over-cap (logique
  couverte par le test pur) ; (d) `ShardProtocol::new` direct avec une allowlist invalide (le
  chemin factory `shard_protocol_factory` qui verify EST couvert). Toutes défensives, logique
  cœur couverte par les tests purs — P3.

## Dimension 2 — Sécurité + protocole (2 surfaces rouge-ligne)

- **Admission AVANT frame CONFIRMÉE** : `ShardProtocol::accept` lit `*conn.remote_id().as_bytes()`
  puis `if !self.admission.is_member(&peer) { conn.close(SHARD_REJECT_NOT_MEMBER.into(), ...);
  return Ok(()) }` — **avant** tout `accept_bi`. Miroir exact `seed_protocol.rs:264`. Un
  non-membre ne fait calculer/echo aucun octet (testé `shard_handshake_rejects_non_member`).
  `remote_id()` = identité QUIC Ed25519 non-spoofable.
- **Crypto Ed25519+JCS** : `canonical_bytes(group, DOMAIN_COMPUTE_GROUP_V1)` ; `signature` +
  `initiator` redondant **hors** canonical (sur l'envelope) ; attribution split-brain
  (`payload.initiator == envelope.initiator`, testé) ; domain separation testée vs
  `DOMAIN_NODE_DIRECTORY_V1`. `shard_protocol_factory` + `ShardProtocol::new` **verifient
  l'allowlist une fois au wiring** (fail-fast, refuse une allowlist forgée).
- **0-bump wire** : `DOMAIN_COMPUTE_GROUP_V1` additif (canonical.rs après `DOMAIN_NODE_DIRECTORY_V1`) ;
  aucun `*_FORMAT_VERSION` touché ; `COMPUTE_GROUP_FORMAT_VERSION = 1` net-new ; `SHARD_ALPN`
  = string ALPN. Confirmé par grep (les 8 `*_FORMAT_VERSION` intacts, fail-fast vert).
- **DoS** : `MAX_SHARD_FRAME_BYTES` (64 MiB) appliqué avant alloc ; `COMPUTE_GROUP_MAX_MEMBERS`/
  `COMPUTE_GROUP_ID_MAX` aux deux bouts (`check_group_caps` dans sign ET verify).
- **Confidentialité** : la doc module de `compute_group.rs` et `shard.rs` énonce explicitement
  que l'allowlist est un **contrôle d'admission, PAS un chiffrement** des activations (SI-4
  collusion résiduel, scope cut #4). Aucun claim de confidentialité faux. ✓

## Dimension 3 — Scope cuts + research grounding

- **0 débordement** : aucune primitive C (`ShardPlan`/`RunProof`/`ShardedSessionManifest`)
  ni D (scheduler water-filling/k-medoids/perf-map qui CONSOMME le RTT). Phase B **expose**
  `conn_rtt` (que D consommera), ne l'utilise pas.
- **PLAN-ADAPT suivi** : A1 `conn.rtt(PathId::ZERO)` (PAS `conn.stats().path.rtt` —
  l'API quinn-proto erronée ; vérifié que le code utilise la bonne API noq) ; A2 framing neuf
  len-prefix (pas `read_to_end` one-shot) ; A3 rejet pré-frame.
- **Day-0** : iroh 0.98 pinné, **0 dép nouvelle** (`Cargo.toml` non modifié — vérifié git diff
  ne touche aucun `Cargo.toml`) ; groupe privé jamais public (aucune découverte ouverte) ;
  kudos non-monétaire (aucun stake/cost/burn dans ComputeGroup) ; llama.cpp RPC **pas
  réintroduit**.
- **P3-D-3** : Phase B n'ajoute **aucun** chemin result-sync `seen.remove` → doc-note correct
  (non déclenché).

## Dimension 4 — Tests sémantiques

Les 6 tests nommés (§5.3) présents et sémantiquement corrects :
- `shard_alpn_registered_in_router` : nœud avec factory accepte SHARD_ALPN ; nœud vanilla le
  **refuse** (discriminateur prouvant que l'enregistrement EST ce qui câble l'ALPN). ✓
- `shard_frame_roundtrip_two_nodes` : **3 frames** sur le **MÊME** `open_bi` (contrat de réuse
  D2), chacune echoée identique. ✓
- `compute_group_signature_roundtrip` : sign→verify, pubkey, version. ✓
- `shard_handshake_rejects_non_member` : prouve qu'**aucun frame n'est echoé** (dial réussit,
  mais open_bi/read échoue car conn fermée) — pas juste un connect raté. ✓
- `shard_handshake_admits_member` : membre échange un frame avec succès. ✓
- `shard_conn_stats_exposes_rtt` : `conn_rtt(&conn).is_some()` + borne saine `< 60s`. ✓

11 tests de rigueur supplémentaires (tamper payload/sig, attribution mismatch, wrong-signer,
oversized membership/id, domain-sep, json roundtrip, is_member, frame codec roundtrip,
frame over-cap rejection) — chacun cible une branche distincte, aucun faux-vert (pas
d'`assert!(true)`/test vide). Tous hermétiques in-process (pas de WAN/GPU). Risque flakiness :
les 4 tests réseau utilisent `MemoryLookup` in-process (pas de relay réel), déterministes.

## Dimension 5 — Patterns + API réelle + qualité prod

- **Mirror fidèle** : `compute_group.rs` ↔ `node_directory.rs` (envelope attribution + caps
  sign-AND-verify + version check) ; `shard.rs` câblage ↔ pattern SEED_ALPN/`ExtraProtocolFactory`
  (`node.rs:294-400`). `accept` utilise `std::result::Result<(), AcceptError>` (l'alias crate
  `Result = Result<T, NexusError>` ne peut pas porter `AcceptError` — corrigé, fail-fast vert).
- **API iroh 0.98 RÉELLE** (vérifiée contre `~/.cargo/registry/.../iroh-0.98.2` + `noq-0.18.0`
  + `noq-proto-0.17.0`) : `Connection::rtt(PathId::ZERO) -> Option<Duration>` (connection.rs:970) ;
  `ConnectionStats` n'a **PAS** de champ `path`/`rtt` (donc `conn.stats().path.rtt` serait FAUX —
  c'était la proposition initiale du preflight basée sur quinn-proto, corrigée A1) ;
  `RecvStream::read_exact` + `ReadExactError::FinishedEarly(usize)` ; `PathId::ZERO` const.
  Le code utilise la BONNE API.
- **lib.rs re-exports** : `compute_group` + `shard` + `SHARD_ALPN` + `DOMAIN_COMPUTE_GROUP_V1`
  exportés, ordre cohérent, aucune collision (fail-fast vert).
- **Qualité prod** : 0 unwrap/panic/unsafe/todo hors test ; constantes nommées (pas de magic
  number : MAX_SHARD_FRAME_BYTES, SHARD_REJECT_NOT_MEMBER, COMPUTE_GROUP_*). Doc `conn_rtt`
  explique pourquoi `PathId::ZERO` (chemin primaire, pas multipath) ; MAX_SHARD_FRAME_BYTES
  justifié (activations 70B).

## Findings (post auto-vérification adversariale)

- **P3-1** Couverture de branche : 4 branches défensives non couvertes par un test réseau
  (cap entrant forgé, payload truncation mid-frame, write over-cap, `new()` allowlist invalide).
  La logique de cap est couverte par les tests purs `frame_header_*`. → Documenté, acceptable.
- **P3-2** L'echo handler de Phase B est un placeholder du forward réel (Phase F) — documenté
  dans la doc module `ShardProtocol`. → Intentionnel, acceptable.
- **P3-3** Le `revision` de ComputeGroup n'a pas de protection rollback dans ce module (c'est
  un concern ingest-layer, mirror `node_directory`) — documenté. → Intentionnel.
- **P3-4** `set_max_concurrent_bi_streams` (tuning iroh 0.98) existe mais n'est pas utilisé ;
  Phase B ne promet aucun tuning de streams (cohérent R6). → Pas de claim, acceptable.

Aucun P0/P1/P2 confirmé. Rigor signal G4 satisfait (findings P3 explorés, pas de sur-confiance).

## Residual risk

Faible. Admission prouvée pré-frame, crypto mirror d'un pattern S75 audité, 0-bump confirmé,
API iroh vérifiée contre la source réelle (correction A1 du preflight appliquée). Le risque
non réductible en hermétique — la convergence WAN du data plane sous churn — est une preuve
Phase K (`b3`), reconnu et routé, jamais revendiqué vert ici. L'echo (Phase B) sera remplacé
par le forward réel (Phase F).

## Codex reconciliation

Codex GPT 5.5 (`codex exec --dangerously-bypass-approvals-and-sandbox`, output brut
`sprint77_phase_b_codex_review.md`) : **verdict global CONFIRMED, 8/8 livrables CONFIRMED, 0
P0/P1/P2 GAP.** Codex a relu la source/diff/registry (sans cargo, comme demandé) et confirmé
indépendamment chaque livrable avec file:line :
1. Crypto ComputeGroup (sign/verify, attribution, is_member exclut l'initiateur) —
   `compute_group.rs:189/195/196/220/227/228/233/149/160-174`.
2. Admission AVANT frame (`remote_id()` :226 → check :227 → close :229-230, `accept_bi` seulement
   :235) — un non-membre ne peut atteindre aucun traitement de frame.
3. Framing (cap avant write :88-94/115-122 ; `FinishedEarly(0)`=EOF :134-137 ; `FinishedEarly(n>0)`
   + autres = Err :138-142 ; cap avant alloc :144-145).
4. **RTT API** : Codex a indépendamment relu `noq-proto-0.17.0/stats.rs:254-289` et confirmé que
   `ConnectionStats` n'a PAS de champ `rtt` (stats ignore rtt/cwnd/mtu) → `conn.rtt(PathId::ZERO)`
   (`shard.rs:160-161`, `iroh-0.98.2 connection.rs:968-971`) est la BONNE API. **Valide la
   correction A1 du preflight** (le preflight avait initialement scanné quinn-proto).
5. 0-bump wire additif (DOMAIN :255, SHARD_ALPN :80, 0 Cargo.toml, aucun `*_FORMAT_VERSION` muté).
6. Tests : **17** (10 compute_group + 7 shard), 6 nommés assertent leur nom, 0 faux-vert.
7. Qualité prod : unwrap/panic/unsafe/todo uniquement sous `#[cfg(test)]` ; pas de stake/cost/burn ;
   caveat confidentialité + SI-4 documentés.
8. Scope : 0 symbole ShardPlan/RunProof/ShardedSessionManifest ; seul `conn_rtt` exposé (pas de
   consommateur D).

**Aucun GAP → aucune correction requise.** Review promue PASS. Le fichier Codex brut n'est pas
réécrit (lightcheck Check 7). L'indépendance externe (GPT 5.5, infra OpenAI non affectée par
l'outage 529 Anthropic) compense le fallback review inline.
