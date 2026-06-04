# Phase Review — Sprint 73 Phase E

## Verdict : PASS

Promu depuis `PASS-PENDING` apres reconciliation Codex GPT 5.5
(`sprint73_phase_e_codex_review.md` : 4/4 livrables CONFIRMES, 0 GAP,
0 PARTIEL). Voir §Codex reconciliation. Aucun P0/P1 ; 3 P2 + 1 P3
documentes (carry S74).

(Rigor signal : 3 findings P2+ documentes / >=1 requis pour PASS rigoureux)

Phase E cable la barre de recherche du shell React sur l'endpoint
daemon `GET /api/daemon/search` (enrichi Phase D, triplet provenance).
Frontend uniquement. Decision D4 gelee (champ dedie Browse via
`searchBrowse()`). Verdict preflight G8 = **SCOPE-CUT-CONSISTENT**.

---

## Resume par livrable

### `web/src/api/daemon.ts` (M, +78)
- NEW `SearchResultSchema` (Zod `.object().strict()`, 12 cles) +
  `SearchResponseSchema` (envelope `{ results, total, took_ms }`,
  `.strict()`).
- NEW `searchBrowse(baseUrl, q, limit=20, offset=0)` qui route via
  `callDaemon` (bearer `authFetch` + union `DaemonResult<T>`) et
  encode `q`/`limit`/`offset` via `URLSearchParams`.
- Types exportes `SearchResult` / `SearchResponse` (`z.infer`).
- Doc-comments explicitent le choix `.nullable()` vs `.optional()`
  (la cle est toujours serialisee cote Rust → `.strict()` exigerait
  qu'elle soit declaree).

### `web/src/pages/Browse.tsx` (M, +221/-14)
- Champ recherche dedie `SearchBar` (input `type="search"` + bouton
  `Effacer`), state local `searchTerm`, `trimmed`, `isSearching`.
- React Query `['daemon-search', coordUrl, trimmed]` avec idiome v5 :
  `enabled: isSearching`, `placeholderData: keepPreviousData`,
  `staleTime: 10_000`. La query browse existante renommee `browseQuery`.
- Rendu conditionnel : `isSearching` → `SearchResultsView`, sinon grille
  browse inchangee (hero masque en mode recherche).
- `SearchResultsView` (gere les 3 kinds `DaemonResult` :
  `unavailable`→`DaemonOfflineBanner`, `error`→`ErrorCard`,
  `data`→grille), `SearchHitCard` (badges `Source vérifiable` /
  catégorie / `P2P` / `Provenance` + lien `Source` garde-https),
  `SearchEmptyState` (FR).
- Helper `isHttpsUrl` (garde XSS) : `repo_url` rendu comme `<a href>`
  uniquement si `https://`.

### `web/src/api/__tests__/daemon.test.ts` (M, +132 → +6 tests)
- `searchBrowse_calls_daemon_search_endpoint` (URL exacte
  `?q=react&limit=20&offset=0`), `percent-encodes a pathological query`
  (`a&b=c d` → `a%26b%3Dc+d`), `returns kind=unavailable on 503`.
- `search_response_schema_parses_triplet`, `parses a hit whose triplet
  is null (non-release op)`, `rejects a hit that omits a provenance key
  (strict, not optional)`.

### `web/src/pages/__tests__/Browse.test.tsx` (NEW, +210 → +4 tests)
- `browse_search_renders_enriched_results` (hit + badge provenance +
  lien repo https), `does not render a non-https repo_url as a link
  (XSS guard)`, `browse_search_empty_state_french`, `keeps the browse
  grid when the query is empty (non-regression)`.
- Fixtures `makeSearchHit`/`makeBrowseEntry` + `mockFetch` route par
  pathname (browse vs search distincts).

### `.planning/active/sprint73_phase_e_preflight.md` (NEW)
- Artefact preflight G8, verdict SCOPE-CUT-CONSISTENT, 5 scans
  (S1a/S1b/S2/S3/S4) presents.

---

## Verification des 4 findings preflight

### Finding 1 — Schema Zod ↔ enveloppe Rust (cle par cle)
**ADRESSE / VERIFIE EXACT.** Trace producer→consumer faite ligne a ligne :

| Cle | Rust (`http.rs:2010-2037` / `search.rs:7-34`) | Zod (`daemon.ts`) | Match |
|---|---|---|---|
| (envelope) | `{ results, total, took_ms }` | `.object({ results, total, took_ms }).strict()` | ✅ |
| `project_id` | `String` | `z.string()` | ✅ |
| `project_name` | `String` | `z.string()` | ✅ |
| `category` | `String` | `z.string()` | ✅ |
| `description` | `String` | `z.string()` | ✅ |
| `op_type` | `String` | `z.string()` | ✅ |
| `source_type` | `String` | `z.string()` | ✅ |
| `score` | `f64` | `z.number()` (pas `.min(0)` — bm25 peut etre negatif) | ✅ |
| `repo_url` | `Option<String>` → `string\|null` | `z.string().nullable()` | ✅ |
| `commit_sha` | `Option<String>` | `z.string().nullable()` | ✅ |
| `archive_hash` | `Option<String>` | `z.string().nullable()` | ✅ |
| `provenance_hash` | `Option<String>` | `z.string().nullable()` | ✅ |
| `is_open_source` | `bool` (jamais null) | `z.boolean()` | ✅ |
| `total` | `u64` | `z.number().int().min(0)` | ✅ |
| `took_ms` | `u64` | `z.number().int().min(0)` | ✅ |

`callDaemon` parse avec `.strict()` (`daemon.ts:249-251`) → throw
`ApiProtocolError` sur cle extra/manquante. Les 4 provenance sont
`.nullable()` (PAS `.optional()`) → fidele : le Rust serialise
TOUJOURS la cle (`null` quand absent). Le test `rejects a hit that omits
a provenance key` prouve que `.strict()`+`.nullable()` rejette
l'omission. Route `/api/daemon/search` enregistree (`http.rs:360`).

### Finding 2 — Pas de scaffolding i18n
**ADRESSE.** `glob web/src/i18n/**` → aucun fichier. Strings FR inline
(ex : `"Rechercher une app par nom, catégorie ou description"`,
`"Aucun résultat"`, `"Source vérifiable"`, `"Effacer"`). Pattern
identique a l'existant Browse.tsx. `scan-en-strings.sh` clean.

### Finding 3 — Route via `callDaemon`/`authFetch` + encode `q`
**ADRESSE.** `searchBrowse` appelle `callDaemon(baseUrl, path, schema)`
qui passe par `authFetch` (bearer `x-sbfb-token`) — pas de `fetch` brut.
`q`/`limit`/`offset` construits via `URLSearchParams(...).toString()`.
Test `percent-encodes a pathological query` : `a&b=c d` →
`q=a%26b%3Dc+d` — la query ne peut pas s'echapper du query-string.

### Finding 4 — Garde XSS `repo_url`
**ADRESSE.** `isHttpsUrl(url)` (type guard `url is string`) ne retourne
`true` que si `url.startsWith("https://")`. `SearchHitCard` :
`const repoUrl = isHttpsUrl(hit.repo_url) ? hit.repo_url : null` → le
`<a href>` n'est rendu que si `repoUrl` non-null. Test `does not render
a non-https repo_url as a link (XSS guard)` : `repo_url:
"javascript:alert(1)"` → `queryByTestId("search-repo-link")` absent.
Texte interpolé `{value}` auto-echappe par React (pas de
`dangerouslySetInnerHTML` — grep 0 match). Lien avec
`rel="noopener noreferrer"` + `target="_blank"` + `stopPropagation`.

---

## Suites (§7.4) — bloc web complet

```
lint (eslint .)        : 0 errors, 5 warnings (pre-existants shadcn/ui,
                         hors diff : badge/button/sidebar/tabs/toggle
                         react-refresh/only-export-components)
tsc --noEmit -p tsconfig.app.json : exit 0, 0 erreur
test:unit (vitest run) : Test Files 24 passed (24)
                         Tests 289 passed (289)   ← 279 → 289 (+10)
build (vite)           : ✓ built in 1.54s (warning chunk-size pre-existant
                         transformers.web/ort, hors diff)
size (size-limit)      : 6/6 OK — main 25.93/50 kB, vendor-react 275.49/290,
                         vendor-query 102.48/120, vendor-ui 262.27/270,
                         CommandPalette 9.81/20, css 122.81/130
scan-en-strings.sh     : "src/ is French-only, clean" (exit 0)
```

**Bloc Rust** : NON relance (Phase E ne touche AUCUN `.rs` —
`git diff HEAD --name-only -- '*.rs'` = 0 fichier). Le bloc Rust complet
tourne deja cote main thread. Aucune contention CPU ajoutee.

### Delta tests — verification honnetete
- **Reel : 279 → 289 (+10 Vitest web).** Confirme par `vitest run`.
- Repartition mesuree : `Browse.test.tsx` (NEW) = 4 tests ;
  `daemon.test.ts` = 29 tests (etait 23, +6). 4+6 = +10. ✅
- L'annonce main thread « +10 Vitest web » est **HONNETE**. Le plan §E.5
  annoncait +4 ; les +6 supplementaires (searchBrowse x3 +
  SearchResponseSchema x3) sont adversariaux deliberes (encode
  pathologique, 503→unavailable, triplet null, strict-omit). A noter
  dans le body comme delta plan +4 → reel +10.
- size-limit : 6/6 inchange.

---

## Modified-file branch coverage (Step 2bis, G9)
Fichiers EXISTANTS modifies : `daemon.ts`, `Browse.tsx`,
`daemon.test.ts` (test). Nouvelles surfaces et leur couverture :

- `daemon.ts::searchBrowse()` → `searchBrowse_calls_daemon_search_endpoint`
  + `percent-encodes...` + `returns kind=unavailable on 503` ✅
- `daemon.ts::SearchResponseSchema` (strict, nullable triplet) →
  `search_response_schema_parses_triplet` + `parses ... null` +
  `rejects ... omits a provenance key` ✅
- `Browse.tsx::SearchResultsView` (3 branches kind) → enriched (data),
  empty (data vide). Branches `unavailable`/`error` couvertes par le
  pattern partage `DaemonOfflineBanner`/`ErrorCard` (composants deja
  testes ailleurs ; ici exercices via `kind:"data"`). **CONCERN mineur**
  P2-1 ci-dessous : pas de test direct du rendu `unavailable`/`error`
  DANS la vue recherche.
- `Browse.tsx::SearchHitCard` + `isHttpsUrl()` →
  `browse_search_renders_enriched_results` (https rendu) + `does not
  render a non-https repo_url` (javascript: bloque) ✅
- `Browse.tsx::SearchEmptyState` → `browse_search_empty_state_french` ✅
- `Browse.tsx` mode-switch `isSearching` → `keeps the browse grid when
  the query is empty (non-regression)` ✅

Aucune methode de logique metier sans test. PASS.

---

## Scope cuts verification (kickoff §7)
- **#11 rate-limit per-client search** (re-eval exigee en Phase E) :
  grep `rateLimit|throttle` dans Browse.tsx → 0 match. Re-eval :
  endpoint loopback (single local user) + debounce de fait via
  `enabled` + `placeholderData: keepPreviousData` ; residual
  T-SEARCH-DOS « acceptable pre-launch » (THREAT_MODEL §11). **Non
  requis S73, carry S74 maintenu.** Conforme preflight S3. ✅
- **#14 pagination boutons (prev/next)** : grep
  `setOffset|setLimit|setPage|pagination|page\+\+` → 0 match.
  `limit=20`/`offset=0` sont des params fixes sans UI. Seul bouton =
  `Effacer` (clear, pas pagination). **Honore.** ✅
- #1 SearchManifest reseau (D3 defer), #2/#3/#4/#5 atelier fork S74 :
  aucun code Factory/fork ajoute (le triplet est rendu display-only).
  Aucun fichier `crates/sbfb-factory` ou `.rs` touche. ✅

---

## Horizon long-terme + documentation amont
- Design doc / decision : D4 gelee au kickoff §363-400 avec 3
  alternatives rejetees (header global / Command Palette full-text /
  bridge-SDK-only). Pas de nouveau module structurant >1 sprint
  (composants UI locaux). ✅ (N/A design doc dedie)
- Idiome v5 React Query applique (preflight S1a APPROACH-ALIGNED) :
  `enabled` + `placeholderData: keepPreviousData` (pas le `keepPreviousData`
  v4 deprecie). ✅
- Solution la plus poussee : `.nullable()` (fidele wire) plutot que
  `.optional()` (tolerant), `.strict()` pour surfacer toute derive wire
  comme erreur protocole. ✅
- Aucune LOC estimee au plan (grep `LOC estim` plan/kickoff → rien de
  prospectif). ✅

---

## Memory consultation (Step 1.5)
Zone : frontend UX, pas de kudos/crypto/deps nouvelles/gouvernance.
- `feedback_approach.md` (toujours) : pick deepest, no band-aid →
  `.nullable()`+`.strict()` (fidelite wire), garde XSS reelle (pas un
  TODO), tests adversariaux. **Respecte.**
- `feedback_context7_systematic.md` : aucune nouvelle dep (react-query
  5.100.9 / zod 3.25.76 deja pinnees) ; preflight S1a/S1b a consulte
  context7 TanStack v5 + advisories. **Respecte / N/A nouvelle dep.**
- `feedback_v1_prod_ready.md` : UX recherche complete (etat vide FR,
  offline banner, clear), pas de « post-v1.0 ». **Respecte.**
Aucune violation memory.

---

## Findings (rigor signal — 3 P2+ documentes)

- **P2-1 (carry S74)** : `SearchResultsView` gere bien `kind:"unavailable"`
  (`DaemonOfflineBanner`) et `kind:"error"` (`ErrorCard`), mais aucun
  test n'exerce DIRECTEMENT ces deux branches dans la vue recherche
  (les tests couvrent `data` enrichi + `data` vide). Les composants
  cibles sont testes ailleurs et le wiring est trivial (switch sur
  `result.kind`), donc non bloquant. Carry-over leger : ajouter 1 test
  `search renders offline banner on 503` en S74 quand la barre evolue.
  `Browse.tsx:204-209`.
- **P2-2 (doc/process)** : drift plan→reel sur le delta tests (plan §E.5
  = +4, reel = +10) et sur la forme de la reponse (plan §293/§301 disait
  « SearchResult enriched 7+5 » alors que l'endpoint renvoie l'enveloppe
  `{results,total,took_ms}` + hit 12 cles). Les deux drifts sont
  resolus correctement dans le code mais DOIVENT etre traces dans le
  body commit (pas d'edition du plan, snapshot preserve), conformement a
  la consigne preflight §Action. Non bloquant.
- **P2-3 (hardening, carry S74)** : la garde XSS `isHttpsUrl` est appliquee
  au lien `repo_url` de `SearchHitCard`, mais les anchors `repo_url`
  PRE-EXISTANTS (`Browse.tsx:264`, `BrowsedProject.tsx`,
  `VerificationDetail.tsx`) restent sans garde de scheme (rendus
  `href={...repo_url}` bruts). Non-regression (Phase E n'introduit pas
  le defaut, elle fait MIEUX sur sa propre surface), mais l'incoherence
  merite une normalisation S74 quand le triplet devient action-driving
  (preflight S3 le signale deja). `Browse.tsx:264`.
- **P3** : `key={`${hit.project_id}-${idx}`}` mixe id+index — robuste
  aux doublons project_id, acceptable. Le bouton `Effacer` est un texte
  plutot qu'une icone X (cosmetique, coherent avec le style pill FR).

---

## Codex gate (§4.5) — zero exemption
- Status : **DONE** — `codex exec` (gpt-5.5, reasoning xhigh, workdir
  racine) lance via PowerShell ; prompt `.git/CODEX_SPRINT73_PHASE_E.txt`.
- Rapport brut (NON reecrit) : `sprint73_phase_e_codex_review.md`.

## Codex reconciliation
- Rapport Codex lu. Verdict : **4 livrables / 4 CONFIRMES, 0 GAP, 0 PARTIEL.**
  - L1 `searchBrowse` via `callDaemon`/URLSearchParams — CONFIRME
    (`daemon.ts:373`, evidence citee).
  - L2 schemas Zod alignes cle-par-cle sur `search_handler`
    (`http.rs:2010`) + `SearchResult` (`search.rs:8`) ; `.nullable()`
    (pas `.optional()`), `is_open_source` boolean, `score` sans min,
    enveloppe `.strict()` — CONFIRME.
  - L3 barre recherche + `isHttpsUrl` garde XSS — CONFIRME.
  - L4 tests avec assertions reelles ; Codex a execute les 2 fichiers
    cibles (33 tests passes) — CONFIRME.
- GAPs P0/P1 a corriger : **aucun.** Aucune boucle de fix declenchee.
- P2/P3 : inchanges (P2-1 branches offline/error vue recherche, P2-2
  drift plan→reel a tracer au body, P2-3 anchors repo_url pre-existants
  sans garde scheme, P3 cosmetiques) — documentes au body, carry S74.
- Suites NON relancees apres Codex (0 GAP → 0 modification de code depuis
  le bloc web vert ; arbre inchange).
- Review final : **PASS**. Sequence respectee : review PASS-PENDING →
  Codex → reconciliation → PASS → commit.

---

## Commit body validation (Step 4 / 4bis)
Body non encore fourni par l'executeur → la verification des 9 headers
`##` se fera au moment du commit. Rappel des sections obligatoires :
`## Contexte`, `## Fichiers`, `## Delta tests`, `## Verification`,
`## Scope cuts`, `## G8 traceability`, `## Pre-launch protocol`,
`## Codex verification`, `## Carry closure`. Le body DOIT :
- annoncer **Vitest 279 → 289 (+10)** (plan +4, +6 adversariaux) ;
- tracer les drifts plan→reel F1 (enveloppe vs bare array) + delta tests
  sans editer le plan ;
- titre `feat(search): Sprint 73 Phase E — ...` (les phases C/D
  utilisent `feat(search)`) ;
- `Co-Authored-By: Claude Opus 4.8 (1M context) <noreply@anthropic.com>`.

## Recommendation
- Codex §4.5 : **DONE** (4/4 CONFIRMES, 0 GAP). Review promu PASS.
- Ready to commit : **OUI** — review PASS, Codex reconcilie, 0 P0/P1.
- Carry-overs S74 (`sprint74_audit_findings.md`) : P2-1 (test branches
  offline/error vue recherche), P2-3 (normaliser garde scheme sur tous
  les anchors repo_url existants).
- Corrections faites avant commit : aucune requise (0 P0/P1).

## Post-commit obligatoire
- [ ] Update `nexus_grid_pivot.md` (tip SHA + Phase E + compteurs :
      Vitest 289)
- [ ] Update `MEMORY.md` (ligne index si description pivot change)
- [ ] Verifier que `sprint73_phase_e_review.md` + `_preflight.md` +
      `_codex_review.md` sont stage dans le commit phase (pas de
      chore(planning) intermediaire detecte : staging = phase directe)
