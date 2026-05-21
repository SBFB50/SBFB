# Sprint 68 Phase D — deep review

HEAD: a201b3e (Phase C) | Working tree: +2 new, 2 modified | Agent: nexus-phase-review-deep (Opus 4.6 1M)

## Verdict : PASS

Promu de PASS-PENDING apres reconciliation Codex GPT 5.5 (4/4 CONFIRME, 0 GAP).

(Rigor signal : 3 findings P2+ documentes / >=1 requis pour PASS)

## Memory consultation

- `feedback_approach.md` : pick deepest, research before code, OSS prior art — N/A (UI composant, pas nouvelle lib). Respecte.
- `feedback_context7_systematic.md` : context7 avant tout code touchant lib/API. Phase D ne touche aucune nouvelle dep (React, lucide-react, shadcn deja en place) — N/A.
- `vision_model.md` : aucune tension — Phase D ne touche ni funding ni gouvernance.
- `feedback_kudos_non_monetary.md` : ProofCard affiche "confidence score" 0-100, PAS kudos. Aucun champ kudos. N/A.
- `fairness_vision.md` : N/A — ProofCard mesure completude preuve projet, pas contribution worker. Aucune tension.
- `nexus_grid_pivot.md` : D5 gelee "Proof Card UI composant shell Browse" confirmee. D1 formule gelee (FORMULA_VERSION=1). Coherent.

## Staging check

- Phase fichiers : 4 (ProofCard.tsx NEW, ProofCard.test.tsx NEW, BrowsedProject.tsx MOD, THREAT_MODEL.md MOD)
- Planning/docs split : preflight.md untracked — devra etre stage dans le commit ou un chore. Le preflight est un artefact planning, pas du code Phase D. Mix acceptable si stage ensemble.
- Untracked accidentels : 0 (les 2 untracked sont des livrables Phase D)

## Suites verification

| Suite | Avant | Apres | Delta | Status |
|-------|-------|-------|-------|--------|
| cargo fmt | - | - | - | ok |
| cargo clippy | - | - | - | ok |
| Rust nextest | 1419 | 1419 | +0 | ok (aucun Rust modifie) |
| Rust doctests | ok | ok | | ok |
| tsc --noEmit | - | - | - | ok (0 errors) |
| ESLint | - | - | - | ok (0 errors, 5 warnings connus T1) |
| Vitest | 270 | 279 | +9 | ok |
| Build web | - | - | - | ok |
| size-limit | 6/6 | 6/6 | - | ok |
| scan-en-strings | - | - | - | ok (clean) |
| Release build daemon | - | - | - | ok |

Vitest delta : +9 (plan prevoyait +4, livre +9 — 8 tests ProofCard.test.tsx + 1 additionnel). Pas de test skip/ignore.

## Branch coverage semantique (deep)

| Element | Test | Appel reel | Assert specifique | Cas limites | Signal |
|---------|------|------------|-------------------|-------------|--------|
| `ProofCard({card, loading})` loading=true | `renders loading state` | oui (render + getByTestId) | oui (proof-card-loading present) | N/A | DEEP-PASS |
| `ProofCard({card: null})` | `renders nothing when card is null` | oui | oui (container.firstChild null) | edge case | DEEP-PASS |
| `ProofCard({card})` score render | `renders the confidence score` | oui | oui ("100/100" text) | N/A | DEEP-PASS |
| `ProofCard({card})` expand/collapse | `expands on click and collapses` | oui (2 clicks) | oui (details present/absent) | toggle cycle | DEEP-PASS |
| `ProofCard` layers render | `renders evidence layers when expanded` | oui (click+getByTestId) | oui (5 layers verified) | all layers | DEEP-PASS |
| `ProofCard` risk factors visible | `shows risk factors when present` | oui | oui (2 factors, text content) | true+false | DEEP-PASS |
| `ProofCard` risk factors absent | `does not show risk factors section` | oui | oui (queryByTestId null) | N/A | DEEP-PASS |
| `ProofCard` risk level badge | `shows risk level badge in expanded view` | oui | oui ("Risque Moyen" text) | medium level | DEEP-PASS |
| `buildLayers()` source layer | implicit via layers test | oui (layer-source tested via 6th layer when repo_url present) | partiel | N/A | SHALLOW-PASS |
| `scoreColor()` / `scoreBgColor()` thresholds | non teste directement | implicit via render | non | 3 thresholds | PARTIAL P3 |
| `BrowsedProject.tsx` proofCardQuery integration | non teste (composant page, pas de test page) | N/A | N/A | N/A | WIRING-UNTESTED P2 |

## Scope cuts semantique (deep)

| # | Libelle | Intention | Grep mecanique | Diff semantique | Signal |
|---|---------|-----------|----------------|-----------------|--------|
| 1 | SearchManifest wire format | Pas de gossip wire pour le score | 0 match | 0 code reseau | CLEAN |
| 2 | Page React /factory | Pas de route React factory | 0 match | 0 route | CLEAN |
| 3 | Babel dogfood via Factory | Pas de creation app Babel | 0 match | 0 app | CLEAN |
| 4 | @dev index tree-sitter | Pas de treesitter | 0 match | 0 code | CLEAN |
| 5 | Template react-vite | Pas de template | 0 match | 0 code | CLEAN |
| 6 | Factory audit log JSONL | Pas de JSONL | 0 match | 0 code | CLEAN |
| 7 | CuratorVouched UI shell | Pas d'UI vouch | 0 match | 0 code vouch | CLEAN |
| 8 | FG8 Provenance Ed25519 | Pas de factory provenance | 0 match | 0 code | CLEAN |
| 9 | FG9 Publish gate complete | Pas de publish gate | 0 match | 0 code | CLEAN |
| 10 | FG10 Review gate | Pas de review gate | 0 match | 0 code | CLEAN |
| 11 | Fuzzing cargo-fuzz/proptest | Pas de fuzz | 0 match | 0 code | CLEAN |
| 12 | Feed format version bump | Pas de version bump | 0 match | 0 code | CLEAN |
| 13 | ProofCard comme feed op | Pas de feed op | 0 match | 0 code reseau | CLEAN |
| 14 | Diff engine avance | Pas de diff semantique | 0 match | 0 code | CLEAN |

## Research grounding (deep)

### Preflight G8

- Fichier : `sprint68_phase_d_preflight.md` existe (untracked)
- Scans : 5/5 (S1a, S1b, S2, S3, S4)
- S1a OSS : 5 projets analyses (OpenSSF Scorecard Visualizer, F-Droid Verification, Sigstore/cosign, VerificationDetail SBFB interne, npms.io)
- Verdict : EXECUTE plan-as-is
- PASS

### Deps/API

Phase D ne touche aucune dep. 0 delta Cargo.toml, 0 delta package.json.

### Coherence code-vs-source

ProofCardData TypeScript (ProofCard.tsx:23-43) vs ProofCard Rust (proof_card.rs:40-51) :

| Champ TS | Champ Rust | Coherent |
|----------|------------|----------|
| `project_id: string` | `project_id: String` | oui |
| `project_name: string` | `project_name: String` | oui |
| `hash: {archive_hash, provenance_hash}` | `hash: ProofCardHash` | oui |
| `license: {spdx, source}` | `license: ProofCardLicense` | oui |
| `freshness: {last_verified_at, age_days, state}` | `freshness: ProofCardFreshness` | oui |
| `provenance: {verified, repo_url, commit_sha, slsa_level}` | `provenance: ProofCardProvenance` | oui |
| `risk: {level, factors}` | `risk: ProofCardRisk` | oui |
| `curation: {curator_count, curator_names}` | `curation: ProofCardCuration` | oui |
| `confidence: number` | `confidence: u8` | oui |
| `formula_version: number` | `formula_version: u32` | oui |

Tous les champs correspondent. Les enums (freshness state, risk level) aussi (`fresh/aging/stale/unknown`, `low/medium/high`) grace a `#[serde(rename_all = "snake_case")]` cote Rust.

**Observation** : `ProofCardData` est un type TypeScript declare dans ProofCard.tsx sans schema Zod. Le plan prevoyait un schema Zod dans `protocol.ts`. La Phase A a livre le schema Zod + dispatch bridge. Phase D utilise `authFetch` + cast `as ProofCardData` dans BrowsedProject.tsx au lieu du bridge `proof_card_get`. C'est acceptable car la page consomme directement l'API daemon (P9 pattern — coordinator proxy), mais cree une duplication : la methode bridge `proof_card_get` (Phase A) ET le fetch direct (Phase D) font la meme chose. Le bridge est pour les iframes sandbox, le fetch direct est pour le shell — les deux chemins sont distincts et justifies.

## Security deep

### Scan automatique

| Fichier | Pattern | Ligne | Severite | Detail |
|---------|---------|-------|----------|--------|
| ProofCard.tsx | - | - | - | 0 unwrap, 0 dangerouslySetInnerHTML, 0 eval, 0 innerHTML |
| BrowsedProject.tsx | - | - | - | encodeURIComponent sur project_id (L189), ok |
| THREAT_MODEL.md | - | - | - | doc only |

### Analyse semantique

- **Inputs non-trustes** : `ProofCardData` provient de l'API daemon via `authFetch`. Le daemon est local (bearer auth). Les strings `risk.factors[]` et `curation.curator_names[]` sont rendues dans JSX (`{variable}`) — React auto-echappe.
- **Pas de dangerouslySetInnerHTML** dans tout le frontend (verifie grep global 0 matches).
- **encodeURIComponent** utilise pour `project_id` dans l'URL (BrowsedProject.tsx:189) — correct, previent l'injection de path.
- **staleTime: 60_000, retry: 1** pour proofCardQuery — raisonnable, pas de flood possible.
- **Pas de XSS via RISK_FACTOR_LABELS** : la map est une constante locale avec des strings francaises hardcodees. Un `factor` inconnu est rendu via `{RISK_FACTOR_LABELS[factor] ?? factor}` — le fallback affiche la string brute mais React l'echappe automatiquement.

## Livrable verification (Claude pre-Codex, ne remplace pas Codex)

| # | Livrable | Statut | Fichier:ligne | Evidence |
|---|----------|--------|---------------|----------|
| 1 | ProofCard.tsx composant carte expandable | CONFIRME | `web/src/components/ProofCard.tsx:147-273` | Composant React export, badge score + expanded card + layers + risk factors |
| 2 | Integration BrowsedProject.tsx | CONFIRME | `web/src/pages/BrowsedProject.tsx:184-196,360-363` | useQuery proof-card + `<ProofCard card={...} loading={...} />` dans top bar |
| 3 | THREAT_MODEL.md §12 T-PROOFCARD-FORMULA-GAME | CONFIRME | `docs/security/THREAT_MODEL.md:605-659,681-685` | Section complete (4 vecteurs, mitigations, table Dimension/Valeur), renumerotation §12→§13, historique v5 |
| 4 | Tests Vitest ProofCard | CONFIRME | `web/src/components/__tests__/ProofCard.test.tsx:1-123` | 8 tests couvrant : score, layers, expand/collapse, risk factors visibles/absents, loading, null, risk badge |

Resume : 4 livrables / 4 confirmes / 0 gaps / 0 partiels

## Patterns drift + horizon long-terme

### Patterns

- P1 (typed fetch path) : Phase D utilise `authFetch` dans BrowsedProject, coherent avec le pattern existant (`verifyQuery` quelques lignes plus bas fait pareil). `ProofCard.tsx` ne fait aucun fetch — le composant recoit ses props. Respecte.
- P4 (React Query only cache) : Phase D ajoute un `useQuery` pour proofCardQuery. Respecte.
- P24 (postMessage bridge) : `proof_card_get` est dans le bridge (Phase A), Phase D utilise le fetch direct shell-side. Pas de conflit — ce sont 2 chemins clients pour 2 consommateurs differents (iframe vs shell).

### Horizon long-terme

- Design doc present : ProofCard est documentee dans SYNTHESIS §4.6 + kickoff §D5 + THREAT_MODEL §12. OK.
- D1..D5 avec alternatives + rationale : kickoff §D5 cite 3 alternatives rejetees (app sbfb-search separee, score badge seul, cache persistant). OK.
- Solution la plus poussee : composant expandable avec couches detaillees (pas juste un badge score). Alignee avec OpenSSF Scorecard Visualizer pattern. OK.
- Aucune LOC estimee au plan : plan §7 et §9 ne contiennent pas d'estimation LOC pour Phase D. OK.

## Commit body validation

### Titre

Le plan prevoit : `feat(shell): Sprint 68 Phase D — Proof Card UI + Browse integration`
Format regex `(feat|fix|docs|chore|test)\((sprint[0-9]+|[a-z_+-]+)\): Sprint [0-9]+ Phase [A-Z] — .+` — MATCH.

### 9 sections body

Draft body non fourni — verification differee au moment de la redaction. CONCERN "draft-body-absent". Le template `.claude/templates/commit_body_phase.txt` doit etre utilise.

### Co-Authored-By

Verification differee (body pas encore ecrit).

## Findings

- **P2-D-1** : `BrowsedProject.tsx proofCardQuery` wiring non teste. Le composant `FullScreenApp` ajoute un `useQuery` pour proof-card et passe les donnees a `<ProofCard>`, mais aucun test de `BrowsedProject.tsx` n'exerce ce chemin. Le composant `ProofCard` est unitairement teste (8 tests), mais le wiring dans la page n'est pas couvert par un test d'integration — `web/src/pages/BrowsedProject.tsx:184-196`. Direction fix : ajouter un test dans un fichier `BrowsedProject.test.tsx` qui mock `authFetch`, verifie que `ProofCard` est rendu avec les bonnes props. Acceptable comme carry S69 si non resolu avant commit.

- **P2-D-2** : `ProofCardData` type declare localement dans `ProofCard.tsx:17-43` sans schema Zod. Le type est utilise en cast direct `(await resp.json()) as ProofCardData` dans `BrowsedProject.tsx:193`. Pas de validation runtime des donnees daemon. Si le daemon retourne un shape inattendu (champ manquant, type incorrect), le composant crasherait silencieusement au runtime. Direction fix : utiliser le schema Zod ProofCard deja defini dans `protocol.ts` (Phase A) pour parser la reponse, ou au minimum re-exporter le type depuis un endroit partage. Le bridge `proof_card_get` (Phase A) passe par le schema Zod — seul le fetch direct shell-side n'est pas valide.

- **P2-D-3** : THREAT_MODEL §12 documente uniquement T-PROOFCARD-FORMULA-GAME mais ne documente pas le vecteur V3 (XSS via risk_factors/curator_names) identifie dans le preflight S3. Le preflight classifie ce vecteur comme "L, mitigue par React auto-escaping" — c'est vrai, mais le THREAT_MODEL devrait le mentionner explicitement comme mitigation documentee, pas laisser le lecteur deviner que React echappe les strings. Direction fix : ajouter une note sous T-PROOFCARD-FORMULA-GAME : "T-PROOFCARD-XSS : mitigue par React JSX auto-escaping + CSP connect-src none sur iframe, N/A pour le shell."

## Codex reconciliation

- Status : FAIT
- Rapport Codex : sprint68_phase_d_codex_review.md (output brut GPT 5.5)
- Livrables : 4/4 CONFIRME, 0 GAP, 0 PARTIEL
- GAPs P0/P1 : 0
- P2 documentes dans body : P2-D-1, P2-D-2, P2-D-3 (carry S69)
- Suites relancees post-correction trust-wording : toutes vertes

## Dimensions explored (evidence audit exhaustif)

| Dimension | Commandes executees | Fichiers lus | Findings |
|-----------|---------------------|--------------|----------|
| Security | grep dangerouslySetInnerHTML (0 match global), grep unwrap/eval/innerHTML (0 match ProofCard), encodeURIComponent verifie | ProofCard.tsx, BrowsedProject.tsx | 0 |
| Patterns | P1, P4, P24 verifies, PATTERNS.md parcourus | PATTERNS.md rust + shell | 0 |
| Scope-cuts | 14 items kickoff §7, grep + lecture semantique diff | kickoff.md §7, diff complet | 0 |
| Branch coverage | 11 elements (8 DEEP-PASS, 1 SHALLOW-PASS, 1 PARTIAL P3, 1 WIRING-UNTESTED P2) | ProofCard.test.tsx lu en entier (123 lignes), 8 tests analyses | 2 (1 P2, 1 P3) |
| Research grounding | preflight 5/5 scans, 0 dep delta, coherence TS-vs-Rust 10/10 champs | proof_card.rs (253 lignes), ProofCard.tsx (274 lignes), protocol.ts, useBridge.ts | 0 |
| Livrables | 4/4 verifies via Read | ProofCard.tsx, BrowsedProject.tsx, ProofCard.test.tsx, THREAT_MODEL.md | 0 |
| Horizon long-terme | design doc + alternatives + LOC | kickoff §D5, plan §7, SYNTHESIS §4.6 | 0 |

## Recommendation

- Ready to commit : OUI (verdict PASS, Codex reconcilie)
- Carry-overs S69 : P2-D-1 wiring untested (acceptable si documente body), P2-D-2 type sans Zod validation (acceptable carry), P2-D-3 THREAT_MODEL XSS mention (acceptable carry)
- Corrections needed : aucune P0/P1

## Post-commit obligatoire

- [ ] Update nexus_grid_pivot.md (tip SHA + description sprint + compteurs tests)
- [ ] Update MEMORY.md (ligne index si pivot description changee)
- [ ] Verifier que review.md est stage dans le commit chore(planning) suivant
