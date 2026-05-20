# Codex Security Track — Sprint 66 Phase B

Commit: `ea87547` | Verifier: Codex independant (Opus 4.6 1M)
Date: 2026-05-19 | Scope: 4 fichiers, 148 insertions, 1 deletion

## Verdict: CLEAN

Aucun gap de securite identifie. Les 6 questions posees sont
toutes resolues favorablement.

---

## Q1 — Pragma synchronous=FULL placement

**Fichier**: `crates/nexus-coordinator-rs/src/db.rs:218`

Sequence actuelle dans `open()`:
```
l.217  conn.pragma_update(None, "journal_mode", "WAL")?;
l.218  conn.pragma_update(None, "synchronous", "FULL")?;
l.219  conn.pragma_update(None, "foreign_keys", "ON")?;
l.221  let migrations = Migrations::new(MIGRATIONS.to_vec());
l.222  migrations.to_latest(&mut conn)?;
```

**Verdict: CORRECT.** L'ordre est semantiquement optimal:
1. WAL d'abord (change le journal mode, prerequis pour que
   synchronous=FULL ait son sens WAL-specifique)
2. synchronous=FULL ensuite (en WAL, FULL garantit fsync du
   WAL a chaque commit — sans cela le defaut NORMAL ne
   protege pas contre corruption apres crash OS)
3. foreign_keys avant migrations (les migrations peuvent creer
   des FK)

SQLite documente que `synchronous` est un pragma runtime, pas
une propriete du fichier — l'ordre WAL puis synchronous est
le seul qui garantit que le mode est effectif pendant les
migrations elles-memes.

## Q2 — open_in_memory() sans synchronous=FULL

**Fichier**: `crates/nexus-coordinator-rs/src/db.rs:227-235`

`open_in_memory()` n'a pas le pragma `synchronous=FULL`. C'est
une question pertinente.

**Verdict: ACCEPTABLE — pas un gap.**

Justification:
- `synchronous` controle le fsync vers disque. Une base
  in-memory n'a pas de fichier disque — le pragma est
  semantiquement sans objet.
- Tous les appels a `open_in_memory()` dans le codebase sont
  dans des modules `#[cfg(test)]` ou dans des test helpers
  (verifie: `db.rs` tests, `dispatcher.rs` tests,
  `public_feed.rs` tests, `kudos_ledger.rs` tests,
  `http.rs` test fixtures avec message "test coordinator DB").
- Aucun appel production a `open_in_memory()` pour
  `CoordinatorDb`. Le daemon utilise exclusivement
  `CoordinatorDb::open(path)` qui a le pragma.

## Q3 — THREAT_MODEL feed: severites et mitigations

**Fichier**: `docs/security/THREAT_MODEL.md:472-539`

### T-FEED-INTEGRITY (Severite H, Likelihood M)
- **Coherence**: H est correct pour un tampering de donnees
  signees sur un reseau P2P untrusted. M likelihood est
  coherent avec les sections §5.1-§5.8 existantes (transport
  iroh-docs est chiffre mais les peers sont untrusted).
- **Mitigation existe dans le code**: `verify_entry()` a
  `public_feed.rs:445-485` — verifie BLAKE3 hash-chain
  (`recomputed != entry_hash` -> reject) + Ed25519 signature
  (`nexus_core_rs::verify()` -> reject). Both paths reject
  with error (pas de silent pass).
- **Residual "Nil"**: correct pour une garantie cryptographique
  (Ed25519 + BLAKE3 sont assumes solides, cf. hypothese H2
  du THREAT_MODEL §1.3).

### T-FEED-SPAM (Severite M, Likelihood M)
- **Coherence**: M/M est correct. C'est un DoS, pas un
  compromis d'integrite (donc pas H).
- **Mitigations existent dans le code**:
  - Rate limit: `FEED_RATE_LIMIT_PER_MINUTE = 5` a
    `public_feed.rs:213`, enforce dans
    `insert_feed_operation_rate_limited()` a l.310-335.
  - Size limit: `MAX_OPERATION_JSON_SIZE = 65_536` a
    `public_feed.rs:210`, enforce dans
    `validate_feed_operation()` a l.226.
  - GCRA externe: `feed_limiter.rs` (governor crate) pour
    feed_sync remote ingestion.
- **Residual "L (Sybil)"**: correct et documente dans les
  residual risks feed (§Residual risks feed, l.534).

### T-FEED-FORGERY (Severite H, Likelihood L)
- **Coherence**: H/L est le pattern standard pour attaque
  crypto (impact catastrophique, requires crypto break).
  Coherent avec §5.3 deploy-from-repo (meme pattern S/T).
- **Mitigation existe**: `verify_entry()` a l.463-482 verifie
  `pubkey_bytes.len() == 32 && sig_bytes.len() == 64` puis
  appelle `nexus_core_rs::verify()`. La branche `else`
  (l.477-481) **rejette** les longueurs invalides — pas de
  bypass silencieux.

### T-FEED-CLOCK-SKEW (Severite M, Likelihood L)
- **Coherence**: M/L est raisonnable. Impact limite (ordering
  by seq, pas by timestamp), detection facile.
- **Mitigation existe**: `FEED_MAX_FUTURE_SECS = 30 * 24 * 3600`
  a `public_feed.rs:490`, enforce dans
  `validate_feed_entry_timestamp()` a l.495-503.
  Test `test_adversarial_future_timestamp_rejected` a l.1592.
- **Residual "L (past timestamps)"**: correct et transparent.
  Le ordering est par `seq` (monotone), pas par timestamp.

### Residual risks feed (l.532-539)
- Sybil, quarantine, revocation: correctement identifies comme
  hors-scope Sprint 66 et programmes Sprint 67+.
- Coherent avec la roadmap v3 Arc 2.

**Verdict Q3: COHERENT.** Severites/likelihoods alignees avec
le modele existant. Toutes les mitigations citees existent dans
le code avec des chemins de reject explicites.

## Q4 — Test coordinator_db_synchronous_full

**Fichier**: `crates/nexus-coordinator-rs/src/db.rs:1313-1323`

```rust
fn coordinator_db_synchronous_full() {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("coordinator.db");
    let db = CoordinatorDb::open(&path).expect("open");
    let sync_val: i64 = db
        .conn
        .pragma_query_value(None, "synchronous", |row| row.get(0))
        .expect("pragma query");
    assert_eq!(sync_val, 2, "synchronous must be FULL (2) in WAL mode");
}
```

**Analyse false-positive**:
- Le test utilise `CoordinatorDb::open()` (pas `open_in_memory`)
  avec un vrai fichier tmpdir — exercice du path production exact.
- `pragma_query_value("synchronous")` retourne un integer:
  0=OFF, 1=NORMAL, 2=FULL, 3=EXTRA. La valeur attendue 2 est
  correcte pour FULL.
- Le test ne peut PAS passer a tort: si le pragma n'est pas
  applique, SQLite retourne le defaut WAL (1=NORMAL en WAL mode,
  2=FULL en journal mode non-WAL). Mais ici le test ouvre avec
  `open()` qui set WAL puis FULL, donc la valeur 2 confirme
  bien que le pragma override a ete applique.
- Edge case: SQLite default en DELETE journal mode est FULL (2).
  Si le WAL pragma echouait silencieusement, le default serait
  aussi 2 et le test passerait. MAIS: `pragma_update("journal_mode",
  "WAL")` utilise `?` (l.217) qui propagerait toute erreur.
  Et le WAL pragma ne peut pas echouer silencieusement sur un
  fichier neuf (pas de concurrent reader lock).

**Verdict Q4: TEST VALIDE.** Pas de false-positive dans le cas
nominal. L'edge case theorique (WAL fail silencieux → default
FULL coincide) est bloque par le `?` error propagation sur le
WAL pragma.

## Q5 — PATTERNS.md §P51 information leakage

**Fichier**: `docs/rust/PATTERNS.md:2554-2607`

Le pattern documente:
- Structure `FeedEntry.op` (public struct, deja dans les sources
  AGPL-3.0)
- Fonctions `try_parse_op`, `op_type` (publiques, meme licence)
- `validate_feed_operation` avec commentaires pseudocode (pas le
  code reel, juste la logique en 3 points)
- Line references: `l.79`, `l.110-112`, `l.115-117`, `l.224-236`
  — verifies comme exacts au moment de l'audit
- Invariants (version policy, serde(default) rationale, canonical
  vs transport fields)

**Verdict Q5: PAS DE LEAKAGE.**
- Le projet est AGPL-3.0: le code source est public par design.
  Les line references ne revelent rien qu'un `git clone` ne
  donnerait deja.
- Le pseudocode `validate_feed_operation` est un resume correct
  sans exposer les regles de validation specifiques (hex-64,
  HTTPS-only, etc.) qui sont dans le code reel.
- Les invariants documentes sont des decisions d'architecture
  publiques, pas des secrets de securite.
- Aucun secret, token, cle, ou donnee sensible dans le pattern.

## Q6 — Regression securite dans les 4 fichiers

### db.rs
- Ajout `synchronous=FULL`: renforcement securite (durabilite).
  Aucune regression. Le test confirme le pragma.
- Pas de `unsafe`, `unwrap` problematique, ou `#[allow]` ajoute.

### README.md (docs/claude/)
- Ajout de la politique "deletions dans chore(cleanup)" — pure
  convention workflow, aucun impact securite.

### PATTERNS.md
- Ajout §P51 documentation pattern — documentation interne,
  aucun impact securite (cf. Q5).

### THREAT_MODEL.md
- Ajout §10 Feed surface — renforcement documentation securite.
  Le renommage §10→§11 pour "Revue et evolution" est cosmetique.
- La note de version v3 reference correctement "T-FEED-1..T-FEED-4"
  bien que les sections utilisent les noms longs (T-FEED-INTEGRITY,
  T-FEED-SPAM, T-FEED-FORGERY, T-FEED-CLOCK-SKEW). Mineur,
  pas de confusion possible.

**Verdict Q6: ZERO REGRESSION.** Les 4 fichiers sont soit des
renforcements (db.rs pragma, THREAT_MODEL surface), soit de la
documentation pure (README, PATTERNS).

---

## Resume

| Question | Verdict | Detail |
|----------|---------|--------|
| Q1 Pragma order | CORRECT | WAL → FULL → FK → migrations |
| Q2 open_in_memory gap | ACCEPTABLE | Test-only, pas de disque, semantiquement N/A |
| Q3 THREAT_MODEL coherence | COHERENT | 4/4 severites alignees, 4/4 mitigations verifiees dans le code |
| Q4 Test false-positive | VALIDE | Assert 2=FULL, WAL `?` bloque l'edge case |
| Q5 PATTERNS info leakage | CLEAN | AGPL-3.0 public, pas de secret |
| Q6 Regression | ZERO | Renforcements + documentation |

**Verdict global: CLEAN** — aucun gap securite identifie dans le
diff Phase B.
