État Git vérifié : `git diff --name-only` ne liste que les 2 fichiers web modifiés. Les nouveaux `docs/operators/QUANTIZATION.md` et `crates/nexus-worker-core/tests/quantization_doc.rs` sont présents mais `??` non suivis dans le working tree.

### Livrable 1 : doc opérateur `QUANTIZATION.md`
- Statut : CONFIRME
- Fichier(s) : `docs/operators/QUANTIZATION.md:31`, `:50`, `:69`, `:78`, `:143`
- Evidence :
```md
31: | Situation | Format recommandé | Pourquoi |
33: | **Cas par défaut** ... | **Q4_K_M** |
34: | **VRAM serrée** ... | **IQ4_XS** |
35: | **Tu serres la taille au plus juste** | **Q4_K_S** |
```
```md
50: | Modèle | Q4_K_M | IQ4_XS | Q2_K | Tient sur 1×16 Go ? |
54: | **14B** | **~8.5 Go** | ~7.6 Go | ~5.8 Go | ✅ **cible single-GPU honnête...** |
56: | **70B** | **42.52 Go** | **37.90 Go** | **26.38 Go** | ❌ **ne tient sur AUCUNE carte 16 Go** |
```
```md
78: Le 70B **ne tient sur aucune carte 16 Go**...
85: - Le **tensor-split mono-machine multi-GPU** (`with_split_mode` +
86:   `with_devices`...) est **hors cible**
88: - Le multi-GPU réaliste pour SBFB = **éclater le modèle sur 2+ machines
89:   à 1 GPU chacune = sharding cross-machine = Sprint 77**
```
```md
150: C'est une **condition d'exactitude / de joignabilité du quorum**, pas
151: une barrière de sécurité :
153: - Deux quants différents ... produisent des tokens **divergents**
154:   ⇒ l'**exact-match** du quorum **ne se forme jamais**.
```

Chiffres cohérents avec les contraintes : 14B Q4_K_M ~8.5 Go <= 16 Go ; 70B Q4_K_M 42.52 Go > 16 Go ; 70B Q2_K 26.38 Go > 16 Go. Pas de formulation “même GGUF = anti-Sybil” trouvée dans la section quorum.

### Livrable 2 : design note caps VRAM
- Statut : CONFIRME
- Fichier(s) : `docs/operators/QUANTIZATION.md:104`, `crates/nexus-worker-core/src/gpu/mod.rs:147`, `crates/nexus-worker-core/src/consent.rs:422`, `crates/nexus-core-rs/src/task.rs:258`
- Evidence :
```md
104: - **Budget VRAM live** : `GpuStats::vram_budget_remaining_bytes(max_vram_fraction)`
105:   (`crates/nexus-worker-core/src/gpu/mod.rs:147`) calcule la VRAM
107: - **Gate d'admission par cap** : `crates/nexus-worker-core/src/consent.rs:422-425`
108:   rejette toute tâche dont `task.estimated_vram_mb` dépasse
```
```rust
147: pub fn vram_budget_remaining_bytes(&self, max_vram_fraction: f32) -> u64 {
148:     let max = max_vram_fraction.clamp(0.0, 1.0);
149:     let allowed = (self.vram_total_bytes as f64 * max as f64) as u64;
150:     allowed.saturating_sub(self.vram_used_bytes)
```
```rust
422: if let Some(max_v) = consent.caps.max_vram_mb {
423:     if task.estimated_vram_mb > max_v {
424:         return AllowOutcome::Reject(RejectReason::CapVram);
```
```rust
258: /// Estimated VRAM footprint in MB. Same contract as
259: /// `estimated_watts` — zero = unknown = VRAM cap inert.
260: #[serde(default)]
261: pub estimated_vram_mb: u64,
```
La doc précise bien que le cap lit l’estimé déclaré par l’app, pas la taille GGUF réelle, et renvoie le câblage VRAM-live à S77 (`QUANTIZATION.md:112-122`).

### Livrable 3 : pointeur panneau “offrir ma puissance”
- Statut : CONFIRME
- Fichier(s) : `web/src/components/GpuConsentDialog.tsx:338`
- Evidence :
```tsx
338: <p
339:   className="text-xs text-white/40"
340:   data-testid="consent-quantization-hint"
342:   Ta carte 16 Go fait tourner des modèles entiers jusqu'à
343:   ≤14B en Q4_K_M. Les très gros modèles (70B) ne tiennent sur
346:   docs/operators/QUANTIZATION.md.
```
`rg "<a|href=|QUANTIZATION\.md"` sur ce fichier ne trouve que la ligne texte `346`; pas de `<a href>` relatif vers le `.md`.

### Livrable 4 : tests
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-worker-core/tests/quantization_doc.rs:41`, `:55`, `:68`, `:81`, `:97`, `web/src/components/__tests__/GpuConsentDialog.test.tsx:144`
- Evidence :
```rust
41: fn quantization_doc_present() {
43:     assert!(path.is_file(), "missing operator doc: {}", path.display());
45:     assert!(
46:         doc.len() > 1024,
```
```rust
55: fn quantization_doc_has_footprint_table() {
57:     for token in ["Q4_K_M", "IQ4_XS", "Q2_K", "14B"] {
58:         assert!(
```
```rust
97: fn llama_cpp_unchanged_doc_only() {
100:     src.contains("with_n_gpu_layers"),
103: for forbidden in ["with_split_mode", "with_devices"] {
105:     !src.contains(forbidden),
```
```tsx
149: const hint = screen.getByTestId("consent-quantization-hint");
150: expect(hint).toHaveTextContent("≤14B");
151: expect(hint).toHaveTextContent("Q4_K_M");
152: expect(hint).toHaveTextContent("docs/operators/QUANTIZATION.md");
```
Les 5 tests Rust nommés attendus existent et contiennent des assertions utiles. Le test web attendu existe et vérifie les trois fragments demandés.

### Invariant anti-scope-creep : `llama_cpp.rs`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-worker-core/src/llm/llama_cpp.rs:148`
- Evidence :
```rust
148: let backend = shared_backend()?;
149: let model_params = llama_cpp_2::model::params::LlamaModelParams::default()
150:     .with_n_gpu_layers(u32::try_from(self.n_gpu_layers.max(0)).unwrap_or(0));
152: let model = llama_cpp_2::model::LlamaModel::load_from_file(
```
`git diff -- crates/nexus-worker-core/src/llm/llama_cpp.rs` est vide. `rg "with_split_mode|with_devices"` ne trouve rien dans ce fichier ; seul `with_n_gpu_layers` est câblé.

## Resume final
- Total livrables vérifiés : 5
- Confirmes : 5
- Gaps : 0
- Partiels : 0
