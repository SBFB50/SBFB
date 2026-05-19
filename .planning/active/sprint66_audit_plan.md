# Sprint 66 — Audit plan pour Sprint 65

**Sprint audite** : 65 — Contrat Public (Arc 1 Fondations, 1/2).
**Tip cloture attendu** : Phase D commit.
**Phase 0 S66 jouera cet audit AVANT la Phase A du Sprint 66.**

---

## Tracks d'audit

### Track A — Suites et compteurs

Verifier que les compteurs annonces dans le commit body Phase D
correspondent aux compteurs reels mesures par `cargo nextest run
--workspace --locked` et `npm run test:unit`. Verifier 0 test
`#[ignore]` ajoute. Verifier delta annonce Phase A (+7 Rust) et
Phase C (+3 Vitest).

### Track B — Securite feed + auth tier

Verifier que `feed_insert()` rejette sans `X-SBFB-Feed-Internal`
header (test `test_feed_insert_rejects_without_internal_header`).
Verifier que `verify_entry()` rejette `version != 1`. Verifier
LOOPBACK_ENDPOINTS_TRUST_TIERS.md documente feed_insert = T0
temporaire.

### Track C — Raw-op migration coherence

Verifier que `FeedEntry.op` est `serde_json::Value` dans le code.
Verifier canonical bytes : `test_canonical_bytes_value_vs_typed`
passe (determinisme JCS preservee). Verifier `try_parse_op()`
existe et gere les ops inconnues. Verifier
`FEED_FORMAT_VERSION = 1` inchange.

### Track D — Deploy→feed wiring

Verifier que `deploy.rs` insere `ReleasePublished` dans le feed
apres `publish_announcement()`. Verifier `repo_url` en `http://`
rejete (test + code). Verifier rollback path (test deploy failure
no feed entry).

### Track E — Badges UI + wording

Verifier 0 occurrence "Verifie" sans qualification dans `web/src/`.
Verifier 0 "open source" hors AGPL dans `web/src/`. Verifier
`scan-trust-wording.sh` passe. Verifier badge dynamique
BrowsedProject (3 etats). Verifier `scan-en-strings.sh` passe.

### Track F — Documents fondateurs

Verifier `docs/trust/TRUST_TAXONOMY.md` present avec 6 niveaux.
Verifier `COMMONS.md` present a la racine.
Verifier `docs/factory/FACTORY_GATES.md` present avec 11 gates
FG0-FG10. Verifier `docs/protocol/SBFB_JSON_V2.md` present avec
spec complete + exemples + strategie versioning.

### Track G — Dette pair / process

Verifier P2-COMMIT-TITLE-FORMAT : README.md §4.1 documente les
types valides (`feat|fix|docs|chore`).
Verifier P2-REVIEW-ORDER : README.md §4.3 documente l'ordre
review → Codex → commit.
Verifier P2-PYTHON-BLOCK-EXEMPTION : bloc Python commente/obsolete
dans le skill review.
Verifier P2-EXPLORER-ESCAPE-SINGLE-QUOTE : `escapeAttr()` escape
`'` dans `examples/sbfb-explorer/app.js`.
Verifier P2-PLAYWRIGHT-SPECS-STALE : 0 fichier `.spec.ts` dans
`web/tests/`, `playwright.config.ts` supprime.

### Track H — Scope cuts + carries

Verifier les 14 scope cuts kickoff §7 tous respectes (aucun item
scope-cut livre accidentellement). Verifier les 8 carries S66
documentes dans kickoff §6. Verifier les 2 items 3/3 MANDATORY
S66 : P2-PROVENANCE-404-BRIDGE et P2-VERIFY-LOCAL-KEY-ONLY.

### Track I — Hygiene process S65 (findings post-sprint)

Analyse ultra-profonde du Sprint 65 — 4 lacunes process
identifiees. Toutes [RESOLVED] par amendement process commite
avant S66 Phase A (hook Check 9 + README §4.2 enforcement +
template + agents body validation).

| ID | Sev | Description | Status |
|----|-----|-------------|--------|
| P2-S65-BODY-FORMAT | P2 | Commits Phase A (`ace05b0`), Phase B (`de9d55f`), Phase C (`54f13eb`) n'utilisaient pas les headers `##` canoniques prescrits par README §4.1 (8 sections obligatoires). Seule Phase D (`9727818`) etait conforme 8/8. | [RESOLVED] — Check 9 ajoute dans `phase-precommit-lightcheck.sh` (`62d8344`), enforcement documente dans README §4.2 (`9727818`). |
| P2-S65-G8-TRACEABILITY | P2 | Section `## G8 traceability` avec SHA du preflight absente des commits Phase A (`ace05b0`), Phase B (`de9d55f`), Phase C (`54f13eb`). Presente uniquement Phase D (`9727818`). | [RESOLVED] — Template body et agents orchestration (`cc8cf1e`) imposent la section G8 traceability. Check 9 valide sa presence. |
| P3-S65-CARRY-CLOSURE | P3 | Section `## Carry closure` absente des commits Phase B (`de9d55f`) et Phase C (`54f13eb`). | [RESOLVED] — Template body inclut `## Carry closure` parmi les 8 sections obligatoires. Check 9 bloque le commit si absente. |
| P3-S65-CODEX-C-PARTIAL | P3 | Codex Phase C = 2/3 livrables PARTIEL (badge fallback couleur non livre + scan-trust-wording scope `docs/` non couvert). Cf. commit planning `a2735a5`. | [RESOLVED] — Gate Codex renforcee dans skill review (`62d8344`). Le partiel est documente dans le commit body Phase C et dans le carry S66. |

**Impact S66** : l'audit gate S66 Phase 0 doit verifier que
Phase A du S66 utilise le format `##` des le premier commit feat.
Le hook Check 9 (`phase-precommit-lightcheck.sh`) bloque
automatiquement les commits non conformes — verifier que le hook
est actif et non bypass (`--no-verify` interdit).

**Verification croisee Track G** : Track G valide les items dette
pair P2 du sprint 65 (COMMIT-TITLE-FORMAT, REVIEW-ORDER, etc.).
Track I couvre les lacunes process du commit body qui n'etaient
pas des items dette pair explicites mais des non-conformites
constatees a posteriori. Les deux tracks sont complementaires.

---

## Verdicts attendus

| Verdict | Condition |
|---------|-----------|
| PASS | 0 P0, 0 P1, >= 1 P2+ documente |
| CONDITIONAL PASS | 0 P0, 1-2 P1 fixables dans la session |
| FAIL | >= 1 P0 OU >= 3 P1 |
