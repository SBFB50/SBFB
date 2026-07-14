Verdict indépendant : **7 livrables confirmés, 2 partiels, aucun GAP complet**. Les deux réserves concernent le même défaut : la fixture `until` du self-test chevauche une autre branche du motif, et P70 affirme à tort que cette suppression serait détectée.

Branche vérifiée : `master`. Aucun fichier n’a été modifié pendant l’audit.

### Livrable 1 : élargissement de `PROMISE_RE`

- Statut : **CONFIRME**
- Fichier(s) : [scripts/check-frontier-contracts.sh:79](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:79)
- Evidence :

```text
until (the )?(Sprint |S)[0-9]
[Ww]hen (Sprint |S)[0-9]+ (lands|activates|ships)
(Sprint |S)[0-9]+\+? (sandbox|allow-list)
(Sprint |S)[0-9]+\+? activates
```

La comparaison machine entre `HEAD` et le fichier vivant donne `old=9 current=13 added=4`. Le motif n’utilise ni `-P`, ni `\b`, ni `\s`, et les appels sont en `grep -E`.

`bash -n scripts/check-frontier-contracts.sh` et `bash scripts/check-frontier-contracts.sh` terminent tous deux avec exit `0`. Le gate complet affiche :

```text
check-frontier-contracts: clean (anti-promise + frontier-tag coverage
[1 tagged] + BLOB_SERVE_CSP non-regression + prompt-kind provenance)
```

### Livrable 2 : self-test de non-vacuité

- Statut : **PARTIEL**
- Fichier(s) : [scripts/check-frontier-contracts.sh:88](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:88)
- Evidence :

```sh
100: 'promise: tool_calls inert until Sprint 22 activates the gate' \
101: 'promise: the schema does not bump when S25 lands' \
102: 'promise: match the name against the S25+ allow-list'; do
103: if ! printf '%s\n' "$_promise_pos" | grep -qE "$PROMISE_RE"; then
104:   echo "PROMISE_RE self-test: vacuous or malformed (...)"
```

La fixture négative est bien testée dans un `if` sans `|| true` aux lignes 108-110, avant la boucle de scan qui commence ligne 113. Les trois positives matchent et la négative ne matche pas.

Une exécution du bloc vivant avec un regex volontairement malformé produit trois diagnostics, positionne `fail=1` et n’est pas interrompue par `set -e`.

Cependant, la première fixture matche simultanément :

- `until (the )?(Sprint |S)[0-9]`
- `(Sprint |S)[0-9]+\+? activates`

Les mutations en mémoire donnent :

```text
UNTIL_FAMILY STILL_MATCHES
WHEN_FAMILY NO_LONGER_MATCHES
CAPABILITY_FAMILY NO_LONGER_MATCHES
ACTIVATES_BRANCH STILL_MATCHES
```

Ce qui manque : une fixture `until-token` ne contenant pas également `Sprint N activates`, ainsi qu’une couverture isolée de la branche autonome `activates` si le self-test prétend protéger les quatre branches.

### Livrable 3 : réécriture immuable de `task_response.rs`

- Statut : **CONFIRME**
- Fichier(s) : [task_response.rs:14](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/task_response.rs:14), [CAPABILITY_TOGGLES.md:42](/C:/Users/FlowUP/Documents/Code/nexus/docs/security/CAPABILITY_TOGGLES.md:42)
- Evidence :

```rust
82: /// Tool calls the worker asks the coordinator to dispatch on
83: /// its behalf. Declarative since Sprint 20: workers emit `[]`
84: /// and the coordinator ignores the field — the tool-calling
85: /// capability was deferred and stays OFF (source of truth:
86: /// `docs/security/CAPABILITY_TOGGLES.md`, gate `tool_calling`).
```

```rust
102: /// Name of the tool to invoke (e.g. `"http_get"`,
103: /// `"fs_read"`). No allow-list is enforced today — the
104: /// tool-calling capability stays OFF and the field is ignored.
106: /// Free-form JSON arguments the tool will receive. Shape is
107: /// delegated to the tool author [...]
```

Le module-doc ligne 14 et le bloc `ToolCall` lignes 93-98 sont également réécrits. Aucun `S22`, `S25` ou `wasmtime` ne subsiste dans ce fichier. Le fichier de référence confirme `tool_calling = OFF`, et la recherche des usages montre que `tool_calls` n’a aucun consommateur de dispatch dans le coordinateur.

### Livrable 4 : snapshot régénéré sans changement wire

- Statut : **CONFIRME**
- Fichier(s) : [task_response.schema.json:5](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/task_response.schema.json:5), [task_response.rs:48](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/task_response.rs:48), [task_response.rs:281](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/task_response.rs:281)
- Evidence :

```text
$.$defs.ToolCall.description
$.$defs.ToolCall.properties.arguments.description
$.$defs.ToolCall.properties.name.description
$.properties.tool_calls.description
```

La comparaison récursive JSON donne `changed_paths=4 non_description=0`. `required`, `$defs`, types et defaults sont donc identiques à `HEAD`.

Les constantes restent :

```rust
48: pub const TASK_RESPONSE_VERSION: u8 = 1;
54: pub const TASK_RESPONSE_DOMAIN_TAG: &str = "TASK_RESPONSE_V1";
```

Le test demandé passe réellement : `1 passed; 0 failed` pour `schemas::task_response::tests::schema_snapshot_matches_struct`. `UPDATE_SNAPSHOTS` était absent de l’environnement pendant le test.

### Livrable 5 : vérité présente sur le digest

- Statut : **CONFIRME**
- Fichier(s) : [verification.rs:28](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/verification.rs:28), [runtime.rs:1473](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-worker-core/src/engine/runtime.rs:1473)
- Evidence :

```rust
30: //! (`validate_quorum_pre_guardrail`). Callers must not treat the
31: //! digest as a weights attestation — to date it binds the model
32: //! NAME string only.

1473: fn model_name_digest(model: &str) -> [u8; 32] {
1474:     blake3_hash(model.as_bytes())
```

Sur `HEAD`, le motif final détecte bien l’ancienne ligne 31 : `digest as a weights attestation until S77.`

### Livrable 6 : nettoyage de `llm/mod.rs`

- Statut : **CONFIRME**
- Fichier(s) : [llm/mod.rs:28](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-worker-core/src/llm/mod.rs:28)
- Evidence :

```rust
28: //! (invalid JSON → signature refuse). But everything upstream of
29: //! sample time (tool-call interception, process-boundary VRAM
30: //! wipe for ephemeral workers, signing inline with sampling)
31: //! requires direct control over the LLM process — impossible
32: //! across the Ollama HTTP boundary.
```

```rust
45: //! against prompt injection is a separate layer : client-side
46: //! redaction (the tool-calling capability stays OFF —
47: //! `docs/security/CAPABILITY_TOGGLES.md`, gate `tool_calling`). See
48: //! `docs/rust/PATTERNS.md §P30` for the longer form warning.
```

Les tags S22/S23/S26 ont disparu de la rationale tout en conservant l’argument de frontière de processus. La référence au sandbox S22 a également disparu. Le SDK de redaction invoqué existe et est branché dans `web/src/sdk/pii/index.ts:3-15` et `web/src/bridge/useBridge.ts:136-143`.

### Livrable 7 : correction du nom de test

- Statut : **CONFIRME**
- Fichier(s) : [task_response.rs:35](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/task_response.rs:35), [schemas/mod.rs:29](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/mod.rs:29), [task_response.rs:281](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/task_response.rs:281)
- Evidence :

```rust
35: //! 2. **Grammar drift** — if a future sprint redefines the
36: //!    canonical domain tag without regenerating the schema
37: //!    snapshot, the test `schema_snapshot_matches_struct`
38: //!    fires.
```

```rust
29: //! A `task_response.schema.json` file lives next to this module as
30: //! a **snapshot** used by `schema_snapshot_matches_struct`:
31: //! if the struct evolves and the snapshot is not regenerated, the
32: //! test fails loudly with a diff.
```

La commande exacte `grep -rn "test_schema_snapshot_matches_struct" crates/` produit une sortie vide, soit **0 hit**. La vraie fonction est bien définie ligne 281 et son test passe.

### Livrable 8 : synchronisation des PATTERNS

- Statut : **PARTIEL**
- Fichier(s) : [docs/rust/PATTERNS.md:3907](/C:/Users/FlowUP/Documents/Code/nexus/docs/rust/PATTERNS.md:3907), [docs/rust/PATTERNS.md:1544](/C:/Users/FlowUP/Documents/Code/nexus/docs/rust/PATTERNS.md:1544), [docs/shell/PATTERNS.md:1868](/C:/Users/FlowUP/Documents/Code/nexus/docs/shell/PATTERNS.md:1868)
- Evidence :

```text
3907-3912 : les 9 branches historiques et les 4 nouvelles sont listées.
3922-3927 : résidu tense-anchoring et ambiguïté S<digit> documentés.
3932-3934 : classe post-SN explicitement laissée hors gate.
1544-1548 : tool_calling OFF et wasmtime jamais câblé.
1868-1871 : même recadrage présent-vrai côté shell.
```

Le décompte machine du motif confirme bien les **13 branches**. Les reformulations P30 sont cohérentes avec le code vivant : SDK de redaction présent, `tool_calling` OFF, et `wasmtime` absent de `Cargo.lock`/des manifests, uniquement banni préventivement dans `deny.toml`.

Le point partiel se trouve lignes 3914-3917 :

```text
3914: A self-test (three positive fixtures — one per new branch
3915: family — plus an anchored negative [...]) fails the gate if
3916: PROMISE_RE [...] silently loses one of the new branch
3917: families [...]
```

Cette affirmation n’est pas vraie pour la famille `until` ni pour la branche autonome `activates`, à cause du chevauchement démontré au livrable 2.

### Livrable 9 : invariant de périmètre

- Statut : **CONFIRME**
- Fichier(s) : [task_response.rs:45](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/task_response.rs:45), [scripts/check-frontier-contracts.sh:79](/C:/Users/FlowUP/Documents/Code/nexus/scripts/check-frontier-contracts.sh:79)
- Evidence :

```text
rust_changed_lines=52 non_doc_comment_lines=0
behavior_signature_hits=0
changed_paths=4 non_description=0
dependency diff: vide
```

Le `git diff` suivi contient exactement les huit fichiers attendus : quatre fichiers Rust dont seules les lignes `//!`/`///` changent, le script, les deux `PATTERNS.md` et le snapshot. Aucun champ, attribut serde, constante, corps de fonction, `Cargo.toml`, `Cargo.lock` ou `deny.toml` n’est modifié. `git diff --check` passe.

`git status` signale aussi deux fichiers Markdown non suivis :

```text
?? .planning/active/sprint82_phase_f_preflight.md
?? .planning/active/sprint82_phase_f_review.md
```

Ce sont des rapports de phase non comportementaux et ils ne font pas partie du `git diff` suivi ; ils sont néanmoins signalés ici pour rendre le working tree exact.

### Vérifications croisées exécutées

- Gate complet : exit `0`.
- Rouge-avant-vert avec le `PROMISE_RE` extrait du fichier vivant sur `HEAD` : hits exacts `task_response.rs:14,84,93,95,100`, `verification.rs:31`, `llm/mod.rs:29`.
- Fixtures finales : trois positives matchées, négative non matchée.
- Regex malformé injecté dans le bloc vivant : diagnostic, `fail=1`, aucun abort `set -e`.
- Ancienne référence de test : 0 hit.
- Lignes ajoutées sous `crates/` et `web/src` contre le motif final : `NO_MATCH_IN_ADDED_LINES`.
- Snapshot ciblé : `1 passed; 0 failed`.

## Résumé final

- Total livrables : **9**
- Confirmés : **7**
- Gaps : **0**
- Partiels : **2**