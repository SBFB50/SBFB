# Sprint 81 Phase E3 — Préflight G8 (Workflow ultracode) — hot-join gossip du curateur souscrit

> **Verdict : PLAN-ADAPT.** La Phase E3 corrige le défaut OBSERVÉ LIVE à l'acceptance zéro-n0 E2
> (`sprint81_t2_e2_zero_n0.json` §residual, 2026-07-05) : un nœud frais fait
> `POST /api/daemon/curators/subscribe {node_id du pair}` À CHAUD, le browse reste vide 60 s+, **aucun
> dial** ne part ; seul un RESTART du daemon rejoue le bootstrap gossip et déclenche le dial. **Root cause
> CONFIRMÉE au code (pas présumée)** : `subscribe_curator` (`http.rs:889`) ne mute que l'attention-set +
> `subscriptions.json` (`iroh_runtime.rs:671-692`) et ne notifie **JAMAIS** la tâche gossip vivante ;
> `bootstrap_peers` est lu **une seule fois** au boot (`runtime.rs:1105`) puis figé dans
> `subscribe_topic` (`runtime.rs:1548`). L'enum interne `GossipCmd` (`runtime.rs:1482-1489`) n'a **aucun
> verbe « rejoins ce pair maintenant »** — alors que son propre doc-comment annonce déjà les commandes
> « from HTTP handlers **or the curator subscribe endpoint** » (`runtime.rs:1480`). Le fix est
> **root-cause, iroh-natif, localisé** : iroh-gossip 0.101 expose EXACTEMENT la primitive manquante,
> `GossipSender::join_peers(Vec<EndpointId>)` (`api.rs:192` → `Command::JoinPeers` `api.rs:382` → dial
> HyParView via la discovery de l'endpoint = chokepoint zéro-n0-compatible par construction). E3 = **4
> edits** : (1) wrapper `TopicSender::join_peers` dans `nexus-core-rs` (aujourd'hui broadcast-only,
> `gossip.rs:497-505`) ; (2) variante interne `GossipCmd::JoinPeers(Vec<String>)` + bras du select loop
> (miroir Outbox/RequestBrowse) ; (3) push depuis `subscribe_curator` **dans le bras Ok**, via le
> `gossip_cmd_tx` **déjà présent** dans `DaemonHttpState` (`http.rs:93`, déjà drivé par `browse_pull`
> `http.rs:1062-1065`) ; (4) tests. **0 bump wire** (`GossipCmd` = mpsc interne process ; `Command::JoinPeers`
> = commande actor iroh in-process, la membership HyParView traverse le réseau **inchangée**), **iroh
> strictement seul**, pins `=1.0.1`/`=0.101.0` intacts, **duress-safe par construction** (le gate
> `curator_subscribe_in_duress` early-return AVANT le subscribe `http.rs:878-888` → le push placé dans le
> bras Ok est structurellement inatteignable sous duress ; aucun nouveau gate).
>
> **Pourquoi PLAN-ADAPT et non EXECUTE** : la **PRÉMISSE du brief** — « (a) seul laisse le browse vide
> parce que l'announce du pair est un événement PASSÉ que le gossip ne rejouera pas » — est **FALSIFIÉE
> par le code**. Sur `join_peers`, le pair émet `NeighborUp{A}` et **REJOUE SON OUTBOX** de
> project-announcements (`runtime.rs:1747-1783`, outbox chargé au boot `runtime.rs:1106`) → le nœud frais
> reçoit les apps **PUBLIÉES** du pair **SANS re-publish et SANS action du pair**. Le « RE-PUBLIER côté
> pair » observé à E2 est un **artefact d'ORDRE de test** (le PC a publié `zn0-acceptance` **APRÈS**
> s'être connecté, `sprint81_t2_e2_zero_n0.json:19`), pas une incapacité du gossip à rejouer. Trois
> conséquences d'adaptation : **(i)** le périmètre (b) du brief — « re-drive complet du boot driver / pull
> directories » — est **HORS-SCOPE E3**, à la fois par le critère du brief (non atteint : le browse n'est
> PAS vide) **et mécaniquement** (au 1er subscribe A n'a **aucun** locator `anchors.json` pour B →
> `repull_directories` **ne peut pas se déclencher**, `iroh_runtime.rs:1261`) ; **(ii)** l'unique résidu
> laissé par (a) = la classification « Tes sources » (`from_subscribed`) via le catalogue signé du pair,
> qui est un **résidu Phase C accepté** (annonce directory LIVE-ONLY, jamais dans l'outbox, jamais rejouée
> sur NeighborUp, `http.rs:1372-1388`) → **DÉCISION PO à ratifier** (recommandation ferme : HORS-SCOPE,
> carry) ; **(iii)** la suffisance de (a) est **code-SUPPORTÉE, pas encore runtime-PROUVÉE** (aucun test
> mesh 2-nœuds n'existe dans le crate, `runtime.rs:3062-3065`) → le **décideur** est le T2 LIVE
> hot-subscribe, à traiter comme **hypothèse-à-prouver-par-test**, jamais comme fait acquis.
>
> G8 : 5 scans (S1a API-gossip / S2 histoire-carry / S3 threat-duress / S4 wire-callsites / S5
> tests-greffe) + 5 vérifications adversariales. Bilan : **4 PLAN-ADAPT-local + 1 EXECUTE-local (S3)**,
> **0 REFUTED**, **0 DESIGN-CONFLICT**. Les réfutations adversariales priment et sont absorbées comme
> faits : premise-falsification calibrée (« hypothèse-à-prouver », pas « INFIRMÉE »), énumération (b)
> élargie (catalogue directory **ET** liste curateur, pas seulement `/nodes`), `parse_bootstrap` **privé
> non réutilisable** depuis `TopicSender` (mirror, pas reuse), duress by-construction à **verrouiller par
> test négatif**, résidu boot-duress pré-existant **hors-E3** routé G.

---

## 1. Le défaut + le périmètre demandé

**Symptôme LIVE (reproduction exacte, `sprint81_t2_e2_zero_n0.json:19-20`)** : nœud frais →
`POST /curators/subscribe {node_id du pair}` à chaud → browse vide 60 s+, **aucun dial**. Après
**RESTART** : `attention set restored from subscriptions.json count=1` + `gossip: subscribing to topic
(non-blocking) count=1` → dial part (discovery + relay) → announce du pair traverse → browse OK.

**Le brief délimite deux items à trancher** :
- **(a)** le join gossip à chaud du pair fraîchement souscrit — **OBSERVÉ, cœur de la demande PO** ;
- **(b)** le re-drive complet du boot driver (pull directories) sur ingest/subscribe — **carry S75 plus
  large** ; in-scope **si** le fix (a) seul laisse le browse vide (announce = événement passé non rejoué).

**Barre UX cible (contrainte PO)** : « je m'abonne à un nœud → je vois ses apps **sans redémarrer et
sans action du pair** ».

**Note d'attribution corrigée** : le champ `residual` de `sprint81_t2_e2_zero_n0.json:20` **mislabelise**
le défaut en « S75 carry re-drive-on-ingest of the one-shot boot driver ». Ce sont **deux one-shots
distincts** (§2.3). Le vrai défaut est le **one-shot du bootstrap gossip** (`runtime.rs:1105`), pas le
boot **seed** driver (`http.rs` `run_boot_seed_driver`). E3 corrige le premier ; le second reste un carry
orthogonal (§8).

---

## 2. Le vrai périmètre, item par item (evidence-adossé, réfutations adversariales appliquées)

### 2.1 (a) IN-SCOPE — cœur E3 : hot-join gossip. Root cause + preuve de suffisance

**Root cause CONFIRMÉE au code** (5 scans concordants, vérifiée source cette session) :

| Fait | Preuve |
|---|---|
| `subscribe_curator` ne produit **aucun** effet gossip | `http.rs:889` : `match state.curator_runtime.subscribe(&req.curator_pubkey_hex)` → bras `Ok(_)` (`:890-896`) sans `gossip_cmd_tx.send` |
| `subscribe()` ne mute que l'attention-set + persiste | `iroh_runtime.rs:675-691` : `parse_pubkey_hex` → `attention.insert` → `persist_subscriptions`, **0 push gossip** |
| `bootstrap_peers` = snapshot boot-only lu **1×** | `runtime.rs:1105` `let bootstrap_peers = curator_runtime.subscribed_pubkeys_hex();` → champ `GossipTaskConfig` (`:1503/1530`) → consommé **une** fois `runtime.rs:1548` `gossip.subscribe_topic(topic_id, bootstrap_peers)` |
| `GossipCmd` n'a **aucun** verbe join | `runtime.rs:1482-1489` : `enum GossipCmd { Outbox(..), RequestBrowse }` — le doc `:1480` annonce pourtant déjà « from HTTP handlers **or the curator subscribe endpoint** » |

**Suffisance de (a) — la prémisse du brief est falsifiée par le code** (mais calibrée) :
sur `join_peers([B])`, A dial B ; B reçoit `NeighborUp{A}` et **rejoue son outbox** de
project-announcements (`runtime.rs:1747-1783` : boucle `for stored in &outbox` → `remint_and_wrap_for_replay`
→ `sender.broadcast(fresh)`), outbox chargé au boot (`runtime.rs:1106`, persistant même isolé). Le nœud
frais A **reçoit et affiche** les apps publiées de B **sans re-publish, sans action du pair**. Le
handler d'ingest project-announcement n'est **pas** subscription-gaté (test à vide `CuratorRuntime::new(None)`
→ entrée `Reachable`, cité par S1a/S4/S5) → la visibilité de base dépend de la **connectivité mesh** (que
`join_peers` restaure), pas de l'attention-set.

**⚠ Calibration adversariale (S1a-corr, S4-corr) — NE PAS sur-affirmer** : « (a) suffit » est
**code-SUPPORTÉ, pas runtime-PROUVÉ**. (i) Aucun test mesh 2-nœuds gossip n'existe dans le crate
(`runtime.rs:3062-3065` : « NO 2-node NeighborUp test exists in this crate ») ; (ii) le récit LIVE E2 est
ambigu (le re-publish observé s'explique par l'ordre de test, mais ce n'est pas prouvé négativement). →
**Traiter la suffisance comme hypothèse-à-prouver par le T2 LIVE hot-subscribe** (§5), pas comme fait
acquis.

**API root-cause disponible (iroh-gossip 0.101, vendored)** :
`GossipSender::join_peers(&self, peers: Vec<EndpointId>)` (`api.rs:192-195`) → `Command::JoinPeers(peers)`
(`api.rs:382`) → membership HyParView → dial via `endpoint.connect` (routé par la discovery de l'endpoint).
`GossipSender` est `#[derive(Clone)]` (`api.rs:171-172`). **C'est le primitive exact** : join à chaud sur un
topic **déjà souscrit**, sans ré-souscription, idempotent (dedup `is_pending` ; re-join d'un pair actif =
no-op membership).

### 2.2 (b) HORS-SCOPE E3 — « re-drive du boot driver / pull directories ». Tranché OUT, double evidence

Le brief conditionne (b) à « (a) seul laisse le browse vide ». **Réfuté** (§2.1). En sus, (b) est
**mécaniquement inapplicable** au 1er subscribe :

- `repull_directories` est gaté sur des **locators `anchors.json` persistés** ET `is_subscribed`
  (`iroh_runtime.rs:1255-1261`). Au **1er** subscribe à chaud, A n'a **aucun** locator pour B (le locator
  n'est écrit qu'**après** ingest d'une annonce directory de B) → `repull_directories` **ne peut pas se
  déclencher**, même si on le re-drivait. Chicken-egg : pas de pull sans locator, pas de locator sans
  annonce directory reçue.
- Le carry S75 « re-drive-on-ingest » vise le **boot SEED driver** (`run_boot_seed_driver`,
  `http.rs` ; acquisition/pin des apps **keep_online de CE nœud**), re-drivé à l'**ingest d'un annuaire
  couvrant**. Il concerne les **PROPRES apps du nœud**, **PAS** « j'ai souscrit à un pair » → **orthogonal**
  à E3 (S2-F8, S4-B-CARRY confirmés).

**Conclusion** : (b) tel que formulé (pull directory re-drive) est **HORS-SCOPE E3** — non déclenchable
sur un subscribe frais + orthogonal au défaut observé. Reste **carry S75** (overdue, §8).

### 2.3 Résidu unique de (a) — classification « Tes sources » (`from_subscribed`). DÉCISION PO

Après (a), les apps publiées de B atterrissent en **« Découvert sur le réseau »** (project-announcement
direct), **pas** en **« Tes sources »** (`from_subscribed`). Raison, evidence-adossée :

- L'annonce **NodeDirectory** (le catalogue signé Ed25519 qui alimente `from_subscribed`,
  `http.rs:943-956`) est **LIVE-ONLY** : « unlike the project announce path this does NOT persist to the
  outbox » (`http.rs:1372-1388`, verbatim) → **jamais rejouée** sur `NeighborUp` (l'outbox ne contient que
  des project-announcements). Le commentaire du code **concède déjà** : « A subscriber that joins LATER
  still needs a live overlap … no outbox replay for directory announcements — **accepted residual of the
  Phase C deferral closure** » (`http.rs:1385-1388`).
- **Énumération élargie (S1a-missed)** : ce résidu couvre **DEUX** payloads non-outbox, pas seulement
  `/nodes` : (i) le `NodeDirectoryEntry` (catalogue → `from_subscribed`/`/nodes`) ET (ii) la
  `CuratorListEntry` (liste curatée signée). Puisque l'endpoint est littéralement `curators/subscribe`, la
  question « est-ce que je vois la LISTE curatée du pair à chaud ? » appartient à la même décision.

**Recommandation ferme = HORS-SCOPE E3, carry** (raisons) :
1. Le **symptôme OBSERVÉ** était « browse **vide**, aucun dial » — **pas** « mauvaise section ». (a) ferme
   le symptôme observé ; les apps **sont visibles et joignables**, seul le **label de section** diffère.
2. Fermer le résidu directory à chaud exige une **surface gossip NEUVE** substantiellement plus grande que
   le défaut observé et **ré-ouvre le déferrai Phase C** : soit (i) un nouveau verbe gossip « request
   directory » (B ré-émet son directory signé à la formation de voisinage), soit (ii) ajout du directory à
   un chemin de replay NeighborUp côté producteur, soit (iii) B pull le catalogue de A au hot-subscribe
   (chicken-egg locator). Aucune n'est « root-cause du défaut observé » ; toutes sont un item Phase-C à
   part entière.
3. La barre UX « je vois ses apps » est **satisfaite par « Découvert »** : l'app est rendue et fetchable.

**→ UNIQUE décision PO à ratifier** (§4 « décisions restantes ») : « je vois ses apps » = **« Découvert »
suffit** (recommandé, E3 ship (a) seul) **OU** exige **« Tes sources »/liste curatée à chaud** (alors E3
absorbe le résidu directory Phase C — périmètre matériellement plus large, plan à ré-instruire). Défaut
retenu par ce préflight : **« Découvert » suffit**.

### 2.4 Symétrie inverse (unsubscribe) — HORS-SCOPE E3, tranché avec evidence

`iroh-gossip 0.101` **n'expose AUCUN verbe leave/remove-peer** : `Command` = `Broadcast` /
`BroadcastNeighbors` / `JoinPeers` **uniquement** (`api.rs:376-383`, vérifié cette session ; le
teardown d'un topic se fait en **droppant** la souscription, pas via une commande). Donc
`unsubscribe_curator` (`http.rs:903-918` → `iroh_runtime.rs:701-734` droppe attention + `directories` +
`anchor_locators`) **ne peut pas** forcer la chute du voisin gossip ajouté par `join_peers`. Le pair reste
voisin HyParView jusqu'au **churn** ; le **gate ingest LIVE** (`is_subscribed=false`, `iroh_runtime.rs:751`
+ les gates fetch `:1029`/`:1147`) **droppe déjà** ses messages → **fuite bornée au transport, pas à
l'ingest**. Aucun fix propre n'existe (pas d'API). **Documenter uniquement** (carry G, §8). E3 couvre
**subscribe → join** seulement — asymétrie assumée et honnête.

---

## 3. Le fix root-cause — 4 edits localisés (câblage, corrections adversariales intégrées)

**Chokepoint** : le `gossip_cmd_tx` existe **déjà** dans `DaemonHttpState` (`http.rs:93`), câblé
`runtime.rs` et **déjà drivé par un handler HTTP** (`browse_pull` → `GossipCmd::RequestBrowse`,
`http.rs:1062-1065`). Le transport est **zéro-nouveau**.

**Edit 1 — `crates/nexus-core-rs/src/gossip.rs` : `TopicSender::join_peers`** (additif ; `TopicSender` est
`Clone`, broadcast-only aujourd'hui `:497-505`) :
```
pub async fn join_peers(&self, peers: Vec<String>) -> Result<()>
```
qui **parse chaque hex → `PublicKey` par-pair, en SKIP+log les mauvais** (surtout **PAS** un
`collect()`-abort), puis délègue à `self.inner.join_peers(parsed).await`.
- **Correction adversariale (S1a/S2/S3/S4/S5 concordantes)** : `parse_bootstrap` (`gossip.rs:431-439`) est
  une **assoc-fn PRIVÉE de `GossipClient`**, **PAS** de `TopicSender` → **non réutilisable** telle quelle.
  Mirror le parse (ou hoister en free-fn `pub(crate)`), **ne pas** supposer `self.parse_bootstrap`.
- **Robustesse hot-path (S1a-missed P2)** : `parse_bootstrap` au boot **collect-abort** (1 hex mauvais
  casse tout le subscribe). Le wrapper hot **doit dégrader** (log + skip), jamais propager/paniquer. NB :
  via `subscribe_curator` le pubkey poussé est **déjà validé** (`subscribe()` fait `parse_pubkey_hex`
  d'abord, `iroh_runtime.rs:675`) donc toujours parseable ; le skip-par-pair est de la **défense en
  profondeur** pour tout appelant batch futur.
- **Type-compat** : `iroh::PublicKey == EndpointId` (subscribe passe déjà `Vec<PublicKey>` à
  `inner.subscribe(Vec<EndpointId>)` et ça compile, `gossip.rs:425`) → `join_peers(Vec<PublicKey>)`
  type-check à l'identique.

**Edit 2 — `crates/nexus-shell-daemon/src/runtime.rs` : variante + bras** :
- variante interne `GossipCmd::JoinPeers(Vec<String>)` à l'enum (`:1482-1489`) ;
- bras dans le select `cmd_rx.recv()` **à côté de `Outbox` (`:1804`) / `RequestBrowse` (`:1837`)** :
  `Some(GossipCmd::JoinPeers(peers)) => { if let Err(e) = sender.join_peers(peers).await { debug!(error=%e, "join_peers failed"); } }`.
  `sender` (le `TopicSender` splitté `:1556`) est en scope. **Semantique send** : logger l'erreur, ne pas
  paniquer (miroir des bras existants).

**Edit 3 — `crates/nexus-shell-daemon/src/http.rs` : push dans `subscribe_curator`** (bras `Ok(_)`
`:889-896`, **après** l'early-return duress `:878-888`) :
```
Ok(_) => {
    let _ = state.gossip_cmd_tx
        .send(crate::runtime::GossipCmd::JoinPeers(vec![req.curator_pubkey_hex.clone()]))
        .await;   // best-effort, miroir browse_pull:1062-1065
    (StatusCode::OK, Json(SubscriptionsResponse { subscribed_curators: state.curator_runtime.subscribed_pubkeys_hex() })).into_response()
}
```
- **Correction sémantique (S5-corr, S4-M2)** : `join_peers` dial par **EndpointId** ; le handler pousse le
  `curator_pubkey_hex` souscrit. Dialable **ssi** la clé souscrite EST l'endpoint-id du pair — **VRAI dans
  le cas observé** (`{node_id du pair}`) et **cohérent avec le boot** (`bootstrap_peers` parse déjà ces
  mêmes hex comme EndpointIds). Pour une **pure clé-curateur signante** non-joignable, `join_peers` est un
  best-effort qui **échoue silencieusement** — exactement comme le bootstrap boot aujourd'hui → **0
  régression**.
- **Idempotence (S1a/S5-missed)** : `subscribe()` renvoie `Ok` même sur re-subscribe (idempotent 200,
  `http.rs:867`) → le push part à **chaque** appel, y compris no-op. **Inoffensif** (join_peers dedup
  `is_pending` + no-op membership pour un pair actif). `subscribe()` n'expose pas « added vs present » → ne
  **pas** conditionner (hors scope root-cause) ; documenter.

**Edit 4 — tests** (§5).

**Bilan câblage** : 0 champ persisté nouveau, `subscriptions.json` inchangé, pins intacts.

---

## 4. Contraintes duress (par construction, à verrouiller par test)

1. **Placement canonique NON-NÉGOCIABLE** : le push `GossipCmd::JoinPeers` vit **dans le bras `Ok(_)`**
   de `subscribe_curator`, **après** l'early-return `curator_subscribe_in_duress == Noop`
   (`http.rs:878-888` → `noop_identity.rs:117-122` : `Duress → Noop`, return 200 liste-vide **avant** tout
   `subscribe`). Sous duress, le code **n'atteint jamais** le bras Ok → **0 `JoinPeers`, 0 dial nouveau**
   sous la clé leurre. Le handler est le **SEUL producteur** de `JoinPeers` (S3-NO-REMOTE-TRIGGER : les
   seuls call-sites prod de `subscribe` sont `http.rs:889` loopback + `runtime.rs:438` boot-config ;
   l'ingest gossip est subscription-gaté et **n'ajoute jamais** d'abonnement → aucune surface distante
   n'induit un subscribe/join).

2. **Pas de gate consommateur (décision minimale, cohérente avec le précédent)** : le bras `RequestBrowse`
   in-task **n'est pas** duress-gaté (gating uniquement au bord HTTP, `browse_pull:1054`). `JoinPeers`
   suit **le même contrat** : gaté au bord HTTP (via l'early-return subscribe), non-gaté in-task.
   Threader `identity_mode` dans `GossipTaskConfig` (qui **n'a pas** ce champ, `runtime.rs:1495-1516`)
   serait un changement plus large que le root-cause. **Contrepartie obligatoire (S4-M4)** : verrouiller
   l'invariant « seul producteur » par (i) un **commentaire** au push + au bras, et (ii) un **test négatif
   duress** (§5) — sinon la sûreté by-construction n'est pas régression-lockée.

3. **Symétrie boot = sens sûr (S3-BOOT-SYMMETRY)** : le boot `subscribe_topic` (`runtime.rs:1548`) dial
   **déjà** tout le set d'abonnés réels sous la clé leurre (non duress-gaté ; `subscriptions.json` = vrai
   data-dir, le duress ne swap que la clé nœud). La join à chaud d'**un** pair est un **sous-ensemble
   strict** de ce que le boot dial déjà → E3 n'introduit **aucun** canal de dial que le boot n'a pas, et
   sous duress E3 dial **0** (Noop) vs le boot qui dial N. Aucun **nouveau fingerprint** distant (l'HTTP
   renvoie toujours 200 liste-vide, `http.rs:881-887`).

4. **Résidu boot-duress PRÉ-EXISTANT, hors-E3 (S3-missed, flag avec doute)** : le dial du boot subscribe +
   le fetch `repull_directories` + le **replay outbox curateur** sous duress ne sont **PAS** couverts par
   `DURESS-BOOT-LEAK` §15.1 (qui ne gate que le wire-**emit** feed/seed). E3 **ne l'aggrave pas** (reste
   duress-safe-par-placement) → verdict inchangé, mais **router G** (§8) : si le PO décide un jour de gater
   l'activité réseau boot sous duress, threader `identity_mode` dans `GossipTaskConfig` sera requis.

---

## 5. Plan de tests (CONTROL/GREEN, groupe nextest, delta chiffré)

**Harness existant** : l'injecteur `mk_state_with_mode_tx(mode, gossip_cmd_tx)` (`http.rs:4371-4399`)
donne au test le `rx` pour asserter **ce que le handler a poussé** ; pattern d'assertion `GossipCmd`
déjà établi (drain `cmd_rx`). Le test duress `daemon_boot_in_duress_mode_rejects_curator_subscribe_real`
(`http.rs:7688+`) est le point d'extension (passer à la variante `_tx` pour asserter le **vide**).

| # | Test | Type | Assertion | Garde | Statut |
|---|---|---|---|---|---|
| 1 | `subscribe_curator` Normal pousse JoinPeers | **CONTROL→GREEN** | RED pré-fix : `rx.try_recv() == Empty` ; GREEN post-fix : exactement `GossipCmd::JoinPeers([pubkey])` | hermétique, 0 réseau, injecteur `_tx` | **MANDATORY / BLOQUANT** |
| 2 | `subscribe_curator` Duress → **rien** | GREEN (négatif) | Duress → early-return → `rx` **EMPTY** (verrouille le by-construction) | hermétique | **MANDATORY** |
| 3 | `subscribe_curator` hex invalide → **rien** | GREEN (négatif) | `subscribe()` `parse_pubkey_hex` échoue → bras `Err` → aucun push | hermétique | **MANDATORY** |
| 4 | `TopicSender::join_peers` core | GREEN | topic single-node isolé → `Ok` (enqueue `Command::JoinPeers`) ; batch avec 1 hex garbage → **skip+log**, pas de panic/abort, `Ok` | hermétique, `nexus-core-rs` | **MANDATORY** |
| 5 | Convergence hot-subscribe 2-nœuds LIVE | **T2 artefact** | B souscrit A à chaud → dial → `NeighborUp` → replay outbox de B → `/browse` de A contient l'app de B **SANS restart, SANS re-publish** ; `b3` byte-integrity PASS | **RIG-gated → `RIG-ABSENT` traçable** | **LIVE — le DÉCIDEUR de suffisance** |

**Explicitement NE PAS ajouter (zombies / anti-patterns du crate)** :
- **PAS** de test mesh 2-nœuds gossip in-process committé : le crate l'**interdit** comme flaky
  (`runtime.rs:3062-3065`). La convergence réelle = **T2 LIVE**, jamais un unit test committé.
- **PAS** de control temporel naïf « browse reste vide après délai borné » (négatif temporel fragile,
  classe Phase C) → remplacé par le **channel-emptiness déterministe** (#1 RED).
- **PAS** de test task-level « JoinPeers persiste un side-effect » : contrairement à `Outbox` (persist DB
  observable), `JoinPeers` sur un nœud isolé n'a **aucun** side-effect observable → limité à la
  **parse-résilience** (couverte par #4).

**Groupe nextest** : la suite committée est **channel-boundary hermétique** (http.rs) + **core** (gossip.rs)
→ **hors** du groupe `two-node-convergence` (`.config/nextest.toml`, max-threads=2, 60 s, filtre
`test(/(convergence_|…)/)`). **Ne PAS nommer** un test committé `convergence_*` (le ferait tomber dans le
groupe capé + risque de flakiness délibérément évité). Un éventuel test **primitive networked** dans
`nexus-core-rs` (join_peers → NeighborUp → delivery, pattern seed_addr `multi_thread`) prouverait le
mécanisme mais reste **classe env-bloquée**, **jamais** le gate hermétique.

**Delta chiffré** : **E3 code = +4 tests hermétiques Rust** (#1 http Normal-push, #2 http Duress-empty,
#3 http invalid-hex-empty, #4 core wrapper) **+ 1 artefact T2 LIVE** (`sprint81_t2_e3_hot_subscribe.json`,
`PASS`/`BLOCK{diagnosis}`/`RIG-ABSENT`). **Optionnel : +1 primitive networked** (env-bloquée, non-gate).
**web = 0** (contrat HTTP `SubscriptionsResponse` byte-identique ; `daemon.ts:596 addAnchor` modélise déjà
le subscribe async). **−0 zombies.**

---

## 6. Invariants & Day-0 (tenus)

- **0 bump wire** — `GossipCmd` = `tokio::sync::mpsc` **interne process** (`runtime.rs:1493`), jamais
  sérialisé ; `Command::JoinPeers` = commande **actor iroh in-process** (irpc local ; le `Serialize`
  dérivé n'est **pas** une surface wire SBFB — la membership HyParView traverse le réseau via le protocole
  **inchangé** d'iroh-gossip). Aucune touche `FeedEntry` / `ProjectAnnouncement` / `NodeDirectoryEntry` /
  enveloppe PoW / `*_FORMAT_VERSION` / `DOMAIN_*_V1`.
- **iroh strictement seul** — `join_peers` est de la surface **par défaut** d'iroh-gossip 0.101 (pas de
  feature, pas de dev-dep). **0 dep runtime neuve.** Pins `=1.0.1` / gossip `=0.101.0` intacts.
- **Verrous S74/S75** — E3 touche **uniquement** la membership gossip (dial d'un pair déjà souscrit).
  Aucune touche seed accept-list / subscription-gating (verrou-5) / directory ingest / app count / verrou-3
  `[seed]` vide / verrou-4 seeder≠auteur / invariant héberger≠publier. Le gate LIVE `is_subscribed` reste
  la seule autorité d'ingest.
- **presets::N0 + zéro-n0 E2 orthogonaux** — E3 ne touche pas la discovery/relais (E2) ; il déclenche un
  dial qui **emprunte** la discovery active (n0 ou self-hosted zéro-n0) — donc l'**efficacité** de (a) est
  **couplée** à la discovery qui marche (S4-M1). Corollaire acceptance : le T2 hot-subscribe doit tourner
  sur le stack discovery **disponible** (n0 pré-EOL, ou E' self-hosted) ; E3 n'ajoute **aucune** capacité
  discovery, il restaure la **parité** avec le dial du boot.
- **Duress** — non re-gaté (§4), by-construction + test négatif. Toolchain 1.94.

---

## 7. Risques résiduels

- **Suffisance de (a) non runtime-prouvée avant le T2 (P1)** — code-supportée seulement ; le T2 LIVE
  hot-subscribe est le **décideur BLOQUANT**. Si le T2 montre que le browse reste vide **malgré** le dial
  (ex. outbox de B vide à l'instant, race receiver-readiness), rouvrir l'analyse — mais **pas** vers (b)
  pull-directory (mécaniquement inapplicable) : ce serait une race de timing, pas un manque de replay.
- **Reconnexion-après-drop (P2, S5-missed)** — `join_peers` connecte **maintenant** mais **n'ajoute pas**
  le pair au bootstrap-set du topic (figé à `subscribe_topic`, `runtime.rs:1548`). Si le lien B↔A tombe
  mid-session, B ne se re-bootstrappe sur A qu'au **reboot** (re-lecture `subscriptions.json`). Résidu du
  fix (a)-seul, pertinent au **design de la fenêtre T2** (un lien flaky peut re-vider le browse). Hors
  root-cause du défaut observé (= le 1er dial), mais à **noter dans le runbook T2**.
- **Idempotence re-subscribe (P3)** — push à chaque appel, y compris no-op ; inoffensif (dedup HyParView).
  Documenté, non-conditionné (subscribe n'expose pas added-vs-present).
- **Résidu directory Phase C non fermé à chaud (P2 → carry)** — « Tes sources »/liste curatée n'arrive pas
  à chaud (§2.3) ; assumé si PO ratifie « Découvert suffit ».
- **Résidu boot-duress pré-existant élargi (P2 → carry G)** — dial/fetch/replay-outbox curateur sous duress
  au boot (§4-4) ; E3 ne l'aggrave pas ; router G avec doute explicite (peut être une décision de scope
  S75 acceptée post-pivot PULL).

---

## 8. Carries sortants (E3 → G, S75)

1. **G (THREAT_MODEL / doc, §15.x)** : (a) **asymétrie unsubscribe** — pas de verbe leave iroh-gossip ; le
   pair reste voisin HyParView jusqu'au churn, ingest droppé par `is_subscribed=false` → fuite transport
   bornée, documenter ; (b) **résidu boot-duress pré-existant** (dial subscribe + fetch repull + replay
   outbox curateur sous clé leurre) non couvert par `DURESS-BOOT-LEAK`, hors-E3, avec doute explicite ;
   (c) note « hot-join curateur duress-safe-par-placement, hérite de l'early-return
   `curator_subscribe_in_duress`, 0 dial nouveau sous duress » ; (d) **reconnexion-après-drop** = résidu
   (a)-seul (join sans ajout au bootstrap-set).
2. **Carry S75 — `re-drive-on-ingest` du boot SEED driver (OVERDUE 3/3)** : distinct du défaut E3
   (orthogonal, §2.2). Escaladé 3/3 « MANDATORY S78 » puis différé Factory-first → **à fermer explicitement
   ou re-justifier « blocker externe »** au prochain audit gate. E3 ne le ferme PAS et ne doit pas prétendre
   le faire.
3. **Carry résidu directory Phase C** — « Tes sources »/liste curatée à chaud (§2.3) : si PO ratifie
   « Découvert suffit », reste résidu accepté ; sinon = item Phase-C à ré-instruire (nouveau verbe gossip
   request-directory OU replay directory côté producteur).

---

## 9. Restitution des scans (fan-out 5 + adversarial)

| Scan | Verdict-local | Findings clés retenus (après adversarial) | Adversarial |
|---|---|---|---|
| **S1a** API-gossip | **PLAN-ADAPT** | root cause exact ; `join_peers` = primitive à chaud correcte (supérieure à ré-souscription) ; wrapper gap ; `GossipCmd::JoinPeer` ; duress-safe ; zéro-n0 par construction ; **(a) suffit pour la carte Browse** ; (b) = catalogue directory hors-outbox | 12 CONFIRMED. Corr : **suffisance calibrée « hypothèse-à-prouver », pas INFIRMÉE** ; `parse_bootstrap` privé (mirror≠reuse) ; citation test `http.rs:3056`→`runtime.rs:3056`. **Missed** : énumération (b) **élargie** (liste curateur + `/nodes`) ; robustesse parse hot-path ; idempotence ; race boot-window sûre |
| **S2** histoire-carry | **PLAN-ADAPT** | root cause unique ; primitive amont existe ; type-compat 0-friction ; câblage GossipCmd ; duress by-construction ; **prémisse « (a) vide » falsifiée** (artefact d'ordre E2) ; directory non-delivered ; **carry re-drive orthogonal + OVERDUE 3/3** ; pas d'API leave ; convention test ; 0 wire | 12 CONFIRMED, 0 refuted. Corr : `parse_bootstrap` privé (hoist/mirror) ; drift ligne S75 `:1718`→`:1779` ; `Command` dérive Serialize mais transport in-process. **Missed** : test négatif duress explicite ; précondition discovery-resolvable ; hypothèse NeighborUp HyParView |
| **S3** threat-duress | **EXECUTE** | duress-correct par placement (push dans bras Ok après early-return) ; symétrie boot sens-sûr ; pas de trigger distant ; re-drive borné 1-fetch ; API sans bump wire ; doc THREAT §15 | 8 CONFIRMED. Corr : ligne `986-988`→`981` ; ce qui est stocké = `TopicSender` wrapper (pas raw `GossipSender`) ; `handle_project_announcement` **non** subscription-gaté (support la suffisance). **Missed** : **résidu boot-duress élargi** (replay outbox curateur sous duress, pré-existant, hors-E3, flag doute) ; preuve clé-hex parseable en EndpointId |
| **S4** wire-callsites | **PLAN-ADAPT** | attention-set lu LIVE partout **sauf** membership gossip → fix isolé ; API `join_peers`→RequestJoin HyParView ; 0 wire ; **(a) suffit** (announcement non-gaté) ; (b) carry ; nudge inutile ; test 2-nœuds absent ; unsubscribe hors-scope | 11 CONFIRMED. Corr : **suffisance OVERSTATED → hypothèse-à-prouver-par-test** ; citation `:1147` (directory-gate) mal-labellée → chemin project **non-gaté** `:2314` + test à-vide `:3160-3211` ; nudge « race receiver » **faux** (récepteurs attachés au boot) ; « Quit » n'existe pas (drop topic). **Missed** : couplage discovery ; clé-curateur vs endpoint-id ; idempotence ; **duress defense-in-depth by-construction seul** ; error-handling du bras |
| **S5** tests-greffe | **PLAN-ADAPT** | root cause + API iroh-native ; 0 wire ; duress-safe ; **CONTROL déterministe channel-boundary** (pas mesh flaky) ; T2 LIVE = vrai décideur ; scope-fork (a)/(b) → PO ; front = 0 ; carry unsubscribe | 15 CONFIRMED. Corr : `parse_bootstrap` privé (mirror, pas reuse) ; sémantique dial-par-EndpointId + cas « valid-hex-mais-injoignable » benign ; contrat « HTTP-gated, task-unchecked » à expliciter. **Missed** : espace-solution (b) non énuméré si PO dit IN ; idempotence ; single-push-point vérifié non-gap ; reconnexion-après-drop ; 0-wire sous-argumenté (membership traverse le réseau, inchangée) |

**Convergence** : 5 scans → **PLAN-ADAPT global** (4 PLAN-ADAPT-local + S3 EXECUTE-local). **0 REFUTED, 0
DESIGN-CONFLICT.** Le fix (a) est EXECUTE-ready ; l'adaptation est **triple** : premise-falsification
calibrée (suffisance = hypothèse-à-prouver), délimitation (b) OUT (double evidence) + résidu directory =
1 décision PO, stratégie de test adaptée (channel-boundary hermétique + T2 LIVE, jamais mesh in-process).

---

## 10. Commit shape (indicatif — E3)

`fix(daemon): Sprint 81 Phase E3 — hot-join gossip du curateur souscrit (join_peers à chaud, 0 bump wire)`

Body : **root cause** — `subscribe_curator` mutait l'attention-set sans notifier la tâche gossip vivante,
`bootstrap_peers` lu 1× au boot (`runtime.rs:1105→1548`) → pair souscrit à chaud jamais dialé jusqu'au
restart. **Fix** — `TopicSender::join_peers` (nexus-core-rs, mirror `parse_bootstrap` **par-pair
tolérant**) + variante interne `GossipCmd::JoinPeers(Vec<String>)` + bras select loop (miroir
Outbox/RequestBrowse) + push depuis le bras `Ok` de `subscribe_curator` via `gossip_cmd_tx` existant
(**après** l'early-return `curator_subscribe_in_duress` → duress-safe par construction, **0 dial nouveau
sous duress**, seul-producteur verrouillé par test négatif). Sur `join_peers`, le pair rejoue son outbox
sur `NeighborUp` → **browse peuplé sans restart et sans action du pair** (prémisse « (a) vide »
falsifiée ; re-publish E2 = artefact d'ordre de test). **HORS-SCOPE tranché** : (b) pull-directory
re-drive (mécaniquement inapplicable au 1er subscribe, pas de locator ; + carry S75 boot-SEED-driver
orthogonal OVERDUE) ; « Tes sources »/`from_subscribed` à chaud = résidu Phase C accepté (carry) ;
symétrie unsubscribe = pas d'API leave iroh-gossip (carry G). **Tests** : +4 hermétiques (Normal-push /
Duress-empty / invalid-hex-empty / core wrapper) + artefact T2 LIVE hot-subscribe
(`sprint81_t2_e3_hot_subscribe.json`, décideur de suffisance, RIG-gated `RIG-ABSENT` traçable). **0 bump
wire** (`GossipCmd` mpsc interne + `Command::JoinPeers` actor in-process, membership HyParView inchangée),
**iroh strictement seul** (surface API-défaut 0.101, 0 dep), pins `=1.0.1`/`=0.101.0` intacts, verrous
S74/S75 5/5, toolchain 1.94, web = 0. Carries G (unsubscribe + résidu boot-duress pré-existant +
reconnexion-après-drop) / S75 (re-drive boot-seed OVERDUE) / Phase-C (résidu directory).
