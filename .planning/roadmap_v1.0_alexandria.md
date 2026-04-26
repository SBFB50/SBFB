# Roadmap v1.0 — Alexandria + repo public

**Ecrit** : 2026-04-26 (synthese 4 agents recherche paralleles).
**Tip** : `a63562e` (S30 Phase B livree).
**Objectif** : premier tag v1.0 + repo public + Alexandria fonctionnelle
sur 3 VPS + machine dev.

---

## 1. Constat de depart

### Ce qui est fait (S0-S30, 30 sprints)

- Protocole wire complet (Task, ProjectAnnouncement, CuratorList,
  CanarySigned, DelegationCert) — tous VERSION=1 pre-launch
- P2P discovery + gossip + blobs via iroh 0.97
- Verified deploy from source (Keyoxide + SLSA L1 provenance)
- Process isolation broker/executor JSON-RPC 2.0 IPC
- Bridge postMessage iframe <-> reseau (3 methodes)
- MCP server local-only Streamable HTTP (3 tools, S26)
- Security hardening profond : Sybil 3 couches, watermark SynthID,
  rate-limit GCRA, PoW escalating, encryption at rest, key rotation,
  capabilities gate-off, OS audit events, quarantine queue
- ~1846 tests (856 Rust / 195 SDK / 394+36f coord / 46 gov / 269
  Vitest / 43 Playwright / 4 size-limit)
- 11 workflows CI GitHub Actions
- README, CONTRIBUTING, SECURITY, BUILDING, CODE_OF_CONDUCT
- Radicle flip procedure documentee (MIRROR_FALLBACK.md §3)
- Gates 1-2 unlocked (S18, S22)

### Ce qui manque

| Gap | Impact | Effort estime |
|---|---|---|
| task_runner est un stub | Worker ne fait rien | **6-8h** (copy LlmBackend + wire) |
| Alexandria = 0 LOC | Pas d'app showcase | **~2500 LOC Python** (2 sprints) |
| Zero test multi-noeud | Protocole non valide P2P | 1 phase (S33) |
| Audit externe non lance | Gate 3 bloque | Budget 50-100k$ (hors planning sprint) |
| Issue/PR templates absents | Contribution externe | 1h |
| PGP fingerprint stub dans SECURITY.md | Cred publique | 10 min |

---

## 2. Decision scope v1.0

**v1.0 = Gate 2 complete + Alexandria showcase.**

Gate 3 requiert un audit externe (50-100k$ Trail of Bits, 4-8
semaines). Ce budget n'est pas secure. On ne bloque PAS le tag v1.0
sur l'audit. Le repo est public avec Alexandria fonctionnelle, la
mention "pre-audit" dans le README est honnete et acceptable pour
un projet solo maintainer open source.

L'audit devient un objectif post-v1.0 quand/si le budget est
disponible. Gate 3 operational s'ouvre apres l'audit.

---

## 3. Plan sprint S31-S35

### S30 (EN COURS) — Finir tel quel

Phases restantes C/D/E. Pas de changement.

| Phase | Contenu | Statut |
|---|---|---|
| C | Warrant canary Niveau 1 FROST DKG | A faire |
| D | G2 HARDENING refresh + split inference research | A faire |
| E | Wrap-up + verification + audit plan S31 | A faire |

### S31 — task_runner reel + carries obligatoires

**Theme** : le worker fonctionne pour de vrai.

| Phase | Contenu | LOC estime |
|---|---|---|
| Phase 0 | Audit gate S30 | - |
| Phase A | **task_runner reel** : copier LlmBackend trait + OllamaBackend depuis nexus-worker-core vers nexus-executor, ajouter CLI --ollama-endpoint, wire execute_task() -> backend.generate(), tests E2E executor -> Ollama | ~300 LOC Rust |
| Phase B | §9.5 output filter wire E2E (carry 2/3 -> 3/3) | ~400 LOC multi-crate |
| Phase C | iroh 0.98 upgrade (lever pin Day 0 #3, sprint dedie absorbe ici) | ~200 LOC + Cargo.toml bumps |
| Phase D | Tor transport phase 1 arti 2.0 (SOCKS proxy, opt-in config) | ~500 LOC Rust |
| Phase E | Wrap-up + verification + audit plan S32 | - |

**Delivrable cle** : `cargo run -p nexus-executor -- --ollama-endpoint http://localhost:11434` recoit un task via IPC et retourne une vraie inference Ollama.

### S32 — Alexandria sprint 1 : backend + MCP tools

**Theme** : le coeur d'Alexandria fonctionne sur 1 noeud.

**Pre-requis** : telecharger corpus 1.1 TB (Kiwix torrent, ~1h30
sur fibre 2.3 Gbps).

| Phase | Contenu | LOC estime |
|---|---|---|
| Phase 0 | Audit gate S31 | - |
| Phase A | Scaffolding `packages/nexus-app-alexandria/` : NexusApp subclass, pyproject.toml, SQLite schema (articles, languages, blob_hash, wikidata_id), migrations | ~300 LOC |
| Phase B | Indexation : libzim lecteur ZIM + tantivy index full-text 324 langues + Wikidata entity linking via qwikidata | ~500 LOC |
| Phase C | 3 MCP tools : knowledge_search (full-text tantivy), knowledge_compare (Wikidata cross-lingual), knowledge_coverage (stats par langue). Enregistrement dans mcp_server.py | ~400 LOC Python + 200 LOC MCP |
| Phase D | 4 TabView tabs UI : Search, Compare, Coverage + integration frontend iframe BrowsedProject | ~350 LOC |
| Phase E | Wrap-up + verification + audit plan S33 | - |

**Decisions techniques gelees** :
- D1 : tantivy (Rust, MIT) pour indexation. Pas meilisearch (BUSL).
- D2 : libzim (Python, MIT) pour lecture ZIM. Pas de Rust wrapper.
- D3 : qwikidata pour entity linking Wikidata.
- D4 : knowledge_drift differe S33 (necessite diffs historiques ZIM).
- D5 : NexusApp SDK plugin (meme pattern que nexus-app-gov).

**Delivrable cle** : `knowledge_search("Tiananmen", lang="zh")` via
MCP retourne des articles Wikipedia depuis l'index local tantivy.

### S33 — Alexandria sprint 2 : P2P + multi-noeud

**Theme** : Alexandria distribue des donnees entre 3+ noeuds.

| Phase | Contenu | LOC estime |
|---|---|---|
| Phase 0 | Audit gate S32 | - |
| Phase A | iroh-blobs distribution : BlobsManager (publish article blob, fetch_ticket from peer, cache locally), gossip topic alexandria.blobs | ~300 LOC |
| Phase B | Deploy shell-daemon sur 2 VPS (binaire release) + test E2E multi-noeud : publish corpus -> fetch cross-node -> cache verify | Infra + ~200 LOC tests |
| Phase C | knowledge_drift (4e MCP tool) : diffs historiques ZIM + timeline UI | ~400 LOC |
| Phase D | Bug fixes protocole P2P (timeout blobs, NAT traversal, cache eviction, gossip discovery lente) + metriques reseau | ~300 LOC |
| Phase E | Wrap-up + verification + audit plan S34 | - |

**Delivrable cle** : VPS 2 cherche "Einstein" -> article arrive de
VPS 1 via iroh-blobs -> VPS 2 le cache -> VPS 3 le fetch depuis
VPS 2 (pas VPS 1). On coupe VPS 1 -> articles toujours accessibles.

### S34 — Polish public + v1.0 prep

**Theme** : le repo est pret a etre vu par le monde.

| Phase | Contenu |
|---|---|
| Phase A | README rewrite showcase (Alexandria demo GIF/video, architecture diagram, "try it now" section), issue/PR templates .github/, PGP fingerprint SECURITY.md |
| Phase B | Playwright E2E Alexandria (search + compare + coverage dans iframe), regression suite complete |
| Phase C | SPRINT_LOG.md rows S31-S34, CLAUDE.md update, HARDENING_ROADMAP last_validated S34 |
| Phase D | Performance : benchmark Alexandria latency (cold cache < 500ms, warm < 50ms), optimisation tantivy index |
| Phase E | Wrap-up + pre-tag checklist + audit plan S35 |

### S35 — Tag v1.0 + go public

**Theme** : flip day.

| Phase | Contenu |
|---|---|
| Phase A | Pre-tag verification : 30+ fail-fast rows, full regression, multi-noeud smoke test VPS |
| Phase B | Execute MIRROR_FALLBACK.md §3.1-3.8 flip sequence (LT-2 Radicle, ~60 min) : flip GitHub/Codeberg public, init Radicle identities, add GHA secrets, update CANARY.txt |
| Phase C | Annonce : Hacker News post, README badges, first public canary publish |
| Phase D | Bug fixes post-publication (inevitable), triage premiers issues externes |
| Phase E | Retrospective v1.0, roadmap v1.1 (D&D ? Surveillance foret ? audit externe ?) |

---

## 4. Carries et LT items triage

### Resolus avant v1.0

| Item | Sprint |
|---|---|
| P2-B-1-S28 CI 3/3 MANDATORY | S30 Phase B (fait) |
| P2-C-1-S28 blob-serve COOP/COEP | S30 Phase B (fait) |
| P2-REVIEW-B-2 §9.5 output filter wire | S31 Phase B |
| P2-REVIEW-C-1 task_runner reel | S31 Phase A |
| Playwright COEP regression | S34 Phase B |

### Differes post-v1.0 (non bloquants)

| Item | Justification |
|---|---|
| Audit externe Cure53/ToB | Budget 50-100k$ non secure |
| Nym mixnet | SDK paused crates.io |
| TEE H100 | Pas de hardware partenaire |
| DKG distribue FROST | Trusted dealer suffit N=3 |
| Full process isolation blob-serve | Rewrite > 1000 LOC |
| LT-1 Kudos fairness | Trigger empirique Gini post-v1.0 |
| LT-3 Sybil families | Trigger 3+ contrib post-v1.0 |
| LT-4 Biometric gate | Trigger partnership post-v1.0 |

### Executes au tag v1.0

| Item | Action |
|---|---|
| LT-2 Radicle flip | MIRROR_FALLBACK.md §3.1-3.8 (~60 min) |
| LT-5 Redundancy persistence | Wire SQLite (code existe, pas branche) |
| LT-6 iroh 0.98 | Absorbe S31 Phase C (lever pin Day 0 #3) |

---

## 5. Infrastructure multi-noeud

### Ressources disponibles

| Noeud | Role | Hardware | Statut |
|---|---|---|---|
| Machine dev | Coordinator + worker (Ollama) + Alexandria source 1.1 TB | RTX 5080 16GB, SSD 4TB, fibre 2.3 Gbps | Operationnel |
| VPS 1 | Shell daemon + Alexandria peer cache | Hetzner CX11, CPU, SSD | Provisionne (S10) |
| VPS 2 | Shell daemon + Alexandria peer cache | Hetzner CX11, CPU, SSD | Provisionne (S10) |
| VPS 3 | pkarr relay + bootstrap DHT | Docker, Hetzner CX11 | Operationnel (S19) |

### Pas de GPU sur VPS

Alexandria = stockage + index, pas de GPU requis. Les VPS CPU
suffisent. Le GPU (RTX 5080) sert uniquement au worker Ollama sur
la machine dev pour prouver le task_runner reel (S31 Phase A).

Le partage GPU distribue (D&D, Surveillance foret) necessitera du
hardware GPU additionnel (laptop RTX 3060+ ou VPS GPU Vast.ai/RunPod).
C'est post-v1.0 (v1.1+).

---

## 6. Timeline estimee

```
S30 (avril 2026)     ████ Canary FROST + docs + wrap-up
S31 (mai 2026)       ████ task_runner + carries + iroh 0.98 + Tor
S32 (mai-juin 2026)  ████ Alexandria sprint 1 (backend + MCP)
S33 (juin 2026)      ████ Alexandria sprint 2 (P2P + multi-noeud)
S34 (juin-juil 2026) ████ Polish public + benchmarks
S35 (juillet 2026)   ██   Tag v1.0 + flip public
                          ^
                          Repo public ~juillet 2026
```

~5 sprints / ~2.5 mois depuis maintenant.

---

## 7. Risques

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | libzim cross-platform (Windows .dll) | Medium | Medium | Fallback : index pre-build sur Linux, servir via VPS |
| R2 | tantivy 324 langues = index > 100 GB | Low | Low | SSD 4 TB, headroom 2.9 TB |
| R3 | iroh 0.98 breaking changes | Medium | High | Pin exact, tester avant merge |
| R4 | NAT traversal VPS <-> machine dev | Medium | Medium | Relay WSS fallback (deja en place) |
| R5 | Hacker News front page = flood | Low | Medium | Rate-limit GCRA + CDN si besoin |
| R6 | Budget audit non secure post-v1.0 | High | Medium | v1.0 = Gate 2 honnete, audit = stretch goal |

---

## 8. Ce document n'est PAS

- Un kickoff sprint (pas de D1..D5 gelees, pas de scope cuts)
- Un plan d'execution (pas de §Phase X detaille)
- Un remplacement du HARDENING_ROADMAP (qui reste authoritative
  pour la securite)

C'est une **vision directrice S31-S35** qui informe les kickoffs
individuels. Chaque sprint aura son propre kickoff avec D1..D5
gelees, design review G1, et G8 preflight par phase.
