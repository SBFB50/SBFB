# Audit S9 — PyO3 bindings (`nexus-core-py/src/lib.rs`)

**Date**: 2026-04-10  
**Auditeur**: Agent S9 (indépendant)  
**Fichier audité**: `crates/nexus-core-py/src/lib.rs` (634 lignes)

---

## Conforme

- Module signature PyO3 0.21+ correcte : `fn nexus_core(m: &Bound<'_, PyModule>)` (lib.rs:615)
- 5 classes exposées : `Node`, `Doc`, `Gossip`, `Blobs`, `Verifier` (lib.rs:618-622)
- `pyo3_async_runtimes::tokio::future_into_py(py, async move { ... })` utilisé partout
- `Bound<'_, PyBytes>` utilisé partout pour les extracteurs de bytes (jamais `&PyBytes`)
- `IntoPyObject` implémenté pour `ByteVec`, `NodeAddrDict`, `GossipEventDict` — remplace bien `IntoPy`/`ToPyObject` (lib.rs:206, 357, 415)
- `cargo check -p nexus-core-py` : **CLEAN**, 0 erreurs, 0 warnings

## Manquant

- Le plan (S9, jour 10-11) mentionne `pyo3-asyncio 0.21` — remplacé par `pyo3-async-runtimes 0.28` (correct, crate renommé). Pas un manque.
- `free functions sign/verify` : le plan demande `sign`/`verify` ; l'implémentation expose `sign_task`, `verify_task_entry`, `sign_result`, `verify_result_entry` (lib.rs:572-608). Noms plus précis, sémantique respectée.

## Déviations

- **Plan Cargo.toml** (plan ligne 473) pin `pyo3 = "0.22"` + `pyo3-asyncio = "0.21"`. Implémentation utilise `pyo3 = "0.28"` + `pyo3-async-runtimes = "0.28"` — delta de version intentionnel et bénéfique (0.28 = API stable `Bound`).
- `generate_secret` retourne un `PyDict` directement (pas via `future_into_py`) car synchrone — conforme mais les erreurs de `set_item` sont silencieusement ignorées via `.ok()` (lib.rs:258-259).

## Qualité

- `#![forbid(unsafe_code)]` + `#![deny(rust_2018_idioms)]` — bon niveau de rigueur
- Zéro `unwrap()` dans le code de binding — toutes les erreurs passent par `?` + `py_err()` helper
- `Arc<Mutex<Option<T>>>` pour Node/Doc/Gossip permet shutdown propre avec erreur Python claire

## Tests

- **SMOKE TEST : PASSE** (`S9 verify OK`)
  - `generate_secret` → `create_node_with_secret` → `docs_create` → `docs_author_create` → `doc.set` → `doc.share_write` (prefix `doc` validé) → `blobs.add_bytes` → `blobs.get_bytes` round-trip : tout OK

## Bugs (DO NOT FIX)

1. **lib.rs:258-259** — `generate_secret`: erreurs de `d.set_item(...)` swallées via `.ok()`. Si `PyDict::set_item` échoue (OOM), la fonction retourne un dict vide sans lever d'exception Python. Devrait utiliser `?` et retourner `PyResult<Bound<'py, PyDict>>`.
2. **lib.rs:477** — `Blobs::get_bytes` attend un hash de 32 bytes via `array32()`. Mais `add_bytes` retourne `ByteVec(hash.to_vec())` — si le hash sous-jacent (`iroh_blobs::Hash`) n'est pas exactement 32 bytes, le round-trip échoue avec `ValueError` cryptique. À vérifier que `Hash::to_vec()` produit toujours 32 bytes (blake3 = 32 bytes, probablement OK, mais non vérifié dans les bindings).
