# Sprint 77 — Phase J review

## Verdict: PASS

Review independante profonde (synthese de 5 dimensions + verification adversariale
ligne-par-ligne du code reel via `git --no-pager diff` + Read des fichiers source).
**0 P0, 0 P1 CONFIRME.** Le PLAN-ADAPT du preflight est applique correctement et
integralement (stub etat-vide 200, member_count agrege, aucun registre invente,
DaemonHttpState intouche, whitelist identite verrouillee). L'AMENDEMENT du preflight
(retrait `pipeline_status` + `verification_level` du DTO Phase J) est **JUSTIFIE**
(prouve a la source : le manifeste n'offre aucune source pour ces 2 champs, les
livrer = variantes d'enum jamais construites = dead_code reel sous `clippy -D warnings`).

PASS-PENDING (non final) : la gate Codex GPT 5.5 reste a passer avant promotion en PASS
et commit. 4 findings doc-honnetete (1 P2 + 3 P3) sont a corriger in-phase AVANT Codex
(typiquement sanctionnes par les rounds Codex) + 1 P2 process (reconciliation delta tests
dans le commit body).

## Resume par dimension

| # | Dimension | Verdict | Findings actionnables retenus |
|---|---|---|---|
| 1 | CORRECTNESS / BUGS | PASS | 0 (3 info cosmetiques) |
| 2 | SECURITE / THREAT | PASS | 2 P3 doc (commentaire « four » + THREAT §16 « Phase J ») |
| 3 | SCOPE / PLAN-ADAPT / DAY-0 | PASS | 1 P2 (delta tests commit body) + 1 P3 (doc-comments route/handler) |
| 4 | TESTS | PASS | 0 (2 P3 couverture optionnelle) |
| 5 | PATTERNS / CONVENTIONS | PASS | 1 P2 (doc-comments route) + 1 P3 (emplacement components/ vs pages/) |

Aucune dimension n'emet de FAIL ni de CONCERN bloquant. Le critere CONCERN
(« amendement PLAN-ADAPT mal justifie ») ne s'applique PAS : l'amendement est
justifie a la source (cf. table findings + section « Decision sur l'amendement »).

## Verification adversariale des findings P0/P1/P2 (CONFIRMED / REFUTED)

| # | Dim | Sev | Titre | Statut | Evidence (file:line) |
|---|---|---|---|---|---|
| F1 | D2/D3/D5 | — | Whitelist identite : la route ne peut PAS serialiser une pubkey membre | **CONFIRMED (sain)** | `ShardSessionView` = `#[derive(Serialize)]` avec UNIQUEMENT `session_id:String` + `member_count:usize` (http.rs:2100-2110) ; `project_shard_session` lit SEULEMENT `manifest.session_id` + `manifest.plan.assignments.len()` (http.rs:2126-2131), ne touche jamais `manifest.initiator` ni `assignments[*].worker_pubkey` ; le manifeste source porte bien `initiator:[u8;32]` (shard_plan.rs:243) + `assignments[*].worker_pubkey:[u8;32]` (shard_plan.rs:18) mais AUCUN n'est copie. Serde derive ne peut emettre que les 2 champs declares. Test `shard_session_projection_hides_member_identities` (http.rs:5269-5330) assert l'absence des bytes hex des 3 pubkeys + des litteraux `worker_pubkey`/`initiator` + `json.len()==2`. **Aucune fuite possible.** |
| F2 | D5 | **P2** | Doc-comments route (l.305-309) + handler (l.2171-2176) sur-claiment « pipeline status, attained verification level » que le DTO 2-champs n'expose pas | **CONFIRMED** | http.rs:307 et http.rs:2174 ecrivent « an AGGREGATE status (member count, pipeline status, attained verification level) » alors que `ShardSessionView` n'a QUE 2 champs et son propre doc-comment (http.rs:2095-2106) explique honnetement le report Phase K. Incoherence interne reelle, residu du DTO 4-champs du preflight. **Fix in-phase 2 lignes.** |
| F3 | D3 | **P2** | Divergence delta tests : Rust +2 / Vitest +9 vs plan §15 estimait +0 / +3 | **CONFIRMED** | `git diff` montre +2 fn Rust (`shard_session_response_pins_empty_envelope` http.rs:5246 + `shard_session_projection_hides_member_identities` http.rs:5269) et +9 Vitest (4 dans ShardSessionPanel.test.tsx + 5 dans daemon.test.ts block getShardSession:976-1056). Le preflight risk #7 a pre-flagge le Rust +2 ; la part Vitest +9-vs-+3 N'EST PAS pre-flaggee. Regle anti-faux-vert S76 (git-count == delta annonce) impose la reconciliation honnete dans le commit body. **Pas un defaut code — discipline process.** |
| F4 | D2/D3/D5 | P3 | Commentaire de test obsolete « the four whitelisted fields » | **CONFIRMED** | http.rs:5274 dit « The view is exactly the four whitelisted fields. » alors que l'assertion en force 2 (`json.len()==2`, http.rs:5320-5324). Residu du DTO 4-champs. **Fix 1 mot.** |
| F5 | D2 | P3 | THREAT_MODEL §16 reference « Phase J » pour le data-plane desormais Phase K | **CONFIRMED** | THREAT_MODEL.md:918 (row I), :1228, :1376 disent « = Phase J/K » / « Phase J/data-plane » pour re-exec in-vivo + transport sketch + arbitrage litige + timeout/fallback SI-9. La Phase J reelle = control-plane read-only stub ; data-plane repousse Phase K. **Rafraichir les 3 mentions au wrap-up + garder SI-9 carry ouvert.** |
| F6 | D5 | P3 | Route `/compute` lazy-load depuis `@/components/` au lieu de `@/pages/` (P12) | **CONFIRMED (mineur)** | App.tsx:86 `lazy: () => import("@/components/ShardSessionPanel")` ; les 9 autres routes chargent depuis `@/pages/*`. ShardSessionPanel est un composant de PAGE (PageHeader, layout pleine page, mirror Nodes.tsx) qui reside dans components/. Contrat technique (Component+default export, ShardSessionPanel.tsx:289) respecte → fonctionne ; seul l'emplacement devie. **Deplacable, non bloquant.** |
| F7 | D4 | P3 | Test panneau « session trouvee » sur-promet « jamais une identite » sans assertion negative FE | **CONFIRMED (benin)** | ShardSessionPanel.test.tsx:118 nom + commentaire affirment « jamais une identite » ; assertions (l.137-140) verifient member-count + label, AUCUNE assertion negative `queryByText(pubkey)`. L'invariant EST tenu en amont (test Rust boundary + schema Zod sans champ pubkey) → 0 faux-vert. **Honnetete de libelle, pas un trou de couverture.** |
| F8 | D4 | P3 | Branches isLoading/isError/unavailable/error de `SessionStatus` non testees unitairement | **CONFIRMED (benin)** | ShardSessionPanel.tsx:223-251 a 4 branches ; les 4 tests panneau couvrent etat-vide/not-found/found/no-coordinator. Precedent S74 SEARCH-VIEW exigeait un test isError. Couverture globale passe (87.27 stmts >= seuil). **Optionnel post-Phase J.** |
| F9 | D1 | info | `member_count = assignments.len()` compte les assignments, pas les workers distincts | **CONFIRMED (non-bug)** | http.rs:2130. Sous le design fige (pipeline-parallel exclusif, addendum §1 = 1 assignment/worker), `len() == #workers`. A documenter si Phase K introduit le multi-bloc/worker. **Non-bug Phase J.** |
| F10 | D1 | info | `truncate()` peut ne pas raccourcir pour 19 caracteres | **CONFIRMED (cosmetique)** | ShardSessionPanel.tsx:283-285, `id.length <= 18` puis slice(0,10)+'...'+slice(-6)=19 chars pour une entree de 19. Aucun impact fonctionnel. **A laisser tel quel.** Mirror de `truncateHex` Nodes.tsx (convention etablie). |

**Aucun finding REFUTE** : tous les findings emis par les 5 dimensions resistent a la
relecture du code. Aucun reviewer n'a hallucine de fuite pubkey (le DTO n'a
physiquement pas le champ — verifie). Aucun vrai bug rate detecte lors de la
relecture adversariale independante.

## Recoupement SECURITE prioritaire — preuve par lecture que la route ne peut PAS fuiter une pubkey membre

Chaine verifiee de bout en bout (allowlist, pas blocklist) :

1. **Le DTO est une struct etroite distincte du manifeste** : `ShardSessionView`
   (http.rs:2100-2110) `#[derive(Debug, Clone, Serialize)]` ne declare QUE
   `session_id: String` + `member_count: usize`. Le `Serialize` derive ne peut
   emettre que ces 2 champs — il n'existe aucun chemin pour serialiser un 3e champ.
2. **La projection lit l'agregat, jamais l'identite** : `project_shard_session`
   (http.rs:2126-2131) construit la vue depuis `manifest.session_id.clone()` +
   `manifest.plan.assignments.len()`. Elle NE serialise PAS le
   `ShardedSessionManifest` (qui, lui, porte `initiator:[u8;32]` shard_plan.rs:243
   + `plan.assignments[*].worker_pubkey:[u8;32]` shard_plan.rs:18 +
   `fallback_node:Option<[u8;32]>` shard_plan.rs:45). Pattern allowlist robuste a
   un futur ajout de champ dans le manifeste (un nouveau champ identite n'apparait
   pas tant qu'on ne l'ajoute pas explicitement a `ShardSessionView`).
3. **Le stub ne sert jamais de manifeste** : `live_shard_session` (http.rs:2137-2139)
   retourne `None` en dur → `shard_session_response` (http.rs:2150-2162) renvoie
   TOUJOURS `{found:false, session:null}`. Aucun manifeste reel n'est jamais
   serialise en Phase J.
4. **Test adversarial bloquant** : `shard_session_projection_hides_member_identities`
   (http.rs:5269-5330) construit 2 workers (0xAA/0xBB) + initiator (0x11),
   `serde_json::to_string(&view)` puis assert `!contains(hex::encode(pubkey))` pour
   les 3, `!contains("worker_pubkey")`, `!contains("initiator")`, `member_count==2`,
   `json.len()==2`. C'est une assertion d'**absence-de-bytes**, pas un comptage de
   champs — elle echouerait si un Serialize exposait quoi que ce soit d'autre.
5. **Auth correcte** : la route vit dans `authed_routes` (http.rs:310-313), bloc
   termine par `.layer(middleware::from_fn_with_state(auth, auth_required))`
   (http.rs:506). `public_routes` = SEULEMENT /health + /blob-serve ;
   `token_route` = SEULEMENT /auth/token. `build_router` merge les trois
   (http.rs:508-512). Bearer + Host + Origin loopback T0 herites, 0 code d'auth ecrit.
6. **Pas d'injection / path-traversal** : `live_shard_session(_session_id)` IGNORE
   le param (http.rs:2137) → 0 usage FS/SQL/interpolation. Cote front,
   `getShardSession` (daemon.ts) `encodeURIComponent(id)`, prouve par le test
   `URL-encodes the session id path param` (daemon.test.ts:1005-1015) : `'a b/c'`
   → `'a%20b%2Fc'` (le `/` → `%2F` neutralise le path-traversal au routing axum).
7. **Front via callDaemon/authFetch, pas le bridge postMessage** : ShardSessionPanel
   importe `getShardSession` (@/api/daemon) → `callDaemon` → `authFetch` (bearer,
   meme-origine), JAMAIS le bridge whiteliste 3-methodes. 0 `dangerouslySetInnerHTML`/
   `innerHTML`/`eval` dans le panneau ; `session_id` rendu en text-node React
   auto-echappe (ShardSessionPanel.tsx:268). Framing UX = admission
   (« Le groupe est prive : seules les machines invitees y participent »,
   ShardSessionPanel.tsx:64-65), JAMAIS « prive = chiffre ».

**Conclusion securite : ZERO fuite d'identite trouvee.** Posture privacy identique
au precedent verifie `seed_count` (http.rs:2511, 200 + agregat `peer_count` sans
identites seeder). Carry SI-9 (withholding) reste correctement ouvert (control-plane
only, data-plane = Phase K).

## Decision sur l'AMENDEMENT (retrait pipeline_status + verification_level)

**JUSTIFIE — decision documentee, PAS un finding.**

Prouve a la source (`crates/nexus-core-rs/src/shard_plan.rs:234-272`) : le
`ShardedSessionManifest` n'a QUE `{version, initiator, session_id, group_id,
revision, plan, model_digest, tokenizer_hash, chat_template_hash}` — AUCUN champ
`pipeline_status` NI `verification_level` NI Task/criticite. Consequence :

- `member_count = plan.assignments.len()` EST derivable d'un manifeste statique stocke.
- `pipeline_status` (forming/running/…) et le niveau de verification ATTEINT
  requierent de la telemetrie d'un pipeline VIVANT qui n'existe pas en Phase J
  (`live_shard_session` → `None`).
- Les inclure forcerait la projection a construire UNE variante d'enum (ex. Forming/N0)
  → les autres variantes `is never constructed` → lint `dead_code` → echec sous
  `cargo clippy --workspace --all-targets --locked -- -D warnings` (gate projet,
  CLAUDE.md commandes cles).

C'est un **fix root-cause** (eviter un contrat impopulable / des variantes d'enum
mortes), PAS un `allow(dead_code)` band-aid ni un defer paresseux du coeur. La row
Zod est TOLERANTE (`ShardSessionViewSchema` SANS `.strict()`, daemon.ts) → l'ajout
additif 0-bump en Phase K est possible sans toucher le wire (teste :
`tolerates an additive field on the session ROW`, daemon.test.ts:1037-1056). La
charge adversariale « tu as juste retire des livrables du plan §13.2 » est REFUTEE :
le manifeste n'offre aucune source pour ces 2 champs en Phase J. Le critere CONCERN
(« amendement mal justifie ») ne s'applique donc pas.

**Exigence** : enoncer l'amendement explicitement dans le commit body (le DTO
Phase J = `{session_id, member_count}` ; status/level = Phase K additif 0-bump,
justification dead_code) pour que l'audit gate ne le lise pas comme un defer silencieux.

## P2 / P3 a documenter / corriger AVANT Codex (commit body)

**A corriger in-phase AVANT Codex (doc-honnetete, typiquement sanctionnee par Codex) :**
1. **[P2 — F2]** Aligner les 2 doc-comments route (http.rs:307) + handler (http.rs:2174)
   sur le DTO reel : « an AGGREGATE status (member count only ; le runtime pipeline
   status + attained verification level sont additifs Phase K) ». Fix root-cause
   2 lignes, pas de band-aid.
2. **[P3 — F4]** Corriger le commentaire de test http.rs:5274 « the four whitelisted
   fields » → « two » (coller a l'assertion `json.len()==2`).

**A documenter dans le commit body :**
3. **[P2 — F3]** Reconciliation delta tests HONNETE : Rust **+2** (les 2 tests daemon ;
   plan §15 estimait +0, carry P2 preflight pre-flagge le Rust) + Vitest **+9**
   (4 ShardSessionPanel + 5 getShardSession ; plan §15 estimait +3). Ne PAS recopier
   l'estimation §15 comme si elle etait observee. La part Vitest +9-vs-+3 N'etait pas
   pre-flaggee → la regle anti-faux-vert S76 (git-count == delta annonce) l'exige.
4. **[Process]** Enoncer l'amendement explicitement (cf. section ci-dessus).
5. **[Doc-defer trace]** Doc THREAT_MODEL §16 / PATTERNS pour la nouvelle route =
   Phase K (route stub 0-serve in-vivo) ; le signaler pour eviter un carry oublie.

**A traiter au wrap-up Phase J (ou Phase K) :**
6. **[P3 — F5]** Rafraichir les 3 mentions « Phase J » data-plane → « Phase K » dans
   THREAT_MODEL §16 (l.918, 1228, 1376) + garder SI-9 explicitement carry ouvert.
   Idealement une phrase §16/§15.3 documentant la route GET /api/daemon/shard-session
   comme surface read-only loopback agregat (miroir seed_count).

**Optionnel (non bloquant, pas requis avant commit) :**
7. **[P3 — F6]** Deplacer ShardSessionPanel.tsx → `web/src/pages/` (+ ajuster import
   App.tsx + test) pour aligner la convention P12 pages/ vs components/.
8. **[P3 — F7]** Ajouter une assertion negative legere au test panneau « session
   trouvee » (le mock ne contient deja pas de pubkey ; aligner le libelle a l'assert).
9. **[P3 — F8]** Ajouter 1 test `isError` (mock fetch rejette) couvrant la branche
   « Erreur reseau », par coherence avec le precedent SEARCH-VIEW.

## Invariants verifies (mecaniquement, git diff)

| Invariant | Etat | Evidence |
|---|---|---|
| **0 bump wire** | TENU | `git diff -G"_FORMAT_VERSION\|_ANNOUNCEMENT_VERSION\|DOMAIN_\|canonical_bytes"` ne renvoie qu'une mention PROSE de `DOMAIN_SHARD_PLAN_V1` dans un doc-comment (la description du seam d'ingest), aucun changement de constante. DTO = `#[derive(Serialize)]` non-signe, ne touche ni `canonical_bytes` ni `DOMAIN_*`. SHARD_PLAN/RUN_PROOF_FORMAT_VERSION restent =1. |
| **0 nouvelle dep** | TENU | `git diff --stat` sur Cargo.toml/Cargo.lock/package.json/package-lock = VIDE. `hex` deja workspace dep (Cargo.toml:66, 73 usages). axum/serde/serde_json/zod/react-query tous presents. |
| **Auth reutilisee** | TENU | Route dans `authed_routes` (http.rs:310-313), `.layer(auth_required)` http.rs:506. 0 code d'auth ecrit. |
| **DaemonHttpState intouche** | TENU | Aucun champ `shard` ajoute (grep struct = header seul, pas de champ). Aucun registre invente — `live_shard_session` → `None` en dur. |
| **Stub 200-not-404** | TENU | `shard_session` renvoie `(StatusCode::OK, Json(...))` ; `shard_session_response` → toujours `{found:false, session:null}`. Precedent `seed_count` (http.rs:2511 = `StatusCode::OK` + agregat). |
| **Whitelist agregat** | TENU | `ShardSessionView` = `{session_id, member_count}` SEULEMENT, jamais pubkey (cf. section securite). |
| **Day-0 D1-D5 intacts** | TENU | Aucun fichier Day-0 (consent.rs, ComputeGroup, ALPN) modifie. D5 (admission != confidentialite) respecte par whitelist + UX. Scope cut #8 (pas de mode public) non viole : projection read-only loopback d'un groupe deja prive. |
| **Zod envelope strict + row tolerante** | TENU | `ShardSessionStatusResponseSchema.strict()` sur `{found, session}` ; `session.nullable()` (PAS `.optional()`) ; `ShardSessionViewSchema` SANS `.strict()`. Teste des deux cotes (envelope-reject + row-tolerate). |

## Tests (delta + couverture)

- **Suites deja VERTES (annonce session, NON re-runnees — mandat review)** :
  - Rust Windows : fmt + clippy OK, nextest **1949/1949** (+2 = les 2 tests daemon), doctest, release OK.
  - Frontend : coverage 87.27/79.01/86.02/88.59 (>= seuils 85/85/78/85), build, size
    (vendor-ui 263.13<270 KB, css 129.02<130 KB), scan-en-strings « src/ is French-only, clean ».
  - E2E hermetique : 41 passed / 1 skipped (@shard) ; compute-shard.spec.ts:27 + :46
    VERTS (BLOQUANT).
- **Tests net-new Phase J (lus, couverture semantique verifiee adversarialement)** :
  - Rust (2) : `shard_session_response_pins_empty_envelope` (200 + `{found:false,
    session:null}` + `json.len()==2`) ; `shard_session_projection_hides_member_identities`
    (whitelist : member_count==2, 0 pubkey/identite, view len()==2). Signatures
    `ShardedSessionManifest::new` (8 args) + `ShardAssignment{8 champs}` conformes a
    shard_plan.rs.
  - Vitest getShardSession (5) : empty-state + URL exacte ; URL-encode (a%20b%2Fc) ;
    found + member_count ; rejet cle ENVELOPE inconnue (`.rejects.toThrow` valide car
    callDaemon throw ApiProtocolError, daemon.ts:290-292) ; tolerance champ additif ROW.
  - Vitest panneau (4) : etat-vide + 2 intentions FR byte-exact + absence-jargon ;
    not-found ; found ; no-coordinator.
  - E2E (2 hermetiques non-tagues + 1 @shard env-gated) : gating PLAN-ADAPT conforme —
    hermetique SANS tag survit `--grep-invert @compute` (package.json:14, BLOQUANT) ;
    @shard via `test.skip(!SBFB_E2E_SHARD || session vide)`, PAS grep-invert (miroir
    compute-tester.spec.ts).
- **Delta reconcilie (a porter au commit body)** : Rust +2 (vs §15 +0), Vitest +9
  (vs §15 +3). Coherent avec 1947→1949 Rust + +9 Vitest annonces.
- **Pas de test zombie legacy-decode** (politique pre-launch respectee). Tous les noms
  cites grep-resolvent a de vraies fn.

## Codex reconciliation

Gate Codex GPT 5.5 (CLI externe `codex exec`, output brut non-reecrit dans
`sprint77_phase_j_codex_review.md`) — **3 rounds, verdict final CLEAN (0 P0/P1/P2/P3)**.
Promotion PASS-PENDING → **PASS**.

- **Round 1** : 5/7 deliverables CONFIRMED ; 2 PARTIAL + 3 P3 (renforcements test/doc-honnetete) :
  (a) test enveloppe — `json["session"].is_null()` passe AUSSI pour une cle absente sous
  serde_json indexing → ne prouvait pas la presence physique de `session` ;
  (b) test panneau « jamais une identite » — pas d'assertion negative FE ;
  (c) commentaire stub data-plane « Phase B » incoherent avec le label Phase K.
- **Corrections in-phase (3, AVANT round final)** : (a) `assert!(obj.contains_key("session"))`
  ajoute (http.rs:5249) ; (b) sentinelle `worker_pubkey`/`initiator` injectee dans le mock +
  `queryByText(LEAK).not.toBeInTheDocument()` (ShardSessionPanel.test.tsx:137-152) ; (c) reword
  commentaire stub (http.rs:2138-2143). 0 logique prod touchee — uniquement test + doc.
- **Round 2** : 6/7 CONFIRMED, 1 P3 residuel (le mot « Phase B » du commentaire reword).
- **Round 3** : **7/7 CONFIRMED, Final Gaps P0/P1/P2/P3 = none.** Amendement CONFIRMED comme
  vrai PLAN-ADAPT (pas un scope-cut paresseux). Invariants (0 bump, 0 dep, DaemonHttpState
  intouche, D5 whitelist) + delta tests (Rust +2 / Vitest +9) reconcilies par lecture.
- **Re-verif dual-platform apres chaque correction** : Windows fmt+nextest shard 2/2 +
  Docker canonique fmt+nextest shard 2/2 + Vitest 54/54 (2 fichiers) — tous verts.

Les findings P3 non bloquants restants du review Workflow (F5 doc THREAT_MODEL §16 « Phase J »→
« Phase K » + SI-9 carry ; F6 emplacement components/ vs pages/) sont routes au wrap-up
Phase K / carry, documentes dans le commit body.

---

```json
{
  "verdict": "PASS-PENDING",
  "p0": 0,
  "p1": 0,
  "p2_p3_for_body": [
    "P2 F2 (in-phase AVANT Codex): aligner doc-comments route http.rs:307 + handler http.rs:2174 sur le DTO 2-champs reel (pipeline status/verification level = Phase K additif)",
    "P3 F4 (in-phase AVANT Codex): corriger commentaire test http.rs:5274 « four whitelisted fields » -> « two »",
    "P2 F3 (commit body): reconciliation delta tests HONNETE Rust +2 (vs plan +0) / Vitest +9 (vs plan +3), part Vitest non pre-flaggee, regle anti-faux-vert S76",
    "Process: enoncer l'amendement (DTO Phase J = {session_id, member_count} ; status/level = Phase K additif 0-bump, justification dead_code) dans le commit body",
    "Doc-defer trace: THREAT_MODEL §16 / PATTERNS = Phase K (route stub 0-serve in-vivo), signaler pour eviter carry oublie",
    "P3 F5 (wrap-up): rafraichir 3 mentions « Phase J » data-plane -> « Phase K » THREAT_MODEL §16 (l.918/1228/1376) + garder SI-9 carry ouvert",
    "P3 F6 (optionnel): deplacer ShardSessionPanel.tsx components/ -> pages/ (convention P12)",
    "P3 F7/F8 (optionnel): assertion negative test panneau + test isError SessionStatus"
  ],
  "amendment_justified": true,
  "one_line": "PASS-PENDING : 0 P0/P1, whitelist identite verrouillee (DTO 2-champs, fuite pubkey impossible), amendement retrait status/level JUSTIFIE a la source (dead_code), stub 200 conforme seed_count, 0 bump wire / 0 dep / auth reutilisee ; 4 doc-honnetete (1 P2 + 3 P3) + reconciliation delta a corriger avant Codex."
}
```
