# Sprint 72 — Kickoff (Factory provider routing — ProviderRouter multi-LLM)

**Ecrit** : 2026-05-31 (post-audit gate S71 PASS `636b9de`).
**Type** : **sprint PAIR** — une phase dette est **reservee**
(Phase B, non-negociable, non-convertible en feature — §6.2.1 Regle 1).
Un item **3/3 MANDATORY** a traiter : **P2-F-3** (prompt file coupling),
1/3 S70 → 2/3 S71 → **3/3 S72** (cf. §6).
**Tip master d'entree** : `636b9de` (audit findings S71 PASS — 0 P0,
0 P1, 1 P2 = P2-H-1, 2 P3 = P3-F-1 + P3-OS-1).
**Phase 0 audit Sprint 71** : **DEJA JOUE** — `636b9de`
(`chore(planning): Sprint 71 audit findings — PASS (S72 Phase 0)`).
**Aucun fix `fix(sprint71)` requis** : le seul P2 (P2-H-1) ne bloque pas
le kickoff (la defense Factory EST en place et testee, seul le catalogue
menace canonique accuse un retard documentaire — il est routé comme
livrable S72 Phase A, cf. §6).
**Version archive** : v2.1 — « Protocole Neutre + Factory/RRV » (OPEN).
S72 continue le meme arc que S71 (Arc 3.5 Factory Complete Vision) —
aucune release publiee depuis, donc **S72 reste v2.1** (SPRINT_LOG.md
§7.3 : v2.1 OPEN, derniere row S71).
**Roadmap source** : `.planning/roadmap_v5_factory_complete_vision.md`.
Sprint **2 sur 6** (S71-S76), Arc 3.5 « Factory Complete Vision ».

---

## Sources context7 + WebSearch consultees (pre-gel)

Recherche G9 effectuee AVANT de figer D1..D5. Dates absolues, versions,
URLs.

### OSS prior art — abstraction provider LLM Rust (D1)

| Source | Date | Finding cle |
|--------|------|-------------|
| context7 `/0xplaygrounds/rig` (rig-core **0.35.0**) | queried 2026-05-31 | Trait `CompletionModel` provider-agnostique + streaming via enum `RawStreamingChoice` / `StreamedAssistantContent` (text deltas, tool call updates, usage). Le pattern « stream d'enum chunks unifie cross-provider » est exactement le `StreamChunk` que SBFB a deja (`llm_bridge.rs:44`). SemVer commence a v1.0, breaking sur main. |
| docs.rs `rig-core/latest` (0.35.0) | 2026-05-31 | `MultiTurnStreamItem::{Text, FinalResponse}` — le stream porte les deltas ET un evenement final avec metadata. Valide le contrat `Delta + Done{cost,duration,result}` de SBFB. |
| crates.io `enum_dispatch` | 2026-05-31 | Proc-macro qui implemente un trait pour un set ferme d'enum variants ; « faster than dynamic dispatch, type info built-in to each variant, avoids vtable lookup ». Reference pour le choix enum vs `dyn Trait`. |
| somethingsblog.com « When Enums Beat dyn Trait » | 2025-04-20 | Pour un set ferme de types connus a la compilation, enum+match = static dispatch performant et type-safe, evite l'overhead vtable, permet l'inlining dans les bras de match. |
| smallcultfollowing.com « Dyn async traits, part 10 » | 2025-03-24 | Etat 2025 de l'async-in-trait dyn-safety : `async fn` en trait n'est pas encore dyn-safe sans `async-trait` (qui box en `Pin<Box<dyn Future>>`). Pousse vers enum-dispatch pour eviter le double-box (Future + Stream). |
| docs.rs `async-trait` | 2026-05-31 | Transforme `async fn` en `Pin<Box<dyn Future + Send>>`. Cout : allocation par appel + perte d'inlining. Alternative comparee, rejetee D1. |

### OSS prior art — Ollama Rust + streaming + migration 0.2→0.3 (D2)

| Source | Date | Finding cle |
|--------|------|-------------|
| context7 `/pepperoni21/ollama-rs` (queried 2026-05-31) | 2026-05-31 | `ModelOptions::default().temperature(f32).seed(i32).num_predict(i32).top_k().top_p()` (0.3.4) ; `GenerationRequest::new(model,prompt).options(opts).system()` inchange ; `generate_stream(req)` (feature `stream`) → stream de `Vec<GenerationResponse>` (`.response` text + `done`). |
| github.com `pepperoni21/ollama-rs` releases (changelog **0.3.0**) | 2026-05-31 | **BREAKING 0.2→0.3 : `GenerationOptions` renomme `ModelOptions`.** Seul breaking touchant le code worker quorum. Import 0.2.6 `ollama_rs::generation::options::GenerationOptions` → 0.3.4 `ollama_rs::models::ModelOptions`. Autres : `model_info` field, match Ollama integer width token counters. |
| crates.io / docs.rs `ollama-rs` stable **0.3.4** | 2026-05-31 | Version stable courante. Workspace SBFB pin `0.2.6` cote worker (Cargo.lock verifie) — le bump 0.2.6→0.3.4 est une **decision PO** (§6.6 pin delibere). |
| WebSearch RustSec advisory-db `ollama-rs` | 2026-05-31 | **Aucune advisory RustSec/CVE sur la lib `ollama-rs`** (les CVE Ollama 2026 — Bleeding Llama CVE-2026-7482 — visent le **serveur** Ollama, pas le crate Rust). 0.3.4 advisory-clean (S1b/G13). |
| `crates/sbfb-factory/Cargo.toml` (lu) | 2026-05-31 | Deps actuelles : `reqwest`, `nexus-core-rs`, `sbfb-manifest` — **PAS** `nexus-worker-core` ni `ollama-rs`. S72 ajoute `ollama-rs 0.3.4 { features=["stream"] }` en dep directe. |
| `crates/nexus-worker-core/src/llm/ollama.rs:160-254` (lu) | 2026-05-31 | `req_build` + `deterministic_options` (greedy seed-fixe quorum B-2) : `GenerationOptions::default().temperature(t).seed(s as i32)` + `req.options(opts)`. C'est le code que le bump 0.3.4 migre (rename → `ModelOptions`, builders survivent). 4 tests quorum S71 a re-verifier. |

### OSS prior art — adapter polling → stream (D3)

| Source | Date | Finding cle |
|--------|------|-------------|
| medium.com (Mitesh S. Jat) « Non-Blocking Async LLM API Spring Boot + Kafka + Ollama » | 2026-03 | Le pattern « Submit and Poll » async est l'etape standard avant un upgrade SSE/WebSocket. Valide submit→poll comme primitive correcte pour une tache de fond. |
| developer.atlassian.com « Forge LLM long-running process + Realtime » | 2026 | Long-running prompt async → stream du resultat a l'UI ; polling acceptable comme transport interne, l'UI voit un evenement final. |
| github.com `tokio-rs/async-stream` + docs.rs `async-stream` (**0.3.6** pin) | 2026-05-31 | `stream!` macro transforme `yield expr` en `sender.send(expr).await` ; pattern loop+`interval.tick().await`+`yield` pour emettre des chunks a intervalle. Deja dep `sbfb-factory`. |
| tokio.rs/tokio/tutorial/streams | 2026-05-31 | `tokio::time::interval` + `yield` dans `async_stream::stream!` = pattern de poll-loop canonique. |

### UX intentions provider (D4)

| Source | Date | Finding cle |
|--------|------|-------------|
| WebSearch « LLM provider selector UI local vs cloud vs network 2025 » | 2026-05-31 | AnythingLLM/Open WebUI exposent un dropdown « All Providers » (local Ollama / cloud OpenAI/Anthropic). Le choix execution = trade-off privacy/latence/cout. Cloud 200-800ms vs local sub-10ms — confirme que le reseau WAN (1-3 tok/s, PO-14) n'est PAS pour du chat live. |

### Versions deps confirmees (lockfile)

`axum 0.8.9`, `async-stream 0.3.6`, `futures 0.3.32`, `reqwest 0.12.28`,
`ollama-rs 0.2.6` (worker — **bumpe 0.3.4 S72**, decision PO), `tokio`
(workspace). **Nouvelle dep S72** : `ollama-rs 0.3.4 { features=["stream"] }`
en dep directe `sbfb-factory` + bump pin worker 0.2.6→0.3.4 (D2). C'est
un bump de pin delibere (§6.6), **pas un wire format** — aucun `*_VERSION`
ne bouge (§1.4 reste correct). Front Operator (`tools/factory-operator/`)
deja sur React 19 / Vite 8 / TS 5.9 / Radix+shadcn / i18next (D4 UX).

**Decision crypto/spec nouvelle ?** Non. Le routing provider est de
l'orchestration subprocess/HTTP. La checklist `[DETER]` crypto/spec
(§6.1.1) ne s'applique pas. La checklist `[DETER]` Rust-first s'applique
a D1/D2/D3 (cf. §4 + design_review.md).

---

## §1 Constat d'entree

### §1.1 D'ou on part

S71 a CLOSE (PASS, `636b9de`) en assainissant la couche compute (B-1 cle
dispatch alignee, B-2 quorum greedy seed-fixe, B-3 1er E2E cross-process),
en durcissant la securite Factory (G2 gate SSE, G7 token+Host+CORS, G9
modele `opus-4-8`, G12 timeout), et en reconciliant le bloc off-sprint
(~14 commits, +5500 lignes) — retro-review RECONCILED + 16 tests de
surfaces auparavant a 0 test. **Le socle compute marche enfin** : une
tache route reellement d'un coordinator vers un worker.

S72 est le **quick win** qui pose la 1ere brique « Factory front-end de
tout SBFB » : le chat Factory peut router sa sous-tache d'inference vers
**Claude cloud** (defaut, inchange), **Ollama local**, ou **le reseau**
(submit→poll async). Aujourd'hui `ChatSendRequest.provider` est capte
mais **jamais lu** — `handle_chat_stream` appelle toujours
`spawn_claude_stream`. S72 cable ce champ.

C'est strictement un sprint de **cablage** : l'infra existe (le contrat
`StreamChunk` SSE unifie, `assemble_prompt`, le dispatch reseau
`/api/v1/tasks/submit`, le backend Ollama dans le worker). Le travail est
d'introduire un point de dispatch `ExecutionTarget` derriere le SSE et de
brancher deux providers neufs sur le contrat `StreamChunk` deja en place.

### §1.2 Ancrage roadmap v2.1 (Arc 3.5)

Arc 3.5 « Factory Complete Vision » (roadmap v5, CANON), 6 sprints
S71-S76. Position : **sprint 2/6**.

```
S71 assainir+securite+reconciliation (DONE, fonde TOUT)
  └─ S72 quick win: chat Factory route les taches existantes  ← ICI
       └─ S73 recherche reseau cablee (FTS5 fraicheur + SearchResult enrichi)
            └─ S74 atelier: rouvrir/forker un projet reseau
                 └─ S75 GPU partage PROUVE cross-machine
                      └─ S76 STRETCH: sharding pipeline
```

**Dependances aval** : S72 livre le point de dispatch `ExecutionTarget`
que S75 reutilisera (router une sous-tache vers un GPU distant = un
`NetworkProvider` mature). S72 NE livre PAS la recherche reseau (S73),
NE livre PAS le fork (S74), NE livre PAS la preuve cross-machine (S75).
**QUICK WIN STRICT** (PO + roadmap §5 risque #6) : phaser strict, ne pas
deborder en « atelier complet ».

### §1.3 Compteurs tests entree (tip `636b9de`)

| Suite | Count |
|---|---|
| Rust nextest | 1528 (compte canonique CI Linux ; le full workspace Windows peut afficher 1532 — ecart `#[cfg]`-gate launcher GTK, cf. audit S71 Track A) |
| Vitest | 279 (front `web/` non touche S71) |
| size-limit | 6/6 |
| **Total** | **~1813** (1528 Rust + 279 Vitest + 6 size) |

Re-mesure exacte au `plan.md §1` sur le SHA reel post-kickoff.

### §1.4 Pre-launch protocol policy (rappel)

Rien n'est pousse vers origin (22 ahead). **Reconciliation locale libre**.

- `TASK_FORMAT_VERSION` et les `*_ANNOUNCEMENT_VERSION` restent a **1**.
  S72 ne touche AUCUN wire format reseau : le NetworkProvider est un
  **client** de `/api/v1/tasks/submit` (`TaskSubmission` existant,
  inchange) — il n'ajoute pas de champ au wire, il consomme le contrat
  S71. Pas de bump.
- `#[serde(default)]` reste legitime pour la robustesse runtime :
  `ChatSendRequest.provider` est deja `#[serde(default = "default_provider")]`
  — un client qui omet le provider obtient `"claude"`, pas un 422. C'est
  de la tolerance runtime, pas de la compat historique.
- Pas de tolerant decoder multi-version. Pas de test « legacy decode ».
- Le `StreamChunk` enum (`llm_bridge.rs:44`) n'est PAS un wire format
  reseau — c'est le contrat SSE local Operator↔front. S'il s'enrichit
  (ex: nouvelle variante de progress reseau), c'est libre (loopback,
  non propage P2P).

---

## §2 Goal

> Sprint 72 cable le **routage d'execution** du chat Factory : la meme
> conversation operateur peut etre executee **sur Claude cloud** (defaut,
> pilote principal — PO-14, inchange), **en local sur Ollama**, ou **sur
> le reseau** (submit→poll async, jamais token-par-token WAN). Un point
> de dispatch `ExecutionTarget` unique derriere le SSE remplace l'appel
> direct a `spawn_claude_stream` ; `ClaudeProvider`, `OllamaProvider`
> (via `ollama-rs 0.3.4`, le worker etant aligne sur cette version) et
> `NetworkProvider` produisent tous le contrat `StreamChunk` deja en
> place. Le champ `provider` (capte mais ignore aujourd'hui) est cable
> de bout en bout (`ChatSendRequest` → `ChatSession` → `handle_chat_
> stream`) et **selectionnable depuis l'UI** : le front Operator recoit
> l'UX intentions COMPLETE (« Executer sur Claude / en local / sur le
> reseau » + etats reseau riches), pas du jargon `provider/kind`. Phase A
> absorbe P2-H-1 (catalogue menace Operator) avant d'etendre la surface ;
> Phase B reserve la dette (P2-F-3 3/3 + carries compute).
> **Critere SMART : 100% des rows fail-fast vertes au
> `sprint72_verification.md §Fail-fast checklist`, mesure binaire au
> Phase F wrap-up.** La fail-fast checklist (24-30 rows executables, cf.
> plan §Fail-fast) EST la source of truth mesurable du goal.

---

## §3 Phase 0 — Audit gate Sprint 71

**DEJA JOUE.** `sprint71_audit_findings.md` (`636b9de`), 9 tracks A-I,
verdict **PASS**. Diff audite `201b24d..0b4e7f3` (38 commits, off-sprint
inclus). Resume :

- **0 P0, 0 P1** — aucun `fix(sprint71)` requis avant Phase A.
- **1 P2** : P2-H-1 — surface Operator `:3001` (write `/api/artifacts/
  draft` + spawn `bypassPermissions`) absente de `THREAT_MODEL.md` +
  `LOOPBACK_ENDPOINTS_TRUST_TIERS.md`. Defense faite+testee (PATTERNS
  §P35), seul le catalogue menace canonique accuse le retard. **Trigger
  explicite (audit S71 Carry-Over) : avant toute extension de la surface
  Operator** — or S72 (ProviderRouter) TOUCHE precisement cette surface
  (le SSE chat). → routé Phase A (exit binaire, §6).
- **2 P3** : P3-F-1 (recap body Phase D « C=EXECUTE » vs reel
  SCOPE-CUT-CONSISTENT, cosmetique) ; P3-OS-1 (`operator_server.rs:519`
  predicat OR duplique, pre-existant S70, benin).

Suites Docker `sbfb-ci` : 0 regression (1 flake timing E2E worker code
intouche S54, 3/3 PASS isole — non classe).

---

## §4 Decisions Day 0 (D1..D5 gelees)

### D1 — Trait `ExecutionTarget` enum-dispatch → `Pin<Box<dyn Stream<StreamChunk>>>`

**Sources consultees** :
- context7 `/0xplaygrounds/rig` (rig-core 0.35.0, queried 2026-05-31) :
  `CompletionModel` trait + streaming enum `RawStreamingChoice` —
  pattern stream-d'enum-chunks unifie cross-provider.
- crates.io `enum_dispatch` (2026-05-31) : enum > vtable pour set ferme.
- somethingsblog.com (2025-04-20) : enum+match = static dispatch,
  inlining, type-safe pour set ferme connu a la compilation.
- smallcultfollowing.com part 10 (2025-03-24) : `async fn` en trait pas
  dyn-safe sans box ; eviter le double-box Future+Stream.
- Code lu : `llm_bridge.rs:44` (`enum StreamChunk`), `llm_bridge.rs:95`
  (`spawn_claude_stream(prompt, model, cwd) -> impl Stream<Item=StreamChunk>`),
  `llm_bridge.rs:61` (`assemble_prompt`).

**Retenu** : introduire un **enum d'execution** ferme a 3 variantes plus
une fonction de dispatch qui renvoie un type-stream **unifie** :

```rust
// crates/sbfb-factory/src/provider_router.rs (NEW)
pub enum ExecutionTarget {
    Claude { model: String },
    Ollama { model: String },
    Network { project_id: String, model: String },
}

pub type ProviderStream =
    std::pin::Pin<Box<dyn futures::Stream<Item = StreamChunk> + Send + 'static>>;

impl ExecutionTarget {
    /// Parse the wire `provider` string into a closed target.
    pub fn from_provider(provider: &str, model: &str, project_id: &str) -> Self { /* … */ }

    /// Dispatch to the matching provider. Each arm returns a stream of the
    /// SAME `StreamChunk` contract — the SSE layer is provider-agnostic.
    pub fn run(self, prompt: String, cwd: PathBuf) -> ProviderStream { /* match self */ }
}
```

`spawn_claude_stream` (`llm_bridge.rs:95`) devient le corps du bras
`Claude` — **comportement inchange** (D6 idle-timeout S71 conserve). Les
3 bras boxent leur `impl Stream` heterogene dans le type-alias
`ProviderStream` commun. `handle_chat_stream` appelle
`ExecutionTarget::run(...)` au lieu de `spawn_claude_stream` directement.

**Rejete** :
- *`async-trait` + `Box<dyn Provider>`* : `async-trait` box chaque appel
  en `Pin<Box<dyn Future>>` (docs.rs async-trait), et un trait avec une
  methode retournant un Stream ajoute un **second** box (Future-of-Stream).
  Double-box + perte d'inlining pour un set de 3 providers fermes connus
  a la compilation. Rejete : enum-dispatch est strictement superieur ici
  (enum_dispatch, somethingsblog 2025).
- *GAT `trait Provider { type Stream: Stream; }`* : RPITIT/GAT en trait
  pousse la complexite de types (lifetimes, dyn-safety) sans benefice —
  on a quand meme besoin d'un type-stream unifie au point SSE
  (`SseStream` est deja `Pin<Box<dyn Stream>>`, `operator_server.rs:801`).
  Rejete : sur-ingenierie pour 3 variantes.
- *`Box<dyn Stream>` direct sans enum (3 fonctions libres + if/else dans
  le handler)* : disperse la logique de routage dans le handler HTTP,
  pas testable en isolation, pas extensible proprement S75. Rejete :
  l'enum centralise le routage et se teste hors-HTTP.

**Implications code** : NEW `crates/sbfb-factory/src/provider_router.rs`
(+ `mod provider_router` dans `lib.rs`) ; `llm_bridge.rs:95`
`spawn_claude_stream` reste, appele par le bras `Claude` ;
`operator_server.rs:898` remplace l'appel direct par `ExecutionTarget::run`.

### D2 — `ollama-rs 0.3.4` partout (Factory dep directe + bump worker 0.2.6→0.3.4)

**Decision PO (Checkpoint §11, arbitrage 2026-05-31)** : adopter le crate
dedie `ollama-rs 0.3.4` **partout** et **aligner le worker dessus** (bump
`nexus-worker-core` 0.2.6→0.3.4) pour eviter la divergence de version
cross-crate. La Factory utilise `ollama-rs 0.3.4` (`generate_stream`) en
dep directe. Le HTTP-direct-reqwest est **rejete** par le PO.

**Sources consultees** :
- context7 `/pepperoni21/ollama-rs` (queried 2026-05-31) :
  `ModelOptions::default().temperature(f32).seed(i32).num_predict(i32)
  .top_k().top_p()` ; `GenerationRequest::new(model,prompt).options(opts)
  .system()` ; `generate_stream(req)` (feature `stream`) → stream de
  `Vec<GenerationResponse>` (`.response` text + `done`).
- github.com pepperoni21/ollama-rs releases changelog **0.3.0**
  (2026-05-31) : **BREAKING — `GenerationOptions` renomme `ModelOptions`**.
  Import 0.2.6 `ollama_rs::generation::options::GenerationOptions` →
  0.3.4 `ollama_rs::models::ModelOptions`.
- WebSearch RustSec (2026-05-31) : 0 advisory sur la lib `ollama-rs`
  (CVE Ollama 2026 = serveur, pas le crate). 0.3.4 advisory-clean (S1b/G13).
- Code lu : `crates/sbfb-factory/Cargo.toml` (pas encore `ollama-rs`),
  `nexus-worker-core/src/llm/ollama.rs:160-254` (`deterministic_options`
  quorum B-2 : `GenerationOptions::default().temperature(t).seed(s as i32)`
  + `req.options(opts)`).

**Retenu** :
1. **Factory** : ajouter `ollama-rs = { version = "0.3.4", features =
   ["stream"] }` en dep directe de `sbfb-factory`. Le bras `Ollama` de
   `ExecutionTarget` (D1) utilise `Ollama::default()` (ou endpoint
   overridable via `SBFB_OLLAMA_ENDPOINT`) + `generate_stream(
   GenerationRequest::new(model, prompt))` ; chaque `GenerationResponse`
   du stream → `StreamChunk::Delta { text: resp.response }`, le chunk
   final (`done`) → `StreamChunk::Done`. Bornage par idle-timeout
   (pattern `spawn_agent_stream` D6 S71). Ollama injoignable →
   `StreamChunk::Error` diagnostic clair (« Ollama introuvable —
   `ollama serve` »).
2. **Worker** : bumper `nexus-worker-core` 0.2.6→0.3.4. **Migration
   mecanique** : le seul breaking touchant le code quorum est le rename
   `GenerationOptions`→`ModelOptions` (`ollama.rs:239-254` +
   import). Les builders `.temperature()` / `.seed(i32)` / `req.options()`
   / `.system()` / `generate()` **survivent** (context7 0.3.4 verifie) —
   l'API seed deterministe est preservee, ce **n'est PAS un
   DESIGN-CONFLICT**. Mais le risque de regression silencieuse du
   determinisme est reel (R7) : les 4 tests quorum S71
   (`verifiable_task_uses_greedy_seed`, `two_honest_workers_same_hash`,
   `quorum_accepts_deterministic_redundancy`,
   `quorum_rejects_nondeterministic_divergence`) deviennent un **critere
   binaire** de la phase migration (Phase C), et le preflight Phase C
   re-verifie l'API seed/options 0.3.4 (S1b) avant la 1ere ligne.

**Rejete** :
- *HTTP direct `reqwest` `/api/generate` NDJSON (client maison)* :
  rejete PO — duplique un client que `ollama-rs` fournit deja teste, et
  laisse le worker sur `0.2.6` (divergence de version cross-crate qui
  reapparaitra au prochain partage de code). Adopter `ollama-rs` partout
  est plus durable (§6.7 horizon long terme).
- *Reuse `nexus-worker-core::OllamaBackend`* : rejete — la decision gelee
  « Factory = crate externe hors daemon » (CLAUDE.md) interdit a
  `sbfb-factory` de tirer le coeur worker (iroh, config, GPU monitor,
  engine). `ollama-rs` est un **crate tiers**, pas worker-core — l'adopter
  ne viole PAS la frontiere d'isolation.
- *Garder le worker sur 0.2.6 + Factory sur 0.3.4* : rejete PO — exactement
  la divergence de version cross-crate qu'on veut eviter. Une seule version
  workspace.

**Implications code** : `crates/sbfb-factory/Cargo.toml` (+`ollama-rs
0.3.4`), `provider_router.rs` bras `Ollama` (`generate_stream` →
`StreamChunk`) ; `crates/nexus-worker-core/Cargo.toml` (bump 0.3.4) +
`src/llm/ollama.rs` (rename `GenerationOptions`→`ModelOptions` + import) ;
4 tests quorum S71 re-verts ; §6.6 bump pin documente.

### D3 — NetworkProvider submit→poll → un seul `Delta` + `Done` (async non-streaming, PO-14)

**Sources consultees** :
- medium.com (Mitesh S. Jat, 2026-03) : pattern « Submit and Poll » async.
- developer.atlassian.com Forge Realtime (2026) : long-running async →
  evenement final a l'UI.
- github.com tokio-rs/async-stream + tokio.rs streams tutorial
  (2026-05-31) : `async_stream::stream!` + loop + `interval.tick().await`.
- Code lu : `http.rs:306` route `POST /api/v1/tasks/submit` →
  `coordinator_submit_task` (`http.rs:1404`) ; `dispatcher.rs:37`
  `submit_task` ; `tasks_api.rs` (`GET /api/v1/tasks/{id}` → `get_task`,
  status string) ; `types.rs:13` `enum TaskStatus { Pending, Dispatched,
  AwaitingQuorum, Completed, Rejected }`.

**Retenu** : `NetworkProvider` est un **client** du daemon local. Il
**soumet** une `TaskSubmission` (prompt assemble, model) via `POST
/api/v1/tasks/submit`, recupere le `task_id`, puis **poll** `GET
/api/v1/tasks/{task_id}` a intervalle (defaut 2s, `tokio::time::interval`
dans `async_stream::stream!`) jusqu'a `completed` ou `rejected`. Pendant
le poll, il emet un `StreamChunk::Debug`/progress (optionnel, label
`status: dispatched/awaiting_quorum`) pour que l'UI montre « en cours sur
le reseau », JAMAIS des `Delta` token-par-token (PO-14, roadmap §5 :
WAN 1-3 tok/s → batch/async, jamais chat live). A `completed`, le
`result_text` final est emis comme **un seul** `StreamChunk::Done`. Un
timeout global borne l'attente. **Le NetworkProvider n'introduit AUCUN
champ wire** — il consomme `TaskSubmission`/`TaskStatus` existants (S71),
pas de bump (§1.4).

**Rejete** :
- *SSE passthrough token-par-token depuis le worker distant* : impossible
  — le dispatch reseau est async non-streaming (le coordinator stocke un
  result, il ne pipe pas de tokens). Promettre du live = mentir sur la
  latence WAN (PO-14, roadmap §5). Rejete.
- *Progress polling « riche » (estimation %, ETA)* : YAGNI pour le quick
  win, le coordinator n'expose pas de progress fin. Un label de statut
  (dispatched/awaiting_quorum/completed) suffit. Differe (S75 dashboard).
- *Streaming via WebSocket vers le coordinator* : sur-ingenierie, le
  contrat existant est HTTP submit+poll. Rejete (hors quick win).

**Implications code** : `provider_router.rs` bras `Network` (client
`reqwest` submit + poll-loop `async_stream` → un `Done`). Le client cible
le daemon loopback local (auth token daemon S16 si requis — a verifier au
preflight Phase C : le submit daemon exige-t-il `X-SBFB-Token` ?).

### D4 — Cabler `provider` : `ChatSendRequest` → `ChatSession` → `handle_chat_stream`

**Sources consultees** :
- WebSearch UX provider selector (2026-05-31) : intentions local/cloud/
  reseau, dropdown, pas de jargon.
- Code lu : `operator_server.rs:52` (`struct ChatSession` persiste
  `model` mais **PAS** `provider`), `:729` (`ChatSendRequest` porte
  `provider` + `model`), `:758` (persiste `model` au send), `:822`
  (`handle_chat_stream` extrait `model` mais **pas** `provider`), `:898`
  (appel direct `spawn_claude_stream`).

**Retenu** : symetriquement au `model` (S71 D4), persister `provider`
dans `ChatSession` au `handle_chat_send` (le SSE GET `/chat/{id}/stream`
n'a pas de body pour le relire). Au `handle_chat_stream`, lire
`session.provider` + `session.model`, construire
`ExecutionTarget::from_provider(&provider, &model, &project_id)` et
appeler `.run(...)`. Le **gate SENSITIVE_ACTIONS reste applique AVANT**
le dispatch (D3 S71 inchange — la securite ne depend pas du provider :
une action sensible est gatee quel que soit le target). Defaut provider =
`"claude"` (PO-14, pilote principal).

**UX intentions COMPLETE in-scope S72 (decision PO Checkpoint §11,
arbitrage 2026-05-31)** : le front Operator (`tools/factory-operator/`,
React 19 / Vite 8 / TS / Radix+shadcn / i18next) recoit l'implementation
COMPLETE — un selecteur d'intentions stylé (« Executer sur Claude / en
local / sur le reseau », CTA en intentions, **jamais** de jargon
`provider`/`kind` — UX obligatoire CLAUDE.md) qui mappe vers
`ChatSendRequest.provider`, + des etats reseau riches (« en cours sur le
reseau » / progress pendant le poll NetworkProvider). Le « stub minimal
+ polish differe S74 » est **rejete** par le PO. Le front complet est
trop gros pour rester un sous-bullet — il a sa **propre phase (E)**, le
backend (Phase D) devant lander **independamment AVANT** le front (E) —
dependance explicite §5 + plan §3. Le front Operator est un package
standalone sans suite Vitest (verifie `package.json` : scripts `build`
= `tsc -b && vite build`, `lint` = `eslint .`, pas de test runner) — la
fail-fast front est `tsc -b --noEmit` + `eslint .` + scan-en-strings si
applicable. Le front respecte la frontiere `tools/factory-ui/src/readonly`
(socle partage S70).

**Rejete** :
- *Re-query le provider depuis une source externe au stream* : le SSE GET
  n'a pas de body, persister dans la session est le pattern deja etabli
  pour `model` (S71). Coherent. Re-query = invente un second mecanisme.
  Rejete.
- *Header `X-SBFB-Provider` sur le GET stream* : melange transport et
  donnee de session, fragile (le front doit re-emettre le header). Rejete.

**Implications code** : `operator_server.rs:52` (`ChatSession` +`provider`),
`:758` (persister `provider` au send, comme `model`), `:822-898`
(`handle_chat_stream` lit `provider`, dispatch `ExecutionTarget`).

### D5 — Deux axes orthogonaux : `ExecutionTarget` (run) vs prompt-adaptation `Provider` (D8/§P53)

**Sources consultees** :
- Code lu : `process.rs:837` `providers_list() = ["claude","codex","gpt",
  "local","human"]` + test `providers_list_is_canonical` ; `docs/rust/
  PATTERNS.md §P53` (S71 D8 : provider prompt-adaptation vs backend
  d'execution, 2 axes orthogonaux deja documentes).

**Retenu** : le `providers_list()` de `process.rs` est l'axe **adaptation
de prompt** (quel agent va consommer le prompt : claude/codex/gpt/local/
human — D8 resolu S71, §P53). Le `ExecutionTarget` de S72 est un axe
**d'execution** distinct (ou tourne l'inference : Claude cloud / Ollama
local / reseau). **NE PAS reutiliser le meme vocabulaire `provider`** pour
les deux — risque de confusion conceptuelle (roadmap : « 3e axe »). Le
champ `ChatSendRequest.provider` existant designe l'**execution** (c'est
le champ que S72 cable). Nommer le nouvel enum `ExecutionTarget` (pas
`Provider`) ancre la distinction dans le type system. Documenter
explicitement la relation des **trois** axes dans `PATTERNS §P55` (NEW) :
prompt-adaptation `Provider` (process.rs) / runtime `LlmBackend`
(worker-core, quorum) / `ExecutionTarget` (Factory chat routing).

**Rejete** :
- *Unifier les trois en un seul `Provider`* : ce sont des concepts
  legitimement distincts (qui adapte le prompt ≠ quel backend execute une
  tache verifiable ≠ ou route le chat operateur). Unifier de force masque
  la difference de modele de menace et de determinisme. Rejete (§P53 a
  deja tranche « 2 axes orthogonaux, non unifies »).
- *Reutiliser le nom `provider` pour l'enum d'execution* : ambiguite
  permanente dans le code et la doc. Le type `ExecutionTarget` est
  auto-documentant. Rejete.

**Implications code** : `provider_router.rs` (nom `ExecutionTarget`),
`docs/rust/PATTERNS.md §P55` (NEW, 3 axes). Pas de modif `process.rs`
(l'axe prompt-adaptation reste inchange).

---

**Acknowledged review findings (G1)** :

Scoring (renseigne par `sprint72_design_review.md`) :
**D1 ✅, D2 ⚠️, D3 ✅, D4 ✅, D5 ✅.**
Rigor signal G4 satisfait (1 ⚠️ sur 5 — dans la cible gold 1-2/5).

- **D2 ⚠️** : suite a l'arbitrage PO, D2 adopte `ollama-rs 0.3.4` partout
  (Factory dep + bump worker 0.2.6→0.3.4). Le ⚠️ Rust-first precedent est
  **RESOLU** (ollama-rs EST Rust-native, version unique cross-crate) ; le
  ⚠️ s'est **deplace** sur le **risque de migration** : le bump touche le
  code quorum greedy-seed fraichement stabilise S71
  (`nexus-worker-core/src/llm/ollama.rs` `deterministic_options`). Le
  changelog 0.3.0 renomme `GenerationOptions`→`ModelOptions` (breaking).
  Decision : **acknowledge + adjust** — l'API seed/options survit au bump
  (context7 0.3.4 verifie : `.temperature()`/`.seed(i32)`/`.options()`
  inchanges → migration mecanique, PAS un DESIGN-CONFLICT), MAIS le risque
  de regression silencieuse du determinisme est reel. Mitigations : (1)
  migration worker faite TOT (Phase C, avant/avec l'Ollama provider) ; (2)
  les 4 tests quorum S71 deviennent un critere binaire de la phase
  migration ; (3) preflight Phase C re-verifie l'API seed 0.3.4 (S1b).
  Risque trace R7 (§9). Le ⚠️ reste acknowledged (un seul, honnete).

---

## §5 Plan Phase outline A..F

### Phase A — Catalogue menace Operator (P2-H-1) + reservation surface SSE

Avant d'etendre la surface SSE (ProviderRouter touche le chat), **fermer
P2-H-1** (audit S71, trigger « avant extension surface Operator ») :
ajouter l'entree Operator `:3001` a `LOOPBACK_ENDPOINTS_TRUST_TIERS.md`
(endpoint + trust tier + capacite write/spawn) ET une entree menace
(CSRF/DNS-rebinding + spawn-agent) dans `THREAT_MODEL.md` referencant
`PATTERNS §P35`. Documenter que le **dispatch reseau** ajoute par S72
(submit→poll vers le daemon loopback) reste dans la frontiere loopback
deja durcie. **Critere : `THREAT_MODEL.md` et `LOOPBACK_ENDPOINTS_TRUST_
TIERS.md` referencent la surface Operator ; P2-H-1 exit condition (audit
S71 Carry-Over) satisfaite.** Phase docs+threat (peu/pas de code).

### Phase B — Dette pair (Regle 1, NON-NEGOCIABLE) : P2-F-3 3/3 + carries compute

Phase dette reservee (sprint pair). **Non-convertible en feature.** Items :
- **P2-F-3 (3/3 MANDATORY)** : couplage wrappers `.claude/agents/*.md` →
  `prompts/agent/*.md` (4 refs). Resoudre par un check mecanique : soit
  un test/lint qui verifie que chaque `prompts/agent/<kind>.md` reference
  par un wrapper existe (fail si rompu), soit documenter le contrat de
  stabilite dans `AGENT_SYSTEM.md` + un garde-fou. **Exit binaire :
  P2-F-3 ferme (check en place OU contrat documente+teste), plus jamais
  carry.** D'abord verifier l'etat reel (discipline §G9 sessions
  fraiches) — si deja resolu dans le code actuel, documenter clos.
- **P2-A-2** (E2E n'asserte pas la signature result) : le E2E
  cross-process S71 asserte `results.len()==1` mais pas
  `ResultEntry::verify_signature()`. Ajouter l'assertion (owner S72 audit
  S71).
- **P3-A-3** (`task_id` partage 2 tests), **P3-B-1** (`as i32` cast seed),
  **P3-B-2** (colonne DB `sha256` misnomer, documente §P53) : nettoyage
  cosmetique si peu couteux ; sinon re-documente.

**Critere : P2-F-3 ferme (3/3, exit binaire), P2-A-2 assertion signature
ajoutee, P3 tranches (fix ou re-doc).** Phase non-convertible en feature.

### Phase C — Bump `ollama-rs` worker (migration quorum) + `ExecutionTarget` + Claude + Ollama

Deux blocs ordonnes (la migration worker vient TOT, R7) :
1. **Migration worker (D2)** : bumper `nexus-worker-core` 0.2.6→0.3.4 +
   `crates/sbfb-factory/Cargo.toml` +`ollama-rs 0.3.4 {features=["stream"]}`.
   Rename `GenerationOptions`→`ModelOptions` (`ollama.rs:239-254` +
   import). **Critere binaire migration : les 4 tests quorum S71
   (`verifiable_task_uses_greedy_seed`, `two_honest_workers_same_hash`,
   `quorum_accepts_deterministic_redundancy`,
   `quorum_rejects_nondeterministic_divergence`) restent VERTS.** Preflight
   Phase C re-verifie l'API seed/options 0.3.4 (S1b) avant la migration.
2. **`ExecutionTarget` + Claude + Ollama** : NEW `provider_router.rs` :
   enum `ExecutionTarget` (D1) + dispatch `ProviderStream`. Bras `Claude`
   = `spawn_claude_stream` (inchange, D6 timeout S71 conserve). Bras
   `Ollama` (D2) = `ollama-rs 0.3.4` `generate_stream` → `StreamChunk`
   (`.response`→Delta, `done`→Done), idle-timeout + diagnostic.

**Critere : ollama-rs aligne 0.3.4 partout + 4 tests quorum verts ;
`ExecutionTarget::run` dispatche Claude + Ollama ; bras Claude prouve
byte-equivalent S71 ; mapping Ollama `generate_stream` → StreamChunk
prouve (stub/skip si Ollama absent).**

### Phase D — NetworkProvider submit→poll + cablage backend `provider`

Bras `Network` (D3) : client submit `POST /api/v1/tasks/submit` + poll
`GET /api/v1/tasks/{id}` (`async_stream` loop, intervalle 2s) → un seul
`Done`, timeout global, diagnostic. Cabler `provider` **backend** (D4) :
`ChatSession +provider`, persiste au send, lu au stream, dispatch
`ExecutionTarget`. Gate SENSITIVE_ACTIONS reste applique AVANT dispatch
(tous targets). `PATTERNS §P55` (3 axes) ecrit ici (docs indissociables
du code, §4.1 README). **Critere : `handle_chat_stream` route selon
`session.provider` ; gate avant dispatch prouve ; provider=network
submit→poll→un Done prouve ; provider=claude inchange.** **Le backend
Phase D lande INDEPENDAMMENT du front Phase E** (dependance §3 plan).

### Phase E — Front UX intentions COMPLETE (decision PO)

Implementation COMPLETE du selecteur d'intentions dans
`tools/factory-operator/` : CTA « Executer sur Claude / en local / sur le
reseau » (intentions, **jamais** jargon `provider/kind`) mappant vers
`ChatSendRequest.provider` ; etats reseau riches (« en cours sur le
reseau » / progress pendant le poll). Respecte la frontiere
`tools/factory-ui/src/readonly` (socle S70). **Critere : le front compile
(`tsc -b --noEmit`) + lint (`eslint .`) propres ; l'intention selectionnee
est transmise au backend (`provider`) ; les strings utilisateur en
francais (scan-en-strings si applicable).** Depend de Phase D (backend
provider cable).

### Phase F — Wrap-up

`sprint72_verification.md` (fail-fast rempli) + `sprint73_audit_plan.md`
(pour S73) + `PATTERNS.md` (§P55 si pas deja Phase D) + memory update +
SPRINT_LOG row + CLAUDE.md. **Critere : 100% fail-fast verts, 2 docs
planning, PATTERNS a jour, memory a jour.**

---

## §6 Items carry/dette

### Items 3/3 (traitement Sprint 72)

| Item | Reports | Phase S72 | Exit condition |
|---|---|---|---|
| P2-F-3 prompt file coupling | **3/3** (1/3 S70 → 2/3 S71 → 3/3 S72) | Phase B | Check mecanique en place (test/lint que chaque `prompts/agent/<kind>.md` reference par un wrapper existe) OU contrat de stabilite documente+teste dans `AGENT_SYSTEM.md`. Verifier d'abord l'etat reel — si deja resolu, documenter clos. Plus jamais carry apres S72. |

P2-F-3 est un item **process** (couplage agents/prompts), pas Factory —
il est neanmoins MANDATORY a 3/3 et entre dans la phase dette (Phase B),
pas en carry. Source du compteur : `sprint70_phase_f_review.md` (cree 1/3
S70), `sprint70_verification.md:132` (« 1/3 »), `sprint71_verification.md
§5` (« 2/3, non escalade, differe S72 »).

### Carry absorbes S72

| Item | Reports | Phase S72 | Exit condition |
|---|---|---|---|
| P2-H-1 threat doc lag Operator | 1/1 (nouveau audit S71) | Phase A | Entree Operator `:3001` dans `LOOPBACK_ENDPOINTS_TRUST_TIERS.md` + entree menace `THREAT_MODEL.md` ref §P35. **Trigger DECLENCHE** : S72 etend la surface Operator (ProviderRouter touche le SSE) — l'audit S71 exige de fermer AVANT extension. |
| P2-A-2 E2E sans signature result | 1/1 (nouveau audit S71) | Phase B | E2E cross-process asserte `ResultEntry::verify_signature()` sur le result lu. |
| P3-A-3 task_id partage 2 tests | 1/1 | Phase B | Cosmetique — fix si peu couteux, sinon re-doc. |
| P3-B-1 `as i32` cast seed u32 | 1/1 | Phase B | Cosmetique — re-doc (seed deterministe, pas de perte). |
| P3-B-2 colonne DB `sha256` misnomer | 1/1 | Phase B | Documente §P53 (deja) — confirmer ou renommer si trivial. |

### Carries reconduits S73

| Item | Reports | Justification (renouvelee) |
|---|---|---|
| P2-A-1 (rand upstream) | exemption | Blocker amont (crate `rand` non publiee fix) — hors scope agent. Re-evalue : toujours non publie au 2026-05-31. |
| P2-A-1(S71) worker-pump iroh-docs hang Windows natif | 2/3 (nouveau S71 → reconduit S72) | E2E worker-pump = CI Linux only (`feedback_wsl_before_push`). Dependance sequentielle : la root-cause iroh-docs pump Windows touche le worker, hors theme S72 (Factory routing). Candidat investigation S73+ ou exemption formelle CI-Linux-only. |
| P2-AUDIT-2 (iroh transitives pre-release) | herite | Pin iroh 0.98 (decision gelee). Re-evalue : pas d'upgrade 1.0 publie stable au 2026-05-31. |
| T-NN+2 (iframe Rust-wasm) | exemption | Depend d'un upstream wasm (PATTERNS §P34). Re-evalue : pas de changement upstream. |
| P3-F-1 (recap body Phase D verdict) | cosmetique | Verdicts reels traces dans les fichiers preflight S71 ; pas de trigger bloquant. |
| P3-OS-1 (operator_server.rs:519 OR duplique) | pre-existant S70 | Trigger = prochaine modif `handle_artifact_draft`. S72 ne touche pas ce handler (route chat, pas artifact). Reconduit. |
| 3×P2 + 3×P3 Phase C S71 / 3×P2 + 1×P3 Phase D S71 | documentes (rigor signal) | Documentes verification.md S71 §5 ; non bloquants. Reconduits si non absorbes. |

### Attention 3/3 S73

| Item | Reports a S72 | Alerte PO |
|---|---|---|
| P2-A-1(S71) worker-pump Windows | 2/3 | Passera **3/3 MANDATORY a S73** — devra entrer dans le plan S73 (root-cause iroh-docs pump Windows OU exemption formelle CI-Linux-only ecrite), pas reporte. |

### ROADMAP_COMMITMENTS (Regle 3 — conditions evaluees)

| LT | Condition | Etat 2026-05-31 |
|---|---|---|
| LT-2 Radicle | tag v1.0 **pousse** vers origin + GitHub Release | **PENDING** — tag v1.0 pose localement mais PAS pousse (22 ahead, rien pousse). Condition NON remplie → reste latent. |
| LT-5 redundancy persistence | 1er deploiement multi-worker OU v1.0 go-live | Latent (post-v1.0, reclass S26). Non declenche. |
| LT-7 self-hosted build worker quorum E2E | pre-v1.0 (Tier 1+2 DONE) | Tier 3 worker quorum cross-machine → **S75** (roadmap v5). `execute_build` dormant conserve (D8 S71, consommateur nomme S75). Non declenche S72. |
| LT-1/LT-3/LT-4/LT-6 | divers | LT-6 resolved S32 ; LT-1 reclass pre-v1.0 (S50, DONE) ; LT-3/LT-4 latent post-v1.0. Aucun declenche S72. |

---

## §7 Scope cuts (exhaustif)

Ce que S72 ne fera PAS, et pour quel sprint c'est garde. Chaque item
re-evalue contre le code actuel (G9, §6.2) — aucun n'est un gap petit
inclus a tort.

| # | Item | Sprint cible | Rationale (factuel) |
|---|---|---|---|
| 1 | Onboarding/packaging atelier (launcher conscient de Factory, doc install operateur, PO-4) | S74 | **NOTE : l'UX intentions complete est IN-SCOPE S72** (Phase E, arbitrage PO) — ce qui reste S74 est le **packaging produit** (launcher, doc install, onboarding operateur). S72 livre l'ecran de selection d'execution fonctionnel ; S74 livre l'experience d'installation. |
| 2 | Pont feed-distant → reindex FTS5 a chaud | S73 | Recherche reseau cablee = sprint suivant (roadmap). S72 ne touche pas `feed_sync.rs`. |
| 3 | Enrichissement `SearchResult` (repo_url+commit+archive_hash+provenance) | S73 | Idem — sans enrichissement un hit ne peut pas fork ; c'est le prerequis S74, pose en S73. |
| 4 | Barre de recherche shell cablee `GET /api/daemon/search` | S73 | Recherche reseau, hors quick win routing. |
| 5 | Decision SearchManifest (recherche opt-in propagee) | S73 (selon audit S72) | Decision posee apres S72. |
| 6 | Commandes `sbfb-factory search/open/fork` (Factory tire du reseau) | S74 | Atelier fork — Factory apprend a tirer. Hors routing. |
| 7 | Notion de projet cible distinct du repo nexus (`process::repo_root`) | S74 | `repo_root` pointe toujours nexus (G17). Le NetworkProvider S72 soumet une tache, il ne fork pas un projet. |
| 8 | Templates etendus (react, pyodide) | S74 | Atelier, hors routing. |
| 9 | GPU partage volontaire prouve cross-machine (consent 4 niveaux, caps, panneau « offrir ma puissance ») | S75 | S72 ROUTE vers le reseau (submit→poll) ; la PREUVE cross-machine + le GPU partage = S75. Le NetworkProvider S72 est cross-PROCESS local (daemon loopback), pas cross-GPU. |
| 10 | Quorum redundancy>1 prouve cross-MACHINE reel (B-3 etendu) | S75 | S72 soumet une tache ; le quorum cross-machine reel = S75. |
| 11 | Sharding pipeline « gros modele » (Petals/Parallax, iroh QUIC) | S76 STRETCH | Jamais avant preuve S75. |
| 12 | Streaming token-par-token depuis un worker reseau distant | jamais (PO-14) | WAN 1-3 tok/s → batch/async. Le NetworkProvider emet un seul Done (D3). Decision PO gelee. |
| 13 | logprobs/watermark verification | V2 compute (post-S75) | Greedy seed-fixe seul (PO-11, S71). |
| 14 | Dashboard contributeur kudos per-task | S75 | Hors routing. |
| 15 | Extraction d'un crate `ollama-client` partage worker/Factory | CADUC | L'arbitrage PO D2 (ollama-rs 0.3.4 partout) rend ce scope cut **caduc** : worker et Factory utilisent maintenant la **meme lib tierce `ollama-rs`** directement (pas de client maison duplique). Pas d'extraction a prevoir — la lib EST le crate partage. |
| 16 | Routage provider/model complet multi-cloud (OpenAI, Gemini, etc.) | hors roadmap | SBFB route 3 targets fermes (Claude/Ollama/reseau). Pas un proxy multi-cloud generaliste (ce serait `liter-llm`/`rig` — hors mission SBFB). |

---

## §8 Tracabilite scope

Mapping de chaque item « What's NOT » du sprint precedent (S71 §8 scope
cuts) sur son traitement S72.

| Item S71 « What's NOT » (§8) | Sprint + Phase S72 |
|---|---|
| #1 ProviderRouter multi-LLM (trait + 3 providers) | **S72 Phases C+D** (c'est le theme S72 — D1/D2/D3) |
| #2 Chat Factory cable sur routage de taches reseau | **S72 Phase D backend + Phase E front** (NetworkProvider + cablage provider + UX intentions, D3/D4) |
| #3 Pont feed-distant → reindex FTS5 | Reconduit **S73** (§7 #2) |
| #4 Enrichissement `SearchResult` | Reconduit **S73** (§7 #3) |
| #5 Barre recherche shell cablee | Reconduit **S73** (§7 #4) |
| #6 Decision SearchManifest | Reconduit **S73** (§7 #5) |
| #7 `sbfb-factory search/open/fork` | Reconduit **S74** (§7 #6) |
| #8 Notion projet cible distinct nexus | Reconduit **S74** (§7 #7) |
| #9 Templates etendus (react, pyodide) | Reconduit **S74** (§7 #8) |
| #10 GPU partage cross-machine | Reconduit **S75** (§7 #9) |
| #11 Quorum redundancy>1 cross-MACHINE | Reconduit **S75** (§7 #10) |
| #12 Sharding pipeline | Reconduit **S76 STRETCH** (§7 #11) |
| #13 logprobs/watermark | Reconduit **V2 compute** (§7 #13) |
| #14 Dashboard kudos per-task | Reconduit **S75** (§7 #14) |
| #15 @dev index tree-sitter | Reconduit **post-Gate 1** (hors arc 3.5 routing ; non re-liste §7 car hors theme — supprime de la liste active S72, restera dans roadmap v4 backlog) |
| #16 Packaging produit Factory | Reconduit **S74** (§7 #1 onboarding/install — l'UX intentions est S72 Phase E, le packaging reste S74) |

---

## §9 Risk register (R1..R7)

| ID | Risque | Likelihood | Impact | Mitigation |
|---|---|---|---|---|
| R1 | Le bras `Claude` de `ExecutionTarget` change subtilement le comportement S71 (regression du gate SSE / idle-timeout) | Moyen | Eleve | D1 : `spawn_claude_stream` reste le corps du bras Claude, INCHANGE. Un test prouve l'equivalence (gate SENSITIVE_ACTIONS appliqué avant dispatch — D4). Phase C critere binaire. |
| R2 | OllamaProvider non testable sans Ollama reel (flaky / skip) | Moyen | Moyen | D2 : test mapping `generate_stream` → StreamChunk sans Ollama reel (stub/feature gate) ; un E2E reel gate sur disponibilite Ollama (skip propre si absent, comme B-3 S71). |
| R3 | NetworkProvider submit→poll exige le token daemon S16 que l'Operator n'a pas | Moyen | Moyen | D3 : verifier au preflight Phase D si `/api/v1/tasks/submit` exige `X-SBFB-Token` ; si oui, l'Operator reutilise le token daemon (loopback, meme pattern S71 C). Documenter. |
| R4 | Scope creep « atelier complet » — front UX complet IN-SCOPE (override PO) | Eleve | Eleve | **Accepte par le PO** (arbitrage Checkpoint §11 : UX intentions complete in-scope S72). Mitige en **phasant le front separement (Phase E)** pour que le backend (Phase D) lande INDEPENDAMMENT — si le front deborde, le routing backend reste livre. Les vrais scope cuts restent stricts (§7 : recherche S73, fork S74, packaging/onboarding S74, GPU/sharding S75-76). |
| R5 | UX intentions incompatible avec dispatch reseau async (utilisateur attend du live, voit un seul Done apres delai) | Moyen | Moyen | D3 + PO-14 : l'UX montre un etat « en cours sur le reseau » (progress pendant le poll), JAMAIS de fausse promesse token-par-token. L'intention front est explicite sur le mode batch reseau. |
| R6 | P2-F-3 (3/3) s'avere deja resolu OU plus gros que prevu | Faible | Faible | Phase B verifie l'etat reel d'abord (§G9). Si resolu → documenter clos. Si gros → c'est un check mecanique borne (~test+lint), pas une refonte. Exit binaire. |
| R7 | Bump `ollama-rs` 0.2.6→0.3.4 casse le determinisme greedy-seed du quorum worker (B-2, stabilise S71) | Moyen | Eleve | D2 : migration mecanique (rename `GenerationOptions`→`ModelOptions`, API seed/options survit — context7 0.3.4 verifie). **Migration faite TOT (Phase C bloc 1, avant l'Ollama provider).** Les 4 tests quorum S71 (`verifiable_task_uses_greedy_seed`, `two_honest_workers_same_hash`, `quorum_accepts_deterministic_redundancy`, `quorum_rejects_nondeterministic_divergence`) = **critere binaire** de la phase migration. Preflight Phase C re-verifie l'API seed 0.3.4 (S1b) avant la 1ere ligne. Si l'API a change de maniere non anticipee → DESIGN-CONFLICT remonte. |

---

## §10 Audit gate pattern — rappel

- **Phase 0** : DEJA JOUE (§3) — `sprint71_audit_findings.md` (`636b9de`),
  verdict PASS (0 P0, 0 P1, 1 P2 routé Phase A, 2 P3). Aucun fix requis.
- **Phase de sortie (F)** : produit les deux livrables obligatoires dans
  un commit `docs(sprint72)` : `sprint72_verification.md` (self-report
  fail-fast rempli) + `sprint73_audit_plan.md` (feuille de route session
  fraiche S73). Sans ces deux fichiers, le sprint ne ferme pas (§3.3).
- Phase F met a jour `docs/rust/PATTERNS.md` (§P55 3 axes si pas Phase D)
  et `docs/shell/PATTERNS.md` si nouveaux patterns/tech debt.

---

## §11 Checkpoint de validation

5 questions (1 par D-choice) pour arbitrage user AVANT le plan detaille.
Dernier moment pour pivoter sans cout.

1. **D1** — `ExecutionTarget` enum-dispatch (3 variantes fermees boxees en
   `Pin<Box<dyn Stream<StreamChunk>>>`) plutot qu'un trait `async-trait`+
   `Box<dyn>` : OK pour figer enum-dispatch (eviter le double-box, valide
   par rig/enum_dispatch) ? Ou preferes-tu un trait extensible des
   maintenant (au prix de la complexite) ?
2. **D2 [ARBITRE PO]** — `ollama-rs 0.3.4` **partout** (Factory dep +
   bump worker 0.2.6→0.3.4) — decision PO actee. Le risque clé : la
   migration touche le code quorum greedy-seed S71 (rename
   `GenerationOptions`→`ModelOptions`, API seed survit). Confirme-tu que
   les **4 tests quorum S71 doivent rester verts** comme critere binaire
   de la phase migration (R7), et que la migration vient TOT (Phase C
   bloc 1) ?
3. **D3** — NetworkProvider **submit→poll → un seul `Done`** (PO-14,
   jamais token-par-token WAN), avec un etat « en cours sur le reseau »
   pendant le poll : OK pour assumer le mode async batch ? (Le live WAN
   est exclu par decision PO.)
4. **D4 [ARBITRE PO]** — Cabler `provider` symetriquement a `model`
   (persiste dans `ChatSession`, lu au stream GET), gate SENSITIVE_ACTIONS
   applique AVANT dispatch quel que soit le target. L'**UX intentions
   COMPLETE est in-scope S72** (Phase E front) — decision PO actee. Le
   backend (Phase D) lande independamment AVANT le front (Phase E) : OK
   pour ce phasage (le scope creep front est assume PO, mitige par phase
   separee) ?
5. **D5** — Nommer l'enum `ExecutionTarget` (pas `Provider`) pour ancrer
   la distinction des **3 axes** (prompt-adaptation `Provider` process.rs
   / runtime `LlmBackend` worker / `ExecutionTarget` Factory chat),
   documente §P55 : OK pour ne PAS unifier (§P53 a deja tranche 2 axes
   orthogonaux) ?
