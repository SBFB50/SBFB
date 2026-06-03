# Sprint 72 Phase C — Pivot Proposal (DESIGN-CONFLICT)

Date: 2026-06-03
HEAD: `08b6cb2`
Origine: ground-truth implementer check pendant Bloc 1 (migration ollama-rs).
Reclasse le preflight `sprint72_phase_c_preflight.md` (verdict PLAN-ADAPT) en
**DESIGN-CONFLICT** sur un fait que le preflight a manque.

---

## 1. Le fait nouveau (ground truth, pas de la doc)

Le preflight a conclu « schemars reste 0.8.22, pas de DESIGN-CONFLICT worker »
en lisant le **changelog 0.3.0** (schemars 0.8.21 → 0.8.22). C'est faux pour le
patch **0.3.4** reellement resolu.

Evidence (`cargo update -p ollama-rs --precise 0.3.4` + source vendored) :

- `~/.cargo/.../ollama-rs-0.3.4/Cargo.toml:209-211` :
  `schemars = { version = "1.2.0", features = ["preserve_order"] }`.
  → ollama-rs **0.3.4 depend de schemars 1.2**, pas 0.8.
- `cargo update` a ajoute la transitive `schemars_derive v1.2.1`.
- `ollama-rs-0.3.4/src/generation/parameters/mod.rs:92` :
  `pub fn new<T: JsonSchema>()` ou `JsonSchema` = **schemars 1.2**.

Consequence : `crates/nexus-worker-core/src/llm/schema_bridge.rs:48`
`JsonStructure::new::<TaskResponse>()` exige `TaskResponse: schemars_1.2::JsonSchema`.
Or `TaskResponse` (`nexus-core-rs/src/schemas/task_response.rs:66`) derive
**schemars 0.8** (pin workspace `0.8.21`). **Le bound n'est pas satisfait → le
worker ne compile plus.** C'est le coeur du conflit.

Pourquoi `new_for_schema(Schema)` ne sauve pas trivialement (Option B non
byte-clean) : `JsonStructure::new::<T>()` strippe les `$ref` via
`SchemaSettings::inline_subschemas` (Ollama ne supporte pas les `$ref`). Or
`task_response_schema()` derive de `TaskResponse` qui **contient `ToolCall`
imbrique** → `schema_for!` 0.8 emet un `$ref`/`definitions`. Reconstruire un
`JsonStructure` depuis ce `serde_json::Value` enverrait un schema avec `$ref` a
Ollama (regression comportementale), sauf a re-inliner a la main.

## 2. Cartographie de l'impact (verrouillee)

| Surface | schemars ? | Touche par le conflit ? |
|---|---|---|
| `nexus-core-rs` TaskResponse/ToolCall derive + `schema_for!` | 0.8 | **Oui** (si on bump le derive) |
| `nexus-core-rs` snapshot `task_response.schema.json` committe | 0.8 output | **Oui** (regen si bump) |
| `nexus-worker-core` `JsonStructure::new::<TaskResponse>()` | exige la version d'ollama-rs | **Oui, site unique de collision** |
| `nexus-worker-core` llama.cpp `TopLevelGrammar::from_json_schema(Value)` | decouple (prend un `serde_json::Value`) | indirect : la **draft** du schema change si bump (feature `llm_llama_cpp`, off par defaut) |
| **`sbfb-factory`** (Ollama execution target, Bloc 2) | **aucun** | **NON** — streame du texte libre, jamais de structured output |

**Cause racine** : la collision vient **exclusivement du bump du worker a 0.3.4**
(D2 « + bump worker 0.2.6→0.3.4 »). `sbfb-factory` sur 0.3.4 n'a aucun probleme
schemars. Le worker seul tire schemars 1.2 dans son `JsonStructure::new::<T>()`.

## 3. Pourquoi c'est un arbitrage PO, pas une decision implementer

- Le PO a arbitre **D2 (ollama-rs 0.3.4 partout + bump worker)** pour eviter une
  divergence de version cross-crate. Le **cout cache** (forcer schemars 0.8→1.2
  sur le struct de schema wire du core) lui etait **invisible** : kickoff §3,
  plan §6, preflight S1b — tous supposaient schemars reste 0.8.
- Le bump 0.8→1.2 **contredit une decision documentee S20** :
  `Cargo.toml:327` « the 0.8 → 1.0 upgrade is an ecosystem-wide churn we avoid ».
- S72 est cadre **« strictement un sprint de cablage »** (kickoff §1.1). Une
  migration majeure de dep sur le schema core n'etait pas dans ce cadre.

PLAN-ADAPT « ne peut PAS toucher Day-0 figees ». Bumper le generateur de schema
du struct wire core a travers une version majeure, contre une decision « we
avoid this churn » explicite, depasse PLAN-ADAPT.

## 4. Options

### Option A — Bump workspace schemars 0.8 → 1.2 (honore D2 tel quel)
Worker reste sur ollama-rs 0.3.4. Bump pin workspace `0.8.21 → 1.2`. Migrer les
derives `TaskResponse`/`ToolCall` + `task_response_schema()` (API `schema_for!`
1.x) + **regenerer le snapshot committe** `task_response.schema.json` + valider
que le changement de draft (draft-07 → 2020-12, `definitions` → `$defs`) reste
accepte par `llguidance::from_json_schema` (workers llama.cpp GPU — surface de
correctness reelle, feature-gated off en CI).
- **+** Etat final propre, une seule version schemars + ollama-rs.
- **−** Plus gros blast radius ; **renverse** la decision S20 ; transforme un
  sprint « routing » en migration de dep majeure sur le schema core ; risque
  correctness llama.cpp (constrained decoding).

### Option B — Contenir schemars 1.2 au chemin Ollama du worker (D2 + workspace reste 0.8)
Worker sur 0.3.4, mais nourrir le schema **sans** `new::<T>()` : soit (b1)
schemars 1.2 en dep renommee + dual-derive `JsonSchema` sur TaskResponse, soit
(b2) construire `JsonStructure` depuis un `Value` **manuellement inline**
(`$ref` resolus) via `serde_json::from_value`.
- **+** Workspace reste schemars 0.8, snapshot intact.
- **−** Complexite/laideur locale (deux schemars dans le graphe worker, ou
  re-inlining maison fragile) ; site `schema_bridge.rs` plus complexe.

### Option C — Re-scope D2 : worker reste ollama-rs 0.2.x, seul sbfb-factory adopte 0.3.4 [RECOMMANDE]
Accepter une divergence de version cross-crate **temporaire** (ce que D2 voulait
eviter). Worker `ollama-rs = "0.2.6"` (pin direct), workspace/factory `0.3.4`.
- **+** **Zero** touche schemars ; chemin quorum/determinisme worker **intact**
  (R7 trivialement sur) ; snapshot intact ; llama.cpp intact ; S72 reste un vrai
  sprint de cablage. Cargo gere deux versions ollama-rs cote a cote (elles ne
  partagent aucun type au runtime).
- **−** Deux versions ollama-rs dans l'arbre jusqu'a un sprint dedie de
  migration worker ; **contredit le but affiche de D2** (« une seule version »).
- Note : la migration worker + sa cascade schemars devient un item de dette
  trace, candidat a son propre sprint (ou decision deliberee), pas un effet de
  bord d'un sprint routing.

## 5. Recommandation implementer

**Option C.** S72 est cadre « cablage / quick win » ; la collision schemars est
le cout cache du bump worker, pas du routing. C reste fidele au scope, met R7 a
l'abri (worker non touche), et reporte la vraie migration worker a une decision
deliberee. La divergence de version est temporaire et inerte (aucun type partage
cross-crate). Mais **C renverse l'arbitrage D2 du PO** — d'ou cet arbitrage.

Si le PO veut l'alignement single-version maintenant et accepte d'elargir le
scope : Option A (et tracer le risque llama.cpp draft). Option B seulement si on
veut 0.3.4 worker SANS bouger le workspace, en acceptant la complexite locale.

## 6. Etat working tree au moment du STOP

Edits Bloc 1 deja poses (non committes), compatibles A/B, partiellement C :
- `Cargo.toml` workspace : pin `ollama-rs "0.2" → "0.3.4"` + commentaire schemars.
- `crates/sbfb-factory/Cargo.toml` : `+ ollama-rs { workspace=true, features=["stream"] }`.
- `crates/nexus-worker-core/src/llm/ollama.rs` : rename `GenerationOptions→ModelOptions`
  + `FormatType::StructuredJson(Box::new(...))` + commentaires.
- `Cargo.lock` : ollama-rs 0.3.4 resolu.

Selon le choix : C → repin worker `ollama-rs = "0.2.6"`, revert les 2 edits
worker ollama.rs (la 0.2.6 garde `GenerationOptions` + `StructuredJson` sans Box).
A → garder + migrer schemars. B → garder + retravailler `schema_bridge.rs`.

---

## 7. Decision PO (2026-06-03) — **Option A retenue**

Le PO choisit **A** : bump workspace schemars `0.8.21 → 1.2`, worker reste sur
ollama-rs 0.3.4, alignement single-version (honore D2 tel quel). La decision S20
« on evite le churn 0.8→1.0 » est **explicitement levee** par cet arbitrage (le
churn est desormais le cout assume de l'adoption ollama-rs 0.3.4 partout). Plan
d'execution :

1. `Cargo.toml` workspace : `schemars` `0.8.21 → 1.2` (features `["derive"]`).
2. `nexus-core-rs/.../task_response.rs` : API stable (`JsonSchema` derive +
   `schema_for!` + `serde_json::to_value` inchanges en 1.x) ; corriger les
   refs doc « draft-07 » / « RootSchema » devenues fausses.
3. `nexus-worker-core/.../schema_bridge.rs` : `JsonStructure::new::<TaskResponse>()`
   recompile (TaskResponse impl schemars 1.2 via son derive) ; corriger le
   commentaire `RootSchema → Schema`.
4. **Regenerer** le snapshot committe `task_response.schema.json`
   (`UPDATE_SNAPSHOTS=1`) — draft-07 → 2020-12, `definitions → $defs`.
5. Verifier que `llguidance::TopLevelGrammar::from_json_schema(Value)`
   (`llama_cpp.rs:487`, feature `llm_llama_cpp`) compile + accepte le nouveau
   draft (risque correctness assume PO — workers GPU feature-gated off CI).
6. R7 : `verifiable_task_uses_greedy_seed`, `two_honest_workers_same_hash`,
   `quorum_accepts_deterministic_redundancy`,
   `quorum_rejects_nondeterministic_divergence` + `deterministic_options_*`
   restent verts. Le commit body documentera le delta D2→ground-truth→A.
