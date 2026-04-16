# Day 0 open questions — sprints S20-S25

Aggregat des **paramètres arbitraires** identifiés lors de la
recherche deep des design docs S20-S25 (avril 2026), à valider
explicitement par l'utilisateur au kickoff de chaque sprint
concerné dans le tableau D1..D5/D9 du `sprint{N}_kickoff.md`.

Document écrit pour préserver les Day-0 candidates AVANT
suppression des design docs S20-S25 (qui seront re-générés
naturellement Day 0 du sprint avec contexte à jour, cf.
`docs/claude/README.md` §6.7 horizon long-terme + research-first).

---

## Sprint 20 — Encryption at rest

| # | Paramètre | Default proposé | Trade-off |
|---|---|---|---|
| Q20-1 | Panic wipe confirmation timeout | 3s 2-tap (Ctrl+Shift+Alt+W) | False-positive accidentel vs raid-imminent UX (3s = compromis) |
| Q20-2 | Keystore fallback (si OS keystore HS) | chacha20poly1305 + KDF passphrase | UX impact (passphrase à mémoriser) vs fail-close strict |
| Q20-3 | Duress PIN slot detection mitigation | second slot keystore `<node_id>-duress` | T4 forensic peut détecter via timing/entropy — assumé |
| Q20-4 | LLM grammar enforcement scope | obligatoire par task-type (registry worker-core) | Cas où JSON schema absent = task rejeté |

## Sprint 21 — Rate-limit + redaction

| # | Paramètre | Default proposé | Trade-off |
|---|---|---|---|
| Q21-1 | NER PII redaction default mode | opt-in plugin (regex baseline obligatoire) | Bundle +50MB (spaCy wasm) vs default opt-out safety |
| Q21-2 | Quarantine queue threshold | ≥5 messages/min from publisher dont kudos < threshold AND PoW solved | False-positive = message lost (pattern SMTP greylist) |
| Q21-3 | Rate-limit cleanup window | 5 min `retain_recent` background task | Memory growth vs CPU overhead |
| Q21-4 | Output filter Unicode tag chars policy | block + alert (Default_Ignorable scan) | False-positive sur emoji legitimes |

## Sprint 22 — Sybil + compute detection + voting

| # | Paramètre | Default proposé | Trade-off |
|---|---|---|---|
| Q22-1 | N_threshold kudos full voice | 100 | Bootstrap UX (~2-3 sem usage actif honnête) vs oligarchie |
| Q22-2 | Default `Task.redundancy_factor` | 3 (configurable {1,3,5}) | Cost x3 base vs Gate 3+ exige 5 high-integrity |
| Q22-3 | Spot-check watermark sampling rate | 1/20 consumer-side (default 5%) | Cost overhead 5% vs détection sensitivity |
| Q22-4 | NVML métriques retention | log-only S22 (pas enforcement) | Privacy worker collecte métriques propres |

## Sprint 23 — Ephemeral workers + escalating PoW + honeypot

| # | Paramètre | Default proposé | Trade-off |
|---|---|---|---|
| Q23-1 | Ephemeral worker restart trigger | N=10 tasks OR T=30 min (whichever first) | Restart overhead vs leak window |
| Q23-2 | Escalating PoW geometric ramp cap | 2^25 difficulty max | DoS-able si trop haut, permissif si trop bas |
| Q23-3 | Eclipse detection neighborhood threshold | >70% same `(AS, /24)` pendant 24h | False-positive ISP NAT typology naturelle |
| Q23-4 | Cooldown reset condition | K=5 quiet hours | Adversary wait-then-retry pattern |
| Q23-5 | Cooldown persistence | `~/.sbfb/cooldown.json` file-watcher | Storage growth + restart-tolerant requis |

## Sprint 24 — Sampling + DNS fallback + key rotation

| # | Paramètre | Default proposé | Trade-off |
|---|---|---|---|
| Q24-1 | Re-run sampling rate | 1-5% adaptive per worker reputation | Cost vs detection sensitivity |
| Q24-2 | Statistical method divergence | hash-equality + embedding similarity fallback | Faux positifs sur outputs stochastiques |
| Q24-3 | DoH providers preset | Cloudflare + Quad9 + Mullvad (3, 2/3 majority vote) | Single point vs jurisdiction diversity |
| Q24-4 | Ed25519 rotation trigger | hybride : compromise-driven + annuel soft-warn | UX hostile si forced annuel |
| Q24-5 | Cross-sign overlap window | 30 jours | Window attaque vs propagation gossip P99 |

## Sprint 25 — Tor + per-app quota + RAG sanitization

| # | Paramètre | Default proposé | Trade-off |
|---|---|---|---|
| Q25-1 | Tor mode user setting | opt-in explicit (`config.toml [transport] mode = "tor"`) | Latence +500ms-2s p50 casse UX Gate 1, OK Gate 3+ |
| Q25-2 | Per-app quota unit | compute-seconds + task-count combined | vs tokens-output (opaque pre-launch) |
| Q25-3 | RAG sanitization tier-3 model | Llama Guard 3 1B opt-in inline | License Llama Community vs AGPL-3.0 — review legal requis |
| Q25-4 | Pluggable transports | lyrebird subprocess (Go binary upstream Tor Project) | vs Rust port (aucun mature 2026) |
| Q25-5 | Bridges discovery | manual user-paste BridgeDB S25 + Snowflake broker auto-fetch S26 | Bootstrap chicken-egg |

---

## Décisions structurelles déjà figées (ne PAS rebattre)

Ces points sont issus de la recherche deep et ne sont PAS des
"open questions" — ils ont une réponse univoque. Listés ici pour
référence kickoff :

- **S25 Tor mode = HTTPS relay fallback only** — Tor refuse UDP
  par design ([SOCKS extensions spec](https://spec.torproject.org/socks-extensions.html)),
  QUIC-over-Tor architecturalement impossible. Hole-punching iroh
  désactivé en mode Tor.
- **S25 obfs4 fork-patch iroh REJETÉ** — lyrebird subprocess via
  Arti gère natif les pluggable transports, pas besoin de fork.
  Scope HARDENING_ROADMAP §3 S25 reduced en consequence.
- **S22 stake-based Sybil resistance REJETÉ Day 0** — kudos
  non-monétaire (cf. memory `feedback_kudos_non_monetary`). Pas
  re-débattre.
- **S22 EigenTrust REJETÉ** — power-iteration cycles + complexity
  + Sybil-farming circular. Linear kudos-weight retenu.
- **S20 Signal SVR / Unduress REJETÉS** — incompatibles P2P pur
  (server-side dependency) ou modèle d'usage différent (disposable
  identity).
- **S23 nvidia-smi --gpu-reset REJETÉ** — kill autres processes
  sur GPU, bloqué par nvidia-persistenced. cudaMemset + restart
  retenu (best-effort, pas physical-erasure).
- **S24 X.509 CRL REJETÉ** — parser surdimensionné. JCS canonical
  bytes signed Ed25519 (pattern S10 curator lists) retenu.

## Tech-debts cross-sprint à tracker dans HARDENING_ROADMAP

Carry-overs identifiés et **inscrits dans HARDENING_ROADMAP §3 S26** :

- **Domain fronting implem** : differé S24→S26 (CDN partner +
  Snowflake-WebRTC + ECH fallback)
- **Arti library-embed** : differé S25→S26 (conditionnel
  arti-client API stable >= 1.0)
- **Snowflake broker auto-fetch bridges** : differé S25→S26

Carry-overs identifiés mais **non encore inscrits dans HARDENING_ROADMAP** :

- **Web of Trust PGP-style Sybil-resistance** : differé Gate 4
  LibanLive (S22 §3.2 alternative B). UX prohibitif sans seeds
  IRL — coupler avec real-world ONG verification.
- **Traffic padding implementation** : differé S25+ après iroh
  upstream PR review (S23 livre doc-only PR draft).

---

## Update procedure

Quand un sprint S20-S25 démarre, sa session fraîche :

1. Lis ce fichier + HARDENING_ROADMAP §3 SXX
2. Pour chaque QXX-N applicable au sprint, propose une réponse
   dans `sprint{N}_kickoff.md` §4 D1..D5 avec rationale et
   alternatives rejetées (cf. `docs/claude/README.md` §6.7)
3. Cite ce fichier dans §3 Research consulté du `sprint{N}_plan.md`
4. Re-fait recherche context7 fresh pour libs/CVE (les versions
   d'avril 2026 auront évolué)

## Refs

- `docs/claude/README.md` §6.7 horizon long-terme + doc AVANT code
- `docs/security/HARDENING_ROADMAP.md` §3 S20-S30
- `docs/security/COMPUTE_THREATS.md` §1.5 (GPU wipe best-effort note)
- Memory `feedback_approach.md` Règle critique horizon long-terme
- Memory `feedback_kudos_non_monetary.md` (S22 stake REJETÉ)
