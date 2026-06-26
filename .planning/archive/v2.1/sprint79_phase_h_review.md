# Sprint 79 — Phase H review

**Phase** : H — Self-check runtime viewer + confirmation CSP daemon + testabilité (T1/T2)
**Méthode** : review Workflow ultracode `wf_54e6b5e8-0ee` (7 dimensions + 7 vérifications adversariales +
synthèse, 15 agents `claude-opus-4-8[1m]`, 1.19M tokens) → verdict initial **FAIL** sur 1 P1. Correctif appliqué
+ re-vérification adversariale ciblée `wf_df1d31f3-109` (2 focus + synthèse, 3 agents) → résiduel STALE-REPORT
fermé. Verdict final ci-dessous.

## Verdict: PASS

(review OK, 0 P0/P1 ouvert ; Codex GPT 5.5 reconcilié 6/6 CONFIRME — cf. §Codex reconciliation.)

## Déroulé review → fix → re-review

### Review initiale (wf_54e6b5e8-0ee) — verdict FAIL

6 dimensions sur 7 PASS-PENDING (D1 test Rust, D2 anti-faux-vert spec, D3 fixtures, D5 Day-0/threat, D6 scope,
D7 docs — toutes OK). **1 P1 réel** trouvé en D4 (T2) et confirmé adversarialement :

- **P1 — faux-vert du chemin coarse de T2** (`app_authoring_capability.sh`). L'ancien chemin « coarse » faisait
  `pass()` sur `PW_EXIT==0` dès que les 3 champs per-test étaient `null`, SANS confirmer qu'un test ait tourné
  non-skippé. Playwright sort 0 sur tout-skippé. 2 vecteurs : (a) `SBFB_E2E_BASE_URL` fuité →
  `test.skip(!HERMETIC)` skip les 3 → exit 0 → PASS creux ; (b) python3 présent mais titres de spec renommés →
  `has()` 'missing' → champs null → coarse exit-0 → PASS creux. Ironie : le gate dont la raison d'être est
  l'anti-faux-vert contenait lui-même un faux-vert → viole le mandat de testabilité README §4.

Findings mineurs : P2 (null-guard asymétrique → BLOCK trompeur sur parse vide), 8×P3 (grep RIG-ABSENT trop
large, littéral URL en clair dans la fixture dirty, wording « browser-side » imprécis, etc.).

### Correctif (root-cause)

**P1** — refonte du verdict T2, `exit 0` n'est jamais cru seul :
1. `unset SBFB_E2E_BASE_URL SBFB_E2E_COMPUTE SBFB_E2E_PROJECT_ID SBFB_E2E_MODEL` dans le sous-shell run (un env
   fuité ne peut plus skipper) — double-verrou avec le `test.skip(!HERMETIC)` describe-level.
2. **python3 REQUIS** (`rig_absent` sinon) — suppression totale du chemin coarse exit-0.
3. Le parser compte `SKIPPED`.
4. **Run-gate title-INDÉPENDANT** AVANT tout per-field : `TESTS_TOTAL>=3` ET `SKIPPED==0` ET
   `TESTS_PASSED==TESTS_TOTAL`, sinon `block "run"`.
5. Garde **title-drift** : run-gate OK mais un per-field `null` → `block "title-drift"` (jamais PASS sur
   breakdown périmé).

**P2** — parse vide (`TESTS_TOTAL` non-numérique) → `rig_absent` (plus de BLOCK csp=null trompeur) ; null-guard
symétrique sur les 3 champs.

**P3 retenus** — grep RIG-ABSENT resserré (`Executable doesn't exist|Please run.*playwright install`) ; littéral
URL retiré de `dirty/index.html` (base64 atob seul) ; docs §P71 + FACTORY_GATES : « browser-side » →
« E2E/client-side » (le sous-test CSP-equality lit via le client `request` Playwright, pas une page rendue).

### Re-vérification adversariale ciblée (wf_df1d31f3-109)

- **FOCUS B (régressions + P2/P3) = CLEAN** : run-gate ne casse pas le happy path (3/3 → PASS) ; comparaisons
  numériques bash sûres (operandes garantis entiers) ; P2/P3 tous confirmés ; Day-0 intacts, 0 bump wire.
- **FOCUS A (adversarial T2) = CONCERN** : P1 d'origine fermé (7/7 points confirmés file:line) MAIS 1 résiduel
  **STALE-REPORT** (improbable, double-verrouillé) : `rm -f "$PW_JSON" || true` avalait son échec + chemin fixe
  → si la suppression échouait ET que Playwright n'écrivait aucun rapport frais, `[ -s "$PW_JSON" ]` passait sur
  un rapport périmé 3/3.

**Fix résiduel** : `: > "$PW_JSON"` (truncate-in-place) avant le run — un rapport non-réécrit reste vide →
`[ -s ]` échoue → `rig_absent`. Une troncature impossible (locked/perms) est elle-même `rig_absent`.

## Preuves empiriques

- Rust : `blob_serve_csp_header_byte_exact_matches_contract` vert ; nextest workspace 1991/1991 (1990→1991, +1) ;
  clippy clean ; fmt OK ; doctests OK.
- Frontend : lint 0 erreurs, tsc OK, vitest 411/411 (flake `localStorage` au 1er run, vert au re-run), scan OK,
  build OK, size OK, coverage OK.
- **T1** `web/e2e/app-authoring.spec.ts` : **3/3 vert** (6.1s) — CSP header byte-equal, clean 0 violation, dirty
  ≥1 violation détectée (capture browser-level à travers l'iframe opaque).
- **T2** `app_authoring_capability.sh` (durci) : `status=PASS`, `tests_passed=3`, `tests_total=3`, `0 skipped`.
- **Preuve env-leak (a)** : `SBFB_E2E_BASE_URL` fuité → T2 force le run hermétique → PASS 3/3 (l'unset neutralise
  le skip ; l'ancien comportement aurait donné un faux-vert).
- `bash -n` OK ; builder fixtures déterministe (sha256 dirty.zip byte-identique sur 2 runs).

## Day-0 & invariants (tous tenus)

Source CSP unique `nexus_core_rs::csp::BLOB_SERVE_CSP` ; 0 report-uri/report-to/meta-CSP ; canal self-check =
`page.on('console')` browser-level, PAS une méthode bridge (protocol.ts inchangé) ; `status=PASS` = verdict de
TEST, jamais autorité de publish ; deliverable servi non muté (fixtures = archives dédiées committées) ; 0 bump
wire/version (http.rs additif 1 test ; .gitignore +3 ; docs-only). Scope == execute_scope du preflight, 0 fuite
Phase I.

## Changement de comportement intentionnel (surfacé)

python3 est désormais OBLIGATOIRE pour T2 (le coarse exit-0 ÉTAIT le faux-vert). Un hôte avec le rig complet
mais sans python3 → RIG-ABSENT (exit 3), non un PASS. Arbitrage correct.

## Scope cuts cohérents

- Restitution verdict `readonly` (surface produit) **différée** : composant non-monté = dead-code,
  `tools/factory-ui` sans infra de test, forcerait un 1er câblage factory-ui→operator hors-scope. Le « viewer »
  du titre est satisfait par le rejeu dans le vrai iframe-host de prod (`BrowsedProject`). L'enforcement
  load-bearing = T1 + test Rust byte-exact, tous deux livrés.
- Parité CI : T1 BLOQUANT via GHA `test:e2e` (step 10c, `--grep-invert @compute`) + `verify.sh` step 15.
  Woodpecker ne lance aucun Playwright (pré-existant, pas de chromium) — décision à consigner en Phase I.

## Codex reconciliation

Codex GPT 5.5 (`codex exec`, rapport BRUT non réécrit dans `sprint79_phase_h_codex_review.md`) :
**6/6 livrables CONFIRME, 0 GAP, 0 PARTIEL.**

- L1 test Rust byte-exact : CONFIRME (`http.rs:7563/7585/7608`, assert_eq vs `BLOB_SERVE_CSP` 200+404 `:7593/:7616`).
- L2 T1 Playwright : CONFIRME (3 tests `:121/:133/:157`, seed publish-blob→publish `:74-101`, `toBe(CSP_CONTRACT)`,
  clean `toHaveLength(0)`, dirty `toBeGreaterThan(0)`).
- L3 fixtures : CONFIRME (builder Node 0-dep, dirty atob, README, zips contiennent index.html, `node --check` OK).
- **L4 T2 anti-faux-vert : CONFIRME** — vérification indépendante : « Je ne trouve pas de chemin `pass()/exit 0`
  sans les 3 contrôles réellement passés » (run-gate `:253`, unset env `:169`, python3 requis `:199`,
  truncate stale-report `:162`, `bash -n` OK). Corroboration croisée du correctif P1 + résiduel STALE-REPORT.
- L5 docs : CONFIRME (§P71 `PATTERNS.md`, addendum runtime `FACTORY_GATES.md`, 0 promesse future introduite).
- L6 gitignore : CONFIRME (`.gitignore:147`).

Aucun GAP P0/P1/P2/P3 → aucune correction post-Codex requise ; suites non re-jouées (0 changement de code post-review).
Verdict review promu `PASS`.
