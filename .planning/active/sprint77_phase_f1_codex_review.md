Verdict global: **CONFIRMÉ**, avec une réserve de vérification runtime: je n’ai pas relancé les 4 tests GGUF ignorés faute de `SBFB_SHARD_TEST_GGUF`, mais j’ai vérifié leurs assertions et le build feature-gaté.

**1. API Partial-Decode C++: CONFIRMÉ**
Les 4 champs sont ajoutés aux params publics et internes, recopiés dans `cparams`, avec défauts whole-model: `patches/llama-cpp-shard.patch:21`, `:35`, `:52`, `:66`, `:77`.  
La coupe LLAMA est cohérente: boucle `[shard_start, shard_end)`, assert borné, gather au dernier layer exécuté, sortie raw boundary hors last et norm/lm_head seulement last: `patches/llama-cpp-shard.patch:91`, `:97`, `:111`, `:129`, `:146`.

**2. P-D `TENSOR_SKIP`: CONFIRMÉ**
La lambda skippe les tenseurs par-couche hors fenêtre via `tn.bid`: `vendor/llama-cpp-sys-2/llama.cpp/src/llama-model.cpp:3082`. `tok_embd` reste inconditionnel, `output_norm/output` sont skippés hors last: `.../llama-model.cpp:3126`, `:3128`, `:3131`.  
Accounting OK: `TENSOR_SKIP` fait `size_data -= nbytes` et `n_created++`, puis le check final compare `n_created != n_tensors`: `.../llama-model-loader.cpp:1101`, `:1106`, `:1107`, `:1315`. Cas tied/non-tied OK: output absent optional ne compte pas; output présent skippé compte.

**3. Fix D1-1 Middle-Shard: CONFIRMÉ**
Le vrai deref était bien `build_inp_embd(model.tok_embd)` inconditionnel: `vendor/llama-cpp-sys-2/llama.cpp/src/models/llama.cpp:13`, et `build_inp_embd` construit toujours le chemin token `ggml_get_rows(tok_embd, ...)`: `.../llama-graph.cpp:1646`, `:1650`.  
Le fix garde `tok_embd` résident sur tout shard: `.../llama-model.cpp:3117`, `:3128`. `output_norm/lm_head` ne sont référencés que dans `if (shard_last)`: `.../src/models/llama.cpp:167`, `:168`, `:175`.

**4. Fix D2-1 `add_embedding` Soundness: CONFIRMÉ**
`embd_n` est stocké à `0` pour batches token/get_one et à `n_embd` pour `new_embeddings`: `vendor/llama-cpp-2/src/llama_batch.rs:187`, `:213`, `:334`.  
`add_embedding` refuse batch token, largeur incorrecte, seq_ids trop larges, puis stride sur `self.embd_n`: `.../llama_batch.rs:244`, `:252`, `:258`, `:265`, `:273`, `:276`. Pour le chemin F1, l’écriture unsafe est bornée par `allocated`, largeur exacte et `n_seq_max`.

**5. Backend Rust `shard.rs`: CONFIRMÉ**
`ShardBackend::load` applique `with_shard_range` avant `load_from_file`, puis valide fenêtre/flags après load: `crates/nexus-worker-core/src/llm/shard.rs:255`, `:264`, `:288`, `:297`. L’abort natif hors-bornes est bien documenté: `shard.rs:235`.  
`forward_tokens` refuse non-first/vide, `forward_hidden` refuse first et valide le shape via `hidden_token_count`; contexte force embeddings + pooling None + même range: `shard.rs:336`, `:361`, `:399`, `:406`, `:421`, `:448`. `top_k` et `hidden_token_count` sont purs et déterministes: `shard.rs:142`, `:169`.

**6. Tests: PARTIEL**
Confirmé pour les 6 hermétiques: ils existent et passent (`cargo test -p nexus-worker-core llm::shard::tests --locked`: 6/6). Les 4 GGUF sont bien `#[ignore]` et le test feature-gaté les liste comme ignorés: `shard.rs:597`, `:616`, `:653`, `:696`.  
Les assertions partial/full et 3-way prouvent bien `len` égal + cosine `>0.999`: `shard.rs:641`, `:643`, `:687`, `:689`. Réserve: je n’ai pas reproduit l’exécution GGUF locale.

**7. Hygiène: CONFIRMÉ**
Workspace reste en caret `0.1.143`, override path vers vendored `0.1.146`: `Cargo.toml:362`, `:484`, `:491`. Lock et manifests vendored confirment `0.1.146`: `Cargo.lock:4623`, `:4635`, `vendor/llama-cpp-2/Cargo.toml:15`.  
MIT restauré et notices présentes: `THIRD-PARTY-NOTICES.md:13`, `:15`, `vendor/llama-cpp-sys-2/llama.cpp/LICENSE:1`. `set_shard_range` absent; seules refs `with_shard_range` restent.

**8. 0-BUMP-WIRE / Scope F1-F2: CONFIRMÉ**
Aucun nouveau `DOMAIN_*` ni `*_FORMAT_VERSION` dans le delta audité. Pas de touch sur `shard_plan.rs`, `canonical.rs`, `consent.rs`, `compute_group.rs`. Les seuls points scope sont `llm/shard`, wrappers llama, patch vendored, Cargo override, notices. Pas de claim ComputeGroup, cap VRAM, signature manifest, ALPN `sbfb/shard/1`, ni Threat Model §16 dans le delta F1.

Commandes vérifiées: `cargo check -p nexus-worker-core --features llm_llama_cpp --locked` OK; `cargo test -p nexus-worker-core --features llm_llama_cpp llm::shard --locked` OK, 6 passed / 4 ignored.

GAP P0/P1: **aucun**. Réserve non bloquante: exécution GGUF ignorée non reproduite ici.