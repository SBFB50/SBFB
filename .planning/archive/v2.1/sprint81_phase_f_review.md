# Sprint 81 Phase F — Review (Workflow ultracode + agent de synthèse)

> Phase F « migration on-disk redb 2→4 validée sur COPIE » (nom canonique,
> regex README §4 `Phase [A-Z]+[0-9]?` ; précédents A2/A3/A4/E2/E3 réels).
> Contrat = `sprint81_phase_f_preflight.md` (verdict **PLAN-ADAPT**, LE
> RÉFÉRENTIEL) : la prémisse Phase D « `blobs.db` redb-v2 illisible » est
> REFUTÉE par les octets (les 2 fichiers redb du store VPS réel sont
> `FILE_FORMAT_VERSION3`) ; `blobs.db` ouvre sous iroh-blobs 0.103 SANS
> migration (0 wipe, 0 perte de pins M18) ; `docs.redb` migre automatiquement à
> l'ouverture (feature défaut `redb-v2-migration`, `TableTypeMismatch` sur les
> tables tuples, temp+swap NON in-place, one-way, `.backup-redb-v2-tuples`
> conservé). Le SEUL résidu de durabilité = fenêtre de crash
> rename↔persist (`migrate_redb_v2_tuples.rs:166↔167`) → store vide au reboot →
> recreate SILENCIEUX non capté par le fail-loud A2 ; la garde optionnelle
> root-cause du préflight §4(e).5 a été ADOPTÉE. Périmètre diff = 4 trackés
> (`Cargo.lock` +3, `nexus-core-rs/Cargo.toml` +16, `dispatch_loop.rs` +5/−1,
> `runtime.rs` +215/−28) + 3 untracked (`sprint81_phase_f_preflight.md`,
> `sprint81_t2_f_store_migration.json`, `tests/store_migration.rs`). **Note :
> contrairement à E3, le préflight F EST committé AVEC la phase** (untracked,
> à stager). Arbre SALE, tip `e05338f` (Phase E3). Synthèse de **7 dimensions**
> (D1 diff / D2 tests / D3 grounding / D4 sécurité / D5 scope / D6 patterns /
> D7 suites) + vérifications adversariales, **toutes re-vérifiées à la source
> sur disque** par cette synthèse (fichier:ligne cité par claim).

## Verdict: PASS

> Promotion post-Codex (2026-07-06, même session) : gate Codex GPT 5.5 jouée
> round 1 = **10/10 CONFIRMÉ, 0 GAP, 0 PARTIEL** (artefact brut
> `sprint81_phase_f_codex_review.md` — Codex a re-exécuté lui-même les tests
> store_migration en vérification). Aucune correction requise, aucune boucle.
> Le verdict initial de synthèse (PASS-PENDING) est promu PASS.

## Codex reconciliation

Rapport Codex lu intégralement : 10 livrables vérifiés indépendamment sur le
diff working-tree APRÈS application des dispositions (§4bis) — la garde aux
2 boundaries, le threading du param, les 5 tests, les dev-deps 0-delta-lock,
l'artefact T2, l'invariant 0 bump wire. 0 GAP, 0 PARTIEL → aucune
correction. Suites re-jouées post-dispositions AVANT le run Codex : Win
nextest 2056/2056 0-skip + fmt/clippy ; Docker sbfb-ci fmt/clippy/doctests
verts + nextest **2060/2060 COMPLET 0 skip** (2e run complet vert d'affilée,
classe env passée entière). Le fichier Codex est l'output brut de
`codex exec -o`, non réécrit.

> **Diff Phase F substantivement CONFORME au contrat préflight PLAN-ADAPT,
> re-vérifié à la source.** **0 P0. 0 P1.** 2 P2 non-bloquants (D1-1 CONFIRMED
> = garde reste armée après migration réussie tant que le backup persiste ;
> D2-F1 arbitré PARTIAL→P3 = branche prod recreate non testée, mécanisme
> d'échappement RÉFUTÉ). Le reste = P3 (couverture best-effort, doc-complétude,
> tripwire de dette). Aucun finding n'atteint une sévérité disposable-in-phase
> obligatoire ; toutes les dispositions sont ACCEPTER-plus-doc, tripwire carry,
> ou test optionnel. Les invariants non-négociables sont TOUS tenus et
> re-vérifiés sur disque : **0 bump wire** (F = format de FICHIER, `git diff`
> = 0 constante wire touchée), **0 dep runtime neuve** (dev-deps only, `redb`
> 3.1.3 + 4.1.0 déjà lockés transitivement, +3 arêtes / 0 nouveau `[[package]]`,
> JAMAIS `redb 2.6.3`), **iroh strictement seul**, verrous S74/S75 intacts,
> **duress orthogonal** (la garde lit `iroh_data_dir`+`backup.exists()`, jamais
> `identity_mode`), **règle no-boot store réel RESPECTÉE** (tests =
> `Store::persistent`/`FsStore::load` ISOLÉS sur copies, jamais `create_node`
> sur store réel), **`sbfb-ideas` jamais `sbfb-ides`**, aucune frontière §6.12
> neuve, aucun test nommé `convergence_*`.
>
> **PASS-PENDING = review OK, Codex PAS encore joué (jamais un verdict final
> committable).** Séquence : réconcilier les suites au body → documenter les
> carries G/K au body → gate Codex → réconciliation → promotion `## Verdict:
> PASS` → commit `feat(core)`/`feat(daemon)`.

## 1. Périmètre et staging

`git diff --stat` + `git status --short` (hors warnings CRLF) = **EXACTEMENT**
le périmètre déclaré, 0 fichier parasite, rien oublié :

- `M Cargo.lock` (+3/−0) — 3 arêtes dev-dep sur le bloc `nexus-core-rs`
  (`redb 3.1.3`, `redb 4.1.0`, `rusqlite`), 0 nouveau `[[package]]`.
- `M crates/nexus-core-rs/Cargo.toml` (+16) — `[dev-dependencies]` `redb = "4.1"`
  + `redb_v3 = { package = "redb", version = "3.1" }` + `rusqlite = { workspace
  = true }` + doc-comment bannissant explicitement `redb = "2.6"` (`:165-166`).
- `M crates/nexus-shell-daemon/src/dispatch_loop.rs` (+5/−1) — 4 call-sites
  harness/test recevant le nouvel arg `None` (`:802`, `:814`, `:874`, `:881`).
- `M crates/nexus-shell-daemon/src/runtime.rs` (+215/−28) — helper
  `docs_migration_backup_path` (`:2579`) + garde
  `refuse_recreate_on_interrupted_migration` (`:2590-2607`) + param
  `iroh_data_dir: Option<&Path>` sur les 2 boot fns + 2 tests boundary
  (`:4462`, `:4511`).
- `?? .planning/active/sprint81_phase_f_preflight.md` — contrat (committé AVEC
  la phase, ne se review pas lui-même ; à stager).
- `?? .planning/active/sprint81_t2_f_store_migration.json` — artefact T2 palier
  empirique PASS (à stager).
- `?? crates/nexus-core-rs/tests/store_migration.rs` — 3 tests intégration
  (`:100`, `:232`, `:311`) (à stager).

**Placement de garde vérifié sur disque** : la garde vit dans le bras `None` de
`match opened` UNIQUEMENT (storage `runtime.rs:2677`, feed `:2802`), AVANT le
`warn!`+`create_doc`. Le chemin nominal `Some(doc)` (`:2660`/`:2785`) et la
première-création M8-absente (bras extérieur `None`, `:2698`/`:2820`) ne
l'atteignent JAMAIS. Call-sites PROD = `Some(iroh_data_dir.as_path())`
(`:702`/`:749`). Helper = `iroh_data_dir.join("docs.redb.backup-redb-v2-tuples")`
(`:2580`), cohérent avec l'output upstream.

## 2. Vérification trois blocs (suites §7.4) — constatées, auditées en cohérence

Suites fournies au contexte, re-vérifiées en cohérence arithmétique (D7 RUN les
5 en cible : 5/5 PASS, dont le gate empirique `real_vps` réellement exécuté,
tarball présent localement) :

- **Rust Win** : fmt 0 ; clippy `--all-targets -D warnings` 0 ; nextest
  workspace **2056/2056 0-skip** (baseline E3 2051 → **+5 EXACT** : 2 gardes
  boundary `runtime.rs` + 3 tests `store_migration.rs`) ; doctests + release OK.
- **Docker sbfb-ci** rust:1.94 : fmt/clippy clean ; nextest **2060/2060
  0-skip COMPLET 0 fail** (2055 → **+5 miroir**, gate empirique inclus — la
  classe env Docker-on-Windows est passée entière ce run) ; doctests OK. Écart
  standing Docker−Win = 4 = les 4 `#[cfg(unix)]` (identique à E3). NOTE : un 1er
  run Docker avait tourné sur l'état PRE-rustfmt (invalide, rejoué).
- **Front (web)** : unit 411 (5 fichiers flaky de charge requalifiés solo
  58/58 — AddAnchorDialog/GpuConsentDialog/ShardSessionPanel/BrowsedProject/
  Deploy, classe `vitest_env_variance`) + coverage 79.01/86.02/88.59 ≥ seuils +
  build + size 6/6 + scan clean. operator 201/201 + 6 gates + budgets.
  **0 fichier `web/`/`tools/` au diff** → insensible par construction.

**Delta tests** : **+5 Rust net**, cohérent (2051→2056 Win / 2055→2060 Docker),
0 test supprimé (`git diff | grep '^-.*fn '` sur les `#[test]` = VIDE), aucun
zombie legacy-decode. **Le gate empirique `real_vps_store_copy_migrates_and_
survives` est env-gaté par EXISTENCE du tarball gitignoré** (`store_migration.rs:
322`/`:338`) : en CI (clone distant) → early-return vert AVANT tout assert
(compte comme test exécuté, jamais nextest-skipped) → +5 tiennent en dev ET CI.

## 3. Findings retenus par dimension (evidence re-vérifiée à la source)

| id | dim | sév | claim (re-vérifié) | evidence disque | disposition |
|---|---|---|---|---|---|
| **D1-1** | D1 | **P2** | La garde reste ARMÉE après une migration RÉUSSIE : son prédicat réel = `Some(dir) && backup.exists()`, pas « migration interrompue ». Le backup est CONSERVÉ au succès (upstream rename PUIS persist, 0 delete) et rien ne le nettoie ; donc tout replica-absent bénin ULTÉRIEUR (reset partiel, coordinator.db porté d'un autre data-dir) + backup lingérant → la garde REFUSE le recreate que l'A2 avait intentionnellement conservé. Le doc-comment `:2588-2589` sur-décrit la cause. | garde `runtime.rs:2594-2604` fire sur `backup.exists()` sans distinguer la cause ; backup KEPT `:2571` (doc) + upstream `migrate_redb_v2_tuples.rs:165-167` sans delete ; `IROH_SELFHOST_OPS.md` = **0 occurrence `backup`** (gap runbook réel) | **ACCEPTER** sous doctrine fail-loud (refus LOUD, diagnosticable, recouvrable ; C4/C5 = ancre solo). (a) ligne runbook : supprimer le sibling APRÈS vérif d'une migration réussie pour ré-armer l'A2 (carry G, converge avec T-STORE-FIXTURE-LEAK) ; (b) préciser le doc-comment (précondition réelle = « backup existe ET replica absent »). Non bloquant. |
| **D2-F1** | D2 | **P2→P3** | La branche PROD recreate (`Some` + backup ABSENT → fall-through `Ok` → recreate procède) n'est exercée par AUCUN test : les 2 boundary testent `Some`+backup-présent (`Err`), les recreate-loud pré-existants passent `None`. **Structure CONFIRMED** ; mais le mécanisme d'échappement du finding (`.exists()` inversé) est **RÉFUTÉ** (un `if !backup.exists()` casse les 2 boundary → `expect_err` panique). L'échappement réel = mutation « `Err` inconditionnel / drop du `.exists()` », différente. | branches `runtime.rs:2594-2606` ; boundary `Some` `:4484`/`:4533` ; recreate-loud `None` `:4400`/`:4440` ; prod `Some` `:702`/`:749` | Arbitré **P3** : gap de couverture réel mais illustration corrigée, et la branche (c) ne diffère de la branche `None` testée que par le fall-through `Ok` de la garde. Optionnel : 1 assertion `Some(tempdir vide)` épinglant que la garde laisse passer le recreate en config prod. |
| **D2-F2 / D4-1** | D2/D4 | P3 | Bras première-création (`None` extérieur) NON gardé — CORRECT (le crash-window préserve la ligne M8 dans `coordinator.db`, fichier SQLite séparé → atterrit toujours sur `Some(row)`→open `None`, là où la garde est posée) mais sans rationale inline sur le bras. | `runtime.rs:2698`/`:2820` (create sans garde) ; garde `:2677`/`:2802` | Optionnel : 1 ligne de commentaire sur le bras `None` (« pas de garde : un first-boot légitime n'a pas de ligne M8 à orpheliner »). Non bloquant. |
| **D2-F3** | D2 | P3 | Deux assertions du gate empirique VACUANTES si collection vide : boucle re-parse BlobTicket `:497-505` (skip si `anchors` vide, `unwrap_or_default` `:496`) ; keep_online/invites/seed_invite `:433-438` (count+`eprintln` sans assert de row-count). La survie forte (keep_online=1, invites=5) ne vit que dans le T2 json. La seule assertion FORTE M8 (`:442-445` contains sbfb-ideas && sbfb-feed) est load-bearing. | `store_migration.rs:433-438`, `:496-505`, `:442-445` | ACCEPTER (gate best-effort, run réel non-vide prouvé T2) ou renforcer : `assert !list.is_empty()` + `keep_online >= 1` pour aligner sur le T2 json. |
| **D4-2** | D4 | P3 | Faux-positif de la garde = MÊME phénomène que D1-1 (backup persiste après succès → désactive l'A2 self-heal pour tout replica-absent légitime ultérieur). Impact borné : échec fail-loud avec remède clair, donnée byte-équivalente restaurable, C4/C5 ancre solo. Directionnellement ALIGNÉ avec la doctrine A2 (refus bruyant > recreate silencieux). | `runtime.rs:2595-2597` (prédicat `backup.exists()`) ; doc `:2571` (KEPT) ; scénarios A2 `:2635-2647` | ACCEPTER en P3 (consolidé avec D1-1 ; le message pourrait mentionner « supprimer le sibling si reset volontaire »). |
| **F-D5-01** | D5 | P3 | Littéral suffixe-backup DUPLIQUÉ sur 2 crates, miroir manuel d'une string OWNED par upstream, sans tripwire dédié : `runtime.rs:2580` (garde daemon) et `store_migration.rs:76` `MIGRATION_BACKUP_SUFFIX` (crate test) sont 2 littéraux INDÉPENDANTS. Si upstream renomme, la garde chercherait un sibling qui n'est plus créé → ré-ouvre la perte silencieuse. Couverture INDIRECTE seule : le test hermétique valide la constante du TEST (`:162-166`), pas le littéral du daemon (crate différent). | `runtime.rs:2580` vs `store_migration.rs:76`/`:78-82`/`:162-166` | **Carry K** : tripwire assertant que le suffixe produit par la migration upstream == le littéral du daemon (OU factoriser via constante partagée ré-exportée), analogue au tripwire « Replica not found » déjà routé §8 K. Ne bloque pas le commit F. |
| **D7-1..7** | D7 | P3 | Cohérence arithmétique EXACTE (Win 2051→2056 +5 ; Docker 2055→2060 +5 ; écart +4 `#[cfg(unix)]` conservé), baseline E3 retrouvée indépendamment (`sprint81_phase_e3_review.md:192-193`), 5 tests dans le scope nextest par défaut (0 `#[ignore]`/`#[cfg]`, aucun ne matche le groupe `two-node-convergence` `.config/nextest.toml`), env-gate par existence tarball → CI skip-green, 0 fichier web au diff, Cargo.lock +3 arêtes 0 version neuve jamais redb 2.6.3. | RUN cible 5/5 PASS (dont `real_vps` 0.314s) ; `git diff Cargo.lock` = +`redb 3.1.3`/+`redb 4.1.0`/+`rusqlite` | **RAS** — 7 confirmations, aucun ajustement requis. |

## 4. Arbitrages adversariaux (réconciliation de synthèse)

Les dimensions D1 et D2 portaient un bloc adversarial explicite ; D3/D4/D5/D6/D7
portaient `adversarial: null` (charge adversariale absorbée en amont par le
préflight PLAN-ADAPT : 5 scans + 5 vérifications, 1 REFUTED matériel + 7 PARTIAL
requalifiés, cf. préflight §3). Re-vérification indépendante sur disque :

- **D1-1 — CONFIRMED (non réfuté)**. La garde `runtime.rs:2590-2607` fire dès
  `iroh_data_dir=Some` ET `backup.exists()`, quelle que soit la CAUSE de
  l'absence de replica. Backup jamais nettoyé (grep repo-wide `remove_file`
  négatif dans `runtime.rs` ; `IROH_SELFHOST_OPS.md` = 0 mention `backup`). Le
  claim causal sur-large est à `:2584-2589` (« signature of an interrupted
  migration ») — vrai seulement pour `opened==Some`. Corroboration interne non
  citée par D1 : le test boundary (`:4462`, jumeau feed `:4511`) forge une ligne
  M8 stale `[0xAB;32]` + un fichier `b"backup"` écrit à la main (`:4476`) sur un
  `create_node` éphémère frais = **le cas faux-positif du finding, PAS une vraie
  migration interrompue** → le test entérine la condition sur-large. **RETENU
  P2, ACCEPTER-plus-doc.**
- **D2-F1 — PARTIAL**. Structure (branche (c) prod non testée) CONFIRMED sur
  disque ; mécanisme d'échappement `.exists()` inversé **RÉFUTÉ** (caught par
  les 2 boundary qui écrivent un backup présent et attendent `Err`).
  **Arbitré P3** : gap réel, illustration corrigée, disposition (assertion
  `Some`+dir-vide) valide.
- **« 0 bump wire » — CONFIRMÉ**. `git diff` = 0 `_VERSION`/`FEED_FORMAT`/
  `ANNOUNCEMENT`/`DOMAIN_*` ; la garde ajoute un PARAMÈTRE de fn, ne touche
  aucune constante wire. F = format de fichier redb.
- **« 0 dep runtime neuve » — CONFIRMÉ**. `Cargo.lock` `redb 3.1.3` (`:7031`) +
  `redb 4.1.0` (`:7040`) PRÉ-existent comme `[[package]]` (transitifs via
  `iroh-docs` feature `redb-v2-migration`) ; le diff n'ajoute que 3 ARÊTES sur
  le bloc `nexus-core-rs`. `grep 'version = "2.6.3"'` redb = **0**. Dev-deps
  strictement sous `[dev-dependencies]`.
- **« duress orthogonal » — CONFIRMÉ**. Signature de la garde =
  `(iroh_data_dir: Option<&Path>, what: &str)` (`:2590-2593`), ne lit jamais
  `identity_mode` ; le chokepoint duress `sync_set_entry_in_duress` (`:2711`/
  `:2832`) est INCHANGÉ et vit APRÈS la garde.
- **« no-boot store réel » — CONFIRMÉ**. Les 5 tests utilisent
  `Store::persistent`/`FsStore::load`/`create_node` éphémère sur tempdirs et
  copies fraîches ré-extraites, jamais un `create_node` complet sur le store
  réel (le gate `real_vps` ouvre `docs.redb`/`blobs.db` ISOLÉS sur une copie
  tar hors-repo).

Aucun finding n'a survécu la re-vérification à une sévérité ≥ P1. Aucun ne
requiert un fix-in-phase bloquant.

## 4bis. Dimensions dégradées D3 (grounding) + D6 (patterns) — constat honnête

Les rapports **D3 (grounding)** et **D6 (patterns)** sont revenus en **STUBS
dégradés** (D3 `resume:"test"`, finding unique `titre:"t"`/`evidence:"e"` ;
D6 `verdict_local:"CONCERN"` avec `findings: []`). Aucune analyse substantive
livrée par ces deux agents. À la différence d'E3 (qui avait REJOUÉ D6/D7 en
agents fallback), ces stubs n'ont pas été rejoués en Agent ; la synthèse a
comblé le trou par **re-vérification directe à la source** :

- **Grounding (D3)** : chaque claim des 5 autres dimensions a été re-cité
  fichier:ligne et vérifié sur disque par cette synthèse (garde `:2590-2607`,
  call-sites `:702`/`:749`/`dispatch_loop.rs`, boundary `:4462`/`:4511`, tests
  `store_migration.rs` intégral, `Cargo.toml`/`Cargo.lock`, T2 json). Le
  grounding est de-facto couvert.
- **Patterns (D6)** : le concern PATTERNS le plus matériel de F — le
  magic-string suffixe dupliqué sans constante partagée (§6.9 named-constants) —
  est capté par F-D5-01 + le manque adversarial D2. Spot-check synthèse :
  identifiants anglais, 0 emoji, `sbfb-ideas` jamais `sbfb-ides` (`:694`,
  `:4472`/`:4481`), noms de tests descriptifs, aucun nouveau magic-number
  au-delà du suffixe déjà tracé (bytes fixtures `[0xAB;32]`/`[0xCD;32]` =
  convention pré-existante). Substance couverte.

**Impact sur le verdict : NUL** — surface F = Rust-only, 4 fichiers code, format
de fichier, indépendamment re-grounded. **Mais dette process honnête** : 2/7
dimensions n'ont pas produit d'artefact valide ; à re-jouer en fallback Agent si
le main thread veut une couverture D3/D6 native avant Codex (non bloquant).

**REJOUÉES post-synthèse (fallback Agent §7.1, même protocole qu'E3) :**

- **D3 grounding (rejeu) — PASS, 2 P3.** Les 6 contraintes load-bearing §5 du
  préflight vérifiées VERTES sur disque une à une (default-features intacts,
  ré-extraction fraîche par run, aucun `create_node` sur store réel, garde =
  extension A2 sans rouvrir le fail-loud, `sbfb-ideas`+`sbfb-feed`, garde aux
  bons repères — 1 définition + 2 call-sites recreate exactement) ; livrables
  §4(e) 1-6 tous livrés ou routés, dont l'adaptation CORRECTE du livrable 4
  (proxy `namespaces-2` au lieu du littéral `open_doc` qui exigerait un Docs
  spawné, interdit par la contrainte 3) ; delta +5 = borne haute cohérente.
  D3-1 (P3) : anti-vacuité records/doc_ticket au gate empirique — couverture
  records actée comme ancre hermétique (documentée au body). D3-2 (P3) :
  message de la garde à enrichir pour le backup stale — **APPLIQUÉ** (remède
  « delete the stale backup to re-arm the self-heal » + renvoi runbook).
- **D6 patterns (rejeu) — CONCERN → résolu, 1 P2 + 3 P3.** **D6-1 (P2,
  CONFIRMÉ, APPLIQUÉ)** : le doc-comment du helper avait été inséré en
  CONTINUATION du bloc doc existant de `boot_storage_namespace` (le bloc
  fusionné s'attachait au helper, la fn de boot perdait sa doc rustdoc) —
  corrigé : les 2 fns Phase F déplacées AVANT le bloc, chaque doc réattaché à
  sa fn. D6-2 (P3) : « env-gated » vs gate-fichier — vocabulaire documenté au
  body, mécanique de skip conforme au pattern repo. D6-3 (P3, APPLIQUÉ) :
  cross-ref réciproque du suffixe backup ajoutée au doc du helper. D6-4 (P3,
  APPLIQUÉ) : provenance upstream `:99` du préfixe temp citée dans le test.
  Conformités : doctrine A2 (marqueur « refusing to silently recreate »
  réutilisé + marqueur discriminant « interrupted redb migration »), gabarit
  T2 9 clés, 0 emoji, langue OK. PATTERNS §P73 candidate (« one-way store
  migration ») → lot docs Phase K.

**Dispositions synthèse §5 appliquées en sus** : runbook NEUF
`docs/release/STORE_MIGRATION_OPS.md` (rollback one-way + nettoyage du backup
post-vérification qui ré-arme le self-heal — la ligne demandée par D1-1, dans
un doc dédié socle de Phase H plutôt que greffée au runbook zéro-n0) ;
branche prod de la garde épinglée (les 2 tests recreate existants passent
désormais `Some(tempdir vide)` — backup absent ⇒ self-heal procède) ;
anti-vacuité du gate empirique (assert `keep_online >= 1` + `!anchors.is_empty()`
épinglés sur CE tarball versionné) ; doc-comment du test boundary précisant
que la garde ne distingue pas « interrompue » de « réussie-backup-restant ».
Re-runs après dispositions : fmt + clippy clean, 15/15 ciblés, **nextest
workspace Win 2056/2056 0-skip re-vert**, Docker complet relancé (consigné au
body avant commit).

## 5. Dispositions pour le main thread

**Aucun fix obligatoire (0 P0/P1, 2 P2 non-bloquants).** Toutes optionnelles,
carry, ou mécaniques :

1. **ACCEPTER D1-1/D4-2 (P2/P3) sous fail-loud** — documenter au body + ajouter
   au runbook `IROH_SELFHOST_OPS.md` (0 mention `backup` aujourd'hui) une ligne :
   « après vérification d'une migration réussie, supprimer le sibling
   `docs.redb.backup-redb-v2-tuples` pour ré-armer l'A2 self-heal » (carry G,
   converge avec T-STORE-FIXTURE-LEAK — le backup = ancien `NamespaceSecret`
   write-cap). Optionnel : préciser le doc-comment `runtime.rs:2588-2589`
   (précondition réelle = « backup existe ET replica absent »).
2. **Carry K (tripwire suffixe backup)** — assertion que le suffixe produit par
   la migration upstream == le littéral daemon `runtime.rs:2580` (OU constante
   partagée ré-exportée entre `runtime.rs:2580` et `store_migration.rs:76`),
   + tripwire fidélité des defs de tables répliquées (`store_migration.rs:52-72`
   vs upstream) — regroupés avec le tripwire « Replica not found » déjà routé.
3. **Optionnels (P3, non bloquants)** : (a) 1 assertion recreate `Some(tempdir
   vide)` épinglant la branche prod (c) [D2-F1] ; (b) commentaire inline sur le
   bras first-boot `None` `:2698`/`:2820` [D2-F2/D4-1] ; (c) `assert
   !list.is_empty()` + `keep_online >= 1` dans le gate [D2-F3] ; (d) renommer le
   test boundary en `..._when_migration_backup_sibling_present` OU documenter
   qu'il ne distingue pas les 2 causes [manque adversarial D1].
4. **Process (non bloquant)** : re-jouer D3/D6 en fallback Agent si couverture
   native souhaitée (§4bis) — sinon acter la re-vérification synthèse.
5. **Stager exactement 7 chemins** : les 4 trackés + les 3 untracked (dont
   `sprint81_phase_f_preflight.md`, committé AVEC la phase F, ET
   `sprint81_t2_f_store_migration.json` ET `tests/store_migration.rs`).
6. **Réconcilier les suites au body** (Win 2056 / Docker 2060 / web+operator) +
   **gate Codex** `codex exec -o` (output brut, jamais réécrit ; critère d'arrêt
   = « CLEAN ou P2/P3 documentés »). Promotion `## Verdict: PASS` après Codex.

## 6. Carries sortants (F → G, K)

Tous **routés par le préflight §8** et re-confirmés ici ; F n'en ouvre aucun
comme dette code neuve non-tracée :

1. **G (THREAT_MODEL / runbook — le sprint réserve les édits THREAT_MODEL à G)** :
   - **T-STORE-MIGRATION-CRASHWINDOW** : la garde ADOPTÉE ferme le silent-loss
     quand le backup a survécu (par construction : rename FIRST) ; tar snapshot
     obligatoire avant toute migration réelle (Win pris, **Mac PENDING**).
     Résidu du faux-positif D1-1/D4-2 (backup lingérant désarme l'A2) → ligne
     runbook (disposition 1).
   - **T-STORE-FIXTURE-LEAK** : la migration CRÉE `docs.redb.backup-redb-v2-
     tuples` (ancien `NamespaceSecret`) + `docs.db.migrate<rand>` non nettoyés,
     non couverts par `*.redb`/`*.db` hors `data/`. Nettoyage runbook converge
     avec la disposition 1.
   - **Correction kickoff C4/C5** : feature `redb-v2-migration` EXISTE + défaut ;
     tripwire « aucun `default-features=false` sur iroh-docs ».
   - **hickory-proto 0.24.4 RUSTSEC-2026-0119** (pré-existant, isolé) — bump en G.
2. **K (dette / tripwire)** :
   - Tripwire suffixe backup daemon==upstream (F-D5-01) + fidélité defs de tables
     répliquées vs upstream (manque adversarial D2) + tripwire « Replica not
     found » (préflight §8 K).
   - Checklist « survivants de l'upgrade » à consigner (`node_key`,
     `coordinator.db`+WAL, `anchors.json`, `default-author`,
     `directory_revision.json`, `subscriptions.json`, `allowlist.sqlite3`).
3. **Snapshot Mac PENDING** (règle bloquante) — porté tant qu'aucun boot Mac n'a
   été snapshotté.

## 7. Residual Risk

- **Gate empirique = 0 signal de régression CI** (manque adversarial D2) :
  `real_vps_store_copy_migrates_and_survives` early-return vert quand le tarball
  gitignoré ou `tar` est absent (`store_migration.rs:322-327`/`:338-343`) =
  toujours le cas CI. Le palier PASS est une observation locale one-shot
  (Windows, 2026-07-06, T2 json), pas un check régression permanent. INTENDU
  (miroir relay-gated `multi_daemon`) et DIVULGUÉ, mais les chemins DIRTY
  (blobs-open recovery + docs repair+migration réels) n'ont pas de protection
  ré-jouable — seuls les 2 anchors hermétiques (`docs_migration` synthétique +
  `blobs_round_trip`) régressent en CI.
- **Fidélité fixture hermétique non épinglée à upstream** : `docs_store_with_
  legacy_tuple_tags_migrates_on_open` forge le store legacy depuis des defs de
  tables RÉPLIQUÉES à la main du module privé upstream (`store_migration.rs:
  38-72`). Un futur bump iroh-docs changeant noms/formes ferait dériver la
  fixture silencieusement (le test resterait vert sur un store auto-forgé qui ne
  correspond plus à ce que redb 2.x écrivait), pendant que le seul test sur le
  layout RÉEL (env-gaté) ne tourne pas en CI → carry K.
- **Faux-positif de la garde** (D1-1/D4-2) : sur un store déjà migré, un backup
  lingérant désarme l'A2 self-heal pour tout replica-absent légitime ultérieur.
  Fail-loud recouvrable (pas de hang, pas de silent-loss), C4/C5 ancre solo,
  aligné doctrine A2. Ligne runbook (disposition 1) le lève.
- **Env session (05-06/07)** : contention cargo Win persiste (crash rustc
  « iroh_blobs rlib » re-run vert) ; ne PAS lancer nextest workspace Win pendant
  Docker/Codex. `docker run` Git Bash = `MSYS_NO_PATHCONV=1` + chemin Windows.

---

**PASS-PENDING** : phase substantivement conforme au contrat préflight
PLAN-ADAPT, minimale et root-cause par conception (garde optionnelle §4(e).5
adoptée), invariants TOUS tenus et re-vérifiés à la source (0 bump wire, 0 dep
runtime neuve jamais redb 2.6.3, iroh seul, verrous S74/S75, duress orthogonal,
no-boot store réel, sbfb-ideas jamais sbfb-ides, 0 frontière §6.12, 0
`convergence_*`). **0 P0, 0 P1.** 2 P2 non-bloquants (D1-1 CONFIRMED
accept-fail-loud ; D2-F1 arbitré P3, échappement réfuté) + P3 (best-effort gate,
doc, tripwire K). D3/D6 = stubs dégradés, comblés par re-vérification source
(impact verdict nul). Carries G/K routés par le préflight. Une fois les suites
réconciliées au body et les carries documentés, le gate Codex peut promouvoir
en `## Verdict: PASS`.
