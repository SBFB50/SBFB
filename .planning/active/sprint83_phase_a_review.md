# Sprint 83 — Phase A — Review

## Verdict: PASS

Review interne OK ; Codex GPT-5.6 Sol round 2 = CLEAN ; exception AM-3 ratifiée
PO (Option A, 2026-07-18). Verdict global = plus sévère des 6 dimensions
corrigées = PASS ; 0 P0/P1/P2 CONFIRMED. Détail en « Codex reconciliation » (fin).

## Périmètre revu (diff stagé — 11 livrables + review.md = 12 ; + codex_review.md au commit = 13)

`git diff --cached --stat` (HEAD 803a451) — **12 fichiers, +1274 l. indexées**
(hors `sprint83_phase_a_codex_review.md`, ajouté à la reconciliation Codex) :
- `crates/sbfb-workflow-core/Cargo.toml` (+16), `src/lib.rs` (+48),
  `src/contracts.rs` (+208), `src/contracts.schema.json` (+51 golden)
- `Cargo.toml` (+1 membre workspace), `Cargo.lock` (+9)
- `scripts/agent/test_agentctl.py` (+98 : 3 fn `test_*` + wiring `main()`)
- `.planning/active/sprint83_gate_matrix.md` (+149 : 47 gates + WA-T + HUB-T)
- `.planning/active/sprint83_t2_spec.md` (+186 : schéma t2 durci + vocab fermé)
- `docs/factory/WORKFLOW_AGENTS_OVERVIEW.md` (+93 : GUIDE amorce)
- `.planning/active/sprint83_phase_a_preflight.md` (+236 : verdict EXECUTE)
- `.planning/active/sprint83_phase_a_review.md` (ce fichier, ≈+180)

Vérifications re-exécutées in-vivo au HEAD : `cargo test -p sbfb-workflow-core
--locked` = 2/2 pass + 0 doctest ; `python scripts/agent/test_agentctl.py` =
EXIT 0 ; `bash scripts/check-frontier-contracts.sh` = clean, EXIT 0
(« DOMAIN census [25 frozen] »).

## R1 Compile/correctness — PASS

7 structs stub `{}` (`contracts.rs:31,40,49,58,67,75,84`) + `deny_unknown_fields`
+ derive `JsonSchema` + title `.v1` ; nommage `Sbfb*` évite le lint clippy
`upper_case_acronyms`. `all_contract_schemas()` insère exactement 7 entrées,
clés = noms `.v1` canoniques. Golden `contracts.schema.json` (51 l., 7 titres,
`$schema` draft 2020-12, `additionalProperties:false`) = snapshot COMMITTÉ
non-vide → `assert_eq!(live, snapshot)` s'exécute réellement en arbre committé
(le chemin write-then-return n'est pris que si absent/vide). 0 dep tierce neuve :
`Cargo.toml:12-14` = serde/serde_json/schemars TOUS `workspace=true` ;
`Cargo.lock` n'ajoute QUE le nœud du crate lui-même (schemars 1.2.1 déjà dans
l'arbre). EXEC : clippy `-D warnings` = 0 warning ; test = 2/2. Aucun P0/P1/P2.

## R2 Tests sémantique — PASS

Suites EXÉCUTÉES (pas seulement lues). Non-vacuité vérifiée fichier:ligne contre
le vrai regex `PHASE_TITLE_RE [A-Z]+[0-9]?` : (1) labels bi-lettres A1/W1/AA/AI2/
AJ1 parsent aux tokens assertés ; (2) nombres nus 3/0/8 → `(None,None)` +
`requires_codex=False` (`test_numeric_phase_label_refused` load-bearing — casserait
si la grammaire s'élargissait) ; (3) `test_phase_grammar_shared_across_hooks` :
le token `Phase[[:space:]]+[A-Z]+[0-9]?` est réellement présent à
`phase-precommit-lightcheck.sh:88` ET `phase-auditor-gate.sh:54` — et ABSENT à
`lightcheck.sh:79` (variante sans `[0-9]?`), donc l'assertion exige bien le
suffixe `[0-9]?` (non-vacuous) ; un hook manquant lèverait `FileNotFoundError`
(échec bruyant). Golden `schema_snapshot_stub` = comparaison stricte en CI
(snapshot committé). Aucun test vacuous, aucune valeur attendue fausse.

## R3 Scope cuts + pre-launch — PASS

0 BUMP WIRE : aucun `const *_VERSION`/`DOMAIN_*` déclaré dans
`crates/sbfb-workflow-core/src/` (seul hit = doc-comment `contracts.rs:15` qui
DÉNIE toute déclaration) ; frontier gate clean, census DOMAIN = 25 GELÉ intact,
BLOB_SERVE_CSP non-régression. CRATE PUR : deps = serde/serde_json/schemars
uniquement, 0 tokio/reqwest/rusqlite/redb/iroh/hyper/axum, 0 route, 0 verbe
bridge, 0 runtime. 7 STUBS VIDES `{}` — aucun champ n'encode un design de phase
future. 14 SCOPE CUTS (kickoff §7) respectés : `WORKFLOW_AGENTS_OVERVIEW.md:87-93`
liste hors-MVP (scheduler DAG distribué, réplication live, co-édition, marketplace,
auto-update, A2A…) — RIEN d'implémenté. DÉCISIONS GELÉES D1..D5 intactes (4 couches,
pack JCS+Ed25519+DOMAIN+subject[] blake3, mémoïsation-de-steps+journal SQLite PAS
replay-total/WASM, Plan-Then-Execute, grammaire `[A-Z]+[0-9]?`). `verification_
blueprint.md` SUPERSEDED correctement non-activé.

## R4 Research-grounding (matrice/t2_spec/GUIDE) — PASS

Les 47 gates de la matrice §1 == exactement les 47 de research §23 (noms + ordre,
0 fabriqué / 0 manquant, 9 G-A11Y-* présents). Chaque phase-propriétaire cohérent
avec l'outline kickoff §5. WA-T1..T6 == §22.5, HUB-T1..T10 == §18 (owners == table
§19). t2_spec : vocab fermé PASS/BLOCK/RIG-ABSENT + ACTED/MIXED/NOT-RUN, règle
bi-temporelle T6=MIXED(AI2)→final(AJ1) présente §2.1, 4 specs T1 nommées. GUIDE :
4 couches D1 + 2 crates D3 + 7 contrats fidèles, 0 jeton STALE-PHASE-K
(grep vide), 0 promesse « Phase X will ». 3 blemishes cosmétiques P3 + 1 INFO
(voir consolidés) — aucun n'est un gate fabriqué/manquant, phase-owner faux, ni
vocab incohérent.

## R5 Sécurité / frontier gate — PASS

Aucun stub ne porte le tag machine `// FRONTIER: <name> domain= version=` (le
seul « FRONTIER » du crate = prose `contracts.rs:4`), donc check(2) du gate
n'exige aucun const domain/version — CONFORME au header opt-in de
`check-frontier-contracts.sh`. Anti STALE-PHASE-K : doc in-code au présent/passif
générique, `PROMISE_RE` ne fire pas. Gate EXÉCUTÉ = EXIT 0, census 25 gelé,
1 tagged, BLOB_SERVE_CSP non-régression. 0 surface d'attaque neuve (crate pur, 7
types vides non activés, 0 désérialisation réseau ; `deny_unknown_fields` →
`additionalProperties:false`). AM-3 (étiquetage docs-contrat) correctement DIFFÉRÉ
au remplissage. Aucun bump THREAT_MODEL requis. clippy `-D warnings` = EXIT 0.

## R6 Complétude livrables + commit — PASS

Les 5 livrables Phase A présents, complets, non-tronqués : matrice (47 gates +
WA-T + HUB-T, 0 orphelin), t2_spec (vocab fermé + schéma versionné + 4 specs T1),
preuve grammaire (3 fn `test_*` + assert 2 hooks, suite verte), 7 stubs schemars +
golden (2/2 PASS, fmt/clippy clean), GUIDE amorcé. Titre commit conforme à la
grammaire ; décision 1-commit `feat(workflow-core)` justifiée (le hook applique
Codex/9-sections aussi à `chore`, `lightcheck.sh:295` — le split n'apporte rien).
Delta tests honnête (+3 py + 2 Rust). 2 blemishes P3 documentaires (voir
consolidés). Staging réel = 11 fichiers, complet et correct.

## Findings consolidés après refutation (P0..P3) — TOUS RÉSOLUS

Aucun P0/P1/P2 CONFIRMED au round review interne. 4 findings P3 (cosmétiques,
tous CONFIRMED) — **les 4 sont CORRIGÉS** avant Codex (voir aussi « Codex round 1 »
plus bas) :
- miscount matrice → « 40 NEUF + 7 ÉTENDU » (`gate_matrix.md`) ✅
- G-FS-PROCESS-EFFECT → Q ajouté en fermeture multi-phases ✅
- G-A11Y-PREFS/AUTHORING → « AI1 (consolidation) » ajouté ✅
- périmètre preflight → `Cargo.lock` + artefacts process ajoutés ✅

Détail d'origine (conservé pour trace) :

- **P3 — miscount synthèse matrice.** `sprint83_gate_matrix.md:83` dit « ~30 NEUF
  + ~7 ÉTENDU » ; comptage réel = **40 NEUF** + 7 ÉTENDU + 0 EXISTANT = 47 (grep
  confirmé : 40 `| NEUF |`). L'invariant 0-orphelin et le total 47 restent
  corrects ; seule la ligne de synthèse est fausse. Fix : « 40 NEUF + 7 ÉTENDU ».
- **P3 — G-FS-PROCESS-EFFECT omet le site Q.** `gate_matrix.md:41` liste
  fermetures « K (ouvre) → AA (ferme) » ; research §23 l.2434 dit « vérifiés en
  K/Q/AA ». Défendable (la matrice suit l'outline opérationnel où row Q ne porte
  que G-HOST-ISOLATION/G-WORKFLOW-CSP) mais écart §23→matrice non signalé. Fix :
  ajouter Q comme contributrice ou noter la subsomption par G-HOST-ISOLATION.
- **P3 — G-A11Y-PREFS (et G-A11Y-AUTHORING) sans phase de fermeture.**
  `gate_matrix.md:77` (PREFS, owner B) et `:73` (AUTHORING, owner T) ont colonne
  Fermetures = « — » alors que l'outline AI1 (kickoff l.663) déclare le blanket
  « G-A11Y-* (ferment) ». Gates non-orphelins (ils ont un owner) ; divergence
  sémantique de fermeture pour 2 gates a11y. Fix : lister AI1 en fermeture, ou
  annoter « enforcé en continu par test, pas de phase dédiée ».
- **P3 — liste « Périmètre du commit unique » du preflight incomplète.**
  `sprint83_phase_a_preflight.md:188-195` énumère 6 puces couvrant 9/11 fichiers
  stagés (la puce `crates/…` en absorbe 4) mais omet `Cargo.lock` (+9 l.,
  nécessaire à `--locked`) et `sprint83_phase_a_preflight.md` (+222 l.). Le
  staging RÉEL est complet et correct (`git diff --cached --stat` = 11 fichiers) ;
  seule la liste documentaire est incomplète. Fix : ajouter les 2 chemins.

INFO (transparence, 0 action) : golden `schema_snapshot_stub` auto-régénère si
snapshot absent/vide (masqué localement mais l'arbre committé garde la garde
anti-drift, §P75) ; nom `every_contract_emits_nonempty_draft_2020_12_object`
légèrement trompeur (stubs vides, « nonempty » = l'objet-schéma) ; docstring
`test_phase_grammar_shared_across_hooks` dit « extract with » là où le token est
un suffixe du regex complet (assertion néanmoins load-bearing) ; table
`[dev-dependencies]` vide dans le Cargo.toml du crate ; asymétrie RIG-ABSENT du
t2_spec (absent au niveau palier, encodé NOT-RUN + `rig_absent_reason`,
intentionnel §2.1) ; nom de job CI prospectif `e2e-workflow` (t2_spec:128) non
figé comme fait ancré AM-1.

## Codex round 1 (GAP) — corrections appliquées + reconciliation

Codex GPT-5.6 Sol (round 1) a rendu **GAP** — verdict correct, il a attrapé des
défauts réels que le round review interne avait sous-évalués. Corrections
appliquées (main-thread) et VÉRIFIÉES :

1. **Matrice — 26 commandes `cargo nextest` invalides** (`-p` sans `run`) →
   corrigées en `cargo nextest run …` (24 `-p` + `hitl` + `verticals`). ✅
2. **Test grammaire trop faible + divergence `\b`** (agentctl rejette `AI22`,
   hooks glouton l'extraient en `AI2`) → `test_phase_grammar_shared_across_hooks`
   RENFORCÉ : parité comportementale rejouée sur les 5 labels réels + divergence
   `\b` connue asservie explicitement (python exit 0). ✅
3. **Schéma t2 vacuous** → durci (if/then Draft 2020-12) : PASS⟹6 paliers ACTED,
   ACTED⟹`evidence` non vide, MIXED⟹`substeps`, BLOCK⟹slug (pas de prose),
   `written_at_phase` enum. **Non-vacuité PROUVÉE** (`jsonschema` 4.26.0 : 4
   contre-tests Codex + phase invalide rejetés, exemple valide accepté). ✅
4. **golden fail-open** → rationale §P75 documentée in-code (bootstrap self-write,
   golden committé ⇒ CI compare). ✅ (pattern canonique, conservé)
5. **review.md stale** → ce fichier régénéré (périmètre + P3 RÉSOLUS). ✅

**AM-3 — RATIFIÉE PO (Option A, 2026-07-18)** : Codex signalait que l'absence de
`// FRONTIER:` sur les stubs contredit la lettre d'AM-3 (qui nomme PackV1/GraphV1/
ProfileV1/ProjectionV1). Résolution technique FORCÉE (annoter exigerait des consts
`DOMAIN_*`/`*_VERSION` inexistants en Phase A → casse le gate + le census 25 +
D2). Le PO ratifie l'exception « identité vide réservée ≠ primitive de frontière
remplie » : `// FRONTIER:` est posé au commit de REMPLISSAGE de chaque contrat
(F/G/H/D/X) ; kickoff AM-3 inchangé (exception documentée preflight §S3).

## Codex reconciliation
Codex GPT-5.6 Sol — round 1 = GAP (5 défauts réels), round 2 = **CLEAN**. Round 2
CONFIRME les 5 corrections (matrice `run`, parité grammaire comportementale,
non-vacuité t2 prouvée par validateur Draft 2020-12, golden §P75, review
régénéré) ; seul résiduel = P3 documentaire (stats périmètre — corrigées ici).
AM-3 non compté comme GAP code (exception documentée + ratifiée PO Option A).
GAPs P0/P1 : 0. Artefact brut : `sprint83_phase_a_codex_review.md` (non réécrit).

## Statut de sortie
Suites vertes : Rust nextest workspace 2110 (2108 S82 +2 crate), python agentctl
exit 0, frontier gate clean (census 25 gelé), t2 non-vacuité prouvée (jsonschema
4.26.0), fmt/clippy/doctest/release verts, web 412/412 (flake de charge écarté).
Prêt au commit `feat(workflow-core): Sprint 83 Phase A`.
