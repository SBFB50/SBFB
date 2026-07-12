# Sprint 81 Phase A4 — Review (Workflow ultracode + arbitrage main-thread)

> Phase A4 (ex-A3b, sous-phase du split PLAN-ADAPT décidé au préflight A3,
> `sprint81_phase_a3_preflight.md §3` + sous-section A3b `l.160-186`) :
> fix(daemon) — le coordinateur ENTRE dans le sync-set iroh-docs de son
> project doc AU BOOT. Arbre SALE, HEAD `7d6b9ea`. 5 dimensions de review
> + 5 vérifications adversariales + arbitrage main-thread de première main
> (git status/diff + lecture code iroh-docs 0.98 + état des jobs suites
> re-vérifiés moi-même).

## Verdict: PASS

> **Diff A4 substantivement PROPRE et CORRECT pour son périmètre déclaré
> (« project doc »).** 0 P0 / 0 P1 sur les 5 dimensions. Le fix root-cause
> est exact (open/create n'entrent PAS le sync-set ; seul `start_sync`
> insère dans `SyncState`, iroh-docs 0.98 `engine/live.rs:414`, re-vérifié),
> 0-bump strict, aucune régression de sécurité (l'autorisation d'écriture
> reste cryptographique ; le reject `NotFound` était de la DISPONIBILITÉ,
> pas de l'auth), tests red→green sémantiquement fondés (CONTROL épingle le
> trou + tripwire bump 0.101, GREEN via le chemin de prod), artefact
> différentiel valide + scrubbé + fidèle au LIVE.
>
> **PENDING** sur 4 axes, aucun n'étant un défaut de correction du diff :
> **(1)** le SEUL gate §7.4 non-évidencié = le bloc **web coverage/build/
> size/scan** (job `biwlo0vgb` = 0 octet, drainé) → **re-run + preuve verte
> MANDATORY avant commit** (`feedback_full_failfast`). **(2)** Deux **P2
> PRÉ-EXISTANTS** (pas des régressions A4) confirmés en Dimension 1 : les
> namespaces **feed/storage** rouvrent HORS sync-set au restart (même
> root-cause, feed = network-visible) et le sélecteur `list_docs().first()`
> est **non-déterministe** dès que feed/storage coexistent → **à DISPOSER
> explicitement (fold OU carry P2 nommé)**, jamais laisser en trou-frère
> silencieux (cohérence avec la prémisse « open != sync-set » du propre
> doc-comment A4 + directive PO « 0 defer du cœur »). **(3)** Artefacts
> process **absents** que le hook lightcheck EXIGERA au commit :
> `sprint81_phase_a4_preflight.md` (Check 8) + `sprint81_phase_a4_codex_review.md`
> (Check 7). **(4)** Body A4 non encore écrit : re-stater les carries
> OUVERTS + enregistrer les fermetures, sans surclamer le b3 PASS.
>
> Les 9 autres findings sont des **P3 non bloquants** (précision doc/carry,
> over-hermétisme de tests, discipline d'écriture du body). Séquence :
> résoudre (1)+(2)-disposition → Codex → commit.

## Portée du diff (re-vérifiée de première main)

`git status --porcelain` à HEAD `7d6b9ea` = 4 fichiers code modifiés + 1
planning untracked, 0 fichier parasite :

- `crates/nexus-core-rs/src/docs.rs` (+31) — wrapper `DocHandle::start_sync(peers)`
  → forward `self.inner.start_sync`, `map_err → NexusError::Docs("start_sync failed: …")`,
  doc-comment ancré 0.98 (`live.rs:414`/`:714`, `state.rs:97`). Signature
  `Vec<iroh::EndpointAddr>` = miroir exact de `iroh_docs::api::Doc::start_sync`.
- `crates/nexus-shell-daemon/src/runtime.rs` (+79/-25) — `pub(crate) fn
  open_project_doc_for_dispatch` (l.2051-2073 : list/open/create IDENTIQUE
  au bloc inline 6c retiré + `start_sync(Vec::new())` appended, fail-fast
  `?` avec `.context` diagnostiquable) + call-site 6c remplacé (l.648) +
  log « (sync-set entered) ». Le net -25 = bloc inline déplacé vers le helper.
- `crates/nexus-shell-daemon/src/dispatch_loop.rs` (+213) — 100 % dans
  `mod tests` (`@@ … mod tests`) : 2 helpers (`boot_persistent_coordinator`,
  `addr_of`) + 2 tests `multi_thread(worker_threads=4)` mode-restart :
  `reopened_project_doc_without_start_sync_does_not_deliver` (CONTROL, l.557)
  et `boot_path_reenters_sync_set_and_delivers_after_reopen` (GREEN, appelle
  `crate::runtime::open_project_doc_for_dispatch`).
- `crates/nexus-shell-daemon/src/http.rs` (+36/-…) — 6 tests `consent_*`
  migrés `mk_state()` → `mk_state_with_sbfb_home(tempdir)` (hermétisme :
  ils lisaient le vrai `~/.sbfb`). 0 impact wire/sécurité.
- `?? .planning/active/sprint81_t2_a4_differential_098.json` (NEW) — artefact
  différentiel vs baseline `a3b_differential_contract`.

**0-bump CONFIRMÉ de première main** : `git diff HEAD` → 0 hit sur
`_VERSION`/`DOMAIN_`/`ALPN`/`JCS`/`canonical`/`FORMAT_VERSION` ; les seules
constantes `_VERSION` de nexus-core-rs (`key_rotation.rs:15`, `schemas/mod.rs`)
INCHANGÉES. `start_sync(vec![])` = control-flow transport, hors wire. Clé
`task:` S71-B1 intacte (le hunk dispatch_loop est 100 % test).

## Restitution par dimension (5 + 5 adversariales)

1. **Correction du fix sync-set boot (docs.rs + runtime.rs, ligne-à-ligne
   vs iroh-docs 0.98)** : PASS pour le périmètre déclaré. Le fix est exact,
   fail-fast SAFE (start_sync n'attend que store/actor/gossip local, le dial
   pair est fire-and-forget `SyncReason::DirectJoin` non-awaité `live.rs:481` →
   un échec WAN transitoire NE fait PAS échouer le boot). **2 P2 PRÉ-EXISTANTS
   + 1 P3** (voir findings).
2. **Sémantique des tests** : PASS — CONTROL prouve VRAIMENT le trou (a2
   rouvert hors sync-set, worker keepalive `doc_b.start_sync(vec![a2_addr])`
   exercé activement NE COMPENSE PAS ; non-livraison MÉCANIQUEMENT permanente,
   pas timing-dépendante) ; GREEN appelle la fn PROD même crate ; delta = +2
   tests EXACTEMENT (2026→2028 Win / 2030→2032 Docker) ; migration consent
   complète (0 `mk_state().await` restant dans un test `consent_`). **1 P3**
   (over-hermétisme du commentaire).
3. **Conformité préflight/carries + scope** : PASS — bloc inline 6c → helper
   + `start_sync(vec![])` ; 0-bump strict vérifiable ; keepalive worker
   RÉFÉRENCÉ non ré-implémenté ; commit shape conforme (adapté ex-A3b→A4) ;
   carries A3 tous routés, aucun silencieusement fermé. **2 P3** (re-cadrage
   carry G « amplification » + discipline body A4).
4. **Sécurité + wire + artefact** : PASS — 0-bump, entrer dans le sync-set
   N'ACCORDE AUCUNE capacité d'écriture (toute entrée `InsertOrigin::Sync`
   passe `validate_entry`→`verify()` double-signature `sync.rs`), redial
   sortant BORNÉ (`get_sync_peers` ≤ `PEERS_PER_DOC_CACHE_SIZE=5`, one-shot),
   artefact JSON valide + scrubé (0 IP/token/sha/path absolu) + fidèle.
   **3 P3** (précision doc-comment : cap non chiffré, note §15.3 à confirmer
   Phase G, « no relay in the hot path » sur-affirmé).
5. **Process + suites §7.4** : PASS-PENDING — Rust A4 post-fmt VERT (job
   `bjzmkmflo` : clippy 0, rerun 3/3, 2× release Finished), Docker 2032/2032,
   fmt re-check EXIT 0. **1 gate NON évidencié = web** (job `biwlo0vgb` vide)
   + 2 artefacts hook absents (préflight/codex A4). **3 P2 + 2 P3**.

## Findings P0-P3 + verdicts adversariaux (arbitrés)

Aucun P0/P1 sur les 5 dimensions. Les désaccords review↔verify étaient nuls
(toutes les findings CONFIRMÉES par leur vérification) ; l'arbitrage porte sur
la **calibration de sévérité** et la **disposition** (fold vs carry).

### P2 — Feed/storage rouvrent HORS sync-set au restart (Dimension 1, PRÉ-EXISTANT)
- **Verdict arbitrage : CONFIRMED, P2, PRÉ-EXISTANT (pas régression A4), à
  DISPOSER (fold OU carry explicite) — ne bloque PAS le commit A4.**
- **Re-vérifié de première main** : `boot_storage_namespace` (`runtime.rs:2550-2565`)
  et `boot_feed_namespace` (`:2645-2659`) partagent le MÊME `docs_client`
  (`:643` = `node.docs()`, un seul store). Sur le bras reopen `Some(doc)`
  avec `row.doc_ticket == Some(t)` (steady-state), ils renvoient le ticket
  persisté VERBATIM SANS `share_write` NI `start_sync` → le namespace reste
  HORS `SyncState`. Seul le sous-bras `None` (ticket pas encore persisté)
  appelle `share_write` (qui, lui, entre le sync-set via `doc_share`). C'est
  la root-cause IDENTIQUE à celle qu'A4 corrige pour le project doc, laissée
  non-fixée pour les 2 frères.
- **Feed = network-visible** : le feed est un vrai chemin d'écriture LIVE
  (`publish_feed_entry_to_docs` `deploy.rs:547`, `feed_sync.rs:650`,
  `runtime.rs:790/846`), pas une simple annonce → un coordinateur redémarré
  ne gossip-broadcast PAS ses écritures feed (`LocalInsert` gated `is_syncing`
  `live.rs:714`) et REJETTE le dial feed d'un pair frais (`state.rs:97`
  NotFound). Même classe S75 « apps invisibles au restart ».
- **Calibration P2 (pas P1)** : (i) PRÉ-EXISTANT depuis S58/S62, pas une
  régression A4 ; (ii) hors du remit explicite « project doc » d'A4 ; (iii)
  **mitigation partielle** — SBFB a aussi le PULL annuaire S75 + anchors et
  le fold materializer gossip Phase A, donc la visibilité live des apps ne
  dépend peut-être pas SEULEMENT du sync feed iroh-docs. Reste un trou de
  convergence réel.
- **Disposition (arbitrage)** : le propre doc-comment A4 affirme le principe
  GÉNÉRAL « open != sync-set » ; laisser les 2 frères silencieux est
  internement incohérent. **Deux voies acceptables** : (a) FOLD — réutiliser
  la discipline `start_sync(Vec::new())` dans le bras `Some(doc)` des deux
  boot sites (+ couverture test équivalente à la discipline CONTROL/GREEN
  qu'A4 applique au project doc, sinon on viole le gate testabilité) ; OU
  (b) CARRY P2 EXPLICITE nommé (body A4 + `sprint81_verification`/audit_plan
  Phase B/K), avec le caveat mitigation-partielle. **Interdit** : trou-frère
  silencieux.

### P2 — `open_project_doc_for_dispatch` sélectionne via `list_docs().first()` (Dimension 1, PRÉ-EXISTANT)
- **Verdict arbitrage : CONFIRMED, P2, PRÉ-EXISTANT, DORMANT, à DISPOSER —
  ne bloque PAS le commit A4.**
- **Re-vérifié de première main** : le helper garde le sélecteur d'origine
  `existing.first()` (`runtime.rs:2058`). `list_docs` forwarde
  `Store::list_namespaces` = scan redb en ordre de CLÉ ASCENDANT sur l'id
  32-octets → pour des `NamespaceId` aléatoires (`create_doc`), ordre
  effectivement arbitraire, PAS l'ordre de création. L'id du project doc
  n'est persisté NULLE PART (pas de ligne M8 ; seul `state.project_doc`
  éphémère à `http.rs:848`), alors que storage/feed le SONT (M8, rouverts par
  id de façon déterministe). Ordre de boot : project (`:648`) AVANT storage
  (`:696`) AVANT feed unconditionnel (`:738`) → 1er boot = store vide →
  project créé seul → `.first()` OK ; tout boot ultérieur présente ≥2-3
  namespaces à `.first()`, project sélectionné seulement si son id trie en
  1er (~1/3).
- **Correction adversariale RATIFIÉE** : le finding original dit « A4 casse
  dispatch — l'opposé de l'intention ». **Imprécis** : le dispatch serait
  DÉJÀ cassé pré-A4 si `.first()` renvoyait le mauvais doc (le coordinateur
  écrirait `task:` / lirait le dispatch depuis feed/storage). A4 ne CASSE
  PAS nouvellement dispatch ; il HÉRITE d'un défaut de sélection pré-existant
  et, en effet de bord, `start_sync` potentiellement le MAUVAIS namespace.
- **Calibration P2 + disposition** : sévère-si-ça-tire, mais PRÉ-EXISTANT
  (S62) et empiriquement dormant (le LIVE VPS marche par chance de tri d'id
  stable par store). La solidité d'A4 en dépend néanmoins. **Fold** =
  sélection déterministe (persister l'id du project namespace + rouvrir par
  id, OU sélectionner le seul id ABSENT de la table M8) + test régression
  (seeder un feed namespace dont l'id trie AVANT). **Carry** = P2 explicite.

### P3 — CONTROL : fenêtre négative 8s (Dimension 1)
- **Verdict arbitrage : CONFIRMED, P3, acceptable tel quel.**
- L'assertion `!await_exact_key(…, 8s)` prouve la non-livraison alors que la
  baseline pré-restart converge en 15s et le GREEN alloue 20s. Défendable car
  la non-livraison est STRUCTURELLE (reject `NotFound` `state.rs:97` quasi
  immédiat), pas une course. Exposition RÉELLE = future : si le bump 0.101
  changeait reject-NotFound en accept-mais-lent, ce tripwire pourrait
  false-pass par timeout. Durcissement OPTIONNEL (asserter l'abort NotFound
  observable). Non actionnable avant commit.

### P3 — Migration consent : commentaire sur-affirme la nécessité (Dimension 2)
- **Verdict arbitrage : CONFIRMED, P3, aucune action code (over-hermétique
  n'est pas un bug).**
- Seul `consent_get_returns_default_config` (`http.rs:8926`) LIT le fichier
  (`load_consent(sbfb_home)` `consent.rs:151-152`) et casserait avec le rig
  L3. Les 5 autres rejettent AVANT toute I/O (`set_consent` 400 avant `save`,
  whitelist 422 en retour EXPLICITE du handler). La migration reste correcte
  et complète ; le commentaire ×6 décrit la sémantique famille-de-routes mais
  sur-affirme la pollution pour les cas de validation.
- **Corrections adversariales RATIFIÉES** : (i) comptage — c'est **5/6**
  non-load-bearing (pas « 4/6 » du titre), 1 seul load-bearing ; (ii) le 422
  n'est PAS un « rejet serde avant le handler » mais un retour EXPLICITE du
  handler (`consent.rs:208-211`/`:229-232`) — la conclusion « aucune I/O
  atteinte » reste juste.

### P3 — Carry Phase G « amplification §15.3 » à re-cadrer (Dimension 3)
- **Verdict arbitrage : CONFIRMED, P3, carry-accuracy, non bloquant.**
- La ligne routée (body A3 `7d6b9ea` + préflight A3 `l.185-186/:309-310`) gate
  la note §15.3 sur « start_sync coordinateur avec peers NON-VIDES ». OR A4
  ÉTABLIT (doc-comments `docs.rs` + `runtime.rs`, + LIVE
  `sprint81_t2_a4_differential_098.json` « re-dialed its persisted peers
  outbound ») que `start_sync(vec![])` re-dial les peers PERSISTÉS à CHAQUE
  boot (`DirectJoin`). Élargir la ligne : « tout start_sync coordinateur —
  Y COMPRIS vec![] au boot, qui re-dial les peers persistés `docs.redb` ;
  borne = known-peer list ≤5, re-vérifier vs iroh-docs 0.101 au bump ».

### P3 — Body A4 : re-stater les carries OUVERTS, ne pas surclamer le b3 (Dimension 3)
- **Verdict arbitrage : CONFIRMED, P3, guidance d'écriture (le diff est
  propre), partiellement déjà couvert.**
- A4 ferme légitimement 2 items du « Carry A4 » : hermétisme `consent_*`
  (6 migrés, les 4 autres `http.rs:10122-10211` déjà hermétiques → 10/10) +
  repose `consent.json` rig L3 permanent. Restent OUVERTS : keepalive
  NeighborDown T2/K, 5 test-rot `multi_daemon`, gossip_exchange standing,
  B/C 0.101. Le JSON ne surclamé PAS (`b3_p1_post_fix` PASS = sanity model
  froid, PAS la preuve — la preuve = `boot_window_pre_submit` à ZÉRO submit +
  les 2 tests hermétiques). **Correction RATIFIÉE** : le carry « re-calibrage
  0.101 » est une **Phase B/C** (body A4 + MEMORY), PAS un track de
  `sprint81_audit_plan.md` (= audit GATE de S80, 11 tracks A..K). Référencer
  `reopened_project_doc_without_start_sync_does_not_deliver` comme tripwire
  du bump dans ce carry Phase B/C.

### P3 ×3 — Précision des doc-comments (Dimension 4)
- **Verdict arbitrage : les 3 CONFIRMED, P3, optionnels.**
- (a) **Cap non chiffré** : « bounded by the store's known-peer list » exact
  mais le plafond dur = `PEERS_PER_DOC_CACHE_SIZE = NonZeroUsize(5)`
  (iroh-docs-0.98 `store.rs:17`, appliqué write `fs.rs:555` + read `:572`) —
  citer ≤5 dials one-shot. (b) **Note THREAT_MODEL §15.3** : le fix A4 est le
  pendant coordinateur du keepalive worker symétrique ; rien dans §15.3
  contredit A4 (`doc_sync.rs` + THREAT_MODEL.md UNTOUCHED, vérifié) ; carry
  Phase G adéquat (severity NONE pour ce commit). (c) **« no relay in the hot
  path »** : SUR-AFFIRMÉ — les peers persistés sont des pubkeys nues
  reconstruites en `EndpointAddr::new(public_key)` sans adresse socket
  (`live.rs:426`), donc résolus via pkarr/N0, après quoi QUIC s'établit
  relais-first avant upgrade direct → un relais PEUT être dans
  l'établissement de connexion. Aligner sur la formulation §15.3
  (« re-résolution via pkarr N0 = chemin de confiance déjà accepté »).
- **Correction RATIFIÉE** : les ancres de ligne des findings 1 & 3
  (« runtime.rs:2055/2056 ») sont FAUSSES — les phrases contestées vivent
  dans le doc-comment ~`2044-2050` (2055/2056 = corps de fonction
  `.list_docs()`/`.await`). Substance inchangée.

### P2 — Web coverage/build/size/scan NON évidencié (Dimension 5)
- **Verdict arbitrage : CONFIRMED, P2, MANDATORY avant commit.**
- **Re-vérifié de première main** : `biwlo0vgb.output` = **0 octet**, mtime
  figée à la création (10:45:20, jamais réécrite). Aucun artefact vert pour
  `test:coverage` / `build` / `size` / `scan-en-strings.sh`. `test:unit`
  (411 solo) et operator (201) SONT attestés par le contexte, mais PAS ce
  bloc précis — qui est justement le contenu du job vide. `feedback_full_failfast`
  rend le bloc frontend obligatoire à CHAQUE commit phase, jamais filtré par
  langage touché. Régression FAIBLE (0 fichier web touché) mais la preuve
  verte est mandatory.
- **Correction RATIFIÉE** : cadrer strictement sur le bloc WEB — le bloc Rust
  A4 post-fmt (`bjzmkmflo`) EST évidencié VERT (CLIPPY-EXIT=0, rerun 3/3
  restart PASS, 2× release Finished).

### P2 ×2 — Préflight A4 + Codex A4 absents (Dimension 5, hook Check 7/8)
- **Verdict arbitrage : les 2 CONFIRMED, P2, hard-blockers au commit,
  séquence process ATTENDUE (pas des défauts-surprise).**
- **Simulé de première main** sur titre `fix(daemon): … Sprint 81 Phase A4` :
  Check 8 (`l.398-401`) `IS_PHASE_IMPL_8` matche `^(feat|fix|…)\(` → `fix(`
  compté → exige `.planning/active/sprint81_phase_a4_preflight.md` → ABSENT
  (ls : seuls a/a2/a3) → exit 2. Check 7 (`l.294-297`) idem → exige
  `sprint81_phase_a4_codex_review.md` → ABSENT → exit 2.
- **Fix** : (a) créer `sprint81_phase_a4_preflight.md` en **POINTEUR** vers
  `sprint81_phase_a3_preflight.md §3` (SPLIT PLAN-ADAPT) + sous-section A3b
  `l.160-186` (qui DESIGN déjà `open_project_doc_for_dispatch` +
  `start_sync(vec![])`, 0-bump) — jamais un re-jeu des 5 scans. (b) Après
  CETTE review, lancer `codex exec` sur le diff A4, coller l'output BRUT
  (`feedback_codex_raw_output`) dans `sprint81_phase_a4_codex_review.md`.
  Ordre non-négociable : review → codex → commit.

### P3 — Piège d'étiquette : titre « Phase A4 », JAMAIS « A3b » (Dimension 5)
- **Verdict arbitrage : CONFIRMED, P3, préventif (contexte a déjà retenu A4).**
- Le regex d'extraction (`l.88`) `Phase[[:space:]]+[A-Z]+[0-9]?` sur « Phase
  A3b » capture « A3 » (b non capturé) → PHASE=a3 → Check 7/8 satisfaits par
  les artefacts a3 EXISTANTS qui reviewent A3a, PAS le diff A4 → bypass
  silencieux du gate Codex. Titre exact : `fix(daemon): Sprint 81 Phase A4 —
  coordinator enters its project-doc sync-set at boot (0-bump)`.
- **Réserve sur la sous-note fmt** : la caractérisation « whitespace-only »
  est PLAUSIBLE mais non reproductible ici (les 4 occurrences `await_exact_key`
  du diff working-tree sont TOUTES des additions de code test neuf, 0 ligne
  `-`/re-wrap d'un `:578` pré-existant ; le job `b663xfaxx` cité n'était pas
  dans mes inputs). Ne pas présenter « rustfmt line-wrap » comme fait établi.

### P3 — Body A4 : 9 sections canoniques Check 9 (Dimension 5)
- **Verdict arbitrage : CONFIRMED, P3, checklist préventive.**
- Check 9 (`l.425-465`) impose 9 headers `^##` : Contexte, Fichiers, Delta
  tests, Vérification, Scope cuts, G8 traceability, Pre-launch protocol,
  Codex verification, Carry closure. Contenu load-bearing cohérent avec le
  diff + le différentiel JSON (cf. « Corrections requises » ci-dessous pour
  les points par section).

## Corrections requises / faites

- **AVANT Codex/commit (MANDATORY, gate §7.4)** : re-jouer le bloc frontend
  `(cd web && npm run test:coverage && npm run build && npm run size &&
  bash scripts/scan-en-strings.sh)` et CAPTURER la preuve verte. NE PAS
  committer sur un job web vide (`biwlo0vgb` = 0 octet). Régression FAIBLE
  (0 fichier web touché) mais preuve verte requise.
- **AVANT Codex/commit (DISPOSITION, non-silencieux)** : trancher les 2 P2
  pré-existants Dimension 1 (feed/storage sync-set + sélecteur `.first()`) :
  soit FOLD dans A4 (avec couverture test équivalente — sinon violation du
  gate testabilité), soit CARRY P2 EXPLICITE (body A4 + verification/audit
  Phase B/K), avec caveat mitigation-partielle pour le feed. Si FOLD, le
  faire AVANT Codex pour qu'il review le diff final.
- **AVANT commit (hook)** : créer `sprint81_phase_a4_preflight.md` (POINTEUR
  A3 §3/A3b) + après cette review `sprint81_phase_a4_codex_review.md` (raw).
- **À l'écriture du body A4 (9 sections)** : (1) Contexte = root-cause
  open/create n'entrent pas le sync-set (`live.rs:414`) → coordinateur
  redémarré (a) ne broadcast pas `task:` (`is_syncing` `live.rs:714`) (b)
  rejette syncs worker (`state.rs:97` NotFound) ; delivery ne tenait qu'au
  `share_write` fragile (`local_worker.rs:306-310`, nudge `http.rs:3459`).
  (2) Fichiers = docs.rs wrapper / runtime.rs helper+call-site / dispatch_loop
  2 tests / http.rs 6 consent / JSON. (3) Delta = +2 (Win 2026→2028, Docker
  2030→2032) ; 6 consent migrés = hermétisme (pas +count). (4) Vérification =
  fmt OK, clippy 0, nextest Win 2028 0-skip, doctests 6/6, release
  daemon+worker+VPS (sha `8c10d2f4`), Docker 2032, **web 411 solo +
  coverage/build/size/scan [RE-ÉVIDENCÉ]**, operator 201, différentiel LIVE
  (boot « sync-set entered », worker 21s post-boot SANS submit → dial accepté
  2s, 0 NotFound, b3 palier1 PASS 26s/30s cold). (5) Scope cuts = 0-bump
  strict, keepalive jamais ré-implémenté, `import_ticket` recreate non touché ;
  **feed/storage sibling P2 = FOLD ou CARRY explicite**. (6) G8 = préflight
  A3 §3/A3b PLAN-ADAPT split, EXECUTE. (7) Pre-launch = 0 bump wire, start_sync
  hors wire, 0 decoder legacy. (8) Codex = artefact a4 raw. (9) Carry closure =
  consent_* hermétisme FERMÉ + repose consent.json + re-calibrer
  is_syncing/start_sync/matcher « Replica not found » vs 0.101 → **Phase B/C**
  (réf. le test tripwire) + THREAT_MODEL warn-only + §15.3 amplification
  (élargie vec![] boot) → Phase G + import_ticket recreate post-launch +
  feed/storage sync-set (si carry).
- **NE PAS présenter** `b3_p1_post_fix` PASS comme preuve du fix (déjà PASS
  via side-effect en baseline) — preuve = `boot_window_pre_submit` (ZÉRO
  submit) + les 2 tests hermétiques.

## État des suites §7.4

- **Rust Win (fmt+clippy+nextest+doctest+release)** : **VERT** — fmt re-check
  EXIT 0 (post-apply), clippy `CLIPPY-EXIT=0` (`bjzmkmflo`), rerun ciblé A4
  3/3 PASS post-fmt, release daemon Finished (6m45), release worker Finished
  (via `bnko252z5`/2e Finished 4m26), nextest 2028/2028 0-skip attesté
  contexte, doctests 6/6 attesté contexte.
- **Docker sbfb-ci** : **VERT** — 2032/2032 0 fail 0 skip (`bmusi03gw`).
  Delta +2 = exactement les 2 tests mode-restart A4.
- **web (test:unit + operator)** : test:unit **411/411 SOLO** (4 flakys de
  charge au parallèle = classe `vitest_env_variance`, 0 fichier web touché) +
  operator **201/201 + build** — ATTESTÉS contexte.
- **web (coverage/build/size/scan)** : ⚠️ **NON ÉVIDENCIÉ** — job `biwlo0vgb`
  = 0 octet, drainé, aucun artefact. **RE-RUN + preuve verte MANDATORY avant
  commit** (seul gate manquant).

## Carries

- **Phase A4 (à disposer AVANT commit)** : feed/storage rouvrent hors
  sync-set (même root-cause, feed network-visible) + sélecteur
  `list_docs().first()` non-déterministe → FOLD ou CARRY P2 explicite, jamais
  trou-frère silencieux.
- **Phase B/C** : re-calibrer `is_syncing`/`start_sync`/broadcast/accept + le
  matcher « Replica not found » + le cap ≤5 contre iroh-docs 0.101 au bump ;
  le test `reopened_project_doc_without_start_sync_does_not_deliver` EST le
  tripwire automatisé du bump (référencer dans le carry, body A4 + MEMORY —
  PAS dans `sprint81_audit_plan.md`).
- **Phase G** : THREAT_MODEL classe warn-only (carry A2) ; §15.3 amplification
  ÉLARGIE (« tout start_sync coordinateur Y COMPRIS vec![] boot re-dial peers
  persistés ≤5 DirectJoin », borne known-peer list, re-vérifier 0.101) +
  surface coordinateur symétrique au keepalive worker ; `cargo deny check`
  post-bump.
- **Phase K / T2** : keepalive worker efficacité WAN (run rig NeighborDown —
  le run boot_window a fait un dial INITIAL accepté, PAS un NeighborDown
  re-dial) ; palier 2 quorum ; 5 test-rot `multi_daemon` (remise-à-niveau CI).
- **Standing (NON A4, ne pas conflater)** : RE-DRIVE-ON-INGEST, SeedAnnounced
  peer_count:0, seeder catalog_len:0, PULL-3 — chemin découverte/seed, pas
  task-delivery. `import_ticket` recreate = dette post-launch.
- **Fermés par A4** : hermétisme `consent_*` (10/10, TEST-ISOLATION-SBFB-HOME
  classe §P72) + repose `consent.json` rig L3 permanent.

## Codex reconciliation

**Joué** (`codex exec` GPT 5.5, output brut `sprint81_phase_a4_codex_review.md`,
round 1) : **8/8 CONFIRMÉ, 0 GAP, 0 PARTIEL** — CLEAN. Les 8 livrables audités
avec évidence fichier:ligne (wrapper docs.rs:385-409 ; helper runtime.rs:2051 +
call-site :648 « sync-set entered » ; CONTROL dispatch_loop.rs:556-629 ; GREEN
:642-722 doc_id assert + re-dial accepté ; 6 consent NO_MK_STATE/USES_TEMP_HOME
+ scan global ALL_CONSENT_TESTS_NO_MK_STATE_AWAIT ; artefact différentiel JSON
parse + NO_SECRET scan ; préflight pointeur §2/§3/§6/§7/§10 ; 0-bump diff
contrôlé 0 hit _VERSION/DOMAIN_/ALPN/canonical/JCS, doc_sync.rs intact).

**Résolution des 2 actions MANDATORY de cette review** :
1. **Gate web re-capturé VERT à machine calme** : vitest 411/411 (coverage
   instrumenté inclus — le flaky GpuConsentDialog est confirmé flaky de CHARGE,
   classe vitest_env_variance), coverage 87.27/79.01/86.02/88.59 ≥ 85/78/85/85,
   build OK, size 129.02/130 kB, scan-en-strings clean.
2. **Disposition des 2 P2 pré-existants Dimension 1 : CARRY EXPLICITE**
   (jamais fold — bisectabilité « iroh STRICTEMENT SEUL », phases petites, et
   ces sites sont RE-MIGRÉS au bump) :
   - **P2-SIBLING-SYNC-SET** : `boot_storage_namespace`/`boot_feed_namespace`
     rouvrent leurs namespaces hors sync-set sur le bras ticket-persisté
     (runtime.rs:2552-2564 / :2647-2659 — même root-cause que le project doc,
     le feed est network-visible ; mitigation partielle : chemin PULL S75 pour
     la découverte). Routé **Phase B/C** (le re-calibrage
     is_syncing/start_sync au bump 0.101 doit statuer sur les 3 sites d'un
     coup) + **Phase K** (T1 convergence feed déjà couvert par les tests
     feed_sync in-process — vérifier qu'ils couvrent le mode restart).
   - **P2-PROJECT-DOC-SELECTOR** : `list_docs().first()` non-déterministe dès
     que plusieurs namespaces coexistent dans le store partagé (le project
     namespace id n'est pas persisté ; pré-existant S58/S62, hérité verbatim
     par le helper A4). Vrai fix = persister l'id (colonne locale, design à
     part) — routé **sprint dette / Phase K** (jamais dans le commit A4).
   Les deux sont consignés au body (section Carry) — pas de trou-frère
   silencieux.

Séquence review PASS-PENDING → gate web + disposition → Codex → réconciliation
→ **PASS** respectée. Commit `fix(daemon)` autorisé.
