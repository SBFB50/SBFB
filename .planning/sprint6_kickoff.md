# Sprint 6 — Kickoff (scope acknowledgement + split proposal)

**Écrit** : 2026-04-11
**HEAD entrée** : `cdf4467` (tip Sprint 5, working tree clean)
**Auteur** : réconciliation post-Sprint 5 après lecture de
`.planning/sprint5_verification.md` §"What's NOT in this sprint",
`.planning/sprint5_plan.md` §10, `docs/shell/PATTERNS.md`
§T1..T3, `.planning/NEXUS_GOV_ROADMAP.md` et reconnaissance
parallèle (4 agents Explore + context7 research).

---

## 1. Constat d'entrée

Sprint 5 a fermé 9 commits (`82691ce` → `cdf4467`), checklist
22/22 verte, 193 Rust + 43 Python coord + 8 Playwright verts,
tsc strict + ESLint + build prod (425 KB / 190 KB / 90 KB)
zéro warning. Shell React livré : `OnboardingEmpty`, `Projects`,
`ProjectDetail` (5 tabs shadcn), `Network` (worker state live
2 s polling), stubs `Browse` + `Curators`.

Le scope bloc `sprint5_verification.md §"What's NOT in this sprint"`
liste **8 chantiers** déférés à Sprint 6 :

1. `nexus-shell-daemon` — sidecar Rust avec Node iroh pour DHT /
   curator gossip
2. Schema-driven tab rendering — vocabulaire à définir
3. Curator list flow — Ed25519 signing + gossip propagation +
   subscribe/unsubscribe
4. DHT browse (pkarr)
5. Migration `nexus-app-gov` 19 tabs (v1.1)
6. Command palette Ctrl+K + keyboard shortcuts
7. Vitest unit tests (T3)
8. Bundle size CI check (T2)

## 2. Scope réaliste — Sprint 6 ≠ 1 sprint

La reconnaissance (agents Explore) a confirmé :

- **`nexus-app-gov` cible v1.1** pèse ≈ 15–20 kLOC (16 tabs
  explicites + 2-3 implicites dans `NEXUS_GOV_ROADMAP.md` §5.1,
  16 tables SQLite gov_*, 31 workers estimés, dépendances
  externes Wikidata / An.gouv.fr / HATVP / yt-dlp / faster-whisper
  / GLiNER). Ce seul chantier excède un sprint de 8-10 jours
  et dépend du schema-driven rendering qui lui-même dépend de
  la SDK extension.
- **`nexus-shell-daemon`** est un nouveau crate Rust
  (pas de précédent dans le repo — `nexus-worker` / `nexus-worker-core`
  est le seul split headless existant). Il doit héberger un
  iroh `Endpoint` long-lived, exposer une API HTTP locale au
  shell React, gérer une clé persistée
  `~/.nexus-grid/shell-daemon/shell.key`, et supporter un
  `running.json` analogue à celui du coordinator. C'est un
  workstream à part entière qui bloque curator gossip + DHT
  browse.
- **Schema-driven tab rendering**, **Vitest** et **bundle CI**
  sont des chantiers frontend petits à moyens qui se font en
  une semaine à deux personnes (ou ~8 jours solo).

**Conclusion** : le bloc "Sprint 6" d'origine représente
~3 sprints de travail. Les regrouper en un seul sprint
violerait le pattern établi Sprint 0..5 (un sprint = un
artefact cohérent, une checklist fail-fast exhaustive, un tip
git propre). On propose donc un split explicite en trois
sprints numérotés 6, 7, 8.

## 3. Split proposé

### Sprint 6 — Shell Foundations + Schema Tabs *(8-10j)*

**Objectif** : préparer le terrain pour la migration gov.
Livrer le vocabulaire `TabView` frozen, le renderer React
correspondant, le Ctrl+K palette, les Vitest unit tests et le
garde-fou CI de bundle size. Hello-world-app et gov devient la
première app 100% schema-driven (1 tab chacune).

**Pourquoi en premier** : aucun blocker externe ; débloque le
chemin critique vers Sprint 8 (gov v1.1) ; purement applicatif
(ni nouveau crate Rust, ni modification iroh). Permet de
valider le vocabulaire avant de le figer pour 19 tabs.

**Phases** : A (SDK view module + coordinator) · B (web
renderer + port hello/gov) · C (Ctrl+K palette) · D (Vitest +
bundle CI) · E (Playwright + verif doc).

**Risk gate** : si le vocabulaire s'avère insuffisant pour
couvrir un tab gov représentatif (e.g., Reseau graph WebGL),
documenter le scope cut et noter les blocks à ajouter v1.2 ;
ne pas élargir.

### Sprint 7 — P2P Discovery Layer *(10-12j)*

**Objectif** : livrer `nexus-shell-daemon`, les listes
curator signées Ed25519, le flow DHT browse via pkarr, et
câbler les pages `Browse.tsx` + `Curators.tsx` sur ce daemon.

**Pourquoi en second** : dépend du shell (Sprint 5) et du
vocabulaire TabView (Sprint 6) pour afficher les résultats
DHT. N'impacte pas la logique app côté Python. Risk iroh
0.97 déjà validé Sprint 2 (`two_nodes_fetch_blob_via_ticket`
+ `two_nodes_docs_sync` prototypes verts).

**Phases pressenties** (outline Sprint 7, à figer dans un
kickoff dédié) :

- **A** : `crates/nexus-shell-daemon-core` + `crates/nexus-shell-daemon`
  (suivre le pattern `nexus-worker-core` / `nexus-worker`).
  Headless-first, tests sans terminal. Démarrage = load clé
  + bind iroh Endpoint (`discovery_n0()`) + écrire running.json
  + écouter HTTP loopback sur port éphémère.
- **B** : PyO3 extension `nexus_core.sign_curator_list` /
  `verify_curator_list_entry` (suivre exactement le pattern
  Sprint 4 `sign_claim`). Ajout `DOMAIN_CURATOR_LIST_V1` dans
  `crates/nexus-core-rs/src/canonical.rs`. Nouvelle struct
  `CuratorListEntry { list, curator_pubkey, signature }`.
- **C** : Rust `GossipClient` extension + `BlobsClient::fetch_ticket`
  exposés à Python (ils manquent dans `nexus-core-py` — confirmé
  par reconnaissance crates/nexus-core-py/src/lib.rs). Shell
  daemon subscribe `topic = blake3("nexus-curator-v1")`, reçoit
  des messages `{list_hash, curator_pubkey}`, récupère le blob
  via ticket, vérifie la signature.
- **D** : pkarr announce/resolve. iroh 0.97 intègre pkarr via
  `Endpoint::builder().discovery_n0()` — rien à installer, juste
  wrapper un `DhtDiscovery::resolve()` dans nexus-shell-daemon
  pour browse public projects. Alimenter `Browse.tsx`.
- **E** : Pages `Browse.tsx` + `Curators.tsx` câblées sur
  `GET /curator-lists`, `POST /curator-lists/subscribe`,
  `GET /dht/browse`. Playwright specs contre shell-daemon réel.

**Dépendances sur Sprint 6** : TabView vocabulary (pour
afficher project cards dans Browse), Ctrl+K palette (pour
ajouter "Subscribe to curator list" comme action palette).

**Décisions à trancher au kickoff Sprint 7** :
- IPC shell ↔ daemon : HTTP loopback sur port éphémère
  (consistant avec coordinator) ou Unix socket / named pipe ?
- Daemon singleton ou multi-instance ?
- Curator-list schema : reuse `CuratorList` déjà esquissé
  dans Sprint 2 doctest, ou redesign ?
- Topic gossip unique `nexus-curator-v1` ou namespacé par
  curator pubkey ?

### Sprint 8 — nexus-app-gov v1.1 (19 tabs) *(14-18j, worst-case)*

**Objectif** : porter les 19 tabs listés dans
`.planning/NEXUS_GOV_ROADMAP.md` §5.1 sur le vocabulaire
TabView + SDK étendue. Livrer une app gouvernance de
démonstration que FlowUP peut lancer publiquement pour le
release v1.0.

**Pourquoi en dernier** : dépend du schema-driven rendering
(Sprint 6), bénéficie des primitives P2P si disponibles
(Sprint 7) pour des tabs "propagation de fact-checks".
C'est le plus gros chantier du cycle : LOC × features ×
datasources externes.

**Phases pressenties** :

- **A** : SDK extension — `AppContext.storage` (namespaced
  SQLite per app), `AppContext.events` (pub/sub in-proc),
  file upload helper, DB migration runner. Ces primitives
  sont listées "missing" par la reconnaissance
  `packages/nexus-sdk/src/nexus_sdk/app.py`.
- **B** : Datasources sync workers — An.gouv.fr scraper,
  HATVP fetcher, Wikidata SPARQL, presse RSS. Un worker par
  source, idempotent, écrit dans SQLite. Feature flags par
  datasource pour dégradation gracieuse.
- **C** : Tabs batch 1 (5 tabs faciles, pure DB) —
  Dashboard, Politicien, Votes, Declarations, Legislation.
  Validation du vocabulaire TabView sur cas réels.
- **D** : Tabs batch 2 (5 tabs LLM) — Contradictions (déjà
  présent, à porter sur schéma + workers réels), Presse
  (sentiment), Recap (résumé hebdo), Comparateur (synthèse),
  Recherche (RAG).
- **E** : Tabs batch 3 (4 tabs advanced) — Reseau (scope cut
  probable: pas de graph WebGL dans TabView vocabulary v1,
  rendu tabulaire en fallback), Timeline, Carte (scope cut
  probable: carte Leaflet = nouveau block type), Videos
  (transcription = pipeline externe).
- **F** : Tabs batch 4 (reste + Alertes) — Affaires, Social,
  Fact-checks, Alertes (WebSocket route — scope cut possible
  vers polling).

**Scope cuts à assumer dès le kickoff** : Reseau graph
(reagraph retiré Sprint 5 Day 0 — ne pas réintroduire sans
décision explicite), Carte Leaflet (idem — leaflet retiré),
charts complexes (restreindre à `chart-line` / `chart-bar`
du vocabulaire Sprint 6, pas de D3 brut). Si un tab a besoin
de plus, c'est v1.2.

**Décisions à trancher au kickoff Sprint 8** :
- Quels datasources on ship pour v1.0 (all 6 ou top 3) ?
- SQLite unique `~/.nexus-grid/projects/<name>/apps/gov/gov.db`
  ou per-datasource ?
- Comment stocker l'index vectoriel (ChromaDB externe, ou
  pur SQLite + sqlite-vec) ?
- Limite de rate des scrapers (pour ne pas se faire bannir) ?

## 4. Sprint 6 proper — décisions Day 0 à geler

Ces 5 décisions doivent être validées par l'utilisateur avant
que `.planning/sprint6_plan.md` soit exécutable. Elles sont
toutes argumentées dans le plan détaillé ; cette section est
un résumé une-ligne-par-décision pour validation rapide.

**D1 — Vocabulaire TabView** : custom minimal
(section, heading, text, kv, metric, table, badge-list,
button, chart-line, chart-bar, empty) — **retenu** plutôt que
`@rjsf/shadcn`. Raisons : view-centric pas form-centric, zéro
nouvelle dep, Tailwind 4 OK, Python producer trivial, bundle
impact ~10 KB, contrôle total des deux côtés. `@rjsf/shadcn`
reste reconsidérable si un tab gov a besoin de forms
élaborées en Sprint 8.

**D2 — Versioning schéma** : `schema_version: 1` littéral dans
chaque `TabView` retourné par un tab. Toute modification est
breaking et doit coincider avec un bump côté SDK Python et
côté renderer React. Même règle que `running.json` et
`WorkerStateSnapshot` Sprint 5.

**D3 — Backwards compat tabs existants** : les 5 tabs shadcn
câblés en dur dans `ProjectDetail.tsx` (Overview / Tasks /
Kudos / Invites / Apps) **restent en dur** — ils ne passent
pas par TabView parce qu'ils consomment des APIs coordinator
natives (`/tasks`, `/kudos`, `/invites`, `/app`), pas des
descriptors d'apps. Le renderer TabView s'applique
uniquement aux tabs **dans `AppsTab.tsx`** (remplace le
`<pre>JSON.stringify(descriptor)</pre>` par le vrai renderer).

**D4 — Portée Vitest Sprint 6** : juste `format.ts` +
`projectStore.ts` + le nouveau `tabview.tsx` renderer. Pas
de tests composants pour les 5 tabs hard-codés (c'est couvert
par Playwright). Objectif : 95%+ line coverage sur ces 3
fichiers, run `<2 s` en CI.

**D5 — Budget bundle + Ctrl+K trigger** : (a) budget CI = main
≤ 475 KB, vendor-react ≤ 210 KB, CSS ≤ 100 KB (headroom +50 KB
sur l'état Sprint 5 pour couvrir TabView + cmdk runtime +
Vitest). Dépassement = échec CI. (b) palette triggers
= `Ctrl+K` (Windows/Linux) et `Cmd+K` (macOS), plus `Escape`
pour fermer. Placement : dans `AppShell.tsx` au niveau racine
(hors `<Outlet>`). Groupes initiaux : Navigation, Projets,
Actions (plus étendus par Sprint 7 + 8).

## 5. Sources consultées

**Planning docs internes** :
- `.planning/sprint5_verification.md` — scope cuts §
- `.planning/sprint5_plan.md` §10
- `.planning/NEXUS_GOV_ROADMAP.md` §5.1 (liste 19 tabs)
- `docs/shell/PATTERNS.md` §P1..P7 + §T1..T3
- `docs/rust/PATTERNS.md` (pattern base Sprint 2 canonical + domain)

**Reconnaissance code** (agents Explore, 2026-04-11) :
- `packages/nexus-app-gov/src/nexus_app_gov/` — 127 lignes, 1 tab
- `packages/nexus-sdk/src/nexus_sdk/app.py` — TabDescriptor
  `{name, icon, fn}`, async descriptor déjà supporté
  `packages/nexus-coordinator/.../api/apps.py:104-105`
- `crates/nexus-core-rs/src/{canonical,task,crypto}.rs` —
  ClaimEntry pattern, DOMAIN_* constants
- `crates/nexus-core-py/src/lib.rs` — PyO3 #[pymodule] :
  gossip/blobs/docs wrapped, `fetch_ticket` NON exposé
- `web/src/pages/ProjectDetail.tsx:178-224` — 5 tabs hard-coded
- `web/src/components/project/AppsTab.tsx:260-335` — raw JSON
  `stringify` du descriptor
- `web/src/lib/format.ts:1-75` + `web/src/stores/projectStore.ts:1-152`
- `web/src/components/ui/command.tsx:1-195` — cmdk wrapper
  shadcn déjà présent, non utilisé
- `web/package.json` — cmdk 1.1.1 présent, vitest absent,
  tailwindcss 4.2.2 via `@tailwindcss/vite`
- `web/scripts/scan-en-strings.sh` — exclude `ui/` déjà,
  extensible

**Lib docs externes** (context7 + docs.rs) :
- `/vitest-dev/vitest` v3/v4 — `defineConfig` depuis
  `vitest/config`, jsdom pour Zustand persist, @testing-library
  séparé
- `/dip/cmdk` — shadcn `<CommandDialog>` + `useEffect` listener
  Ctrl+K documenté
- `/rjsf-team/react-jsonschema-form` + `@rjsf/shadcn` — évalué
  puis **écarté** en faveur du vocabulaire custom (cf D1)
- `/websites/rs_iroh` — pkarr intégré dans iroh 0.97 via
  `Endpoint::builder().discovery_n0()` (pour Sprint 7)

## 6. Checkpoint de validation

Avant d'écrire le code Sprint 6 (commit de `d77f122`-style
`docs(sprint6): kickoff + detailed plan`), l'utilisateur doit :

1. Valider le split 6 / 7 / 8 (ou proposer un autre découpage)
2. Valider les 5 décisions Day 0 D1..D5 (ou les challenger)
3. Valider que le scope bloc "What's NOT in this sprint" de
   Sprint 5 est bien couvert cumulativement par Sprint 6 + 7 + 8
   (cf tableau §7 ci-dessous)

## 7. Traçabilité scope

| Item Sprint 5 "What's NOT" | Sprint | Phase |
|---|---|---|
| nexus-shell-daemon | **7** | A |
| schema-driven tab rendering | **6** | A + B |
| curator list flow Ed25519 | **7** | B + C |
| DHT browse (pkarr) | **7** | D + E |
| 19-tab nexus-app-gov migration v1.1 | **8** | A..F |
| command palette Ctrl+K | **6** | C |
| Vitest unit tests (T3) | **6** | D |
| bundle size CI (T2) | **6** | D |
| worker HTTP API (axum) | **rejeté** | — (confirmé D3 S5) |
| mobile responsive < 1280px | **rejeté** | — desktop-only |

Les deux items "rejetés" restent rejetés — pas de revisite
sans déclencheur externe.

---

**État** : kickoff rédigé, décisions Day 0 gelées en attente
de validation. Plan détaillé Sprint 6 proper dans
`.planning/sprint6_plan.md`. Sprint 7 + 8 gardent le statut
"outline" et auront leurs propres kickoff/plan dédiés.
