# Sprint 72 — Design Review Board (G1)

**Date** : 2026-05-31 (revise apres arbitrage PO Checkpoint §11 :
D2 ollama-rs partout + front UX complet in-scope).
**Sprint** : 72 — Factory provider routing (ProviderRouter multi-LLM, QUICK WIN)
**Reviewer** : self-review profond (auto-challenge systematique, agent
`nexus-sprint-kickoff` fallback portable)

---

## Scoring

| D# | Titre | Source recente | Alternative | [DETER] Crypto | [DETER] Rust | Code verifie | Verdict |
|---|---|---|---|---|---|---|---|
| D1 | Trait `ExecutionTarget` enum-dispatch → `Pin<Box<dyn Stream<StreamChunk>>>` | ok (rig-core 0.35.0 2026 ; enum_dispatch) | ok (async-trait+Box<dyn>, GAT, Box<dyn Stream> compares) | N/A | ok (3 alternatives Rust, retenu Rust-native) | ok (`llm_bridge.rs:44,95` lus) | ✅ |
| D2 | `ollama-rs 0.3.4` partout (Factory dep + bump worker 0.2.6→0.3.4) | ok (ollama-rs 0.3.4 docs.rs 2026-05-31 ; changelog 0.3.0 GenerationOptions→ModelOptions) | ok (HTTP direct reqwest, reuse worker-core, ollama-rs partout compares) | N/A | ok (Rust-native adoptee partout, version unique) | ok (`ollama.rs:160-254` quorum + `Cargo.toml` lus) | ⚠️ (risque migration quorum) |
| D3 | NetworkProvider submit→poll → un seul `Delta`+`Done` (async non-streaming, PO-14) | ok (Spring Boot submit/poll 2026, Forge realtime) | ok (poll-to-single-chunk vs progress-polling vs SSE-passthrough) | N/A | ok (Rust `reqwest`+`async-stream`, deja dep) | ok (`http.rs:1404`, `tasks_api.rs`, `types.rs:13` lus) | ✅ |
| D4 | Cabler `provider` dans `ChatSession` + dispatch `handle_chat_stream` + UI selectable | ok (code present, gap factuel S72 ; AnythingLLM/OpenWebUI dropdown 2026) | ok (persist session vs requery vs header) | N/A | N/A (cablage interne + front TS) | ok (`operator_server.rs:52,729,822,898` + `factory-operator/package.json` lus) | ✅ |
| D5 | Deux axes orthogonaux : `ExecutionTarget` (run) vs prompt-adaptation `Provider` (D8/§P53) | ok (§P53 S71, process.rs:837) | ok (unifier vs nommer distinct vs 3e champ) | N/A | N/A (taxonomie) | ok (`process.rs:837` providers_list + §P53 lus) | ✅ |

**Resume** : D1 ✅, D2 ⚠️, D3 ✅, D4 ✅, D5 ✅.
Rigor signal G4 satisfait (1 ⚠️ sur 5).

Le G1 gold standard est 1-2 ⚠️ sur 5 ; 1/5 est dans la cible. Le ⚠️ s'est
**deplace** suite a l'arbitrage PO : il ne porte plus sur le compromis
Rust-first (D2 adopte maintenant `ollama-rs`, Rust-native, version unique
cross-crate — Rust-first SATISFAIT) mais sur le **risque de migration**
(le bump 0.2.6→0.3.4 touche le code quorum greedy-seed fraichement
stabilise S71).

---

## Findings

### D2 ⚠️ — bump `ollama-rs` 0.2.6→0.3.4 touche le code quorum determinisme S71

**Detail** : l'arbitrage PO adopte `ollama-rs 0.3.4` partout (Factory en
dep directe + bump `nexus-worker-core` de 0.2.6) pour eviter la
divergence de version cross-crate. Cote Rust-first c'est **superieur** au
HTTP-direct precedent (lib Rust-native dediee, version unique, pas de
client maison). MAIS le bump touche `nexus-worker-core/src/llm/ollama.rs`
qui implemente `deterministic_options` (greedy seed-fixe, B-2, stabilise
S71 Phase B avec 4 tests quorum). Le changelog ollama-rs **0.3.0** a un
breaking change qui touche precisement ce code : **`GenerationOptions`
est renomme `ModelOptions`** (source : github.com/pepperoni21/ollama-rs
releases). Risque : la migration casse silencieusement le determinisme du
quorum (seed mal forwarde) sans casser la compilation.

**Verification API (context7 ollama-rs 0.3.4, queried 2026-05-31)** :
- `ModelOptions::default().temperature(f32).seed(i32).num_predict(i32)
  .top_k(u32).top_p(f32)` — les builders `.temperature()` et `.seed()`
  utilises par `deterministic_options` (`ollama.rs:244-251`) **SURVIVENT**
  au bump (memes signatures, juste le type renomme). Import change :
  `ollama_rs::generation::options::GenerationOptions` (0.2.6) →
  `ollama_rs::models::ModelOptions` (0.3.4).
- `GenerationRequest::new(model, prompt).options(opts)` et `.system()`
  **inchanges**. `ollama.generate(req)` inchange.
- `generate_stream(req)` (feature `stream`) → stream de
  `Vec<GenerationResponse>` (`.response` text + `done`) — l'API Factory.
- **Donc PAS un DESIGN-CONFLICT** : l'API seed deterministe survit, c'est
  une migration mecanique (rename type + import) + re-test obligatoire. Le
  ⚠️ est un risque de regression, pas un blocage de gouvernance.

**CVE/RUSTSEC (S1b/G13)** : aucune advisory RustSec sur `ollama-rs`
(verifie WebSearch rustsec 2026-05-31 ; les CVE Ollama 2026 — Bleeding
Llama CVE-2026-7482 — visent le **serveur** Ollama, pas la lib Rust).
0.3.4 advisory-clean.

**Decision** : **acknowledge + adjust** — le ⚠️ reste (risque migration
reel sur du code quorum sensible), et le kickoff §4 D2 + §9 R7 + le plan
§6 ajoutent les mitigations :
1. La migration worker (rename `GenerationOptions`→`ModelOptions`, adapter
   imports) est faite **TOT** (Phase C, avant/avec l'Ollama provider).
2. Les 4 tests quorum determinisme S71 (`verifiable_task_uses_greedy_
   seed`, `two_honest_workers_same_hash`, `quorum_accepts_deterministic_
   redundancy`, `quorum_rejects_nondeterministic_divergence`) deviennent
   un **critere binaire** de la phase migration — ils doivent rester verts.
3. Le preflight Phase C re-verifie l'API seed/options 0.3.4 (S1b) avant la
   1ere ligne de migration.

Le ⚠️ est honnete : un seul, sur le risque migration, pas rubber-stamp.

---

## Checklist [DETER] (applicable)

### Crypto/spec
- N/A — aucun D-choice S72 ne touche crypto ni spec standardisee. Le
  routing provider est de l'orchestration subprocess/HTTP. La signature
  des taches reseau (NetworkProvider) reutilise le chemin Ed25519+JCS
  deja signe cote coordinator (S71), Factory ne re-signe rien. Le bump
  ollama-rs ne touche aucune primitive crypto (seed deterministe ≠ crypto).

### Rust-first
- [x] D1 cite alternatives Rust-native production (rig-core enum/trait,
  enum_dispatch, async-trait) — retenu Rust-native enum-dispatch.
- [x] D2 retenu **Rust-native** `ollama-rs 0.3.4` partout (Factory dep +
  bump worker) — **Rust-first SATISFAIT**, version unique cross-crate, pas
  de client HTTP maison. Le ⚠️ porte sur le risque migration (quorum), pas
  un gap Rust-first. Alternatives comparees : HTTP direct reqwest (rejete :
  duplique un client que `ollama-rs` fournit), reuse `worker-core` (rejete :
  decision gelee Factory hors daemon).
- [x] D3 retenu Rust-native (`reqwest` + `async-stream`, deja dep).
- Exemptions : D4 (cablage interne + front UX TS), D5 (taxonomie/doc) —
  D4 front est de l'UX frontend (exemption Rust-first §6.1.1).
