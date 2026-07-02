# Sprint 80 — Phase G — Preflight (deep, 5 scans + 3 adversariaux)

**Date** : 2026-06-28
**Phase** : G — backend Rust `GET /api/gates` dans la crate `sbfb-factory` (`operator_server.rs`). 0 route au daemon (Factory hors daemon, Day-0 #9).
**Verdict** : **PLAN-ADAPT**

> Le plan littéral est correct sur le fond et compatible Day-0, mais corrigé par evidence OSS concrète sans toucher aucun invariant : (1) sémantique gate-live = **registre à statut restitué** (pas de run sur GET) — confirmé par 4 prior-arts + RFC 9110 ; (2) ajout de l'état **`not_applicable`** (apport SARIF) ; (3) **désamorçage du faux dilemme A-vs-C** sur la shape `issues` (les FG ne portent jamais d'issues sur la route → seules les issues lint/audit, DÉJÀ structurées, y arrivent) ; (4) reframe de V5/V6 du binaire « dégradés/carry » vers **livrables au niveau fichier en S80**. Aucun adversaire n'a réfuté sur un invariant Day-0 → pas de DESIGN-CONFLICT.

> Orchestration : Workflow ultracode `wf_3f7d9185-be6`, 11 agents Opus 4.8 1M (6 scans → draft → 3 adversariaux → synthèse), 755k tokens. Faits load-bearing **re-vérifiés en main-thread** avant code : `GateResult{gate,passed,issues:Vec<String>}` Debug-only (gates.rs:13-18) ; `LintResult{ok,errors,warnings}` + `LintDiagnostic{code,message,file:Option<String>}` Serialize, **pas de champ ligne** (process.rs:411-423) ; `lint_planning_data` borné read_dir early-return `ok:true` (process.rs:425-436, `ok=errors.is_empty()` :510) ; `handle_lint` déjà GET (operator_server.rs:375) ; précédent F `handle_git_diff(State)→Json` (:1322) + route `authed` (:197) ; harness HTTP `TestServer`/`raw_get`/`server.get().json()` (tests/operator_server.rs).

---

## 1. Synthèse des 5 scans

| Scan | Objet | Verdict | Apport load-bearing |
|---|---|---|---|
| **S1a** OSS prior-art | Modéliser un « état de gate vivant » read-only | EXECUTE + PLAN-ADAPT mineur | GitHub Checks (lifecycle≠verdict), LSP (push jamais recalculé), **SARIF `kind=notApplicable`/`pass`**, clippy `rendered` lossless + spans optionnels. Registre à statut, pas run-sur-GET. |
| **S1b** Deps/CVE | Dépendances Phase G | EXECUTE | **0 ajout** : axum/serde/serde_json/walkdir/regex déjà présents. 0 CVE active. Aucun « gate runner » externe (YAGNI + heurterait « 0 verdict calculé »). |
| **S2** Décisions/internals | Scope réel des gates SBFB | (recommande registre) | Les 6 FG ciblent un **workspace d'app en publish** ; aucun ne tourne sensément sur `state.root`. Aucun gate-log persisté. `lint_planning_data` = source déterministe du `passed`. |
| **S3** Threat model | Sécurité de la route | EXECUTE si read-only/0-input/1:1 | Threat 1 (DoS/side-effect GET), Threat 2 (leak chemins — FG6 émet `pattern_name` PAS la valeur), Threat 3 (path-traversal si `workspace` paramétrable), Threat 4 (cohérence auth/CSP). Seul DESIGN-CONFLICT possible = un GET qui exécute les gates. |
| **S4** Wire/invariants | Format de réponse | EXECUTE | **0 bump wire, 0 `*_VERSION`** (API loopback interne). Enveloppe `{gates:[...]}` miroir `handle_providers`. `state` enum distinct, jamais `passed:bool`. Named constants pour les 6 noms FG. |

---

## 2. Verdicts adversariaux intégrés

| Lentille | Verdict | mustFix intégré |
|---|---|---|
| Idempotence/side-effect HTTP (RFC 9110) | **survives** | (optionnel, non bloquant) test d'idempotence : appeler `gates_live_data` deux fois sur la même fixture et comparer l'égalité byte-pour-byte du JSON. |
| Shape `issues` over/under-eng. + dette cœur | **survives-with-fix** | **BLOQUANT (phase code)** : quand un gate processus a À LA FOIS `errors` ET `warnings`, ne pas droper les warnings → émettre une entrée `blocking` (errors) + une entrée `informational` (warnings). VÉRIFIÉ : `LintResult` a deux Vec distincts (process.rs:411-416). |
| DESIGN-CONFLICT invariants/Day-0 | **survives** | — |

Aucune réfutation sur invariant Day-0 → verdict global ≠ DESIGN-CONFLICT.

---

## 3. Décision load-bearing FIGÉE

### 3.1 Sémantique gate-live
Registre des gates connus à **statut restitué**, lecture pure, 0 input, `workspace = state.root` en dur (ferme Threat 3). **Aucun scan publish synchrone sur GET.**
- FG4/FG5/FG6/FG7/FG8 → `not_run` (issues vides) → satisfait « ≥1 non exécuté » de T1.
- FG-CSP-authoring → `not_applicable` (SARIF ; jamais un faux `passed`).
- Gate(s) processus = `lint_planning_data(&state.root)` (process.rs:425, VÉRIFIÉ read_dir borné sur `.planning/active`, idempotent, DÉJÀ GET-safe via /api/lint) → fournit le « ≥1 passed ».
- **Pas de VERSION** (API loopback interne, pas un wire propagé).

### 3.2 Shape `issues`
**Décision C** (message-only sur `GateResult`, NE PAS refactorer le cœur publish) + struct-vue route-local `GateIssueView{message, file:Option<String>, line:Option<u32>}` peuplée depuis `LintDiagnostic` (DÉJÀ `Serialize`, process.rs:418-423). **Insight décisif vérifié** : les FG étant `not_run`/`not_applicable`, leur `Vec<String>` plat N'ATTEINT JAMAIS la route → rattachement gate↔fichier obtenu sans toucher gates.rs ni parser. A rejeté (casse pipeline.rs/atelier.rs/Display/tests = cœur publish non-délégable S79). B rejeté (band-aid).

Shape JSON exacte (enveloppe objet, miroir `handle_providers`/`handle_git_diff`, 0 agrégat top-level) :
```json
{
  "gates": [
    { "gate": "FG5-sandbox",      "status": "not_run",        "issues": [] },
    { "gate": "FG-CSP-authoring", "status": "not_applicable", "issues": [] },
    { "gate": "lint-planning",    "status": "passed",         "issues": [] },
    { "gate": "lint-planning",    "status": "blocking",
      "issues": [ { "message": "review still at PASS-PENDING", "file": "sprintN_phase_x_review.md", "line": null } ] }
  ]
}
```
- `status` = enum string 5 valeurs DISTINCTES jamais aplaties : `not_run` / `not_applicable` / `passed` / `informational` / `blocking`. JAMAIS `passed:bool` brut, JAMAIS de champ racine `overall`/`all_passed`/`verdict`/score.
- `GateIssueView { message: String, file: Option<String>, line: Option<u32> }` (line toujours `None` en S80 — `LintDiagnostic` n'a pas de champ ligne, VÉRIFIÉ process.rs:418-423).
- `message` = libellé de règle uniquement, JAMAIS la valeur d'un secret (FG6 émet `pattern_name`, pas la valeur ; et étant `not_run`, ses issues n'atteignent pas la route — invariant TESTÉ par « FG carry 0 issue »).

### 3.3 Conséquence V5/V6
V5 **livrable au niveau fichier** S80 (`LintDiagnostic.file`), pas de ligne (LintDiagnostic n'a pas de champ ligne). V6 **livrable** S80 (groupement par gate + file). Carry P1 S81 réduit : refactor `GateIssue{path,line?,message}` pour la ligne fine + restitution d'un GateResult publish persisté (inexistant aujourd'hui). **Acter au plan** : remplacer le binaire « A=>S80 / C=>carry » par « C + vue structurée => V5/V6 niveau fichier S80 ».

---

## 4. Approche d'implémentation (miroir 1:1 du précédent F `bb35d39`)

**Handler** (`operator_server.rs`, à côté de :1322) :
```rust
/// Sprint 80 Phase G: the live registry of Factory gates as a 1:1 diagnostic —
/// each entry restitutes a distinct status (not_run/not_applicable/passed/
/// informational/blocking), never an aggregated verdict. Reads state.root, 0
/// user input, runs no publish scan on GET. Envelope {gates:[...]}.
async fn handle_gates(State(state): State<OperatorState>) -> Json<serde_json::Value> {
    Json(serde_json::json!(crate::gates::gates_live_data(&state.root)))
}
```
0 input → pas de validation `is_safe_git_rev`/Path. `workspace = state.root` EN DUR.

**Route** (sous-routeur `authed`, juste après :197) : `.route("/api/gates", get(handle_gates))`. Hérite bearer constant-time + Host anti-rebinding + Origin + chemin cookie + CSP self-origin. Lecture seule sans input → CSRF en lecture sans effet ; ne déclenche pas le trigger « endpoint write/spawn ».

**Logique** dans `crate::gates` (cohésion avec GateResult + noms de gate) — `pub fn gates_live_data(root: &Path) -> GatesView`. Structs-vue neuves `#[derive(Serialize)]` (NE PAS sérialiser GateResult : Debug-only, `passed:bool` aplatit) : `GatesView{gates}`, `GateEntryView{gate,status,issues}`, `GateStatus` enum snake_case `{NotRun,NotApplicable,Passed,Informational,Blocking}`, `GateIssueView{message,file,line}`.

Construction : FG4/FG5/FG6/FG7/FG8 → `NotRun` (issues vides) ; FG-CSP-authoring → `NotApplicable` ; puis `lint_planning_data(root)` → **mapping mustFix** : si `errors` non vide → entrée `Blocking` (issues=errors) ; si `warnings` non vide → entrée `Informational` (issues=warnings) ; si `errors.empty() && warnings.empty()` → entrée `Passed`.

**Named constants** (S4 §4, README §6.9) : centraliser les 6 noms FG (+ `lint-planning`) en `pub const GATE_*: &str`, utilisés à la construction des `GateResult` (definition point, dans gates.rs) + au registre. **Scope minimal** : ne pas réécrire les call-sites publish de comparaison (pipeline.rs `.contains("FG4")`).

**Provenance doc-comment** : présent immuable (anti STALE-PHASE-K §6.12), jamais promissoire.

**Tests double couche** (miroir F) :
- (a) **Unit hermétique** `gates.rs#[cfg(test)]` : `gates_live_data` sur tempdir-fixture → ≥1 `NotRun` ET ≥1 `Passed` DÉTERMINISTE (fixture propre → `.planning/active` absent → lint ok → Passed) + FG carry 0 issue (secret non-leak structurel) + FG-CSP `NotApplicable` + un 2e test mustFix (fixture avec erreur STALE_PASS_PENDING + warning ORPHAN_FILE → entrée `Blocking` ET entrée `Informational` distinctes, `file` restitué, `line` None).
- (b) **HTTP** `tests/operator_server.rs` (`server.get().json()`) : `operator_gates_endpoint` (200 + `body["gates"].is_array()` non vide + 0 champ racine `overall`/`passed` + chaque entrée porte `status` string & `issues` array, jamais `passed:bool`). Shape-only sur le repo live (non-déterministe) ; sémantique → unit hermétique. + `operator_gates_requires_auth` (401 sans token, prouve l'enregistrement dans `authed`).

**Front** : **différé Phase H** (mirroir strict du précédent F qui a livré la route + tests Rust, et a différé le client TS + le câblage à H ; operator.ts:217 / OrientationBar / useRailStatus forward-référencent déjà `/api/gates`). Phase G = backend-pur, 0 fichier front touché.

---

## 5. Risques résiduels / cibles adversariales

1. **« lint_planning_data tourne sur GET » ≈ contradiction du « 0 run sur GET »** — DÉFENSE durcie et VÉRIFIÉE : borné (`read_dir` non récursif sur `.planning/active`, PAS WalkDir), early-return `ok:true` si dossier absent, déterministe, idempotent, DÉJÀ exposé en GET via /api/lint. La prohibition vise les scans publish FG (WalkDir+regex repo-wide).
2. **Nomenclature « gate » pour lint** — DÉFENSE : gate de discipline réel, restitué 1:1, jamais agrégé ; nom explicite (`lint-planning`) distingue le scope repo-discipline du scope publish.
3. **T1 fragile si lint renvoie Blocking sur le repo réel** — l'assertion « ≥1 passed » DOIT être hermétique (fixture tempdir propre), pas live. Le test HTTP live reste shape-only.
4. **Fuite chemins repo via issues** (sévérité L) — delta faible vs /api/git/diff déjà exposé ; invariant TESTÉ : FG carry 0 issue (le secret-finding FG6 n'atteint jamais la route).
5. **Dérive de scope vers A** — refuser fermement le refactor de `GateResult.issues` (cœur publish testé non-délégable S79).
6. **Named constants** — garder le minimum (definition points gates.rs + registre), ne pas réécrire les comparaisons publish.

---

## 6. Scope

**DANS le scope S80 Phase G** :
- `GET /api/gates` read-only, 0 input, route additive dans `authed`, miroir `handle_git_diff`.
- `gates_live_data` + structs-vue (`GatesView`/`GateEntryView`/`GateStatus`/`GateIssueView`).
- Registre statique FG (not_run/not_applicable) + gate processus lint (passed/blocking/informational) avec mustFix errors/warnings.
- Named constants des 6 noms FG (+ `lint-planning`) utilisées aux definition points + registre.
- Tests unit hermétique (2) + HTTP (2).
- V5/V6 au niveau fichier (rattachement gate↔fichier via `LintDiagnostic.file`).

**HORS scope (carry/différé)** :
- Front (client TS `getGates` + câblage rail/panneau gates) → **Phase H** (mirroir F).
- Refactor `GateResult.issues -> Vec<GateIssue{path,line?,message}>` (carry P1 S81 ; touche le cœur publish).
- `line` fin sur les issues / `line` sur LintDiagnostic (carry P1 S81).
- Persistance de gate-log + restitution d'un GateResult de publish ENREGISTRÉ (inexistant aujourd'hui).
- POST `force run` d'un gate frais (unsafe, hors-scope S80).
- Champ VERSION/schema_version (non pertinent).
- Réécriture rétroactive des call-sites publish (comparaisons pipeline.rs) pour les named constants.
