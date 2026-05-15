# Sprint 62 — Audit findings

**Date** : 2026-05-15
**Auditeur** : session fraiche (pas de contexte S62 implementation)
**Sprint audite** : Sprint 62 (feed sync P2P + anti-spam minimal)
**Source** : `sprint63_audit_plan.md` (5 dimensions)
**Tip audite** : `933ff0d`

---

## Verdict : PASS

0 P0, 0 P1, 5 P2, 2 P3.
Rigor G4 satisfait (5 P2+ documentes).
Aucun fix bloquant requis avant Sprint 63 Phase A.

---

## §1 Metriques verifiees

| Metrique | Attendu | Observe | Verdict |
|---|---|---|---|
| Rust nextest | >= 1299, 0 fail | 1299 pass, 0 fail | PASS |
| Vitest | >= 258 | 258 pass | PASS |
| size-limit | 6/6 | 6/6 | PASS |
| clippy | 0 warnings | 0 warnings | PASS |
| Commits code | 4 feat + 8 fix | 4 feat + 7 fix(feed) | NOTE (voir P3-1) |
| Delta tests | +17 (plan +12) | +17 confirme | PASS |

---

## §2 Dimension 2.1 — Feed sync correctness

### 2.1.1 publish_feed_entry_to_docs() PoW nonce — PASS

`feed_sync.rs:55-56` : si `pow_nonce` absent, calcule via
`compute_feed_pow()` avant publication iroh-docs. Nonce garanti
present dans le namespace. Test `test_feed_entry_roundtrip_json`
couvre le roundtrip avec `pow_nonce: Some(42)`.

### 2.1.2 ingest_doc_entry() rejette PoW invalide/manquant — PASS

`feed_sync.rs:160-171` : `None` → reject (line 168),
`Some(nonce)` → `verify_feed_pow()` → reject si invalide (line 163).
Tests `test_feed_pow_verification` et
`test_feed_pow_different_hashes_different_nonces` dans public_feed.rs.

### 2.1.3 ingest_doc_entry() rejette rate limit — PASS

`feed_sync.rs:203` : `if apply_rate_limit &&
!feed_limiter.check_author(...)` → reject. Dedup AVANT rate-limit
(line 173-197) pour eviter que le backfill historique consomme les
tokens GCRA. Tests `test_feed_rate_limiter_rejects_excess` et
`independent_authors` dans feed_limiter.rs.

### 2.1.4 Backfill checks — PASS (avec exemption intentionnelle)

`feed_sync.rs:547` : backfill appelle
`ingest_doc_entry(..., false)` — rate-limit desactive. PoW reste
enforce (ingest_doc_entry verifie PoW inconditionnellement, lines
160-171). L'exemption rate-limit est intentionnelle
(commit 5d52b6c) : les entrees historiques ont deja ete validees
par le noeud d'origine et sont des faits accomplis. Le test
integration 6+ entries manque (voir P2-1).

### 2.1.5 import_and_subscribe atomique — PASS

`feed_sync.rs:485` : `docs_client.import_and_subscribe(ticket)`
appel unique retournant `(doc_handle, live_stream)`. Commentaire
line 483-484 documente explicitement la garantie d'atomicite.

---

## §3 Dimension 2.2 — PoW design

### 2.2.1 pow_nonce absent de FeedEntryCanonical — PASS

`public_feed.rs` : `FeedEntryCanonical` (lines ~94-100) ne contient
que version, op, author_pubkey, timestamp, prev_hash. `pow_nonce`
uniquement dans `FeedEntry` (transport-level). Commentaire lines
79-82 : "Not part of FeedEntryCanonical".

### 2.2.2 FEED_FORMAT_VERSION = 1 — PASS

`public_feed.rs:19` : `pub const FEED_FORMAT_VERSION: u16 = 1`.
Test `assert_eq!(FEED_FORMAT_VERSION, 1)` confirme. Pre-launch
policy respectee.

### 2.2.3 #[serde(default)] documente — PASS

`public_feed.rs:83` : `#[serde(default)]` sur `pow_nonce`.
Documentation lines 81-82 explique : "local entries omit it
(self-trust), remote sync enforces it". Runtime tolerance, pas
compat historique.

### 2.2.4 FEED_POW_DIFFICULTY = 16 — PASS

`public_feed.rs:138` : `pub const FEED_POW_DIFFICULTY: u32 = 16`.
Commentaire : "16 bits ~ 65k iterations ~ 10-50 ms". Coherent
avec P2P_THREATS.md §1.4 ("cout reel pour T1-T2").

---

## §4 Dimension 2.3 — Rate limiter isolation

### 2.3.1 Independance structurelle — PASS

`FeedRateLimiter` (feed_limiter.rs) : struct independant,
`DefaultKeyedRateLimiter<String>` keyed par `author_pubkey`,
quota 5 ops/min. `StorageWriteLimiter` (storage_limiter.rs) :
struct independant, keyed par `"{author}:{app}"`, quota 10/min.
Zero etat partage, modules distincts, governor instances distincts.

### 2.3.2 retain_recent() — PASS

Deux `tokio::spawn` independants avec `interval(Duration::from_secs(300))`
pour chaque limiter. Cadence 5 min conforme.

### 2.3.3 Quota 5/min coherent — PASS

`FEED_OPS_PER_MINUTE = 5` dans feed_limiter.rs. Coherent avec
kickoff D3 et P2P_THREATS.md §1.4. Usage normal ~1-2 ops/deploy,
headroom 5x.

---

## §5 Dimension 2.4 — Wire format stability

### 2.4.1 FeedEntry serde roundtrip — PASS

Test `test_pow_nonce_serde_default` : serialise avec
`pow_nonce: None`, deserialise JSON sans champ → `None`. Avec
`Some(42)` → roundtrip correct. Test
`test_feed_entry_roundtrip_json` dans feed_sync.rs confirme.

### 2.4.2 Backward compat — PASS

JSON sans pow_nonce deserialise en `None` grace a
`#[serde(default)]`. Pas de version bump necessaire.

### 2.4.3 Canonical bytes non impactes — PASS

`to_canonical()` exclut pow_nonce et seq. Seuls les champs signes
sont dans le canonical. Test `test_canonical_bytes_feed_deterministic`
confirme la stabilite.

---

## §6 Dimension 2.5 — Items carry

### P2-IMAGE-DEP image 0.25 (3/3 MANDATORY S63) — CONFIRME

`crates/nexus-launcher/Cargo.toml` : `image = "0.25"`. Footprint
~15 transitives tray icon. Pas sur chemin critique feed.

### P2-PLAYWRIGHT-REFACTOR (3/3 MANDATORY S63) — CONFIRME

`web/playwright.config.ts` : global-setup configure. Le fichier
`tests/global-setup.ts` (150 lignes) est complet (coordinator
subprocess + health check). Le probleme pyproject.toml
post-S50 persist (refactor complet requis).

### P2-G-1 exe lock intermittent — CONFIRME ACTIF

Pre-existant. Monitoring continu. Aucun fix S62.

### P2-FEED-INSERT-NO-AUTH-TIER (S64+) — CONFIRME

`feed_sync.rs:378` : `feed_insert` endpoint accepte toute requete
bearer-authentifiee. Pas de validation auth tier. Acceptable 2-3
noeuds pilotes.

### P2-FEED-SUBSCRIBE-JOINHANDLE — CONFIRME

`feed_sync.rs:280` (`spawn_feed_subscribe`) et line 541
(`feed_join`) : `tokio::spawn(...)` sans stockage du JoinHandle.
Si task panic, aucune detection. Pattern pre-existant (storage_api).

### P2-BACKFILL-6PLUS-TEST — CONFIRME GAP

Code fix 5d52b6c merge (backfill exempt rate-limit). Test
`test_feed_offline_catchup` dans multi_daemon.rs couvre 5 entries.
Aucun test avec 6+ entries meme auteur + rate-limit actif sous
gate SBFB_INTEGRATION=1.

---

## §7 Findings

### P2 findings (5)

**F1 P2-FEED-PUBLISH-ORPHAN** (nouveau)
`feed_sync.rs:396-433` : dans `feed_insert`, l'insertion SQLite
(`insert_feed_operation`) et la publication iroh-docs
(`publish_feed_entry_to_docs`) sont deux operations sequentielles
non-transactionnelles. Si la DB reussit mais iroh-docs echoue
(line 422-433), l'entree existe localement mais ne se propage
jamais. Le noeud local continue sa hash-chain (prev_hash pointe
vers l'entree orpheline), creant un gap invisible pour les noeuds
distants. Le commentaire line 80-83 documente cette limitation
(split DB/iroh-docs). Pattern herite de AppStorage.
Impact : faible (iroh-docs set = ecriture locale, echec improbable).
Carry S64+ avec le redesign transactionnel.

**F2 P2-SUBSCRIBE-STREAM-BREAK** (nouveau)
`feed_sync.rs:298-299` et `564-565` : quand le subscribe stream
rencontre une erreur, le handler `break` et le task se termine
avec un log `info!("feed subscribe ended")`. Aucune logique de
reconnexion. Le noeud cesse silencieusement de recevoir les
entrees feed distantes apres une erreur reseau transitoire.
Impact : desync silencieuse apres erreur reseau.
Carry S63+ (reconnexion subscribe avec backoff).

**F3 P2-BACKFILL-6PLUS-TEST** (confirme audit plan)
Code fix correct (5d52b6c). Preuve test integration manquante
pour 6+ entries meme auteur avec rate-limit actif. Carry S63.

**F4 P2-FEED-SUBSCRIBE-JOINHANDLE** (confirme audit plan)
JoinHandle non trackee dans `spawn_feed_subscribe` et `feed_join`.
Si task panic, pas de detection. Pattern pre-existant. Carry S63+.

**F5 P2-FEED-INSERT-NO-AUTH-TIER** (confirme audit plan)
Pas de validation auth tier sur `feed_insert` endpoint. Acceptable
2-3 noeuds. Carry S64+.

### P3 findings (2)

**F6 P3-COMMIT-COUNT-DISCREPANCY** (nouveau)
L'audit plan §3 indique "4 (A-D) + 8 fix = 12 commits". Le commit
stack reel montre 4 feat + 7 fix(feed) = 11 commits code. Le
commit `872c7c9 fix(feed): SourceBecameStale.reason whitelist`
(entre Phase A et B) n'est pas comptabilise dans le "8 fix" du
plan qui liste "3 + 3 + 2 = 8". Ecart mineur de comptage dans
les docs, pas dans le code.

**F7 P3-BACKFILL-ORDERING-IMPLICIT** (nouveau)
`feed_sync.rs:543` : `get_many_by_prefix("feed/")` retourne les
entries en ordre lexicographique par cle. L'ordre per-auteur est
correct grace au zero-padding du seq (`{seq:010}`), mais cette
garantie est implicite (pas de test dedie ni de commentaire).
Impact : aucun tant que le schema de cles ne change pas.

---

## §8 S61 P2 resolution check

Les 4 P2 critiques identifies dans l'audit S61 sont resolus :

| Item | Resolution | Verification |
|---|---|---|
| F2 P2-INCREMENTAL-NO-VERIFY | `materialize_incremental` appelle `verify_entry(entry)` per-entry (feed_materializer.rs:134) | RESOLU Phase A |
| F3 P2-VALIDATION-STRICTE | `validate_feed_operation` verifie hex-64, HTTPS URL, hex-40, hex-64, reason (public_feed.rs:197-235) | RESOLU Phase A |
| F4 P2-TRANSACTION-ATOMIQUE | `insert_feed_operation` wrappe dans `BEGIN IMMEDIATE / COMMIT` (public_feed.rs:252) | RESOLU Phase A |
| F6 P2-SPEC-TRUST-CONTRACT | Section trust model ajoutee dans PUBLIC_FEED_SPEC.md | RESOLU Phase A |

---

## §9 Gate scission check

Le gate de scission Phase C (D5) est verifie dans
`sprint62_phase_C_review.md` : 3/3 criteres PASS (offline catch-up,
replay idempotent, 2+ noeuds multi-daemon). Phase D procedait
legitimement.

---

## §10 Scope cuts check

Les 10 scope cuts du kickoff §7 sont tous respectes. Aucun scope
creep detecte dans le commit stack (4 feat ciblent exactement les
Phases A-D, les 7 fix sont des corrections post-review dans le
perimetre feed).

---

## §11 Carries actifs S63

| Item | Compteur | Classification |
|---|---|---|
| P2-IMAGE-DEP image 0.25 | 3/3 | **MANDATORY S63** |
| P2-PLAYWRIGHT-REFACTOR | 3/3 | **MANDATORY S63** |
| P2-G-1 exe lock intermittent | reouvert | carry S63+ monitoring |
| P2-FEED-INSERT-NO-AUTH-TIER | 1/3 | carry S64+ |
| P2-FEED-SUBSCRIBE-JOINHANDLE | 1/3 | carry S63+ |
| P2-BACKFILL-6PLUS-TEST | 1/3 | carry S63 (test manquant) |
| F1 P2-FEED-PUBLISH-ORPHAN | 1/3 | carry S64+ (redesign) |
| F2 P2-SUBSCRIBE-STREAM-BREAK | 1/3 | carry S63+ (reconnexion) |
| F1 P2-VERSION-NOT-STORED | 2/3 | carry S63+ |
| F5 P2-IROH-INFRA-TIMEOUT | 2/3 | carry S63+ |
| P2-A-1 rand blocker upstream | exemption | exemption externe |
| P2-AUDIT-2 iroh transitives | exemption | exemption externe |
