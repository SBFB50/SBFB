# Audit S1 — `crypto.rs` (Sprint 2)

**Module**: `crates/nexus-core-rs/src/crypto.rs` (399 lignes)
**Plan ref**: Sprint 2, Jour 7 — Ed25519 sign/verify, BLAKE3 helpers, chain hash pour kudos ledger.

---

## Conforme

- `KeyPair` (generate, from_secret_bytes, secret_bytes, public_bytes, sign) — complet.
- `verify(pubkey, msg, sig) -> Result<()>` standalone — complet.
- `blake3_hash(data) -> [u8; 32]` — complet.
- `Blake3Chain` (new, from_head, head, append) avec formule `H_{i+1} = BLAKE3(H_i || entry_i)` — conforme au plan kudos ledger.
- `load_or_generate`: crée les répertoires parents manquants, gère le cas "fichier absent", rejette les fichiers de taille incorrecte.
- Perms Unix 0600 via `set_owner_only_perms` (cfg-gate unix/non-unix) — conforme à la doc.
- Aucune dépendance iroh/tokio — conforme à l'exigence "synchronous, Send+Sync".
- API dalek vérifiée sur registry local (dalek 2.2.0) :
  - `SigningKey::from_bytes(&[u8; 32]) -> Self` — infaillible, conforme.
  - `Signature::from_bytes(&[u8; 64]) -> Self` (ed25519 crate 2.2.3) — infaillible, conforme.
  - `VerifyingKey::verify` via trait `Verifier` importé explicitement — conforme.
  - `Blake3Hasher::new() / update(&[u8]) / finalize() -> Hash`, `Hash::as_bytes() -> &[u8; 32]` — conforme.

## Manquant

Rien. La spec Jour 7 ("Ed25519 sign/verify, BLAKE3 helpers, chain hash pour kudos ledger") est couverte intégralement.

## Déviations

- Le plan ne précisait pas `load_or_generate` pour ce module (c'était listé dans `node.rs` Jour 1-2). L'avoir ici est une amélioration : la fonction est partageable par node.rs et worker.rs sans duplication.
- `Blake3Chain::Default` implémenté (délègue à `new()`) — non exigé mais bon usage idiomatique Rust.

## Qualité

- **Zéro `.unwrap()` / `.expect()`** dans le code de production.
- `set_owner_only_perms` appelé avec `.ok()` (best-effort) — intentionnel et documenté (line 134).
- Docs inline couvrent format clé, format signature, sécurité `secret_bytes`, sémantique TOCTOU (Windows ACL).
- Pas de `unsafe`. Pas de dépendances non nécessaires.
- Constantes publiques exposées (`PUBLIC_KEY_LENGTH`, `SECRET_KEY_BYTES`, `SIGNATURE_BYTES`, `BLAKE3_BYTES`) — facilite les bindings PyO3.

## Tests

```
cargo test -p nexus-core-rs --lib crypto
running 15 tests ... test result: ok. 15 passed; 0 failed; 0 ignored
```

Couverture : keypair random, sign/verify roundtrip, rejet mauvais message, rejet mauvaise clé publique, restauration depuis secret, blake3 déterministe, blake3 sensible à l'entrée, chain genesis, chain avance, chain reproductible, chain détecte permutation, chain resume depuis checkpoint, load_or_generate crée fichier+sous-répertoire, load_or_generate relit fichier existant, load_or_generate rejette fichier trop court.

Lacune mineure : pas de test `load_or_generate` avec un chemin en lecture seule (permission refusée) pour valider le bras `Err(e) => Err(NexusError::Io(e))` à la line 137.

## Bugs (DO NOT FIX)

**Potentiel TOCTOU mineur** (line 133-134) : `fs::write` puis `set_owner_only_perms` sont deux appels séparés. Entre les deux, la fenêtre est infime mais la clé est brièvement lisible avec les permissions héritées du répertoire. Pas exploitable en pratique sur un `tempdir` applicatif, mais à noter pour les environnements à haute sensibilité. Mitigation complète nécessiterait `O_CREAT | O_WRONLY | O_EXCL` avec chmod atomique via `nix::unistd::fchmod`.

Pas d'autres bugs identifiés.
