# Sprint 73 Phase A Preflight

Date: 2026-06-03
HEAD: `845bea6`
Verdict: **EXECUTE**

## Evidence Rules
- Claim policy: chaque claim ci-dessous cite un chemin de fichier, une sortie
  de commande, une URL/date de recherche, ou une hypothese explicite.
- Local sources read:
  - `prompts/agent/preflight.md` (procedure portable, source of truth)
  - `.planning/active/sprint73_plan.md` (§Phase A lignes 84-139, §3 research,
    §5 fail-fast, §9 risks)
  - `.planning/active/sprint73_kickoff.md` (§4 D5 lignes 402-442, §6 carries,
    §7 scope cuts)
  - `crates/nexus-coordinator-rs/src/validator.rs` (1-574, entier)
  - `crates/nexus-shell-daemon/src/validator_loop.rs` (1-217, entier)
  - `crates/nexus-shell-daemon/src/http.rs` (1485-1581 handler, 4102-4244 tests)
  - `crates/nexus-coordinator-rs/src/guardrails.rs` (1-165)
  - `crates/nexus-coordinator-rs/src/db.rs` (375-444 set/get_task_result)
  - `crates/nexus-shell-daemon/src/tasks_api.rs` (150-204 GET /result)
  - `crates/nexus-core-rs/src/task.rs` (30, 61, 350-432 wire ResultPayload)
  - `docs/security/THREAT_MODEL.md` (§14 lignes 768-806)
  - `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` (§2, §3, §3.1, §8)
  - `docs/security/HARDENING_ROADMAP.md` (front-matter, §3)
  - memory `feedback_approach.md` (pick-deepest, no band-aid, G8 procedural)
- Commits read (S2): `16ad15e` (S38 Phase C), `110c003` (S72 Phase D),
  bodies lus via `git show <sha> --no-patch --format=%B`.
- Commands run:
  - `git rev-parse --short HEAD` -> `845bea6`
  - `grep -n` localisation handler/guardrail/set_task_result (sorties citees)
  - `git log --all --oneline -- <validator.rs|validator_loop.rs|guardrails.rs>`
  - `grep -A1 "^name" Cargo.lock` -> axum 0.8.9 / rusqlite 0.36.0 /
    libsqlite3-sys 0.34.0 / tokio 1.52.3

## Scope
- Plan source: `.planning/active/sprint73_plan.md` §Phase A (lignes 84-139).
- Target files:
  - `crates/nexus-coordinator-rs/src/validator.rs` (split `validate_result`)
  - `crates/nexus-shell-daemon/src/http.rs` (~1485-1540, reorder)
  - `crates/nexus-shell-daemon/src/validator_loop.rs` (62-80, injecter gate)
  - `docs/security/THREAT_MODEL.md` (§14, claim ~786-790)
  - `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` (§3 ligne 56 + §2/§8)
  - `docs/security/HARDENING_ROADMAP.md` (front-matter + §3)
- Deps/APIs/specs touchees: **AUCUNE nouvelle dep**. `guardrails` est deja
  `pub mod` (lib.rs:21) ; `default_output_chain()` deja importe par le daemon
  (http.rs:1507). Phase A reordonne du code existant.
- Security/protocol surfaces: T0 loopback `GET /api/v1/tasks/{id}/result`
  (tasks_api.rs:160) ; chemin gossip-sourced validator_loop (Sprint 38) ;
  kudos credit. Aucun wire/canonical/DOMAIN_*/`*_VERSION` touche.
- Tests expected (plan §A.3 + fail-fast rows 6-11):
  `submit_result_rejected_by_guardrail_persists_nothing`,
  `submit_result_accepted_persists_after_guardrail`,
  `validator_loop_rejected_result_not_persisted`,
  `validator_loop_accepted_result_persisted`,
  `quorum_guardrail_runs_on_agreed_text` + grep doc presence.

## S1a OSS Prior Art
- Domain: ordering d'un output guardrail / content-moderation rail
  vis-a-vis l'ecriture persistante (validate-then-persist vs
  guardrail-before-persist) pour des sorties LLM.
- Sources (2026-06-03):
  - NeMo Guardrails (NVIDIA) — Output Rails :
    https://docs.nvidia.com/nemo/guardrails/latest/getting-started/5-output-rails/README.html
    « an output rail can reject the output, **preventing it from being
    returned to the user** » ; « the system prevents that response from
    being returned to the user **before it reaches them** ».
  - openai-agents-python — Guardrails :
    https://openai.github.io/openai-agents-python/guardrails/
    « the SDK immediately raises an `OutputGuardrailTripwireTriggered`
    exception and halts the Agent execution » ; « the exception is raised
    **before the final output is returned** to the user ». (SBFB
    `guardrails.rs` est un port de ce style — cf. commit `16ad15e`.)
  - guardrails-ai — on_fail policies :
    https://github.com/guardrails-ai/guardrails/blob/main/docs/hub/concepts/on_fail_policies.md
    `OnFailAction.EXCEPTION` : « This exception is raised **before any value
    is returned** ».
  - Datadog LLM guardrails best practices :
    https://www.datadoghq.com/blog/llm-guardrails-best-practices/
    « output filtering should happen after model generation **but before
    persisting or presenting to users** ».
- Finding: **APPROACH-ALIGNED**. Les trois frameworks matures bloquent /
  raise AVANT que la sortie ne soit retournee/commitee — exactement le
  split pre/post-guardrail du plan D5 (persist UNIQUEMENT apres passage).
  Le pattern actuel (persist-then-check, http.rs:1500 puis 1507) est
  precisement l'anti-pattern que ces frameworks evitent par construction.
  Coherence interne supplementaire : SBFB applique deja « guardrail avant
  action » a l'**input** (`default_input_chain` AVANT dispatch,
  http.rs:1412-1425) ; D5 etend le meme idiome a l'output.
- Impact: aucune adaptation. Le plan est l'approche SOTA.

## S1b Dependencies, CVEs, Release Notes
- Scanned: axum, rusqlite, libsqlite3-sys, tokio, hex (les seules libs
  traversees par le handler HTTP + le persist DB).
- Commands/sources: `grep -A1 "^name" Cargo.lock` (versions epinglees) :
  - `axum 0.8.9`, `rusqlite 0.36.0`, `libsqlite3-sys 0.34.0`
    (SQLite 3.50.x bundled), `tokio 1.52.3`.
  - `crates/nexus-coordinator-rs/Cargo.toml` + `nexus-shell-daemon/Cargo.toml`
    : toutes en `{ workspace = true }`, aucune ligne ajoutee/bumpee par
    Phase A (verifie : Phase A ne modifie aucun Cargo.toml).
- Finding: **clean**. Phase A n'ajoute ni ne bumpe AUCUNE dependance
  (kickoff §«Versions deps confirmees» : « Aucune nouvelle dep S73 »
  confirme par lecture des Cargo.toml). Le travail est un reordonnancement
  d'appels deja presents (`set_task_result`, `default_output_chain().run`).
  Aucun CVE crypto/wire/network/sandbox/signing applicable a un reorder
  interne. Aucune release breaking pertinente.

## S2 Historical Decisions
- Commands:
  - `git log --all --oneline -- crates/nexus-coordinator-rs/src/validator.rs`
    -> `110c003` (S72 D), `0daff81` (S71 B), `0cb576d` (S55 C),
    `c3cb386` (S36 B), `de054f9` (S35 C).
  - `git log --all --oneline -- crates/nexus-shell-daemon/src/validator_loop.rs`
    -> `16ad15e` (S38 C), `511658f` (S38 A) + S55/S59.
  - `git log --all --oneline -- crates/nexus-coordinator-rs/src/guardrails.rs`
    -> `16ad15e` (S38 C, NEW) + S39-S42.
  - `git show 16ad15e --no-patch --format=%B`, `git show 110c003 ... `.
- Decisions crossed:
  1. **S38 Phase C `16ad15e`** (origine du wire guardrail) : body =
     « Wire dans coordinator_submit_result : validate_result(Accepted) ->
     guardrail chain check -> credit kudos ». A cette epoque,
     `validate_result` **ne persistait aucun texte** (la colonne
     `result_text` n'existait pas ; `set_task_result` n'ecrivait que
     status+worker+hash). Le guardrail-apres-validate etait **benin** :
     aucun texte rejete n'etait stocke ni lisible. Ce n'est PAS une
     decision deliberee « persister avant filtrer ».
  2. **S72 Phase D `110c003`** (introduction du bug) : body =
     « M16 ALTER TABLE tasks ADD COLUMN result_text ; set_task_result
     (+result_text) ... Persiste le texte a l'acceptation ». Le body
     repete la prose S38 (« le result_text du worker est consomme pour le
     guardrail puis droppe ») qui etait vraie a S38 mais devient **fausse**
     une fois le texte persiste DANS `validate_result` avant le guardrail.
     S72 D a aussi ecrit les claims doc fausses (THREAT_MODEL §14 « texte
     deja filtre », LOOPBACK ligne 56). C'est une **regression introduite**
     par S72 D, detectee par l'audit S72 independant
     (P2-RESULT-TEXT-GUARDRAIL-ORDER, headline) — pas une decision figee.
  3. Le **quorum path** (`validator.rs:155`,
     `set_task_result(best_hash, best_hash)`, S72 D) persiste AUSSI sans
     guardrail prealable. D5 le couvre (guardrail sur texte agree `best_hash`
     AVANT persist, cf. plan §A.2).
- Reverse-commit check: aucune decision threat-model ne stipule « persister
  le texte avant filtrer ». La sequence S38->S72 montre une regression, pas
  un retournement d'une decision valide. Le fix D5 **re-aligne** le code sur
  l'intention originale S38 (le guardrail gate le credit kudos ET la
  completion). Aucun `<rejected_sha>` ne contredit le fix.
- Finding: **clean** (confirmed regression, non-blocking). Le fix est une
  reparation de regression S72 D ; il ne rouvre aucune decision figee.

## S3 Local Patterns And Threat Model
- Threats/contracts checked:
  - **T0 loopback** `GET /api/v1/tasks/{id}/result` (tasks_api.rs:160) :
    retourne `result_text` des que `status='completed'`. Or
    `set_task_result` met `status='completed'` ET `result_text` dans le
    **meme UPDATE atomique** (db.rs:402-407). Sur le chemin HTTP actuel
    (http.rs:1500 persist -> 1507 guardrail), un texte rejete par le
    guardrail est deja `completed` + lisible via GET /result avant le
    rejet 400. **Surface fermee par D5** : sur rejet, 0 ligne `completed`,
    GET /result renvoie 404 (status non completed). Aligne sur
    THREAT_MODEL §9.5 « Output filter » (toxicite / CSAM potential).
  - **Chemin gossip-sourced** `validator_loop::process_result`
    (validator_loop.rs:53-96) : appelle `validate_result` (ligne 62) avec
    **ZERO guardrail** — c'est le chemin des resultats reseau non-confiants
    (Sprint 38). D5 y injecte le gate avant persist + skip kudos sur rejet.
    Ferme la fuite la plus sensible (contenu reseau non filtre persiste).
  - **Kudos credit** : sur le chemin HTTP, le credit est deja conditionne
    au guardrail (http.rs:1524 apres le check 1508) ; D5 conserve cet
    invariant et l'ajoute au validator_loop (kudos:65 ne doit pas crediter
    sur rejet).
- HARDENING_ROADMAP status: `last_validated: 2026-05-13` (front-matter) ;
  §3 = « Sprint roadmap Sprint 18-30 » (backlog historique clos). Aucune
  pre-requirement HARDENING pour S73 (les triggers_revalidate listes ne
  matchent pas Phase A). P2-HARDENING-ROADMAP-META-STALE = recadrage doc
  (note « backlog clos » + `last_validated: 2026-06-03`) — non-bloquant,
  c'est l'objet meme du lot doc Phase A.
- Finding: **clean (defense-in-depth ameliore, pas de regression)**. D5
  **ferme** une surface T0 (contenu rejete servable via GET /result) sur
  les 2 chemins ; aucune menace T0-T5 deja couverte n'est regressee. Les
  claims doc fausses (THREAT_MODEL §14, LOOPBACK ligne 56) sont corrigees
  dans la meme phase. Lot doc absorbe (P2-TIER-MODEL : §3.1 Operator existe
  deja depuis S72 P2-H-1 mais §2 vocab + §8 matrice AD1-AD5 ne formalisent
  pas l'Operator -> ajout legitime non-bloquant).

## S4 Protocol And Wire Invariants
- Wire/security files checked:
  - `crates/nexus-core-rs/src/task.rs` : `ResultPayload`/`ResultEntry`
    (350-432) signes via `canonical_bytes(&payload, DOMAIN_RESULT_V1)` ;
    `TASK_FORMAT_VERSION = 1` (ligne 61). `result_text` est deja un champ
    wire (ligne 359, ajoute S72). **Phase A ne touche aucune de ces
    structures** — elle reordonne QUAND `set_task_result` (DB locale) est
    appele vs le guardrail (filtre en memoire).
  - `validator.rs` / `validator_loop.rs` : grep `canonical|DOMAIN_|
    *_VERSION` -> seules occurrences = noms de variables de log
    (`canonical_sha256`, `best_hash`), pas le wire canonical.
- VERSION/domain/canonical status: `TASK_FORMAT_VERSION`,
  `FEED_FORMAT_VERSION`, `*_ANNOUNCEMENT_VERSION`, `DOMAIN_RESULT_V1`
  **tous inchanges**. `set_task_result` UPDATE = DB locale (rusqlite),
  pas un wire. Aucun tolerant decoder, aucun `serde(default)` ajoute.
- Day 0 status: **preserved**. Le split pre/post-guardrail est un
  reordonnancement de timing interne (kickoff D5 « Zero wire »).
  Pre-launch protocol honore (rien pousse origin, aucun bump).
- Finding: **clean**. Phase A est confirmee « no wire » : timing interne
  + doc. Aucun bump requis ni introduit.

## Plan Adaptation
N/A (verdict EXECUTE — aucune adaptation requise).

## Risks And Scope Cuts
- Blocking risks: **none**.
- Non-blocking risks:
  - **R4 (plan §9) — reorder casse des tests lisant un result apres 400** :
    AUDIT FAIT. Les 4 tests HTTP existants `result_submit_*`
    (http.rs:4102-4244) : `accepts_valid` utilise un texte qui PASSE le
    guardrail (reste vert apres reorder) ; `rejects_bad_signature`,
    `rejects_unknown_task`, `rejects_completed_task` rejettent AVANT le
    guardrail dans `validate_result_pre_guardrail` (status/sig/quorum) ->
    inchanges. Aucun test existant ne soumet un texte qui TRIP le guardrail
    puis lit le result. Impact reel = faible ; le split est transparent
    pour les chemins verts. Seul comportement nouveau = le cas
    guardrail-trip (couvert par le nouveau test A.1).
  - Implementation note (non-bloquant) : le split doit faire transiter le
    texte a filtrer entre pre et post — single = `payload.result_text`,
    quorum = `best_hash` agree (validator.rs:155). Faisable sans changer la
    signature wire ; option = `validate_result_pre_guardrail` retourne le
    texte candidat (ou un enum portant le texte) que le post-guardrail
    persiste. C'est de la mecanique interne, pas un finding.
- Scope cuts still honored (kickoff §7) : Phase A ne touche ni recherche
  (C/D/E), ni dette (B), ni SearchManifest (#1), ni rate-limit (#11). Reste
  strictement securite + lot doc menace.

## Action
- **EXECUTE**: proceder avec Phase A comme planifie (split
  `validate_result` pre/post-guardrail ; HTTP pre -> `default_output_chain`
  -> post ; validator_loop gate avant persist + skip kudos sur rejet ;
  quorum sur `best_hash` agree avant persist ; corriger claims THREAT_MODEL
  §14 + LOOPBACK ligne 56 ; P2-TIER-MODEL §2/§8 ; P2-HARDENING-ROADMAP-META
  recadrage + `last_validated: 2026-06-03`).
- Le commit body doit citer ce preflight (G8 traceability), enregistrer le
  finding S1a APPROACH-ALIGNED (NeMo / openai-agents / guardrails-ai) et le
  finding S2 confirmed-regression (S72 D `110c003`).
