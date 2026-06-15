<!-- SPDX-License-Identifier: AGPL-3.0-or-later -->

# Strategie de test E2E du frontend / applications du protocole

> HEAD `8dfb4f7`. Constat PO valide : le backend Rust est tres
> couvert (~1755 nextest Win / ~379 Vitest UNIT mockes), mais le
> frontend du protocole n'est jamais teste de bout en bout — et la
> majorite des bugs vecus a la main sont des bugs d'integration que
> le unit ne peut pas attraper par construction.

---

## 1. Diagnostic : le trou de couverture, quantifie

### 1.1 La pyramide est inversee

| Couche | Outil | Volume | Reel ou mocke | Verdict |
|---|---|---|---|---|
| Daemon / worker CLI + startup | Rust `nextest` + `e2e.rs` | 1755 Win / e2e.rs 7 tests daemon + 11 worker | **reel** (HTTP bind, /health, singleton, SIGINT) | solide, mais s'arrete au niveau API |
| API daemon (Zod) | Vitest `daemon.test.ts` (977 LOC, 47+ tests) | tous les endpoints Zod-parses | **mocke** (`vi.stubGlobal('fetch')`) | vert mais aveugle au wire reel |
| Pages React | Vitest pages (5/10) | Browse, BrowsedProject, NodeCatalog, Nodes, Deploy | **mocke** (`makeBrowseEntry()` fabrique a la main) | happy-path dominant |
| Bridge postMessage | Vitest `useBridge.test.ts` (175 LOC) | 15 methodes | **mocke** (`fakeWindow = {postMessage: vi.fn()}`) | aucun vrai iframe |
| **E2E navigateur reel** | **Playwright** | **0 dans l'arbre canonique** | — | **ABSENT** |

Le `vitest.config.ts:15` l'admet noir sur blanc :
`"Everything else is covered by Playwright (real coordinator)."`
— sauf que Playwright **n'existe plus dans l'arbre**. Le commentaire
designe une couche fantome.

### 1.2 Le mecanisme du mock drift (pourquoi un test vert coexiste avec une UI cassee)

Les reponses daemon dans les tests sont **fabriquees a la main**,
jamais derivees du serializer Rust :

- `web/src/pages/__tests__/Browse.test.tsx:51-65` — `makeBrowseEntry()`
  hardcode `curator_pubkey='22'*32`, `source='curator'`. Si le daemon
  emet une 4e valeur de `source`, ou ajoute un champ requis, le test
  reste **vert** et le vrai daemon **brick** la page.
- `web/src/api/daemon.ts:290-292` — `callDaemon()` jette
  `ApiProtocolError` sur echec Zod. Les schemas Zod sont `strict` et
  miroir du Rust `DaemonStateSnapshot` (`daemon.ts:31-45`), mais aucun
  test ne confronte un Zod a une **vraie** sortie `serde_json`. Un
  champ requis ajoute cote daemon = toutes les pages basculent
  "daemon offline" alors que le daemon tourne.
- `web/src/api/daemon.ts:152-201` — `BrowseEntrySchema` a 15 champs
  dont 5 `.optional()` (`source`, `archive_ticket`, `is_open_source`,
  `is_own`, `from_subscribed`). Le composant suppose souvent leur
  presence (`is_own ? renderKeepOnline : null`) — un daemon mixed-version
  qui omet le champ ne casse pas Zod mais casse le rendu. Jamais teste.

### 1.3 Les pages et flux SANS aucun test

**5 pages orphelines sur 10** (`web/src/pages/`) :

| Page | Ce qu'elle fait | Etat error/loading teste ? |
|---|---|---|
| `Curators.tsx` | subscribe/unsubscribe `/daemon/curators` (useQuery+useMutation) | non |
| `Network.tsx` | poll worker-state toutes les 2s | non |
| `Projects.tsx` | health poll 5s + coordinators | non |
| `ProjectDetail.tsx` | 5 tabs (getProject/listTasks/listKudos/listInvites/listApps) | non |
| `OnboardingEmpty.tsx` | statique + AddCoordinatorDialog | non (peu critique) |

Les 5 rendent toutes un etat "Aucun noeud actif" (`Network.tsx:50`,
`Curators.tsx:31`, `ProjectDetail.tsx:37`, `Projects.tsx:16`) —
**aucun** test ne valide ce rendu, ni un timeout fetch, ni un 503.

**Flux multi-etapes 0% testes** (zero integration) :
- `publish (/deploy) -> /browse -> l'entree apparait -> "Ouvrir" -> iframe charge`
- `recherche : taper -> resultats -> effacer -> retaper` (race keystroke + staleTime)
- `fork -> redeploy local -> reconciliation de l'ancienne entree cache`
- `subscribe -> ingestion catalogue annuaire -> split "mes sources" / "decouvert"`

**Races temporelles jamais simulees** (mock fetch = instantane) :
- `Browse.tsx:121` — `browsePull()` puis `setTimeout(refetch, 2000)`
  hardcode. Sous latence, le refetch tire des donnees stale.
- 6 caches React-Query avec `staleTime` divergents (30s browse, 10s
  search, 5min verify, 2s poll network) — zero test de drift de
  coherence inter-cache. Ex. `BrowsedProject.tsx` : badge
  "Verification echouee" (verify cache 5min) qui survit a une entree
  redevenue fraiche (browse cache 30s).

**Le symptome live de S75 Phase G confirme le trou** : l'acceptance
LIVE a observe "SeedAnnounced ne converge pas cross-noeud" et
"l'annuaire du seeder n'annonce pas ce qu'il seede" — exactement le
genre de divergence backend->UI que seul un E2E reel cross-noeud
attrape, jamais un Vitest mocke.

### 1.4 Constat process : une anomalie CI dormante

`.github/workflows/ci.yml:84-85` execute encore
`cd web && xvfb-run npx playwright test`. Le `playwright.config.ts`
pointe `testDir: ./tests` + `globalSetup: ./tests/global-setup.ts`,
mais **le config et les 26 specs n'existent que dans
`.claude/worktrees/{dazzling-cannon,agent-a97fde30}/`**, pas dans
`web/`. Le dependency `@playwright/test ^1.59.1` et le script
`test:e2e` sont presents (`web/package.json:11,51`) mais le runner
ne vise rien. Cette etape echoue (ou est masquee) — c'est un
**finding pour l'audit gate S75**.

> Correction a la recon : le `global-setup.ts` canonique
> (`web/tests/global-setup.ts:11-12`, S63 Phase A) spawn le **daemon
> Rust directement**, PAS un coordinateur Python (retire S50-S51). Le
> `playwright.config.ts` du worktree mentionne encore `fixtures.ts` +
> coordinateur Python : c'est un commentaire **perime**. Le setup
> canonique est daemon-only et coherent. Il n'y a donc PAS de conflit
> d'architecture a resoudre — juste un config a re-deriver du
> daemon-only setup deja present.

### 1.5 Recoupement par un audit independant (GPT-5.5) — deux findings supplementaires, confirmes en code

Un second audit multi-agents independant (GPT-5.5, 2026-06-12) converge
sur le meme diagnostic (pyramide inversee, 5/10 pages sans test, harness
Playwright mort, mock drift) et le confirme par mesure directe :
`npx playwright test --list` renvoie **0 tests in 0 files** puis une
erreur de matcher Vitest (la CI [10] "protege" donc le vide). Il remonte
deux findings que la version precedente de ce document sous-ponderait,
**verifies en code ici** :

1. **Divergence d'allowlist bridge TS vs Rust — CONFIRMEE
   (correctness/securite, pas juste un trou de test).**
   `web/src/bridge/protocol.ts:20` expose **15** methodes ;
   `crates/sbfb-manifest/src/lib.rs:52` (`BRIDGE_METHOD_ALLOWLIST`) n'en
   autorise que **10**. Les **5** presentes au runtime TS mais absentes
   de l'allowlist manifeste Rust : `pii_redact`, `storage_version`,
   `provenance_get`, `provenance_verify`, `feed_cursor_get` (ajoutees au
   bridge en S21/S58/S63 sans mise a jour du crate manifeste). Double
   consequence : (a) un validateur de manifeste rejetterait a tort une
   app declarant `provenance_get` — drift TS/Rust classique, exactement
   ce qu'un test de contrat etendu au manifeste attrape ; (b) plus
   serieux, le manifeste d'app **ne resserre pas** les capacites runtime
   — aucune enforcement `manifest.methods -> dispatch` trouvee a la
   recon, donc le moindre privilege par app n'est pas reellement
   implemente (toute app obtient les 15 methodes). La Couche C doit
   *verrouiller* cette frontiere, pas seulement la tester.

2. **Race auth/autoregister au tout premier ecran.** Le boot monte React
   **avant** que `fetchAuthToken()` soit resolu — le premier rendu peut
   partir sur un etat sans token (autoregister same-origin pas encore
   pret), produisant un faux "daemon offline" transitoire ou une requete
   non authentifiee. Finding distinct du simple "pas de Playwright" :
   bug-hypothese concret a couvrir par le smoke `boot/auth` (Couche B,
   parcours 1).

**Lecture** : la convergence de deux audits multi-agents independants
sur le meme verdict est un signal de robustesse ; ces deux findings
*renforcent* la priorisation — la Couche A doit inclure les contrats du
**manifeste** (pas seulement les routes daemon), et la Couche C doit
asserter l'alignement `protocol.ts` == `sbfb-manifest` ET que le
manifeste contraint reellement le runtime.

---

## 2. La strategie en couches

Quatre couches, du moins cher / plus rentable au plus cher. Chaque
couche attrape une classe de bug que les autres ratent.

### Couche A — Tests de CONTRAT (anti mock-drift) — LE PLUS RENTABLE

**Idee** : capturer de **vraies** reponses du daemon hermetique (le
`global-setup.ts` existe deja, il sait booter un daemon) et les
rejouer comme **fixtures golden** contre les schemas Zod et les
mocks Vitest. Un test de contrat echoue des que le wire Rust derive
du schema TS.

- **Ce qu'elle attrape que les autres ratent** : le mock drift. Champ
  requis ajoute/retire cote Rust, enum `source` etendu, `schema_version`
  bumpe, `.optional()` qui devient absent — toute divergence
  Zod-vs-serde devient rouge avant la prod.
- **Comment** : un petit harness Rust (ou un script qui appelle le
  daemon hermetique boote par `global-setup.ts`) qui dump les reponses
  de `/api/daemon/info`, `/browse`, `/curators`, `/search`, `/nodes`
  dans `web/src/api/__tests__/fixtures/golden/*.json`. Un test Vitest
  `daemon.contract.test.ts` parse chaque golden avec le schema Zod
  reel — si Zod jette, drift detecte. Idealement genere en CI a chaque
  run pour rester frais (pas un snapshot fige a la main).
- **Cout** : faible. Reutilise `global-setup.ts` + les schemas Zod
  existants. ~1 fichier harness + 1 suite contrat. C'est le meilleur
  ratio bug-attrape / effort.

### Couche B — E2E navigateur smoke (vrai daemon + vrai shell + Chromium)

**Idee** : restaurer Playwright et couvrir les **parcours coeur** en
navigateur reel, contre le daemon hermetique de `global-setup.ts` +
le shell servi par `npm run dev`.

- **Ce qu'elle attrape que les autres ratent** : persistance
  localStorage entre navigations, races de polling (`/health` 5s,
  worker-state 2s), CSP iframe, error boundaries (`DaemonOfflineBanner`),
  middleware auth loopback (bearer + Host + Origin), rendu TabView
  contre une vraie reponse daemon vs JSON mock.
- **Parcours de la 1ere vague** (priorite §3) :
  1. `shell-onboarding-empty-state` — localStorage vide -> "Bienvenue"
     + CTA "Ajouter coordinateur" (spec existe en worktree).
  2. `browse-daemon-offline` — `/browse` sans daemon -> `DaemonOfflineBanner`
     + 503 `{kind:'unavailable'}` (spec existe en worktree).
  3. `loopback-auth` — `/health` public, sans token -> 401, mauvais
     token -> 401, Origin set -> 403, Host rebound -> 403 (spec existe).
  4. **publish->browse->open** (NOUVEAU) — deployer via `/deploy`,
     attendre l'entree dans `/browse`, cliquer "Ouvrir", asserter que
     l'iframe blob-serve charge. C'est LE flux ou le PO voit des bugs.
  5. **recherche** (NOUVEAU) — taper -> resultats -> effacer -> retaper,
     asserter pas de stale et pas de `query.isError` Zod sur les
     reponses rapides successives.
  6. `command-palette` — Ctrl+K -> naviguer (spec existe).
- **Cout** : moyen. Le harness existe a 80% (`global-setup.ts` +
  `global-teardown.ts` canoniques). Il faut restaurer le config et
  re-deriver les specs.

### Couche C — E2E du BRIDGE (le plus gros angle mort)

**Idee** : charger une **example app** (`examples/sbfb-explorer/` qui
embarque `sbfb-bridge.js`) dans un **vrai iframe** `sandbox="allow-scripts"`
et exercer les **15 methodes whitelist** (`protocol.ts:20-44` :
`task_submit`, `storage_get/set/list/delete`, `pii_redact`,
`identity_pubkey`, `node_status`, `browse_list`, `storage_version`,
`provenance_get/verify`, `feed_cursor_get`, `search`, `proof_card_get`).

- **Ce qu'elle attrape que les autres ratent** : la livraison reelle
  `postMessage` cross-origin (`useBridge.ts:200,209` envoie en
  `targetOrigin='*'`), les origin checks, le respect CSP du sandbox,
  l'ordering des heartbeats, le round-trip complet
  app->host->daemon->app, et que la forme de `BridgeResponse` matche
  exactement ce que le resolver de Promise du SDK attend. Les tests
  Vitest utilisent un `fakeWindow.postMessage = vi.fn()` — ils ne
  testent **rien** du canal reel.
- **Parcours de la 1ere vague** :
  - `bridge-heartbeat` (spec existe) — iframe reel emet >=2 heartbeats
    en 2s.
  - `bridge-push-event` (spec existe) — host -> iframe push, callback
    `onEvent` echoue.
  - **bridge-method-roundtrip** (NOUVEAU) — pour les methodes lecture
    sans worker (`browse_list`, `node_status`, `identity_pubkey`,
    `storage_get/set`), asserter un round-trip complet contre le daemon
    hermetique. `task_submit` reste un smoke "la requete part" (pas de
    worker live dans l'env Playwright).
- **Frontiere a verrouiller (finding GPT-5.5, confirme en code §1.5)** :
  la suite bridge doit asserter (a) que l'ensemble runtime TS
  (`protocol.ts`, 15) est aligne avec l'allowlist manifeste Rust
  (`sbfb-manifest`, 10 aujourd'hui — 5 methodes de drift a reconcilier)
  et (b) que la liste `methods` d'un `SBFB.json` **restreint reellement**
  les methodes dispatchables pour cette app (moindre privilege par app).
  Aujourd'hui les deux divergent et le manifeste ne semble pas applique
  au runtime — la Couche C transforme ce constat en garde-fou execute.
- **Cout** : moyen-eleve. Necessite un test page qui monte l'iframe +
  injecte le SDK reel depuis `examples/`. C'est l'angle mort de plus
  haute valeur securite (l'iframe est la frontiere de confiance).

### Couche D — Matrice multi-OS + cross-machine (POST-AUDIT, hors 1ere vague)

**Idee** : etendre la matrice CI (`win/mac/linux`) en reutilisant le
pattern du workflow `build-worker.yml`, puis l'E2E cross-machine
distribue (2 noeuds reels).

- **Ce qu'elle attrape que les autres ratent** : le loopback Windows
  natif (Named Pipe DACL, `PeerCredsVerified`) jamais teste hors Unix ;
  la convergence reelle SeedAnnounced cross-noeud (le symptome S75-G) ;
  le split "mes sources"/"decouvert" en conditions reseau reelles.
- **Cout** : eleve (macOS CI = minutes payantes ; cross-machine =
  partiellement manuel pre-audit, deja la realite des acceptances
  LIVE PC<->Mac<->VPS). **Explicitement hors 1ere vague.**

---

## 3. Priorisation par densite de bug

Ordre concret, du plus douloureux (la ou le PO voit deja des bugs)
au moins urgent :

1. **Couche A (contrat) d'abord.** Le mock drift est la cause racine
   silencieuse : un changement de wire Rust passe tous les tests verts
   et casse l'UI en prod. C'est le moins cher (reutilise schemas +
   `global-setup.ts`) et il blinde **toutes** les autres couches
   (les goldens deviennent la verite des mocks). Premiere ligne de
   defense.

2. **Couche B, parcours `publish->browse->open`.** C'est
   litteralement le flux ou le PO "rencontre beaucoup de bugs en
   testant a la main". Race `browsePull -> setTimeout 2s -> refetch`
   (`Browse.tsx:121`), drift de cache, "Projet introuvable" si on
   clique trop vite apres deploy. Plus haute densite de bug vecu.

3. **Couche B, error boundaries + auth** (`browse-daemon-offline`,
   `loopback-auth`). Specs deja ecrites en worktree, restauration quasi
   gratuite, couvrent les etats "daemon offline" jamais testes sur les
   5 pages orphelines.

4. **Couche C, bridge round-trip.** Le plus gros angle mort
   structurel (frontiere de confiance iframe), mais legerement moins
   "vecu au quotidien" que publish/browse. Vient apres B.

5. **Couche B, pages orphelines** (`Curators`, `Network`, `Projects`,
   `ProjectDetail`) — au moins un smoke "rend l'etat aucun-noeud + rend
   l'etat data" par page. Comble le 5/10 pages sans aucun test.

6. **Couche D** (multi-OS + cross-machine) — post-audit, deuxieme
   vague.

---

## 4. Plan de restauration du harness

L'infra est a 80% presente. Il manque le config + les specs + la
reconciliation CI.

### 4.1 Remettre le config et le runner

- Copier `.claude/worktrees/dazzling-cannon/web/playwright.config.ts`
  vers `web/playwright.config.ts`. **Nettoyer** le commentaire perime
  (`fixtures.ts` / coordinateur Python) : le setup est daemon-only.
  Garder `testDir: ./tests`, `globalSetup/Teardown` (deja canoniques),
  `workers: 1`, `webServer: npm run dev`, `extraHTTPHeaders` bearer
  `TEST_AUTH_TOKEN` (deja aligne avec `global-setup.ts:30`).
- `web/tests/global-setup.ts` et `global-teardown.ts` sont DEJA
  canoniques et corrects (daemon Rust hermetique port 18765, token
  hex fixe, attente `/health` 200). Ne pas les retoucher.
- Supprimer les artefacts `.tmp/` traines dans `web/tests/.tmp/` et
  les ignorer (`.gitignore`).

### 4.2 Reconcilier l'anomalie CI [10]

`ci.yml:84-85` execute deja `xvfb-run npx playwright test`. Aujourd'hui
ca echoue (testDir vide) ou est masque. **Deux temps :**
- Court terme (finding audit S75) : documenter que l'etape pointe une
  suite absente — soit la rendre **non-bloquante** explicitement, soit
  la restaurer.
- A la restauration : l'etape passe au vert avec la 1ere suite smoke.
  Ajouter l'upload de `trace`/`screenshot` on-failure (le config a deja
  `retain-on-failure`). **Garder Linux/Chromium uniquement** pour la
  1ere vague (multi-OS = Couche D).

### 4.3 La 1ere suite smoke minimale executable

Restaurer **6 specs deja ecrites** (triage SAFE depuis worktree) :
`shell-onboarding-empty-state`, `browse-daemon-offline`,
`loopback-auth`, `command-palette`, `bridge-heartbeat`,
`bridge-push-event`. **Ne PAS** restaurer en bloc les 10 `gov-*.spec.ts`
(app gov = couche applicative, pas le coeur protocole ; defere).
Ecrire **2 specs neuves** haute-valeur : `publish-browse-open.spec.ts`
et `search-flow.spec.ts`.

Premier commande locale de validation :
`cd web && npx playwright test --project=chromium`.

### 4.4 Premiers fichiers a creer (concret)

| Fichier | Couche | Contenu |
|---|---|---|
| `web/playwright.config.ts` | infra | depuis worktree, commentaire nettoye |
| `web/src/api/__tests__/daemon.contract.test.ts` | A | parse les goldens reels avec les schemas Zod |
| `web/src/api/__tests__/fixtures/golden/*.json` | A | dumps reels du daemon hermetique (genere CI) |
| `web/tests/publish-browse-open.spec.ts` | B | deploy -> apparait dans browse -> open iframe |
| `web/tests/search-flow.spec.ts` | B | taper/effacer/retaper, anti-stale |
| `web/tests/bridge-method-roundtrip.spec.ts` | C | 15 methodes whitelist via iframe reel + `sbfb-bridge.js` |
| (restaure) `web/tests/{onboarding,browse-offline,loopback-auth,command-palette,bridge-heartbeat,bridge-push-event}.spec.ts` | B/C | depuis worktree |

---

## 5. Decoupage en phases / lots

Chaque lot = un increment testable et commitable atomiquement.

- **Lot 0 — Reconciliation CI (finding audit).** Constater + documenter
  l'anomalie `ci.yml:84-85`. ~0,5 j. (Va dans l'audit gate S75, pas un
  commit feature.)
- **Lot 1 — Couche A contrat.** Harness dump goldens + `daemon.contract.test.ts`
  + fixtures. ~1-1,5 j. **Le plus rentable, a faire en premier.**
- **Lot 2 — Restaurer harness Playwright + 4 specs smoke shell/auth.**
  `playwright.config.ts` nettoye + onboarding/browse-offline/loopback-auth/command-palette
  + CI [10] vert. ~1-1,5 j.
- **Lot 3 — Parcours coeur publish->browse->open + search.** 2 specs
  neuves, le flux le plus bugue. ~1,5-2 j.
- **Lot 4 — Couche C bridge.** Restaurer heartbeat/push-event + ecrire
  `bridge-method-roundtrip` (15 methodes, iframe reel). ~2 j.
- **Lot 5 — Smoke pages orphelines.** Curators/Network/Projects/ProjectDetail :
  etat aucun-noeud + etat data. ~1,5 j.
- **Lot 6 (post-audit) — Couche D.** Matrice win/mac/linux + cross-machine.
  Plusieurs jours, paye en minutes CI.

Lots 0-5 = environ **8-9 jours** = un petit sprint qualite ou un
mini-cycle etale. Lot 6 = vague suivante.

---

## 6. Vehicule process et sequencement

**(a) D'abord : l'anomalie CI = finding de l'audit gate S75.**
Le prochain pas canonique est S76 Phase 0 = audit gate S75
(`sprint76_audit_plan.md`). L'etape CI [10] qui pointe une suite
absente est exactement le type de constat qu'un audit gate doit
router. Lot 0 entre la, sans ceremonie supplementaire.

**(b) Ensuite : UN SEUL sprint qualite couvrant la TOTALITE de
l'objectif (directive PO 2026-06-12).** Pas de cap artificiel a 6-7
phases : le "4-7 phases A-G" est une CONVENTION indicative, pas un
ordre ; le nombre de phases sert l'objectif. La regle d'ingenierie
"mono-OS stable d'abord, puis matrice" est un ORDRE DE PHASES, PAS une
raison de scinder en deux sprints. Le sprint couvre donc toutes les
couches A->D, ordonnees pour preserver cette rigueur. Decoupage
indicatif (~10 phases) :

| Phase | Contenu | Couche |
|---|---|---|
| 0 | audit gate (route findings CI morte + drift allowlist 15 vs 10) | — |
| A | restaurer harness Playwright (config + reconcilier CI) + specs smoke, Linux/Chromium vert et STABLE | B (infra) |
| B | couche contrat : goldens regeneres en CI + Zod + contrat **manifeste** | A |
| C | parcours coeur : publish->browse->open, recherche | B |
| D | bridge E2E + alignement allowlist TS/Rust (quick-fix 10->15) + verrou de frontiere | C |
| E | enforcement bridge par app (fix design ; preflight G8 intra-phase ; peut casser des apps au manifeste incomplet -> arbitrage PO dans la phase) | C |
| F | smoke pages orphelines (Curators/Network/Projects/ProjectDetail) | B |
| G | matrice CI multi-OS win/mac/linux — UNE FOIS la suite mono-OS stable (ordering, pas frontiere de sprint) | D |
| H | cross-machine : orchestration API-level scriptee (SSH) + acceptance LIVE manuelle PC<->Mac<->VPS (modele S75 Phase G) | D |
| I | wrap-up : verification + audit_plan + docs | — |

**(c) Les deux seules contraintes reelles — ni l'une ni l'autre n'est
un cap de process :**
- **Ordre, pas scission** : la matrice multi-OS (Phase G) vient APRES
  A-F (mono-OS vert et stable). Mettre 3 OS des la Phase A = debugger
  l'OS ET l'immaturite de la suite en meme temps. C'est dans le MEME
  sprint, juste plus tard dans la sequence.
- **Un seul gate EXTERNE** : R-iroh-audit P0 borne la PORTEE du test
  cross-machine LIVE reel (Phase H) au pilote ferme — exactement comme
  a l'acceptance S75 ou le live PC<->Mac<->VPS etait un geste manuel de
  cloture, pas une absence. La matrice CI multi-OS (Phase G) n'est PAS
  gatee (runners GHA, pas le reseau prod). Le cross-machine entre donc
  dans le sprint, sous la forme que le pilote ferme autorise.

**Sequencement clair :**
`audit gate S75 (route les findings) -> UN sprint qualite full-objectif
(Phases 0->I, ~10 phases, mono-OS stable avant matrice) -> aucune "vague
differee" sauf ce que R-iroh-audit borne (test reseau LIVE complet
multi-machine).`

---

## 7. Ce que ca ne resout pas (honnete)

- **E2E ne remplace pas le unit.** Les ~379 Vitest restent la verite
  des branches/format/store purs (`format.test.ts`, `projectStore.test.ts`).
  L'E2E couvre l'integration, pas la combinatoire fine. La pyramide
  doit se re-equilibrer, pas s'inverser dans l'autre sens.
- **Le flake est inevitable.** Les races qu'on veut attraper (poll 2s,
  staleTime 30s) rendent l'E2E sensible au timing. Mitigation : `retries: 1`
  en CI (deja dans le config), `workers: 1` (deja), attentes sur etat
  (web-first assertions) plutot que `sleep`. Un budget flake doit etre
  assume, pas nie.
- **Cout macOS CI.** La Couche D coute des minutes payantes ; a ne
  lancer que post-audit et peut-etre seulement sur tags/releases, pas
  chaque PR.
- **Le cross-machine reel reste partiellement manuel pre-audit.** Les
  symptomes S75-G (SeedAnnounced non-converge, seeder `catalog_len:0`)
  s'observent surtout en acceptance LIVE PC<->Mac<->VPS, qui restera
  un geste semi-manuel tant que R-iroh-audit P0 maintient le pilote
  ferme. L'E2E hermetique mono-machine ne reproduit pas la topologie
  reseau reelle.
- **Pas de couverture coordinateur Python a restaurer.** Contrairement
  a ce que suggerent les vieux commentaires worktree, il n'y a plus de
  coordinateur Python (retire S50-S51) ; le setup daemon-only est
  complet. Aucune dette Python a recreer — un faux probleme a ne pas
  poursuivre.
- **Les goldens de contrat peuvent eux-memes deriver** s'ils sont
  figes a la main. D'ou la recommandation de les **regenerer en CI**
  depuis le daemon hermetique a chaque run, pas de les commiter comme
  snapshot mort (sinon on recree un mock drift de second ordre).
