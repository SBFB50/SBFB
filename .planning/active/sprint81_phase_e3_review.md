# Sprint 81 Phase E3 — Review (Workflow ultracode + agent de synthèse)

> Phase E3 « hot-join gossip du curateur souscrit » (nom canonique, regex
> README §4 `Phase [A-Z]+[0-9]?` ; précédent A4/E2 réels). Déclencheur =
> défaut OBSERVÉ LIVE à l'acceptance zéro-n0 E2
> (`sprint81_t2_e2_zero_n0.json` §residual, 2026-07-05) : `POST
> /api/daemon/curators/subscribe` à chaud → browse vide, aucun dial jusqu'au
> restart (le bootstrap set gossip est lu 1× au boot). Contrat =
> `sprint81_phase_e3_preflight.md` (verdict **PLAN-ADAPT**, prémisse « (a)
> seul laisse le browse vide » falsifiée par le code — replay outbox sur
> NeighborUp ; périmètre (b) pull-directory re-drive HORS-SCOPE, mécaniquement
> inapplicable au 1er subscribe faute de locator). Périmètre diff = 3 fichiers
> code (`gossip.rs` / `runtime.rs` / `http.rs`) + `sprint81_plan.md` §E3 + 2
> untracked E3 (préflight + `sprint81_t2_e3_hot_subscribe.json`). Arbre SALE,
> HEAD `a085853`, diff NON committé. Le fichier untracked
> `sprint81_phase_f_preflight.md` est HORS PÉRIMÈTRE (Phase F à venir, NON
> reviewé). Review menée sur le diff COMPLET + grounding vendored
> `iroh-gossip-0.101.0`, synthèse de **5 dimensions** (D1 diff / D2 tests /
> D3 grounding / D4 sécurité / D5 scope) + vérifications adversariales,
> réconciliées ici avec re-vérification à la source. **LECTURE SEULE — 0
> cargo/npm relancé** en synthèse (suites §7.4 fournies au contexte ; runs
> ciblés légers permis, Docker sbfb-ci + bloc operator EN COURS en background,
> réconciliés par le main thread AVANT commit).

## Verdict: PASS

> Promotion post-Codex (2026-07-05, même session) : gate Codex GPT 5.5 jouée
> round 1 = **9/9 CONFIRMÉ, 0 GAP, 0 PARTIEL** (artefact brut
> `sprint81_phase_e3_codex_review.md`). Aucune correction requise, aucune
> boucle. Le verdict initial de synthèse (PASS-PENDING) est promu PASS.

## Codex reconciliation

Rapport Codex lu intégralement : 9 livrables vérifiés indépendamment sur le
diff working-tree (evidence fichier:ligne par livrable, dont le livrable 9
« 0 bump wire » re-prouvé par `git diff --numstat -- Cargo.toml Cargo.lock`
vide + enum `GossipCmd` non sérialisé). 0 GAP, 0 PARTIEL → aucune
correction, aucun re-run de suites requis au-delà des re-runs ciblés déjà
consignés (§4bis dispositions appliquées AVANT le run Codex). Le fichier
Codex est l'output brut de `codex exec -o`, non réécrit.

> **Diff Phase E3 substantivement CONFORME au contrat préflight PLAN-ADAPT,
> re-vérifié à la source.** 0 P0. **0 P1.** Aucun P2. Uniquement des P3
> (doc-complétude, couverture de glue best-effort sans effet observable,
> hygiène de commit) — **aucun fix obligatoire**, aucune disposition
> bloquante. Le fix est root-cause, iroh-natif, localisé : `TopicSender::
> join_peers` (nexus-core-rs, parse par-pair tolérant) + variante interne
> `GossipCmd::JoinPeers(Vec<String>)` + bras select loop (miroir
> Outbox/RequestBrowse) + push depuis le bras `Ok` de `subscribe_curator` via
> `gossip_cmd_tx` existant, APRÈS l'early-return duress → duress-safe par
> construction et verrouillé par test négatif. Les invariants non-négociables
> sont TOUS tenus (0 bump wire, iroh strictement seul, pins `=1.0.1`/`=0.101.0`
> intacts, verrous S74/S75, duress non re-gaté), vérifiés à la source cette
> session. Le delta +4 tests hermétiques (2047→2051) est EXACT. Aucune
> frontière docs-contrat §6.12 neuve n'est créée (env = input lu par le code,
> `GossipCmd` = mpsc interne, `SubscriptionsResponse` byte-identique). Aucun
> finding adversarial n'a été REFUTED — mais aucun n'atteignait la sévérité
> P2 : les 5 dimensions convergent en PASS local, les résidus P3 sont soit
> déjà routés carry G/S75 par le préflight, soit de la doc-complétude
> optionnelle, soit de l'hygiène de commit mécanique.
>
> **PASS-PENDING = review OK, Codex PAS encore joué (jamais un verdict final
> committable).** Séquence : réconcilier Docker sbfb-ci + bloc operator au
> body → documenter les carries G/S75/Phase-C au commit body → gate Codex →
> réconciliation → promotion `## Verdict: PASS` → commit `fix(daemon)`.

## 1. Périmètre et staging

`git status --short` (hors warnings CRLF) = **6 lignes**, 0 fichier parasite
dans le périmètre E3 :

- `M .planning/active/sprint81_plan.md` — déclaration §Phase E3 (précédent
  A4/E2, +35 lignes).
- `M crates/nexus-core-rs/src/gossip.rs` — `TopicSender::join_peers` + test
  `join_peers_skips_bad_ids_and_enqueues_valid` (+77).
- `M crates/nexus-shell-daemon/src/http.rs` — push dans le bras `Ok` de
  `subscribe_curator` + 3 tests channel-boundary (+153/−7).
- `M crates/nexus-shell-daemon/src/runtime.rs` — variante `GossipCmd::
  JoinPeers(Vec<String>)` + bras select loop (+19).
- `?? .planning/active/sprint81_phase_e3_preflight.md` — artefact préflight
  (contrat, ne se review pas lui-même ; à stager).
- `?? .planning/active/sprint81_t2_e3_hot_subscribe.json` — artefact T2
  (palier live RIG-ABSENT tracé ; à stager).

**HORS PÉRIMÈTRE (ne PAS stager au commit E3)** : `??
.planning/active/sprint81_phase_f_preflight.md` — préflight Phase F à venir,
untracked, non reviewé.

## 2. Vérification trois blocs (suites §7.4)

Suites **fournies au contexte (déjà jouées), auditées en cohérence — non
relancées en synthèse** :

- **Rust Win** : fmt 0 ; clippy `--all-targets -D warnings` 0 ; nextest
  workspace **2051/2051 0-skip** (baseline E2 2047 → **+4 EXACT** : 3 tests
  http channel-boundary + 1 test core wrapper) ; doctests OK ; release build
  `nexus-shell-daemon` vert. **RED-avant-GREEN prouvé** pour le CONTROL #1
  (`send` retiré → `try_recv` Empty FAIL = reproduction du défaut pré-fix ;
  restauré → PASS), consigné dans le T2 §observed.
- **Docker sbfb-ci** rust:1.94 + **bloc operator** : **EN COURS au lancement
  de la review** — résultats réconciliés par le main thread AVANT commit.
- **Front (web)** : lint + tsc + unit 411 (1 flaky `GpuConsentDialog` timeout
  de charge requalifié 17/17 solo — classe `vitest_env_variance` documentée) +
  coverage vert + build + size 6/6 + scan-en-strings clean. **AUCUN fichier
  front touché** (`git status` = 0 `web/`/`tools/`) → insensible par
  construction.

**Delta tests** : **+4 Rust net**, cohérent (2047→2051), 0 zombie, aucun test
`legacy-decode` (E3 ne redéfinit aucun format persisté). **Aucun test committé
nommé `convergence_*`** (vérifié : `git diff | grep convergence` = NONE) → le
groupe nextest capé `two-node-convergence` n'est pas alimenté. La convergence
réelle 2-nœuds = **T2 LIVE**, jamais un unit test committé (interdit comme
flaky, `runtime.rs:3062-3065`).

## 3. Findings retenus par dimension (evidence re-vérifiée à la source)

Toutes les severités = **P3** (aucun P0/P1/P2). Aucun finding n'est bloquant.

| id | dim | sév | claim (re-vérifié) | evidence disque | disposition |
|---|---|---|---|---|---|
| **D1-1** | D1 | P3 | Le commentaire inline http.rs n'énonce pas la précondition « dialable ssi la clé souscrite EST l'endpoint-id du pair » ; couverte seulement par le préflight §3. Parité assumée avec le bootstrap boot (best-effort silencieux sinon), 0 régression. | `http.rs:901-905` (push) ; `api.rs:192` `join_peers(peers: Vec<EndpointId>)` | document-in-body (reword optionnel, non bloquant) |
| **D1-2** | D1 | P3 | Doc-comment cross-crate : `gossip.rs` (nexus-core-rs, crate socle) référence nommément le caller `subscribe_curator` (nexus-shell-daemon). Honnête (décrit le présent immuable, `subscribe()` fait `parse_pubkey_hex` en amont) mais couple légèrement la doc core à un caller daemon. | `gossip.rs:517-521` (« Callers routed via `subscribe_curator` … skip path is defense in depth only ») | document-in-body (reformulation générique optionnelle) |
| **D2-1** | D2 | P3 | Le bras select `GossipCmd::JoinPeers` (`runtime.rs:1852`) est exécuté par ZÉRO test (contrairement au frère `Outbox`, testé task-level `runtime.rs:3094`) : les 3 tests http assertent ce qui est POUSSÉ au canal (le helper `mk_state_with_mode_tx` ne spawne AUCUNE tâche gossip, vérifié `http.rs:4389-4420`), le test core #4 appelle `sender.join_peers` en direct. Sous-branch `Err(e)=>debug!` a fortiori non couvert. | `runtime.rs:1847-1855` ; `http.rs:4389-4420` (helper sans spawn) | ACCEPT (couvert par composition) — asymétrie JUSTIFIÉE : JoinPeers sur nœud isolé n'a AUCUN effet observable (vs side-effect DB d'Outbox), et le crate interdit les tests mesh 2-nœuds in-process. NE PAS ajouter de test mesh committé. Doc-note optionnelle. |
| **D2-2** | D2 | P3 | L'arm `Err` de `inner.join_peers` (`map_err`, `gossip.rs:535-538`) non couvert : sur nœud isolé vivant l'acteur enfile `Command::JoinPeers` et renvoie Ok (membership hint, pas connect-await). | `gossip.rs:535-538` | ACCEPT — chemin d'erreur d'un send acteur in-process, miroir du bras `Err` non testé de `broadcast`. Non-bloquant. |
| **D2-3** | D2 | P3 | Composite « input all-garbage → parsed vide → early-return Ok » non asserté séparément (les 2 branches constituantes — skip-warn `:524-530` + `is_empty` `:532` — sont couvertes chacune par le test core #4 cases a/c). | `gossip.rs:532-534` ; test #4 `gossip.rs:790-809` | ACCEPT — composite trivial. Option low-cost = 1 ligne. Non-bloquant. |
| **D2-4** | D2 | P3 | Test #2 duress n'auto-assert pas le corps « liste vide » (couvert par le sibling pré-existant intact `daemon_boot_in_duress_mode_rejects_curator_subscribe_real`) ; l'apport E3-pertinent de #2 = le canal-vide (verrou by-construction). | `http.rs:6670` (#2) ; sibling `http.rs:7849-7860` | no_change_needed — corps-vide déjà régression-locké ailleurs ; purement informationnel. |
| **D4-1** | D4 | P3 | Asymétrie unsubscribe : `iroh-gossip 0.101` n'expose AUCUN verbe leave (`Command` = Broadcast/BroadcastNeighbors/JoinPeers seuls, `api.rs:376-383` vérifié) → le pair reste voisin HyParView jusqu'au churn ; ingest droppé par `is_subscribed=false` → fuite bornée au transport. | `api.rs:376-383` (pas de leave) ; `iroh_runtime.rs:751` (gate ingest) | carry-G (déjà routé préflight §8.1a) — 0 changement code E3 ; documenter THREAT_MODEL Phase G. |
| **D4-2** | D4 | P3 | Reconnexion-après-drop : `join_peers` connecte maintenant mais n'ajoute pas le pair au bootstrap-set du topic (figé `subscribe_topic`, `runtime.rs:1548`) → re-bootstrap seulement au reboot. | `gossip.rs:517-534` (hint) ; `runtime.rs:1548` (set figé) | carry-G (déjà routé préflight §8.1d) — 0 changement code E3. |
| **D5-1** | D5 | P3 | Artefacts de phase untracked à stager au commit, en excluant le préflight Phase F hors-périmètre. | `git status` : `?? sprint81_phase_e3_preflight.md`, `?? sprint81_t2_e3_hot_subscribe.json` (E3, à stager) ; `?? sprint81_phase_f_preflight.md` (HORS E3) | NOTE hygiène — état pré-commit normal ; rappel mécanique. Aucun changement code. |

## 4. Arbitrages adversariaux (réconciliation de synthèse)

Les 5 rapports de dimension portaient `adversarial: null` (aucune réfutation
soumise séparément) — la charge adversariale a été **absorbée en amont dans le
préflight PLAN-ADAPT** (5 scans + 5 vérifications adversariales, 0 REFUTED, 0
DESIGN-CONFLICT, cf. préflight §9) puis re-vérifiée à la source par cette
synthèse. Résultat de la re-vérification indépendante sur disque :

- **« 0 bump wire » — CONFIRMÉ (non réfuté)**. `pub enum GossipCmd` n'a
  **aucun** `#[derive(Serialize)]` (`runtime.rs:1482`, `tokio::sync::mpsc`
  interne process) ; `Command::JoinPeers(Vec<EndpointId>)` dérive
  Serialize/Deserialize côté iroh (`api.rs:376-383`) **mais** c'est de l'irpc
  in-process — la membership HyParView traverse le réseau via le protocole
  iroh-gossip **inchangé**, pas une surface wire SBFB. Aucune touche
  `FeedEntry` / `ProjectAnnouncement` / `NodeDirectoryEntry` / `*_FORMAT_VERSION`
  / `DOMAIN_*_V1`. **Réfute-candidat neutralisé.**
- **« duress-safe par construction » — CONFIRMÉ**. Le push (`http.rs:903`) vit
  dans le bras `Ok(_)`, **après** l'early-return `curator_subscribe_in_duress
  == Noop` (`http.rs:878-888`, return 200 liste-vide AVANT le `match subscribe`).
  Le grep confirme `http.rs:903` = **SEUL producteur** prod de `JoinPeers` (le
  consommateur est `runtime.rs:1847`). Verrouillé par le test négatif #2
  (duress → canal EMPTY, clé VALIDE `"cd"×32` isolant la cause duress).
- **« pas de frontière §6.12 neuve » — CONFIRMÉ**. `SubscriptionsResponse`
  byte-identique (le diff ne touche QUE l'indentation de la ligne de champ
  `subscribed_curators:`, pas la struct ni son `deny_unknown_fields`). Les env
  vars ne sont pas touchées ; `GossipCmd` est un canal interne. Aucune
  étiquette test-acteur due — cohérent avec le REFUTED D5-1 d'E2 (env = INPUT
  lu par le code, mpsc = interne).
- **« signature type-compat » — CONFIRMÉ**. `TopicSender.inner: GossipSender`
  (`gossip.rs:494`) ; le wrapper passe `Vec<PublicKey>` à
  `GossipSender::join_peers(Vec<EndpointId>)` (`api.rs:192`) — `iroh::PublicKey
  == EndpointId`, compile vert (nextest 2051).
- **« delta +4 exact » — CONFIRMÉ**. 4 fn de test ajoutées, chacune 1×, 0
  supprimée : `gossip.rs` #4 + `http.rs` #1/#2/#3.

Aucun finding n'a survécu la re-vérification à une sévérité ≥ P2. Aucun ne
requiert un fix-in-phase bloquant.

## 4bis. Rejeu fallback des dimensions crashées (D6 patterns / D7 suites)

Le Workflow review a perdu 2 dimensions sur cap de retries StructuredOutput
(pipeline[5]=D6, pipeline[6]=D7) — la synthèse ci-dessus a tourné sur 5/7
dimensions. Conformément au fallback §7.1 (précédent Phase E : D4/D5
rejouées en Agent avant-plan), les deux dimensions ont été REJOUÉES en
agents fallback après la synthèse, résultats consignés ici sans réécrire la
synthèse :

- **D6 patterns/conventions — PASS, 1 P3.** Nommage/langue/emoji conformes ;
  doc-comments au registre des fichiers hôtes, claims vérifiés vrais
  (`parse_bootstrap` collect-abort confirmé `gossip.rs:431-439` ; « ONLY
  producer » confirmé — unique `send` prod) ; miroir structurel des bras
  select exact ; aucun magic number introduit (`channel(8)`, `"cd".repeat(32)`,
  topic `nexus-grid-test/` = conventions pré-existantes) ; plan §E3 au gabarit
  A4/E2 ; T2 JSON = miroir exact des clés top-level d'e2. PATTERNS.md :
  recommandation PAS d'entrée dédiée (geste trop local) ; la leçon « set lu
  1× au boot + muté au runtime = mutation inerte jusqu'au restart » à folder
  dans le lot PATTERNS du wrap-up K (carry existant « pattern seam » E2).
  **D6-1 (P3, cosmétique)** : ordre des bras du match ≠ ordre de déclaration
  de l'enum — **APPLIQUÉ post-review** (bras `JoinPeers` déplacé après
  `RequestBrowse`, fmt + tests ciblés re-verts).
- **D7 suites §7.4 — PASS, 0 finding matériel.** Arithmétique EXACTE :
  Win 2047→2051 (+4), Docker 2051→2055 (+4 miroir, écart standing +4
  `#[cfg(unix)]` conservé) ; baseline 2047 retrouvée indépendamment
  (`sprint81_phase_e2_review.md:92`) ; les 4 tests E3 dans le périmètre
  nextest par défaut (aucun `#[ignore]`/`cfg`, aucun ne matche le filtre
  `two-node-convergence` `.config/nextest.toml:37`) ; requalification
  GpuConsentDialog légitime (0 fichier `web/` au diff) ; liste de blocage
  avant commit = les 2 items mécaniques attendus (Codex + promotion verdict).
  Corroboration latérale : `boot_path_reenters_sync_set` matche le filtre
  du groupe lourd 2-nœuds, ce qui soutient la requalification env du run
  Docker (PASS solo 5.9s sur les DEUX plateformes au re-run).

**Dispositions optionnelles appliquées post-synthèse** (comments only, fmt +
tests ciblés re-verts après chaque édit) : D1-1 (doc-comment `http.rs` bras
`Ok` — nuance « dialable ssi clé souscrite = endpoint-id du pair, sinon
best-effort silencieux comme le bootstrap boot »), D1-2 (doc-comment
`gossip.rs` reformulé générique, plus de référence descendante au handler
daemon dans le crate core), D6-1 (ordre des bras).

## 5. Dispositions pour le main thread

**Aucun fix obligatoire (0 P0/P1/P2).** Toutes optionnelles ou mécaniques :

1. **Réconcilier au body les suites background** — Docker sbfb-ci rust:1.94
   (attendu 2044→**2048** Docker, +4 miroir Win) + bloc operator (attendu
   inchangé, 0 fichier operator touché). BLOQUANT-mécanique avant commit (le
   hook lightcheck l'exige), pas un finding.
2. **Stager exactement 5 chemins** (D5-1) : `git add` les 3 fichiers code +
   `sprint81_plan.md` + `sprint81_phase_e3_preflight.md` +
   `sprint81_t2_e3_hot_subscribe.json`. **NE PAS** inclure
   `sprint81_phase_f_preflight.md` (hors-périmètre, laisser untracked).
3. **Documenter les carries au commit body** (§6 ci-dessous) — G (asymétrie
   unsubscribe + reconnexion-après-drop + résidu boot-duress pré-existant) /
   S75 (re-drive boot-SEED-driver, orthogonal, **explicitement NON fermé** par
   E3) / Phase-C (résidu directory `from_subscribed` à chaud, si PO ratifie
   « Découvert suffit »).
4. **Reword doc-comment OPTIONNEL** (D1-1/D1-2, non bloquant) — si souhaité,
   1 phrase au commentaire `http.rs` (« dialable ssi la clé souscrite est
   l'endpoint-id du pair ; sinon best-effort silencieux, comme le bootstrap
   boot ») et reformulation générique du doc-comment `gossip.rs:519-521`. Peut
   être laissé tel quel (les deux sont honnêtes et factuellement vrais).
5. **Gate Codex** — lancer `codex exec -o` (output brut, jamais réécrit) après
   réconciliation body. Critère d'arrêt de boucle = « CLEAN ou P2/P3
   documentés ». Promotion `## Verdict: PASS` uniquement après Codex.

## 6. Carries sortants (E3 → G, S75, Phase-C)

Aucun carry n'est OUVERT par E3 comme dette neuve ; tous sont **routés par le
préflight §8** et re-confirmés ici. E3 ne prétend fermer aucun d'eux :

1. **G (THREAT_MODEL / doc §15.x)** : (a) asymétrie unsubscribe (pas d'API
   leave iroh-gossip, `api.rs:376-383` ; fuite transport bornée, ingest gaté) ;
   (b) résidu boot-duress **pré-existant** (dial subscribe + fetch repull +
   replay outbox curateur sous clé leurre, non couvert par `DURESS-BOOT-LEAK`
   §15.1 ; E3 ne l'aggrave PAS) ; (c) note « hot-join curateur duress-safe-par-
   placement, 0 dial nouveau sous duress » ; (d) reconnexion-après-drop (join
   sans ajout au bootstrap-set).
2. **S75 — `re-drive-on-ingest` du boot SEED driver (OVERDUE 3/3)** : distinct
   du défaut E3 (orthogonal, préflight §2.2 : concerne les PROPRES apps
   keep_online du nœud, pas « j'ai souscrit à un pair »). E3 ne le ferme PAS et
   ne doit PAS prétendre le faire → à fermer explicitement ou re-justifier
   « blocker externe » au prochain audit gate.
3. **Résidu directory Phase C** : « Tes sources »/`from_subscribed` + liste
   curatée signée n'arrivent pas à chaud (annonce NodeDirectory LIVE-ONLY,
   jamais dans l'outbox, `http.rs:1372-1388`). Si PO ratifie « Découvert
   suffit » (défaut recommandé du préflight §2.3), reste résidu accepté ;
   sinon = item Phase-C à ré-instruire (nouveau verbe gossip request-directory
   OU replay directory côté producteur).

## 7. Residual Risk

- **Suffisance de (a) non runtime-prouvée avant le T2 (P1-classe préflight, PAS
  un finding review)** : la convergence hot-subscribe est **code-SUPPORTÉE**
  (replay outbox sur NeighborUp, `runtime.rs:1747-1783`) mais **pas
  runtime-PROUVÉE** hermétiquement — aucun test mesh 2-nœuds committé (interdit
  flaky). Le **décideur BLOQUANT reste le T2 LIVE hot-subscribe**
  (`sprint81_t2_e3_hot_subscribe.json`, verdict `RIG-ABSENT` traçable au
  commit, replay procédure documentée). Feature PROVISIONAL + carry P1 tant que
  le T2 live n'est pas rejoué PASS (posture identique à E2, dont le palier live
  a été rejoué same-day une fois l'infra provisionnée). Ce n'est PAS une
  régression review — c'est la barre de testabilité par-sprint appliquée.
- **Reconnexion-après-drop** : un lien B↔A flaky mid-session peut re-vider le
  browse (join sans re-bootstrap), pertinent au design de la fenêtre T2, à
  noter dans le runbook T2. Hors root-cause du défaut observé (= le 1er dial).
- **Couplage discovery** : E3 n'ajoute AUCUNE capacité discovery ; il restaure
  la parité avec le dial du boot. L'efficacité de (a) est couplée à la
  discovery active (N0 pré-EOL ou E2 zéro-n0 self-hosted) — le T2 doit tourner
  sur le stack disponible.
- **Env session (05/07)** : contention cargo Win persiste (crash rustc
  « iroh_blobs rlib » re-run vert) ; ne PAS lancer nextest workspace Win
  pendant Docker/Codex. `docker run` Git Bash = `MSYS_NO_PATHCONV=1` + chemin
  Windows.

---

**PASS-PENDING** : phase substantivement conforme au contrat préflight
PLAN-ADAPT, minimale et root-cause par conception, invariants TOUS tenus et
re-vérifiés à la source (0 bump wire, iroh seul, pins intacts, verrous S74/S75,
duress non re-gaté + test négatif). **0 P0, 0 P1, 0 P2.** Uniquement des P3
(doc-complétude optionnelle D1-1/D1-2 + couverture de glue best-effort sans
effet observable D2-1..D2-4 + hygiène de commit D5-1) — aucun bloquant. Carries
G/S75/Phase-C routés par le préflight, non ouverts comme dette neuve. Une fois
Docker sbfb-ci + bloc operator réconciliés au body et les carries documentés,
le gate Codex peut promouvoir en `## Verdict: PASS`.
