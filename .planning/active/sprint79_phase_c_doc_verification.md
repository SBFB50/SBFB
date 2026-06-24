# Sprint 79 Phase C — Vérification de l'artefact-doc (`prompts/agent/app-authoring.md`)

**Cible** : `prompts/agent/app-authoring.md` (161 lignes), livré Phase C commit `95aba5b`,
**INCHANGÉ depuis** (`git log 95aba5b..HEAD -- prompts/agent/app-authoring.md` vide ; HEAD `17ead31`).
**Doctrine** : `.planning/research/doctrine_contrat_pour_llm.md` §2/§3 (couches CODE / ÉTIQUETTE /
COMMIT / GUIDE+llms.txt / arête provenance).
**Méthode** : Workflow ultracode `wcuf12nta` (7 agents Opus 4.8 1M, 413 K tokens, 94 tool-calls) —
6 vérifications **par EXÉCUTION** (sed/grep/cargo/bash, jamais lecture seule) + synthèse adversariale.
**Règle** : ne rien réécrire ; verdict + findings ancrés file:line + routage.

## Verdict: CONCERN

L'artefact est **correct et vert** (aucun ref dans le vide, aucun hash divergent, test-contrat
vert) mais c'est un **GUIDE NON GATÉ** : ses ~21 source_refs et 5 hashes ne sont résolus par
aucun gate déterministe → surface de drift FUTUR réelle, **bornée P2** (pré-launch, mono-auteur,
inchangé, l'autorité reste le code que la fiche reconnaît elle-même). **Routé Phase I (clôture).**
**0 P0 / 0 P1.**

## Preuves par exécution (load-bearing, toutes reproduites)
- **A — Forme** : 161 lignes ; cadre non-autoritaire présent (« consumed and displayed, never
  authoritative » l.8-9 + disclaimer clôture l.160 « it never lifts it ») ; 9 pièges + 5 marqueurs
  canoniques présents ; 0 commande hors-bande / URL live (seul le disclaimer). **PASS.**
- **B — Source_refs + hashes** : les 7 paths du pack existent (`docs/factory/knowledge/animejs/`).
  Spot-checks adversariaux confirmés par `sed` : `README.md:67-68` = 5 pièges canoniques ;
  `PRIMITIVES.md:664` = « anime ne court-circuite pas tout seul une timeline » ; `:1134` = type-match
  morphTo ; `:1171/1178` = `parsedTargets[0]` ; `:3356` = box-shadow non-composite. Les **5 hashes
  inline** (`app-authoring.md:147-154` : 8faa36021466192a, 663c90b1a1f10cb9, a8790812191c1c5b,
  a63150afd6e9a719, 31835934518dbe5e) sont **TOUS un sous-ensemble** des 9 hashes de `MANIFEST.json`
  (0 divergence, 0 orphelin). **PASS** (résolution réelle).
- **C — Drift-gating** : `check-frontier-contracts.sh` scanne `find crates web/src` (:62) + `find
  crates` (:104) → **exclut `prompts/agent/` par construction** (`grep prompts` → exit 1) ;
  `prompt_kinds_resolve_to_existing_files` (process.rs:934-940) ne fait que `.exists()` ; le test
  n'asserte que 5 strings CSP (process.rs:962-969), **jamais** les 5 hashes ni les path-refs.
  → les ~21 refs + 5 hashes de la fiche sont **NON GARDÉS**. **CONCERN (porteur du verdict).**
- **D — Honnêteté** : 0 promesse future (anti STALE-PHASE-K), 0 « shipped/LIVE » faux, seules des
  négations doctrinales d'autorité. **PASS.**
- **E — Test-contrat + recompute** : `cargo nextest -p sbfb-factory -E 'test(app_authoring) +
  test(animejs_manifest)'` = **vert** (le test-contrat asserte 5 marqueurs × parité claude/local ;
  `animejs_manifest_hashes_match_promoted_layers` RECOMPUTE blake3 16-hex == MANIFEST = oracle de
  recompute frais MANIFEST==fichier). `check-frontier-contracts.sh` exit 0 — **mais ne prouve RIEN
  sur l'artefact** (ne lit pas `prompts/`). **PASS** (avec la nuance C).
- **F — Cadence** : étiquette par-phase livrée DANS son commit (`95aba5b` contient `app-authoring.md`
  + l'entrée PROMPT_KINDS) = conforme §3. Phase de clôture (Diataxis `docs/factory/` + `llms.txt` +
  `WIRING_SPEC` + `check-factory-docs.sh`, mirror S77 N) **planifiée Phase I**. **PASS** —
  réserve : le scope source-ref-check de Phase I ne liste PAS le prompt (voir P2-2).

## Verdict de couche (doctrine §2/§3)
**GUIDE NON GATÉ**, pas une ÉTIQUETTE drift-gated. La fiche se positionne comme passerelle
non-autoritaire (l.8-9) = couche 4 GUIDE, pas couche 2 ÉTIQUETTE générée. Un GUIDE hand-distillé
n'est conforme §3 que si ses source_refs sont **source-ref-checkés** par un gate déterministe —
**PROUVÉ ABSENT**. L'**étiquette par-phase** (prompt-kind + drift-gate d'EXISTENCE
`prompt_kinds_resolve_to_existing_files`) est PRÉSENTE et verte dans `95aba5b` ; mais l'**arête de
provenance** des refs/hashes du prompt n'est validée par aucune machine. Conforme à la cadence (la
synthèse GUIDE n'est figeable qu'à la clôture, §3) → **NON-bloquant Phase C**, mais le gating de
l'arête provenance du prompt est un **P2 à acter en Phase I**.

## Findings

### P0 / P1
Aucun.

### P2 (routage : phase-clôture I)
- **P2-1 — GUIDE non gaté** : les ~21 source_refs `path:N` et les 5 hashes blake3 16-hex inline
  (`prompts/agent/app-authoring.md:147-154`) ne sont résolus/comparés par AUCUN gate déterministe.
  `check-frontier-contracts.sh:62/104` (exclut `prompts/`), test CSP-strings-only
  (`process.rs:962-969`), `prompt_kinds_resolve_to_existing_files` = `.exists()` seul
  (`process.rs:934-940`). **Routage : Phase I** — étendre `check-factory-docs.sh` pour résoudre
  chaque `path:Symbol/line` + asserter `hash == MANIFEST.json` sur `app-authoring.md`.
- **P2-2 — Drift-surface concrète + scope I incomplet** : les 5 hashes inline sont une COPIE des
  digests `MANIFEST.json`. `animejs_manifest.rs:25` garde MANIFEST↔couches, **RIEN** ne garde la
  copie du prompt↔MANIFEST → une rotation légitime du pack laisse le prompt mentir, suite verte.
  **PROUVÉ** : le plan Phase I scope le source-ref-check sur `docs/factory/WIRING_SPEC.md`
  (`sprint79_plan.md:424-427`) et **n'inclut PAS** le prompt dans ses fichiers touchés
  (`sprint79_plan.md:444-448`). **Routage : Phase I — scope à ÉTENDRE explicitement** (sinon le gap
  reste non couvert). [Action de suivi : amendement du plan Phase I dans le même chore.]

### P3 (bornage / observations)
- **P3 — Sévérité bornée** : pré-launch, mono-auteur, artefact inchangé depuis `95aba5b`, autorité
  réelle = le code que la fiche reconnaît (`:11` « source of truth stays the code », `:160` « it
  never lifts it »). Risque = drift FUTUR au moment d'une rotation de pack, pas fausseté présente.
- **P3 — Fragilité line-ref** : `PRIMITIVES.md:3168` cité dans 2 pièges (motion-path + onScroll) et
  vit sous le header `events.onScroll` (3123) ; double-citation fidèle mais un réordonnancement
  futur de la section casserait silencieusement les 2 refs (seul le hash fichier est auto-gardé).
  Couvert si le source-ref-check `path:Symbol/line` de Phase I est étendu au prompt.
- **P3 (in-phase, aucune action)** — Parité claude/local du test = no-op structurel aujourd'hui
  (0 token strippable dans la fiche, `strip_cloud_references` = no-op) ; valeur = garde forward,
  reconnu dans le commentaire (`process.rs:950-951`). Revendication assumée et exacte.
- **P3 (in-phase, aucune action)** — Redondance pédagogique cohérente : marqueurs canoniques 2×
  (`UMD…` l.27+100, `motion-path cx=0` l.53+131, `morphTo mono-trace` l.69+131) — intentionnel,
  0 divergence de contenu.

## Réfutations adversariales
**Aucun finding réfuté** — tous étayés par commande/sortie réelle. Re-vérifications confirmant
(et non réfutant) : 5 hashes == sous-ensemble MANIFEST (contre hash-rot présent) ; test-contrat
3/3 (contre rouge) ; gate exclut bien `prompts/` (`grep` exit 1, contre couverture cachée) ; le
test n'asserte QUE des strings CSP (`process.rs:962-969`, confirme la drift-surface) ; le scope
Phase I ne liste PAS le prompt (`sprint79_plan.md:444-448`, confirme le gap).

## Conclusion
Artefact-doc Phase C = **CONCERN non-bloquant**. Correct, honnête, résolu, testé, conforme à la
cadence par-phase. Le seul gap réel = **l'arête de provenance (refs+hashes) n'est pas
source-ref-checkée** → 2 P2 routés **Phase I (clôture)**, dont le **scope du `check-factory-docs.sh`
de Phase I doit être étendu explicitement** pour inclure `prompts/agent/app-authoring.md`
(amendé dans le même chore).
