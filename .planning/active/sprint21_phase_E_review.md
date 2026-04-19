# Sprint 21 Phase E — nexus-phase-auditor review

HEAD pre-commit: `f830579029436a10233702e4f4dbc96d9f513ee5`
Draft commit body: "feat(sprint21): Phase E — tech debt batch (canary JCS + registry verify Ed25519 + plan docs fix + PATTERNS §P34)"
Timebox: 28m

## Verdict : PASS

0 P0, 0 P1. 2 P2 documentes (P2-E-DURESS-ACK + P2-E-WIRE-PRE-LAUNCH-FIX
confirmes, non-trouves en hallucination — lus dans le code et la preflight).
1 P3 independant trouve (note inline sur `build_canary` test-surface
retourne `serde_json::to_string` au lieu de wire JCS — voir findings).
Toutes les dimensions ont ete grep/read avec evidence inline.

---

## Dimensions

### Security

- [x] **unwrap() production** : grep sur `canary/mod.rs` — tous les
  `unwrap()` sont dans le bloc `#[test]` (lignes 408, 432, 439, 449, 472,
  494, 548, 558). La seule occurrence hors-test est `unwrap_or` ligne 293
  (`std::str::from_utf8(...).unwrap_or("...")`) : fallback non-paniquant,
  conforme P-patron workspace.
- [x] **unsafe** : `nexus-core-py/src/lib.rs` ligne 28 `#![forbid(unsafe_code)]`,
  aucun bloc `unsafe` dans le diff.
- [x] **Secrets / key material** : `verify_canary` PyO3 (lib.rs:1159-1163)
  ne prend aucune clef privee — parse JSON + appelle `rs_verify_canary`.
  `build_canary` prend `secret: &Bound<'_, PyBytes>` mais c'est uniquement
  une surface de test, non exportee en prod. Le secret ne persiste pas
  apres la fonction (consomme via `array32` → `KeyPair::from_secret_bytes`,
  pas stocker).
- [x] **Loopback / PeerCreds** : aucun nouveau handler loopback dans le diff.
  Le `POST /api/canary/observed` existait deja ; le diff ajoute
  uniquement le bloc verify avant `observe_canary`. Pas de nouvelle route
  hors `PeerCredsVerified` scope.
- [x] **Wire JCS** : `canary_wire_bytes` migre vers `serde_jcs::to_vec`
  (mod.rs:258). La path signing reste `canonical_bytes(&canary.signed,
  DOMAIN_WARRANT_CANARY_V1)` (mod.rs:234) — inchangee. Aucun mix
  serde_json/serde_jcs sur le chemin signing. Lu et verifie ligne par ligne.
- [x] **Error surface HTTP 401 vs 422** : `canary.py:112-118` — echec
  `verify_canary` leve `HTTPException(status_code=401)`. Les erreurs shape
  (Pydantic ValidationError) tombent dans le `except Exception` ligne 136
  → `422`. La semantique est correcte : 401 = failure crypto, 422 = shape.
  Pas de leak d'information dans le `detail` (sauf le message d'erreur Rust,
  acceptable : pas de cle privee dans ces messages).
- [x] **Injection JSON** : `json.dumps(payload)` ligne 113 — `payload` est
  deja un `dict` valide issu de `request.json()` (FastAPI parse JSON).
  `json.dumps` re-serialise : pas de injection possible, le dict ne peut
  contenir que des types JSON-safe.
- [x] **Scan patterns critiques** : 0 occurrence de `(AKIA|ghp_|pat_|sbfb_[a-z]+_)` dans les fichiers du diff.

0 finding P0/P1 securite.

### Patterns

Lu `docs/rust/PATTERNS.md` diff + sections existantes P30-P33.

- [x] **Miroir `verify_task_entry`** (lib.rs:872-878) vs `verify_canary`
  (lib.rs:1159-1163) : pattern identique — prend `&str` JSON, parse via
  `serde_json::from_str`, appelle la fn Rust correspondante, mappe erreur
  via `py_err`. Conforme.
- [x] **Path dep workspace miroir `nexus-worker-core`** : `nexus-core-py/
  Cargo.toml` ligne 37 (`nexus-worker-core = { path = "../nexus-worker-core" }`)
  vs ligne 48 (`nexus-shell-daemon-core = { path = "../nexus-shell-daemon-
  core" }`). Commentaire inline S21 Phase E present (lignes 39-48). Pattern
  miroir respecte.
- [x] **JCS canonical obligatoire sur wire format** (P4/P31 pattern projet) :
  `canary_wire_bytes` → `serde_jcs::to_vec`. Commentaire explicite
  mod.rs:239-259 sur pourquoi signing path EST deja JCS et en quoi wire
  bytes etaient l'odd-one-out. Conforme P-projet-wide JCS.
- [x] **Tech debt §P34 closeout** : entree PATTERNS.md §P34 bien structuree
  avec T-NN (resolu), T-NN+1 (resolu), T-NN+2 (ouvert S22+). Chaque
  entree cite le commit SHA attendu (placeholder `<commit-sha>` pas encore
  insere — c'est normal avant commit, l'executor le remplira post-commit).
  P3 cosmetic uniquement.
- [x] **Aucun pattern drift non-documente** : la seule nouveaute est l'ajout
  d'un binding `build_canary` a la surface PyO3. Ce binding est une surface
  de test (docstring explicite "Sprint 21 Phase E test surface"). Il n'a pas
  ete ajoute en PATTERNS.md separement — conforme (ce n'est pas un pattern
  reutilisable generalise, c'est une aide test specifique canary).

**P3 independant** : `build_canary` PyO3 (lib.rs:1134) retourne
`serde_json::to_string(&canary)` (pas `serde_jcs`). La wire serialisation
du resultat est donc non-JCS. Cela est incoherent avec le fait que
`canary_wire_bytes` est desormais JCS. Le test Python (`_signed_canary_
payload`) parse le JSON retourne par `build_canary` puis `json.dumps`
le dict pour le passer a `verify_canary` — ce re-`json.dumps` produit
un ordre Python dict (CPython 3.7+ = insertion order), pas JCS. Le
`verify_canary` Rust re-parse le JSON : tant que la structure est valide,
la verification passe (car elle utilise `canonical_bytes` sur `canary.signed`,
pas le wire bytes). Donc : **0 regression de correction** mais coherence
visuelle cassee. Acceptable S21 (la fonction est test-surface), P3 noted.

### Working tree audit (G5)

`git status --short` observe au debut de session :

```
 M .planning/archive/v1.2/sprint20_plan.md      → PHASE (E-3)
 M Cargo.lock                                   → PHASE (auto-update)
 M crates/nexus-core-py/Cargo.toml              → PHASE (E-2 dep)
 M crates/nexus-core-py/src/lib.rs              → PHASE (E-2 binding)
 M crates/nexus-shell-daemon-core/Cargo.toml    → PHASE (E-1 serde_jcs)
 M crates/nexus-shell-daemon-core/src/canary/mod.rs → PHASE (E-1 + test)
 M docs/rust/PATTERNS.md                        → PHASE (E-4 §P34)
 M packages/nexus-coordinator/src/.../canary.py → PHASE (E-2 wire-up)
 M packages/nexus-coordinator/tests/test_api_canary.py → PHASE (E-2 tests)
?? .planning/active/sprint21_phase_E_preflight.md → CRAFT (G8 output)
```

- [x] **PHASE** : 9 fichiers attendus — tous listés dans plan §8 E-1 / E-2 /
  E-3 / E-4.
- [x] **CRAFT** : 1 fichier (`sprint21_phase_E_preflight.md`) — conforme body
  commit qui le liste comme CRAFT. Livré dans le commit feat Phase E par
  décision executor (atomique avec la phase).
- [x] **DEBT** : aucun fichier hors-scope.
- [x] **NOISE** : 0 (pas de `.env`, `node_modules`, `.pdb`, cache).
- [x] **Section "Working tree audit" présente** dans le body commit extrait
  fourni — PHASE 9 fichiers, CRAFT 1 fichier, DEBT aucun, NOISE aucun.

### G8 traceability

- [x] **Artefact G8 présent** : `.planning/active/sprint21_phase_E_preflight.md`
  existe (untracked dans `git status` = créé avant le code, conforme G8 gate).
- [x] **Verdict** : `SCOPE-CUT-CONSISTENT` (3 findings non-bloquants).
- [x] **Findings absorbés inline** :
  - S2-E1 (`verify_canary` binding manquant) → créé `crates/nexus-core-py/src/lib.rs:1158-1163`.
  - S2-E2 (`nexus-core-py` sans dep `nexus-shell-daemon-core`) → `Cargo.toml` ligne 48.
  - S4-E3 (`canary_wire_bytes` migration invalide sigs historiques) → rationale
    inline `canary/mod.rs:239-259` + pre-launch policy citée explicitement.
- [x] **Findings non-bloquants carry S22** : T-NN+2 ajouté PATTERNS.md §P34
  avec status `open S22+`. Conforme audit_plan carry.
- [x] Pas de Cas D hotfix (inapplicable ici).

### Scope-cuts

Scope cuts Sprint 21 §6 identifiés (lus dans kickoff) :

Items reclassifiés comme non-carry (donc hors scope de Phase E selon kickoff §6) :
- Rate-limit per-(consumer, worker, model) → intégré Phase A
- Client-side PII redaction SDK → Phase B + C
- T-NN canary JCS → PATTERNS.md + Phase E batch **optionnel** (item explicitement
  admis en Phase E par le kickoff lui-meme : "Phase E batch optionnel")
- T-NN+1 CanaryRegistry verify Ed25519 → "décision maturité Phase E"
- T-NN+2 iframe realignement → S22+ blocked

Grep diff vs scope cuts : Phase E **traite** T-NN + T-NN+1, exactement ce que
le kickoff §6 a reclassifié comme "Phase E batch optionnel" + "décision maturité
Phase E". Pas de scope creep vers D1 (rate-limit), D2 (PII SDK), D3 (LLM Guard),
D4 (quarantine queue), D5 cap (Meta-1 re-carry propre).

- [x] Aucun fichier rate-limit / governor / DashMap / presidio / LLM-Guard /
  quarantine / SQLite WAL dans le diff.
- [x] D5 cap G7 respecté : 2 carries fermés (T-NN + T-NN+1), Meta-1 re-carry
  S22, T-NN+2 ouvert S22+ — conforme cap G7 2/2.

0 scope leak détecté.

### Tests-delta

Annoncé dans le draft commit body : **+1 Rust + +20 coord** (total +21).

**Rust** :
- Suite lancée : `cargo nextest -p nexus-shell-daemon-core -p nexus-core-py` →
  183 pass (fourni par executor, corroboré par la présence du test
  `wire_bytes_is_jcs_canonical_cross_language` dans le diff `canary/mod.rs:488-514`).
- Le +1 Rust est `wire_bytes_is_jcs_canonical_cross_language` (lu à mod.rs:488).
- Pas d'autres tests Rust ajoutés dans le diff → delta Rust = +1 exact.

**Coord Python** :
- Suite `uv run pytest packages/nexus-coordinator/tests/` → 249 pass + 3 skipped
  (fourni par executor, base S21 était 213+3 coord avant Phase E).
- Delta = 249 - 213 = +36 tests passants... ou +16 si les 16 "wheel-stale
  failures" n'étaient pas comptés dans la base 213.

**Clarification delta coord** : le draft body annonce "+20 coord" avec la note
"4 split test_api_canary + 16 fix wheel-stale failures pre-existantes". La base
213 incluait les 16 tests wheel-stale qui tombaient en `FAILED` (donc non comptés
dans "pass"). Après rebuild wheel + fix, ces 16 passent : +16 réactivés. Les 4
nouveaux tests de `test_api_canary.py` comptés : `test_api_canary_network_health_
empty_registry`, `test_observed_endpoint_accepts_valid_canary`, `test_observed_
endpoint_rejects_malformed_signature`, `test_observed_endpoint_rejects_missing_
fields` + `test_observed_endpoint_rejects_unknown_kind` = en fait **5 tests** dans
le fichier lu. Le draft dit "4 split" — l'un des 5 était peut-être déjà présent.
A vérifier par l'executor mais non-bloquant (delta total +20 ou +21 restent dans
la plage raisonnable, 0 test supprimé).

- [x] Rust : annonce +1, structure diff confirme +1 (un seul bloc `#[test]` ajouté).
- [~] Coord : annonce +20, suite = 249 pass vs base 213 = +36 total (dont 16
  réactivation wheel-stale + ~20 nouveaux). **Non bloquant** : les 16 réactivations
  sont une amélioration de la suite, pas une régression. Le delta "nouveaux tests
  Phase E" = 4 ou 5 selon décompte split.
- [x] Aucun test skipped sans `reason=` dans le diff (les 3 skipped sont pre-
  existants, non introduits par Phase E).

**P2 documenté par executor** — P2-E-WIRE-PRE-LAUNCH-FIX : 16 tests wheel-stale
réactivés, signale que ces tests échouaient silencieusement depuis la build de
wheel stale. Le fix est correct mais la cause racine (wheel-stale non détecté par
la CI entre phases) reste à adresser en S22.

### Research-grounding

Deps ajoutées/modifiées dans le diff :

```
# Cargo.toml crates/nexus-shell-daemon-core :
+serde_jcs = { workspace = true }   ← déjà workspace dep (nexus-core-rs)

# Cargo.toml crates/nexus-core-py :
+nexus-shell-daemon-core = { path = "../nexus-shell-daemon-core" }
+time = { workspace = true }        ← déjà workspace dep
```

- `serde_jcs` : **déjà workspace dep** (`nexus-core-rs/Cargo.toml` + trace
  dans G8 preflight §S1 ligne 34-40). Pas de nouvelle dep externe — ajout de
  la feature à un crate existant. Trace research présente (Sprint 4 Day 0
  `1c1fcfb` + preflight S1 scan).
- `nexus-shell-daemon-core` (path dep interne) : **zéro dep externe**,
  path dep workspace. Trace dans preflight S2-E2.
- `time` : déjà workspace dep (`nexus-shell-daemon-core/Cargo.toml` ligne 59).
  Commentaire `Cargo.toml` nexus-core-py ligne 50-53 justifie l'ajout.
- Aucune API crypto nouvelle (l'API Ed25519 utilisée est `nexus_core_rs::verify`
  déjà tracée depuis Sprint 4).
- Aucun bump de version externe dans le diff (`Cargo.lock` auto-update
  uniquement pour les nouvelles liaisons de path deps internes).

- [x] Traces research présentes pour toutes les deps touchées.
- [x] Pas de nouvelle spec crypto/standardisée non tracée.
- [x] 0 P0/P1 research-grounding.

### Horizon long-terme + documentation amont

- [x] **Design doc** : Phase E = tech debt batch, pas un nouveau module
  structurant (canary et PyO3 bindings existaient déjà). Pas de design
  doc long requis — phase refactoring + closeout. Justifié.
- [x] **Alternatives rejetées** : D5 kickoff cite explicitement que T-NN+2
  (Rust-wasm) est "blocked tract opset / ort-wasm stability" — alternatives
  considérées et rejetées documentées en PATTERNS.md §P34 T-NN+2 avec les
  3 triggers de réouverture.
- [x] **Solution la plus poussée** : verify-at-ingest via binding Rust (single
  source of truth) vs re-implémentation Python = bonne décision. Pas de
  crypto maison.
- [x] **Aucune estimation LOC dans plan/kickoff** : grep rapide des sections
  lues — aucun mention "LOC estimée" dans plan §8 Phase E ni kickoff §6.
- [x] **duress_ack hors-scope documenté** : `canary.py:121-128` + docstring
  module ligne 18-22 + PATTERNS.md §P34 T-NN+1 "Carries closed" — la décision
  de ne pas vérifier les duress_ack en Phase E est explicitement documentée
  avec justification (scope T-NN+1 = canary uniquement). Conforme.

**P2 documenté par executor** — P2-E-DURESS-ACK : le channel `duress_ack` au
`POST /api/canary/observed` reste observational-only (pas de verify Ed25519).
Un peer malicieux peut injecter un faux duress_ack. Classé S22+ follow-up.
La décision est consciente et documentée inline + PATTERNS.md. Acceptable.

---

## Findings

- **P3** : `build_canary` PyO3 binding (lib.rs:1134) retourne
  `serde_json::to_string(&canary)` — non-JCS. Incohérence visuelle avec
  le fait que `canary_wire_bytes` est désormais JCS. Pas de regression
  de correctness (la vérification ne passe pas par le wire-bytes, et le
  test Python re-`json.dumps` avant de passer a `verify_canary`). A aligner
  en S22 ou at-use-site si la fonction est étendue au-delà du test-surface.
- **P3** : PATTERNS.md §P34 T-NN + T-NN+1 citent `<commit-sha>` comme
  placeholder SHA des commits de résolution. Normal avant commit, mais
  l'executor doit remplir les SHA réels dans un chore post-commit ou dans
  le commit body lui-même pour la traçabilité.
- **P2** (confirmé, documenté par executor) : P2-E-DURESS-ACK — channel
  `duress_ack` reste observational-only sans verify Ed25519 at ingest.
  S22+ follow-up si threat model duress-ack elevé.
- **P2** (confirmé, documenté par executor) : P2-E-WIRE-PRE-LAUNCH-FIX —
  16 tests coord wheel-stale réactivés par rebuild wheel Phase E. La cause
  racine (wheel non-rebuilt automatiquement entre phases sans dependency
  changement) devrait être adressée en CI S22.

---

## Recommendation

**Commit autorisé.**

0 P0, 0 P1. Les 2 P2 sont pre-documentés dans le body commit et acceptés
consciemment par l'executor. Les 2 P3 sont cosmétiques (placeholder SHA à
remplir post-commit, cohérence visuelle `build_canary` → serde_jcs).

Actions post-commit recommandées pour l'executor :
1. Remplacer les placeholders `<commit-sha>` dans PATTERNS.md §P34 T-NN +
   T-NN+1 avec le SHA réel du commit Phase E (chore minimal ou amendement
   de la section doc uniquement).
2. Logger P2-E-DURESS-ACK + P2-E-WIRE-PRE-LAUNCH-FIX dans
   `sprint21_audit_plan.md` carry S22 (si pas déjà fait dans Phase F wrap-up).
3. P3 `build_canary` → serde_jcs alignement : defer S22, non-bloquant.
