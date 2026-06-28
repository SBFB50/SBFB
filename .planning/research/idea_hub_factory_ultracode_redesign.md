# Idea Hub × Factory × Ultracode — Redesign confiné en couches (note de recherche)

**Date :** 2026-06-28
**Statut :** NOTE DE RECHERCHE — Cas C en préparation. RIEN n'est engagé : aucun code, aucun wire, aucune phase. Document destiné à `.planning/research/idea_hub_factory_ultracode_redesign.md`, à transformer en kickoff lors d'un futur sprint.
**Méthode :** consolidation d'une carte de code (7 aires, file:line) + 6 dimensions de conception A→F passées au crible d'un panel sceptique (verdicts intégrés ci-dessous : les 6 sont revenues `NEEDS-CORRECTION`, aucune `CONFLICTS-FROZEN`). Le présent document RESTITUE les conceptions corrigées, jamais les versions naïves.
**Horizon :** v2.1, pari mainteneur-solo-augmenté (micro-noyaux 1-3 humains), commun humaniste non-monétaire.

---

## 1. Résumé exécutif + VERDICT de confinement

### 1.1 Ce que la recherche établit

Le projet possède déjà ~70-80% des briques pour un idea hub refait, un claim signé propagé, un onboarding compute, et un pipeline ultracode in-app. Le travail neuf est minoritaire et bien circonscrit. MAIS la recherche a remonté **un défaut structurel qui domine tout le reste** et qu'on présentait à tort comme résolu : **le substrat de convergence cross-nœud est cassé des deux côtés (doc iroh-docs ET feed public).** Tant que ce point n'est pas réglé, toute mesure de funnel cross-nœud est `PROVISIONAL`.

### 1.2 VERDICT de confinement (réponse directe à « regarde nos research »)

Le confinement n'est PAS la binaire « sandbox vs Operator » — la recherche fait apparaître **TROIS tiers** load-bearing, et la frontière de handoff entre le commun et le privilégié est **une donnée, jamais un flux de contrôle** :

1. **LE COMMUN** = app SBFB sandboxée, classe Viewer, zéro privilège, zéro clé. Confinée par `BLOB_SERVE_CSP` (`connect-src 'none'`, `csp.rs:33` VÉRIFIÉ) + iframe `sandbox="allow-scripts"` SANS `allow-same-origin` (origine opaque, `BrowsedProject.tsx:599-609`). Le bridge postMessage est le seul canal (`protocol.ts:20-49`, 16 méthodes). **L'app PROPOSE ; elle ne signe jamais.**

2. **LE TIER INTERMÉDIAIRE** = front shell de confiance (loopback, authed bearer/cookie) : « partager mon GPU » (`consent.rs:151-242`, `Network.tsx:154-326`), « brancher mon LLM » (`worker.toml`). NI sandbox NI Operator. Le brief rangeait ces capacités dans « Operator privilégié » — le code dit qu'elles vivent dans le front shell privilégié servi par le daemon.

3. **LE PRIVILÉGIÉ-LOCAL** = l'Operator (`operator_server.rs:230-241` bind `127.0.0.1`, CSP control-center `:349-361`, auth 3-contrôles `auth.rs:307-357` + cookie HttpOnly per-boot). Seul territoire qui spawn des agents réels (PTY `claude`, `terminal.rs:69-89`), écrit des artefacts, lit gates/diff/sprint-history.

**Le handoff sandbox → Operator n'existe pas et ne doit pas exister.** Triple impossibilité VÉRIFIÉE : (i) `connect-src 'none'` coupe tout fetch/WS/SSE depuis l'app ; (ii) le dispatcher bridge route TOUS les case vers `coordUrl`/daemon, JAMAIS vers l'Operator (`useBridge.ts:235-412`) ; (iii) l'Operator est un axum séparé hors daemon avec auth propre et cookie cross-port non joignable. Au-delà de l'impossibilité technique, c'est **indésirable** : une app sandboxée ne doit jamais déclencher une exécution privilégiée (« le hub PROPOSE, n'arbitre jamais »).

**Donc le handoff est une PRIMITIVE DE DONNÉE :** un claim signé (op feed typée, content-adressé `idea_id = blake3`), que l'Operator INGÈRE comme **évidence** (jamais commande) dans le flux context-pack déjà là (`operator_server.rs:540-613`, `chat_history_authoritative:false`), après quoi l'humain pilote une vraie session agent. Le seul affordance shell admis est un **hint passif de navigation humaine** vers l'Operator (0 token, 0 payload, 0 autorité), modèle `OnboardingEmpty` « le shell n'est pas un spawner de process ». REJET explicite : deep-link-qui-lance, route shell-qui-lance, méthode bridge ciblant l'Operator.

### 1.3 Les 4 caveats d'honnêteté brutale (à lire avant tout enthousiasme)

- **PROVISIONAL — convergence cross-nœud :** le feed public NE converge PAS « gratuitement par gossip ». `boot_feed_namespace` (`runtime.rs:2555-2633`) est le **miroir ligne-à-ligne** du `boot_storage_namespace` buggé : branche `None ⇒ create_doc()` (`runtime.rs:2617-2625`), donc chaque nœud frais fabrique un `NamespaceId` disjoint. La convergence exige un échange MANUEL de ticket (`feed_ticket` GET + `feed_join` POST, `http.rs:386-388`). Le carry S75 le confirme empiriquement : `SeedAnnounced` (une op feed) ne convergeait PAS à l'acceptance (`peer_count:0`, PULL-3). **Porter le funnel sur le feed ne rend PAS le bug namespace « sans objet » ; le feed hérite du MÊME défaut.** La distribution du ticket/locator feed (auto-join à la S75 `anchors.json`, OU namespace déterministe, OU vraie propagation gossip) est un **prérequis BLOQUANT**, pas un détail.

- **Pari falsifiable :** le maillon manquant du pipeline est le **CHAMPION / claim signé** (« je prends cette idée »), PAS la structuration. Leçon dure : Decide Madrid, 21 000 propositions → ~2 abouties, parce que personne ne RÉCLAME. Le pari est que l'agentique rend le mainteneur solo capable de mener un claim jusqu'à une app — pas que des « communautés » émergent.

- **Base-rate réel ~2/50 :** une idée sur ~25 aboutit. Le funnel doit AFFICHER ce taux comme statistique descriptive honnête pour calibrer les attentes, JAMAIS comme gate ni ranking.

- **PoW dormant non-câblé :** `EscalatingPolicy` (`pow.rs:447-491`) et `AgeWitness` (`age_witness.rs:52-60`) existent et sont TESTÉS, mais DORMANTS pour l'écriture (câblés compute seulement). Le PoW feed actuel (16 bits, `BLAKE3(entry_hash‖nonce)`) est **rejouable cross-auteur** et fut effectivement rejoué verbatim (pivot PULL S75). Le funnel de claims est donc poisonable tant que le PoW différentiel (modèle Tor) n'est pas câblé à l'écriture.

---

## 2. État de l'existant : ~80% réutilisable + le bug namespace + corrections de mémoire

### 2.1 Réutilisable tel quel (ancré file:line)

**Idea hub & app sandboxée**
- App idea hub fonctionnelle, clés préfixées `ideas/<uuid>` + `votes/<id>/<pubkey>` déjà alignées sur le routing daemon : `examples/sbfb-ideas/app.js:80-168`, `SBFB.json:7-9`.
- Bridge = 16 méthodes whitelistées (PAS 3), enum Zod `BridgeMethodSchema` : `web/src/bridge/protocol.ts:20-49`. Dispatch host (vraie frontière) : `web/src/bridge/useBridge.ts:111-176`, `:235-412`. Miroir Rust déclaratif + test de parité : `crates/sbfb-manifest/src/lib.rs:52-91`, `:195-246`.
- Identité = node_id iroh via `/api/daemon/info` : `useBridge.ts:305-312`, `http.rs:76,209`.
- CSP source unique + middleware : `crates/nexus-core-rs/src/csp.rs:33`, `crates/nexus-shell-daemon/src/http.rs:549-566`.

**Feed public (substrat des ops signées)**
- Enveloppe signée Ed25519/JCS sur `DOMAIN_FEED_V1`, hash-chain BLAKE3 par-auteur, op opaque `serde_json::Value` (raw-op P51) : `crates/nexus-coordinator-rs/src/public_feed.rs:147-219`, `:660-698`.
- Op typée 0-bump (doc inline + test de non-bump) : `public_feed.rs:116-132`, `:774-840` (`seed_announced_raw_op_no_version_bump`). `KNOWN_OP_TYPES` `:349-355`, `validate_known_operation` `:357-423`, exact-key-set anti-smuggling `:330-341`.
- Champ additif 0-bump (forme `project_name`/`category` WIRE-1) : `public_feed.rs:40-53`.
- PoW feed 16 bits + GCRA 5/min : `:225-267`, `:284`, `:456-499`. `replay_all` : `:561-583`.
- Émission/ingest/subscribe : `feed_sync.rs:121-148`, `:219-431`, `:438-518`. FTS op_type indexé : `search.rs:105-137`, `:294-338`.

**Crypto & anti-Sybil**
- Gate anti-usurpation `node_id == author_pubkey` : `seed_registry.rs:206-250`, `:232`. Registry borné (caps/TTL/hex-normalize/éviction stalest) : `:36-58`, `:166-194`, `:252-279`.
- Hashcash publisher/topic/time-bound : `pow.rs:85-219`, `:333-432`. EscalatingPolicy (dormant) : `:447-491`. AgeWitness : `age_witness.rs:52-60`, `:226-242`. verifiable_draw : `verifiable_draw.rs:46-136`. canonical JCS : `canonical.rs:353`.
- Curator lists (signées, révision, abonnement, jamais ranking) : `curator.rs:101-126`, `:183-234`, `:287-320`. Kudos non-monétaire : `kudos_ledger.rs:1-47`.

**Operator & process**
- Provider routing `ExecutionTarget {Claude|Ollama|Network}` : `provider_router.rs:62-113`, `:147-240`, `:321-500`. Context-pack scellé : `operator_server.rs:540-613`. Draft gardé + rejet PASS : `:707-765`, allowlist `:28-35`. Gate keyword SENSITIVE_ACTIONS : `:37`, `:916-948`, `:1018-1041`, `:1123-1137`. PTY claude : `terminal.rs:69-89`. Gates/diff/sprint-history : `gates.rs:131-188`, `operator_server.rs:186-198`.
- Process portable (verdict RESTITUÉ jamais calculé) : `process.rs:352` (`has_final_pass_verdict`), `:359` (`extract_verdict`), `:572` (`audit_commit_data`). Grammaire phase + module SSOT : `process.rs:548-549`, `phase.rs:45-59`. Hooks backstop : `phase-auditor-gate.sh:104`, `phase-precommit-lightcheck.sh`.

**Compute / GPU**
- Consentement 4 niveaux + Caps + watcher fail-closed : `consent.rs:86-129`, `:390-434`, `:447-571`. Surface HTTP : `crates/nexus-shell-daemon/src/consent.rs:151-242`. Front : `Network.tsx:154-326`, `GpuConsentDialog.tsx:88-408`, client `web/src/api/consent.ts:38-61`, `:117-178`. Worker co-localisé : `local_worker.rs:110-160`, `:280-379`. Backends LLM : `llm/mod.rs:50-69`, `llm/factory.rs:40-62`, `config.rs:278-359`.

**Factory chaîne app**
- Scaffold : `template_engine.rs:277`. Fork durci : `fork.rs:129`, `:204`, `:280-360`. Redeploy + gate CSP : `atelier.rs:70`, `:95-99`. Pipeline publish : `pipeline.rs:15`, `:55-62`. Gate CSP non-délégable : `gates.rs:527`, path-containment `:260`, FG5/FG6 `:208`, `:270`.

### 2.2 Le bug namespace non-partagé (localisé précisément)

ROOT CAUSE : au boot, branche `None ⇒ docs_client.create_doc()` **inconditionnel** quand la table DB locale est vide → chaque nœud frais fabrique un `NamespaceId` aléatoire disjoint.
- Storage : `crates/nexus-shell-daemon/src/runtime.rs:2456-2549`, branche None `:2532-2540`.
- **Feed (découverte critique) : `runtime.rs:2555-2633` est le MIROIR du précédent**, branche None `:2617-2625`.

La mécanique join/ticket/subscribe EXISTE et passe les tests (`storage_api.rs:478-591`, `multi_daemon.rs:231-316`) mais le ticket n'est JAMAIS distribué automatiquement : ni dans `SBFB.json`/provenance/announcement, ni gossipé, ni dans la whitelist bridge. Les endpoints `GET /api/daemon/storage/ticket/{app}` + `POST /api/daemon/storage/join` (et leurs équivalents feed `http.rs:386-388`) sont des routes admin hors-bande jamais câblées au shell. D'où le bug live 2026-06-11 : deux nœuds frais rendent l'idea hub sur des namespaces disjoints.

**Conclusion :** le gap n'est pas l'absence de mécanisme (join existe) mais l'absence de **bootstrap de distribution du ticket** + l'absence d'auto-join-before-create. Et il touche le feed autant que le doc.

### 2.3 Corrections de mémoire (STALE / FAUX trouvés)

- **« bridge = 3 méthodes »** (`CLAUDE.md` §Modèle de rendu + §Décisions gelées) : FAUX. 16 méthodes (`protocol.ts:20-49`, miroir `sbfb-manifest:67-91`). Le « 3 » date de S13.
- **« process.rs ~524-525 = `[A-G][0-9]?` » + « dette regex phase ouverte »** : STALE. La grammaire est `[A-Z]+[0-9]?` à `process.rs:548-549`, gardée par test `phase_title_re_accepts_unbounded_multi_letter` (`:1028-1045`). Les lignes 524-525 sont du code `run_lint_planning`. Aucun `[A-G][0-9]?` actif ne subsiste (seulement commentaires/tests décrivant l'ancien bug). **La seule dette grammaticale réelle restante** = `REQUIRED_BODY_SECTIONS` (`process.rs:551-561`) valide 7/9 sections en `\s*$` exact alors que `agentctl.py:35-52` + le hook bash + la tolérance suffixe S80 sont préfixe-tolérants.
- **« provenance = clone → Keyoxide → zip → provenance.json »** : nuance. Le code (`provenance.rs:17-29`) est une attestation Ed25519 signée par le nœud déployeur ; Keyoxide est la vérif d'identité SÉPARÉE. De plus `provenance.rs:102-124` a un `canonical_bytes` **hand-rolled** (JSON trié manuel) qui n'utilise PAS le helper partagé `canonical_bytes` JCS — **à ne pas copier** dans une nouvelle op.
- **« GOVERNANCE.md »** : N'EXISTE PAS dans le repo (seulement `node_modules`, sans rapport). Recommandation vision jamais exécutée, bus-factor P0.
- **`DOMAIN_CLAIM_V1` est déjà pris** par `task::Claim` (compute, `canonical.rs:82-83`) — une op d'idée ne doit PAS le réutiliser ni s'appeler `Claim`.
- **« GET /api/gates = le point le moins câblé / Phase G à faire »** : le backend `gates_live_data` EST écrit (`gates.rs:131`, `operator_server.rs:198/1334`) mais NON COMMITÉ (working tree `M`), et le front `VerifyScene.tsx:30` affiche encore « gates non câblées ».
- **« Operator action-gated »** : nuance. `SENSITIVE_ACTIONS` ne garde que les 3 chemins CHAT. Le PTY (`terminal.rs:69-89`) spawn un `claude` interactif réel NON filtré — c'est une garde UI (`Terminal.tsx:48`), pas backend.
- **`composabilité app-à-app = MYTHE`** : CONFIRMÉ exact (`connect-src 'none'` + sandbox sans `allow-same-origin` + `identity_pubkey` = node_id partagé).

---

## 3. Architecture cible en couches + primitive de handoff

### 3.1 Les 5 territoires et leur découpe

| Territoire | Crate / surface | Confinement | Rôle dans le redesign |
|---|---|---|---|
| **Protocole** | `nexus-core-rs` / `nexus-coordinator-rs` | wire, Ed25519, BLAKE3, PoW, JCS | source de vérité du COMMUN (ops feed) |
| **Daemon** | `nexus-shell-daemon` | loopback authed, blob-serve CSP, feed_sync, storage | signe les ops au nom du nœud, propage, sert le bridge |
| **Shell front** | `web/` | trusted-privilégié-UI (origine daemon, authed, PAS sandbox) | onboarding compute, offre GPU, hint passif Operator |
| **App / sandbox** | iframe blob-serve | untrusted, `connect-src 'none'`, sandbox sans same-origin | idea hub Viewer-class : PROPOSE, 0 clé |
| **Operator** | `sbfb-factory` operator_server | privilégié-local, hors daemon, hors CSP scellé | pipeline ultracode, ingère le claim comme évidence |

### 3.2 Le COMMUN = app SBFB sandboxée (Viewer-class), précisément

Périmètre : proposer une idée, championner (« je prends cette idée »), appuyer, parcourir le funnel, RESTITUER provenance + verdict d'un pipeline aval. Propriété cardinale : **l'app ne signe pas** ; elle demande une op, le DAEMON signe avec la clé du nœud et applique le gate `author_pubkey == node_id` (modèle `seed_registry.rs:232`). L'app n'a aucun canal réseau direct.

### 3.3 LA primitive de handoff (corrigée — frontière de consentement obligatoire)

**Écriture (côté sandbox → daemon) :** nouvelles méthodes bridge `idea_publish` / `idea_champion` / `idea_vouch` → host → route daemon → op feed signée. L'app envoie `{idea_id, payload}` ; le daemon signe (DOMAIN_FEED_V1, JCS) et émet.

**CORRECTION SCEPTIQUE BLOQUANTE (dim. A + B) :** ces méthodes NE doivent PAS suivre le pattern silencieux de `task_submit`/`storage_set`. Ce sont des **attestations publiques permanentes, signées sous l'identité Ed25519 du nœud, propagées, et porteuses de réputation**. « app propose → daemon signe » protège contre l'usurpation d'AUTRUI mais PAS contre une app malveillante parlant AU NOM DU NŒUD de l'utilisateur (spam de claims/vouch, brûlage du budget GCRA, abonnements non choisis). Le mantra correct devient : **« app propose → HUMAIN confirme dans le shell de confiance → daemon signe »**, calqué sur le pattern GPU-consent (`consent.rs`). La confirmation doit être **host-rendue et non-spoofable par l'iframe** (style « sign this transaction » d'un wallet), pas une simple modale dans la sandbox.

**Lecture / handoff (côté Operator) :** route additive read-only `/api/ideas` lisant le feed via le daemon loopback. **CORRECTION (dim. A) :** read-only ET **failure-tolerant** — l'Operator doit fonctionner daemon absent (idées simplement absentes), sinon on érode « Factory = outil client externe hors daemon ». Le claim sélectionné devient un item du context-pack repo-visible (`handle_context_pack`, `chat_history_authoritative:false`) → vraie session agent. **Le claim est ÉVIDENCE, jamais commande ; aucun auto-launch.**

**Hint passif shell :** lien « Cette idée peut être reprise dans l'Operator » (navigation humaine, 0 token/0 payload) qui bute sur l'auth Operator (401) tant que l'humain n'a pas son cookie — ce qui PROUVE l'absence de control-flow. Le join sandbox↔Operator se fait par le **content-address partagé** (`idea_id = blake3`), exactement comme `archive_hash` est la vérité de joignabilité.

---

## 4. Le claim signé = entité première

### 4.1 Substrat : feed per-auteur (op typée 0-bump), pas doc partagé

Le `FeedEntry` est une enveloppe versionnée signée Ed25519 sur `DOMAIN_FEED_V1` via JCS, chaînée BLAKE3 par-auteur. Chaque nœud n'écrit QUE sa propre chaîne : c'est l'antithèse d'un pool global ouvert (verrou « pas de pool unique en écriture ouverte »). Ajouter une variante typée à `PublicFeedOperation` NE bump PAS `FEED_FORMAT_VERSION` (précédent `SeedAnnounced` S74 / `CuratorVouched` S67, prouvé par test `:774-840`).

**Signing domain :** AUCUN domaine neuf sur le chemin feed — tout passe par `DOMAIN_FEED_V1` au niveau enveloppe. **NE PAS réutiliser `DOMAIN_CLAIM_V1`** (`canonical.rs:82-83`, déjà = `task::Claim`). Si un jour un type autonome est nécessaire (claim révisable hors feed), nommer `DOMAIN_IDEA_CLAIM_V1` et copier le patron NodeDirectory (`node_directory.rs:78-294`).

### 4.2 Les ops (data-model)

```
IdeaPublished  { idea_id, title, tags }                      // dénominateur du funnel
IdeaChampioned { idea_id, champion_node_id == author_pubkey } // LE CLAIM = entité première
IdeaVouched    { idea_id }  /  IdeaUnvouched { idea_id }      // endossement pluriel (miroir Curator)
IdeaResigned   { idea_id }                                    // le champion abdique → réclamable
```
- `idea_id = blake3(canonical_bytes(JCS{title, tags, ...}))` réutilisant `canonical.rs:353` (jamais le hand-rolled de provenance). **CORRECTION (dim. B) : RETIRER `created_at` du préimage** — sinon identité non-déterministe → fragmentation en N quasi-doublons gonflant le dénominateur et dispersant les champions.
- `IdeaChampioned` : gate `champion_node_id == author_pubkey` à l'ingest (copie `seed_registry.rs:232`). Horodatage = `FeedEntry.timestamp` natif borné (`FEED_MAX_FUTURE_SECS`). Aucune signature payload-level (invariant F-3).
- Validation : exact-key-set par op (copie `:330-341`), hex-64, bornes de longueur.
- Émission : `emit_idea_*` calqués `emit_seed_announced` (lock DB sync PUIS publish async).

### 4.3 Namespace iroh-docs partagée imbriquée (le vrai fix de convergence)

**CORRECTION SCEPTIQUE FATALE (dim. B/F) :** porter le funnel sur le feed ne suffit PAS — le feed partage le bug de bootstrap-namespace. Le sprint DOIT livrer un mécanisme de convergence. Trois options, à arbitrer PO :

1. **Auto-join feed via anchors (recommandé)** : étendre le pattern S75 `anchors.json` pour que le boot driver auto-joigne le namespace feed (aujourd'hui il ne joint que les SEEDS).
2. **Namespace feed déterministe** : dériver le `NamespaceId` d'une graine bien-connue au lieu de `create_doc()` aléatoire.
3. **Namespaces imbriquées en READ-ticket** (pour tout doc mutable légitime, ex. commentaires) : distribuer un **read-ticket** (`Mode::Read`, pas write — l'actuel n'utilise que `share_write()` `runtime.rs:2534`) via une op feed 0-bump `SharedNamespaceAnnounced {app_id, namespace_id, read_ticket, origin_node_id==author_pubkey}` ; `boot_storage_namespace` devient **JOIN-before-CREATE**. Chaque nœud écrit dans SA namespace per-nœud (write-owned, clés `votes/<id>/<pubkey>` déjà namespacées par pubkey) ; le lecteur union toutes les namespaces read-jointes. **Garde-fou (dim. B #8) : gater le join à l'auteur déployeur / aux curateurs abonnés** (sinon flood de namespaces annoncées = griefing storage). Différer cette option (b) tant qu'aucun consommateur concret n'existe.

Sans ce mécanisme, le claim-sur-feed est aussi non-convergé que le doc. **À tester en E2E 2-nœuds-frais : les claims convergent SANS join manuel, avec `b3 PASS` (gate testabilité README §4).**

### 4.4 Anti-Sybil : PoW dormant différentiel (modèle Tor)

**CORRECTION (dim. B/F) : câbler le PoW différentiel comme DÉFAUT, pas en option.** Le PoW feed plat 16 bits est rejouable et fut rejoué (S75). Sur le chemin émission/ingest des claims :
- **Identité établie ⇒ ~0** : tarif de base. `established = AgeWitness valide (≥7j + témoin ≥30j) OU (longueur de chaîne ≥ N ET âge ≥ D)` mesuré localement.
- **Identité fraîche ⇒ plein tarif one-shot** : la 1ère entrée d'une chaîne (genesis, `prev_hash=GENESIS`) exige un Hashcash publisher-bound + topic-bound (~20-22 bits, lié à `author_pubkey`, non rejouable). Identité jetable = coûteuse une fois.
- **Sous flood ⇒ escalade** : `EscalatingPolicy` keyée par `champion_node_id` (rampe géométrique, reset quotidien). **Caveat honnête (dim. B #7) :** le critère « ancienneté de chaîne » offre un chemin pre-warm bon marché et la rotation d'identité rafraîchit le budget → préférer witness-only une fois `PowFallback` retiré au v1.0 ; documenter le coût asymétrique-non-parfait.
- **Budget « pair frais < 1 min » sanctuarisé** : le plein tarif ne s'applique qu'à l'ÉCRITURE. LIRE le funnel est gratuit — **mais seulement après convergence (§4.3)**.

### 4.5 Funnel mesuré, base-rate ~2/50

Instrument = le feed. `FunnelStats = replay_all` GROUP BY op_type JOIN `idea_id`. Étages : `IdeaPublished` (50) → `IdeaChampioned` (réclamée) → roadmap (local) → `ReleasePublished{idea_id}` (publiée) → maintenue (~2).

**CORRECTIONS (dim. A/B/F) :**
- **Scoper le comptage à la VUE CURÉE** (listes curator abonnées), JAMAIS un GROUP BY brut sur `replay_all` — un count global non scopé = ranking global par la porte de derrière. `ClaimRegistry` calqué `seed_registry.rs:206-279` (cardinalité de champions distincts, jamais classement).
- **Ne jamais trier les idées par count agrégé dans une vue par défaut** — ordre = récence ou liste-curateur signée seulement ; counts en cardinalité plate.
- **« app maintenue » dérivée d'un `ReleasePublished` frais SIGNÉ PAR L'AUTEUR (provenance-backed, commit_sha frais) UNIQUEMENT.** NE PAS relabelliser la fraîcheur `SeedAnnounced` en « maintenue » (confond héberger et maintenir). Garder « joignable/seedé » comme axe séparé.
- Le `~2/50` est RESTITUÉ comme note descriptive honnête, caveats affichés : per-vue-curée (pas de nombre réseau-global officiel), best-effort, décroît avec le TTL.

---

## 5. Onboarding compte : brancher son LLM + partager son GPU

### 5.1 Les 3 axes orthogonaux (à ne jamais conflater)

- **PRODUIRE (servir les autres)** = consentement GPU L1-L4 + Caps (`consent.rs:86-105`). DÉJÀ complet (S16/S76).
- **SERVIR-EN-LOCAL (mon backend)** = `LlmConfig{backend, ollama, llama_cpp}` (`config.rs:278-303`). Aujourd'hui éditable uniquement via `worker.toml`.
- **CONSOMMER (où tourne MON inférence)** = `ExecutionTarget {Claude|Ollama|Network}` (`provider_router.rs:62-113`). Vit uniquement dans le chat Operator.

Les 4 modes (none/local/cloud/network) sont l'axe CONSOMMER. Le partage GPU est l'axe PRODUIRE, orthogonal. Le wizard pose DEUX questions distinctes.

### 5.2 Travail neuf (~30%)

- Modèle `ComputeMode {None, Local, Cloud, Network}` + `LlmConfig` persisté dans `compute.json` **sibling de `consent.json`** (NE PAS fusionner — orthogonalité). Config LOCALE, zéro wire, zéro bump.
- `GET /api/v1/compute/preflight` (GPU présent ? Ollama joignable + modèles ? llama.cpp compilé ? claude CLI authentifié ?) — comble le seul vrai trou (réutilise `ollama_diagnostic` `provider_router.rs:118-134`).
- `GET/POST /api/v1/compute(/set)` avec fail-loud LlamaCpp-sans-feature (miroir `factory.rs:40-62`).
- **Fix root-cause** : `local_worker.rs:296` `provision()` lit `compute.json` et écrit `local_llm` dans `worker.toml` (aujourd'hui le worker co-localisé n'adopte que level+caps, jamais la config LLM).

### 5.3 Corrections sceptiques (dim. C) — important pour le placement

- **MOUNT-POINT : NE PAS étendre `OnboardingEmpty`** (`OnboardingEmpty.tsx:3-11` ne s'affiche QUE « when no daemon is serving », état d'erreur que la plupart ne voient jamais). Monter le wizard sur une vraie surface de 1er-run / Réglages → Compute, atteignable APRÈS connexion daemon (à côté de `OfferPowerCard`).
- **HOOK : compute onboarding STRICTEMENT optionnel, HORS chemin critique.** Le défaut `None` DOIT atteindre du contenu utile (Browse → ouvrir une app) sans aucune étape compute. Le vrai hook < 1 min du pair médian = « ouvre une app et ça marche », PAS « branche ton Ollama » (sinon download multi-Go = jamais < 1 min).
- **`compute_status` bridge : RETIRER de C** (aucun consommateur dans le périmètre ; l'app ne peut rien en faire d'actionnable). Le livrer avec la dimension `llm_compose` (exposer l'IA du nœud aux apps = capacité distincte, host-médiée, consentement propre — HORS périmètre).
- **`pull-model` SSE : RETIRER.** Le preflight détecte « modèle absent » et affiche la commande `ollama pull <modèle>` en clair (conforme no-CDN, sans surface streaming privilégiée).
- **Cloud/central :** renommer « Cloud (IA hébergée) » ; JAMAIS le défaut (défaut None, Local mis en avant) ; fermer l'option clé-API in-product (rester sur le CLI `claude` déjà authentifié, zéro secret au repos) ; envisager de scoper Cloud au contexte Factory où il vit déjà.
- **Reload :** `consent.json` est hot-reloaded (`ConsentWatcher`). `compute.json` n'est lu qu'au `provision()`/spawn → documenter « la config LLM s'applique au prochain spawn worker » OU étendre le watcher.

---

## 6. Le pipeline Factory Ultracode in-app (idée → claim → roadmap → sprint → phases)

### 6.1 Architecture à deux étages (anti-superviseur)

CLAUDE.md gèle (amendement 2026-06-17, superviseur supprimé `42c7448`) : « plus de superviseur process dédié ; l'orchestration EST le séquenceur ». Donc :
- **Étage intérieur (RÉUTILISÉ INCHANGÉ)** = le Workflow ULTRACODE exécuté dans une VRAIE session `claude` (PTY) pour chaque phase (preflight→code→review→Codex→commit). C'est de facto la « vraie session agent + preuves repo » exigée pour shell/commit/push/verdict final.
- **Étage extérieur (NEUF, mince)** = une VUE de pipeline dans l'Operator qui RESTITUE l'état depuis les artefacts repo, drafte les plans depuis un claim, et garde les points de consentement humain.

### 6.2 CORRECTION SCEPTIQUE BLOQUANTE (dim. D) : ne PAS ressusciter GO/BLOCK

La proposition naïve d'une `POST /api/pipeline/advance` qui « vérifie la garde » et renvoie `blocked:{...}`, plus une « table des gardes » autorisant/refusant chaque transition, **EST une consultation GO/BLOCK entre étapes** — exactement ce que `42c7448` a retiré. Que le verdict soit restitué et non calculé ne sauve pas : le décret abolit une **autorité de séquençage séparée**, pas seulement le calcul.

**Architecture corrigée :**
- SUPPRIMER `/api/pipeline/advance` en tant que GATE. Ne garder que :
  - `GET /api/pipeline/state` — restitution PURE (par phase `{step, restituted_verdict, source_artifact}`), read-only, calquée `handle_gates`.
  - `POST /api/pipeline/prepare-step` — assemble + scelle le context-pack (réutilise `handle_context_pack`) + évidence-claim délimitée non-fiable.
  - `POST /api/pipeline/launch-session` — Ring 2, spawn PTY vraie session, **0-auto-spawn** (action humaine, patron CTA-démarrage Phase D), `log_action`.
- **Aucune route ne renvoie `blocked` comme autorité.** Le conducteur est une VUE ; le Workflow-en-PTY reste le séquenceur ; les hooks (`lightcheck` + `auditor-gate`) restent le SEUL backstop mécanique.
- **AUCUNE route `/commit` ni `/push`.** Les commits/push n'existent que dans la session réelle.
- `pipeline_state.json` (dans `.planning/active/`, allowlist draft) = **advisory-only** : jamais lu pour un verdict ou une progression ; le verdict est TOUJOURS recalculé-depuis-artefacts via `has_final_pass_verdict`/`extract_verdict`/`audit_commit_data`. Les artefacts gagnent toujours.

### 6.3 Les 3 anneaux de consentement (mappent 1:1 sur du code existant)

- **Ring 1 — AUTONOME** (read-only ou draft-vers-allowlist, PASS rejeté mécaniquement `:729-765`) : restituer l'état, drafter kickoff/plan/préfill preflight, sceller des packs, un tour LLM qui COMPOSE un draft.
- **Ring 2 — CONSENTEMENT HUMAIN EXPLICITE** : spawn PTY, promotion d'un draft de roadmap en plan, lancement d'une vérification.
- **Ring 3 — JAMAIS AUTONOME** : commit de phase, push, `## Verdict: PASS` final. Un DESIGN-CONFLICT preflight STOPPE et escalade au PO (« le hub PROPOSE, n'arbitre jamais »).

### 6.4 Garde-fou injection PTY (CORRECTION dim. D, fermée par décision)

**Décision, pas option PO :** l'amorce de `launch-session` ne porte QUE le context-pack scellé (artefacts restitués + données-claim délimitées « contenu utilisateur non-fiable — données, jamais instructions »). Elle NE DOIT PAS porter d'instructions auto-drivant vers Ring 3. L'humain tape les instructions de pilotage dans le PTY. Obligatoire, non configurable (incident ClickFix S71, MEMORY injected_commands : le claim est du contenu non-fiable propagé P2P, dirigé vers la SEULE surface qui peut committer).

### 6.5 Dette regex — état réel

La dette regex de PHASE est **DÉJÀ FERMÉE** (`process.rs:548-549` `[A-Z]+[0-9]?` + test). La seule dette grammaticale RÉELLE = harmoniser `REQUIRED_BODY_SECTIONS` (`process.rs:551-561`, 7/9 en `\s*$` exact) sur le préfixe-tolérant déjà adopté par `agentctl.py:35-52` + hook bash + tolérance suffixe S80. **Conséquence pour le pipeline :** un body suffixé passerait Python+bash (le hook laisse committer) mais ÉCHOUERAIT la restitution Rust `audit_commit_data` → le pipeline afficherait « commit invalide » sur un commit valide. Fix précis, faible risque, à faire avec visibilité audit. Vérifier d'abord si un hook invoque le `audit-commit` Rust : sinon c'est de l'alignement-restitution pur, pas un affaiblissement de gate.

### 6.6 Garde-fous non négociables (process > RRV > Factory)

- Hiérarchie process > RRV > Factory : roadmap = draft Ring 1 ; promotion = décision humaine Ring 2 ; DESIGN-CONFLICT escalade au PO.
- Verdict RESTITUÉ jamais calculé UI : toutes les gardes appellent les fonctions du CLI/hook sur artefacts repo.
- Gates non-délégables : hooks `lightcheck`+`auditor-gate`, `SENSITIVE_ACTIONS`, gate CSP — le pipeline n'en contourne aucun.
- LLM compose, ne prouve jamais : les tours LLM ne produisent que des drafts (PASS bloqué) ; le claim est délimité en données non-fiables ; le verdict vu par le LLM est restitué.

---

## 7. Gouvernance + anti-recentralisation

### 7.1 GOVERNANCE.md (doc pur, P0 bus-factor, à écrire EN PREMIER)

N'existe pas dans le repo. À écrire avant le code (zéro risque code, ferme le P0). Contenu :
- **§1 Nature** : commun humaniste non-monétaire (Wikipedia/Tor/Linux ; OpenBSD solo-maintainer), AGPL-3.0, pas de moat/fondation/startup. Pari falsifiable affiché : micro-noyaux 1-3 humains augmentés. Base-rate ~2/50 affiché.
- **§2 Les 5 verrous comme invariants constitutionnels.** **CORRECTION SCEPTIQUE (dim. E) : énumérer VERBATIM les 5 verrous canoniques de `sprint75_kickoff.md §4`, ne PAS inventer une liste sous le même nom :**
  1. zéro champ cible/hôte (pas de ciblage),
  2. redondance additive jamais substitutive (node-Browse = sur-ensemble STRICT, `known_browse_entries` compte honnêtement),
  3. VPS = « mon serveur » possessif, jamais défaut compilé (`default_curators` vide `config.rs:249-250`, seed défaut VIDE),
  4. provenance/signature TOUJOURS celles de l'auteur (seed != autorité),
  5. suggestion déclenchée par l'état observé, jamais poussée.
  Les principes abstraits (pas de serveur central / pas d'admin / pas de ranking) vont en PRÉAMBULE, jamais re-étiquetés « les 5 verrous ».
- **§3 Table BDFL CAN/CANNOT** : pouvoir sur le CODE SBFB (commit/merge/release, AGPL) = réel/assumé ; pouvoir sur le RÉSEAU (supprimer/bannir/ranker/censurer) = **non-existant par construction** (verrous V1-V5). La curation du mainteneur = UNE curator list parmi d'autres, forkable, non-autoritaire.
- **§4 Procédure d'amendement** : une PR qui modifie GOVERNANCE.md, porte rationale + supersede, passe les MÊMES gates process (preflight/review/Codex/commit). **CORRECTION (dim. E) : NE PAS propager la constitution via une op feed publique** (overreach + confusion de catégorie — le réseau n'a aucun « état de gouvernance »). La garder en git AGPL, couverte par la signature de release ; au plus un pointeur de découverte content-adressé (`blake3`).
- **§5 Succession / bus-factor** : code AGPL + git ⇒ quiconque fork si le mainteneur disparaît ; le réseau tourne (pas de serveur central à mourir).

### 7.2 FROST — CORRECTION SCEPTIQUE MAJEURE (dim. E)

La promesse « FROST K-of-N pour college curateur + succession, REUSE total » est **vraie côté VÉRIFICATION** (une signature FROST-agrégée est un Ed25519 RFC 8032 byte-identique → `CuratorListEntry::verify_signature` marche inchangé, `frost.rs:7-12`) mais **FAUSSE côté PRODUCTION** : le primitif existant est **in-process, un seul `FrostCanarySigner` détenant TOUTES les K key packages** (`frost.rs:29-38`), et `dkg.rs:2-13` est un trusted-dealer (une machine génère toutes les parts). La vraie orchestration DKG cross-process est documentée hors-scope (« Sprint 25-30 », jamais construite). **Un FROST 2-of-3 où une machine voit les 3 parts ne distribue aucun pouvoir : bus-factor reste 1.**

Conséquence : soit scoper la VRAIE cérémonie cross-process (chaque steward signe sur SA machine, parts jamais co-localisées) comme NEUF de 1ère classe, soit restreindre honnêtement aux cas que le trusted-dealer + distribution air-gapped supportent. Distinguer : **succession** (signe rarement → air-gapped tolérable) vs **college curateur** (listes à révision monotone re-signées souvent → cérémonie multi-round par révision irréaliste → garder mono-curateur Ed25519 par défaut, FROST optionnel pour chartes à faible churn). Placement : la cérémonie à seuil reste dans le CLI daemon (`sbfb canary frost ...`) ; l'Operator l'INVOQUE seulement (conforme « Factory hors daemon »).

### 7.3 Curator list = unité de gouvernance polycentrique

Extensions ADDITIVES (pré-launch, redéfinit v1) à `CuratorList` :
- **Charte affichable** : champ `charter` (~2-4KB, sous le bound A-4) — ligne éditoriale / critères. S'abonner devient un choix éditorial informé.
- **Deny/mute-lists** : discriminant `kind: ListKind {Allow, Deny, StarterPack}`. **CORRECTION (dim. E) : la soustraction doit être TRANSPARENTE + réversible + auditable** (afficher « N entrées masquées par la deny-list X abonnée », un-hidable au clic ; le compte honnête `known_browse_entries` reste COMPLET). Sinon friction avec le verrou-2 (vue = sur-ensemble honnête). Une deny-list n'est JAMAIS de la modération globale : recommandation abonnable, effet local au seul abonné, aucun octet supprimé (BLAKE3 sert toujours).
- **Starter packs importables 1-clic** : bundle d'abonnements pour une 1ère vue utile < 1 min. **CORRECTION (dim. E, Q6) : OFFRE affichée opt-in OK, AUTO-import INTERDIT** (préserve verrou-3 « seed défaut VIDE »). Le pack « officiel » du mainteneur doit être déclaré NON-AUTORITAIRE et forkable dans GOVERNANCE.md.
- Op feed optionnelle `CuratorListAnnounced` (raw-op 0-bump) pour la découverte d'existence.

### 7.4 Idea hub zéro-ranking-global

Vue par-nœud, promotion par choix individuel (s'abonner). Aucun compteur d'upvote universel : le nombre est calculé PAR-NŒUD depuis l'ensemble d'abonnement → deux nœuds voient des ordres différents → pas de vérité globale. Kudos = tri-MOU local optionnel débrayable, **jamais gate dur** ; aucun seuil kudos ne gate publier/championner/voter/être-affiché (conforme `kudos_ledger.rs:1-7`, non-transférable, log_utility <10x).

---

## 8. Data-model idée→research→phase EN ÉCRITURE + frontière donnée-P2P vs artefact-local

### 8.1 Modèle à deux moitiés reliées par un cordon

| Entité | Substrat | Mutabilité | Signé | Rôle |
|---|---|---|---|---|
| genèse idée (`IdeaPublished`) | feed | append-only | Ed25519 FeedEntry | événement commun, PoW-gaté |
| **claim (`IdeaChampioned`)** | **feed** | append-only | Ed25519, claimer==author | **maillon manquant** |
| vouch/désaveu (`IdeaVouched`/`Unvouched`) | feed | append-only | Ed25519 | curation plurielle, abonnement |
| corps long / commentaires | doc iroh (OPT) | mutable | per-key node_id | annotation, join paresseux (différé v1) |
| research / roadmap | Operator `.planning/` | fichier repo | git commit | artefact souverain |
| **kickoff (`claims_idea`)** | **Operator `.planning/`** | fichier repo | git commit | **le cordon côté repo** |
| phases / reviews / codex | Operator `.planning/` | fichiers repo | git + hooks | process, verdict restitué |
| app publiée (`claims_idea`) | feed `ReleasePublished` | append-only | Ed25519 + provenance | fin du funnel |

**Énoncé de frontière :** la moitié P2P est permissionless / signée / NON-AUTORITAIRE (PROPOSE, COUNT, abonnement, zéro ranking) ; la moitié repo est gated / agent-pilotée / verdict-restitué (EXECUTE). Le cordon `idea_id` est écrit une fois de chaque côté. Aucun côté ne calcule de verdict sur l'autre.

### 8.2 Le cordon et sa fermeture honnête

- `IdeaChampioned.idea_id` (feed) + `claims_idea: <idea_id>` en frontmatter du kickoff (repo) + champ additif 0-bump `claims_idea: Option<String>` sur `ReleasePublishedPayload` (forme `project_name`/`category` WIRE-1, `public_feed.rs:40-53`).
- **CORRECTION (dim. D/B) : par défaut, `claims_idea` sur le wire public est à éviter au profit d'une restitution LOCALE idée→app** (le nœud claimant-publieur connaît son propre lien ; funnel per-nœud-curé). Lier publiquement claim→app permettrait d'agréger cross-nœud « quelles idées ont produit le plus d'apps » = ranking global de facto. Si gardé public, le restreindre à un lien de provenance nu, interdire toute co-publication de score, et ne compter QUE ce que CE nœud a ingéré.
- **CORRECTION honnêteté du cordon (dim. F) :** `claims_idea` (frontmatter) et `IdeaChampioned.idea_id` (feed) sont écrits indépendamment, AUCUN cross-check enforçable en append-only. Le JOIN mesure des liens **DÉCLARÉS, pas VÉRIFIÉS** → ajouter une vérification SOUPLE au moment du draft Operator (le `idea_id` du kickoff DEVRAIT correspondre à un `IdeaPublished`/`IdeaChampioned` visible), soft-check, pas gate dur (reste PROPOSE). Documenter la limite.

### 8.3 Migration des votes — CORRECTION budget (dim. F)

**NE PAS migrer les votes haute-cardinalité doc→feed par défaut.** Chaque vote deviendrait une op gossipée globalement et rejouée depuis genèse (`replay_all`) → feed croît sans borne → un pair frais doit ingest+replay+reindex tout l'historique avant < 1 min de contenu utile. Garder `IdeaPublished`/`IdeaChampioned` (basse cardinalité, load-bearing) sur le feed ; pour les vouches, soit op feed (signal de curation acceptable, scopé à la vue curée) soit doc per-key lazy. Prévoir une vue funnel **fenêtrée/paginée** plutôt qu'un `replay_all` intégral. Découpler la refonte UX des votes du data-model coeur (garder la phase atomique).

### 8.4 Identité du champion (question ouverte tranchée à acter)

`identity_pubkey` retourne le node_id partagé du nœud. Un claim est donc attribué au NŒUD, pas à un humain nommé. Pour les micro-noyaux 1-3 humains, **node = champion** est cohérent (héberger != publier, modèle storage writes) — mais à DOCUMENTER explicitement : le gate `author_pubkey == node_id` atteste « ce NŒUD a claimé », pas « cet HUMAIN ». Une identité par-humain est un gros chantier hors MVP.

---

## 9. Roadmap mappable à un futur sprint Cas C

Numérotation indicative (grammaire `Phase [A-Z]+[0-9]?`, `Phase 0` = audit gate). RÉUTILISE vs NEUF par phase. Chaque phase porte sa testabilité (T1 E2E hermétique BLOQUANT + T2 acceptance JSON, README §4).

| Phase | Objet | RÉUTILISE | NEUF | Testabilité |
|---|---|---|---|---|
| **0** | Audit gate sprint précédent | pattern Phase 0 permanent | — | gate P0/P1 |
| **A** | **GOVERNANCE.md** (doc pur, P0 bus-factor, EN PREMIER) | vision_communs, 5 verrous `sprint75_kickoff §4` | constitution BDFL CAN/CANNOT + procédure amendement | review humaine (doc) |
| **B** | **Convergence feed/storage** (PRÉREQUIS BLOQUANT) | `anchors.json` S75, `storage_join`/`feed_join`, `multi_daemon.rs` | auto-join-before-create (option 1/2/3 §4.3), read-ticket | **E2E 2-nœuds-frais convergent SANS join manuel + `b3 PASS`** |
| **C** | Ops feed claim 0-bump | `public_feed.rs` patron SeedAnnounced, exact-key-set, `seed_registry:232` | `IdeaPublished/Championed/Vouched/Resigned` + validate + `idea_id` blake3 (sans created_at) | tests no-bump + ingest gate |
| **D** | Anti-Sybil différentiel | `pow.rs` Hashcash + EscalatingPolicy, `age_witness.rs` | câblage écriture (genesis plein tarif, escalade keyée) | tests PoW/escalade + flood sim |
| **E** | Frontière bridge + consentement host | `protocol.ts`/`useBridge.ts`/manifest parité | méthodes `idea_*` + **modale consentement host non-spoofable** + routes daemon signantes | parité 2-côtés + test confirmation obligatoire |
| **F** | App idea hub refaite (Viewer-class) | `examples/sbfb-ideas` squelette UX | intentions « Proposer/Je prends/J'appuie », funnel per-vue, co-champions pas concurrents | E2E rendu + vue funnel |
| **G** | Funnel mesuré + cordon | `replay_all`, FTS op_type, `ClaimRegistry` calqué SeedRegistry | scoping vue-curée, `claims_idea` (local par défaut), soft-check Operator | tests COUNT per-vue + base-rate restitué |
| **H** | Onboarding compute (optionnel, hors chemin critique) | consent S16/S76, `ExecutionTarget` S72, `local_worker` | `compute.json`, `/compute/preflight`, fix `provision()`, wizard Réglages | T1 défaut None → Browse marche sans compute |
| **I** | Operator : pipeline-comme-VUE | `process.rs *_data`, `handle_context_pack`, PTY, gates/diff/sprint-history | `GET /api/pipeline/state` (restitution PURE), `prepare-step`, `launch-session` 0-auto-spawn ; **PAS de `/advance` gate, PAS de `/commit`** | restitution read-only + injection-guard PTY |
| **J** | Fix dette body-sections (parité validateurs) | `agentctl.py` préfixe-tolérant | `REQUIRED_BODY_SECTIONS` Rust préfixe-tolérant | test parité 3-validateurs |
| **K** | Curator-as-governance (charte + deny-list transparente + starter pack opt-in) | `CuratorList`, RevocationCache | `kind`/`charter`, filtre soustractif transparent, import 1-clic | tests deny-list un-hidable + import réversible |
| **…** | (FROST cross-process si scopé NEUF 1ère classe — sinon hors-sprint) | `dkg.rs`/`frost.rs` côté vérif | cérémonie cross-process réelle (sinon bus-factor reste 1) | — |
| **Z** | Wrap-up + verification + audit_plan sprint+1 | pattern wrap-up | carries P1/P2 | suites §7.4 |

**Ordre critique :** A (doc, zéro risque) → **B (convergence, BLOQUANT — rien n'a de sens cross-nœud sans)** → C/D/E/F/G (le commun) → H (compute, découplable) → I/J (pipeline) → K (gouvernance polycentrique). Si B ne peut être garanti vert cross-nœud, **scoper le funnel single-nœud/my-node-centric pour v1 et étiqueter cross-nœud `PROVISIONAL` + carry P1** (cohérent vision micro-noyaux).

---

## 10. Conformité décisions gelées + questions PO ouvertes

### 10.1 Table de conformité

| Décision gelée | Statut | Note |
|---|---|---|
| 5 verrous anti-recentralisation (`sprint75 §4`) | TENU | énumérer VERBATIM dans GOVERNANCE.md ; deny-list transparente pour verrou-2 |
| Operator privilégié-local JAMAIS sandbox / Viewer sandbox OK | TENU | COMMUN = Viewer-class ; pipeline = Operator ; preview = iframe enfant CSP |
| CSP `connect-src 'none'` + sandbox sans same-origin + bridge whitelist seul canal | TENU | toute capacité neuve = méthode bridge whitelistée + miroir manifest ; jamais app→Operator direct |
| **Consentement host avant op signée-nœud** | RENFORCÉ | correction sceptique : `idea_*` exigent confirmation host non-spoofable (PAS silencieux comme task_submit) |
| kudos non-transférable, jamais cost/stake/burn/gate dur | TENU | claim gratuit ; PoW = coût calcul anti-spam Tor, pas stake ; kudos = tri-mou local débrayable |
| zéro ranking global / chaque nœud cure sa vue / promotion par abonnement | TENU | funnel = comptes per-vue-curée, jamais leaderboard ; pas de tri par count agrégé |
| pré-launch wire raw-op 0-bump | TENU | ops idée typées sans bump (test SeedAnnounced) ; `claims_idea` additif ; `DOMAIN_CLAIM_V1` NON réutilisé |
| process > RRV > Factory ; hub PROPOSE jamais n'arbitre ; verdict RESTITUÉ jamais calculé UI | TENU | pipeline = VUE pas séquenceur (pas de GO/BLOCK ressuscité) ; DESIGN-CONFLICT escalade PO |
| LLM compose depuis evidence, jamais ne crée de preuve | TENU | claim = preuve crypto Ed25519 du daemon ; LLM consomme comme évidence délimitée |
| héberger != publier, seeder != auteur, BLAKE3 = vérité | TENU | `claimer==author` ; **« maintenue » dérivée de release-auteur, JAMAIS de seed** (correction) |
| Factory = outil client externe hors daemon | TENU | Operator hors daemon ; `/api/ideas` read-only failure-tolerant ; FROST côté CLI daemon |
| Launcher Rust / backend Rust / no-CDN / AGPL | TENU | tout additif Rust ; `ollama pull` via API user (pas CDN SBFB) |
| intentions-pas-jargon en CTA | TENU | « Proposer / Je prends / J'appuie / Préparer la roadmap » |
| **convergence cross-nœud** | **PROVISIONAL** | feed ET doc partagent le bug bootstrap-namespace ; B = prérequis bloquant |
| **FROST multi-humain** | **NON-LIVRÉ** | primitif in-process trusted-dealer seulement ; bus-factor reste 1 sans cérémonie cross-process |
| **GOVERNANCE.md** | **INEXISTANT** | à écrire Phase A (P0) |

### 10.2 Questions PO ouvertes (honnêtes)

1. **Convergence (la plus importante) :** option auto-join anchors (1), namespace déterministe (2), ou read-ticket imbriqué (3) §4.3 ? Et : faire de PULL-3 un prérequis BLOQUANT, OU scoper le funnel single-nœud v1 + carry P1 `PROVISIONAL` ?
2. **Énumération des 5 verrous :** confirmer la liste VERBATIM `sprint75_kickoff §4` (et non une articulation inventée) à constitutionnaliser dans GOVERNANCE.md.
3. **FROST :** scoper la vraie cérémonie cross-process comme NEUF 1ère classe (le 70% qui donne la valeur), ou rester sur trusted-dealer air-gapped pour la seule succession (jamais le college curateur à révision monotone) ?
4. **Identité du champion :** acter node-key = champion (recommandé MVP, cohérent storage) ou prévoir une identité par-humain (gros chantier) ?
5. **Substrat du commun :** feed-native primaire (recommandé) + fix convergence ; conserver le doc iroh seulement pour commentaires lourds (différés v1) ?
6. **Migration des votes :** garder le toggle mutable (per-doc, défendable comme non-global) ou passer à `IdeaVouched`/`Unvouched` append-only (change la sémantique UX, attention budget feed) ?
7. **`claims_idea` :** local par défaut (recommandé, pas de ranking global) ou champ wire public restreint ?
8. **Rétractation du claim :** `IdeaResigned` append-only (recommandé) vs type autonome révisionné NodeDirectory ? Auto-réouverture ou fenêtre de staleness ?
9. **Anti-Sybil :** PoW différentiel DÉFAUT pour `IdeaChampioned` ET `IdeaVouched`, ou seulement scoper la métrique à la vue curée ? Seuils `established` (N/D) et difficulté genesis (20-22 bits) ?
10. **Onboarding compute :** mount-point Réglages (recommandé, PAS OnboardingEmpty) ; `compute_status`/`pull-model` retirés de ce scope confirmé ; Cloud renommé + jamais défaut + clé-API hors-produit ?
11. **Pipeline :** confirmer la VUE pure (pas de `/advance` gate, pas de `/commit`) ; ingress du claim via daemon loopback read-only OU export fichier repo (isolation plus stricte) ; suites §7.4 restituées-seulement (recommandé) ou exécutées par l'Operator ?
12. **GOVERNANCE.md :** Phase A du sprint (recommandé) ou sprint dédié ? Stewards initiaux nommés réels (bus-factor ≥ 3 exige des humains — actuellement solo) ?
13. **Hint passif shell vers l'Operator :** acceptable (navigation humaine 0-autorité), ou même un lien visuel brouille-t-il la frontière PROPOSE/arbitre au point de s'en passer ?

---

*Fin de la note de recherche. Aucun engagement. Toute exécution passe par un kickoff Cas C, un audit gate Phase 0, et le process per-phase complet (deep preflight 5 scans + review + Codex). Les caveats `PROVISIONAL` (convergence cross-nœud, FROST multi-humain) et le base-rate honnête `~2/50` doivent survivre intacts jusqu'au kickoff — ne pas les propager comme acquis.*