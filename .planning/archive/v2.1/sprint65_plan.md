# Sprint 65 — Plan (Contrat Public)

**Ecrit** : 2026-05-18.
**Tip master** : `b7469ae`.
**Roadmap** : Sprint 1/11, v2.1 Arc 1 Fondations.

---

## §1 Etat verifie a l'entree

| Suite | Count | Commande |
|---|---|---|
| Rust nextest | 1326 | `cargo nextest run --workspace --locked` |
| Rust doctests | ok | `cargo test --workspace --locked --doc` |
| cargo fmt | 0 diff | `cargo fmt --all --check` |
| cargo clippy | 0 warnings | `cargo clippy --workspace --all-targets --locked -- -D warnings` |
| Vitest | 265 | `(cd web && npm run test:unit)` |
| size-limit | 6/6 | `(cd web && npm run size)` |
| release build | ok | `cargo build -p nexus-shell-daemon --release` |
| **Total** | **~1597** | |

---

## §2 Decisions Day 0 (gelees)

| D# | Decision | Implication code |
|---|---|---|
| D1 | Feed raw-op : PublicFeedOperation → serde_json::Value | `public_feed.rs`, `feed_materializer.rs`, `feed_sync.rs` |
| D2 | Taxonomie confiance 6 niveaux | `docs/trust/TRUST_TAXONOMY.md` |
| D3 | Vocabulaire "source verifiable" enforcement | `scripts/scan-trust-wording.sh`, UI pages |
| D4 | Deploy→feed wiring + auth tier (X-SBFB-Feed-Internal) | `deploy.rs`, `feed_sync.rs`, `http.rs` |
| D5 | Factory gates FG0-FG10 + SBFB.json v2 spec | `docs/factory/`, `docs/protocol/` |

---

## §3 Graphe de dependances inter-phases

```
Phase A (securite + raw-op + docs) — standalone
Phase B (badges UI)              — depend de A (TRUST_TAXONOMY.md defini)
Phase C (badge dynamique + scan) — depend de B (badges migres)
Phase D (gates + dette + wrap-up) — depend de C (scan-trust-wording cree)
```

Phase A est standalone. B depend de la taxonomie definie en A.
C depend des badges migres en B pour le comportement dynamique.
D depend de C pour integrer scan-trust-wording dans la CI finale.

---

## §4 Phase A — Securite feed + raw-op + TRUST_TAXONOMY

### §4.1 Scope

Phase A resout le MANDATORY P2-FEED-INSERT-NO-AUTH-TIER, migre
le feed vers raw-op, wire deploy→feed, et ecrit les documents
fondateurs. C'est la phase la plus dense du sprint.

### §4.2 Livrables

**L1 — Auth tier feed_insert (MANDATORY P2-FEED-INSERT-NO-AUTH-TIER)**

Fichier : `crates/nexus-shell-daemon/src/feed_sync.rs`

En debut de `feed_insert()` (l.445), ajouter un guard :
```rust
// Reject external callers — only internal daemon code paths
// (deploy, bridge) set this header.
let internal = headers
    .get("x-sbfb-feed-internal")
    .and_then(|v| v.to_str().ok())
    == Some("1");
if !internal {
    return (StatusCode::FORBIDDEN, Json(serde_json::json!({
        "error": "feed insert requires internal auth"
    }))).into_response();
}
```

Fichier : `crates/nexus-shell-daemon/src/http.rs`

Sur la route interne (deploy→feed), injecter le header
`X-SBFB-Feed-Internal: 1` dans l'appel. Si deploy.rs appelle
`feed_insert` directement via le coordinator (pas HTTP), le guard
s'applique uniquement sur la route HTTP.

**L2 — Version guard verify_entry (P2-VERIFY-ENTRY-VERSION-GUARD)**

Fichier : `crates/nexus-coordinator-rs/src/public_feed.rs`

En debut de `verify_entry()` ou au debut du path de verification :
```rust
if entry.version != FEED_FORMAT_VERSION {
    return Err(format!(
        "unsupported feed version {}, expected {}",
        entry.version, FEED_FORMAT_VERSION
    ));
}
```

**L3 — Migration raw-op FeedEntry.op → serde_json::Value**

Fichier : `crates/nexus-coordinator-rs/src/public_feed.rs`

1. `FeedEntry.op` : `PublicFeedOperation` → `serde_json::Value`
2. `FeedEntryCanonical.op` : idem
3. Ajouter `pub fn try_parse_op(op: &serde_json::Value) -> Option<PublicFeedOperation>` :
   tente le parse, renvoie `None` pour les ops inconnues.
4. Ajouter `pub fn op_type(op: &serde_json::Value) -> Option<&str>` :
   extrait `op["op_type"].as_str()`.
5. `validate_feed_operation()` : si `try_parse_op` retourne `Some`,
   valider comme avant. Si `None`, accepter l'op comme unknown
   (seule contrainte : MAX_OPERATION_JSON_SIZE).
6. `insert_feed_operation_inner()` : adapter pour prendre un
   `serde_json::Value` au lieu de `PublicFeedOperation`. Extraire
   `op_type` pour le FeedEntryRow. Serialiser le payload complet.
7. `to_canonical()` : copier le `Value` tel quel dans
   FeedEntryCanonical.

Fichier : `crates/nexus-coordinator-rs/src/feed_materializer.rs`

`rows_to_entries()` (l.189) : remplacer
`serde_json::from_str::<PublicFeedOperation>` par
`serde_json::from_str::<serde_json::Value>`. Reconstruire
le Value complet (re-injecter `op_type` dans le Value si
stocke separement dans la colonne `op_type`).

Fichier : `crates/nexus-shell-daemon/src/feed_sync.rs`

`ingest_doc_entry()` : adapter le parsing pour
`serde_json::Value` au lieu de `PublicFeedOperation`.

**L4 — PUBLIC_FEED_SPEC.md §9 update**

Fichier : `docs/protocol/PUBLIC_FEED_SPEC.md`

Ajouter section §9.1 "Forward Compatibility" :
- "Adding a new operation type is NOT a breaking change"
- Nodes MUST store and propagate unknown op_types
- Nodes MUST verify hash-chain and signature for unknown ops
- Nodes MUST NOT interpret or act on unknown ops
- `FEED_FORMAT_VERSION` bump only when envelope structure changes

**L5 — TRUST_TAXONOMY.md**

Fichier : `docs/trust/TRUST_TAXONOMY.md`

6 niveaux + 3 dimensions transversales. Chaque niveau porte :
- Label court (pour UI)
- Assertion positive (ce que le niveau garantit)
- Non-assertion explicite (ce qu'il ne garantit PAS)
- Condition technique (comment le niveau est atteint)
- Verification (comment un tiers peut verifier)

**L6 — COMMONS.md**

Fichier : `COMMONS.md` (racine)

Convention anti-capture du projet :
- License AGPL-3.0 (OSI, copyleft reseau)
- Pas de CLA, pas de fondation, pas de governance token
- Pattern OpenBSD solo maintainer
- Apps du reseau = "source verifiable" (pas "open source")
- Fork encouraged, contributions bienvenues

**L7 — Deploy→feed wiring**

Fichier : `crates/nexus-shell-daemon/src/deploy.rs`

Apres `publish_announcement()` (l.237-250), inserer :
1. Construire `ReleasePublished` payload depuis les memes
   donnees que l'annonce (project_id, repo_url, commit_sha,
   artifact_hash, is_open_source, provenance_hash, app_version)
2. Appeler `insert_feed_operation()` via le coordinator
3. Si insert echoue : log warning, ne PAS rollback l'annonce
   gossip (fire-and-forget). Le feed entry manquant sera
   detectable par verify_chain.

Egalement : rejeter `repo_url` en `http://` dans le deploy
handler (l.72 : `starts_with("http")` → `starts_with("https://")`).

**L8 — Tests**

| Test | Fichier | Vecteur |
|---|---|---|
| test_feed_insert_rejects_without_internal_header | feed_sync.rs ou tests/ | auth tier reject 403 |
| test_verify_entry_rejects_wrong_version | public_feed.rs | version guard |
| test_unknown_op_roundtrip | public_feed.rs | unknown op stored + verified |
| test_canonical_bytes_value_vs_typed | public_feed.rs | determinisme JCS |
| test_deploy_inserts_release_published | deploy.rs | wiring deploy→feed |
| test_deploy_rejects_http_repo_url | deploy.rs | http:// reject |
| test_deploy_failure_no_feed_entry | deploy.rs | rollback path |

Cible : +7 Rust minimum.

### §4.3 Critere d'acceptation

- `feed_insert()` retourne 403 sans header interne
- `verify_entry()` rejette version != 1
- Unknown op stockee, propagee, hash-chain verifiee
- Canonical bytes identiques pour Value et struct typee
- Deploy→ReleasePublished auto-insere dans le feed
- Deploy rejette http:// repo_url
- TRUST_TAXONOMY.md present avec 6 niveaux
- COMMONS.md present

---

## §5 Phase B — Migration badges UI

### §5.1 Scope

Migrer tous les textes UI vers la nomenclature TRUST_TAXONOMY.md.
Pas de nouveau composant — modifications de textes et d'icones.

### §5.2 Livrables

**L1 — Browse.tsx**

- `"Verifie"` + `ShieldCheck` → `"Provenance"` + `FileCheck`
- Condition inchangee : `entry.provenance_hash` present

**L2 — BrowsedProject.tsx**

- Badge : `"Verifie"` → `"Provenance"` + `FileCheck`
- `"Auto-publie"` → `"Upload direct"` (N0)
- Ajout note : "Provenance auto-attestee (SLSA L1)" sous le badge

**L3 — GpuConsentDialog.tsx**

- L2 : `"Projets open source verifies"` →
  `"Apps deployees depuis un depot public (provenance auto-attestee)"`
- L2 description : adapter coherence taxonomie

**L4 — Network.tsx**

- `"L2 -- Open source"` → `"L2 -- Depot public"`

**L5 — Curators.tsx**

- `"curator de confiance"` → `"curator"` (retirer "de confiance")

**L6 — Protocol Explorer (app.js)**

- `"Le code sur le reseau = le code du depot"` →
  `"L'archive reseau est construite depuis le depot source par le noeud local. C'est une auto-attestation."`
- `"Le modele F-Droid/Linux"` →
  `"Inspire par F-Droid -- les apps publiques sont deployees depuis leur code source."`
- `"Chaine de preuve"` → `"Chaine de provenance"`
- `"Open source par construction"` → `"Source verifiable par construction"`
- Footer : `"open source deployee"` → `"a source verifiable deployee"`

**L7 — PUBLISH_MODEL.md**

- `"open source verifie"` → `"Release avec provenance auto-attestee"`
- Tous les etats du tableau §3 adaptes

**L8 — Tests existants mis a jour**

Adapter les assertions de texte dans :
- `BrowsedProject.test.tsx`
- `VerificationDetail.test.tsx`
- Tout test qui matche "Verifie" ou "open source"

Cible : +0 nouveau test (modifications assertions), possible
+2-3 si nouvelles assertions ajoutees.

### §5.3 Critere d'acceptation

- 0 occurrence "Verifie" sans qualification dans Browse/BrowsedProject
- 0 "open source" hors contexte AGPL dans l'UI
- 0 "de confiance" dans un contexte automatique
- scan-en-strings.sh clean (textes francais preserves)
- Vitest vert

---

## §6 Phase C — Badge dynamique + scan-trust-wording

### §6.1 Scope

Le badge "Provenance" dans BrowsedProject passe dynamiquement
a "Signature verifiee" (vert) ou "Echoue" (rouge) apres
verification API auto. Script CI non-regression wording.

### §6.2 Livrables

**L1 — Badge dynamique BrowsedProject**

Fichier : `web/src/pages/BrowsedProject.tsx`

1. A l'ouverture de la page projet, appel automatique
   `provenance_verify` via le bridge ou l'API daemon
2. Etats du badge :
   - Initial : "Provenance" (neutre, avant verification)
   - Transitoire : "Verification..." (spinner, pendant l'appel)
   - Succes : "Signature verifiee" (vert, FileCheck)
   - Echec : "Verification echouee" (rouge, AlertTriangle)
3. Cache du resultat dans le state React (session, pas
   localStorage) pour eviter les appels repetitifs

**L2 — scan-trust-wording.sh**

Fichier : `scripts/scan-trust-wording.sh`

```bash
#!/usr/bin/env bash
# Scan UI and public docs for trust-related wording violations.
# Exit 1 if any forbidden pattern found.
set -euo pipefail

VIOLATIONS=0
# ... grep patterns for forbidden terms
# ... whitelist for legitimate uses
exit $VIOLATIONS
```

Patterns scannes :
- `"Verifie"` sans "Signature" devant dans web/src/ et examples/
- `"open source"` sans "AGPL" context dans web/src/ et docs/
- `"de confiance"` dans web/src/ (sauf Curators contexte humain)
- `"Le code sur le reseau"` dans web/src/ et examples/

Whitelist : strings dans fichiers de test, SPRINT_LOG.md,
archive/ (historique).

**L3 — Tests**

- Test Vitest badge dynamique : mock provenance_verify → success,
  verify text change
- Test Vitest badge echec : mock provenance_verify → failure,
  verify red state
- Test Vitest etat transitoire

Cible : +3 Vitest minimum.

### §6.3 Critere d'acceptation

- Badge change dynamiquement apres verification
- Etat transitoire visible
- scan-trust-wording.sh passe sans faux positif
- Cache session fonctionne (pas d'appel a chaque re-render)

---

## §7 Phase D — Gates Factory + dette pair + wrap-up

### §7.1 Scope

Livrables documentaires pour Factory. Dette pair process.
Wrap-up sprint complet.

### §7.2 Livrables

**L1 — FACTORY_GATES.md**

Fichier : `docs/factory/FACTORY_GATES.md`

11 gates avec pour chaque :
- ID (FG0-FG10)
- Nom
- Description (2-3 lignes)
- Input (ce que la gate recoit)
- Output (ce que la gate produit)
- Critere de passage
- Sprint d'implementation cible (S67/S68/S69)

**L2 — SBFB_JSON_V2.md**

Fichier : `docs/protocol/SBFB_JSON_V2.md`

Spec du manifest v2 :
- `schema_version: 2`
- Champs requis : name, display_name, description
- Champs optionnels : category, license, lang, bridge.methods,
  bridge.events, tech.type, tech.build_command, requirements
- Retro-compatibilite v1 : tous les nouveaux champs optionnels,
  parser accepte v1 et v2
- Exemples JSON complets (v1 minimal, v2 complet)

**L3 — P2-COMMIT-TITLE-FORMAT**

Clarification dans `docs/claude/README.md` ou hook :
format exact `feat|fix|docs|chore(scope): Sprint N Phase X — titre`.

**L4 — P2-REVIEW-ORDER**

Doc amendment dans `docs/claude/README.md` : clarifier l'ordre
review → codex → commit.

**L5 — P2-PYTHON-BLOCK-EXEMPTION**

Reclassification : resolved. Le pivot S50 a supprime tout le
code Python. La clause Python dans les skills est obsolete.
Supprimer ou marquer comme obsolete dans le SKILL.md concerne.

**L6 — P2-EXPLORER-ESCAPE-SINGLE-QUOTE**

Fichier : `examples/sbfb-explorer/app.js`

1 LOC : ajouter `'` escape dans la fonction `escapeAttr()`.

**L7 — P2-PLAYWRIGHT-SPECS-STALE**

Supprimer les fichiers Playwright zombies :
- `web/playwright.config.ts`
- Tout fichier `.spec.ts` residuel
- Tout fixture Playwright non utilise

**L8 — verification.md**

Fichier : `.planning/active/sprint65_verification.md`

Fail-fast checklist complète avec toutes les rows.

**L9 — sprint66_audit_plan.md**

Fichier : `.planning/active/sprint66_audit_plan.md`

Plan d'audit pour S65 (7-8 tracks).

**L10 — Mise a jour CLAUDE.md + SPRINT_LOG.md**

Mettre a jour les compteurs, carries, etat du projet.

### §7.3 Critere d'acceptation

- FACTORY_GATES.md present avec 11 gates
- SBFB_JSON_V2.md present avec spec complete
- 5 items dette fermes (COMMIT-TITLE-FORMAT, REVIEW-ORDER,
  PYTHON-BLOCK-EXEMPTION, EXPLORER-ESCAPE-SINGLE-QUOTE,
  PLAYWRIGHT-SPECS-STALE)
- Fail-fast checklist verte
- scan-trust-wording.sh + scan-en-strings.sh clean

---

## §8 Delta tests estime

| Phase | Rust | Vitest | Detail |
|---|---|---|---|
| A | +7 minimum | +0 | auth tier, version guard, raw-op, deploy wiring |
| B | +0 | +0 (modifications) | assertions texte mises a jour |
| C | +0 | +3 minimum | badge dynamique etats |
| D | +0 | +0 | docs only + 1 LOC fix |
| **Total** | **+7** | **+3** | |
| **Sortie estimee** | **1333** | **268** | **~1607** |

---

## §9 Fail-fast checklist (template verification.md)

| # | Check | Commande | Critere |
|---|---|---|---|
| 1 | cargo fmt | `cargo fmt --all --check` | 0 diff |
| 2 | cargo clippy | `cargo clippy --workspace --all-targets --locked -- -D warnings` | 0 warnings |
| 3 | cargo nextest | `cargo nextest run --workspace --locked` | >= 1333 |
| 4 | cargo doctests | `cargo test --workspace --locked --doc` | ok |
| 5 | release build | `cargo build -p nexus-shell-daemon --release` | ok |
| 6 | npm lint | `(cd web && npm run lint)` | 0 errors |
| 7 | tsc | `(cd web && npx tsc --noEmit -p tsconfig.app.json)` | 0 errors |
| 8 | Vitest | `(cd web && npm run test:unit)` | >= 268 |
| 9 | npm build | `(cd web && npm run build)` | ok |
| 10 | size-limit | `(cd web && npm run size)` | 6/6 |
| 11 | scan-en-strings | `(cd web && bash scripts/scan-en-strings.sh)` | clean |
| 12 | scan-trust-wording | `bash scripts/scan-trust-wording.sh` | clean |
| 13 | sync-bridge-sdk | diff sbfb-bridge.js copies | identical |
| 14 | auth tier reject | `cargo nextest run -E 'test(feed_insert_rejects)' -p nexus-shell-daemon` | PASS |
| 15 | raw-op roundtrip | `cargo nextest run -E 'test(unknown_op)' -p nexus-coordinator-rs` | PASS |
| 16 | deploy→feed wire | `cargo nextest run -E 'test(deploy_inserts_release)' -p nexus-shell-daemon` | PASS |
| 17 | TRUST_TAXONOMY.md | `test -f docs/trust/TRUST_TAXONOMY.md` | exists |
| 18 | COMMONS.md | `test -f COMMONS.md` | exists |
| 19 | FACTORY_GATES.md | `test -f docs/factory/FACTORY_GATES.md` | exists |
| 20 | SBFB_JSON_V2.md | `test -f docs/protocol/SBFB_JSON_V2.md` | exists |
| 21 | 0 "Verifie" sans qualif | `scan-trust-wording.sh` row | clean |
| 22 | Badge dynamique | Vitest badge states | PASS |
| 23 | Playwright zombies | `! test -f web/playwright.config.ts` | supprime |
