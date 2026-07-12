# Sprint 81 Phase D — Review (Workflow ultracode + agent de synthèse)

> Phase D « iroh-blobs cascade + redb4 » (`sprint81_plan.md:165-177`,
> supersédée par le préflight PLAN-ADAPT `sprint81_phase_d_preflight.md`
> — la lettre décrivait une « recompilation de la couche blobs sous
> 0.103 + validation d'ouverture du store redb4 » que le bump Phase B
> (`c899d54`) a déjà absorbée [nextest 2038 vert sous blobs 0.103] et
> dont la « validation redb4 » N'EST PAS exécutable en D [ouvrir un vrai
> store = migration one-way docs / hard-fail `UpgradeRequired` blobs →
> routé F]). Le VRAI périmètre code se réduit à **2 items minces** :
> [DOC-ONLY] `blobs.rs:481` « 0.100 »→« 0.103 » (carry B fermé) +
> [TEST] +1 round-trip BlobTicket PUR (contrat `anchors.json`). Arbre
> SALE, HEAD `f70fa5f`, diff NON committé (1 fichier code modifié +
> 1 untracked planning, `git status --short` = 2 lignes). Review menée
> sur le diff COMPLET ligne-par-ligne + grounding vendored/daemon.

## Verdict: PASS

> *(Promu de PASS-PENDING après le gate Codex — cf. `## Codex
> reconciliation` en fin de fichier. Le corps ci-dessous est l'état au
> moment de la review, conservé verbatim.)*
>
> **Diff Phase D substantivement CONFORME au contrat préflight qui
> supersede la lettre.** 0 P0 / 0 P1. Le diff est minimal et exactement
> celui annoncé : `crates/nexus-core-rs/src/blobs.rs` (+48/−1), une
> seule ligne de code prod touchée (un doc-comment de test) + un test
> pur neuf. **0 code fonctionnel, 0 `Cargo.lock`/`deny.toml`, 0 store
> ouvert, 0 constante wire, 0 dep** — la bisectabilité est préservée (la
> recompilation appartient à B). L'item DOC-ONLY est factuel : le blanket
> `impl ContentDiscovery for IntoIterator` reste ordre-d'itération en
> 0.103 (préflight byte-diff `downloader.rs:562-573`, contraste `Shuffled`
> `:585`) ; seul le numéro de version était périmé, et c'était la
> **dernière** mention `0.100` de `blobs.rs` (grep post-fix = vide, `:87`
> déjà correct depuis B). Le test neuf `blob_ticket_string_round_trips_
> under_current_lock` est PUR (aucun nœud, aucun store, aucun dial :
> `EndpointAddr` construit en mémoire, relay `.invalid` RFC 2606, IP
> `192.0.2.7` TEST-NET-1 RFC 5737 — non-routables), son API est
> compile-prouvée et **mirroir de tests existants** (`EndpointId::from_str
> (&hex::encode(KeyPair::generate().public_bytes()))` = `:498` verbatim ;
> `BlobTicket::new(addr, hash, Raw)` = `:433`), et il verrouille un
> contrat RÉEL (`AnchorLocator.ticket` = `iroh_blobs::ticket::BlobTicket`
> string persisté dans `anchors.json`, `iroh_runtime.rs:260-268`,
> re-parsé au boot via `from_str`). Le S2-10 REFUTED du préflight
> (anchors.json = ticket persisté, pas un hash nu) est confirmé au code.
>
> **PASS-PENDING = review OK, Codex PAS ENCORE JOUÉ** (gate bloquante
> review → Codex → commit). **1 P2 + 1 P3**, aucun ne bloque le commit
> SI le body les honore :
> **(P2-1)** le test prouve l'idempotence encode/parse **INTRA-0.103**
> (round-trip sous le lock courant), PAS la **survie cross-version** d'un
> ticket écrit sous 0.98 puis re-parsé sous 0.103 — qui est le vrai bord
> d'upgrade. Le test idéal (fixture 0.98-genuine) est **légitimement
> interdit** par la politique pre-launch (littéral committé = zombie,
> crate 0.98 hors lock, préflight §8). Contrôles compensatoires RÉELS :
> self-heal non-fatal (`runtime.rs:2278` `if let Ok(ticket) = …from_str
> (stale)`, `AnchorLocator` doc `:254` « stale ticket tolerated ») + carry
> F boot-COPIE (c). **Le body ne doit PAS sur-vendre** : décrire le test
> comme un verrou de format intra-version + tripwire de bump, jamais
> comme une preuve de survie d'upgrade.
> **(P3-1)** l'assertion `got_addr == addr` verrouille le `EndpointAddr`
> **complet** (id + relay + direct) verbatim, plus fort que la surface
> load-bearing SBFB (seule la HASH l'est ; l'adresse est éphémère,
> re-résolue pkarr — préflight §4.2). C'est un tripwire de version
> délibérément plus strict (rougit sur un futur changement d'encodage
> addr d'iroh), fonction voulue — mais coupler le test à la fidélité
> d'encodage addr d'iroh mérite une note pour qu'un futur mainteneur ne
> lise pas un changement légitime de format addr iroh comme une
> régression SBFB.
>
> Séquence : honorer P2-1/P3-1 au body → Codex → réconciliation →
> promotion PASS → commit `chore(deps)`.

## Scope And Staging

`git status --short` = **2 lignes**, 0 fichier parasite :

- `crates/nexus-core-rs/src/blobs.rs` (` M`, +48/−1) — **le fichier de
  phase** : (a) `:481` doc-comment `0.100`→`0.103` dans
  `fetch_falls_back_to_seeder_when_anchor_offline` (carry B → D fermé) ;
  (b) test neuf `blob_ticket_string_round_trips_under_current_lock`
  (`:359-398`), `#[test]` synchrone pur.
- `?? .planning/active/sprint81_phase_d_preflight.md` — planning (relu
  en entier), artefact d'accompagnement de l'atomique code (comme
  Phase C avec son préflight untracked). Ce présent review.md est le
  second artefact planning attendu. Aucun autre fichier.

**Cohérence module Rust** : `git diff --cached '*.rs' | rg '^\+pub mod '`
= vide ; la seule surface publique inchangée. Le diff ne touche QUE le
`mod tests` (`#[cfg(test)]`, `:300+`) sauf le doc-comment `:481` (déjà
dans un test). **0 code prod.**

**INTOUCHÉS prouvés par absence du diff** : `node.rs` (0 doc-stale
numérique — refs génériques « iroh-blobs »/« redb »), `Cargo.toml`,
`Cargo.lock`, `deny.toml`, tout `DOMAIN_*`/`_FORMAT_VERSION`
(grep du diff = 0 hit), `canonical.rs`, `iroh_runtime.rs`, `web/`,
`tools/`. **Aucun store redb ouvert** (règle bloquante préflight §9
tenue : `create_node`/`MemStore` ou pur ; ici le test est PUR, 0 store).

## Three-Block Verification

Suites §7.4 **DÉJÀ JOUÉES ET VERTES deux plateformes** (fournies au
contexte, auditées en cohérence — non relancées : lourdes) :

- **Rust Win** : fmt 0 ; clippy `--all-targets -D warnings` 0 ; nextest
  **2039/2039 0-skip** (baseline C 2038 → **+1 EXACT**, cohérent avec le
  seul test neuf) ; doctests OK ; release build 6m07 OK.
- **Docker sbfb-ci** rust:1.94 : fmt 0 ; clippy 0 ; nextest **2043/2043
  0 fail 0 skip COMPLET** (baseline C 2042 → +1 exact ; classe env
  Docker-on-Windows VERTE ce run, comme A4 — `SBFB_TEST_HTTP_TIMEOUT_
  SECS=120` appliqué) ; doctests OK.
- **web** : lint 0 err (5 warnings préexistants) ; tsc OK ; test:unit
  **411/411 en re-run SOLO** ; coverage 87.27/79.01/86.02/88.59 ≥ seuils ;
  build/size OK (css 129.02/130) ; scan-en-strings clean. **0 fichier
  `web/` au diff** — attesté, non contredit.
- **factory-operator** : lint/tsc/unit 201/build/size OK ; gates **6/6
  re-run SOLO** ; E2E Playwright **10/10**. **0 fichier `tools/`** — idem.

**Requalifications env (documentées, NON régressions)** : au run
parallèle 4-blocs chargé, (1) web test:unit 7 fails puis coverage 2 fails
→ **re-runs solo 411/411 verts** = classe `vitest_env_variance` (charge
jsdom parallèle, memory dédiée) ; (2) `scan-front-discipline` SELF-TEST
FAILED (score/tsx MISS) au même run → **re-run solo gates 6/6 vert** =
même classe env ; (3) 3 shells background tués par fin de session (pas
flakiness des suites — Win nextest/doctests avaient DÉJÀ fini verts,
release + Docker re-joués verts ensuite). Aucune de ces requalifications
ne touche le crate `nexus-core-rs` de Phase D (test pur unit, insensible
à la charge jsdom/HTTP-loopback/Docker-on-Windows).

**Preuve sémantique ciblée** : l'API du test neuf est compile-prouvée
(présente dans nextest 2039) ET adossée à deux tests existants qui
mintent identiquement (`:433`, `:498`). Suite lourde non relancée
(directive : verte, +1 exact confirmé aux deux plateformes).

## Delta Tests

**+1 Rust net** — annoncé et observé cohérent partout : Win 2038→2039,
Docker 2042→2043. Le préflight §8 disait « +1..2, viser +1 net » (le
libellé plan « +1..3 » sur-estime : fetch local + tags + round-trip
2-nœuds DÉJÀ couverts, `blobs.rs:310`/`:529-576`/`:409-453`). **Conforme,
0 zombie ajouté, −0.** L'unique test neuf est bien le round-trip
BlobTicket PUR ; le 2e test « optionnel » du préflight §8 (troncature-16
`fetch_hash_multi`) a été **volontairement omis** (trou pré-existant,
non-régression, routé K) — décision de budget correcte, pas un livrable
manquant.

## Modified-File Branch Coverage

Le diff n'introduit **aucune branche ni méthode de code prod** (le doc-
comment `:481` est prose ; le test neuf EST de la couverture). Le test
neuf lui-même est linéaire (0 `if`/`match`), pas de branche interne à
couvrir. Les assertions couvrent :
- `parsed.to_string() == ticket_str` — idempotence d'encodage.
- `got_hash == hash` — la HASH (champ load-bearing) survit.
- `got_format == BlobFormat::Raw` — le format survit.
- `got_addr == addr` — le `EndpointAddr` peuplé (id + relay + direct)
  survit verbatim (égalité structurelle ; `BTreeSet<TransportAddr>`
  ordre-indépendant — validé par le PASS).

Couverture existante NON dupliquée (préflight §4.3 respecté) :
`add_then_get_roundtrip`, `two_nodes_fetch_blob_via_ticket` (round-trip
end-to-end addr peuplé), `seeder_fetches_tags_pins_blob`,
`fetch_hash_multi_rejects_empty_providers`. Le vrai gap comblé = le
round-trip PUR dédié au format-string `anchors.json`. **Pas de logique
métier non testée > 10 lignes.**

## Security And Protocol

- **Test PUR sans réseau** : `EndpointAddr` construit en mémoire, relay
  `https://relay.sbfb.invalid./` (TLD `.invalid` réservé RFC 2606), IP
  `192.0.2.7:4433` (TEST-NET-1 RFC 5737) — **aucun dial possible**, non-
  routables par construction.
- **Garde anti-store-réel structurelle** : aucun `FsStore`, aucun
  `create_node`, aucun `data_dir` — `#[test]` synchrone. Impossible de
  toucher un store redb réel (règle bloquante préflight §9 : store réel
  interdit avant Phase F PASS ; la migration docs one-way / hard-fail
  blobs `UpgradeRequired` n'est PAS déclenchable ici).
- **0 bump wire SBFB** : `blobs.rs` porte 0 `DOMAIN_*`/`_FORMAT_VERSION`
  (grep du diff vide). Le format string BlobTicket est celui d'iroh
  (compat-upgrade), pas un wire SBFB ; la HASH exposée au front / DB /
  feeds reste hex SBFB découplée de `Hash::Display`.
- **`unwrap()`/`panic!`** : le test utilise `.expect(...)` sur des
  parses statiques (URL/socket/pubkey constants) — légitime en `#[test]`
  (échec = bug d'infrastructure de test, pas un chemin runtime).
- **Grep sensibles** : `unsafe|todo!|unimplemented!` dans le diff = 0.

## Research And G8

- **G8 préflight présent et complet** : `sprint81_phase_d_preflight.md`,
  verdict **PLAN-ADAPT** (4e « lettre pré-bump », structurel/attendu —
  A/A2/A4/C de même classe), 5 scans (S1a OSS byte-diff vendored / S1b
  deps-CVE-lock / S2 décisions / S3 threat / S4 wire) + 5 vérifications
  adversariales, evidence-adossé item par item. Le code SUIT l'approche
  corrigée du préflight, PAS la lettre du plan.
- **Grounding vérifié au repo (non pris sur parole)** :
  - `AnchorLocator.ticket` = `pub ticket: String` doc-commenté « the most
    recent `iroh_blobs::ticket::BlobTicket` string » (`iroh_runtime.rs:
    260-267`) — le test verrouille un contrat réel persisté.
  - Self-heal non-fatal confirmé (`runtime.rs:2278` `if let Ok(ticket) =
    …from_str(stale)`) — un `from_str` KO au boot = branche vide, jamais
    un panic.
  - Blanket `ContentDiscovery` ordre-d'itération 0.103 — byte-diff
    vendored du préflight (S1a-12/S2-3), classe de revendication vérifiée
    toujours vraie.
  - API `EndpointAddr::new`/`with_relay_url`/`with_ip_addr` — compile-
    prouvée (nextest 2039) + `EndpointId::from_str(&hex::encode(...))`
    identique à `:498`.
- **iroh strictement seul (D7)** : 0 dep ajoutée par D (code-only,
  lock/deny inchangés). **Toolchain 1.94 inchangée (D6).**

## Scope Cuts

- **Conformité PLAN-ADAPT** : rien au-delà du scope préflight. 0 code
  prod fonctionnel, 0 `deny.toml`/`Cargo.lock`, aucun store ouvert,
  **scope F non anticipé** (aucune tentative de « valider redb4 »,
  aucune ouverture de store — correctement laissé à F sur COPIE).
- **Carry B `:437`→`:481` FERMÉ** ici (doc-stale `0.100`→`0.103`).
- **Carries F/G/K laissés hors-code (correct)** : F (dualité redb docs-
  migrate/blobs-hard-fail + durabilité pins keep-online + anchors
  graceful-degrade boot-COPIE + snapshot Mac), G (THREAT_MODEL migration
  + quinn-proto CVE bonus), K (troncature-16, sites BlobTicket daemon,
  WS-3/PD-5 NON déclenché car test pur 0 helper). Aucun n'est du code D.
- **Sémantiques compile-invisibles** (tag=racine-GC keep-online, ordre
  providers fetch_hash_multi) : tranchées documentation-only par le
  préflight (byte-identiques 0.100↔0.103), 0 code touché — respecté.
- **docs-contract §6.12 (test-acteur)** : le consommateur du primitif
  `BlobTicket::from_str`/`into_parts` testé est le **même crate**
  (`nexus-core-rs`, module test) ; le format string BlobTicket est
  d'iroh (compat-upgrade), pas un wire SBFB ; `anchors.json` est un
  fichier local lu/écrit par le MÊME daemon (pas un autre nœud, pas un
  runtime distinct, pas un client externe). **N-A-no-new-frontier** —
  aucune étiquette requise (préflight §11 confirmé). À consigner au body.

## Codex verification

Non joué (driver-side pre-Codex). Gate BLOQUANTE review → Codex →
commit. Codex doit remplacer `PASS-PENDING` par `## Verdict: PASS` (ou
CONCERN/FAIL) après vérification indépendante + réconciliation des
dispositions P2-1/P3-1 au body.

**Security delta** : aucun. Test pur non-routable, 0 store, 0 wire,
0 dep, 0 code prod. Le doc-comment corrige une donnée de version
périmée. Surface d'attaque inchangée.

## Commit Body Draft

Shape indicatif (préflight §13), 9 sections canoniques :

- `## Contexte` — PLAN-ADAPT (lettre pré-bump absorbée par B) ; vrai
  périmètre = doc-stale + 1 test pur ; « validation redb4 » = F.
- `## Fichiers` — `blobs.rs` (+48/−1) : doc-comment `:481`
  `0.100`→`0.103` + test `blob_ticket_string_round_trips_under_current_
  lock`.
- `## Delta tests` — **+1 Rust** (2038→2039 Win / 2042→2043 Docker),
  −0 zombie ; +3 du libellé plan sur-estime (redondance existante).
- `## Verification` — Win fmt/clippy/nextest 2039 0-skip/doctests/
  release ; Docker sbfb-ci fmt/clippy/nextest 2043 complet/doctests ;
  web + operator verts (re-runs solo) ; requalifs env documentées.
- `## Scope cuts` — 0 store ouvert (F) ; troncature-16 omise (K) ;
  sémantiques GC/ordre-providers doc-only (byte-identiques).
- `## G8 traceability` — préflight PLAN-ADAPT, 5 scans, S2-10 REFUTED
  réhabilité (anchors.json PERSISTE un ticket).
- `## Pre-launch protocol` — pas de fixture 0.98 forgée (zombie interdit) ;
  round-trip sous le lock courant ; 0 bump wire.
- `## Codex verification` — [après Codex].
- `## Carry closure` — carry B `:481` fermé ; carries F/G/K énumérés.

**Contraintes body à honorer (P2-1/P3-1)** : décrire le test comme
verrou **intra-version** + tripwire de bump, **jamais** comme preuve de
survie d'upgrade 0.98→0.103 (self-heal + F couvrent le bord d'upgrade) ;
noter que l'assertion addr verbatim est un tripwire de format iroh
délibérément strict.

## Findings

### P0 — aucun

### P1 — aucun

### P2 (1, à documenter au body ; ne bloque pas si le body l'honore)

**P2-1 — Le round-trip prouve l'idempotence INTRA-0.103, pas la survie
cross-version d'un ticket 0.98 sur disque.** `blobs.rs:359-398` mint →
`to_string` → `from_str` s'exécute intégralement **sous le lock courant
(0.103→0.103)** : c'est trivialement vrai et n'exerce PAS le bord
d'upgrade, qui est la vraie question d'un sprint d'upgrade iroh (un
`anchors.json` écrit par un daemon 0.98 se re-parse-t-il sous 0.103 au
boot ?). **Pourquoi ce n'est pas un défaut bloquant** : (a) le test
idéal — une fixture ticket 0.98-genuine committée — est **légitimement
interdit** par la politique pre-launch (littéral = zombie, crate 0.98
hors lock, préflight §8/CLAUDE.md) ; (b) contrôles compensatoires RÉELS
et vérifiés au code : self-heal non-fatal (`runtime.rs:2278`
`if let Ok(ticket) = …from_str(stale)` → branche vide, `repull` rend 0,
catalogue ré-arrive au prochain live-announce ; `AnchorLocator` doc
`:254` « stale ticket tolerated ») + carry Phase F boot-COPIE (c)
[`anchors.json` écrit sous 0.98 dégrade sans panic] ; (c) la HASH est le
seul champ load-bearing, l'adresse est re-résolue pkarr. **Disposition** :
le body doit décrire le test comme un verrou de **format intra-version**
+ tripwire de bump, JAMAIS comme une preuve de survie d'upgrade — sinon
un lecteur conclut à tort que la durabilité `anchors.json` à travers
l'upgrade est prouvée. Le doc-comment du test EST déjà honnête (« must
stay stable **under the pinned iroh-blobs** ») ; c'est la narration body
à garder alignée.

### P3 (1, documentable au body, ne bloque pas)

**P3-1 — Assertion addr verbatim plus stricte que la surface load-bearing
SBFB.** `blobs.rs:392-396` : `assert_eq!(got_addr, addr, …)` verrouille
le `EndpointAddr` **complet** (id + relay + direct) octet-structurel,
alors que SBFB ne load-bear que sur la HASH (l'adresse est éphémère,
re-résolue pkarr — préflight §4.2 : « la HASH est le seul champ load-
bearing »). C'est un tripwire de version **délibérément plus strict**
(rougira sur un futur changement légitime d'encodage addr côté iroh),
ce qui est la fonction voulue d'un test `_under_current_lock`. **Risque
mineur** : un futur mainteneur pourrait lire une évolution de format addr
iroh (hash toujours stable) comme une régression SBFB. **Disposition** :
noter au body / en commentaire de code que l'assertion addr est un
tripwire de fidélité d'encodage iroh, distinct de la garantie load-
bearing SBFB (hash). Non-blocking ; l'assertion reste bénéfique (garde
plus large).

### Note G4 (rigor)

Phase **délibérément triviale par conception** (48 lignes, 1 doc-line +
1 test pur, 0 code prod, PLAN-ADAPT — le poids analytique a vécu en
Phase B + dans le préflight à 5 scans). Le trigger CONCERN=« 0 finding
après exploration exhaustive » n'est **PAS** atteint : exploration
exhaustive menée (diff ligne-par-ligne, grounding vendored + daemon
vérifié au code, API confirmée par mirroir de tests existants + delta
+1 exact deux plateformes), **2 findings substantifs réels** remontés
(P2-1 honnêteté body sur intra-vs-cross-version, P3-1 sur-spécification
addr). Aucun P2 manufacturé pour un quota (anti-pattern #3) — la
distinction intra/cross-version EST matériellement importante dans un
sprint d'upgrade iroh.

## Residual Risk

- **Résidu `EndpointAddr` serde peuplé cross-version** (P2-1, préflight
  §4.2/§12) : la byte-identité stricte 0.98→0.103 d'un ticket peuplé
  n'est pas indépendamment diffée ; fermée empiriquement pour 0.103 par
  le round-trip (addr peuplé) + self-heal boot ; le cas 0.98-genuine
  reste un non-scénario pre-launch (route NOTE F).
- **Décision F `blobs.db` illisible** : hard-fail `UpgradeRequired`
  (0.103 a retiré `db.upgrade()`) → discard+refetch (perte pins keep-
  online M18/S74) vs shim, pendante, à trancher AVANT tout boot store
  réel. Hors D (aucun store ouvert ici).
- **Env session** : classe Docker-on-Windows / vitest_env_variance /
  kills background — documentées, **non pertinentes** pour le test pur
  `nexus-core-rs` de D (unit synchrone, 0 réseau, 0 store, 0 jsdom).
- **Claims CVE web-sourcés** (quinn-proto 0.11.14 bonus) : traités
  bonus PLAUSIBLE, 0 impact action D, routés G.

---

**PASS-PENDING** : phase cohérente, vérifiée driver-side (2 plateformes
vertes, +1 exact), minimale par conception. 0 P0/P1. 1 P2 (honnêteté
body intra-vs-cross-version) + 1 P3 (assertion addr), tous deux non-
bloquants si le body les honore. Le gate Codex doit promouvoir en
`## Verdict: PASS` après réconciliation.

## Codex reconciliation

- **Codex GPT 5.5** (`codex exec`, round 1, artefact BRUT
  `sprint81_phase_d_codex_review.md`) : **4/5 CONFIRMÉ, 0 GAP,
  1 PARTIEL**. Le PARTIEL (Livrable 4) ne vise pas le code : la formule
  du préflight « blobs.rs/node.rs = 0 `DOMAIN_*`/`_FORMAT_VERSION`,
  grep vide » était trop large — `node.rs:67`/`:78` portent des
  références DOCUMENTAIRES (doc-links vers
  `crate::seed::SEED_FORMAT_VERSION` /
  `crate::compute_group::COMPUTE_GROUP_FORMAT_VERSION`, constantes
  définies dans `seed.rs`/`compute_group.rs`, NON touchées par la
  phase). **RÉCONCILIÉ** : préflight précisé aux 2 sites (bandeau +
  §11, mention « précision post-Codex round 1 ») ; substance intacte
  (aucune constante wire DÉFINIE ni touchée dans blobs.rs/node.rs).
  0 fix code → pas de boucle complète requise (aucun GAP code).
- **Vérification adversariale post-review** (Workflow `wf_b7dc8399`,
  3 sceptiques Opus 4.8, **3/3 CONFIRMED, 0 P0/P1/P2 manqué**) :
  - **P2-1 CONFIRMÉ** ; correction d'EVIDENCE intégrée : le contrôle
    compensatoire load-bearing du chemin boot `anchors.json` est
    `repull_one_directory` (`iroh_runtime.rs:1305`, arm `Ok(Err)`
    avaleur `:1273`) + le test dédié `repull_tolerates_bad_locator`
    (`iroh_runtime.rs:2659`) — le `runtime.rs:2278` cité au corps de
    review est le chemin OUTBOX-replay (lui aussi non-fatal ; sur
    parse-fail il ré-publie le ticket stale verbatim — comportement
    déjà routé carry F/K, P3 informationnel). Le body du commit cite
    le chemin repull.
  - **P3-1 CONFIRMÉ**, assert addr verbatim SÛR tel quel — NE PAS
    relâcher en hash-only (RelayUrl = point fixe serde
    `serialize_str(as_str)`→`Url::parse`, jamais re-`from_str` d'une
    string brute ; `BTreeSet<TransportAddr>` = égalité d'ensemble
    déterministe ; lock exact + `--locked` → un break exige un bump
    délibéré, jamais silencieux). **Raffinement lossy-wire INTÉGRÉ AU
    CODE post-review** : le wire du ticket ne préserve qu'UN relay +
    les IPs et droppe extra-relays/`Custom` par design (iroh-blobs
    `ticket.rs:72-73`) → +4 lignes de doc-comment au test (contrainte
    de forme de l'addr), fmt 0 + re-run ciblé PASS. **Diff final
    blobs.rs = +52/−1** (le +48/−1 du corps de review = état
    pré-raffinement ; les repères du test/assert cités plus haut sont
    décalés d'autant, ex. doc-stale `:481`→`:488`).
  - **Sweep angles morts : 0 manqué** — doc-comment vrai en 0.103
    (`downloader.rs:562-570` ordre d'itération vs `Shuffled`
    `:585-589`) ; test unique, +1 structurel exact ; parses statiques
    sûrs (RelayUrl sans restriction de scheme, FQDN point-final =
    forme recommandée) ; `EndpointId` depuis pubkey générée = classe
    d'échec inatteignable (pattern préexistant vert `:498`) ; format
    review conforme lightcheck (un seul `^## Verdict`).
- **Dispositions honorées au body du commit** : le test est décrit
  comme verrou de format **intra-version** + tripwire de bump — jamais
  comme preuve de survie d'upgrade 0.98→0.103 (bord d'upgrade couvert
  par `repull_tolerates_bad_locator` + carry F boot-COPIE) ; l'assert
  addr est un tripwire d'encodage iroh délibéré, borné à la forme
  relay-unique+IP que le wire préserve.
