# Sprint 80 — Phase D — Review (avant commit)

**Phase** : D — Terminal-PTY-as-VERIFY (bootstrap) + diff de commits passés + knowledge advisory + brouillon de pack + folds §5.1.
**Date** : 2026-06-27.
**Orchestration** : Workflow ultracode `wf_22fa5bae-888` (12 agents Opus 4.8 1M — 6 dimensions + passe adversariale + synthèse).
**Préflight** : `sprint80_phase_d_preflight.md` — verdict **PLAN-ADAPT** (5 adaptations, 0 DESIGN-CONFLICT, 0 Day-0 touchée).
**Nature du diff** : front greenfield React 19 (`tools/factory-operator`) consommant des routes Operator **existantes** + **1 seul edit backend Rust** (fold D1 daisyui) dans la crate `sbfb-factory`.

> **Note process (honnêteté, pour l'audit gate S81)** : 2 des 6 agents de dimension ont dégénéré au niveau `StructuredOutput`
> (security = erreur retry-cap après vraie investigation 37 tool calls / 113k tokens ; correctness = stub `{title:"t"}` après
> 57 tool calls / 143k tokens). La review reste fiable parce que (a) leurs **vérificateurs adversariaux** ont mené des passes
> complètes de remplacement (adv:correctness = passe routes/shapes/hooks/invariants → 0 défaut ; adv:doctrine/patterns/tests/backend
> tous `upheld`) ; (b) la dimension SECURITY est **absorbée** par doctrine+correctness (les surfaces sensibles Phase D = MUR, pack,
> auth cookie, terminal — toutes couvertes par les invariants ci-dessous) ; (c) l'agent de synthèse a fait ses propres lectures ;
> (d) le **main-thread** a vérifié les invariants cardinaux + traité les findings. Le verdict ne repose PAS sur les 2 agents dégénérés.

---

## 1. Scope revu

Diff NON commité Phase D sur HEAD=`6991d51` (Phase C).

- **Backend (1 edit)** : `crates/sbfb-factory/src/operator_server.rs` (`AUTHORING_KNOWLEDGE_MANIFESTS` += daisyui + commentaire passé/présent) ; `crates/sbfb-factory/tests/operator_server.rs` (2 assertions daisyui dans les 2 tests authoring_knowledge). 0 nouveau champ struct (U2 satisfait par l'existant `PreflightPhase.file`).
- **Front API** : `src/api/operator.ts` (shapes/calls sprint-history, commit-diff, actions/log, context-pack, audit, lint, terminal-sessions/cast, chat-log, terminalWsUrl).
- **Front lib** : `src/lib/cast.ts` (parseur asciicast v2 pur) ; `src/lib/verdict.ts` (`VERIFY_ETAT` + tons reviewTone/preflightTone/toneText/toneBg).
- **Front catalog/state** : `src/catalog/surfaces.ts` (`SECONDARY_SURFACES` named const) ; `src/state/useOperator.ts` (surface + openSurface/closeSurface/preparePack + setMode clear surface) ; `src/state/useCommitDiff.ts` (key-derive).
- **Front composants** : VerifyScene/Terminal/TerminalXterm/ContextPackInspector/Mur ; SurfaceHost/ProcedeSurface/DiffView/ConformiteCard/SessionsSurface/CastReplay/CastXterm/KnowledgeSurface ; câblage App.tsx + Rail.tsx (lazy). `VerifyPlaceholder.tsx` supprimé (git rm).
- **Config** : `.size-limit.json` (vendor-xterm 360KB + vendor-xterm-css 6KB), `vite.config.ts` (manualChunks @xterm), `vitest.config.ts` (exclude xterm modules), `scripts/scan-front-discipline.sh` (exclude *.test).
- **Tests** : cast/verdict/ProcedeSurface/SessionsSurface/ContextPackInspector/ConformiteCard/DiffView/useOperator + `e2e/verify.spec.ts`.

Frozen « Factory hors daemon » tenu (la réponse `authoring_knowledge` est un tableau JSON Operator loopback, pas une enveloppe wire gossip/feed). Routes consommées = toutes existantes au HEAD (`operator_server.rs:172-201`).

---

## 2. Résumé des 6 dimensions (après filtrage adversarial)

| # | Dimension | Verdict | Findings retenus |
|---|---|---|---|
| 1 | Correctness (routes/shapes/hooks/invariants) | **PASS** | 0 finding (agent dégénéré → adv:correctness passe complète, 0 défaut) |
| 2 | Sécurité | **PASS** | 0 finding nouveau (absorbée par doctrine/correctness) ; 1 carry Phase C re-noté |
| 3 | Doctrine + scope-cuts (« 0 verdict calculé UI ») | **PASS** | 0 finding (5 praise confirmés) |
| 4 | Qualité des tests (sémantique) | **PASS** | 1 P2 (DiffView sans test) → **CLOSED** ; 4 P3 (2 CLOSED) |
| 5 | Backend Rust D1 + pre-launch wire | **PASS** | 1 P3 (asymétrie blake3 daisyui vs animejs) |
| 6 | Patterns + research-grounding | **PASS** | 3 P3 (1 CLOSED) |

**Cœur load-bearing CORRECT et corroboré ligne à ligne.** Les 11 appels de `src/api/operator.ts` correspondent EXACTEMENT aux routes enregistrées (`operator_server.rs:172-201`) — 0 path fantôme. Les shapes TS mirrorent fidèlement les structs serde Rust (`CommitDiff`/`FileDiff`/`DiffHunk`/`DiffLine` `sprint_history.rs:958-984` ; `DiffLine.kind` émis `add`/`del`/`ctx` `:1118/1129/1140` == union TS ; `ActionLogEntry` ; `list_sessions {name,path,size_bytes}`). Pattern key-derive react-hooks partout (état écrit uniquement en résolution async, loading/reset dérivé par clé `resolved.key===requested`, `onStatusRef` mis à jour dans un effet jamais au render).

---

## 3. Passe adversariale — synthèse

Les dimensions doctrine / tests / backend / patterns ont chacune été repassées adversarialement : **verdict `upheld`**, chaque finding confirmé reproductible à la ligne citée contre le working tree. La dimension correctness (agent dégénéré en stub) a été **entièrement re-jouée** par son adversaire (adv:correctness, 23 tool calls) : passe routes/shapes/hooks/invariants → `upheld`, aucun défaut P0/P1/P2. La dimension security (agent en erreur retry-cap) est absorbée par doctrine+correctness (§4/§6). Aucun verdict-signal réel n'a été dégradé.

Nuances honnêtes : un finding praise situait `queryByText('✓')==null` « AVEC ok:true » alors qu'il est dans le test `ok:false` (`ConformiteCard.test.tsx:37`) ; `chat_history_authoritative==false` est à `tests/operator_server.rs:993` (cité 994). Inexactitudes de pointage, pas de fond.

---

## 4. Invariants cardinaux — tenus et vérifiés

- **« 0 verdict calculé UI »** — `verdict.ts` toneText/toneBg = `switch` retournant des **littéraux** oklch (jamais de classe construite au runtime) ; `reviewTone`/`preflightTone` mappent une STRING **restituée** du backend vers un Tone, `=== 'PASS'` est la SEULE comparaison ; `VERIFY_ETAT` ne dit jamais PASS. `ProcedeSurface.tsx` VerdictPill rend `verdict ?? '—'` lu de `PhaseHistory`. Grep production adversarial : 0 score/jauge/%/trust-score. Garde double : `verdict.test.ts` (`not.toMatch(/\bPASS\b/)`) + `e2e/verify.spec.ts` (textContent runtime ≠ /PASS/, 0 `\d+%`).
- **MUR sans contournement** — `Mur.tsx` « aucun Forcer · Override · Bypass · Exécuter quand même » ; seul CTA avant = `onPrepare` → `useOperator.preparePack` (`setSurface('knowledge')`) = handoff vers le pack scellé, **jamais exécuté** ; `requires_gate` restitue le MUR et `return` avant `streamStart`. Registre des refus lecture-seule (`SessionsSurface.tsx`, `⛔ rejected`, aucun « réessayer en forçant »).
- **Connaissance non-autoritaire** — `ContextPackInspector.tsx` « il ne clôt aucun verdict » + références hachées / contenu jamais inliné / dérive = fraîcheur ; `KnowledgeSurface.tsx` « fraîcheur ≠ verdict » ; backend `chat_history_authoritative==false` maintenu.
- **Diff = vérité Rust** — `DiffView.tsx` rend les hunks de `CommitDiff` (`parse_unified_diff` Rust), 0 re-diff JS (couvert par `DiffView.test.tsx`) ; `CastXterm` écrit `castOutput(parseCast(raw))` verbatim.
- **Provenance U2 sans lecture de contenu** — `ProcedeSurface.tsx` provenance = nom du fichier préflight, title « lecture du contenu en S81 » (scope cut S81 différé et documenté in-app).

---

## 5. Backend Rust (fold D1) — conforme

- `operator_server.rs:524-527` — `AUTHORING_KNOWLEDGE_MANIFESTS` passe de 1 à 2 entrées au chemin exact `docs/factory/knowledge/daisyui/MANIFEST.json` (corpus + MANIFEST git-trackés, promus S79 Phase F, déjà provenance-checkés par `tests/daisyui_manifest.rs` — **non recréé**, adaptation préflight 4 honorée).
- `operator_server.rs:516-523` — commentaire strictement passé/présent, 0 promesse future « Phase X will/adds » → **anti STALE-PHASE-K** tenu ; l'ancien fragment « animejs only » supprimé.
- `tests/operator_server.rs` — 2 assertions daisyui **non-vacantes** : `.find(...).expect()` panique si absent + `assert_eq!(daisyui["exists"], true)` (`file_hash` ne pose `exists:true` qu'après un `std::fs::read` réussi). Dual-write context-pack ET chat/session ; assertions animejs non régressées. `cargo nextest -E 'test(authoring_knowledge)'` = **2/2 PASS**.
- **Pre-launch** : pas de wire format versionné, 0 `_VERSION`/bump/`serde(default)` — conforme CLAUDE.md §Pre-launch.

---

## 6. Sécurité (absorbée doctrine+correctness)

MUR sans Forcer/Override/Bypass (0 affordance de contournement) ; « Préparer le pack » = handoff non-exécutant (aucun spawn) ; auth cookie HttpOnly automatique (`credentials:'same-origin'` conservé, 0 header token JS) ; Operator hors CSP / browser=client (Day-0 tenu) ; connaissance consommée jamais autoritaire. `cast.ts` = parseur pur sans DOM (lignes malformées sautées, queue tronquée sans throw) ; `CastXterm`/`TerminalXterm` écrivent le flux verbatim dans xterm (pas d'injection HTML). Le terminal ne démarre que sur **CTA explicite** (`e2e/verify.spec.ts` `terminal-start` visible, pas d'auto-spawn `claude` au switch de mode). `getTerminalCast(name)` encode le nom (`encodeURIComponent`) + backend valide le path traversal (`operator_server.rs:1231-1253`).

- **(Carry Phase C, hors-diff)** `sse_gate` backend forge le frame `requires_gate` par `format!` brut — non exploitable (msg constant) + front défensif ; à durcir si le message gate devient dynamique → carry sprint dette Rust (inchangé en Phase D).

---

## 7. Findings — disposition

### P0 / P1 — aucun.

### P2 — **CLOSED in-phase**

- **DiffView (J11) sans test** → **CLOSED** : ajout de `src/components/surfaces/DiffView.test.tsx` (3 tests) — header avec compteurs backend, chaque ligne verbatim avec ses numéros de ligne restitués (add only-new, del only-old, ctx both), état vide honnête. Couvre le mapping `kind → ton/préfixe/gutter` (régression-garde de « diff = vérité Rust »).

### P3 — fermés ou carry

- **CLOSED** — Footguns `cast.ts` (event `length<3` skip + défauts header width/height→80/24) : 2 tests ajoutés à `cast.test.ts`.
- **CLOSED** — Tightness dérive D2 : `ContextPackInspector.test.tsx` asserte désormais `findAllByText(/dérive — relu/).length===1` + une référence non-dérivée rendue sans marqueur.
- **CLOSED** — CSS xterm async non mesurée : entrée `.size-limit.json` `vendor-xterm-css` (6KB, actuel 3.94) ajoutée → 5/5.
- **CARRY** — Asymétrie blake3 daisyui (`exists==true` seul) vs animejs (recompute) : défendable (code path `file_hash` partagé déjà prouvé + octets daisyui provenance-checkés par `tests/daisyui_manifest.rs`). Optionnel → carry sprint dette.
- **CARRY (Phase G/H)** — Cycle de vie WS/PTY du terminal non exercé E2E (spawn `claude` en CI = lourd) : E2E qui clique `terminal-start`, vérifie ouverture WS + fermeture propre → Phase G/H.
- **CARRY** — `VERIFY_ETAT.awaiting` défini mais non rendu (placeholder machine d'état Phase H, documenté + itéré par `verdict.test.ts`) : câbler en H sinon retirer.
- **CARRY (mineur)** — `ConformiteCard` avale silencieusement un échec `getLint` (dégradation `Promise.allSettled` intentionnelle) : micro-indicateur « lint indisponible » optionnel.

---

## 8. Suites §7.4 (toutes vertes)

| Contrôle | Statut |
|---|---|
| `npm run lint` (ESLint) | **0 erreur** |
| `tsc --noEmit` + `npm run build` | **OK** (index hero **35,96 KB** < 40 KB) |
| `vitest` unit | **77 passed** (16 fichiers) |
| Gates discipline (`scan-front-discipline` exclut `*.test` + strip `=== 'PASS'`/re-grep prod ; no-radix ; no-tw-config) | **clean ×3** |
| `size-limit` | **5/5** (app 35.97<40 · vendor-react 189.82<210 · css 19.25<20 · vendor-xterm 341.54<360 · vendor-xterm-css 3.94<6) |
| `cargo fmt --check` | **clean** |
| `cargo clippy --workspace --all-targets` | **clean** |
| `cargo nextest --workspace` | **2009/2009** (dont sbfb-factory 215, `authoring_knowledge` 2/2) |
| `playwright test` (E2E T1) | **6/6** (boot 2 + steer 2 + **verify 2**) |

Gate `scan-front-discipline` : exclusion `*.test.{ts,tsx}` justifiée (les tests référencent un verdict **restitué**, pas de l'UI shippée) ; la logique strip-then-recheck reste active sur la production. Exclusions de couverture xterm honnêtes et documentées (`vitest.config.ts`).

---

## 9. Scope cuts respectés (kickoff §Out)

- **Sessions = liste simple, PAS un board multi-agents** — `SessionsSurface.tsx` « NOT a multi-agent board (cut) ». ✓
- **Procédé = arbre read-only, PAS un canvas/timeline** — expand/collapse read-only, frise V8 + provenance U2. ✓
- **PAS de lecteur de contenu d'artefact (→ S81)** — provenance U2 = nom du fichier, contenu reporté S81 documenté in-app. ✓
- **Aperçu scellé / Proof Card absents (→ S81)** — non introduits. ✓
- **Auto-bascule arrachée au stream interdite** — bascule = MODE manuel (`Rail.tsx` « bascule manuelle · jamais auto » ; `setMode` manuel, clear surface). ✓
- **Day-0** : React 19, Base UI seule dep runtime (0 @radix-ui), Motion non utilisée en D (OK), Tailwind v4 oklch, Factory hors daemon (0 route daemon), cookie auth. ✓

---

## 10. Delta tests

- **Vitest** : 52 → **77** (**+25**), 8 fichiers Phase D (cast, verdict, ProcedeSurface, SessionsSurface, ContextPackInspector, ConformiteCard, DiffView, useOperator+surface) + E2E `verify.spec.ts` (2→4 ; +2 verify). Le +1 vs review initiale = test reduced-pack D2 (fix Codex round 1).
- **Rust** : **2009 inchangé** — le fold D1 est un edit de const + 2 assertions **in-place** dans des tests existants (`authoring_knowledge`), **0 nouveau test fn**. `cargo nextest --workspace` = 2009/2009.

---

## 11. Verdict + justification

Aucun P0, aucun P1. Les invariants cardinaux Phase D (0 verdict calculé UI, MUR sans force, connaissance non-autoritaire, diff = vérité Rust, provenance restituée sans lecture de contenu), les 5 adaptations du préflight PLAN-ADAPT, les scope cuts §Out et les contraintes Day-0 sont tenus et corroborés ligne à ligne contre le backend réel et la suite de tests. Le seul edit backend (D1 daisyui) est minimal, conforme et provenance-prouvé. Le P2 (DiffView sans test) et 3 P3 ont été **fermés in-phase** ; les P3 restants sont des carries explicites (asymétrie blake3, WS/PTY E2E Phase G/H, awaiting state Phase H, lint silencieux). Suites §7.4 toutes vertes.

Le commit body Phase D couvre ses 9 sections `##`.

## Codex reconciliation

Codex GPT 5.5 (`codex exec`, cross-model) — 3 rounds, output brut dans
`sprint80_phase_d_codex_review.md` (non réécrit) :

- **Round 1** : 9 CONFIRMÉ / 0 GAP / 3 PARTIEL. Le cross-model a attrapé un **P1
  que l'agent review correctness (dégénéré) avait raté** : `ChatLog.context_pack`
  typé comme le `ContextPack` complet alors que le backend `handle_chat_session`
  scelle un pack RÉDUIT (sans agent_system/process_docs/active_artifacts) → le
  chemin D2 `sealedMap(groups(reducedPack))` itérait des tableaux absents (capturé
  par le `.catch`, donc D2 dégradait SILENCIEUSEMENT à 0 marqueur avec une vraie
  session). + 2 partials mineurs (test chat/session daisyui sans `exists`, type).
- **Fix P1 + partials** : `groups()`/`sealedMap()` coalescent les tableaux absents
  (`?? []`) → D2 marche sur les champs partagés (dont `authoring_knowledge`, sa
  cible réelle) ; `ChatLog.context_pack: Partial<ContextPack>` (type honnête) ;
  test reduced-pack ajouté ; assertion `exists` au site chat/session. Suites
  re-vertes (Vitest 77, nextest workspace 2009).
- **Round 2** : 9 CONFIRMÉ / 0 GAP / 3 PARTIEL — L2/L9 passés CONFIRMÉ ; nouveaux
  partials L10 (ProcedeSurface ne rendait pas `deliverables`/`findings` — or le plan
  A1/U1 liste `findings`) + L12 (E2E n'assertait pas un verdict réel) + L3.
- **Fix L10 + L12** : arbre de procédé enrichi (deliverables + findings restitués,
  `data-testid` + test) ; E2E asserte un verdict-pill réel (filter regex
  EXECUTE|PLAN-ADAPT|PASS…). Suites re-vertes.
- **Round 3** : **11 CONFIRMÉ / 0 GAP / 1 PARTIEL**. Le seul PARTIEL = L3 : les
  shapes TS ne mirrorent pas TOUS les champs serde (`SprintHistory` omet
  roadmap/commits/verification/tests aggregate ; `ContextPack` omet
  operator_intent). **INTENTIONNEL et documenté** : on type uniquement les champs
  CONSOMMÉS par le front ; les champs JSON supplémentaires sont ignorés au runtime
  (comportement TS standard, pas un bug). Aucun champ manquant n'est lu. → P3,
  pas de fix (ajouter des types morts serait du gold-plating YAGNI).

Boucle Codex arrêtée à round 3 (critère : CLEAN ou P2/P3 documentés ; 0 GAP, 1 P3
intentionnel). Le P1 D2 et les partials substantiels (L2/L9/L10/L12) sont fermés.

## Verdict: PASS
