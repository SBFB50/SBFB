# Sprint 21 Phase B — nexus-phase-auditor review

HEAD pre-commit: `63afe4e` (Phase A rate-limit)
Draft commit body: "feat(sprint21): Phase B — client-side PII redaction SDK iframe (onnxruntime-web + GLiNER PII edge)"
Timebox: ~35m

## Verdict : PASS

(0 P0 + 0 P1 + 2 P2 documentés — rigor signal G4 satisfait)

---

## Dimensions

### Security

- [x] Semgrep non disponible (Windows env) — fallback grep manuel effectué.
- [x] `as any` / `@ts-ignore` : aucun dans les nouveaux fichiers SDK. Un seul cast `as unknown as` dans `wrapper.ts:51` pour adapter l'import dynamique `@huggingface/transformers` dont les typings TS ne correspondent pas au runtime v4 — pattern légitime et commenté inline.
- [x] `unsafe` Rust : aucune modification Rust dans ce diff (Phase B = TypeScript/JavaScript uniquement).
- [x] `unwrap()` / `panic!` : N/A (aucun fichier Rust).
- [x] loopback / peer creds : non touché par Phase B.
- [x] Wire format JCS : non touché par Phase B (bridge = postMessage local, pas wire Rust).
- [x] `postMessage(response, "*")` dans `useBridge.ts:208` — usage de `"*"` comme targetOrigin. Anti-pattern connu et pré-existant depuis Sprint 13 (commit `c32d9c7`). Justification inline : l'iframe est sandboxée `allow-scripts` sans `allow-same-origin`, son origin est `null` (opaque origin sandbox). Passer l'origin exacte serait impossible. Ce pattern était audité et accepté Sprint 13. Aucun nouveau risque introduit par Phase B.
- [x] Secrets / tokens hardcodés : aucun.
- [x] **ONNX scaffold path retourne toujours `[]`** : `OnnxModelHandle.detect()` ligne 107 retourne systématiquement une liste vide après avoir exécuté l'inférence ONNX. Le post-processing span-logits → findings n'est pas implémenté. La détection GLiNER ONNX est architecturalement câblée (tokenisation + `session.run()` appelés) mais les sorties sont ignorées. Le `wrapper.ts` compense automatiquement via le fallback regex quand `filtered.length === 0` (ligne 164-167). Résultat : les défenses regex (email/phone/CC/SSN/IBAN) sont actives, mais le modèle ONNX ne contribue aucune détection supplémentaire en Phase B. Sécurité dégradée vs. les ambitions initiales mais pas un P1 car : (a) la défense regex reste active pour les 5 entités critiques, (b) la couche 2 Presidio coord-side (Phase C) ferme le gap, (c) la scaffold limitation est documentée inline avec la promesse d'un follow-up. Classé **P2** (documentation incomplète dans commit body).

### Patterns

- [x] **P1 — Typed coordinator client** : `dispatchPiiRedact` en `useBridge.ts` n'appelle pas le coordinator — dispatch local correct selon design. Les 3 autres méthodes bridge continuent de passer par `coordinator.ts`. Pattern respecté.
- [x] **P2 — base-ui render prop** : aucun composant React modifié par Phase B. N/A.
- [x] **P3 — Zustand curried create** : aucun store Zustand modifié. N/A.
- [x] **P4 — React Query only cache** : aucun `useEffect(() => fetch(...))` dans les nouveaux fichiers. N/A.
- [x] **P5 — CORS loopback** : non touché. N/A.
- [x] **P23 — CSP iframe** : Phase B n'étend pas les permissions CSP de l'iframe user. Le SDK vit côté host shell, pas dans l'iframe — CSP `connect-src 'none'` de l'iframe user est préservée. PASS.
- [x] **P24 — postMessage bridge protocol** : le pattern documente "Only three methods are whitelisted: `task_submit`, `storage_get`, `storage_set`". Phase B ajoute `pii_redact` comme 4e méthode, ce qui est l'intention de D2. Le texte de P24 est désormais **obsolète** — il ne reflète plus le nombre de méthodes. **P2 pattern drift** : P24 doit être mis à jour pour lister la 4e méthode et référencer le commit Phase B. Aucun pattern P33 n'a été ajouté pour décrire la convention d'extension bridge et le design host-shell singleton.
- [x] **P12 — Code splitting** : `web/src/sdk/pii/wrapper.ts` utilise des `import()` dynamiques pour `onnxruntime-web` et `@huggingface/transformers`. Le build résultant crée des chunks séparés (`ort.bundle.min-*.js` 393 KB + `transformers.web-*.js` 551 KB), hors budget intentionnel car lazy-loaded. Main bundle `index-*.js` reste 15 KB. PASS.
- [ ] **P24 mis à jour** : NON — voir finding P2-1 ci-dessous.

**Pattern drift détecté** : Phase B étend une convention (bridge whitelist) sans mettre à jour le pattern documenté (P24) ni en créer un nouveau (P33 pour le design SDK host-shell singleton + méthode locale). À tracker comme P2.

### Working tree audit (G5)

- [x] **PHASE** (17 fichiers code/tests) : `web/.gitignore`, `web/package.json`, `web/package-lock.json`, `web/public/models/.gitkeep`, `web/public/sbfb-bridge.js`, `web/src/bridge/__tests__/protocol.test.ts`, `web/src/bridge/__tests__/useBridge.test.ts`, `web/src/bridge/protocol.ts`, `web/src/bridge/useBridge.ts`, `web/src/sdk/pii/__tests__/fallback.test.ts`, `web/src/sdk/pii/__tests__/policy.test.ts`, `web/src/sdk/pii/__tests__/wrapper.test.ts`, `web/src/sdk/pii/fallback.ts`, `web/src/sdk/pii/index.ts`, `web/src/sdk/pii/policy.ts`, `web/src/sdk/pii/wrapper.ts`, `web/tests/bridge-pii-redact.spec.ts`. Tous listés dans plan §5.3 (avec drift sbfb-bridge.js path documenté dans commit body). ATTENDU.
- [x] **CRAFT** (2 fichiers planning) : `.planning/active/sprint21_phase_B_preflight.md` + `.planning/research/S21_phase_B_iframe_pii_sdk_design.md`. Le commit body reconnaît ces fichiers dans Working tree audit G5 (CRAFT). En général, CRAFT devrait précéder en `chore(planning)` distinct. Exception ici : le design doc et le preflight sont des pré-requis explicites du plan Phase B (§5.1 et §5.2), pas des artéfacts accidentels. Leur co-commit avec le code phase suit le pattern établi depuis Sprint 19 (preflight + design doc inclus dans le commit phase). Cohérent avec la convention du projet. ACCEPTABLE.
- [x] **DEBT** : 0 fichier tech debt ou scope-cut hors périmètre. CLEAN.
- [x] **NOISE** : 0 fichier accidentel (node_modules, cache, .env). `web/public/models/` exclu du commit via `.gitignore` avec `.gitkeep` tracké. CLEAN.
- [x] Section "Working tree audit G5" présente dans le commit body avec catégorisation PHASE/CRAFT/DEBT/NOISE. PASS.

### G8 traceability

- [x] `.planning/active/sprint21_phase_B_preflight.md` présent dans le diff staged.
- [x] Verdict du preflight : **EXECUTE plan-as-is** (S1+S2+S3+S4 clean).
- [x] S1 scan backbone : `knowledgator/gliner-pii-edge-v1.0` ModernBERT / quint8 45.8 MB / opset compat ORT Web 1.24.3 confirmés. CVE-2026-1839 vérifié non-applicable npm. PASS.
- [x] S2 historical : 0 commit archive rejetant l'approche. PASS.
- [x] S3 threat model : T4 (model extraction) + T2 (PII leak passive) + T1 (network observer) couverts. PASS.
- [x] S4 wire invariants : aucun `*_VERSION` bumpé. PASS.
- [x] Artefact G8 présent AVANT ecriture code (preflight.md daté 2026-04-19, HEAD Phase A `63afe4e`). Gate G8 respecté. PASS.

### Scope-cuts

Scope cuts §8 du kickoff vs diff staged :

| Scope cut | Grep diff résultat |
|---|---|
| Rust-wasm iframe PII SDK realignement Option G (S22+) | Mentionné dans design doc uniquement (CRAFT, pas code) — CLEAN |
| Threading ORT WASM | Mentionné dans design doc uniquement — CLEAN |
| IBAN MOD 97 strict | Commentaire inline `fallback.ts:98` *documentant* l'absence intentionnelle (« deliberately keep this permissive; layer 2 Presidio applies MOD-97 ») — CLEAN, commentaire explicatif pas implémentation |
| Prometheus metrics PII findings | Mentionné dans design doc + commit body scope cuts — CLEAN |
| Presidio coord-side (Phase C) | 0 fichier `packages/nexus-coordinator/` dans le diff — CLEAN |
| LLM Guard (Phase C) | 0 fichier Python dans le diff — CLEAN |
| Policy hot-reload host shell (DEFAULT_POLICY statique) | `policy.ts` utilise `DEFAULT_POLICY` statique — scope cut respecté, documenté commit body |

Aucun scope creep détecté.

### Tests-delta

| Suite | Baseline S21 entrée | Annoncé (body) | Réel (compté) |
|---|---|---|---|
| Vitest | 241 | +15 | +15 (6 fallback + 3 policy + 2 wrapper + 3 protocol.test.ts + 1 useBridge.test.ts) |
| Playwright | 38 | +5 | +5 (`bridge-pii-redact.spec.ts`) |
| Rust | 652 (post-Phase A) | 0 | 0 |
| Python | inchangé | 0 | 0 |
| **Total delta** | | **+20** | **+20** |

Plan §5.4 projetait +10 Vitest + 5 Playwright = +15. Réel = +15 Vitest + 5 Playwright = +20. Surplus de +5 Vitest sur les tests protocol bridge (3 PiiRedactPayloadSchema + 1 pii_redact method accepté + 1 surplus useBridge). Justification dans commit body cohérente.

**Note audit** : plan §5.4 listait le test `4. policy.ts hot-reload live policy runtime` — non écrit. Policy hot-reload est explicitement scope-cut dans le commit body (`DEFAULT_POLICY statique`) et dans plan §5.3. Acceptable. Le total +15 Vitest est atteint par les 5 tests surplus bridge (pas un gap de couverture).

**Alerte scaffold** : les tests wrapper.ts (2 tests) exercent uniquement le modèle via `ModelLoader` stubé. La vraie inférence ONNX (non-stubée) n'est testée ni en Vitest ni en Playwright — les Playwright tests utilisent un handler JS minimal inline qui reproduit le regex fallback côté fixture. Ce design est délibéré (CI sans asset ONNX 45.8 MB) mais signifie que le gap de post-processing détaillé dans Security ci-dessus n'est pas detectable par les tests. **P2 finding** (pas P1 : le fallback est actif et testé).

### Research-grounding

- [x] `onnxruntime-web 1.24.3` : tracé dans `plan.md §3 Research consulté` + `kickoff.md §Sources` avec URL npm Microsoft mars 2026. Trace présente. PASS.
- [x] `@huggingface/transformers ^4.0.0` : tracé dans `plan.md §3 Research consulté` (v4 npm 2026-02). Trace présente. PASS. Version range `^4.0.0` (caret) : intentionnel selon kickoff §D2 « v4.x ». Risque minime (minor bumps npm HF ont un bon historique compat). P3 cosmétique — pourrait être épinglé `4.0.0` pour reproductibilité.
- [x] `knowledgator/gliner-pii-edge-v1.0` ONNX model : tracé dans research backbone resolution + preflight S1 scan. Apache-2.0 vérifié compatible AGPL-3.0. PASS.
- [x] CVE-2026-1839 HuggingFace Transformers RCE : vérifié non-applicable npm dans preflight S1. PASS.
- [x] RUSTSEC / npm-advisory `onnxruntime-web` : best-effort search effectué en preflight. PASS.
- [x] Spec API bridge postMessage (pattern S13) : référencée via commits S13 dans plan. Pas de nouvelle spec standardisée introduite. PASS.

Aucune dépendance crypto/spec standardisée nouvelle sans trace.

### Horizon long-terme + documentation amont

- [x] **Design doc présent** : `.planning/research/S21_phase_B_iframe_pii_sdk_design.md` (423 lignes, daté 2026-04-19) couvre les 3 options architecturales + rationale (a/b/c) + flow séquentiel + backbone modèle + lazy-load strategy + CSP requirements + bundle targets. Présent AVANT le code phase (co-commité avec le code, artefact G8 preflight précède). PASS.
- [x] **Alternatives rejetées citées** : D2 kickoff §D2 documente rejet Full Rust-first (tract opset 9-18 vs GLiNER opset 19, zero precedent wasm32-browser), rejet GLiNER.js (inactif mars 2025), rejet redact-core (solo maintainer), rejet Pyodide+Presidio (500 MB overhead), rejet Gretel gliner-bi-small (349 MB iframe). Tech debt T-NN+2 S22+ explicite. PASS.
- [x] **Solution la plus poussée** : `onnxruntime-web 1.24.3` = standard industrie Microsoft pour ONNX dans le browser. Alternatives plus auditées n'existent pas dans cet espace (tract n'est pas officiellement documenté wasm32-browser). Le rejet tract est factuel (opset gap) pas un court-circuit convenience. PASS.
- [x] **Aucune estimation LOC dans plan/kickoff** : plan §5.2 mentionne « Wrapper JS thin ~300-500 LOC » dans le design doc. **Attention** : cette mention est dans le design doc pre-req (`.planning/research/S21_phase_B_iframe_pii_sdk_design.md`) et dans le kickoff §D2 (« ~300-500 LOC »). Selon §6.7 README, les estimations LOC au plan/kickoff sont interdites ; seule la mesure retrospective est légitime. Cependant, la mention est présente dans le kickoff gelé D1..D5 depuis sprint ouverture, pas introduite par Phase B. Phase B n'a pas ajouté de nouvelle estimation LOC. **P3** (nit prééxistant, pas un finding Phase B).
- [x] **ONNX scaffold gap** : le design doc (§4 Scaffold path) reconnaît que le post-processing span-logits n'est pas implémenté en Phase B et qu'un follow-up est prévu. La transparence du design doc est PASS. Le commit body ne le mentionne pas explicitement — **P2 finding** (voir Security + Tests-delta findings).

---

## Findings

- **P2-1** : `docs/shell/PATTERNS.md §P24` est désormais obsolète : le texte dit « Only three methods are whitelisted: `task_submit`, `storage_get`, `storage_set` » alors que Phase B ajoute `pii_redact` comme 4e méthode whitelist. De plus, aucun pattern P33 n'a été créé pour documenter la convention host-shell-singleton SDK + méthode bridge locale (pattern réutilisable pour phases futures). Action : mettre à jour P24 + ajouter P33 dans `docs/shell/PATTERNS.md`. Peut être inclus dans le commit Phase B ou dans un `chore(patterns)` avant Phase C — ne bloque pas le commit Phase B.
  — `docs/shell/PATTERNS.md:1467`

- **P2-2** : Le commit body ne mentionne pas explicitement que `OnnxModelHandle.detect()` retourne toujours `[]` en Phase B (post-processing span-logits non implémenté). La défense ONNX est architecturalement présente (tokenisation + inference s'exécutent) mais sans output exploité. Le commit dit « le SDK charge lazy le modèle ONNX GLiNER-edge ou tombe sur fallback regex » — ce qui est fonctionnellement vrai (la branche fallback est toujours prise quand `filtered.length === 0`) mais ne révèle pas que la branche ONNX non-fallback n'est jamais atteinte. Recommandation : ajouter dans le commit body une ligne explicite « Note: span-logit post-processing scaffold (wrapper.ts:104-107) — ONNX inference runs but output decoder pending follow-up; regex fallback active for all 5 entity types in Phase B ». Cela évite toute ambiguïté pour l'audit Phase C ou Sprint 22. Ne bloque pas le commit.
  — `web/src/sdk/pii/wrapper.ts:104-107`

- **P3** : `@huggingface/transformers` est épinglé à `"^4.0.0"` (caret range) dans `web/package.json`. `onnxruntime-web` est épinglé exactement à `1.24.3`. Le kickoff §D2 précise `v4.x` comme cible, donc le range est intentionnel. Risque faible (HF npm minor bumps historiquement compat dans une major). Opportunité de le pinner à `4.0.0` exact pour la reproductibilité si souhaité. Non-bloquant.
  — `web/package.json`

---

## Recommendation

**Commit autorisé** (0 P0, 0 P1). Les deux P2 sont non-bloquants par nature :

- P2-1 (PATTERNS.md) : résolvable dans ce commit ou dans un `chore(patterns)` dédié avant Phase C. Recommandé dans ce commit pour cohérence atomique.
- P2-2 (commit body) : si l'exécuteur peut amender le body avant de finaliser le commit, ajouter la note scaffold. Sinon acceptable en l'état car `wrapper.ts` est autosuffisamment commenté.

Les 5 critères d'acceptation Phase B (plan §5.5) sont verts selon les evidences de vérification fournies.

Carry-over obligatoire Sprint 22 audit findings :
- T-NN+3 : `wrapper.ts` span-logit post-processing pending (ONNX detection completeness gap Phase B → S22+).
- P24 update : si non résolu in-phase (P2-1 ci-dessus).
