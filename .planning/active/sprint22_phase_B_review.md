# Sprint 22 Phase B — nexus-phase-auditor review

HEAD pre-commit: `eaa8d4f` (chore(planning): sprint22 Phase B — G8 preflight verdict SCOPE-CUT-CONSISTENT)
Draft commit body: "feat(sprint22): Phase B — GLiNER span-logits decoder iframe SDK"
Timebox: 40m

## Verdict : PASS

(0 P0 / 0 P1 / 3 P2 documentés — rigor signal G4 satisfait)

P2-B-1 pré-identifié (end-to-end ONNX coverage, carry S23 Track B).
P2-B-2 finding additionnel auditeur (fallback sémantique résiduel scaffold, vestige commentaire
ligne 309 incorrect post-wire). P3-B-1, P3-B-2, P3-B-3 nits.
Commit autorisé sous réserve d'entrée des P2 dans `sprint22_audit_plan.md`.

---

## Dimensions

### Security

- [x] **grep scan patterns critiques** : 0 findings bloquants.
  - `grep -nE 'unwrap\(\)|unimplemented!|todo!|panic!'` sur diff : 0 résultat
    (pas de Rust modifié).
  - `grep -nE 'secrets|AKIA|ghp_|pat_|sbfb_[a-z]+_[a-zA-Z0-9]{20,}'` : 0 secret.
  - `console.log|warn|error` : 0 dans le diff.
- [x] **sigmoid numérique stable** : `decoder.ts:87-91` — branche `x >= 0` évite
  `exp(-x)` overflow (logit > 50 → `exp(-50) ≈ 0`, stable) ; branche `x < 0`
  évite `exp(x)` underflow (logit < -50 → `exp(-50)/1 ≈ 0`, stable). Implémentation
  correcte, vérifié `sigmoid(4.0)=0.9820`, `sigmoid(0.2)=0.5498`, `sigmoid(-20)≈0`.
- [x] **Threshold check directionnel** : `decoder.ts:124` — `if (prob <= threshold) continue`
  = strict greater-than conforme upstream `probs > threshold`
  (canonical `decoder.py` : `torch.where(probs > threshold)`). Correct.
- [x] **Span bounds check** : `decoder.ts:118-119` — `endToken = s + k + 1`, puis
  `if (endToken > numTokens) continue`. Tout span dont la fin déborde la séquence
  tokenisée est rejeté avant push. Pas d'accès out-of-bounds possible.
- [x] **Greedy dedup overlap** : `decoder.ts:152-156` — condition
  `!(span.endToken <= k.startToken || span.startToken >= k.endToken)`.
  Test manuel : span [1,4) vs span [2,5) → `!(4 <= 2 || 1 >= 5) = !(false||false) = true`
  = collision correcte. Span [0,2) vs span [2,4) → `!(2 <= 2 || 0 >= 4) = !(true||false) = false`
  = adjacents non-chevauchants, pas de collision. Logique conforme flat-NER.
- [x] **toFinding offset access** : `decoder.ts:172-173` —
  `tokenOffsets[span.startToken]` et `tokenOffsets[span.endToken - 1]`.
  `decodeSpans` garantit `endToken <= numTokens` (ligne 119) et `numTokens =
  tokenOffsets.length`, donc `endToken - 1 <= numTokens - 1` = index valide.
  Pas d'out-of-bounds possible.
- [x] **toFloat32Array BigInt edge case** : `wrapper.ts:173-188` — branche
  `ArrayBuffer.isView(data)` attrape `BigInt64Array`/`BigUint64Array` et appelle
  `Float32Array.from(data as ArrayLike<number>)`. JS autorise implicitement
  `BigInt(4n)` → `Number(4)` dans ce cast, numériquement correct mais
  sémantiquement incorrect. En pratique le GLiNER span head exporte toujours
  float32 (jamais BigInt), donc ce chemin ne se déclenche jamais en prod.
  Voir P3-B-3 ci-dessous.
- [x] **loopback/wire/zip** : diff 100% `web/src/sdk/pii/` (TypeScript pur, iframe-side).
  Aucun listener loopback, aucun zip extract, aucun wire format Rust. N/A.
- [x] **JCS canonique** : module pur, 0 sérialisation wire. N/A.
- [x] **unsafe** : 0 bloc unsafe (pas de Rust modifié). N/A.

### Patterns

- [x] **P24 — postMessage bridge** (`docs/shell/PATTERNS.md:1461-1478`) :
  `decoder.ts` et les modifications `wrapper.ts` sont internes à la couche
  SDK. Le bridge dispatch `pii_redact` (Sprint 21 Phase B) appelle
  `getSharedDetector().detect()` via `index.ts` ; Phase B ne modifie pas
  l'interface publique bridge. Pattern P24 respecté.
- [x] **Lazy-load pattern** (`wrapper.ts:263-293`) : `GlinerPiiDetector.ensureLoaded()`
  — modèle chargé uniquement au premier appel `detect()`. Le pattern S21 Phase B
  (`d5b0035`) est préservé intégralement. Pas de régression.
- [x] **Fallback regex préservé** : `wrapper.ts:302, 304, 311, 315` — toute erreur
  d'inference ou load failure délègue à `fallbackDetect()`. Défense en profondeur
  couche 1 (iframe) toujours opérationnelle.
- [x] **SPDX header** : `decoder.ts:1` = `// SPDX-License-Identifier: AGPL-3.0-or-later`.
  `wrapper.ts:1` = idem. `decoder.test.ts:1` = idem. `wrapper.test.ts:1` = idem.
  Convention respectée dans les 4 fichiers du diff.
- [x] **JSDoc style** : toutes les fonctions exportées ont un JSDoc complet avec
  description + type de paramètres (`@param` implicites via TypeScript) +
  pre-conditions documentées. Cohérent avec le style S21 Phase B.
- [x] **Export pattern index.ts** : `decoder.ts` n'est PAS re-exporté via `index.ts`
  (vérifié `grep -n "decoder" web/src/sdk/pii/index.ts` → 0 résultat). Le module
  est privé au `wrapper.ts` comme attendu. Pas de leak de l'API interne.
- [x] **Singleton module-scope** : `wrapper.ts:327-331` — `sharedInstance` module-scope,
  `getSharedDetector()` retourne le singleton. Pattern S21 Phase B preservé.
- [x] **P12 — code splitting** (`docs/shell/PATTERNS.md §P12`) : `decoder.ts` est
  un import statique dans `wrapper.ts` (pas un dynamic import), acceptable car
  il s'agit d'un module pur sans dépendances extérieures lourdes (~3 KB). Les
  dépendances lourdes (`onnxruntime-web`, `@huggingface/transformers`) restent
  en dynamic import (`wrapper.ts:64-65`). Pattern P12 respecté.

### Working tree audit (G5)

- [x] **PHASE** : 4 fichiers stagés, tous couverts par `plan.md §5.2` :
  `web/src/sdk/pii/decoder.ts` (nouveau), `web/src/sdk/pii/wrapper.ts`
  (modifié), `web/src/sdk/pii/__tests__/decoder.test.ts` (nouveau),
  `web/src/sdk/pii/__tests__/wrapper.test.ts` (modifié). Comptage exact.
- [x] **CRAFT** : 0 fichier planning/docs dans le staging. `git status --short`
  montre uniquement les 4 fichiers attendus + rien d'autre.
- [x] **DEBT** : 0 fichier scope-cut ou tech-debt hors phase.
- [x] **NOISE** : 0 fichier accidentel. Clean.
- [x] **Section "Working tree audit"** : draft commit body mentionne
  `web/src/sdk/pii/*` uniquement + scope cuts respectés. Conforme G5.

### G8 traceability

- [x] Artefact G8 présent : `.planning/active/sprint22_phase_B_preflight.md`
  — commit `eaa8d4f`, verdict **SCOPE-CUT-CONSISTENT** (2026-04-19).
- [x] Fichier présent : `sprint22_phase_B_preflight.md` (verdict : SCOPE-CUT-CONSISTENT)
- [x] 4 scans S1-S4 documentés :
  - S1 : canonical GLiNER decoder context7 `/websites/urchade_github_io_gliner`
    (2026-04-19) + `onnxruntime-web 1.24.3` CVE clean + `@huggingface/transformers
    ^4.0.0` `^4` range défensif (v5 break confirmé Issue #324).
  - S2 : décisions historiques traversées (uniquement `d5b0035` scaffold, pas de
    décision rejetée pertinente). Clean.
  - S3 : threat model A4 User consent PII couvert (defense-in-depth couche 1).
    Autres threats non-scope Phase B.
  - S4 : 0 wire format Rust touché, tous `_VERSION` inchangés, pre-launch policy
    respectée.
- [x] Verdict SCOPE-CUT-CONSISTENT : finding S1-B-1 (pseudocode 3-tensor vs
  canonical single-tensor) résolu inline, pas de carry S+1 requis per preflight.
- [x] Findings non-bloquants : S1-B-1 résolu in-phase, documenté dans commit body
  + preflight.md. Aucun carry S23 requis pour ce finding. Les 2 P2 nouveaux
  (P2-B-1 + P2-B-2) sont ajoutés à `sprint22_audit_plan.md` par l'exécuteur.

### Scope-cuts

- [x] `redundancy voting` : `grep -nE 'redundancy.voting'` → 0 match dans le diff.
- [x] `traffic padding` : 0 match.
- [x] `sandbox tool-calling` / `tool.call` : 0 match.
- [x] `Radicle` / `radicle` / `RADICLE` : 0 match.
- [x] Playwright e2e : 0 fichier `playwright` / `*.spec.ts` dans le diff.
  Confirm carry S23 Track B documenté `plan §5.3` + `plan §12`.
- [x] Rust / Python wire format : 0 fichier `.rs` / `.py` / `Cargo.toml` /
  `pyproject.toml` dans le diff. Phase B est 100% TypeScript iframe.
- [x] Cap G7 : aucun slot G7 consommé (pas de nouvelle tech debt formelle
  ouverte en Phase B, P2-B-1 est un carry pré-connu).

### Tests-delta

- [x] **Vitest** : annoncé +8 (256 → 264, draft body), plan §5.3 projeté +6.
  Réel mesuré : `npm run test:unit` → **264 passed (24 fichiers)**.
  Delta réel = **+8 vs baseline 256** = conforme draft body.
  Ratio delta +8 / plan +6 = 1.33x, within garde-fou < 2.5x. Accepté.
- [x] **Mapping tests vs plan §5.3** :
  1. `decodeSpans::promotes a single strong span` ✓ (plan test 1 decodeSpans_single_entity)
  2. `decodeSpans::drops spans below the sigmoid threshold` ✓ (plan test 3 threshold_filter)
  3. `decodeSpans::refuses spans whose end token runs past the sequence` ✓ (plan test 1 valid-span check)
  4. `decodeSpans::returns [] on empty tokens or empty tensor` ✓ (plan test 4 empty_input)
  5. `greedyDedup::keeps the highest-scoring span among overlapping candidates` ✓ (plan test 2 overlapping)
  6. `greedyDedup::preserves every span when none overlap` ✓ (plan test 5 non_overlap_preserved)
  7. `toFinding::translates token-index spans to character offsets` ✓ (nouveau, non dans plan §5.3, légitime)
  8. `wrapper::detects at least two entities on the Phase B fixture text` ✓ (plan test 6 detect_real_fixture)
  - Note : +2 tests vs plan (+7 decodeSpans/greedyDedup/toFinding au lieu de +5 prévus
    pour decoder.test.ts, +1 wrapper.test.ts conforme plan). Delta +8 total exact.
- [x] **Rust workspace** : non modifié Phase B. Baseline 666 inchangé (confirmé
  implicitement — diff 0 fichier Rust).
- [x] **Python coord** : non modifié Phase B. N/A.
- [x] **Playwright** : non modifié Phase B (carry S23 Track B documenté). N/A.
- [x] **TypeScript check** : `npx tsc --noEmit -p tsconfig.app.json` → 0 erreur.
- [x] **Lint** : `npm run lint` → 0 erreur, 7 warnings pre-existants
  (`sbfb-bridge.js` + `components/ui/*.tsx`) non introduits par Phase B.
  Vérifié : aucun warning ne pointe vers `web/src/sdk/pii/`.
- [x] **Cumul S22** : baseline 1436 → Phase A +7 = 1443 → Phase B +8 = **1451**.
  Conforme draft body.

### Research-grounding

- [x] **Deps ajoutées/bumpées** :
  - `git diff HEAD -- web/package.json web/package-lock.json` → 0 nouvelle dep,
    0 version bump. `onnxruntime-web 1.24.3` et `@huggingface/transformers ^4.0.0`
    pré-existants depuis S21 Phase B `d5b0035`. Clean.
  - `Cargo.toml` / `pyproject.toml` : non touchés. N/A.
- [x] **Trace §Research consulté plan §3** : section `GLiNER span-decoder Phase B` —
  liste `urchade/GLiNER paper output format (sigmoid + greedy dedup)` +
  `GLiNER.js npm référence algorithme (TS ~1500 LOC)` + `@xenova/transformers.js v4`
  + `onnxruntime-web 1.24.3`. Traces présentes.
- [x] **Canonical decoder tracé context7** : preflight.md §S1 documente
  `context7 /websites/urchade_github_io_gliner` query `decoder.py::_decode_batch`
  du 2026-04-19. Algorithme `sigmoid → threshold → valid-span → greedy_search`
  extrait et cité inline. Trace valide (< 6 mois).
- [x] **WebSearch CVE** : preflight.md §S1 documente `onnxruntime-web 1.24.3 CVE 2026`
  → aucun résultat. `GLiNER.js npm 2026 breaking change` → seule issue = transformers
  v5 (mitigée `^4.0.0`). Clean.
- [x] **API crypto / spec standardisée** : aucune nouvelle API crypto dans le diff.
  Module pur float32 arithmetic + sigmoid. N/A.

### Horizon long-terme + documentation amont

- [x] **Design doc présent** : `.planning/research/S21_phase_B_iframe_pii_sdk_design.md`
  (cité dans `web/src/sdk/pii/index.ts:9`) — trace écrite avant le code Phase B S21
  (architecture host-shell singleton + lazy ONNX + regex fallback + defense-in-depth
  Presidio). Phase B est la wire-up de la primitive documentée S21, pas un nouveau
  module structurant. Pas de nouveau design doc requis.
- [x] **Alternatives rejetées** : D2 scope γ kickoff §4 D1..D5 — Phase B = "B
  span-logits decoder" confirmé. Preflight.md §S2 confirme absence de décision
  rejetée pertinente dans l'historique `web/src/sdk/pii/`. Pas d'alternative concurrente
  pour le décodeur (GLiNER.js référencé comme algo référence, non-dep runtime).
- [x] **Solution la plus poussée** : module pur TypeScript sans dépendances ORT
  dans le decoder (testabilité jsdom), délégation des cas edge via `toFloat32Array`
  (robustesse ORT variants). Conforme principe "deepest technical option" — isolation
  claire decoder/wrapper, interfaces typées `OrtTensorLike`, `TokenizerEncoding`.
- [x] **Aucune estimation LOC dans plan §5** : plan §5.2 note "~250 LOC TS" —
  c'est une estimation prospective contraire à `README §6.7`. Voir P3-B-1
  (advisory hors-phase, finding additionnel auditeur déjà noté P3-S22A-1 pour
  les phases B-F).

---

## Findings

- **P2-B-1** (pré-identifié executeur, confirmé) : End-to-end ONNX runtime non
  exercé en CI. `jsdom` ne peut pas charger `onnxruntime-web` WASM ni le modèle
  45 MB. Le chemin `OnnxModelHandle.detect()` → `decodeSpans()` → `toFinding()`
  complet n'est couvert que par des fixtures synthétiques. La correction
  nécessite soit un mini-modèle ONNX dédié (< 10 MB), soit un test Playwright
  avec modèle stubbed. Documenté `plan §5.3 + §12` carry S23 Track B. —
  `web/src/sdk/pii/wrapper.ts:140-161` (chemin non exercé CI).

- **P2-B-2** (finding additionnel auditeur) : Comportement résiduel scaffold dans
  `wrapper.ts:308-311` — quand le modèle retourne 0 entités (texte sans PII),
  `filtered.length === 0` déclenche systématiquement `fallbackDetect()`, même
  quand le modèle a légitimement conclu "aucune entité". Le commentaire L309
  dit encore "Scaffold path returns empty" — ce commentaire est incorrect
  post-Phase B (le scaffold a été remplacé). Conséquence : texte sans PII reçu
  par une iframe sera toujours traité par le regex fallback après le modèle, ce
  qui peut produire des faux positifs regex sur du texte que GLiNER a
  correctement jugé sain. Comportement actuellement "defense in depth" acceptable,
  mais le commentaire induit en erreur et la sémantique devrait évoluer post-S22
  quand le modèle est validé. Fix : mettre à jour le commentaire L309 + ajouter
  un flag `model_result_trusted: boolean` dans `PiiPolicy` pour piloter le
  comportement (ou supprimer le fallback sur empty quand `use_model === true` et
  ready). Carry S23. — `web/src/sdk/pii/wrapper.ts:308-311`.

- **P3-B-1** (advisory hors-phase, already noted P3-S22A-1) : `sprint22_plan.md §5`
  contient `~250 LOC TS` prospectif contraire à `README §6.7`. Non-bloquant.
  Advisory : l'exécuteur peut supprimer opportunément lors du commit Phase F chore.

- **P3-B-2** (pré-identifié executeur, confirmé) : Branches défensives dans
  `toFloat32Array` (`wrapper.ts:173-188`) non exercées par les tests unitaires.
  Les stubs injectés dans `wrapper.test.ts` retournent directement des `PiiFinding[]`
  (bypass complet de `toFloat32Array`). Les branches `ArrayBuffer.isView` +
  `Array.isArray` + throw ne sont couvertes que par le test `decoder.test.ts` sur
  des `Float32Array` natifs. Le cas `DataView` et le cas `Array<non-number>` sont
  particulièrement non exercés. Robustesse runtime ORT variants non validée.
  Carry S23 Track B (même entrée que P2-B-1).

- **P3-B-3** (finding additionnel auditeur) : `toFloat32Array` (`wrapper.ts:175-179`) —
  commentaire L177-178 documente correctement que `BigInt64Array`/`BigUint64Array`
  "ne supportent pas le numeric indexing", mais le code les laisse passer à
  `Float32Array.from(data as ArrayLike<number>)`. En pratique, JS autorise
  implicitement `BigInt → Number` dans ce contexte, mais `Float32Array.from`
  passera par `Number(bigint_value)` avec perte de précision pour grands BigInts.
  Le GLiNER head exporte toujours float32 (jamais BigInt), donc jamais déclenché
  en prod. Nit cosmétique : ajouter une garde explicite
  `if (data instanceof BigInt64Array || data instanceof BigUint64Array) throw new Error(...)`.
  — `web/src/sdk/pii/wrapper.ts:175-179`.

---

## Recommendation

**Commit autorisé.** 0 P0 / 0 P1.

Actions requises avant ou pendant Phase F :
1. Entrée P2-B-1 dans `sprint22_audit_plan.md` carry S23 Track B (end-to-end ONNX
   coverage, mini-model fixture ou Playwright stub).
2. Entrée P2-B-2 dans `sprint22_audit_plan.md` carry S23 (commentaire L309 incorrect
   post-scaffold + sémantique fallback sur empty model result).

P3-B-1 : advisory LOC estimations plan §5 — supprimer lors Phase F chore.
P3-B-2 : advisory branches non exercées toFloat32Array — carry S23 Track B avec P2-B-1.
P3-B-3 : nit BigInt guard explicite toFloat32Array — fix cosmétique Phase F ou S23.

Cumul S22 carry-overs actifs post-Phase B :
- P2-S22A-1 (dashmap dep stale worker-core)
- P2-S22A-2 (sprint21_verification.md row 21 chemin incorrect)
- P2-S22A-3 (PATTERNS.md §P33 struct snapshot stale)
- P2-B-1 (end-to-end ONNX unexercised CI)
- P2-B-2 (fallback sémantique résiduel scaffold + commentaire incorrect)
