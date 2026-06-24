# Sprint 79 Phase C — Preflight G8 (prompt-kind `app-authoring`)

**Méthode** : Workflow ultracode `w0m6am3p5` (6 agents Opus 4.8 1M, 500 K tokens,
89 tool-calls) — fan-out 5 scans factuels (S1a content-grounding / S1b deps / S2
décisions historiques / S3 threat / S4 contrat+gate Phase B) + synthèse adversariale.
Faits load-bearing re-vérifiés en main-thread (3 spot-checks PRIMITIVES.md).

## Verdict : EXECUTE

Le plan §Phase C tient tel quel. Phase C = additive pure : ajouter `"app-authoring"`
à `PROMPT_KINDS` (`process.rs:7-16`) + créer `prompts/agent/app-authoring.md`. Aucune
décision Day-0 gelée n'est contredite.

### Faits concordants (5 scans, vérifiés adversarialement)
1. **Mécanisme** — `prompt_filename` (`process.rs:72-77`) résout `app-authoring` →
   `app-authoring.md` via le bras générique `format!("{other}.md")`, **sans cas spécial
   ni alias** (D2 « aucun alias requis » mécaniquement exact). PROMPT_KINDS a 8 kinds,
   `app-authoring` absent.
2. **Étiquette drift-gatée** — `prompt_kinds_resolve_to_existing_files`
   (`process.rs:888-905`) itère PROMPT_KINDS et assert l'existence du `.md` → kind sans
   fichier = build ROUGE. C'est l'étiquette générée drift-gatée de cette **frontière LLM**
   (§6.12). La gate ne peut pas pourrir.
3. **Contenu ancré** — les 5 marqueurs T1a sont couvrables verbatim ; le contrat
   `README.md:67-68` énumère les 5 pièges canoniques. **Ancrage = PRIMITIVES.md/README,
   PAS synthesis.json** (qui n'a pas de clé csp/pitfalls ; il fait autorité pour
   cross_products/novelty_levers/novelty_heuristic uniquement).
4. **0 dep** — donnée statique (un `.md` + un `&'static str`), `Cargo.lock` inchangé.
5. **0 bump wire** — `process.rs` ne lit aucun `*_FORMAT_VERSION` ; un prompt-kind est une
   frontière LLM, PAS une frontière Rust-type → **hors** registre `// FRONTIER:` opt-in
   (`schema_for!` Rust-type only).
6. **0 autorité** — `artifact-draft` anti-PASS + `chat_history_authoritative=false`
   intacts ; `prompt_data` n'émet aucun verdict.

### Spot-checks main-thread (vérification adversariale des faits agents)
- box-shadow non-composite / glow `::after` opacity-only : **confirmé**
  PRIMITIVES.md:750/2105/3356/3450 (« box-shadow n'y est PAS (et ne doit pas l'être) »).
- `morphTo` 'd'/'points' même type + `getPath` premier élément : **confirmé**
  PRIMITIVES.md:43-45.
- reduced-motion « anime ne court-circuite pas tout seul » : **confirmé** PRIMITIVES.md:664
  (+ 998/1014/1063/1109/1255 pattern seek/revert).
- **Nuance écartée** : le sous-claim « box-shadow parse en COMPLEX pas COLOR » n'est pas
  confirmé tel quel → la fiche se borne au terrain confirmé (non-composite + hors set par
  défaut + ::after opacity), sans s'appuyer dessus.

## Gate Phase B (`check-frontier-contracts.sh`) — exigences pour le commit C
- **Volet 1 (anti-promesse STALE-PHASE-K)** : APPLICABLE (`process.rs` sous `crates/`). Le
  commentaire d'arête-provenance ne doit porter aucune promesse future (`Phase X
  will/adds/ships`, `Sprint N will`, `lands in Phase`). Le commentaire prescrit pointe le
  passé immuable → conforme.
- **Volet 2 (registre `// FRONTIER:`)** : N/A (frontière LLM, pas Rust-type). NE PAS
  annoter `app-authoring`.
- **Volet 3 (BLOB_SERVE_CSP)** : N/A (Phase C ne touche pas `blob_serve.rs`).
- **Exécution** : `bash scripts/check-frontier-contracts.sh` doit sortir `clean` avant commit.

## Arête de provenance (passé immuable seulement)
Près de l'entrée PROMPT_KINDS : commentaire pointant `Sprint 79 Phase C, décision D2` +
mécanisme de résolution. Aucune promesse future.

## Watch-items honorés pendant le code
- **Atomicité** : kind + `.md` dans LE MÊME commit (traversée ROUGE→VERT auto-prouvée).
- **Surface minimale** : ne toucher QUE `PROMPT_KINDS`. Pas `prompt_filename`, pas
  `KIND_ALIASES`, pas `PROVIDERS`.
- **`morphTo mono-trace`** : libellé verbatim requis, explicité « un seul tracé résolu,
  même type d⇄d ou points⇄points, getPath premier élément ».
- **reduced-motion** : toujours « l'app DOIT brancher l'état-final » ; garde-fou CSS
  `0.001ms !important` présenté comme complément web standard HORS pack.
- **0 autorité** : aucun `## Verdict: PASS`, aucune dispense CSP/FG, aucune commande
  hors-bande, aucune URL réseau live ; pointeurs path+hash lus localement.
- **Parité provider** : aucun garde-fou CSP sur une ligne strippée par
  `strip_cloud_references` (websearch/context7/mcp) ; test asserte la parité claude/local.
- **Poids tokens** : synthèse distillée + pointeurs path+hash `depth=deep`, PAS recopie du
  corpus (docs.json 781 KB / primitives.json 314 KB).
- **Hashes** : re-vérifiés vs `MANIFEST.json` à l'écriture (docs.json=a8790812191c1c5b,
  primitives.json=8faa36021466192a, synthesis.json=a63150afd6e9a719,
  anime-types.d.ts=31835934518dbe5e).

## Dette latente PRÉ-EXISTANTE (NE PAS élargir en Phase C — carry sprint80)
Deux plafonds `['A'..'G']` codés en dur (`process.rs:154` `detect_current_phase` +
`sprint_history.rs:249` `build_sprint_summary`) → les phases H/I de S79 seront invisibles
au status-detection. **Orthogonal au prompt-kind, non aggravé par Phase C.** À router au
`sprint80_audit_plan.md` (Phase I).

## Delta tests prévu
+1 Rust : `app_authoring_prompt_surfaces_csp_markers` (5 marqueurs T1a × providers
claude/local — rend **T1a runnable ici**). L'invariant `prompt_kinds_resolve_to_existing_files`
couvre automatiquement l'existence du `.md`.
