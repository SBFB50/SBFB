# Plan de phase (proposition) — "Ton impact (prive)" — MVP contribution/visibilite

## Statut process (proposition hors-roadmap ; audit gate S75 d'abord ; passerait par preflight G8/review/Codex)

**Ceci est une PROPOSITION, pas un sprint ouvert.** Le travail decrit ici ne s'execute pas tant que deux conditions ne sont pas remplies :

1. **Decision PO d'insertion roadmap.** Le systeme contribution/visibilite **n'est PAS sur la roadmap v5** (`roadmap_v5_factory_complete_vision.md`). v5 arc 6 sprints = S71 assainir compute / S72 provider routing / S73 recherche reseau / S74 atelier fork / S75 GPU partage cross-machine / S76 sharding. **Ce MVP doit etre explicitement insere par le PO** (soit comme phase d'un sprint existant, soit comme mini-cycle hors-sprint du type UX-ARRIVAL, soit comme S-numerote dedie).
2. **Passage par le cycle Cas B standard.** Comme toute phase de code : preflight G8 (4 scans S1-S4) AVANT le 1er Edit, review-deep post-code, Codex multi-round, gates lightcheck, commit atomique. Aucun raccourci.

**Position dans le process actuel.** Le prochain pas canonique reste : **S76 Phase 0 = audit gate S75** (`sprint76_audit_plan.md`, 13 tracks), PUIS kickoff S76 (GPU partage cross-machine). Le sharding est S77. Ce plan ne deplace rien de tout ca ; il decrit une capacite *additionnelle* prete a degainer si le PO la priorise.

**Renvoi vision.** Source : `.planning/research/vision_noeud_institutionnel_credentials_civiques.md`. Le MVP correspond a :
- **C12-lite** (§9, candidat C12) : comptabilite d'impact, mais cantonnee a l'agregation locale lecture-seule de ce qui est *deja enregistre*. **Re-decoupe apres revue adversariale : les compteurs blob-serve octets/pairs sont SORTIS du MVP** (cf. §Hors-perimetre — ils mesurent la mauvaise chose).
- **Niveau "Discret" du curseur** (§8.6, niveau 1) : `node_id` au transport, **zero claim public**, impact visible **en prive seulement**. Aucune publication, aucun curseur publie.
- **"Ton impact" prive** (§8.4 maquette + §8.5 trois classes de verite : prouvable / mesurable / inconnaissable). **Le MVP ne livre que la classe "prouvable" (compute du noeud) + l'etat de soutien actuel.** La classe "mesurable" (octets servis aux pairs) est renvoyee a C12 cote provider iroh-blobs.

**Note de dependance materielle (re-cadree post-revue).** §8.5 et la maquette §8.5 (cas Babel) decrivent "fenetre seul-hebergeur du 2 au 14 mai" et "octets servis a 9 pairs distincts". La revue adversariale a etabli **trois constats de code bloquants** qui re-decoupent le MVP :
1. **`/blob-serve` sert l'iframe du navigateur LOCAL, pas les pairs distants.** `blobServeUrl` construit l'URL sur le daemon local 127.0.0.1 (`web/src/api/daemon.ts:704`) et est consomme comme `src` d'iframe (`BrowsedProject.tsx:598`) ; la route est `.nest` sous `public_routes` hors auth, pensee pour le rendu local. Les octets servis AUX PAIRS transitent par le chemin **iroh-blobs provider** (`fetch_ticket` / `fetch_hash_multi` / `Downloader`), un sous-systeme TOTALEMENT distinct. Donc un compteur `bytes_served` sur blob-serve mesurerait *les octets que ton propre navigateur a tires pour rendre tes propres apps* — l'exact contraire de la semantique maquette "octets servis a 9 pairs". Le labelliser "octets servis (hebergement)" serait FAUX et trompeur.
2. **L'identite pair n'existe a aucun handler.** Le serveur tourne sur `axum::serve(listener, router)` (`runtime.rs:1054`) **SANS** `into_make_service_with_connect_info::<SocketAddr>()`. Aucun `ConnectInfo` n'est propage ; le handler `blob_serve` (`http.rs:2822`) extrait `State + Path` seuls. Capter "pairs distincts" exigerait de modifier le bootstrap serveur partage (tous les listeners TCP + UDS/NP) — une feature de plomberie serveur, pas un compteur additif in-primitive.
3. **Pas d'historique seeder.** `SeedRegistry` est EPHEMERE (jamais persiste, `seed_registry.rs:2-13`), `pinned_at` (M18) est ecrit mais jamais lu, aucune table `peer_count_log`/`seeder_presence_history`.

**Consequence (decision de re-decoupage).** Le MVP est re-base sur **le seul socle reellement prouvable et mesurable localement** : (a) le compute du noeud filtre par son identite worker (classe "prouvable" §8.5.1), et (b) l'etat de soutien ACTUEL (`keep_online` + seeders vus recemment, best-effort). **Les compteurs octets/pairs sont entierement scope-cut du MVP** ; ils reviennent comme feature distincte cote provider iroh-blobs (candidat **C12**, cf. §Roadmap), pas dans une phase MVP-en-un-commit.

---

## Objectif (goal-backward)

**Capacite utilisateur livree.** Un utilisateur ouvre une page privee "Ton impact" dans le shell et voit, en lecture seule, un resume honnete de ce que **son propre noeud** a permis, agrege depuis des donnees deja presentes localement :
- **Compute (classe prouvable)** : nombre de credits enregistres et kudos **que SON PROPRE noeud a calcules en tant que worker** (par projet et au total), depuis le registre kudos hash-chaine local, **filtre par `worker_node_id == self`** — jamais le compute d'autres workers que ce noeud aurait simplement orchestres.
- **Etat de soutien (classe mesurable, best-effort)** : apps que ce noeud garde en ligne (`keep_online`) et seeders distants vus recemment (`SeedRegistry.count_recent`), avec labels best-effort distincts.

**Hors MVP (re-decoupe) :** aucune metrique "octets servis" ni "pairs distincts" (mauvais layer + pas d'identite pair — cf. §Hors-perimetre). Aucune fenetre seul-hebergeur (historique absent).

**Comment on saura que c'est livre (criteres d'acceptance).**
1. `GET /api/daemon/impact` (loopback, auth triple) renvoie un JSON agrege bien forme, derivable **entierement de donnees locales** (kudos table filtree par `self_node_id` + keep_online table + SeedRegistry RAM), sans aucun fetch reseau, sans aucune lecture de contenu de tache, **sans aucun compteur blob-serve**.
2. Une route shell `/impact` (lazy) rend une vue lecture seule (pattern `KudosTab`) affichant les sections Compute (ton GPU) / Etat de soutien, avec labels d'honnetete ("calcule par ton noeud", "etat actuel best-effort").
3. La page affiche explicitement **"Visibilite : prive — toi seul vois cette page"** (maquette §8.4) et ne contient **aucun** bouton de publication actif (CTA "Publier une partie..." rendu inerte, jamais un button silencieux — verrou §8.7 UX).
4. Le total compute affiche **n'inclut JAMAIS** les credits d'un `worker_node_id != self` (verifie par test : seed de 2 worker_node_id distincts, assert que seul le self apparait).
5. Tests neufs verts : Rust nextest (`nexus-coordinator-rs` + `nexus-shell-daemon`), Vitest web, fmt/clippy/tsc/scan-en-strings tous verts.
6. **Zero changement wire** : aucun `*_FORMAT_VERSION` ni `*_ANNOUNCEMENT_VERSION` touche, aucun champ canonical signe modifie, aucune migration de schema SQLite.

---

## Perimetre MVP (ce qui EST dans la phase)

1. **Agregation impact cote coordinator, filtree par identite worker locale** : nouvelle fonction lecture-seule `impact_summary(db, self_node_id, now)` dans `nexus-coordinator-rs`. Elle agrege le compute **du noeud lui-meme** via `db.list_kudos_entries(Some(self_node_id))` (deja parametre par `worker_node_id`, `db.rs:1025-1032`) et `db.get_worker_kudos_total(self_node_id)` (`db.rs:594`), restreint par projet aux entrees dont `worker_node_id == self`. Joint l'etat `keep_online` via `list_keep_online_enabled` (`db.rs:768`). **Jamais `list_kudos_entries(None)`** (qui retourne TOUS les workers — sur-revendiquerait le compute d'autrui).
2. **Recuperation de l'identite worker du noeud** : `self_node_id` est lu depuis `DaemonHttpState.node_id` (`http.rs:76`, deja utilise par `seed_count`/`record_announced`) et passe a `impact_summary`. C'est la cle d'honnetete du verrou "seeder/compteur != auteur" transpose au compute.
3. **Route loopback `GET /api/daemon/impact`** : handler lecture-seule lisant `DaemonHttpState` (coordinator_db + seed_registry + node_id), assemblant la reponse agregee. Ajoutee DANS le builder `let authed_routes = Router::new()` (debut `http.rs:275`), au cote de `/api/daemon/nodes` / `/api/daemon/seed-count`. Aucune nouvelle couche d'auth — le middleware triple-check existant (`.merge(authed_routes)`) couvre.
4. **Vue shell `/impact`** : route React lazy (`App.tsx`), composant `Impact.tsx` (pattern read-only `KudosTab` : `UseQueryResult`, `Card`/`Badge`), helper `getImpact()` dans `daemon.ts` (pattern `callDaemon` + schema Zod `ImpactSchema` strict-envelope), strings francais inline (contrainte : pas d'i18n externe, scan-en-strings.sh).
5. **Labels d'honnetete obligatoires** dans l'UI : "calcule par ton noeud" (compute, prouve hash-chaine), "etat actuel best-effort, pas un historique" (soutien), `peer_count` avec son propre label "pairs ayant annonce seeder cette version, vus recemment — best-effort, peut sur-estimer". Banniere "prive — toi seul vois cette page". CTA publication inerte.

**Tient en UN commit de phase (encore plus vrai sans T1/T2)** : 1 fichier Rust modifie (`http.rs`) + 1 nouvelle fonction agregation (`nexus-coordinator-rs`, `impact.rs` ou a cote de `kudos_ledger`) + 1 route + 4 fichiers web (App.tsx, Impact.tsx neuf, daemon.ts, 1 test Vitest). **Aucun fichier `blob_serve.rs` touche.**

---

## Hors-perimetre / scope cuts

- **COMPTEURS OCTETS/PAIRS BLOB-SERVE — SCOPE-CUT INTEGRAL (re-decoupage post-revue, P1).** Trois raisons cumulees :
  1. **Mauvaise chose mesuree.** `/blob-serve` sert l'iframe du navigateur LOCAL (`blobServeUrl` -> 127.0.0.1, `daemon.ts:704` ; consomme `src` iframe `BrowsedProject.tsx:598` ; route `.nest` sous `public_routes` hors auth). Les octets servis aux PAIRS passent par le chemin provider iroh-blobs (`fetch_ticket`/`fetch_hash_multi`/`Downloader`), sous-systeme distinct. Un `bytes_served` blob-serve compterait *les consultations locales de l'utilisateur*, PAS l'hebergement — label "octets servis (hebergement)" = FAUX.
  2. **Pas d'identite pair.** `axum::serve(listener, router)` sans `into_make_service_with_connect_info` (`runtime.rs:1054`) : aucun `ConnectInfo` a aucun handler. "Pairs distincts" exigerait de modifier le bootstrap serveur partage (TCP + UDS/NP) — feature de plomberie, pas un compteur additif.
  3. **Pacte vie-privee §8.5 classe 3.** "Un reseau qui ne surveille pas ses lecteurs ne peut pas compter ses lecteurs." Capter `remote_addr` la ou il n'y en a aucun = instaurer une surveillance des lecteurs, frontiere a NE PAS franchir.
  **Routage : la vraie metrique "octets servis aux pairs" = instrumentation du chemin provider iroh-blobs (events upload/provide cote blobs store), candidat C12 ulterieur** (cf. §Roadmap), feature distincte non triviale (§9 la flag deja "events upload iroh-blobs a confirmer preflight"). Si jamais re-introduit, ce ne sera **jamais** une liste de `remote_addr` persistee/exfiltrable — uniquement un cardinal approximatif borne (HyperLogLog ou set cap+eviction), avec un test asserrant qu'aucune structure n'expose les identites individuelles ("compteur, jamais liste").
- **Aucun format wire** : zero nouveau `op` de feed, zero champ ajoute a `Task`/`ResultPayload`/`ProjectAnnouncement`/`NodeDirectoryEntry`/`CuratorList`. Aucun `*_FORMAT_VERSION` touche.
- **Aucune publication** : "Ton impact" est strictement local lecture-seule. Le CTA "Publier une partie..." (maquette §8.4) est **rendu inerte** (div non-cliquable + badge, jamais un button silencieux — verrou UX §8.7 / pattern S74-A AvailabilitySheet "Bientot"). La publication opt-in = niveaux 2+ du curseur = **C13, phase ulterieure**.
- **Aucun cross-node** : aucune lecture de l'impact d'un autre noeud, aucune agregation distante, aucun fetch. Tout vient du SQLite local + caches RAM locaux.
- **Aucun curseur de visibilite publie** : le MVP est fige au niveau "Discret" (§8.6 niveau 1). Le selecteur 5-positions = **C13**, hors MVP.
- **Aucune lecture de contenu de tache** : on n'affiche jamais `result_text` ni `prompt`. Uniquement compte, metadonnees non-contenu (kudos amount, created_at), et hash. (Cf. §8.5 classe 1 : "le compte des traductions, jamais leur contenu".)
- **Aucune vue "compute orchestre par mon noeud" (requester)** dans le MVP : le compute affiche = uniquement ce que MON GPU a produit (`worker_node_id == self`). Si le PO veut AUSSI une vue requester ("compute que j'ai permis en orchestrant en tant que coordinateur"), c'est une 2e section distincte, label non ambigu ("valide par ton noeud" vs "calcule par toi"), **jamais fusionnee sous 'ton impact'** — a router en decision PO (cf. §Decisions PO #2).
- **Aucune attestation / reconnaissance signee tierce** : la section "Reconnaissance" / "merci signe" de la maquette §8.4 = **C11**, hors MVP. C11 a DEUX gates dependance (cf. §Roadmap) : (a) convergence cross-node `SeedAnnounced` ET (b) primitive d'ecriture d'op de feed **C1 `feed_append_app_op`** qui n'existe pas dans la whitelist bridge (`web/src/bridge/protocol.ts` : 15 methodes, AUCUNE `feed_append`/signature). Sans C1, une attestation ne peut meme pas etre EMISE.
- **FENETRE SEUL-HEBERGEUR SCOPE-CUT** : la maquette §8.5 ("seul hebergeur du 2 au 14 mai : 4 acces n'ont existe que par toi") est **infaisable en lecture seule**. `SeedRegistry` est EPHEMERE (jamais persiste, `seed_registry.rs:2-13`), `pinned_at` (M18) ecrit mais **jamais lu**, et **aucune table d'historique** `peer_count_log`/`seeder_presence_history`. Detecter "fenetre seul-hebergeur" exigerait de persister des snapshots `(peer_count, timestamp)` ou une nouvelle table = nouveau schema, hors MVP lecture-seule. Le MVP affiche l'etat de soutien *actuel* ("tu seedes N apps maintenant"), jamais une fenetre historique. **Routage : C12 phase ulterieure (persistance snapshots).**
- **Aucune granularite GPU par projet/editeur** : C9 (allowlist worker par editeur) hors MVP. Le compute est agrege par `project_id` deja present sur `kudos`, sans nouvelle granularite. **C9 reste un candidat independant non sequence dans cette serie impact/visibilite — a reprendre au kickoff GPU S76** (et non oublie ; cf. §Roadmap note).
- **Etiquetage app des taches** : §8.5 classe 1 note "verifier l'etiquetage par app des taches pour l'agregation par projet". L'agregation par projet repose sur `kudos.project_id` (deja present). Si l'etiquetage est fiable, OK ; sinon le MVP agrege par `project_id` tel quel sans pretendre a une granularite app-level superieure. **Question purement technique -> tranchee au preflight reel (PLAN-ADAPT), pas PO.**

---

## Decoupage en taches atomiques (ordonne)

> **Re-decoupage post-revue : T1 et T2 (compteurs blob-serve) sont SUPPRIMES.** Le MVP commence directement a l'agregation kudos filtree. La numerotation est resequencee T1->T4.

### T1 — Agregation impact cote coordinator (lecture seule, filtree par worker local)
- **Fichiers** : `crates/nexus-coordinator-rs/src/kudos_ledger.rs` (a cote de `get_project_kudos` ligne 134) OU nouveau module `impact.rs` dans `nexus-coordinator-rs`. Reutilise `db.list_kudos_entries(Some(self_node_id))` (`db.rs:1025`, branche `worker_node_id = ?1`), `db.get_worker_kudos_total(self_node_id)` (`db.rs:594`), `db.list_keep_online_enabled` (`db.rs:768`), `db.get_keep_online` (`db.rs:725`). `effective_score()` (EMA, `kudos_ledger.rs:124`) pour la recence.
- **Fait** : fonction `pub fn impact_summary(db, self_node_id: &str, now_secs) -> ImpactSummary`. Structure `ImpactSummary { per_project: Vec<ProjectImpact>, totals: ImpactTotals }` ou `ProjectImpact { project_id, credits_count, kudos_lifetime, kudos_effective_now, kept_online: bool, archive_hash: Option<String> }` et `ImpactTotals { projects_count, credits_count, kudos_lifetime, kudos_effective_now, kept_online_count }`. **Cle d'honnetete** : l'agregation par projet ne compte QUE les entrees `worker_node_id == self_node_id` (le compute que ce noeud a reellement calcule, §8.5 classe 1 "ton GPU a produit N resultats"). `effective_kudos` reutilise `effective_score()` (coherence philosophie recence §8.4). `credits_count` = nombre d'entrees kudos du self par projet (label honnete "credits enregistres", PAS "N taches" : une tache peut produire 0 entree si rejetee/guardrail, granularite credit pas garantie 1:1). **Aucune lecture de `result_text`.**
- **Test** : `nexus-coordinator-rs` — `impact_summary_aggregates_across_projects` : **seed des credits de 2 `worker_node_id` differents (self + un autre)**, assert que **seul le self apparait** dans les totaux et `per_project` (preuve du filtre anti-sur-revendication), `per_project.len()==2` pour les projets du self, `kept_online` reflete `set_keep_online`. Test `impact_summary_empty_is_well_formed` (noeud vierge -> structure vide, pas d'erreur).

### T2 — Route loopback GET /api/daemon/impact
- **Fichiers** : `crates/nexus-shell-daemon/src/http.rs` (handler neuf calque sur `seed_count` ligne 2284 ; enregistrement dans le builder `let authed_routes = Router::new()` debutant **`http.rs:275`**, au cote de `/api/daemon/nodes` / `/api/daemon/seed-count` — ancrage par NOM du bloc, pas par numero de ligne ; le bloc est `.merge(authed_routes)` ligne ~492).
- **Fait** : handler `async fn daemon_impact(State<Arc<DaemonHttpState>>) -> impl IntoResponse`. Lit `state.node_id` (`http.rs:76`), lock `coordinator_db`, appelle `impact_summary(db, &state.node_id, now)`, lit `state.seed_registry.count_recent(...)` pour l'etat de soutien actuel par app gardee en ligne. Assemble `Json(serde_json::json!({"impact": {...}}))`. Pattern envelope existant. **Route additive — n'altere aucune reponse existante** (`/browse` byte-identique).
- **Test** : `nexus-shell-daemon` — `daemon_impact_route_returns_aggregate` (integration handler, pattern tests `kudos_api`) : monte un state minimal, GET impact, assert structure JSON + status 200. Le preflight S4 doit re-verifier que la route finit **sous `.merge(authed_routes)`** et passe le triple-check.

### T3 — Helper front daemon.ts + schema Zod
- **Fichiers** : `web/src/api/daemon.ts` (helper `getImpact()` calque sur `getDaemonInfo` ligne 305 / `listBrowse` 338 ; schema `ImpactSchema` calque sur les schemas strict-envelope existants, ex. `BrowsePullResponseSchema` ligne 344 `.strict()`).
- **Fait** : `export function getImpact(baseUrl): Promise<DaemonResult<Impact>>` via `callDaemon(baseUrl, "/api/daemon/impact", ImpactSchema)`. `ImpactSchema = z.object({ impact: z.object({...}) }).strict()` (envelope strict ; champs internes tolerants selon politique pre-launch runtime). **Aucun champ `bytes_served`/`distinct_peers` dans le schema** (scope-cut). Type `Impact` exporte.
- **Test** : `web` Vitest — `daemon.impact.test.ts` : mock fetch renvoyant un payload valide -> assert parse OK + `kind: "data"` ; payload malforme -> `ApiProtocolError`.

### T4 — Vue shell /impact (lecture seule)
- **Fichiers** : `web/src/App.tsx` (route lazy `/impact`, pattern code-split lignes 39-84) ; `web/src/pages/Impact.tsx` (neuf, pattern read-only `KudosTab.tsx`).
- **Fait** : route `/impact` lazy. Composant `Impact` : `useQuery(["daemon-impact", coordUrl], () => getImpact(coordUrl))`, etats loading/error/data (pattern `KudosTab` query.isLoading/isError). Sections : **Compute** (credits par projet + total, "calcule par ton noeud"), **Etat de soutien** (apps gardees en ligne + `peer_count` avec son label honnete "pairs ayant annonce seeder cette version, vus recemment — best-effort, peut sur-estimer"). **Aucune section "octets servis" ni "pairs distincts"** (scope-cut). Banniere **"Visibilite : prive — toi seul vois cette page"**. CTA **"Publier une partie..."** rendu **inerte** (div non-cliquable + badge "Bientot", JAMAIS un button — verrou §8.7). Strings francais inline.
- **Test** : `web` Vitest — `Impact.test.tsx` : rend avec query mockee data -> assert sections presentes + banniere prive + CTA inerte non-cliquable (assert pas de `<button>` actif sur "Publier"). Render avec query error -> ErrorCard.

---

## Contrat de donnees de la route impact (re-base sur le perimetre net)

`GET /api/daemon/impact` (loopback, auth triple X-SBFB-Token + Host + Origin). Reponse :

```json
{
  "impact": {
    "compute": {
      "per_project": [
        {
          "project_id": "ideas-hub",
          "credits_count": 312,
          "kudos_lifetime": 84210,
          "kudos_effective_now": 51200,
          "label": "calcule par ton noeud (registre kudos local, worker = toi)"
        }
      ],
      "totals": {
        "projects_count": 3,
        "credits_count": 470,
        "kudos_lifetime": 120300,
        "kudos_effective_now": 73400
      },
      "scope_label": "uniquement le compute que TON noeud a calcule (worker_node_id == ton identite) ; jamais le compute d'autres workers"
    },
    "support_now": {
      "kept_online": [
        { "project_id": "ideas-hub", "archive_hash": "abc123...", "kept_online": true }
      ],
      "kept_online_count": 2,
      "seeding_projects": [
        {
          "project_id": "ideas-hub",
          "archive_hash": "abc123...",
          "peer_count": 3,
          "peer_count_label": "pairs ayant annonce seeder cette version, vus recemment — best-effort, peut sur-estimer ; jamais une preuve de disponibilite (seul le content-addressing l'est)"
        }
      ],
      "label": "etat actuel best-effort, pas un historique"
    },
    "notes": {
      "visibility": "prive — agregation locale, jamais publiee",
      "honesty": "compute = recus signes hash-chaines de TON noeud (prouvable) ; soutien = etat local + annonces seeder best-effort (mesurable, peut sur-estimer)"
    }
  }
}
```

**Re-base post-revue** : la section `hosting` avec `bytes_served`/`distinct_peers`/`requests_total` est **RETIREE du contrat** (mauvais layer + pas d'identite pair + pacte §8.5 classe 3). L'etat d'hebergement est fusionne dans `support_now` (ce que ce noeud garde en ligne + seeders distants vus recemment).

**Labels d'honnetete cables dans le payload** (pas seulement l'UI) :
- Compute = "calcule par ton noeud", filtre `worker_node_id == self` (prouvable, hash-chaine — §8.5 classe 1). Le `scope_label` rend explicite que le compute d'autrui n'est jamais compte.
- `peer_count` = label propre, distinct du label de ligne, herite explicitement du caractere best-effort/forgeable du `SeedRegistry` (`count_recent` peut OVER-state) ; jamais presente comme preuve de disponibilite.
- Pas de fenetre seul-hebergeur, pas d'octets/pairs (scope-cut), pas d'attestations tierces, pas de score comparable cross-node (jamais de classement — verrou §8.4).

---

## Wire-safety + check des 5 verrous

**Changement canonical : ZERO.** Aucun champ signe (`Task`, `ResultPayload`, `KudosEntry` hashable, `ProjectAnnouncement`, `NodeDirectoryEntry`, `CuratorList`, `SignedList`) n'est modifie. Aucun `canonical_bytes`/`DOMAIN_*` touche. Aucun `*_FORMAT_VERSION` ni `*_ANNOUNCEMENT_VERSION` bump. La table SQLite `kudos` et `keep_online` sont **lues** sans modification de schema (pas de migration M-N). **Aucun fichier blob-serve touche** (compteurs scope-cut). Politique pre-launch respectee trivialement : on ne touche pas au canonical.

**Les 5 verrous anti-recentralisation — chacun OK car lecture purement locale :**
1. **Pas de defaut pre-installe / pas de gate d'admission** : "Ton impact" est une vue passive ; elle n'ajoute aucun curateur par defaut, aucune ancre, aucun abonnement. OK.
2. **Additive, jamais substitutive** : la page n'exige rien, ne conditionne aucun acces a aucun contenu. Lecture seule de l'existant. OK (§8.3 verrou 2).
3. **Volontaire / abonnement volontaire** : aucune contribution declaree ou activee par cette page (le toggle keep_online vit ailleurs, AvailabilitySheet) ; "Ton impact" ne fait que *refleter* l'etat existant. Aucun profil de contribution rempli par defaut. OK.
4. **Redondance additive / provenance auteur — RENFORCE par le filtre worker** : la page ne republie rien, n'usurpe aucune autorite editoriale. "seeder != auteur" tenu, **et "compteur != auteur" tenu** : le compute affiche est filtre par `worker_node_id == self`, donc il reflete ce que MON GPU a calcule, jamais le travail d'autres workers que ce noeud aurait orchestre. OK (correction P1 wire_verrous + completude).
5. **Subscribed-only / pas de claim public** : **zero publication, zero cross-node**. L'impact ne quitte jamais le loopback. Niveau "Discret" (§8.6.1) : aucun claim signe, aucune annonce. Le CTA "Publier" est inerte. OK.

**Kudos non-monetaire** : on lit `amount` (log_utility tokens) et `effective_score` (EMA) tels quels. **Aucune conversion**, aucun lexique cost/stake/burn/refund/achat introduit. L'impact n'achete ni privilege ni priorite (§8.4 garde-fou). Le code et les strings UI evitent tout vocabulaire monetaire (verifie par revue manuelle + esprit `feedback_kudos_non_monetary`).

**Best-effort, claim != preuve** : le compute (kudos du self) est labellise "prouvable" car hash-chaine signe. Le `peer_count` seeder est labellise "best-effort, peut sur-estimer" et n'est jamais une autorite. Distinction §8.5 cablee dans le contrat.

**UX intentions sans jargon** : la page parle de "ce que ton noeud a calcule", "apps gardees en ligne" — jamais `fetch_and_pin`, `keep_online`, `M18`, `SeedRegistry`, `blob-serve`, `worker_node_id` a l'ecran.

---

## Plan de test (commandes reelles §7.4)

**Rust** (iteration ciblee puis verification complete) :
```bash
cargo nextest run -p nexus-coordinator-rs --locked       # T1 impact_summary (filtre worker)
cargo nextest run -p nexus-shell-daemon --locked         # T2 route impact
# verification finale avant commit phase :
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --locked
cargo test --workspace --locked --doc
cargo build -p nexus-shell-daemon --release
```

**Frontend** :
```bash
cd web && npm run lint && \
  npx tsc --noEmit -p tsconfig.app.json && \
  npm run test:unit && npm run test:coverage && \
  npm run build && npm run size && \
  bash scripts/scan-en-strings.sh
```

**Tests neufs Rust** :
- `nexus-coordinator-rs` : `impact_summary_aggregates_across_projects` (**inclut le scenario 2-worker_node_id : assert que le compute d'autrui n'apparait pas**), `impact_summary_empty_is_well_formed`.
- `nexus-shell-daemon` : `daemon_impact_route_returns_aggregate` (integration handler, status 200 + structure, route sous `authed_routes`).

**Tests neufs Vitest** :
- `daemon.impact.test.ts` : parse OK / `ApiProtocolError` sur payload malforme.
- `Impact.test.tsx` : sections rendues + banniere prive + CTA inerte (assert pas de button actif "Publier") + etat error.

**Delta tests cumule estime (re-estime apres scope-cut T1/T2)** : Rust **+3 a +4** (au lieu de +5/+6 — plus de tests blob-serve), Vitest +2 a +4 (selon granularite des cas). Couverture web a verifier >= seuils existants (85/85/78/85). `scan-en-strings.sh` doit rester vert (strings UI francais).

---

## Pre-evaluation preflight G8 (4 scans S1-S4)

> Rappel : ceci est une **pre-evaluation anticipee**. Le preflight reel se fait via le skill `nexus-phase-preflight` AVANT le 1er Edit, et peut conclure differemment.

- **S1 — SOTA delta** : aucune dependance nouvelle, aucune lib/API externe touchee. `AtomicU*` non requis (compteurs blob-serve scope-cut), Axum routing (deja present), Zod/React Query (deja present web). Pas de context7 requis (pas de nouvelle lib). **Verdict anticipe : RAS.**
- **S2 — decisions historiques traversees** : (a) **kudos non-monetaire** (`feedback_kudos_non_monetary`) — respecte (lecture seule, zero conversion). (b) **politique pre-launch wire** — respectee (zero bump, zero migration). (c) **UX intentions sans jargon** (D17 / S70 canon) — respecte. (d) **§8.3/§8.4 garde-fous vision** — respecte (prive, pas de classement, pas de conversion). (e) **verrou "seeder/compteur != auteur"** — respecte via le filtre `worker_node_id == self`. **Verdict anticipe : aucune decision violee.**
- **S3 — threat model** : surface = nouvelle route loopback lecture-seule sous auth triple existante. Pas de nouvelle surface d'exposition reseau ; jamais de cross-node ; pas de PII (on ne lit ni `result_text` ni `remote_addr`). **Le seul point sensible de la version precedente (capture identite pair blob-serve) est ELIMINE par le scope-cut** : le MVP ne touche plus blob-serve, donc le pacte vie-privee §8.5 classe 3 n'est plus en tension. **Verdict anticipe : couvert, CLEAN** (plus de condition residuelle — le scope-cut a ferme le point S3).
- **S4 — wire invariants** : zero changement canonical (cf. §Wire-safety). Aucun producteur/consommateur de format signe touche. Le preflight S4 reel re-verifie que la route `/api/daemon/impact` finit sous `.merge(authed_routes)`. **Verdict anticipe : invariants intacts.**

**Verdict global anticipe : EXECUTE (plein, non conditionnel).** Justification : apres le re-decoupage qui sort les compteurs blob-serve, la feature est purement additive, locale, lecture-seule ; aucune decision gelee violee ; **le seul point S3 conditionnel de la version precedente (`distinct_peers` / capture identite pair) est resolu en amont par le scope-cut**, pas reporte au preflight reel. Il ne reste aucun `[A CONFIRMER PREFLIGHT]` materiel touchant le threat model. La seule question technique residuelle (etiquetage app via `kudos.project_id`) est un PLAN-ADAPT pur (arbitre = code/recherche, pas PO).

> **Note frontiere preflight vs PO (correction process_conformance P2).** Le preflight G8 tranche le "comment" technique sans user (PLAN-ADAPT). Seul un DESIGN-CONFLICT (Day-0/wire/threat) escalade au PO. Dans ce plan : (1) les questions threat-model/produit -> PO = insertion roadmap, vue requester eventuelle, lexique non-monetaire (cf. §Decisions PO) ; (2) la question purement technique "etiquetage app via `kudos.project_id` suffisant ?" -> preflight reel SANS PO. Aucun item n'est route a la fois vers preflight ET PO.

---

## Forme du commit

**Titre** : `feat(impact): Sprint N Phase X — Ton impact (prive) MVP lecture-seule`

(scope `impact` ; le N/X depend de l'insertion roadmap decidee par le PO.)

**Note vehicule d'insertion (correction process_conformance P3).** Selon le vehicule choisi par le PO (cf. §Decisions PO #1), la forme exacte du commit et les gates applicables different :
- **Si mini-cycle hors-sprint (modele UX-ARRIVAL)** : le titre portera un identifiant de mini-cycle resolu (ex. `Mini-cycle IMPACT-MVP`) accepte par le hook `phase-precommit-lightcheck`. UX-ARRIVAL a tourne en PROCESS LEAN/PLAN-ADAPT (hors gates classiques) — si ce vehicule est retenu, **nommer explicitement les gates reellement appliques** (au minimum : preflight G8 + lightcheck + tests verts ; review-deep/Codex au jugement PO comme pour UX-ARRIVAL).
- **Si phase d'un sprint (Cas B standard, Phase A)** : le hook lightcheck (Check 5/9) exige un `Sprint N Phase X` resolu ET, pour une Phase A, l'existence d'un `sprint{N}_design_review.md` OU le statut **`G1 skipped`**. Comme ce MVP n'introduit **aucune decision Day-0 nouvelle** (feature additive lecture-seule, 0 wire, 0 dep), `G1 skipped` est le statut probable et legitime.
- **Tension levee** : le plan ne promet PAS simultanement "modele UX-ARRIVAL lean" et "cycle Cas B standard sans raccourci". Le PO choisit l'un OU l'autre au moment de l'insertion ; ce document liste les gates de chaque branche pour qu'aucun ne soit improvise.

**Body — 9 sections :**
1. **Contexte** : renvoi vision C12-lite + §8.4-8.6 niveau Discret ; proposition inseree par decision PO ; re-decoupage post-revue (compteurs blob-serve sortis).
2. **Agregation coordinator filtree worker** : `impact_summary(db, self_node_id, now)` lecture-seule, `list_kudos_entries(Some(self))`, EMA recence, filtre anti-sur-revendication.
3. **Identite worker** : `self_node_id` lu depuis `DaemonHttpState.node_id`, cle du verrou "compteur != auteur".
4. **Route loopback** : `GET /api/daemon/impact` sous auth triple (`authed_routes`), additive, byte-identique pour les routes existantes.
5. **Front /impact** : route lazy + `Impact.tsx` read-only + `getImpact()` + `ImpactSchema` strict, strings FR, CTA publication inerte, banniere prive.
6. **Wire-safety** : zero canonical, zero bump, zero migration, 5 verrous OK (lecture locale + filtre worker), kudos non-monetaire intact.
7. **Scope cuts** : compteurs octets/pairs blob-serve (mauvais layer + pas d'identite pair + pacte §8.5.3 -> C12 provider), fenetre seul-hebergeur (historique absent), publication/curseur/cross-node/attestations (C1/C11/C13/C14 ulterieurs), contenu de tache jamais lu, vue requester optionnelle differee.
8. **Tests** : commandes §7.4 toutes vertes ; tests neufs enumeres (dont scenario 2-worker_node_id).
9. **Delta tests cumule** : `nextest Win <avant> -> <apres> (+R)` / `Docker <...>`, `Vitest <avant> -> <apres> (+V)`, coverage >= seuils, size 6/6, 0 bump wire, 0 dep.

---

## Roadmap des phases suivantes (annexe vision — NON planifiee, NON candidate a execution immediate)

> **Statut de cette annexe (correction scope_realisme P3).** SEUL le MVP ci-dessus (perimetre net : T1-T4) est candidat a execution maintenant. La serie B-F ci-dessous est de la **doc d'orientation vision**, PAS du scope MVP. Chaque phase B-F depend de decisions PO + du fix de propagation feed + de son propre preflight S4. Aucune action de code requise par cette section.

> **Prerequis transversal a poser tot : la fiabilisation de la propagation feed cross-node** (bug `SeedAnnounced` non-converge, `peer_count:0 ~10 min`, observe a l'acceptance S75, deja route dans `sprint76_audit_plan.md`). **Ce prerequis gate toute phase qui publie ou agrege une op de feed cross-node** — notamment C3 (agregation cross-node) et C11/E (attestation). Tant qu'il n'est pas resolu, les phases B+ qui dependent de la propagation restent bloquees ou en mode "transport humain" (§2.3).

> **Mapping candidat — couverture de la vision (correction completude P1).** Cette serie B-F couvre **la moitie §8 de la vision** (impact/visibilite : C7/C8/C10/C11/C13/C14). Les candidats **civiques §1-3 NE SONT PAS couverts par ce plan** et sont listes ici pour transparence du mapping :
> - **C1 — `feed_append_app_op`** (primitive d'ecriture d'op de feed via bridge) : **prerequis DUR de C11/E** (sans elle, aucune attestation ne peut etre EMISE ; absente de la whitelist bridge `protocol.ts`, 15 methodes). Doit etre planifie EN AMONT de E. Voir ligne dediee dans le tableau.
> - **C2** (signature bridge), **C4** (canal service authentifie), **C5** (doc certificat), **C6** (credentials anonymes) : **serie civique §1-3, HORS ce plan contribution/visibilite.** A router separement (kickoff civique dedie), non sequences ici. Le silence n'est pas un oubli : ils sont explicitement exclus de cette serie produit.
> - **C3 — agregation cross-node** : nomme comme gate-par-propagation dans le prerequis ci-dessus ; depend de C1 (lire/ecrire des ops de feed). Voir ligne dediee.

| Phase | Candidat vision | Resume | Dependances | Surface wire | Prerequis propagation feed |
|---|---|---|---|---|---|
| **(amont E)** | **C1 — `feed_append_app_op`** | Primitive bridge d'ECRITURE d'op de feed (signature incluse) ; aujourd'hui ABSENTE de la whitelist `protocol.ts`. Sans elle, aucune attestation (C11) ni agregation cross-node (C3) ne peut etre emise. | Bridge whitelist + signature (cf. C2 si signature deleguee). | **Methode bridge neuve** + raw-op feed (0-bump enveloppe `FeedEntry`). | Pour l'EMISSION : non (ecrit local). Pour l'EFFET cross-node : oui (gate par propagation). |
| **(amont E)** | **C3 — agregation cross-node** | Lire et agreger des ops de feed (contributions/attestations) emises par d'AUTRES noeuds. | **C1** (primitive d'op de feed) + propagation. | Lecture d'ops de feed signees existantes ; potentiellement extension bridge. | **Oui — gate dur** (agreger l'op d'un autre noeud suppose sa propagation fiable). |
| **B** | **C7 — Mirror one-click** | Enumerer l'annuaire ingere -> `fetch_and_pin_multi` par app, caps (Mo, N apps) DANS la primitive (§P59). | Primitives S74/S75 (`fetch_and_pin_multi`, annuaire ingere) deja la. | **Aucune** (orchestration de primitives locales existantes). | Non — purement local (pin de ce qu'on a deja ingere). |
| **C** | **C8 — Profil de contribution + "Ce que tu donnes"** | A l'abonnement, declarer seed-catalogue (caps) / ancre / GPU ; vue revocabilite "Ce que tu donnes" (maquette §8.7) sous Network ; stocke config locale, publication opt-in SEPAREE. | C7 (mirror) ; decision PO lexique non-monetaire. | Config locale = aucune ; publication opt-in = claim signe -> wire (raw-op feed, 0-bump). | Pour la **partie config locale** : non. Pour la **publication opt-in** : oui (gate). |
| **D** | **C10 — Casier protocolaire** | Onglet "Historique" de noeud : timeline signee verifiable (feed hash-chaine + annonces + provenance ; anti-rollback, equivocation exhibable). 3 formes : onglet `/node/:id`, app SBFB type explorer, observatoire *parmi N*. | Extension de `feed_cursor_get` au cross-node : **la methode existe deja en lecture LOCALE** (`protocol.ts:39`) ; la nouveaute est l'**exposition cross-node + octets canoniques au bridge** [A CONFIRMER PREFLIGHT]. | Lecture de feed signe existant ; extension bridge `feed_cursor_get` cross-node. | **Oui** — lire le feed d'un autre noeud suppose sa propagation fiable. |
| **E** | **C11 — Attestation de contribution** | Recu structure signe par le beneficiaire { contributeur pubkey/anonyme, app/hash, periode, volumes }, raw-op feed 0-bump, non-transferable, affiche dans "Ton impact" + casier. | **DEUX gates dependance** : (a) **C1 `feed_append_app_op`** (sans elle, l'attestation ne peut meme pas etre EMISE) ET (b) decision PO lexique (cousin kudos, memes interdits) ; **C12** pour les volumes (fenetre seul-hebergeur). | **Raw-op feed** (`ContributionAttested`/equiv), 0-bump `FEED_FORMAT_VERSION` (politique pre-launch). | **OUI — gate dur.** Une attestation signee par A vers B doit atteindre B : depend de la convergence cross-node ET de C1. **Bloque tant que SeedAnnounced non resolu ET tant que C1 absente.** |
| **(C12 provider)** | **C12 — Octets servis aux pairs + fenetre seul-hebergeur** | Instrumentation du chemin **provider iroh-blobs** (events upload/provide cote blobs store) pour les octets reellement servis aux pairs + persistance de snapshots `(peer_count, timestamp)` pour la fenetre seul-hebergeur. **C'est la vraie metrique d'hebergement** que le MVP a du scope-cut (blob-serve = mauvais layer). | Events upload iroh-blobs (§9 "a confirmer preflight") + nouveau schema snapshots. | Aucune sur le wire P2P si purement local ; cardinal pairs = HyperLogLog/set borne, **jamais liste de remote_addr**. | Non (mesure locale du provider) — mais feature non triviale, hors MVP-un-commit. |
| **F (split)** | **C13 — Curseur visibilite** | Selecteur 5 positions par noeud (anonyme->identifie) orchestrant claims opt-in / Keyoxide multi-forge / attestation civile §3 / Tor niveau 0 ; exceptions par audience ; avertissements portes-a-sens-unique. | C8/C11 ; UX avertissements bloquants. | Config daemon (niveau) + activation des claims (chaque claim publie = wire selon le niveau). | Partiel — la *config* du niveau est locale ; l'*effet* (publier des claims) depend de la propagation. |
| **F (split)** | **C14 — Drapeau visibilite tache + lecture publique Babel** | Drapeau par tache compute choisi par le demandeur : privee (recu-par-hash + metadonnees) vs publique (contenu liable -> panneau impact cliquable, campagnes corpus publics). | Etiquetage app des taches (C12) ; **decision PO surface wire `Task`**. | **Oui — champ wire `Task`** (drapeau visibilite). **Premier vrai changement canonical de cette serie.** | Non directement — mais la *lecture publique* cross-node depend de la propagation. |

**Note canonical pre-launch (correction process_conformance P3).** Pour C14 (champ wire `Task`) et tout changement canonical de cette serie : **la politique pre-launch s'applique** — redefinition v1 **in-place** du canonical (PAS un simple `serde(default)` propage comme verite historique), suppression immediate des tests legacy-decode, **pas de bump** de `*_FORMAT_VERSION`. Le "0-bump" cite pour C1/C8/C11 (ajout d'`op` de feed) n'est PAS un blanc-seing : chaque phase wire repasse son propre preflight S4 et re-verifie `canonical_bytes`/`DOMAIN_*` au moment de la phase.

**Note de coherence sur l'ordre** : B (C7) et la partie locale de C (C8) sont faisables **sans** attendre la propagation feed (purement locales). C1 (primitive d'ecriture) est un prerequis amont de C3 et E. D (C10), E (C11) et les effets publics de C/F **dependent du prerequis propagation** (`SeedAnnounced`) ET, pour E, de C1. Recommandation : programmer B et la config-locale de C juste apres ce MVP ; livrer C1 avant E ; reserver E (C11, le "merci signe" qui rend la maquette §8.4 complete) **apres** la resolution du bug de convergence ET la livraison de C1 — sinon les attestations partent dans le vide.

---

## Decisions PO ouvertes (a trancher avant execution)

> Frontiere process : ces items sont **gouvernance/produit/threat-model -> PO**. Les questions purement techniques (etiquetage app via `kudos.project_id`, faisabilite des getters, placement exact de route) sont tranchees au **preflight reel sans PO** (PLAN-ADAPT).

1. **Insertion roadmap** : ce MVP entre-t-il comme phase d'un sprint existant, mini-cycle hors-sprint (modele UX-ARRIVAL), ou S-numerote dedie ? Ce choix determine aussi la forme du commit et les gates (cf. §Forme du commit). **Bloquant — rien ne s'execute sans cette decision.**
2. **Vue compute : self-only ou aussi requester ?** Le MVP affiche UNIQUEMENT le compute que TON GPU a calcule (`worker_node_id == self`, §8.5 classe 1 "ton GPU a produit N resultats"). Le PO veut-il AUSSI une 2e section distincte "compute que j'ai permis en orchestrant en tant que coordinateur" (les credits que ce noeud a octroyes a d'autres workers) ? Si OUI : section separee, label non ambigu ("valide par ton noeud" vs "calcule par toi"), **jamais fusionnee sous 'ton impact'**. **Defaut recommande : self-only au MVP** (le plus honnete, le moins ambigu).
3. **Fenetre seul-hebergeur + octets servis aux pairs (C12)** : confirmer le scope-cut MVP. Les VRAIES metriques d'hebergement (octets servis aux pairs, fenetre seul-hebergeur) exigent (a) l'instrumentation du chemin provider iroh-blobs et (b) la persistance de snapshots `(peer_count, timestamp)` = nouveau schema -> **C12 ulterieur, pas ce MVP**. Si le PO juge ces metriques essentielles, elles forment une phase C12 dediee non triviale.
4. **CTA "Publier une partie..."** : confirmer qu'il reste **inerte** au MVP (verrou §8.7) et que la publication (niveaux 2+ du curseur) est explicitement renvoyee a C13. Pas de bouton silencieux.
5. **Lexique** : valider qu'aucune etiquette UI/JSON n'introduit de vocabulaire monetaire ni de "score comparable" (jamais de classement — §8.4).
6. **Series civiques §1-3 (C2/C4/C5/C6) et C1** : confirmer qu'elles sont hors de cette serie produit impact/visibilite et routees a un kickoff civique dedie. **C1 (`feed_append_app_op`)** est neanmoins un prerequis DUR de C11/E (attestation) — le PO doit savoir que E ne peut PAS se livrer sans C1, qui n'existe pas aujourd'hui.

---

## Journal de revue

> Revue adversariale 4 axes : scope_realisme, wire_verrous_safety, process_conformance, completude_vs_vision. Tous les P1 traites ; P2 traites quand pertinents ; P3 statues. Verification de code faite avant correction (claims des reviewers confirmes).

### Findings P1 traites (obligatoires)

- **[scope_realisme P1 #1 — blob-serve mesure la mauvaise chose]** TRAITE par re-decoupage : T1/T2 (compteurs blob-serve octets/pairs) **entierement sortis du MVP**. Verifie en code : `blobServeUrl` -> 127.0.0.1 (`daemon.ts:704`), consomme `src` iframe (`BrowsedProject.tsx:598`), route `.nest` sous `public_routes`. Les octets aux pairs passent par le provider iroh-blobs (chemin distinct). La vraie metrique = candidat C12 cote provider, route en annexe. Le contrat de donnees retire `hosting.bytes_served/distinct_peers/requests_total`.
- **[scope_realisme P1 #2 / wire P1 #2 / completude P1 — `distinct_peers` = feature plomberie + pacte vie-privee]** TRAITE : `distinct_peers` scope-cut **par defaut et de maniere assumee** (plus en option [A CONFIRMER PREFLIGHT]). Raison technique exacte inscrite : `axum::serve(listener, router)` sans `into_make_service_with_connect_info` (`runtime.rs:1054`) -> aucun `ConnectInfo` a aucun handler ; capter une identite pair = modifier le bootstrap serveur partage. "Pairs distincts" documente comme C12 cote provider. Garde-fou "compteur, jamais liste" pose pour toute re-introduction future.
- **[wire P1 #1 / completude P1 — melange seeder/auteur dans compute : `list_kudos_entries(None)` sur-revendique le compute d'autrui]** TRAITE, c'est la correction de fond la plus importante. Verifie en code : la table `kudos` est keyee par `worker_node_id` (`db.rs:578-588`, `credit` credite le worker qui a calcule) ; `list_kudos_entries(None)` retourne TOUS les workers ; `list_kudos_entries(Some(worker_node_id))` (`db.rs:1025-1032`) et `get_worker_kudos_total(worker_node_id)` (`db.rs:594`) existent deja. Correction : `impact_summary` prend `self_node_id` (lu depuis `DaemonHttpState.node_id`, `http.rs:76`) et filtre `worker_node_id == self`. Test `impact_summary_aggregates_across_projects` etendu au scenario 2-worker_node_id (assert que le compute d'autrui n'apparait pas). Verrou 4 reformule "compteur != auteur".
- **[wire P1 #2 — pacte vie-privee §8.5 classe 3 si `distinct_peers` retenu]** TRAITE : scope-cut `distinct_peers` devenu le DEFAUT (decision PO #2 honoree), retire du contrat de reference et des taches. Point S3 du preflight passe de "couvert sous reserve" a CLEAN.
- **[completude P1 #1 — trou de mapping C1->C3/C11 + civiques §1-3 non mappes]** TRAITE : ajout de lignes roadmap dediees **C1 (`feed_append_app_op`)** et **C3 (agregation cross-node)** en amont de E. La cellule "Dependances" de E nomme desormais C1 EN PLUS de la propagation (DEUX gates). C2/C4/C5/C6 explicitement exclus avec renvoi "serie civique §1-3, hors ce plan". Verifie : la whitelist bridge `protocol.ts` (15 methodes) ne contient ni `feed_append` ni signature -> C1 est bien un manque reel.

### Findings P2 traites

- **[scope_realisme P2 / wire P2 — cadrage `tasks_count` self vs reseau]** TRAITE par le filtre worker (meme correction que P1 compute). De plus, `tasks_count` renomme **`credits_count`** avec label "credits enregistres" (1 entree = 1 credit ; une tache peut produire 0 entree si rejetee/guardrail) — adresse aussi le P3 etiquetage des deux axes.
- **[wire P2 — numeros de ligne perimes pour l'enregistrement de route]** TRAITE : la consigne T2 est re-ancree sur le **NOM du bloc** (`let authed_routes = Router::new()`, debut `http.rs:275`, `.merge` ~492), au cote de `/api/daemon/nodes` / `/api/daemon/seed-count`, plus sur un numero de ligne fragile. Le preflight S4 re-verifie `.merge(authed_routes)`.
- **[wire P2 — `peer_count` melange deux verites]** TRAITE : `peer_count` conserve mais dote de son **propre label** distinct ("pairs ayant annonce seeder cette version, vus recemment — best-effort, peut sur-estimer ; jamais une preuve de disponibilite") coherent avec `count_recent`/`MAX_REGISTRY_BUCKETS` (le registre peut OVER-state).
- **[scope_realisme P2 — perimetre net executable + re-base contrat + delta tests]** TRAITE : contrat re-base (section `hosting` retiree, fusion dans `support_now`), delta tests re-estime Rust **+3 a +4** (au lieu de +5/+6).
- **[process_conformance P2 — verdict EXECUTE trop fort]** TRAITE : apres le scope-cut, le verdict redevient **EXECUTE plein non conditionnel** car le seul point S3 conditionnel (`distinct_peers`) est ferme en amont. La note S3 passe a CLEAN ; il ne reste aucun `[A CONFIRMER PREFLIGHT]` materiel touchant le threat model.
- **[process_conformance P2 — melange roles preflight vs PO]** TRAITE : note de frontiere ajoutee sous le preflight + en tete des Decisions PO. Items threat-model/produit -> PO ; "etiquetage app via `kudos.project_id`" -> preflight reel sans PO (PLAN-ADAPT). Aucun item route a la fois vers les deux.
- **[completude P2 — C9 orphelin]** TRAITE : mention explicite ajoutee ("C9 reste un candidat independant non sequence dans cette serie impact/visibilite — a reprendre au kickoff GPU S76"), levant l'ambiguite oubli-vs-defer.

### Findings P3 statues

- **[wire P3 / scope_realisme P2 — `tasks_count` approximation]** ACCEPTE et corrige : renomme `credits_count`, label "credits enregistres" (litteralement vrai). Pur etiquetage, aucun impact code structurel.
- **[scope_realisme P3 — roadmap B-F deborde du plan de phase MVP]** ACCEPTE : la section B-F est marquee plus nettement "annexe vision NON planifiee, NON candidate a execution immediate", SEUL le MVP (T1-T4) etant candidat. Aucune action de code.
- **[process_conformance P3 — forme commit/G1 selon vehicule]** ACCEPTE et traite : note "vehicule d'insertion" ajoutee sous §Forme du commit, levant la tension "modele UX-ARRIVAL lean" vs "Cas B standard" (le PO choisit l'un, les gates de chaque branche sont nommes ; `G1 skipped` probable car 0 decision Day-0 nouvelle).
- **[process_conformance P3 — politique canonical pre-launch pour C14]** ACCEPTE et traite : note canonical pre-launch ajoutee sous le tableau B-F (redefinition v1 in-place, suppression tests legacy-decode, pas de bump, preflight S4 par phase ; "0-bump" n'est pas un blanc-seing).
- **[completude P3 — incoherence prose/tableau sur C3 + precision `feed_cursor_get`]** ACCEPTE et traite : ligne C3 materialisee dans le tableau (coherence prose/tableau) ; cellule D precise que `feed_cursor_get` existe deja en lecture LOCALE (`protocol.ts:39`) et que la nouveaute est l'exposition cross-node + octets canoniques.

### Findings acceptes-as-is (sans changement de fond)

- Le **decoupage ordonne T->T** reste juge solide par tous les axes ; seule la suppression de T1/T2 (blob-serve) le modifie, renforcant l'atomicite "un commit".
- Les **5 verrous anti-recentralisation** etaient deja correctement tenus (0 wire, 0 cross-node, CTA inerte) ; la seule addition est le renforcement du verrou 4 par le filtre worker.
- Le **scope-cut "fenetre seul-hebergeur"** etait deja correct et bien justifie par le code (SeedRegistry ephemere, `pinned_at` jamais lu, pas de table d'historique) ; conserve tel quel, complete par le routage C12.
