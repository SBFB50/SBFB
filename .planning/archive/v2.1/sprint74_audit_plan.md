# Sprint 73 — Audit Plan (consomme par la session fraiche S74)

**Ecrit** : 2026-06-04 (Phase F Sprint 73).
**Sprint audite** : **Sprint 73** (recherche reseau cablee — FTS5 fraicheur +
SearchResult enrichi triplet provenance + barre shell + guardrail securite +
fermeture dette worker-pump 3/3).
**Executeur** : session fraiche S74, Phase 0 (Cas A audit gate).
**Produit attendu** : `.planning/active/sprint73_audit_findings.md`
(verdict PASS / CONDITIONAL PASS / FAIL).
**Tip audite** : commit Phase F `docs(sprint73)` (HEAD au demarrage S74 ;
tip code phases = `9472085`).

---

## §0 Mode d'emploi pour la session fraiche S74

**Ordre de lecture impose** (forme une opinion AVANT de lire les self-reports) :

1. Ce fichier (`sprint74_audit_plan.md`) — la feuille de route.
2. Le **diff complet** S73 : `git diff 845bea6..9472085` (kickoff → tip code).
   Les 5 phases : A `845bea6..6f5ff30`, B `6f5ff30..a4e1542` (+ chore
   `5361fd8` normalisation review header, sans code), C `a4e1542..47c9ff7`,
   D `47c9ff7..0f86e5a`, E `0f86e5a..9472085`.
3. `sprint73_kickoff.md` §4 (D1..D6 gelees) + §7 (14 scope cuts).
4. Le code livre, dans l'ordre des tracks ci-dessous.

**A NE PAS lire avant d'avoir forme une opinion** :
`sprint73_verification.md` (self-report — l'agent livreur a ecrit le code ET
la verification ; valeur de confirmation nulle pour un audit independant) et
les `sprint73_phase_*_review.md` (reviews du livreur). Les lire **apres** pour
comparer, pas pour se faire une opinion.

**Format du livrable** : `sprint73_audit_findings.md` (§7 ci-dessous).

**Contexte non-standard a connaitre** :
- **Phase B est une phase dette pure** (non-convertible en feature), reservee
  bien que S73 soit impair, fermant **7 P2 herites** dont **P2-A-1 worker-pump
  iroh-docs Windows = 3/3 MANDATORY** (escalade G7, plus jamais carry). La
  fermeture repose sur le passage `multi_thread` (zero `#[cfg(windows)]`) +
  un fix de race recv-vs-shutdown synchronise sur l'ecriture observable.
- **Phase A corrige un invariant de securite** (P2-RESULT-TEXT-GUARDRAIL-ORDER,
  headline audit S72) : guardrail de sortie AVANT persist `result_text` sur les
  **2 chemins** (HTTP + gossip `validator_loop`), via un split
  `validate_result_pre_guardrail` (no persist) / `validate_result_post_guardrail`.
- **D3 a DEFERE SearchManifest** (arbitrage user Checkpoint §11) : feed-local +
  design note de la forme correcte. L'audit verifie que c'est **tranche**
  (decide, documente, zero code wire), pas oublie — R6 du kickoff anticipe la
  contestation « livrable roadmap manquant ».
- **Phases C & D ont DEFERE leur verification Docker Linux canonique au
  wrap-up Phase F** (changement pur Rust platform-agnostic). Phase F a execute
  le run Docker Linux (sbfb-ci, libgtk-3-dev, rustc 1.95.0) couvrant M17 +
  hot-reindex : **1570/1570, 0 skip**. L'audit confirme que ce run a bien tourne
  (pas juste un re-cite du 1556/1560 de Phase B anterieur a M17).

---

## §1 Critere verdict audit S73

| Verdict | Condition |
|---------|-----------|
| **PASS** | 0 P0, 0 P1, >= 1 P2+ documente |
| **CONDITIONAL PASS** | 0 P0, 0 P1 mais conditions a surveiller S74 |
| **FAIL** | >= 1 P0 ou >= 1 P1 non resoluble dans l'audit |

Rigor signal G4 : PASS exige >= 1 P2+ documente. 0 P0/P1 **et** 0 P2+
= CONCERN (audit trop superficiel). S73 expose au minimum ~10 P2+ candidats
(ci-dessous, surfaces par une scrutiny adversariale multi-agent du diff) — un
audit qui n'en confirme aucun est suspect.

---

## §2 Tracks audit S73 (ce que Phase 0 S74 doit verifier)

> **Note de provenance** : les points de scrutiny ci-dessous proviennent d'une
> analyse adversariale par phase (6 agents independants, ~539k tokens, sur le
> diff brut S73, Phase F). Ils sont des **questions a verifier**, pas des
> findings confirmes. Chaque point porte une severite « si faux » — l'audit
> tranche.

### Track A — Suites verification

Relancer la fail-fast `sprint73_verification.md §Fail-fast checklist`. Attendu :

- **Canonique CI Linux** (Docker `sbfb-ci`, rustc 1.95.0, `apt libgtk-3-dev`
  pour `atk-sys`) : **1570 run / 1570 passed / 0 skip**.
- **Windows natif** : **1566 passed / 0 skip**. Ecart **+4 vs Linux** =
  tests `#[cfg(unix)]`-gates (UDS peer-cred `auth.rs`, e2e unix-gates) absents
  sous Windows — **structurel, pas un skip masque** (apparait des Phase B :
  memory 1556 Win / 1560 Linux). Ne PAS auditer le compte sur Windows seul
  (`feedback_wsl_before_push`).
- Vitest `web/` **289** (24 fichiers), `factory-operator` Vitest **7** (infra
  NEW Phase B), size-limit **6/6**, clippy 0 warning, fmt exit 0, doctests 0 fail.
- **Sous-point A.1 (worker-pump, gate `cargo test` shared-process)** : le hang
  P2-A-1 original se manifestait sous **`cargo test` partage** (teardown
  multi-binaire), PAS sous `nextest` (process-par-test). Le nextest 1570/1566
  vert ne re-prouve donc PAS la fermeture sur le gate qui flakait (ci.yml /
  verify.sh / .woodpecker utilisent `cargo test`). Phase F a relance
  `cargo test -p nexus-shell-daemon -p nexus-worker-core --locked` sur Windows
  natif (gate d'origine) — verifier dans `verification.md` que ce run est vert
  (sinon la fermeture 3/3 n'est prouvee que sous nextest).

**Findings routes ici (Track A)** :
- **Reconciliation compte** : entree canonique Linux S72 = **1544**. Sortie S73
  Linux = **1570** (+26). Windows = 1566 (+22). Deltas par phase **mesures
  Windows** : A +5, B +7, C +5, D +5, E +0 = +22. L'ecart +4 (Linux-only
  `#[cfg(unix)]`) explique la difference +26 vs +22. **Verifier la decomposition
  par `nextest list` sur les 2 plateformes** (Track E) : confirmer que les 4
  tests sont `#[cfg(unix)]`, pas des skips.
- **SQLite 3.49.2 vs 3.50.x** : le kickoff §3 ecrivait SQLite 3.50.x ; la
  realite bundled (`libsqlite3-sys 0.34`) est **3.49.2**. `INSERT OR REPLACE` +
  WAL + FTS5 UNINDEXED sont stables sur 3.49.2 — confirmer qu'aucun comportement
  FTS5 version-sensible (tokenizer, contentless) n'est suppose.

### Track B — Security review (guardrail ordering + search surface + provenance)

Le coeur securite S73 = (1) guardrail AVANT persist sur les 2 chemins (D5),
(2) la nouvelle surface search (endpoint + barre shell), (3) l'integrite du
triplet provenance enrichi.

**B.1 — Guardrail-before-persist, completude de TOUS les chemins persist (D5,
candidat P1)** : grep tous les appelants **production** (non-`#[cfg(test)]`) de
`db.set_task_result` — attendu : **uniquement** `validate_result_post_guardrail`
(`validator.rs:158`), appele depuis `http.rs:1531` et `validator_loop.rs:~91`,
chacun **apres** `default_output_chain().run()`. Verifier :
- (a) `coordinator_submit_result` (`http.rs:1500-1558`) : pre → guardrail →
  post ; sur trip, **400 + zero `set_task_result` + zero kudos** (confirme Phase F :
  `http.rs:1516-1530`, aucun persist sur la branche `!gr.passed`).
- (b) `validator_loop::process_result` (`validator_loop.rs:62-91`) : guardrail
  injecte AVANT persist (le chemin gossip n'avait **aucun** guardrail avant S73) ;
  trip → log + skip persist + **pas de credit kudos**.
- (c) **Aucun 3e chemin** ne flippe un task a `completed` ou n'ecrit
  `result_text` en contournant `set_task_result` (network-provider completion,
  accumulateur quorum, UPDATE direct).
- **Enforcement architectural** : `ResultValidator` (struct+impl) est gate
  `#[cfg(test)]` (`validator.rs:~299,304`) — c'est la SEULE chose qui empeche
  la re-introduction d'une composition `pre+post` sans guardrail. Le type
  `PendingResultPersist` **ne porte aucune preuve** que le guardrail a tourne :
  l'ordre est une **convention d'appelant**, pas un invariant de type. **Verifier
  qu'aucune fn `pub` production ne compose pre+post sans guardrail.** (Si un futur
  appelant oublie le guardrail → re-ouverture du bug S72. Candidat P1 si trouve.)

**B.2 — Quorum × guardrail (candidat P2, caracterise Phase F)** : sur le chemin
quorum (`redundancy>1`), `validate_quorum_pre_guardrail` (`validator.rs:~205`)
ecrit le texte worker dans `task_results` (table d'accumulation des votes)
**avant** tout guardrail. Quand le quorum est atteint et que le texte agree
(`best_hash`) trip le guardrail :
- Le `tasks.result_text` reste NULL, 0 kudos → **l'invariant headline tient**
  pour la surface **recuperable** (`GET /result`).
- MAIS : (a) le texte rejete reste dans `task_results` (etat interne quorum,
  **pre-existant**, pas expose par `GET /result` — verifier qu'**aucun** endpoint
  ne lit `task_results.sha256` brut) ; (b) la branche trip ne pose **aucun
  statut terminal** (ni Completed ni Rejected) — la tache reste `awaiting_quorum`
  et re-trip a chaque soumission ulterieure (zombie). Comparer a la branche
  divergence (`validator.rs:~279`) qui, elle, appelle `update_task_status(Rejected)`.
- **Decision audit** : (a) est-ce un P2 precision-doc (qualifier « jamais
  persiste » → surface recuperable, pas tout stockage) ? (b) est-ce un P2
  disponibilite (zombie task) ou by-design (re-dispatch tolere) ? **Aucun test
  daemon (HTTP/loop) n'exerce un resultat redundancy>1 qui trip le guardrail** —
  le seul test quorum (`quorum_guardrail_runs_on_agreed_text`) appelle la
  fn `pre` directement, bypass le daemon.

**B.3 — HTTP→loop re-broadcast (candidat P2)** : `http.rs:1551-1553` envoie
`ResultEvent::NewResult(entry)` sur `result_event_tx` APRES persist + kudos. Le
`validator_loop` re-consomme et re-lance `validate_result_pre_guardrail`.
Verifier que ce re-traitement est un **no-op** (le task est `completed`, garde
de statut `set_task_result UPDATE WHERE status IN ('pending','dispatched',
'awaiting_quorum')` `db.rs:~443` → pas de double-persist, pas de double-credit
kudos). **Aucun test n'asserte ce no-op** (risque inflation kudos / fairness).

**B.4 — Guardrail context vide (candidat P2)** : les 2 chemins construisent
`GuardrailContext { system_prompt: "", user_prompt: "", model_output }`
(`http.rs:1510`). Lire `guardrails.rs::default_output_chain` et confirmer
qu'**aucune** regle ne depend du prompt (relevance/grounding/prompt-leak) — sinon
elle no-op silencieusement. **Pre-existant** (le reorder a preserve les prompts
vides), pas une regression Phase A — a confirmer-ou-documenter.

**B.5 — XSS guard `repo_url` (candidat P2, Phase E)** : `isHttpsUrl`
(`Browse.tsx:~145`) = `startsWith('https://')`. Verifier que `javascript:`,
`data:`, `http:`, et **protocol-relative `//evil`** echouent tous (le test ne
couvre que `javascript:`). Confirmer `rel="noopener noreferrer"` + `target="_blank"`
sur l'ancre, et qu'aucun autre champ hit (project_name, description) n'est
injecte en HTML/href. **Carry P2-3** : la garde n'est PAS normalisee sur les
3 ancres `repo_url` pre-existantes (`Browse.tsx:264`, `BrowsedProject`,
`VerificationDetail`) — non gardees. (Threat = DOM du shell trusted, pas l'iframe
sandbox `allow-scripts` sans `allow-same-origin`.)

**B.6 — `is_open_source=true` sans `provenance_hash` (candidat P2)** : le
validateur feed (`public_feed.rs:285`) rejette cette combinaison pour les ops
feed → lignes feed coherentes. MAIS `extract_index_fields` lit les 2 champs
**independamment** (`search.rs:~210`) sans cross-check, et `index_entry` (chemin
browse, S74) prend une `Provenance` arbitraire sans validation. Aujourd'hui seul
le feed valide atteint l'index (browse non-cable). **S74 browse-indexing doit
re-appliquer l'invariant spec §2.1** ou le JSON search peut afficher
`is_open_source=true` / `provenance_hash=null` (gap integrite provenance pour le
fork S74).

**B.7 — `last_err` fuite token/URL (candidat P3)** : `provider_router.rs:~386`
surface `last_err` au timeout. Confirmer que le token bearer (header
`X-SBFB-Token`, pas dans l'URL) et les `status_url`/`result_url` ne sont
**jamais** interpoles dans le message d'erreur (reqwest `Display` peut embarquer
l'URL). Loopback single-user → P3, mais confirmer l'invariant token-free.

### Track C — Patterns review

Verifier la coherence post-S73 de `docs/rust/PATTERNS.md` :
- **§P56** (FTS5 hot reindex D1 + triplet UNINDEXED D2) : confirmer qu'il decrit
  le rowid=seq, le helper `extract_index_fields` partage, la migration M17
  DROP/recreate, et la tripwire collision rowid browse/feed.
- **§P54** (worker-pump) : la section « P2-A-1 closure (Sprint 73 Phase B) »
  doit refleter le fix `multi_thread` + l'exception virtual-time + le statut
  **CLOSED**. **Candidat P2/P3** : la regle « tout test pilotant le pump en
  real-time DOIT etre multi_thread » — verifier
  `runtime.rs:1906 rate_limit_gate_reloads_live_policy` : c'est un test qui spawn
  le pump en **REAL time** (notify file-watcher, pas `tokio::time::pause`) mais
  reste `#[tokio::test]` (current_thread). Soit il viole la regle §P54 (a
  convertir, carry), soit il est genuinement exempt (a documenter). **Le run
  Windows nextest 1566 0-skip l'inclut et il passe** — mais sous `cargo test`
  partage (gate d'origine du hang) la question reste : peut-il hang ? (Cf. A.1.)
- **§P53/§P55** : lot P3 doc S72 corrige en Phase F (rename `ModelOptions`/0.3.4,
  P2-A-2 ferme, `Box<dyn LlmBackend>` trait pas Deref-enum, `PROVIDERS: &[&str]`).
  Confirmer.
- **C.1 — Convention d'appelant vs type (D5)** : cf. B.1 — l'invariant
  guardrail-before-persist est une convention, pas encode dans le type. Pattern
  a surveiller.
- **C.2 — Freshness narrative vs ReleasePublished (candidat P2)** :
  `extract_index_fields` (`search.rs:183-226`) ne lit que `reason`/`comment` ;
  `ReleasePublishedPayload` (`public_feed.rs:32-40`) n'a **ni reason, ni
  project_name, ni category** → un hot-upsert d'un ReleasePublished indexe
  `description=''` → **invisible a la recherche full-text**. Seuls
  CuratorVouched/SourceBecameStale (porteurs de `reason`) deviennent matchables.
  **Le claim headline « un projet gossipe devient cherchable a l'instant »
  est materiellement surcote pour l'op la plus importante (publication projet).**
  Aucun des 5 tests Phase C ne couvre un ReleasePublished. Decision audit :
  est-ce un gap fonctionnel (le nom de projet devrait etre indexe) ou un
  scope-cut implicite vers S74 ?
- **C.3 — rowid=seq vs browse auto-rowid (candidat P2, tripwire S74)** :
  `index_entry` (`search.rs:67-97`) INSERT **sans** rowid explicite → FTS5
  assigne `max(rowid)+1`, partageant l'espace rowid des upserts feed (rowid=seq).
  Confirmer que les 2 appelants `index_entry` (`http.rs:6419,6459`) sont
  `#[tokio::test]` (browse-indexing **test-only** en prod aujourd'hui). Si S74
  cable le browse-indexing prod : un upsert feed `seq=N` clobbe silencieusement
  une ligne browse rowid=N (INSERT OR REPLACE). Tripwire doc `search.rs:241-244`
  + §P56. **Verifier qu'aucun appelant prod `index_entry` n'a glisse.**
- **C.4 — best-effort upsert sans catch-up runtime (candidat P2)** :
  `feed_sync.rs:268-279` ne fait que `warn!` sur echec upsert (l'insert durable
  est deja commit). `rebuild_from_feed` semble **boot-only** (`runtime.rs:778`).
  Un echec SQLITE transitoire laisse 1 entree non-cherchable pour toute la duree
  de vie du noeud. Confirmer si acceptable + s'il existe une metrique de drift.

### Track D — Scope cuts compliance

14/14 scope cuts (kickoff §7 / plan §7) auto-reportes dans `verification.md §6`.
Verifier par grep exhaustif qu'aucune ligne S73 ne touche : SearchManifest
reseau-large (#1, defere D3), `search/open/fork` Factory (#2, S74), projet cible
distinct nexus (#3, S74), reseau→fork (#4, S74), templates etendus (#5, S74),
GPU cross-machine (#6, S75), quorum cross-MACHINE (#7, S75), sharding (#8, S76),
Tantivy (#9, gele), @dev tree-sitter (#10), rate-limit per-client search (#11,
re-eval Phase E), webhook/SSE feed push (#12), token-par-token WAN (#13, jamais
PO-14), pagination boutons (#14).

- **D.1 — Scope cut #11 (candidat P2)** : le kickoff §7 #11 imposait une
  **re-evaluation binaire en Phase E**, pas un defer en blanc. Le commit Phase E
  justifie par « endpoint loopback single-user + debounce de fait (enabled +
  keepPreviousData) ». Verifier : (a) `GET /api/daemon/search` est bien
  loopback-only **derriere** `auth_required` (ordre route `http.rs:360` vs layer
  `http.rs:436`) ; (b) il n'y a **aucun** debounce keystroke dans `Browse.tsx`
  (le seul rate-bound est React Query `staleTime` — contournable en tenant une
  cle) → si « debounce de fait » est faux, le residual T-SEARCH-DOS repose
  **uniquement** sur loopback single-user, pas sur un debounce. (c) `q` sans
  longueur max, `offset:usize` non-clamp passe a SQLite `LIMIT/OFFSET`
  (`search.rs:122`). THREAT_MODEL §11 doit documenter le residual acceptable.
- **D.2 — SearchManifest defer (D3, candidat P2/P3)** : confirmer
  `.planning/research/s73_searchmanifest_index_node_design.md` (NEW, ~250 l)
  capture la forme correcte (noeud-index opt-in Ed25519, default OFF, requetes
  jamais broadcast, 7 modeles OSS, critere declenchement). Confirmer **ZERO code
  wire** : `PublicFeedOperation` (`public_feed.rs:~85`) a **exactement 4
  variantes** (SearchManifestPublished n'est qu'un commentaire forward-compat
  `:78`), aucun struct/fn/serde tag `SearchManifest`, `FEED_FORMAT_VERSION=1`.
  R6 : verifier que c'est **TRANCHE** (decide + documente + PO-13 honore), pas
  oublie.
- **D.3 — Network default model (candidat P3)** : `default_model_for_provider
  ('network')` retourne `llama3.2:latest` (`operator_server.rs:331`), soumis
  verbatim au worker **distant** (`provider_router.rs:345`). Un nom de modele
  local-Ollama comme defaut d'une tache reseau (le worker distant peut avoir
  d'autres modeles). Confirmer que `SBFB_NETWORK_DEFAULT_MODEL` est l'echappatoire
  voulue + que c'est la dette P2-OLLAMA-MODEL-PICKER, pas une capacite produit
  nouvelle (scope creep).

### Track E — Tests delta coherence

Verifier les deltas par phase (mesures Windows) : A +5, B +7, C +5, D +5, E +0
Rust = +22 ; Vitest `web/` +10 (279→289) ; `factory-operator` +7 (0→7 infra NEW).
- **E.1** : reconcilier 1544 (entree Linux) → 1570 (sortie Linux, +26) vs +22
  Windows. L'ecart +4 = `#[cfg(unix)]` (cf. A). Confirmer par `nextest list`.
- **E.2 — Phase A** : 5 tests (`quorum_guardrail_runs_on_agreed_text`,
  `submit_result_rejected_by_guardrail_persists_nothing`,
  `submit_result_accepted_persists_after_guardrail`,
  `validator_loop_rejected_result_not_persisted`,
  `validator_loop_accepted_result_persisted`). **Gap** : aucun test daemon
  redundancy>1 × guardrail-trip (cf. B.2) ; branche `post_guardrail → Err`
  (`http.rs:1531`, `validator_loop.rs:91`) untestee (review P3 « defensive »).
  Confirmer que les assertions sont load-bearing (`get_task_result().result_text
  .is_none()` + `kudos_total==0`).
- **E.3 — Phase C/D test-fidelity (candidat P2)** : `reindex_hot_is_idempotent`
  asserte `total==1` mais **pas** que le contenu a ete REWRITTEN (upsert seq=42
  reason A puis B, asserter search trouve B pas A) ; `extract_index_fields_
  shared_with_rebuild` compare sur UNE op porteuse de reason, **pas** un
  ReleasePublished (le cas ou la derive triplet importe) ;
  `migration_m17_recreates_index_unindexed` simule via `clear_all`+rebuild sur
  DB in-memory (toutes migrations appliquees), **jamais** un upgrade reel
  user_version 16→17 sur table peuplee (review P3-2). Les noms de tests
  promettent plus que les assertions ne prouvent.
- **E.4 — Phase E Vitest +10 vs plan +4** : le +6 « adversarial deliberate »
  doit exercer des branches distinctes (encode pathologique, 503→unavailable,
  triplet null, strict-omitted-key reject, XSS non-https, grille vide). Confirmer
  exact 289/289, pas du padding.

### Track F — Review files quality + presence exhaustive

- **5 preflight** (A-E) : verifier les verdicts G8 (Phase F bilan
  `verification.md §5`) — attendu A EXECUTE, B EXECUTE, C EXECUTE, D EXECUTE,
  E SCOPE-CUT-CONSISTENT (drift plan→reel enveloppe). **5 reviews** (A-E) toutes
  promues `## Verdict: PASS` (format exact). **5 codex_review** bruts.
- **§4.4 ratio** : `Phase review files present: 5/5`.
- **Lire les codex_review BRUTS** (pas les resumes commit-body) : en particulier
  **Phase A Codex Run 1** a trouve un **PARTIAL** (`ResultValidator` composant
  guardrail-less) ferme en Run 2 par le gate `#[cfg(test)]` — confirmer la
  reconciliation. Verifier chaque PARTIEL/GAP Codex reconcilie.

### Track G — Carry-overs

- **CLOSED S73 (Phase B, 7 P2)** : verifier chaque cloture (test + code) :
  P2-A-1 worker-pump 3/3 (**plus jamais carry** ; cf. A.1 gate `cargo test` +
  C.2 §P54 completude), P2-TEST-ZOMBIE (fixtures git self-contained — verifier
  hermeticite : `user.email/name` explicite, `current_dir` temp, body 9 headers
  exacts), P2-OPERATOR-TIMEOUT (30s configurable), P2-OPERATOR-NO-TEST-RUNNER
  (Vitest 7), P2-POLL-DIAGNOSTIC-LOSS (`last_err`, cf. B.7), P2-SYNC-FS-ASYNC
  (`spawn_blocking`), P2-OLLAMA-MODEL-PICKER (per-provider, cf. D.3).
- **CLOSED S73 (Phase A, 3 doc)** : P2-RESULT-TEXT-GUARDRAIL-ORDER (cf. B.1-B.4),
  P2-TIER-MODEL (Operator tier formel LOOPBACK §2.1/§8.1), P2-HARDENING-ROADMAP-
  META-STALE (recadre §3 + last_validated 2026-06-03).
- **CLOSED S73 (Phase F, 2 process)** : P2-PREFLIGHT-TRANSITIVE-DEPTH (S1b lock +
  `cargo tree -d`) + P2-PREFLIGHT-WIRE-CONTRACT-DEPTH (S4 trace producteur→
  consommateur) — amendes dans `prompts/agent/preflight.md` + skill +
  agent-deep. Verifier presence.
- **Nouveaux S74 (P2/P3 non bloquants — router vers tracks)** : cf. les candidats
  ci-dessus, notamment :
  - **P2-SEARCH-VIEW-THROW-SKELETON** (NEW, Phase E, auditor-found, **pas dans
    le self-report**) : `callDaemon` THROW `ApiProtocolError` sur `strict()`
    parse-fail (`daemon.ts:249-251`) ; `SearchResultsView` (`Browse.tsx:200-210`)
    n'a **aucune** branche `query.isError` → un drift Rust↔Zod yield un
    `LoadingSkeleton` **infini** (spinner bloque), pas une carte d'erreur.
    Distinct de P2-1. A inscrire.
  - P2-FRESHNESS-RELEASE-UNINDEXED (C.2), P2-ROWID-PARTITION (C.3, tripwire),
    P2-UPSERT-NO-CATCHUP (C.4), P2-M17-BOOT-RECOVERY-WARN-ONLY (warn-only
    `runtime.rs:781` → index silencieusement vide si rebuild echoue post-DROP),
    P2-SEARCH-RATE-LIMIT-RESIDUAL (D.1), P2-QUORUM-GUARDRAIL-RESIDUE (B.2),
    P3-runtime-1906-cfg (C.2 §P54).
  - **P2-1** (Phase E, review-acknowledged) : branches unavailable/error du
    search view non testees. **P2-3** (Phase E) : scheme-guard non normalise
    sur les ancres `repo_url` pre-existantes.
- **Reconduits (exemptes / hors-scope)** — verifier qu'aucun n'atteint 3 reports
  sans exemption (escalade G7) :
  - P2-A-1 (rand upstream) — blocker amont, exemption externe.
  - P2-AUDIT-2 (iroh transitives pre-release) — pin 0.98 gele.
  - T-NN+2 (iframe Rust-wasm) — depend wasm amont (PATTERNS §P34).
  - P3-OS-1 (operator_server OR duplique) — pre-existant, **non touche S73**.
  - **LT-2 Radicle** — trigger PENDING (tag v1.0 pose localement, **PAS pousse**
    vers origin ; 37 ahead). Reste latent.
  - LT-5/LT-7 (worker quorum E2E) — post-v1.0 / S75.

### Track G1 presence (P1 bloquant si absent)

Verifier que `sprint73_design_review.md` existe dans `active/` (ou migre
`archive/v2.1/` au S74 Phase 0) avec scoring G1 (D1 ✅ D2 ⚠️ D3 ⚠️ D4 ✅ D5 ✅
D6 ✅). Present sur sprint feature non-trivial = OK. Absent = **P1** (gate
bypasse). Present sans scoring = P2.

### Track H — HARDENING review

S73 ajoute/modifie : (1) l'ordre guardrail (nouvel invariant securite — un
texte rejete n'est jamais `completed`/recuperable), (2) la surface search
(endpoint + barre shell interactive → trafic accru, cf. D.1 T-SEARCH-DOS),
(3) la migration FTS5 M17 (DROP/recreate). Comparer `HARDENING_ROADMAP.md §3`
(recadre Phase A) vs livre :
- Confirmer `THREAT_MODEL §14` (guardrail AVANT persist, lignes 789-793) +
  `§11` (search surface, recadre boot→hot Phase C) coherents avec le code.
- **H.1 — M17 recovery window (candidat P2)** : un noeud existant qui upgrade
  DROP son index peuple a la migration, recovery via `rebuild_from_feed`
  best-effort au boot qui **ne fait fail boot** (`runtime.rs:781 warn!`). Si le
  rebuild echoue post-DROP → index silencieusement VIDE jusqu'a un reboot
  reussi. Pre-launch recoverable (feed durable, D2) mais confirmer que la
  recovery est atteinte et non-gatee derriere un echec silencieux.
- **H.2 — browse-row reconstructibility** : `rebuild_from_feed` ne restaure que
  `source_type='feed'`. M17 DROP **tout**. Le claim « integralement
  reconstructible » est vrai **uniquement** pour la tranche feed. Browse-rows
  (S74) seraient perdues — carry.
- Pour chaque item HARDENING prescrit S73 non livre : scope-cut justifie ou
  blocker → sinon P2 (drift). Track informatif (P2).

### Track I — Meta-process

- **P2-PREFLIGHT-* (Phase F)** : les amendements S1b (graphe transitif) + S4
  (trace wire producteur→consommateur) repondent **directement** a la lecon
  meta S72 (2 DESIGN-CONFLICT consecutifs sous-estimaient les deps transitives
  + le contrat wire cross-composant). Verifier que les amendements sont
  concrets (commandes `cargo tree -d`, exemple Phase E nullable-vs-optional),
  pas du boilerplate.
- **Process env (inchange S72→S73)** : `nexus-phase-review-deep` ET
  `nexus-process-supervisor` **non enregistres** → reviews = fallback agent
  `general-purpose` independant ; supervision = hooks backstop (D17). Verifier
  que les reviews independantes existent et portent un verdict.
- **Docker-sur-Windows** : note process — la repro CI Linux fidele exige
  `MSYS_NO_PATHCONV=1` + `bash -c` (pas `-l`) + **`apt-get install libgtk-3-dev`**
  (atk-sys) + `CARGO_TARGET_DIR` isole (eviter contention avec le target Windows
  natif tournant en parallele). Sans libgtk-3-dev, `cargo test --no-run` echoue
  a la compilation (`atk-sys`) — un exit code peut etre masque par un `| tail`.
- **nextest vs cargo test** : le gate canonique (ci.yml/verify.sh/.woodpecker)
  utilise `cargo test` (shared-process), PAS nextest. Le hang worker-pump
  (P2-A-1) et les flakes env (serial_test) se manifestent sous `cargo test`
  partage. Auditer le compte sous nextest **et** confirmer le worker-pump sous
  `cargo test` (cf. A.1).
- **Verification C/D deferee a F** : confirmer que le run Docker Linux de Phase F
  couvre bien M17 + hot-reindex (pas un re-cite du 1556/1560 de Phase B
  anterieur a M17). cf. §0.
- **Commit discipline** : 5 phases (A/B fix, C/D feat search, E feat shell) +
  chores (kickoff `845bea6`, normalize `5361fd8`, Phase F docs). Bodies 9
  sections phases code. Codex gate 5/5. Verifier les SHA `6f5ff30`/`a4e1542`/
  `47c9ff7`/`0f86e5a`/`9472085`. Phase F = `docs(sprint73)` titre **sans « Phase
  X »** → hooks lightcheck Check 5/7/8/9 NON armes (precedent S71, README §3.3).

---

## §3 S74 Objective — Atelier fork (contexte, hors audit)

Apres l'audit S73, S74 ouvre (roadmap v5 §3, Arc 3.5, sprint **4/6**) l'**atelier
fork** : rouvrir/forker un projet reseau decouvert via la recherche S73. S73 a
enrichi `SearchResult` avec le triplet provenance (`repo_url`+`commit_sha`+
`archive_hash`+`provenance_hash`) que **S74 reutilise pour forker**
(`repo_url@commit_sha` forge, ou `archive_hash` blob en repli — PO-5). Sans cet
enrichissement, un hit de recherche ne peut pas declencher un fork. **Pre-requis
a inscrire au plan S74** (carries S73) : le partition rowid browse/feed (C.3)
AVANT tout browse-indexing prod ; la re-application de l'invariant
`is_open_source⇒provenance_hash` au chemin browse (B.6).

---

## §4 Out of scope pour l'audit (NE PAS rebattre)

L'audit S73 **audite**, il ne re-conçoit pas. Ne pas rebattre :
- **D1..D6 gelees** : reindex hot upsert `INSERT OR REPLACE` par seq (D1),
  triplet UNINDEXED + M17 DROP/recreate (D2), **defer SearchManifest** feed-local
  + design note (D3), barre = champ dedie Browse `searchBrowse()` (D4), guardrail
  AVANT persist 2 chemins (D5), worker-pump fix `multi_thread` cross-platform
  (D6).
- **Les 14 scope cuts** (kickoff §7) — fork/templates/packaging S74, GPU/quorum
  cross-machine S75, sharding S76, Tantivy gele, token-par-token jamais (PO-14).
- **Pre-launch policy** : pas de bump `*_VERSION` tant que rien n'est pousse
  (37 ahead, rien pousse) ; M17 = schema **local** SQLite, pas un wire ;
  `FEED_FORMAT_VERSION=1` ; canonical editable. Ne PAS exiger de migration wire.
- Re-corriger un P2/P3 deja documente (router vers S74+ phases, pas le
  re-implementer en Phase 0).

---

## §5 Track HARDENING drift (P2 informatif) — rappel

Cf. Track H. Drift cumule sur 3+ sprints sans justification → remonter le signal
pour revalider `HARDENING_ROADMAP.md` lui-meme. S73 = nouvel invariant securite
(guardrail ordering) + extension surface (search interactive). Le recadre §3
(P2-HARDENING-ROADMAP-META-STALE, Phase A) doit tenir.

---

## §6 Verdict global attendu

- **PASS** : 0 P0, 0 P1 → S74 Phase A demarre direct. **Scenario attendu** —
  les ~10 P2 candidats sont des questions de couverture-test / precision-doc /
  hazards latents S74 (browse-indexing), non bloquants ; les invariants headline
  (guardrail AVANT persist sur la surface recuperable ; worker-pump 3/3
  empiriquement vert ; defer SearchManifest tranche) tiennent ; G1 present.
- **CONDITIONAL PASS** : 1-3 P1 fixables → S74 Phase A bloque tant que les
  `fix(sprint73): ...` ne sont pas landed. **Candidats P1 a trancher** :
  (1) B.1 — un 3e chemin production compose `pre+post` sans guardrail (re-ouvre
  S72) ; (2) A.1/C.2 — le worker-pump hang **sous `cargo test`** ou
  `runtime.rs:1906` peut deadlock (3/3 non clos) ; (3) D — un wire bump /
  variante SearchManifest a glisse.
- **FAIL** : >= 1 P0 ou >= 3 P1 → re-conception partielle.

---

## §7 Livrable final attendu

`sprint73_audit_findings.md` (pattern Sprint 6/7), sections :
1. **Auditeur** — id session, duree.
2. **Tip audite** — SHA master pris comme base (tip code `9472085`).
3. **Verdict global** — PASS / CONDITIONAL PASS / FAIL.
4. **Une section par track A-I** avec verdict (PASS / CONCERN / FAIL) + findings.
5. **Findings list sorted by severity** — table P0 → P3.
6. **Commits fix attendus** — si CONDITIONAL PASS, liste `fix(sprint73): ...`
   prealable au kickoff S74.
7. **P2 a logger en tech debt** — items vers `PATTERNS.md` sans code change.
8. **P3 laisses sans action** — nits ignores.
9. **Notes on audit completeness** — ce qui n'a pas ete couvert et pourquoi.

**Critere SMART** : la fail-fast `verification.md` rejoue verte en CI Linux
(**1570/1570, 0 skip**) + worker-pump vert sous `cargo test` partage Windows +
0 P0/P1 non resolu = S74 kickoff debloque (atelier fork).

**Exit Gate** : l'audit S73 est complet quand `sprint73_audit_findings.md` porte
un verdict avec >= 1 P2+ (G4), couvre les 9 tracks, ingere le diff complet S73
(Phases A-E), confirme que les invariants headline (guardrail AVANT persist sur
la surface recuperable ; P2-A-1 3/3 CLOSED ; defer SearchManifest tranche) sont
sains, et tranche les 3 candidats P1 du §6.
