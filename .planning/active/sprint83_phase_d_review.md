# Sprint 83 Phase D — Review

**Phase** : D — Extraction du contrat Factory (`FactoryProcessProfile.v1`).
**Méthode** : Workflow ultracode `wf_efde68fd-2c7` — 7 agents Opus 4.8 1M (5 dimensions
review + vérification adversariale + synthèse). 838 131 tokens, 133 tool-calls, 0 erreur.
Verdicts dimensions : D1 correctness/tests = PASS, D2 honnêteté/grounding = PASS, D3
invariants/sécurité/wire = PASS, D4 patterns/simplification = PASS, D5 livrables/body/carry =
CONCERN (body pas encore écrit — advisory).

## Verdict: PASS

**Synthèse Workflow** : 0 P0/P1 confirmé. 12 findings CONFIRMED, tous **P3**
(design-accepté / cosmétique / précision-doc). L'unique P2 (gates.rs anchored-not-digested)
correctement rétrogradé en P3 (claim d'enforcement test-protégée + detect_stale non-autoritaire).
Le CONCERN D5 était advisory (le commit body doit porter Scope cuts + Carry closure).

## Polish appliqué post-review (adressant les findings avant Codex)

1. **FG8 → Enforced (correction d'honnêteté, la plus substantielle)**. Le reviewer a confirmé
   sur `crates/sbfb-factory/src/pipeline.rs:73-80` que `run_gate_fg8_provenance` hard-block
   **inconditionnellement** (hors `if !skip_gates`, `return Err`), EXACTEMENT comme
   FG-CSP-authoring (:55-62). Marquer FG-CSP Enforced mais FG8 Documented était incohérent
   sous ma propre définition d'axe (« non-bypassable »). Corrigé : FG8 = Enforced, ancre
   `gates.rs:run_gate_fg8_provenance`. Enforced count 5 → **6** (G1, G5, FG-CSP, FG8, 2 hooks).
   FG4 re-titré « recorded, non-blocking » ; FG5/FG6 « relaxable via --skip-gates ».
2. **Disclosure asymétrie ancre-vs-digest** (module-doc) : les ancres pointant dans un fichier
   AUSSI digéré (G1/G5/2 hooks → hooks digérés) gagnent la protection PROFILE_STALE runtime ;
   les 2 ancres publication pointent dans `gates.rs` (NON digéré, churny) → protection test-time
   seulement (le contrôle négatif de `documented_vs_enforced_marked` rougit la CI si le symbole
   disparaît). Rend le choix PLAN-ADAPT vs plan D.1 (« code courant » digéré) honnête et explicite.
3. **Needles resserrés** (robustesse anti-rot) : G5 `staging`→`coherence staging` ;
   hook lightcheck `Codex`→`Codex review`. Ancres publication `run_gate_csp_authoring` /
   `run_gate_fg8_provenance` déjà spécifiques ; auditor `## Verdict: PASS` spécifique.
4. **lib.rs § Phase A status** : « seven … empty bodies » → « most bodies are still empty
   stubs ; the first filled is FactoryProcessProfile.v1 (Phase D) » (miroir contracts.rs, fin
   de l'asymétrie doc cross-module).
5. **Couverture Display** : nouveau test `error_display_names_source_and_reason` exerce
   `ExtractError::Display` + les 2 bras `StaleReason` (Missing / DigestMismatch). Crate
   18 → **19 tests**.
6. **Précision canonicalise** : doc clarifie « EVERY `\r` byte (not only paired CRLF) …
   acceptable : sources markdown/shell, single trust domain, jamais binaire ».

Re-vérif post-polish : `cargo nextest -p sbfb-workflow-core` 19/19 ; clippy/fmt verts ;
`check-frontier-contracts` exit 0 (census 25, len==7, `// FRONTIER:` prose sans colon-espace) ;
`check-factory-docs` exit 0 ; golden régénéré (6 enforced, needles résolus).

## P3 restants documentés (non-bloquants, cf. §4.5 loop stop criterion)

- `factory_profile.rs` `iter().find()` first-match (extract/detect_stale) — un SourceId
  dupliqué à bytes divergents serait silencieusement dédupliqué ; edge-only, aucun caller réel
  ne passe de doublon (les inputs sont construits depuis `SourceId::ALL`). Non corrigé (edge
  théorique) — l'ordre de sortie reste déterministe (itération sur `SourceId::ALL`).
- `factory_profile_schema()` mineure redondance avec `all_contract_schemas()['FactoryProcessProfile.v1']`
  (ce contrat EST dans le registre, contrairement au frère `resolved_profile_schema`) —
  parity-defensible (accessor public propre, utilisé par `schema_title_is_the_identity`).
  Conservé.

## Vérification §7.4 (les 3 blocs + dual-platform)

- **Rust Windows** : fmt ✓, clippy `--workspace --all-targets -D warnings` ✓, nextest workspace
  (delta +9 : 2137 → 2146 ; 1 flake env `dispatch_loop::boot_feed_namespace_reenters` confirmé
  vert en solo 14/14), doctest ✓, release `nexus-shell-daemon` ✓.
- **Rust Docker sbfb-ci (rust:1.94, Linux)** : `sbfb-workflow-core` 19/19 PASS — prouve le
  déterminisme cross-OS du digest (extract_matches_golden vert sur les 2 OS).
- **Frontend web** : lint ✓, tsc ✓, vitest **529/529** (0 changement web, T1 = N-A).
- **Operator** : vitest **201/201** (0 changement operator).

## Livrables D.2 (tous présents)

1. `FactoryProcessProfile.v1` schéma schemars complet (roles 9, modes 10, G1-G10, FG0-FG10 +
   FG-CSP-authoring, 2 hooks finalité, preflight scans, body 9 sections, blocs vérif, audit
   tracks, authority order, invariants). 2. Extraction lieuse aux digests blake3 de 6 sources.
   3. Détecteur `PROFILE_STALE` fail-closed nommant la source dérivée. 4. Marquage
   `documented|enforced` par règle, ancre mécanisme résolue pour les Enforced. 5. Golden
   `factory_profile.golden.json` STRICT. 6. Étiquette docs-contrat frontière (commentaire prose).

## Carry closure

- **Ferme G-FACTORY-CONTRACT** (gate_matrix row 28) : 3 buckets séparés (process/publication/
  finality), enforced count==6 prouvé par test, PROFILE_STALE bloquant testé rouge-avant/bloc-après.
- **Définit le contrat candidat G-REVIEW-INDEPENDENCE** (row 29, runtime-fermé AB) : invariants
  `codex_output_rewritable=false`, `reconciliation_required_before_pass=true`,
  `pass_pending_committable=false`, `gate_check_is_authority=false` + rôles distincts
  driver/reviewer/codex-verifier/auditor.

## Scope cuts (à consigner dans le body)

(a) `audit_tracks` lié à AGENT_SYSTEM §5.4 = **9**, PAS le canon 11-track de README §7.1 (J/K
standing) — divergence notée in-code. (b) Clôture GUIDE + `llms.txt` de `FactoryProcessProfile.v1`
**différée à AJ1** (AM-3) — étiquette per-phase = prose + goldens ; `check-factory-docs` exit 0
sans elle. (c) PLAN-ADAPT vs plan D.1 : `gates.rs` (« code courant ») lié par MechanismAnchor
(résolu au test) et **non** par digest (churny → cry-wolf) — disclosure honnête au module-doc.

## Codex reconciliation

Codex GPT-5.6 Sol (`--sandbox read-only`, `codex exec -o`, output brut non réécrit). **7 tours
de raffinement adversarial** sur la sémantique d'enforcement (le cœur honnêteté du contrat) —
chaque tour a rapproché le marquage de la vérité, 0 P0 sur tout le parcours :

- **R1** : G8 marqué Documented mais le hook Claude Check 8 le bloque → G8 Enforced ; `detect_stale`
  fail-open sur sources vides/partielles → itère `SourceId::ALL` fail-closed.
- **R2** : doublon de source (first-match masque une dérive) + `path` jamais vérifié → rejet des
  doublons (2 côtés) + check `path == id.path()` ; `StaleReason::Malformed`.
- **R3** : `detect_stale` vérifiait les 6 digests mais PAS le contenu → check d'intégrité contenu
  complet (`canonical_profile`, `ContentMismatch`, `source_id: Option`).
- **R4** : `TOOLING.md` = les hooks Claude ne sont PAS la source de vérité ; l'autoritaire
  `agentctl` ne fait que WARN le preflight → **G8 → Documented** ; vecteur réordonné passait →
  check d'ordre `.eq(SourceId::ALL)`.
- **R5** : FG8 « non-delegable » FAUX (skippé redeploy + exception N0) → **FG8 → Documented** (seul
  FG-CSP vraiment non-délégable) ; axe reframé honnête (« bloque sur le chemin autoritaire en
  opération normale », pas « impossible à contourner ») ; **SCOPE CUT disclosé** (fichiers-mécanisme
  churny non-digérés → dérive sémantique non couverte par PROFILE_STALE).
- **R6** : titre auditor « exact `## Verdict: PASS` » sur-précis (agentctl tolère un `:` espacé) +
  « non-bypassable » résiduel dans le doc d'enum → titre softené + doc d'enum reframé ; **jugé P2**
  (sur-précision de titre + tolérance qui est un détail d'agentctl, pas du code Phase D).
- **R7** : artefact final brut sur l'arbre corrigé (cohérence body/hook Check 7).

Enforced : 7 → 6 → **5** (G1, G5, FG-CSP-authoring, phase-precommit-lightcheck, phase-auditor-gate).
**Résiduels documentés (P2/P3, boucle stoppée au critère « CLEAN ou P2/P3 documentés »)** : (a) les
markings d'enforcement dérivent des fichiers-mécanisme autoritaires (agentctl/pipeline/gates/
TOOLING) NON digérés — dérive sémantique de ces fichiers non captée par PROFILE_STALE (scope cut
disclosé module-doc, future work) ; (b) l'auditor-gate tolère `## Verdict : PASS` espacé (leniency
agentctl, hors code Phase D) ; (c) exemption G1 déclarée + scope phase-commit des hooks (disclosés).
Aucun P0/P1 non-adressé subsiste ; ce qui est fail-closed prouvé = le CONTENU du contrat (rôles,
modes, identités de gate, body, invariants) vs ses sources doc/hook.
