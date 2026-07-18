# Sprint 82 Phase F — Preflight (G8)

Date : 2026-07-14. Phase F élargit `PROMISE_RE`
(`scripts/check-frontier-contracts.sh:66`) à la classe « until/when
Sprint N activates/lands », réécrit les commentaires-promesse de
`crates/nexus-core-rs/src/schemas/task_response.rs` vers le passé
immuable, et ajoute un self-test de non-vacuité du motif. Ferme les
carries **S79-P2-1** (P2, 2 reports) + **S80-G-2** (P3) — toucher
fermant §6.2.1, aucun report sec de plus. Preflight ultracode =
Workflow 11 agents (5 scans S1a/S1b/S2/S3/S4+sweep + 5 vérifications
adversariales par scan ; le scan S2-history a échoué au formatage de
sortie [StructuredOutput cap], sa matière historique est couverte par
S3 vérifié + la ré-extraction main-thread des verbatims carries).
Toutes les ancres ci-dessous re-vérifiées au disque le 2026-07-14
(arbre propre, tip `f727f8c`).

## Verdict: PLAN-ADAPT

Le plan est exécutable, aucune décision Day-0/PO n'est touchée (aucun
DESIGN-CONFLICT). Mais **cinq faits du plan sont incomplets ou inexacts**
et imposent une exécution corrigée :

1. **« les 4 commentaires task_response.rs (:14, :84-85, :95) »** —
   ÉNUMÉRATION INCOMPLÈTE. Le hit-set réel du motif fidèle à la classe =
   **5 lignes {14, 84, 93, 95, 100}** (sweep exécuté sur les 366 fichiers
   du scope, vérifié adversarialement). La ligne **:93** (« until Sprint 22
   activates ») était DÉJÀ nommée par S80-G-2 (`sprint80_audit_findings.md:226-233`,
   « ancres NOMMÉES : :14/:84-85/:93/:95 ») — le plan S82 l'a perdue en
   recopiant. La ligne **:100** (« Match against the S22+ allow-list ») est
   neuve (aucun verbe futur, attrapée par la branche `S[0-9]+\+? allow-list`).
   Réécriture par-bloc : `:14` (//! module), `:82-85` (/// tool_calls),
   `:90-95` (/// ToolCall), `:99-100` (/// name).
2. **Périmètre de réécriture = 3 fichiers / 7 sites, pas 1** — le plan ne
   nomme que `task_response.rs`, mais tout motif fidèle à la classe tire
   aussi **`crates/nexus-core-rs/src/verification.rs:31`** (« until S77 » —
   promesse GENUINE stale : S77 est passé et le texte :23-30 confirme au
   présent que la transition n'a pas eu lieu, digest = nom du modèle, pas
   les poids) et **`crates/nexus-worker-core/src/llm/mod.rs:29`** (« for
   S22 sandbox » ; les frères S23/:30 et S26/:31 du même énoncé ne sont
   pas gate-forcés → réécrire la phrase-rationale entière :28-33 pour ne
   pas laisser une demi-liste stale). Sans ces réécritures, le gate casse
   sur les 3 surfaces CI (critère machine « exit 0 » inatteignable).
   L'alternative « rétrécir le motif pour coller au scope 1-fichier du
   plan » (until mot-plein « Sprint » + `+` obligatoire) aveuglerait
   DÉLIBÉRÉMENT le gate à 2 promesses stale genuine = band-aid contraire
   à la doctrine root-cause — écartée.
3. **Snapshot wire à régénérer (absent du plan)** — schemars 1.2.1 copie
   les doc-comments `///` dans les `description` du schéma généré ; les
   promesses S20/S22 vivent VERBATIM dans
   `task_response.schema.json:5/:11/:43`. Réécrire `:82-85`/`:90-95`/`:100`
   force `UPDATE_SNAPSHOTS=1 cargo test -p nexus-core-rs
   schema_snapshot_matches_struct` + commit du snapshot dans le MÊME commit
   atomique (sinon test rouge). Diff attendu **descriptions-only** (squelette
   required/$defs/types piloté par la struct intacte). La ligne :14 (//! module)
   et les 2 autres fichiers (//! module) n'alimentent AUCUN snapshot.
   **0 bump wire** : `TASK_RESPONSE_VERSION=1` (:48) et
   `TASK_RESPONSE_DOMAIN_TAG` (:54) intouchés, aucun champ/attribut serde
   modifié ; le schéma est la grammaire des backends (Ollama JsonStructure,
   llama.cpp llguidance) mais les descriptions ne contraignent pas le
   sampling — et en PROD `params.schema` reste `None`
   (`engine/runtime.rs:1435-1447` n'appelle jamais `.with_schema`) : la
   grammaire est une capacité dormante exercée par les seuls tests.
4. **La promesse :84-85 est SPLIT sur 2 lignes physiques** (« until the
   Sprint 22 » / « tool-calling sandbox activates the gate ») — grep -E
   est line-oriented : AUCUN motif par-ligne ne peut matcher token+verbe à
   cheval. C'est la RÉÉCRITURE qui la supprime, pas le gate ; le gate
   empêche les régressions mono-ligne. Le self-test de non-vacuité doit
   être MONO-LIGNE et ne pas prétendre prouver la détection multi-ligne.
   Attendu vert→rouge→vert : le hit-set pré-réécriture dans
   `task_response.rs` = **{14, 93, 95}** + :84 via la branche
   `until (the )?Sprint` et :100 via `allow-list` (set complet {14,84,93,95,100}).
5. **Sémantique de réécriture contrainte par 2 claims sécurité vivants**
   (S3 vérifié CONFIRMED) : le tool-calling sandbox est **DÉFÉRÉ post-S25,
   jamais livré** (`HARDENING_ROADMAP.md:307-308`, `CAPABILITY_TOGGLES.md:42`
   gate `tool_calling` OFF « Déferré post-S25 per S22 scope cuts ») et
   **wasmtime n'est pas une dépendance** (banni préemptif `<43.0.1`,
   `deny.toml:156/:170` ; décision gelée CLAUDE.md « OS sandbox pour
   Factory, pas wasmtime » ; aucun doc n'engage le tool-calling sur
   wasmtime — `COMPUTE_THREATS.md:532-534` le décrit sans wasmtime). La
   réécriture au passé ne doit réaffirmer NI « S22 » NI « S25 » comme
   activation future, NI substituer un mécanisme (« OS sandbox ») jamais
   décidé pour cette surface. Fait immuable à énoncer : `tool_calls` est
   déclaratif et ignoré du coordinator depuis S20 ; le champ est porté sur
   le wire pour stabilité de forme ; source vivante = gate `tool_calling`
   de `docs/security/CAPABILITY_TOGGLES.md` (pointeur autorisé, pas de
   numéro de sprint en dur).

## Scans

### S1a — Prior-art self-test regex — `EXECUTE`, low
Le script n'a AUCUN self-test de PROMISE_RE aujourd'hui (:65-79) ; le seul
précédent interne = garde anti-silent-removal `// FRONTIER: ShardPlan`
(:122-128) — modèle à répliquer. Design retenu : self-test inline en tête
du check (1), fixtures positive + négative, `printf | grep -qE` (BusyBox-safe,
flags -q/-E déjà exercés par le script). Fixtures validées au grep réel :
POS `promise: tool_calls inert until Sprint 22 activates the gate` matche ;
NEG `the values the consumer will read once a future sprint adds a field`
silencieuse (+ négatifs additionnels silencieux : « node A adds a blob »,
« valid until the deadline », « the wasmtime sandbox is loaded »).

### S1b — Deps + câblage — `EXECUTE`, low
0 dépendance ajoutée (schemars/serde_json déjà workspace deps). Gate câblé
sur EXACTEMENT 3 surfaces : `.github/workflows/ci.yml:135-136` (step [15]),
`.woodpecker/ci-linux.yml:89-92` (step dédié, image `bash:5@sha256:2003…`,
0 depends_on = bloquant), `scripts/verify.sh:98-99` (step 15). AUCUN hook
local (backstop commit inexistant — conforme README:2552-2556 « OPT-IN »).
`PROMISE_RE` défini uniquement à :66, 0 duplication exécutable. **Sync doc
requise** : `docs/rust/PATTERNS.md §P70 :3904-3907` est la seule énumération
verbatim des branches — déjà stale de 2 branches (« S[0-9]+ will »,
« When Sprint [0-9]+ » absentes) → folder les 2 manquantes + la classe neuve
dans le même toucher. Les autres docs (README §6.12/§7.1, AGENT_SYSTEM §7,
shell PATTERNS) sont génériques, 0 sync ; claim « 3 surfaces » reste exact.
Process substitution `< <(find …)` = bash requis : fourni sur les 3 surfaces.

### S2 — Décisions historiques — `EXECUTE`, med (agent formatage KO, matière ré-extraite)
Verbatims carries (main-thread, fichiers vivants) : **S79 P2-1**
(`sprint79_audit_findings.md:47-48`, P2) = « Commentaires-promesses
non-scrubbés dans un fichier wire-format + gate anti-promise aveugle »,
cite :14/:93/:95, reco « scrubber les 3 commentaires OU élargir PROMISE_RE
à la classe until/when Sprint N » ; routage S80 constaté inexistant.
**S80-G-2** (`sprint80_audit_findings.md:226-233`, P3) = « S79 P2-1
bucket-route sans ancre nommée », ancres NOMMÉES :14/:84-85/:93/:95,
« fermeture = scrub des 4 commentaires + élargissement PROMISE_RE
(candidat sprint dette) ». Phase F EST cette fermeture (fusion actée
`sprint82_phase_e_ledger_reconciliation.md:37/:78/:240`). Histoire du
tool-calling S22 : jamais activé — déféré post-S25 (CAPABILITY_TOGGLES:42),
wasmtime écarté (12 CVE avril 2026, décision gelée CLAUDE.md). Provenance
fichiers : `task_response.rs` dernier touch `3c9ea1b` (S72 Phase C) ;
`check-frontier-contracts.sh` dernier touch `8d7ee81` (S79 Phase F).
Bug doc pré-existant SUR le fichier édité : `task_response.rs:37` référence
`test_schema_snapshot_matches_struct` — le fn réel (:278) s'appelle
`schema_snapshot_matches_struct` (sans préfixe) → toucher d'hygiène
in-passing (1 mot, même fichier, même classe dette-doc).

### S3 — Threat model — `EXECUTE`, low (GO, correction de vérité)
0 régression sécurité ; la phase CORRIGE deux claims faux vs docs sécurité
vivants (détail au point 5 du verdict). `THREAT_MODEL.md` est muet sur
tool_calls/TaskResponse/wasmtime (grep = 0 hit) — cohérence à établir avec
CAPABILITY_TOGGLES/HARDENING_ROADMAP/COMPUTE_THREATS, pas avec THREAT_MODEL.
**Risque dominant identifié (design du self-test)** : la boucle scan (1)
fait `grep -nE "$PROMISE_RE" "$f" || true` (:70) — le `|| true` avale
l'exit 2 d'un regex MAL-FORMÉ → un PROMISE_RE cassé rend le check
silencieusement vacant-vert. Le self-test doit donc asserter un match
POSITIF sans `|| true` (un motif malformé y produit exit 2 → fail=1 avec
diagnostic, fail-closed). Impact génératif de la régénération snapshot :
nul par construction (descriptions = annotations non-structurelles ;
en prod le schéma n'est même pas attaché — dormant, tests-only).

### S4 — Wire invariants — `EXECUTE`, very low
0 bump : les 4 sites `task_response.rs` sont TOUS des commentaires
(:14 //!, :82-85 ///, :90-95 ///, :100 ///) ; consts :48/:54 et attributs
serde :67/:80/:86/:97 hors périmètre. Aucun golden/hash ne fige les octets
du schéma hors du snapshot nommé ; `schema_is_task_response`
(`schema_bridge.rs:59-60`) compare live-vs-live → insensible aux
descriptions. Le `.schema.json` est HORS scope du find du gate (extensions
*.rs/*.toml/*.sample/*.ts/*.tsx) — le snapshot suit par régénération, jamais
par scan. FEED/Task/ProjectAnnouncement intouchés. **Regression-map des
voisins à NE PAS attraper** (8 lignes in-scope, motif lâche interdit) :
`key_rotation.rs:13` (deferred to S26), `verification.rs:27` (réf backend
« (llm_llama_cpp, Sprint 77) » — non flaguée, correct), `http.rs:1762/:1814`
(carried/deferred to the S76 audit), `iroh_runtime.rs:519` (routed to the
S76), `build_executor.rs:7` (deferred to S57+), `runtime.rs:910` (until
W9.1 wires), `llama_cpp.rs:596` (carried to S21) — toutes restent VERTES
sous le motif recommandé (vérifié adversarialement).

### S5 — Sweep d'impact du motif (cœur mécanique) — `PLAN-ADAPT`, med
Scope reproduit : 366 fichiers. PROMISE_RE actuel byte-clean sur tout le
scope (gate vert baseline, exit 0 vérifié) ET aveugle aux cibles (0 hit
task_response.rs — postulat du plan vrai). **Motif recommandé R1** (validé
0 faux positif corpus complet, double-vérifié par l'adversarial) = motif
actuel + 4 branches :
`until (the )?(Sprint |S)[0-9]` · `[Ww]hen (Sprint |S)[0-9]+ (lands|activates|ships)` ·
`(Sprint |S)[0-9]+\+? (sandbox|allow-list)` · `(Sprint |S)[0-9]+\+? activates`.
Hit-set exhaustif = 3 fichiers / 7 sites (détail au verdict). **Piège
prouvé** : une branche verbe autonome `(Sprint |S)[0-9]+ (lands|ships)`
crée 3 faux positifs sur récit AU PASSÉ (« Sprint 3 ships the » gpu/mod.rs:6,
« Sprint 20 ships only » llm/mod.rs:327 + schema_bridge.rs:45) — lands/ships
restent ancrés par `[Ww]hen` ; `activates` autonome = 0 collatéral courant.
`\+?` littéral ERE BusyBox-safe (testé). **Risque résiduel documenté**
(adversarial) : les branches sandbox/allow-list/activates ne sont pas
tense-anchored — un récit passé futur « the Sprint 22 sandbox was added
last year » matcherait ; 0 collatéral dans l'arbre actuel, comportement de
classe assumé (toute adjacence « Sprint N sandbox » est suspecte à la
review) — à consigner dans l'en-tête du script. **Caveat de complétude
hors-classe** (consigné, PAS un livrable F) : l'idiome « post-SN / for SN
(frozen) / reserved for SN » (~15 occurrences .rs/.ts + 3 *.schema.json
générés hors scope du find) décrit des scope-cuts ENCORE OUVERTS
(présent-vrai), défendablement hors classe STALE-PHASE-K — aucun fix ;
noté pour l'audit Track K.

## Vérification adversariale

5 vérifications par-scan (1 par scan abouti) : 0 REFUTED sur les faits
load-bearing ; 6 NUANCED corrigés et intégrés supra (dont : :14 = //! module
donc hors-snapshot ; hit-set {14,93,95} pas {14,95} ; « est consommé comme
grammaire » → capacité dormante prod schema=None ; §P70 déjà stale de 2
branches). 2 findings MISSING/CRITICAL de l'adversarial S1a intégrés au
verdict (collisions verification.rs:31 + rate_limit.rs:37). NOTE : la
collision `rate_limit.rs:37` (« les défère S22+ … Phase A », français)
signalée par UN adversarial n'est PAS dans le hit-set du motif R1 final
(vérifié par le sweep + son adversarial : « S22+ puisqu'il » n'a ni
sandbox/allow-list/activates adjacent ni until/when — silencieuse) ; elle
ne rentre au périmètre QUE si une branche plus large était retenue — elle
ne l'est pas. Le scan S1a lui-même a renvoyé un stub (« probe ») — son
adversarial a re-fait le travail sur les faits du contexte : intégré.

## Approche d'exécution (adaptée)

1 commit atomique, 6 fichiers :
1. `scripts/check-frontier-contracts.sh` — PROMISE_RE :66 + 4 branches ;
   self-test non-vacuité inline (POS+NEG, sans `|| true`, diagnostic
   explicite) juste après la définition ; en-tête (2) enrichi (nouvelle
   classe + note tense-anchoring résiduel).
2. `crates/nexus-core-rs/src/schemas/task_response.rs` — blocs :14, :82-85,
   :90-95, :99-100 au passé immuable (sémantique point 5) + fix :37
   (`test_schema_snapshot_matches_struct` → `schema_snapshot_matches_struct`).
3. `crates/nexus-core-rs/src/schemas/task_response.schema.json` — régénéré
   `UPDATE_SNAPSHOTS=1` (diff descriptions-only à vérifier).
4. `crates/nexus-core-rs/src/verification.rs` — :31 (« until S77 ») au
   passé/présent factuel (name-digest, pas d'attestation poids, gate backend
   file-exposing — sans horizon-sprint).
5. `crates/nexus-worker-core/src/llm/mod.rs` — phrase-rationale :28-33
   réécrite (argument architectural au présent, drop des tags S22/S23/S26).
6. `docs/rust/PATTERNS.md` — §P70 :3904-3907 énumération synchronisée
   (+ 2 branches déjà manquantes).

Critère machine final : `bash scripts/check-frontier-contracts.sh` exit 0
(scan + self-test) ; contrôle rouge-avant-vert : motif R1 sur l'arbre
PRÉ-réécriture flague exactement {task_response.rs:14,84,93,95,100 ;
verification.rs:31 ; llm/mod.rs:29} ; `cargo nextest run -p nexus-core-rs`
vert (snapshot régénéré) ; suites §7.4 complètes avant commit.

## Invariants à préserver

- 0 bump wire (`TASK_RESPONSE_VERSION=1`, domain tag, struct byte-stable) ;
  0 dep ; 0 route ; T1 = N-A (aucun `web/src/api` touché) ; T2 = N-A.
- BusyBox-safe strict (grep -E pur, `\+`/`\.` seuls échappements, pas de
  \b/\s/-P) — surface d'enforcement effective = Woodpecker bash:5.
- Diff snapshot descriptions-only ; réécritures au passé SANS réintroduire
  verbe-présent adjacent à un token sprint (« Sprint 22 lands the gate »
  resterait rouge — préférer passé/état : « the gate was never activated »,
  « tool_calls stays declarative »).
- Carries S79-P2-1 + S80-G-2 : fermeture consignée au commit body (toucher
  fermant, compteur 2 reports soldé).
