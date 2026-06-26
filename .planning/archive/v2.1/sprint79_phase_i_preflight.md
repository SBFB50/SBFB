# Sprint 79 — Phase I preflight (G8) — Docs-contract closure Factory

Meta : 2026-06-25 · Cas B (pre-code) · HEAD `545c78e` (arbre git propre, Phase I non demarree) · methode Workflow fan-out 5 scans (S1a/S1b/S2/S3/S4) + verification adversariale · synthese verifiee par re-grep direct.

Phase I = derniere phase S79, couche GUIDE de la doctrine contrat-pour-LLM (noeud 4 §2/§3). Livre : `docs/factory/{README,EXPLANATION,HOW_TO_WIRE,REFERENCE}.md` (Diataxis FR, REFERENCE corps EN) + `docs/factory/llms.txt` + `docs/factory/WIRING_SPEC.md` + 1 exemple runnable `include!` + `scripts/check-factory-docs.sh` cable CI 3 surfaces + section factory au `llms.txt` racine + wrap-up final (dual-platform AVANT push). 0 nouvelle primitive, 0 bump wire, 0 dep.

---

## S1a OSS prior-art

Modele a cloner = `scripts/check-sharding-docs.sh` (241 lignes, `set -euo pipefail`, BusyBox-safe : pas de `grep -P`, `--include`, `\b`, `mapfile`). Re-scoper `docs/sharding/*` -> `docs/factory/*`.

Anatomie des volets (lignes -> action -> re-scope factory) :

| Volet | Lignes | Action | Re-scope factory |
|---|---|---|---|
| Existence | 35-45 | les 4 docs `ALL_DOCS` doivent exister | `docs/factory/{README,EXPLANATION,HOW_TO_WIRE,REFERENCE}.md` |
| (1) link-check | 47-66 | tout lien md repo-relatif resout depuis le dir du doc ; ancre `#…` strippee `${target%%#*}` | clone tel quel |
| anchor_present | 68-85 | `grep -qF` marqueur litteral dans fichier cible (allowlist HARD-CODEE sharding THREAT §16, §P64-69…) | RE-SCOPER : ancres factory reelles (`FACTORY_GATES.md`, `PATTERNS §P71`, `csp.rs:BLOB_SERVE_CSP`, `gates.rs:run_gate_csp_authoring`) |
| (2) honesty-gate | 87-99 | `require_marker` PROVISIONAL + `admission != confidentialite` | RE-SCOPER : la Factory n'est PAS PROVISIONAL au sens sharding ; voir S3 marqueurs |
| (3) french-body | 101-113 | liste EN-UI BusyBox-safe sur 3 FR_DOCS, REFERENCE EN exempte | clone tel quel (meme split FR/EN) |
| (4) couche agent | 115-232 | bloc le plus dense (source-ref-check + required-anchor + Truth-Stack) | clone + re-scope |

Sous-volets du bloc (4) a cloner :
- Existence Phase-N (129-134) : `WIRING_SPEC.md`, `<DOCS_DIR>/llms.txt`, racine `llms.txt`, `examples/*`, exemple `.rs`, le test lifteur.
- source-ref-check (156-194) : LE coeur. Token backtique `` `path` ``/`` `path:Symbol` `` dont le path est rank-1 (`crates|docs|web|scripts`) doit resoudre : fichier existe (172) ; Symbol non-numerique -> `grep -qF` (178-184) ; numerique -> ligne dans `[1, wc -l]` (185-191). `.planning/` n'est PAS rank-1 (jamais resolu).
- required-anchor allowlist (196-212) : piliers que WIRING_SPEC DOIT ancrer. RE-SCOPER factory : `BLOB_SERVE_CSP`, `run_gate_csp_authoring`, `PROMPT_KINDS`, `app-authoring`, `authoring_knowledge`, `TemplateConfig`, `handle_context_pack`, `MANIFEST.json`.
- Truth-Stack header (214-221) : `anchor_present` de `repo files > .planning/active/ > commits > prompts > chat` + `Not evidenced`. Clone tel quel.
- honesty-gate extension (223-232) : `PROVISIONAL/S78` + racine `llms.txt` `sharding subsystem only`. RE-SCOPER (cf. cross-gate watch-item).

`llms.txt` (format llmstxt.org confirme in-repo) : H1 obligatoire -> blockquote `>` resume -> sections H2 = listes `- [nom](url): description` -> `## Optional`. Structure attendue `docs/factory/llms.txt` : H1 `# SBFB factory — agent index` ; blockquote 1-liner + Truth-Stack + `Not evidenced` (grep-forces) ; sections `## Start here` (WIRING_SPEC + FACTORY_GATES), `## Gate primitives (rank-1 source)` (gates.rs + csp.rs), `## Runnable examples`, `## Human docs (Diataxis, French)`, `## Optional`.

Diataxis (precedent sharding cite https://diataxis.fr/) : EXPLANATION->Explanation, HOW_TO_WIRE->How-to, REFERENCE->Reference (corps EN), README->orientation/hub (PAS un Tutorial). INCOHERENCE a acter : si le plan mappe "Tutorial->README", trancher entre (a) README=orientation fidele au precedent (pas de quadrant Tutorial), ou (b) README porte un vrai tutoriel pas-a-pas (5e usage fusionne, a documenter comme tel).

Modele `include!` runnable (modele reel `docs/sharding/examples/sign_verify.rs` + lifteur `crates/nexus-core-rs/tests/shard_sign_verify.rs:16-19`) : commentaires `//` PAS `//!` (un inner-doc apres include! casse le compile) ; `#[test]` propres ; nextest n'execute pas les `[[example]]` mais compile+execute le test d'integration qui `include!` -> drift d'API = build rouge. ECART MAJEUR : ce modele lifte une API LIB (`use nexus_core_rs::…`) — voir S4 R1 (`sbfb-factory` bin-only).

Soeur : `scripts/check-frontier-contracts.sh` (memes 3 sites de cablage) garde deja `BLOB_SERVE_CSP` non-regression + prompt-kind provenance -> eviter le double-gate du meme symbole dans l'allowlist factory.

Fichiers cles : `scripts/check-sharding-docs.sh`, `scripts/check-frontier-contracts.sh`, `llms.txt`, `docs/sharding/{llms.txt,README.md,WIRING_SPEC.md}`, `docs/sharding/examples/sign_verify.rs`, `crates/nexus-core-rs/tests/shard_sign_verify.rs`.

---

## S1b Deps & portabilite

VERDICT 0-DEP CONFIRME : 0 dependance Cargo/npm nouvelle, 0 bump wire.

`crates/sbfb-factory/Cargo.toml` : `[dev-dependencies]` (40-44) = `tempfile` + `serial_test` UNIQUEMENT (workspace). AUCUNE section `[[test]]` -> tests d'integration auto-decouverts depuis `crates/sbfb-factory/tests/*.rs`. Aucune entree `[[test]]` requise pour ajouter un test. (Nuance forte : voir S4 — la cible LIB n'existe pas ; l'exemple runnable se loge plutot dans `crates/nexus-core-rs/tests/*` qui a la lib `nexus_core_rs`.)

Contraintes portabilite (cible = bash:5 + grep BusyBox, PAS POSIX sh) — interdits documentes en en-tete des 2 scripts : `grep -P`, `--include`, `\b`, `\s` dans regex grep, `mapfile`/`readarray`. Autorises : `[[ ]]`, `case`, process substitution `< <(...)`.

Constructs portables a reproduire :
- en-tete : `#!/usr/bin/env bash` + `set -euo pipefail` + `SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"` + `REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"` + `cd "$REPO_ROOT"` + `fail=0`.
- greps `-oE`/`-nE`/`-qF`/`-qxF`/`-hoE`/`-qvF` toujours `-E` (ERE) jamais `-P` ; `grep -h` LOAD-BEARING sur multi-fichiers (sinon prefixe `filename:` casse `grep -qxF`) ; `|| true` apres chaque grep susceptible non-zero.
- boucle `while IFS= read -r x; do … done < <(cmd)` ; string-ops pures `${var#pre}`/`${var%suf}`/`${var%%#*}`/`case` ; `wc -l < "$f" | tr -d ' '` ; `printf '%s\n'`.
- sortie : accumulateur `fail=1` -> `if [ "$fail" -ne 0 ]; then … exit 1; fi` puis `echo "...: clean (...)"; exit 0`.

Cablage CI BLOQUANT — 3 surfaces (lignes exactes confirmees par re-grep) :

Surface A — `.github/workflows/ci.yml` (apres 122) :
```yaml
      # ── sharding docs lint ───────────
      - name: "[14] sharding docs check"
        run: bash scripts/check-sharding-docs.sh
      - name: "[15] frontier contracts check"
        run: bash scripts/check-frontier-contracts.sh
```
-> ajouter `[16] factory docs check` -> `run: bash scripts/check-factory-docs.sh`.

Surface B — `.woodpecker/ci-linux.yml` (apres 82), digest image pinne IDENTIQUE obligatoire :
```yaml
  - name: factory-docs-check
    image: bash:5@sha256:2003051c5eb5154cbd44fd4b1a2b8f1be886517b383813c998c72cb15840357f
    commands:
      - bash scripts/check-factory-docs.sh
```
(Cette image bash:5 EST la raison de la contrainte BusyBox.)

Surface C — `scripts/verify.sh` (apres 112) : `step 21 "bash scripts/check-factory-docs.sh"` puis `bash scripts/check-factory-docs.sh`, avant `==> verify.sh passed all steps`.

---

## S2 Decisions historiques

Alignement doctrine contrat-pour-LLM CONFIRME :
- §2 (graphe, noeud 4) : `GUIDE + llms.txt` = index navigable, propriete cardinale anti-hallucination, cadence cloture (1 phase). Plan Phase I (`sprint79_plan.md:410-416`) cite explicitement le noeud GUIDE §2/§3. Conforme.
- §3 (cadence tranchee) : GUIDE = UNE seule phase de cloture, image figeable seulement a la fin. Phase I = derniere phase, livre QUE la synthese + doc-lint factory-scope + wrap-up. La regle + le gate GENERIQUE (`check-frontier-contracts.sh`) ont ete canonises AVANT en Phase B (`b27079c`). Conforme.
- §6 (ou ca vit) nomme S79 : "1re instance concrete". Generalisation portable dans `docs/agent/AGENT_SYSTEM.md:262-266` (5 couches + compagnons, posee Phase B). Aucun ecart.
- §7/§10 (trous reels) : `source-ref-check GENERIQUE absent` + `llms.txt SBFB absent` + `docs/factory/knowledge/** absent` = exactement ce que Phase I ferme cote Factory. Conforme.

Modele S77 Phase N exact — commit `a795700` (Phase N) precede de `91be0e4` (Phase M qui a cree la couche humaine + le gate). Livrables a cloner cote factory :

| Fichier S77 | Equivalent factory Phase I |
|---|---|
| `docs/sharding/WIRING_SPEC.md` (5 sections ordre fixe, 49 source_refs rank-1, Truth-Stack header + Not evidenced) | `docs/factory/WIRING_SPEC.md` (required-anchor `PROMPT_KINDS`, `app-authoring`, `run_gate_csp_authoring`, `BLOB_SERVE_CSP`, `authoring_knowledge`, `TemplateConfig`, `handle_context_pack`, `MANIFEST.json`) |
| `docs/sharding/llms.txt` + `llms.txt` racine | `docs/factory/llms.txt` + section factory racine |
| `docs/sharding/examples/sign_verify.rs` (lifte verbatim, fixtures `#[cfg(test)]` inlinees, `//` pas `//!`) | `docs/factory/examples/<x>.rs` (voir S4 R1 : cible API lib `nexus_core_rs::csp`) |
| `crates/nexus-core-rs/tests/shard_sign_verify.rs` (`include!`) | exemple `include!`-e cote `nexus-core-rs/tests/*` |
| `scripts/check-sharding-docs.sh` (cable 3 surfaces) | `scripts/check-factory-docs.sh` (cable 3 surfaces) |

Reste P2 semantique — confirme SEUL restant cote Phase I :
- Le volet DETERMINISTE (hash+path) est DEJA FERME generiquement par `check-frontier-contracts.sh` volet 4 PROMPT-PROVENANCE (Phase B `b27079c`, cable CI 3 surfaces) : 4a digest blake3 16-hex inline == digest `MANIFEST.json` ; 4b chemin `docs/factory/knowledge/...` cite doit exister. Prouve adversarialement + Codex 4/5.
- RESTE a Phase I = le volet LINE-SEMANTIC : pour chaque `prompts/agent/*.md` knowledge-backed, verifier que chaque source_ref pointe une ligne qui existe (borne numerique). NUANCE CRITIQUE (differente de S77) : les refs de `prompts/agent/app-authoring.md` sont nom-de-fichier-nu + lignes/plages relatifs au pack (`PRIMITIVES.md:112/627/1013/1454`, `README.md:81`, `PRIMITIVES.md:1107-1179`), PAS `path:Symbol` rank-1. Le gate ne peut verifier que l'EXISTENCE-DE-LIGNE (miroir branche numerique `check-sharding-docs.sh:185-191`) ; le "supporte ENCORE la claim" reste la revue LLM adversariale (cf. `check-frontier-contracts.sh:51-52`). Ne pas sur-revendiquer. Scope prompt deja inclus au plan (`sprint79_plan.md:457` liste `prompts/agent/app-authoring.md`).

Day-0 figees (kickoff 177-234) NON violees : "aucune nouvelle primitive de code" (plan I:416, seul artefact code = 1 exemple lifte verbatim) ; 0 bump wire / 0 dep ; scellage 100% Factory non-delegable (Day-0 #8, Phase I documente, n'introduit aucune dispense) ; source CSP unique `BLOB_SERVE_CSP` (Day-0 #4, Phase I la reference) ; connaissance CONSOMMEE jamais autoritaire (renforce par honesty-gate). Aucun conflit historique avec une couche docs GUIDE. Signaux d'alignement : STALE-PHASE-K-COMMENTS (anti-promesse, garder 0 promesse future) + incident out-of-band S77 Phase N (banner flippe en "shipped" -> refuse, verrouille par honesty-gate -> reproduire cote factory).

---

## S3 Threat & honnetete

Etat REEL des livrables S79 (`070c7a9..545c78e`) — LIVE-teste vs PROVISIONAL :

| Livrable | Statut | Caveat que la GUIDE ne doit JAMAIS franchir |
|---|---|---|
| Gate CSP `run_gate_csp_authoring` (Phase E) | LIVE-teste (`pipeline.rs:52` BLOQUANT hors `if !skip_gates` ; 13/13 CSP, anti-drift, JSON-mirror ; source unique `csp.rs:33`) | lint STATIQUE deterministe : faux-negatifs assumes (`fetch(atob)`, URL runtime, `//host`) ; PAS une garantie runtime ni preuve d'absence d'exfiltration |
| Self-check viewer runtime (Phase H) | LIVE-teste (filet) (T1 3/3, T2 `status=PASS` 3/3 0-skip, test Rust byte-exact http.rs 200+404 ; Codex 6/6) | "Filet, pas preuve totale" : `blockedURI` opaque, echec COEP-pur ne fire pas, auteur peut supprimer son rapport ; `status=PASS` = verdict de TEST, jamais autorite de publish |
| Knowledge packs anime.js + daisyUI (A/F) | LIVE-teste (artefact drift-gated, hash-recompute blake3) | CONSOMME/AFFICHE jamais autoritaire : `chat_history_authoritative=false`, 0 verdict PASS, artifact-draft anti-PASS |
| Template daisyui (G) | LIVE-teste (`test_csp_gate_daisyui_template_passes` vrai gate) | build-time `@tailwindcss/oxide` binaire natif non-vendore = hors-couverture gate CSP runtime (P3 supply-chain) |
| Prompt-kind `app-authoring` (C/D/F) | cablage LIVE-teste ; efficacite generative NON-CABLEE-IN-VIVO | la fiche se resout + surface marqueurs = prouve ; qu'elle PRODUISE de bonnes apps via LLM = non mesure -> `Not evidenced` |
| Copilote Ollama keyless (G) | plomberie LIVE-testee ; boucle generative NON-CABLEE-IN-VIVO | ne PAS pretendre "le copilote genere des apps daisyui" : aucune session LLM reelle Ollama->app evidenced |

HONNETETE CARDINALE NIVEAU SPRINT : aucun element S79 n'a ete exerce par un parcours utilisateur in-vivo end-to-end (auteur reel -> gate -> self-check -> publish -> rendu chez un pair). Construit + teste hermetiquement, jamais "deploye-et-eprouve". Parcours author->publish->render-cross-peer = PROVISIONAL / Not evidenced -> router `sprint80_audit_plan.md`. La GUIDE ne doit JAMAIS ecrire "shipped"/"LIVE"/"en production" du statique/local.

Caveat cardinal Factory — formulation EXACTE (verbatim plan `sprint79_plan.md:433-434`, 2 clauses, confirmees fideles au code) :
> "lint statique CSP != garantie runtime complete [self-check viewer requis] ; connaissance CONSOMMEE jamais autoritaire [0 verdict PASS]"

Clause 1 <-> `FACTORY_GATES.md:132-139` + `:160-183` ("Filet, pas preuve totale") + `THREAT_MODEL.md:765/:774`. Clause 2 <-> `kickoff:227-228` + `FACTORY_GATES.md:183` + Phase D review (`chat_history_authoritative=false` aux 2 handlers). Adoptable telle quelle, aucune re-formulation requise.

Marqueurs REQUIS dans le honesty-gate de `check-factory-docs.sh` (grep BLOQUANT sur les 6 docs) :
1. Caveat cardinal 2-clauses present.
2. `PROVISIONAL` pour tout item non-in-vivo (parcours end-to-end ; efficacite generative copilote/prompt-kind).
3. `Not evidenced` marqueur de claim non-prouvee.
4. BAN faux-positif `shipped`/`LIVE`/`production`/`deploye`/`in-vivo` applique a un livrable statique/local non adosse a PROVISIONAL/preuve.
5. Truth-Stack ordering verbatim `repo files > .planning/active/ > commits > prompts > chat`.
6. `0 verdict PASS` / `chat_history_authoritative=false` non contredits.
7. french-body (REFERENCE.md corps EN exempte).

Surface securite Phase I = AUCUNE introduite : docs Markdown inertes + `check-factory-docs.sh` (clone grep/compare) + 1 test `include!` + cablage CI YAML. 0 execution de contenu untrusted, 0 nouvelle route daemon, 0 autorite de verdict deplacee, 0 bump wire, 0 dep. CAUTION FORWARD (review d'implementation) : `check-factory-docs.sh` reste STRICTEMENT grep/compare, jamais `eval`/source d'un `.md` (injection-commande depuis un doc malveillant).

---

## S4 Wire & symboles

TABLEAU symbole -> fichier:ligne (re-grep direct, 545c78e) :

| Symbole (WIRING_SPEC `path:Symbol`) | Definition reelle | Statut |
|---|---|---|
| `PROMPT_KINDS` | `crates/sbfb-factory/src/process.rs:7` | EXISTE |
| kind `app-authoring` | `crates/sbfb-factory/src/process.rs:22` (9e entree) | EXISTE |
| fiche `prompts/agent/app-authoring.md` | present (cite MANIFEST animejs/daisyui) | EXISTE |
| test couplage kind<->fichier | `crates/sbfb-factory/src/process.rs:933` `prompt_kinds_resolve_to_existing_files` | EXISTE |
| `run_gate_csp_authoring` | `crates/sbfb-factory/src/gates.rs:386` (`pub fn`) ; wiring `pipeline.rs:52` | EXISTE |
| `BLOB_SERVE_CSP` | `crates/nexus-core-rs/src/csp.rs:33` (`pub const`) ; re-export `lib.rs:104` | EXISTE |
| `authoring_knowledge` (champ context-pack) | helper `operator_server.rs:368` ; emis `:430` + `:687` ; const `AUTHORING_KNOWLEDGE_MANIFESTS:363` | EXISTE |
| `TemplateConfig` | `template_engine.rs:218` ; tableau `TEMPLATES:227` ; 5e entree daisyui `:260` | EXISTE |
| `handle_context_pack` | `operator_server.rs:375` ; route `:128` | EXISTE |
| `MANIFEST.json` | `docs/factory/knowledge/{animejs,daisyui}/MANIFEST.json` | EXISTE (les 2) |
| test MANIFEST blake3 | `tests/animejs_manifest.rs:5` + `tests/daisyui_manifest.rs:5` | EXISTE |

0 ABSENCE. Tous les symboles que le WIRING_SPEC referencera existent.

Finding doc-exactitude (NON bloquant, a corriger DANS le WIRING_SPEC) : `BLOB_SERVE_CSP` a ete DEPLACE en Phase E (`blob_serve.rs:286` -> `crates/nexus-core-rs/src/csp.rs:33`). Le plan/design citent encore l'ancre perimee (`sprint79_plan.md:282,312`). Le WIRING_SPEC DOIT citer `csp.rs:33` (sinon source-ref-check echoue sur ancre morte). Orthographe exacte `run_gate_csp_authoring` (PAS `run_gate_authoring_csp`).

Preuve 0-bump wire : arbre git PROPRE (`git status --porcelain` vide) ; tous les `*_FORMAT_VERSION` de `nexus-core-rs/src` = 1 (TASK/CURATOR/SEED/POW/KEY_ROTATION/NODE_DIRECTORY/COMPUTE_GROUP/ACTIVATION_COMMIT/SHARD_PLAN/RUN_PROOF/PIN_FILE) ; `FEED_FORMAT_VERSION=1` (`public_feed.rs:20`) ; `SBFB.json schema_version=2` (`sbfb-manifest/src/lib.rs:150`). Perimetre plan = docs/factory/*, llms.txt, scripts, CI yml, tests + `+1 Rust example include!` + `1 script doc-lint`. 0 fichier wire, 0 dep — conforme Day-0.

Crate cible `include!` — FINDING BLOQUANT (R1) : `sbfb-factory` est BINAIRE-PURE : `Cargo.toml` = AUCUN `[lib]`, `src/lib.rs` ABSENT (verifie `ls` -> No such file), `src/main.rs` + `mod gates;` ; `grep "use sbfb_factory" crates/sbfb-factory/tests/` = 0 hit ; les 2 tests d'integration (`operator_server.rs:13`, `process_cli.rs:6`) pilotent le BINAIRE via `Command::new(env!("CARGO_BIN_EXE_sbfb-factory"))`. Consequence : un `include!` dans `crates/sbfb-factory/tests/*` qui ferait `use sbfb_factory::gates::run_gate_csp_authoring` NE COMPILE PAS (cible lib inexistante) ; et aucune sous-commande CLI ne lance le gate CSP isolement (il vit dans `Command::Publish` -> `run_publish_pipeline` via `pipeline.rs:52`, exigeant repo_url + gate provenance). Le modele S77 (`sign_verify.rs` lifte `use nexus_core_rs::…` API lib) ne se transpose PAS tel quel a la formulation plan `sprint79_plan.md:457` (`crates/sbfb-factory/tests/*`). Severite P1 bloquant pour le livrable "+1 Rust example include!", resolu sans cout par Option B (cf. Adaptations).

---

## Verification adversariale

Aucune refutation DURE de l'hypothese de fond (couche GUIDE Factory miroir S77 Phase N atteignable). Tous les faits porteurs des scans corrobores par re-grep direct ce jour :
- Symboles WIRING_SPEC : TOUS EXISTENT (`run_gate_csp_authoring`=gates.rs:386, `BLOB_SERVE_CSP`=csp.rs:33, `PROMPT_KINDS`=process.rs:7, `app-authoring`=:22, test=process.rs:933). S4 exact, 0 absence.
- Cablage CI 3 surfaces EXACT : ci.yml `[14]`/`[15]` (117-122), woodpecker (74-82, digest `bash:5@sha256:2003...0357f`), verify.sh step 19/20 (108-112). Nouvelle entree = `[16]`/`factory-docs-check`/`step 21`.
- Cross-gate root llms : `check-sharding-docs.sh:232` = `require_marker "$ROOT_LLMS" "sharding subsystem only"` (substring `grep -qF`) ; `llms.txt:8` porte cette ligne. Ajouter une section factory = sur TANT QUE la ligne banner n'est pas reecrite.
- Honnetete : aucun "LIVE" abusif. "Gate CSP LIVE-teste" = mecanisme cable BLOQUANT + tests verts, jamais "prouve l'absence d'exfiltration". Copilote/prompt-kind efficacite generative correctement `Not evidenced`/PROVISIONAL. 0 refutation honnetete.
- Day-0 / wire : arbre propre, 0 fichier wire touche, plan declare 0 bump / 0 dep / 0 nouvelle primitive. `nexus-core-rs` EST path-dep de `sbfb-factory` (fait porteur pour R1-fix Option B). 0 violation.

Deux assertions du PLAN (pas des scans) mecaniquement infaisables telles qu'ecrites, confirmees par re-grep :
- R1 (CONFIRME) : `include!` lifte dans `crates/sbfb-factory/tests/*` appelant le gate Factory = infaisable (crate binaire-pure, aucune sous-commande CSP-gate isolee). PLAN-ADAPT obligatoire.
- R2 (CONFIRME) : format source_refs du prompt-kind = nom-nu + lignes/plages, != `path:Symbol` rank-1 S77. PLAN-ADAPT mineur.
- R3 (CONFIRME) : ancre `blob_serve.rs:286` perimee dans plan/design ; reelle = `csp.rs:33`. PLAN-ADAPT.

Aucun DESIGN-CONFLICT : aucune nouvelle primitive ni refactor bin->lib n'est necessaire si Option B (`use nexus_core_rs::csp`) est retenue. Aucune Day-0 figee touchee.

---

## Adaptations PLAN-ADAPT

1. R1 (BLOQUANT, livrable +1 Rust example include!) — `sbfb-factory` binaire-pure (Cargo.toml sans `[lib]`, `src/lib.rs` absent, 0 `use sbfb_factory` dans tests/, pilotage subprocess CARGO_BIN_EXE). ADAPTER : l'exemple `docs/factory/examples/<x>.rs` fait `use nexus_core_rs::csp::{BLOB_SERVE_CSP, none_directives, CSS_URL_ALLOW};` (tous pub, re-export `lib.rs:104`) et est `include!`-e dans `crates/nexus-core-rs/tests/*.rs` (math `concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/factory/examples/<x>.rs")`). La preuve runnable porte sur la constante CSP source-de-verite `nexus_core_rs::csp`, PAS sur `run_gate_csp_authoring` (bin-prive gates.rs:386, non liftable en `use`). Ne PAS pretendre que l'exemple execute le gate Factory. Le WIRING_SPEC peut quand meme CITER `gates.rs:386` (grep-resoluble par source-ref-check). Corrige `sprint79_plan.md:457`.
2. R2 — `check-factory-docs.sh` volet semantique resout `PRIMITIVES.md:N`/`README.md:N-M` contre `docs/factory/knowledge/{animejs,daisyui}/` (existence-de-ligne, miroir `check-sharding-docs.sh:185-191`), PAS le rank-1 `path:Symbol`. Ne pas sur-revendiquer une semantique "supporte encore la claim" deterministe (= revue LLM adversariale, cf. `check-frontier-contracts.sh:51-52`).
3. R3 — WIRING_SPEC cite `crates/nexus-core-rs/src/csp.rs:33` pour `BLOB_SERVE_CSP` (pas `blob_serve.rs:286`) et `gates.rs:386` pour `run_gate_csp_authoring` (orthographe exacte).
4. Watch-item cross-gate (non-bloquant) — ajouter la section factory au root `llms.txt` DOIT preserver la sous-chaine `sharding subsystem only` (`llms.txt:8`, asserte `check-sharding-docs.sh:232`), OU mettre a jour cette ligne du gate sharding. Soit `## Factory (app-authoring)` ajoutee sans toucher le banner, soit banner reformule + regex `require_marker` ajustee dans les DEUX gates.
5. Watch-item honnetete (non-bloquant) — honesty-gate factory porte le caveat cardinal 2-clauses verbatim (`sprint79_plan.md:433-434`) ; PROVISIONAL/Not evidenced pour parcours author->publish->render + efficacite generative ; BAN `shipped`/`LIVE`/`production`. Eviter de re-gater dans l'allowlist factory les symboles deja gardes par `check-frontier-contracts.sh` (`BLOB_SERVE_CSP` + prompt-kind). Script strictement grep/compare, jamais `eval`/source d'un `.md`. Cabler aux 3 surfaces (`[16]` GHA, `factory-docs-check` woodpecker meme digest bash:5, `step 21` verify.sh).

---

## Verdict: PLAN-ADAPT

L'objectif Phase I tient (tous symboles existent, cablage CI reproductible a l'identique, 0 bump wire, 0 dep, 0 Day-0 violee, 0 fausse claim d'honnetete) MAIS le plan tel que litteralement ecrit contient une hypothese mecanique infaisable (exemple `include!` lifte dans `crates/sbfb-factory/tests/*` ciblant un gate bin-prive d'une crate binaire-pure) qui exige une decision explicite avant la 1re ligne de code : Option B `use nexus_core_rs::csp` (recommandee, fidele S77, 0 changement de structure). Ajouts R2 (format source-ref nom-nu) et R3 (ancre `csp.rs:33`) evidence-backed. Aucun DESIGN-CONFLICT : aucune nouvelle primitive ni refactor bin->lib requis. Aucune Day-0 figee touchee.
