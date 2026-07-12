# Sprint 81 Phase A3a — Review (Workflow ultracode + synthèse arbitrage)

> Phase A3a (sous-phase du split A3a/A3b décidé au préflight PLAN-ADAPT,
> `sprint81_phase_a3_preflight.md §3`) : phase OBSERVATIONNELLE **0-fix-code** —
> baseline transport LIVE 0.98 committée + ressources rig. Arbre SALE, HEAD `23f3be8`.
> 5 dimensions de review + 5 vérifications adversariales + synthèse main-thread de
> première main (git status/diff + lecture code + état on-disk du rig re-vérifiés moi-même).

## Verdict: PASS

> Review substantivement PROPRE (rendue PASS-PENDING par le Workflow, PROMUE PASS après
> résolution du P1 et réconciliation Codex — cf. `## Codex reconciliation`). **Un seul
> P0/P1 confirmé** — le gate fail-fast Rust Win était ROUGE
> (`consent_get_returns_default_config`) au run capturé, **pollution environnementale**
> (le rig L3 `~/.sbfb/consent.json` posé cette phase) sur un test non-hermétique,
> **0 code changé**. Fichier rig **PARKÉ on-disk** (`consent.json.rig-l3`) puis **preuve
> verte RE-CAPTURÉE** : nextest **2026/2026 passed 0-skip** + doctests **6/6** + fmt/clippy/
> release verts (P1 SOLDÉ, cf. État des suites). Les P2/P3 sont des raffinements de doc
> (routage carries, note du palier restart, schéma JSON) portés au body/JSON, non bloquants.

## Portée du diff (re-vérifiée de première main)

`git status --porcelain` à HEAD `23f3be8` = EXACTEMENT 4 changements, 0 fichier code :

- `M .gitignore` — +4 lignes (règle `*.redb` + doc-comment NamespaceSecret, `l.8-11`).
  `git check-ignore foo.redb` => ignoré ; `*.db` NE matche PAS `.redb` — règle légitime,
  couvre `docs.redb` (capacité d'écriture).
- `?? .planning/active/sprint81_t2_baseline_098.json` — livrable central, verdict-only,
  clé-par-palier, axe différentiel `iroh_baseline:"0.98"`.
- `?? .planning/active/sprint81_a3a_integration_run_098.txt` — run `SBFB_INTEGRATION=1`
  archivé, home absolu scrubbé en `<repo>`.
- `?? .planning/active/sprint81_phase_a3_preflight.md` — artefact G8 historique du préflight
  (frozen ; précède les observations live ; NE PAS réécrire).

Rien d'autre ne fuit : `data/vps-store-098/`, `.b3_*.json/.log`, `scripts/acceptance/rig.local.env`,
`~/.sbfb/consent.json.rig-l3` tous `!!` ignorés / hors-repo. 0-bump CONFIRMÉ (aucune
sérialisation/JCS/`DOMAIN_*`/ALPN/`*_FORMAT_VERSION` touchée). Gate `sprint81_design_review.md`
PRÉSENT (21961 o).

## Restitution par dimension (5 + 5 adversariales)

1. **Vérité des claims JSON vs code/git** : PASS — les DONNÉES du JSON sont fidèles au run
   archivé (blob 400 @`multi_daemon.rs:120`, feed 403 ×4 @`:47/:374`, `is_success()` false
   @`:514/:628`, gossip panic 33s @`:182`). Provenance S65 `ace05b0` du header
   `x-sbfb-feed-internal` CONFIRMÉE (git log -S, hit unique). Blob-serve zip-only depuis S12
   CONFIRMÉ (`blob_serve.rs:103` `ZipArchive::new`→`InvalidZip`→400). Boot-window
   `share_write` AVANT spawn CONFIRMÉ au call-graph. **2 P3 formulation** (voir findings).
2. **Scrub / sécurité des artefacts** : PASS — 0 IP / 0 token / 0 SSH / 0 username / 0 path
   absolu ; seul hit `NamespaceSecret` = mot descriptif ; `NamespaceId(35701c15..)` = id
   PUBLIC tronqué 8-hex (PAS le secret, porté par `docs.redb` gitignoré) ; pseudonymes
   nodeVPS/nodePC/nodeMac ; `last_response`/`${RESP}`/`$WORKER_LOG` absolu DROP confirmés.
   **1 P3 défense-en-profondeur** (règle `rig.local.env` racine — voir findings).
3. **Conformité préflight PLAN-ADAPT + scope** : PASS — les 5 livrables A3a du préflight §3
   couverts + évidenciés ; **0-fix-code STRICT** (`git diff --name-only HEAD` = `.gitignore`
   seul) ; VPS-redeploy au HEAD 0.98 = prérequis légitime (route project-info requise par le
   harness absente du binaire S76), rollback préservé, binaire buildé depuis HEAD committé ;
   honnêteté baseline respectée (le JSON ne surclamé PAS la mort du blocker S77). **2 P2 +
   2 P3** (voir findings).
4. **Qualité / re-jouabilité différentielle (post-bump 1.0.1)** : CONCERN→doc — JSON parse
   VALIDE, schéma évolution fidèle de `sprint80_t2_acceptance.json` (verdict-only + axe
   différentiel). Les 3 « P1 » initiaux ont été DÉGRADÉS par la vérification adversariale
   (un REFUTED, deux → P2/P3). Reste des raffinements de schéma/note. **1 P2 + 3 P3** (voir
   findings).
5. **Process + suites §7.4** : PASS-PENDING — commit shape conforme au préflight §9 ;
   body 9 sections cadré ; **1 P1 = gate Win ROUGE (pollution rig, 0 code)** + **1 P3 clippy/
   doctest non-évidenciés** (sous-note du re-run propre) + **1 P3 carry test-isolation** ;
   les « P2 Docker/web incomplets » sont **REFUTED** (logs complets = verts/2028+2 env-blocked).

## Findings P0-P3 + verdicts adversariaux (arbitrés)

### P1 — Rust Win nextest ROUGE : `consent_get_returns_default_config` (pollution rig L3)
- **Verdict arbitrage : CONFIRMED, P1, MAIS trivialement + opérationnellement corrigeable
  (déjà à moitié appliqué).**
- **Re-vérifié de première main** : `mk_state()` pose `sbfb_home: None` (`http.rs:4429`) ;
  le test (`http.rs:8926-8942`) asserte `body["level"]==1` ; `get_consent`→`load_consent(None)`
  →`consent_path(None)`→`auth::sbfb_home()` (`nexus-shell-daemon-core/src/auth.rs:72-85`, lit
  `$SBFB_HOME` sinon `%USERPROFILE%\.sbfb`). Le rig L3 posé cette phase (`allowed_project_ids:
  [35701c15..]`, `level:3`) faisait lire `level=3 ≠ 1` → FAIL → fail-fast (1030 non-run) →
  `error: test run failed`. **Environnemental, PAS régression** : 0 code changé ; Docker
  propre de CE fail (home conteneur vierge).
- **État on-disk vérifié maintenant** : `~/.sbfb/consent.json` **ABSENT** ; le rig est PARKÉ
  en `consent.json.rig-l3` (horodaté 21:32). **Donc un re-run Win est VERT** (`no live
  consent.json — test will be GREEN`). Le JSON documente déjà ce parking (`:39`).
- **Résidu bloquant** : la preuve verte (nextest **2026 passed 0-skip**) n'est PAS encore
  capturée dans un artefact. Committer contre le run ROUGE violerait README §7.4 +
  `feedback_full_failfast`.

### P2 — Carries du 1er run réel non routés à une phase/owner (Dimension 3)
- **Verdict arbitrage : CONFIRMED, P2, documentation-only.**
- Le 1er run `SBFB_INTEGRATION=1` (`first_real_run_ever:true`) expose 5 test-rot + 1
  product-signal. Le JSON CATÉGORISE (`:28`) mais ne ROUTE pas. **Nuance retenue** : le
  `gossip_exchange` EST déjà rattaché par classe au carry standing S75 SeedAnnounced (`:26`
  + préflight §10-5). Le gap réellement non-routé = **les 5 test-rot `multi_daemon`** (angle
  mort CI depuis S65). Routage légitime dans le **body du commit A3a** (pas encore créé) +
  `sprint81_audit_plan`/`verification`, pas dans le JSON verdict-only. Non bloquant (P2 se
  documente).

### P2 — Différentiel A3b « BLOCK→PASS » live désormais inobservable (Dimensions 3 & 4)
- **Verdict arbitrage : CONFIRMED, P2, à re-scoper dans l'acceptance A3b (pas ici).**
- Le préflight prédit une baseline restart `BLOCK{delivery}` (§3 `:138`) et A3b comme flip
  live `BLOCK→PASS` (§3 `:176`). OR la baseline observée restart-no-remint = **PASS 6s**
  (`json:46`) via le side-effect `share_write` du submit-path (`local_worker.rs:306-310`
  AVANT l'échec de spawn). **Confirmé au code de première main** : `ensure_spawned`
  (`http.rs:3459`, submit-path nudge doc `:162-163`) → court-circuite si child vivant
  (`local_worker.rs:122-124`) sinon `provision()`→`share_write()@307` AVANT `cmd.spawn()@191/227`
  → spawn échoue (binaire absent) → `warn "failed to spawn"@157` → `st.child` reste None →
  chaque submit re-arme. Conséquence : la re-run A3b du palier restart sera **PASS→PASS**, la
  preuve A3b réelle = le test hermétique red→green (préflight §6.1) + la suppression de la
  DÉPENDANCE au side-effect. Le fix se pose dans l'acceptance PROPRE d'A3b (le préflight est
  frozen), pas dans A3a. Non bloquant pour A3a.

### P2 — Incohérence inter-artefacts + précondition sous-enregistrée (Dimension 4)
- **Verdict arbitrage : CONFIRMED, P2 (rétrogradé du P1 initial), documentation-only.**
- Le JSON (`:48`) corrige implicitement le préflight §2.2 (`:72-74`) qui rangeait
  `local_worker.rs:307` sous « au mint d'invite, jamais au boot » — le `share_write` se
  déclenche aussi sur le submit-path via `ensure_spawned`. Le JSON a RAISON (vérifié) mais ne
  signale pas qu'il corrige le préflight, et la précondition load-bearing (« every submit »
  n'est vrai que TANT QUE le binaire worker VPS est absent → child jamais posé → re-provision)
  n'est enregistrée qu'à demi (`json:48` note bien « worker binary absent » ; manque
  l'implication d'idempotence once-vs-every-submit). Résidu = ~2 lignes de note dans le palier
  restart. NB : la citation review « §5.4 :223-226 » est erronée (ces lignes = doc_share/
  start_sync `actor.rs:407`, pas `share_write`) — non matériel.

### P3 (non bloquants, à documenter)
- **D1-1** : « test-rot early-returned green in CI since S65 » conflate le gating d'intégration
  (early-return vert, PRÉCÈDE S65 pour tous les tests rotés — feed créés S62 `cd7c46a`,
  blob S33 `3d3bd96`) et la divergence d'auth (header, `ace05b0` = S65). Substance correcte,
  formulation amalgamante. Reformulation optionnelle de `json:28`.
- **D1-2 / D4-F3** : « re-arming on every submit » honnête mais implicite — préciser « TANT
  QUE le spawn échoue ; un worker spawné avec succès n'arme qu'une fois » (renforce A3b).
- **D2-scrub** : `rig.local.env` à la RACINE non couvert par `.gitignore` (seul
  `scripts/acceptance/rig.local.env` l'est, `l.151`) ; le fichier réel EST au chemin canonique
  et ignoré → 0 fuite actuelle ; gap purement latent. Règle large `rig.local.env` optionnelle.
- **D4-vocab** : palier integration porte `verdict:"BLOCK"` bare (sémantique b3 empruntée)
  + `per_test` emploie `"FAIL{...}"` hors vocab fermé + pas de `schema_version`/`vocabulary`.
  BAS (les champs `passed:4/failed:6` + `per_test` keyé portent le différentiel). Optionnel :
  renommer en token snapshot (`SNAPSHOT{4P/6F}`) + `schema_version` + bloc `vocabulary`.
- **D4-budget** : paliers b3 sans `budget_s`(=GATE_TIMEOUT 30) ni model/prompt de référence.
  Ajout optionnel (surtout palier 2 quorum, byte-identité seed/model/prompt).
- **D4-namespaceid** : `NamespaceId(35701c15..)` brut de journal retenu (id public tronqué,
  load-bearing, non-sensible) — pseudonymisation `<projectDoc>` optionnelle pour cohérence.
- **D5-clippy/doctest** : Win artefact va jusqu'à fmt+nextest ; clippy non-évidencié +
  doctests à 0 (effet du fail-fast amont). **Sous-note du re-run propre P1** : le re-run vert
  doit surfacer `clippy --all-targets -D warnings=0` + un compte doctests non nul.
- **D5-carry test-isolation** : `consent_get_returns_default_config` + 5 voisins Sprint-46
  via `mk_state()` (`sbfb_home:None`) lisent le vrai `~/.sbfb` ; le helper hermétique
  `mk_state_with_sbfb_home(tempdir)` (`http.rs:4362`) existe. Classe §P72 / TEST-ISOLATION-
  SBFB-HOME (fermée S80). **HORS scope A3a (0-fix-code)** — carry déjà routé à A3b DANS le
  JSON (`:39`). Confirmer qu'il atterrit dans `sprint81_audit_plan`/`verification`.
- **D3-keepalive** : le carry « keepalive worker jamais prouvé LIVE » RESTE OUVERT — ni le
  fresh-enroll PASS ni le restart PASS ne font FIRE le keepalive (runs 6-14s sans NeighborDown).
  L'artefact attribue correctement la delivery au side-effect, ne surclamé pas. Router vers un
  run rig NeighborDown (T2 palier 2 / Phase K).

### Findings REFUTED par la vérification adversariale (aucune action)
- **D4 « palier integration différentiel-aveugle »** : REFUTED — `passed:4`/`failed:6` entiers
  + `per_test` keyé rendent un PASS→FAIL et le changement de compte machine-détectables ; le
  fix suggéré « figer passed==4/failed==6 » est DÉJÀ satisfait. Résiduel P3 = wording
  `replay_contract` (`diff verdicts` → « diff per_test+passed+failed pour le palier »).
- **D5 « Docker sbfb-ci INCOMPLET »** : REFUTED — snapshot tronqué lu ; le run a COMPLÉTÉ
  `Summary [113.949s] 2030 tests run: 2028 passed (16 slow), 2 failed, 0 skipped` ; les
  tests `operator_server` SLOW ont tous PASSÉ ; les 2 fails = classe env daemon-spawn/networked
  connue (`start_headless_boots_and_shuts_down_on_signal` + `convergence_incremental_task_
  reaches_remote_replica`), PAS des timeouts operator. Body : reporter « 2028 passed + 2
  env-blocked (classe connue) », jamais « Docker 2030 clean ».
- **D5 « pipeline web INCOMPLET »** : REFUTED — snapshot 7 lignes lu ; log complet = tous les
  étages VERTS (coverage 87.27/79.01/86.02/88.59 > planchers, build 6.68s, size 129.02/130kB,
  scan-en-strings SCAN-OK, vitest 411). 0 fichier web changé → 0 régression.

## Corrections requises / faites

- **AVANT commit (P1, obligatoire)** : re-jouer le bloc Rust Win fail-fast dans l'état PARKÉ
  actuel (aucun `consent.json` vivant) et CAPTURER la preuve verte (nextest **2026 passed
  0-skip** + `clippy --workspace --all-targets -D warnings=0` + doctests count non nul +
  fmt + release). NE PAS committer contre l'artefact ROUGE `bi11o684k`. Fix physique déjà
  appliqué (fichier parké) ; il ne manque QUE l'évidence verte.
- **DÉJÀ FAIT (on-disk)** : `~/.sbfb/consent.json` parké en `consent.json.rig-l3` ; règle
  `.gitignore *.redb` posée ; store VPS sous `data/vps-store-098/` ignoré.
- **À porter au body / JSON (P2/P3, documentation, non bloquant)** : (i) router les 5 test-rot
  `multi_daemon` vers une phase de remise-à-niveau (K/dette) dans le body ; (ii) sharpen la
  note du palier restart (réconcilier préflight §2.2 + once-vs-every-submit + « PASS→PASS
  post-A3b n'est pas une non-régression, preuve = test hermétique ») ; (iii) reporter Docker
  honnêtement (2028 + 2 env-blocked). Optionnels cheap : `schema_version`+`vocabulary`,
  `budget_s`+model/prompt, reformuler `json:28`/`replay_contract`.

## État des suites §7.4 (4 jobs)

- **Rust Win (nextest+fmt+clippy+doctest+release)** : au run capturé = **ROUGE** (995/2026
  passed, 1 fail `consent_get_returns_default_config`, 1030 non-run, fail-fast) — **pollution
  rig, 0 code**. État on-disk = PARKÉ → **re-run attendu VERT 2026 passed 0-skip** (à CAPTURER
  avant commit, P1). clippy/doctest à re-surfacer au re-run propre.
- **Docker sbfb-ci** : **COMPLET** — `2030 tests run: 2028 passed (16 slow), 2 failed, 0
  skipped` ; 2 fails = classe env daemon-spawn/networked connue (tolérée, crate non touché).
  `multi_daemon` PASSE cette fois (réseau hôte sain). ACCEPTABLE.
- **web** : **COMPLET VERT** — coverage 87.27/79.01/86.02/88.59 (> 85/78/85/85), build 6.68s,
  size 129.02/130kB, scan-en-strings SCAN-OK, vitest 411/411. 0 fichier web changé.
- **operator** : **VERT** — vitest 201/201 + built. 0 écart baseline.

## Carries

- **A3b** : (a) re-scoper l'acceptance A3b (preuve = test hermétique red→green mode-restart +
  suppression de la dépendance au side-effect `share_write`, PAS un flip live BLOCK→PASS —
  la baseline restart est déjà PASS via side-effect) ; (b) carry test-isolation `consent_*`
  non-hermétiques → migrer vers `mk_state_with_sbfb_home(tempdir)` (déjà routé JSON `:39`) ;
  (c) reposer `consent.json.rig-l3` → `consent.json` pour la re-run différentielle rig.
- **Phase B/C** : re-calibrer `is_syncing`/`start_sync`/broadcast/accept + le matcher
  « Replica not found » contre iroh-docs 0.101 au bump (le fix A3b en dépend).
- **Phase G** : THREAT_MODEL classe warn-only (carry A2) ; note amplification §15.3 pour tout
  `start_sync` coordinateur peers non-vides ; re-jouer `cargo deny check advisories` post-bump.
- **Phase K / T2** : keepalive worker efficacité WAN (run rig NeighborDown) ; palier 2 quorum
  (prérequis Ollama-Mac DONE ; reste binaire arm64 nexus-worker) ; libellé T1 honnête
  (relay-gated early-return silencieux) ; les 5 test-rot `multi_daemon` (remise-à-niveau CI).
- **Standing (NON A3, ne pas conflater)** : RE-DRIVE-ON-INGEST, SeedAnnounced peer_count:0,
  seeder catalog_len:0, PULL-3 — chemin découverte/seed, pas task-delivery.

## Codex reconciliation

**Joué** (`codex exec` GPT 5.5, output brut `sprint81_phase_a3_codex_review.md`, round 1) :
**6/7 CONFIRMÉ, 0 GAP, 1 PARTIEL**. Le PARTIEL (Livrable 1) est un point de vocabulaire de
schéma — `verdict: "BLOCK"` nu au palier integration + valeurs `FAIL{...}` dans `per_test`
vs le libellé « vocabulaire fermé » du prompt (recoupe le P3 D4-vocab ci-dessus). Corrigé
par ajout d'une `vocabulary_note` au JSON : le verdict de palier suit la paire
status+diagnosis (la forme exacte qu'émet le harness b3 `emit_artifact`), `per_test` reste
au vocabulaire nextest PASS/FAIL{cause} (résultats bruts de suite, pas des verdicts de
palier). Tous les claims code (a)-(e) confirmés par Codex avec évidence fichier:ligne
(feed_sync.rs:596, http.rs:3153, consent.rs:183/397, local_worker.rs:183/191/306,
runtime.rs:643, docs.rs:106). Scrub re-vérifié indépendamment par Codex (0 home-prefix,
0 IPv4, 0 token). Corrections JSON = fichiers planning non-code → suites §7.4 non
invalidées (re-run non requis).

**Résolution du P1 review AVANT Codex** : bloc Rust Win re-capturé VERT dans l'état parké —
nextest **2026/2026 passed 0-skip** (exit 0) + doctests **6/6 passed** + fmt/clippy/release
verts. Le P1 « preuve verte » est soldé.

Séquence review PASS-PENDING → Codex → réconciliation → **PASS** respectée. Commit
`chore(acceptance)` autorisé.
