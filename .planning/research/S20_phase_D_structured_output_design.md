# Sprint 20 Phase D — Structured output design (Option 2 étendue)

**Ecrit** : 2026-04-18 (session fraiche, post-Phase C `16b94ba` +
chore(planning) review archive `2e045f1`).
**Tip master d'entree** : `2e045f1`.
**Statut** : figé avant implementation. Toute deviation au plan
du kickoff §D4 documentee ci-dessous + carry P2-D1 Phase F pour
update plan §7 +kickoff §D4.

---

## 1. Contexte

### 1.1 Drift plan §Phase D vs code réel

Le plan `sprint20_plan.md §7` + kickoff §D4 supposent que le worker
embed `llama-cpp-2` (Rust binding llama.cpp) avec feature llguidance
activee au build llama.cpp (`-DLLAMA_LLGUIDANCE=ON` cmake). Réalité
observée session fraiche 2026-04-18 :

```
crates/nexus-worker-core/Cargo.toml
  ollama-rs = { workspace = true }    # version pin workspace "0.2"
crates/nexus-worker-core/src/ollama.rs
  (unique place qui parle LLM, tout passe par HTTP vers Ollama daemon)
```

Le workspace **ne dépend pas de `llama-cpp-2` ni de `llguidance`**.
L'inference LLM passe par **HTTP vers Ollama daemon local** (port
11434 par défaut). Ollama lui-meme embed llama.cpp mais nous y
accédons comme un service externe.

Commentaire `ollama.rs:6-10` confirme le pattern en l'état :

> This module is the *only* place in `nexus-worker-core` that
> depends on `ollama-rs`. Everything else in the engine talks to
> the `OllamaClient` trait so the implementation can be swapped
> in tests or **replaced by a different LLM backend (llama.cpp RPC,
> a remote Ollama over SSH, ...) without cascading changes.**

L'abstraction `trait OllamaClient` existe deja comme **hook
d'extension** prévu pour exactement ce pivot. Phase D ne fait pas
une refonte mais un **renommage + extension** : `trait OllamaClient`
devient `trait LlmBackend`, et on ajoute une deuxième implementation
`LlamaCppBackend` behind feature-flag.

### 1.2 Décision FlowUP 2026-04-18

Confronté aux 3 options (Ollama format seul / llama.cpp direct /
scope-cut), l'utilisateur a validé **Option 2 étendue** :
`trait LlmBackend` + deux impls (`OllamaBackend` + `LlamaCppBackend`)
avec default production = llama.cpp.

Rationale : alignement long-terme S22/S23/S26 (tool-calling sandbox,
ephemeral workers + VRAM wipe, PQC task signing) qui exigent
contrôle process LLM direct, impossible via Ollama HTTP daemon.

---

## 2. Architecture retenue

### 2.1 Trait `LlmBackend`

Rename du trait existant `OllamaClient` → `LlmBackend`, zone de
responsabilité identique :

```rust
#[async_trait]
pub trait LlmBackend: Send + Sync {
    /// Probe daemon/runtime reachability + list installed models.
    async fn healthcheck(&self) -> HealthCheck;

    /// Run text-generation request with optional JSON Schema
    /// enforcement. Implementations that cannot enforce schema
    /// natively must run a **defensive validator** after generate
    /// and return `LlmBackendError::SchemaViolation` on mismatch.
    async fn generate(&self, params: GenerateParams) -> LlmBackendResult<GenerateResponse>;
}
```

`GenerateParams` gagne un champ :

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GenerateParams {
    pub model: String,
    pub prompt: String,
    pub system: Option<String>,
    pub temperature: Option<f32>,
    /// JSON Schema (draft-07) that the response MUST satisfy.
    /// Backends enforce at sample-level (llama.cpp via llguidance)
    /// or via native `format` param (Ollama v0.5+), and all
    /// backends run a defensive `serde_json::from_str` validator
    /// after generate regardless (belt-and-suspenders).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub schema: Option<serde_json::Value>,
}
```

### 2.2 `OllamaBackend` (rename `OllamaHttpClient`)

- Wrap `ollama-rs 0.2.6` `Ollama::generate(GenerationRequest)`
- `params.schema` → `GenerationRequest::format(FormatType::StructuredJson(JsonStructure))`
- `JsonStructure::new::<T: JsonSchema>()` requires `schemars` derive
  → on passe par un shim `json_structure_from_value(&serde_json::Value)`
  que l'on implémente via `serde_json::from_value::<RootSchema>(schema)`
  puis construction manuelle. Source : ollama-rs 0.2.6
  `parameters.rs:33-48`, `JsonStructure { schema: RootSchema }`,
  field privé → contribution upstream hors-scope S20, on contourne
  en construisant `RootSchema` directement via `serde_json`.
- Healthcheck/retry inchangés (code déjà éprouvé S4+).

### 2.3 `LlamaCppBackend` (nouveau, feature-gated)

Behind `#[cfg(feature = "llama_cpp")]`. Pipeline :

```
GGUF model file path (~/.nexus-grid/models/<tag>.gguf)
   │ LlamaBackend::init() (global, once)
   │ LlamaModel::load_from_file(&backend, path, &model_params)
   │ model.new_context(&backend, ctx_params)
   ▼
LlamaContext (per-request OR pooled)
   │ Tokenize prompt → LlamaBatch
   │ ctx.decode(batch)
   │ Loop until EOS or max_tokens :
   │   logits = ctx.get_logits()
   │   if schema_matcher present :
   │     mask = llguidance::Matcher::compute_mask()
   │     apply mask to logits (softmax over allowed only)
   │   token = sampler.sample(logits)
   │   schema_matcher.consume_token(token)
   │   ff_tokens = matcher.compute_ff_tokens()
   │   → batch.add(token, pos+1, seq_id, true)
   │   ctx.decode(batch)
   ▼
Decoded text string
   │ Defensive validator : serde_json::from_str::<schema>(&text)
   ▼
GenerateResponse
```

Crates utilisées :

- **`llama-cpp-2 = "0.1.143"`** (utilityai/llama-cpp-rs) — Rust
  binding officiel llama.cpp, High reputation. Expose
  `LlamaBackend`, `LlamaModel`, `LlamaContext`, `LlamaBatch`,
  `LlamaSampler`. Pas de feature `llguidance` Cargo — le flag cmake
  `-DLLAMA_LLGUIDANCE=ON` **n'est pas nécessaire** car on bridge
  llguidance côté Rust (pattern custom sampler, cf. ci-dessous).
- **`llguidance = "1.7"`** (guidance-ai/llguidance) — Microsoft,
  High reputation. API `ParserFactory::new_simple(&tok_env)` +
  `TopLevelGrammar::from_json_schema(serde_json::Value)` +
  `Matcher::new(parser)` + `matcher.compute_mask()` +
  `matcher.consume_token(...)` + `matcher.compute_ff_tokens()`.
  Source : context7 `/guidance-ai/llguidance` 2026-04-18.

Custom sampler intégrant llguidance :

```rust
use llama_cpp_2::sampling::LlamaSampler;
use llguidance::{api::TopLevelGrammar, toktrie::ApproximateTokEnv,
                 Matcher, ParserFactory};

struct LlguidanceSampler {
    matcher: Matcher,
    inner: LlamaSampler,  // temp + top-p chain
}

impl LlguidanceSampler {
    fn sample(&mut self, logits: &mut [f32]) -> LlamaToken {
        let mask = self.matcher.compute_mask().unwrap();
        for (tok_id, logit) in logits.iter_mut().enumerate() {
            if !mask.is_allowed(tok_id as u32) {
                *logit = f32::NEG_INFINITY;  // forbid
            }
        }
        let token = self.inner.sample_from_logits(logits);
        self.matcher.consume_token(token).unwrap();
        token
    }
}
```

`ApproximateTokEnv::single_byte_env()` suffit pour la premiere
implementation ; upgrade vers un tok_env fidèle au tokenizer GGUF
(plus performant sur 128k tokenizer) en optimisation S21+.

### 2.4 Feature matrix Cargo

```toml
[features]
default = ["ollama"]            # dev ergonomics : no cmake deps
ollama = []                     # always available
llama_cpp = ["dep:llama-cpp-2", "dep:llguidance"]
llama_cpp_cuda = ["llama_cpp", "llama-cpp-2/cuda"]
llama_cpp_metal = ["llama_cpp", "llama-cpp-2/metal"]
llama_cpp_vulkan = ["llama_cpp", "llama-cpp-2/vulkan"]

[dependencies]
ollama-rs = { workspace = true }
llama-cpp-2 = { version = "0.1.143", optional = true }
llguidance = { version = "1.7", optional = true }
schemars = { workspace = true }   # JSON Schema derive for Ollama
serde_json = { workspace = true }  # schema Value parse
```

**Tension résolue** : FlowUP a validé "default production = llama.cpp"
(kickoff §D1 + decision 2026-04-18). Réalité build : cmake +
potentiel CUDA toolkit + potentiel NASM Windows = friction dev.

Résolution :
- **Cargo `default = ["ollama"]`** : pas de friction build sur
  machine fresh / CI Windows minimal. Dev peut `cargo build`
  direct sans cmake installe.
- **Worker runtime config `[llm] backend = "llama_cpp"`** est la
  valeur **recommandée production**, documentée dans
  `docs/shell/PATTERNS.md §P30 operator runbook`. Si la feature
  `llama_cpp` n'est pas compilée mais le worker.toml pointe
  `llama_cpp`, erreur loud au startup :
  `LlmBackendError::UnsupportedBackend { requested: "llama_cpp",
  compiled_features: ["ollama"], hint: "rebuild with --features
  llama_cpp" }`.
- **Config `Config::default()` Rust = `backend = "ollama"`** : si
  `worker.toml` n'a pas de section `[llm]`, on tombe sur ollama
  (le backend toujours compilé) sans surprise.

Cela respecte "default production = llama.cpp" via la documentation
+ l'erreur runtime explicite, tout en évitant que le build Windows
vanille plante sur cmake manquant.

### 2.5 Config `worker.toml`

```toml
[llm]
backend = "llama_cpp"  # "ollama" | "llama_cpp"

[llm.ollama]
endpoint = "http://localhost:11434"
timeout_secs = 300

[llm.llama_cpp]
model_path = "~/.nexus-grid/models/qwen2.5-7b-instruct-q4_k_m.gguf"
n_ctx = 4096
n_gpu_layers = 1000   # -1 = all, 0 = CPU-only
n_threads = 8
```

`Config` struct migre :

```rust
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    // ...
    pub llm: LlmConfig,  // NEW — remplace l'ancien `ollama: Ollama`
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LlmConfig {
    #[serde(default = "default_backend")]
    pub backend: BackendKind,
    pub ollama: OllamaConfig,
    #[serde(default)]
    pub llama_cpp: LlamaCppConfig,
}

fn default_backend() -> BackendKind { BackendKind::Ollama }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BackendKind {
    Ollama,
    LlamaCpp,
}
```

**Breaking change** wire config `worker.toml` : l'ancienne section
`[ollama]` devient `[llm.ollama]`. Pre-launch protocol policy CLAUDE.md
autorise : `*_VERSION = 1` jusqu'au tag v1.0, pas de tolerant
decoder multi-version. Les fichiers `worker.toml` existants sur
machine dev seront regenerables via `nexus-worker init --force`.

---

## 3. Versions pinnees + CVE cross-check (2026-04-18)

| Crate | Version | Source | RustSec advisory-db | Notes |
|---|---|---|---|---|
| `ollama-rs` | `0.2.6` (workspace) | [crates.io](https://crates.io/crates/ollama-rs/0.2.6) | 0 findings | Workspace pin déjà en place. `FormatType::StructuredJson(JsonStructure)` serialize directement `schema.serialize(serializer)`. Source : `~/.cargo/registry/src/.../ollama-rs-0.2.6/src/generation/parameters.rs:14-24`. |
| `llama-cpp-2` | `0.1.143` | [docs.rs](https://docs.rs/crate/llama-cpp-2/0.1.143) | 0 findings | Context7 `/utilityai/llama-cpp-rs` High reputation 2026-04-18, 42 snippets. `LlamaSampler::grammar` GBNF support natif, mais on bridge llguidance cote Rust pour 50µs/token + JSON Schema/Lark. |
| `llguidance` | `1.7` | [crates.io](https://crates.io/crates/llguidance), Context7 `/guidance-ai/llguidance` | 0 findings | Microsoft. `ParserFactory::new_simple(&tok_env)` + `TopLevelGrammar::from_json_schema(Value)` + `Matcher::compute_mask() / consume_token()`. ~50µs mask computation 128k tokenizer. |
| `schemars` | `0.8.21` | Transitif via ollama-rs 0.2.6 | 0 findings | Workspace-pin direct requis pour derive `JsonSchema` sur `TaskResponse`. v1.0 existe mais ollama-rs 0.2.6 pin 0.8.21 → pin matching. |

**CVE cross-check sources consultees** :
- RustSec advisory-db (cargo-audit workspace — reprise baseline S18)
- WebSearch "<crate> cve 2026" sur les 3 nouveaux
- Context7 query docs direct (Last-Validated dates via metadata)

**Aucune zone rouge nouvelle ajoutee au R-register** (cf. memory
`nexus_grid_pivot.md`). `R-wasmtime-cve` + `R-iroh-audit` +
`R-libcrux-hax` + `R-pyodide-escape` inchangées. llama.cpp upstream
n'a pas de CVE Critical open au 2026-04-18 (verifie ollama issue
tracker + GitHub advisories llama.cpp repo).

**Frontmatter G2 trigger** : `docs/security/HARDENING_ROADMAP.md`
`last_validated` bump 2026-04-18 dans la Phase F wrap-up (cohérent
avec le pattern S19).

---

## 4. Alternatives rejetees

### 4.1 Ollama `format` param seul (Option 1 de la conversation 2026-04-18)

- Livre le goal fonctionnel (schema enforce au worker)
- **Mais ne débloque rien pour S22 (tool-calling sandbox) / S23
  (ephemeral workers + VRAM wipe) / S26 (PQC task signing)** — tous
  exigent contrôle process LLM direct
- Ollama HTTP = abstraction leak héritée depuis Sprint 0
- Perf : ~200µs/token GBNF interne Ollama vs 50µs llguidance
- **Rejeté** : fix court-terme qui ne tient pas le "ultra long terme"
  feedback_approach

### 4.2 llama.cpp direct SEUL (remplacement Ollama complet)

- Plus gros scope Phase D, friction build dev (cmake + CUDA + NASM
  Windows)
- Casse la baseline "any dev can `cargo build`" (CI notamment)
- Perd le chemin Ollama comme fallback sain pour dev quick
- **Rejeté** : trop radical, casse trop de workflows sans gain
  additionnel vs dual-backend trait

### 4.3 Scope-cut Phase D → Sprint 21 (Option 3 de la conversation)

- Repousse l'item HARDENING_ROADMAP §3 S20 item 4 sans research
  complementaire
- Bloque la chaine de défense T5 worker-compromise (signature chain
  break sur garbled JSON)
- **Rejeté** : pas de raison technique de différer, le travail est
  faisable S20 avec scope étendu raisonnable

### 4.4 XGrammar (mlc-ai)

- Performance comparable llguidance (~50µs/token)
- **Pas supporté par llama-cpp-2 Rust binding** (integration vLLM/
  SGLang only, confirmé arxiv 2501.10868 + MLC blog 2025-2026)
- **Rejeté** : incompatible avec stack Rust native
- Source : [MLC blog](https://blog.mlc.ai/2024/11/22/achieving-efficient-flexible-portable-structured-generation-with-xgrammar) + [sglang PR #3298](https://github.com/sgl-project/sglang/pull/3298)

### 4.5 Outlines (Python)

- Overhead IPC Python → Rust (serialize grammar + token cross-language)
- Brise Option G workspace strict (Python vs Rust separation)
- **Rejeté** : non-aligné architecture

### 4.6 JSON Mode / tool_use OpenAI-compat

- Pas de standard unifié entre backends (Ollama, llama.cpp server,
  vLLM)
- Custom compat layer coord-side = travail supplémentaire
  non-amortissable
- **Rejeté** : pas générique, pas performant

### 4.7 GBNF natif llama.cpp (sans llguidance)

- `LlamaSampler::grammar(&model, grammar_str, "root")` supporte
  GBNF via `llama-cpp-2` directement (context7 confirme, exemple
  fonctionnel)
- Performance ~200µs/token, JSON-only (pas de Lark/regex/CFG)
- **Rejeté** : moins performant, moins expressif. Si on embed
  llama.cpp + custom sampler, autant prendre llguidance 50µs.
- Sauvegardé comme **fallback runtime** : si `llguidance::Matcher`
  échoue à builder la grammar (schema malformed), on retombe sur
  `LlamaSampler::grammar` avec GBNF JSON generique (pas ideal mais
  pas de panic).

### 4.8 llguidance via build flag llama.cpp `-DLLAMA_LLGUIDANCE=ON`

- Alternative officielle documentée dans llguidance README
- Requiert rebuild llama.cpp avec flag cmake + lien contre `libllguidance.so/.a`
- Cost op' : operator doit connaître le flag + le documenter
- **Rejeté** : bridge Rust direct (llguidance crate) = installation
  automatique via cargo, pas de flag cmake manuel. Meme resultat
  fonctionnel, zero friction operator.

---

## 5. Source-of-truth schema `TaskResponse`

### 5.1 Rust struct comme source-of-truth

```rust
// crates/nexus-core-rs/src/schemas/task_response.rs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct TaskResponse {
    /// Version canonical = 1 (pre-launch protocol policy).
    pub version: u8,
    /// Domain tag for serialize bytes (avoid schema confusion).
    pub domain: String,  // must == "TASK_RESPONSE_V1"
    /// The actual LLM output content.
    pub content: String,
    /// Optional reasoning trace when the model was prompted to
    /// emit one (CoT / structured reasoning).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<String>,
    /// Tool calls the worker wants the coordinator to execute
    /// (S22+ tool-calling sandbox gates these). Empty list ok
    /// at S20.
    #[serde(default)]
    pub tool_calls: Vec<ToolCall>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}
```

### 5.2 Schema générée automatiquement

`schemars::schema_for!(TaskResponse)` produit le JSON Schema
draft-07 au runtime. On en extrait une `serde_json::Value` pour :
- Passer à `ollama_rs::generation::parameters::JsonStructure` (via
  shim `json_structure_from_value`)
- Passer à `llguidance::TopLevelGrammar::from_json_schema(value)`

### 5.3 Fichier `task_response.schema.json` = snapshot test

Le fichier `crates/nexus-core-rs/src/schemas/task_response.schema.json`
n'est **pas** source-of-truth — c'est un **snapshot genere** depuis
la struct Rust. Test dédié `test_schema_snapshot_matches_struct`
verifie que le snapshot matche ce que `schemars` produit, protégeant
contre un drift silencieux (Rust struct modifiée sans regenerer le
snapshot).

---

## 6. Tests plan (cible +20 à +25)

### 6.1 Schema + trait (core, ~8 tests)

1. `task_response_schema_parses_as_valid_json_draft_07`
2. `task_response_schema_snapshot_matches_schemars_generate`
3. `task_response_serde_roundtrip_preserves_all_fields`
4. `task_response_version_must_be_1`
5. `task_response_domain_tag_rejects_wrong_value`
6. `llm_backend_trait_stub_healthcheck_returns_ready`
7. `llm_backend_trait_stub_generate_deterministic`
8. `generate_params_schema_field_serde_roundtrip`

### 6.2 OllamaBackend (~6 tests, existants conservés + 2 nouveaux)

9-14. Tests existants ollama.rs (`client_from_config`,
`looks_like_connection_refused`, `healthcheck_is_ready_helper`,
`retry_with_backoff_*`, `live_healthcheck`, `generate_params_builder`)
— conservés.
15. `ollama_backend_wires_format_json_structure_when_schema_present`
16. `ollama_backend_defensive_validator_rejects_schema_violation`

### 6.3 LlamaCppBackend (~8 tests, feature-gated `llama_cpp`)

Tests `#[cfg(feature = "llama_cpp")]` :

17. `llama_cpp_config_from_toml_parses`
18. `llama_cpp_config_validates_model_path_expanded`
19. `llguidance_matcher_builds_from_task_response_schema`
20. `llguidance_compute_mask_returns_allowed_bitmask`
21. `llguidance_consume_token_advances_state`
22. `llguidance_sampler_forbids_masked_tokens`
23. `llama_cpp_backend_healthcheck_returns_error_without_model` (pas
    de GGUF sur CI ; verif loud error path)
24. `llama_cpp_backend_stub_integration_via_matcher_only` (mock
    LlamaContext, ne charge pas de GGUF réel)

**Note** : les tests qui chargent un GGUF réel (500MB+) sont
`#[ignore]` + `cargo test --features llama_cpp,integration_llama_cpp`
pour les dev avec GGUF présent. CI reste sur feature `ollama`
default, aucun GGUF requis.

### 6.4 Config select + backend factory (~3 tests)

25. `config_llm_backend_defaults_to_ollama_when_section_missing`
26. `config_llm_backend_llama_cpp_rejected_without_feature`
    (`#[cfg(not(feature = "llama_cpp"))]` path)
27. `llm_backend_factory_returns_correct_impl_per_config`

### 6.5 Bench criterion (~2 tests)

28. `bench_llguidance_compute_mask_task_response_schema` : <100µs
    sur CPU moderne 2026 (SBFB CI)
29. `bench_ollama_format_param_serialize_task_response_schema` :
    <50µs

**Total projection : +25-29 tests.** Delta Rust workspace 598 →
~625. Pas d'estimation LOC (feedback_approach).

---

## 7. Threat model alignment — S22/S23/S26 unlocks

### 7.1 S22 tool-calling sandbox

`ToolCall` structure déjà prévue dans `TaskResponse.tool_calls:
Vec<ToolCall>`. S22 ajoutera :
- Allow-list au niveau `ToolCall.name`
- Wasmtime sandbox execution des tool functions
- **Prerequis** : worker doit pouvoir **intercepter** les tool calls
  avant `TaskResponse` signature. Avec Ollama HTTP c'était indirect
  (stringly-typed texte), avec `LlamaCppBackend` le sampler peut
  emit un event "tool_call sampled" au moment où le matcher accepte
  un token de `tool_calls` field. Hook disponible via
  `Matcher::consume_token()` inspection.

### 7.2 S23 ephemeral workers + VRAM wipe

- Ollama daemon : process séparé, ne peut pas être wiped de manière
  atomic avec le worker SBFB (Ollama a un cache model KV persistant
  entre requetes).
- `LlamaCppBackend` : le `LlamaContext` appartient au processus
  worker ; on peut `drop(context)` + `memset(gpu_buffer, 0)` à la
  frontière de task + nouvelle model load = wipe propre VRAM.
- Signal CUDA API `cudaMemset` accessible via `llama-cpp-2` bindings
  (feature `cuda`).

### 7.3 S26 PQC task signing (ML-DSA)

- TaskResponse signature doit lier contenu LLM + worker identity +
  timestamp. Avec Ollama la signature se fait **après** le round-
  trip HTTP (mauvais : Ollama pourrait retourner un contenu
  différent de ce qu'il a produit).
- `LlamaCppBackend` : signature peut s'insérer **inline dans le
  sampler** — tokens accumulés + HMAC chaîné → Ed25519/ML-DSA à EOS.
  Chain-of-custody preservée.

---

## 8. Warnings + caveats documentés

### 8.1 « Grammar ≠ prompt injection defense » (prominent §P30)

Le `LlamaCppBackend` + `llguidance` enforce le **format** de sortie
(JSON Schema), **pas le contenu**. Un prompt injection user-side
peut toujours faire produire au modèle du JSON valide structurel
mais sémantiquement malveillant. Ex : `TaskResponse { content:
"<exfiltrated secret>", tool_calls: [...] }` — valid schema, mauvais
content.

Defense anti prompt injection = responsabilité S22+ tool-calling
sandbox (allow-list + wasmtime jail) + S21 client-side redaction
SDK. Grammar = defense complémentaire (signature chain integrity),
pas substituable.

Documentation prominent en haut `docs/rust/PATTERNS.md §P30` :

> ⚠️ **STRUCTURED OUTPUT GRAMMAR IS NOT A DEFENSE AGAINST PROMPT
> INJECTION.** The grammar enforces *format* (JSON Schema), not
> *content*. A successful prompt injection can still produce
> schema-valid responses with malicious payload. Defense against
> prompt injection belongs to S22+ (tool-calling sandbox +
> wasmtime jail) and S21+ (client-side redaction SDK). Cf.
> `docs/security/HARDENING_ROADMAP.md audited_findings 2026-04-18
> "S21 grammar ≠ prompt injection defense"`.

### 8.2 Build chain operator runbook

- cmake 3.26+ requis pour `llama-cpp-2` build (Windows : Visual
  Studio Build Tools 2022 + cmake)
- CUDA toolkit 12.6+ pour feature `llama_cpp_cuda` (Linux/Windows)
- macOS feature `llama_cpp_metal` : Xcode Command Line Tools
- Documentation complète : `docs/shell/PATTERNS.md §P30` Phase D.

### 8.3 Model path user responsability

`LlamaCppConfig.model_path` pointe un fichier GGUF que le user doit
télécharger manuellement (Ollama cache répartit auto, llama.cpp non).
Phase D ne couvre pas le download auto. Item **carry implicite pour
S21+** : `nexus-worker model pull <tag>` sous-commande qui mirror
la UX Ollama — hors scope S20.

### 8.4 Forward compat schema v2

Pre-launch policy = `TASK_RESPONSE_V1` only, pas de tolerant
decoder. Si un worker run un binaire ancien (S20 wire) et reçoit
un futur wire S25+ (v2), il rejette loud. Acceptable pre-launch
(aucun noeud tiers ne parle les protocoles).

---

## 9. Migration fichier `ollama.rs` → `llm/`

Structure cible :

```
crates/nexus-worker-core/src/llm/
├── mod.rs              # trait LlmBackend + GenerateParams + HealthCheck + erreurs
├── ollama.rs           # OllamaBackend (ex-OllamaHttpClient) + StubBackend
├── llama_cpp.rs        # LlamaCppBackend (feature-gated)
├── schema_bridge.rs    # JsonStructure from_value shim + llguidance Matcher builder
└── factory.rs          # build_backend(config: &LlmConfig) -> Box<dyn LlmBackend>
```

Le module `crates/nexus-worker-core/src/ollama.rs` actuel devient
`src/llm/ollama.rs`. Tests conservés. Alias `pub use llm::*` dans
`lib.rs` pour minimiser les changements en cascade.

**Imports en cascade à mettre a jour** (grep `use crate::ollama::`
ou `nexus_worker_core::ollama::`) :
- `engine/` modules
- `consent.rs` (probable)
- Tests d'intégration dans `tests/`
- `nexus-worker` binary crate

---

## 10. Working tree audit G5 anticipation

Categorisation attendue du commit feat :

| Fichier | Categorie |
|---|---|
| `Cargo.toml` (workspace + worker-core : deps + features) | PHASE |
| `crates/nexus-core-rs/src/schemas/mod.rs` (nouveau) | PHASE |
| `crates/nexus-core-rs/src/schemas/task_response.rs` (nouveau) | PHASE |
| `crates/nexus-core-rs/src/schemas/task_response.schema.json` (snapshot) | PHASE |
| `crates/nexus-core-rs/src/lib.rs` (exports) | PHASE |
| `crates/nexus-worker-core/src/llm/mod.rs` (nouveau, trait) | PHASE |
| `crates/nexus-worker-core/src/llm/ollama.rs` (depuis ollama.rs) | PHASE |
| `crates/nexus-worker-core/src/llm/llama_cpp.rs` (nouveau) | PHASE |
| `crates/nexus-worker-core/src/llm/schema_bridge.rs` (nouveau) | PHASE |
| `crates/nexus-worker-core/src/llm/factory.rs` (nouveau) | PHASE |
| `crates/nexus-worker-core/src/ollama.rs` (deleted) | PHASE |
| `crates/nexus-worker-core/src/lib.rs` (module layout) | PHASE |
| `crates/nexus-worker-core/src/config.rs` ([llm] section) | PHASE |
| `crates/nexus-worker-core/Cargo.toml` (features + deps) | PHASE |
| `crates/nexus-worker-core/benches/llm.rs` (nouveau) | PHASE |
| `docs/rust/PATTERNS.md` (§P30) | PHASE |
| `docs/shell/PATTERNS.md` (§P30 operator) | PHASE |
| `.planning/research/S20_phase_D_structured_output_design.md` | PHASE (design doc) |

CRAFT 0 · DEBT 0 · NOISE 0 · PHASE ~18.

---

## 11. Carry-over audit Phase F

Si nexus-phase-auditor détecte ces items pendant la review Phase D :

- **P2-D1 (plan drift)** : mettre à jour `sprint20_plan.md §7` +
  `sprint20_kickoff.md §4 D4` pour refléter le pivot vers
  dual-backend trait. Non-bloquant, docs-only Phase F.
- **P2-D2 (versions bump)** : `llguidance 0.7` mentionne dans plan
  → actual crate version 1.7. Update plan + kickoff + design_review
  Phase F.
- **P3-D1 (`nexus-worker model pull` cmd)** : carry S21+ (model
  download UX) pour symétrie avec Ollama auto-cache.
- **P3-D2 (`tok_env` fidèle tokenizer GGUF)** : optimisation perf
  S21+ pour exploitation 128k tokenizer optimale.

---

## 12. Décision finale pre-implementation

Architecture figée. Implementation suit le plan §2 ci-dessus.
Déviations seront documentées dans le commit body Phase D +
carry-over Phase F.

**Risk residuel** : `llama-cpp-2 = "0.1.143"` breaking changes API
entre publish et implementation. Mitigation : pin exact version +
Cargo.lock. Si context7 snippets d'il y a quelques semaines divergent
du code réel, iterate via `cargo check` + docs.rs source direct.

Co-authored-by the feedback_approach rule : "ultra long terme +
plus poussé + documente avant reflexion et code".
