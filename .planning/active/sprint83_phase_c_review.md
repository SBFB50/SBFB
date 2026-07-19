## Verdict: PASS

Review Phase C Sprint 83 « autorite du bridge » (intersection host-side per-call,
fail-closed sur set declare, grandfather legacy, egress getter-TOCTOU ferme). Etat
FINAL apres **review Workflow ultracode** (13 agents Opus 4.8 1M, 6 dimensions +
verification adversariale) + **Codex GPT-5.6 Sol round-4** reconcilie (cf. §Codex
reconciliation). La review Workflow a rendu **0 P0/P1** ; les **4 P2 + 5 P3** confirmes
ont ete traites. **1 P1 (regression `sbfb-ideas`) trouve APRES la review** par un audit
de completude adversarial (6e agent du Workflow de verif Codex), **corrige in-loop +
garde-fou machine** (cf. §3bis) ; Codex a **independamment confirme** le blast-radius
(seul sbfb-ideas casse) et 0 autre bypass. Suites vertes (Rust 2137/2137, web 529/529).

---

## 1. Etat de verification (arbre reconcilie apres fixes review + fix P1)

- Rust : fmt 0 · clippy workspace 0 · **nextest 2137/2137** (0 skip ; +8 vs 2129 :
  `grant_v0::tests` 7 + `bridge_methods_intersect_grants` 1) · doctest 1/1 · release OK.
  (1 flake transitoire observe a un re-run [classe carry 7 dispatch/sigint/boot], **vert
  au re-run propre 2137/2137** — le fix P1 ne touche aucun source Rust.)
- Web : tsc 0 · lint 0 (5 warnings historiques tolerated) · **vitest 529/529** (+11 vs
  518 : garde `example_manifest_declarations` — declare⊇reached + drift-guard méthodes +
  HELPER_COVERAGE helpers) · coverage **86.94 / 78.94 / 86.13 / 88.35** (≥ 85/78/85/85,
  inchangee — le garde teste des manifestes d'exemples hors `web/src`) · build + size 6/6.
  (2 flakes `/déjà/i` UI sous charge coverage, **verts au re-run** propre.)
- e2e : `bridge-grant-intersection` (2) + `workflow-hub-a11y` (8) + Browse/shell
  **22/22** hermetiques.
- Gates : `check-frontier-contracts` clean — bloc parite bridge 16==16 + **self-test
  de non-vacuite committe** · scan-en clean.

**Delta tests** : **+8 Rust** (grant_v0 model + empty-declared + off-allowlist bound) ·
**+39 web** (authority model + intersection integration + getter-TOCTOU + off-allowlist
+ revocation before/after ; **+11 garde declare⊇reached example-manifest [P1] + drift/
HELPER_COVERAGE**) · **+2 e2e** (smoke + framing).

## 2. Dimensions (review Workflow — 6 dimensions, verification adversariale)

security-deep · correctness · tests-semantics · wire-scope · grounding-scopecuts ·
docs-contract — **0 P0/P1 en review Workflow**. Cœur verifie independamment : choke-point
UNIQUE `isAuthorized` (`useBridge.ts:183`) AVANT tout dispatch (pii_redact offline inclus) ;
algebre d'ensembles fail-closed EXACTE Rust↔TS (expiry `>=`, revocation-first, None/null
= identite) ; getter-TOCTOU reellement ferme par clone-once-scan-send (`useBridge.ts`).

## 3. Findings confirmes + dispositions

**P2 (4) — tous traites :**
- **F1 — chemin fail-OPEN non documente** : une app qui DECLARE un `bridge` (regime
  fail-closed) est grandfathered aux 16 verbes pendant la fenetre async de resolution
  au 1er montage ET durablement sur tout echec de fetch `SBFB.json` (repli grandfather,
  cache react-query ~60s). → **Residu documente** : `THREAT_MODEL` §5.2
  `T-BRIDGE-RESOLVE-FAILOPEN` (ACCEPTE pre-launch, NON-escaladant — une app hostile
  omettrait la clef `bridge` -> grandfather de toute facon ; l'intersection ne peut que
  RETRECIR la surface vs l'etat pre-Phase-C ; a resserrer post-v1.0 ; le contenu untrusted
  DANS une app declaree pendant la fenetre reste un residu P3 accepte).
- **F2 — recit « rien ne casse » sur-generalise (⇒ a mene au P1 §3bis)** : 6/7 exemples
  declarent un `bridge` ; l'assertion « declare ⊇ appele verifie en review » ne couvrait
  que les call-sites `bridge.*` DIRECTS et **a manque le chemin SDK transitif** — cf. §3bis
  (P1 `sbfb-ideas`). → **Narrative corrigee** (THREAT_MODEL v21 « 1 correction de
  declaration » + audit transitif) ; **garde machine construit ce sprint** (ne reste plus
  un simple NOTE deferee).
- **F3 — bloc parite cite « mutation-teste » sans self-test committe** : le seul test
  parite (`lib.rs allowlist_mirrors_host_dispatch_schema`) compare a un EXPECTED Rust,
  ne lit jamais `protocol.ts`. → **Self-test de NON-VACUITE committe** ajoute au bloc
  parite (`check-frontier-contracts.sh` : un drop d'une methode DOIT changer le set ->
  prouve la discriminance du COMPARATEUR ; l'extracteur borne les noms a `[a-z0-9_]`,
  ce n'est PAS une preuve exhaustive de tout drift). La claim « mutation-teste » est
  retiree (THREAT_MODEL + gate reformules).
- **F4 — THREAT_MODEL presente 5 operandes sans caveat d'inertie** : row I + v21
  listaient `... ∩ politique ∩ consentement` alors que Phase C ne cable ACTIVEMENT que
  `declare ∩ grant`. Le code EST honnete (`authority.ts` « inert v0 »). → **Caveat inerte
  v0 ajoute** a la row I + v21 (politique + consentement INERTES, activees L/L1/…).

**P3 (5) — traites :**
- **F5 — e2e smoke vs texte du plan** : le plan C.3/C.4 decrit 4 refus ; l'e2e livre est
  un smoke + framing (precedent `workflow-hub-a11y.spec.ts`), la preuve AUTORITAIRE du
  refus vit dans `intersection_authoritative.test.ts` (MessageEvent reel via le
  choke-point). Artefacts HONNETES (le spec le declare lui-meme). → Documente ici + body.
- **F6 — symetrie Rust↔TS** : (1) `declared = Some(empty) -> refuse-all` n'avait pas de
  test Rust -> **test `declared_empty_refuses_all` ajoute** ; (2) noms hors-allowlist :
  TS `resolveDeclaredAuthority` DROPPE, Rust `validate` REJETTE le manifeste ->
  **divergence documentee** (module doc `grant_v0`, securite-equivalente : aucun n'ACCORDE).
- **F7 — assertion parite quasi-tautologique** (`new Set(BRIDGE_METHODS)).toEqual(
  ALL_DISPATCH_METHODS)` derive des deux cotes du meme const) → **renforcee** : comparaison
  vs une LISTE LITTERALE des 16 noms (un rename est desormais attrape).
- **F8 — `reads).toBe(1)` non-discriminant dans le test getter-TOCTOU** → **commentaire +
  ordre corriges** : le discriminateur rouge-avant/vert-apres est
  `JSON.stringify(sent).not.toContain("needs")` ; `reads` = signal secondaire.
- **F9 — serde defaults permissifs (latent, non-exerce)** : `Grant` derivait
  Deserialize + `#[serde(default)]` vers le permissif. → **Derive serde RETIRE** de
  `Grant` (YAGNI — 0 call-site (de)serialise en Phase C ; re-ajout avec defauts
  fail-closed a la persistance des grants, documente en module doc).

## 3bis. P1 trouve POST-review (audit de completude adversarial) + fix root-cause

- **Finding (P1, requires_code_fix)** : `sbfb-ideas` declare une clef `bridge` (regime
  fail-closed) avec `[storage_get, storage_set, storage_list, storage_delete,
  identity_pubkey]` mais **omet `storage_version`**, que l'app appelle **transitivement** :
  `examples/sbfb-ideas/app.js:337` cable `bridge.onStorageUpdate(...)` → SDK
  `sbfb-bridge.js:320` `onStorageUpdate` poll `getStorageVersion` toutes les 3 s →
  `_call("storage_version")` (erreurs avalees `.catch(() => {})`). Sous Phase C
  l'intersection **refuse `storage_version`** des que l'autorite declaree resout au-dela
  de la fenetre grandfather de montage → **live-sync d'Ideas Hub silencieusement casse**.
  Cela **falsifiait** la claim porteuse `THREAT_MODEL` v21 « aucune app ne casse
  (declare ⊇ appele) ». Codex et la review Workflow ont valide la mecanique de
  l'intersection mais n'ont jamais cross-verifie declare-vs-appele-transitif par app.
- **Fix (root-cause, PAS band-aid)** :
  1. `storage_version` ajoute a `examples/sbfb-ideas/SBFB.json` (methode allowlist valide,
     live-sync restauree ; declare ⊇ appele retabli).
  2. Narrative `THREAT_MODEL` v21 corrigee (« 1 correction de declaration » + note d'audit
     transitif ; les 5 autres apps declarantes verifiees exhaustivement).
  3. **Garde machine** `web/src/bridge/__tests__/example_manifest_declarations.test.ts` :
     pour CHAQUE exemple declarant un `bridge`, verifie `declared ⊇ methodes-atteintes`
     (helpers `bridge.*` directs + transitif `onStorageUpdate→storage_version` + `_call`
     inline). **Prouve rouge-avant** (retrait de `storage_version` → `undeclared
     (fail-closed refused): [storage_version]`) **/ vert-apres**. Drift-guard
     `HELPER_MAP_COVERS_SDK` (un nouveau `_call()` du SDK doit etre mappe).
- **Audit exhaustif (blast-radius CONFIRME independamment)** : compute-tester
  `[task_submit,task_result]` ✓, sbfb-factory-viewer `[browse_list,search,proof_card_get,
  storage_get]` ✓ (sur-declare), daisyui/velocity/workflow-hub `[]` ✓ (n'invoquent aucun
  verbe), sbfb-explorer sans clef ✓ (grandfather). **Seul `sbfb-ideas` etait casse.**

## 4. Scope cuts honnetes

- e2e = smoke + framing (convention foundation, cf. a11y) ; le refus est prouve par
  l'unit/integration (`intersection_authoritative.test.ts` + `authority.test.ts` + Rust
  `grant_v0::tests`).
- policy / consent = **couches inertes v0** (code honnete `inert v0`, doc alignee) ->
  cablees en L/L1.
- revoke-in-flight + value/nom-encoding = **residus documentes** (hors scope Phase C).
- **1 correction de declaration** (`sbfb-ideas` +`storage_version` transitif) : declarations
  existantes rendues AUTORITAIRES ; declare ⊇ appele tenu apres correction + **garde
  machine ce sprint**. Le gate CI declare-⊇-appele-transitif GENERIQUE (apps tierces
  arbitraires, hors exemples repo) reste un carry.

## 5. Conformite arbitrage PO

Q1 (grandfather absent-key / fail-closed declared, reconciliee) · Q2 (fold getter-TOCTOU
+ residu value/nom-encoding accepte) · Q3 (wording : Phase C = sous-ensemble host-side
5 couches du modele 7 couches) — **tous respectes**. Invariants : 0 wire bump, census
DOMAIN 25 gele, `contracts.rs` len==7, `SENSITIVE_ACTIONS` substring NON etendu, 0 dep
runtime neuve (le garde P1 = test-only, 0 dep). No-band-aid respecte : le P1 est ferme
par un garde machine, pas par le seul patch de donnees.

## Codex reconciliation

**Codex GPT-5.6 Sol reasoning max, round-4** (`sprint83_phase_c_codex_review.md`, output
brut NON reecrit) — verdict **PARTIEL, 0 P0/P1** : « Aucun bypass P0/P1 ni autre exemple
actuellement sous-declare ». **CONFIRME** : choke-point unique (§1), getter-TOCTOU +
refus synchrone (§3), parite fonctionnelle 16/16 + drift detecte (§4), resolution/fallback
grandfather (§6), round-2 CLEAN, et **§4(c) audit courant COMPLET** — Codex a re-audite
INDEPENDAMMENT les 6 apps declarantes (table) et confirme le blast-radius : **seul
`sbfb-ideas` etait casse, corrige ; aucun autre sous-declare** ; SDK vendored byte-identiques.

Dispositions des findings residuels (tous P2/P3, batches + re-verifies) :
- **§2 P3 — divergence latente `NaN`/`Infinity`** (Rust `u64` vs TS `number`) : 0 impact
  prod Phase C (le resolver n'emet que `null`). → **Note doc ajoutee** `authority.ts`
  (`expiresAtMs` MUST be finite u64-range ; a valider a la persistance des grants).
- **§4 P3 — prologue « Four deterministic checks »** alors qu'il y en a 5. → **corrige**
  `check-frontier-contracts.sh:8` (« Five »).
- **§4(b) P2 — garde incomplet** (le drift-guard ne compare que les ensembles de methodes ;
  un NOUVEAU helper→methode-deja-connue passerait ; regex excluait les chiffres ;
  `appSources` top-level `.js` seul). → **durci** : `HELPER_COVERAGE` (chaque helper public
  du SDK DOIT etre dans la table → un nouveau helper force la mise a jour) + `appSources`
  recursif `.js/.mjs/.cjs` + regex `[a-z0-9_]+`. HTML-inline reste hors scope (les exemples
  chargent le bridge via `<script src>`, jamais inline) — documente dans le test.
- **§5 — docs-contract stale ailleurs** (contrat/reference du bridge 3→16, frontiere Phase C).
  → **sweep applique** : DFD `THREAT_MODEL.md:99`, `SBFB_JSON_V2.md:48` (contrat app-facing
  `bridge.methods` + note AUTORITAIRE), `EXTERNAL_AUDIT_SCOPE.md:108`, `README.md:263` +
  `README_EN.md:163` (`sbfb-ideas` +`storage_version`). **`CLAUDE.md:90/602` (Decisions
  gelees) trace au wrap-up docs-contrat** (deferral explicite du preflight §85-93 —
  frozen-decisions, correction en lockstep sprint).
- **Round-3 GAP `codex_review.md:23/30/40`** (anciens libelles mutation/179/comptes) :
  **VACUOUS** — Codex a lu l'artefact round-1/2 en DEBUT d'analyse puis l'a **ecrase** par
  le present output round-4 ; ces lignes n'existent plus. `codex_review.md` = output brut,
  jamais reecrit (feedback_codex_raw_output).

Codex n'a pas pu lancer vitest (sandbox read-only, EPERM `.vite-temp`) ; suites executees
cote main-thread. **Suites relancees apres les fixes** : web **529/529** + coverage ≥ seuils
+ gate `check-frontier-contracts` clean ; Rust **2137/2137** inchange (le sweep ne touche
0 source Rust). **Boucle Codex arretee** (critere feedback_codex_loop_stop_criterion : 0
P0/P1, residuels P2/P3 tous fixes ou traces). Sequence review PASS-PENDING → Codex round-4
→ reconciliation → PASS respectee.
