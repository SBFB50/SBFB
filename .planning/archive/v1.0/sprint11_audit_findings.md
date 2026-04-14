# Sprint 11 — Audit findings (Sprint 12 Phase 0 gate)

**Auditeur** : session fraiche Claude Code, sans historique Sprint 11
**Date** : 2026-04-12
**Tip audite** : `0a295b9` (master, dernier commit = docs Sprint 11)
**Audit plan joue** : `.planning/sprint11_audit_plan.md` (9 tracks A-I)
**Methode** : 7 agents paralleles + verification manuelle des findings critiques

---

## Verdict global : CONDITIONAL PASS

0 P0, 3 P1, 14 P2, ~10 P3.

Les 3 P1 doivent etre fixes en commits `fix(sprint11): ...` sur
master AVANT le premier commit Sprint 12 Phase A.

---

## Track A — Self-publish gossip correctness

**Verdict** : CONCERN (0 P0, 0 P1, 2 P2, 2 P3)

`publish.rs` est bien structure : `ProjectAnnouncement` a un serde
roundtrip teste, `from_gossip_bytes()` retourne `Result` (jamais de
panic), les versions != 1 et types != "project" sont rejetes avec
erreurs typees. `browse.rs` gere correctement `BrowseSource::Direct`
avec dedup par `project_id` via `DashMap::insert`. `iroh_runtime.rs`
branche correctement entre messages curator et project via
`is_project_announcement()`. Aucun `unwrap()` sur du parse
utilisateur en code de production.

- **[P2] A-01** : pas de test de rejet d'un message tronque (JSON
  partiel). `is_project_announcement(b"{\"type\": \"project\"")` (sans
  accolade fermante) n'est jamais exerce. Le runtime gere
  correctement (serde retourne `Err`), mais le test matrix ne couvre
  pas ce cas.

- **[P2] A-02** : `node_id` dans `ProjectAnnouncement` est un
  `String` non valide — `from_gossip_bytes()` valide `v` et
  `msg_type` mais n'applique aucune verification de longueur ou
  format hex sur `node_id`. Un pair peut injecter une annonce avec
  `node_id: ""` qui finit dans le `BrowseAggregator` comme entree
  avec `project_id: ""`, surfacee dans l'UI comme `Unreachable`.
  Pas de panic, mais pollution silencieuse.

- **[P3] A-03** : double parse JSON par message project
  (`is_project_announcement` + `from_gossip_bytes`). Pas d'impact
  correctness, a optimiser si le volume gossip croit.

- **[P3] A-04** : `curator_pubkey = String::new()` pour les entrees
  `Direct` — intentionnel mais non documente dans un commentaire.

---

## Track B — Coordinator auto-publish integration

**Verdict** : PASS (0 P0, 0 P1, 2 P2, 2 P3)

`_auto_publish()` dans `coordinator.py` est non-bloquant : attrape
`httpx.HTTPError` et continue avec un warning log si le daemon est
down. `POST /project/publish` ne prend pas de body externe — il
construit le payload entierement depuis l'etat du coordinator. Le
daemon proxy valide que le body est JSON avant de forwarder. 4 tests
couvrent les cas publish OK, daemon down/503, auto-publish public,
et endpoint project/publish.

- **[P2] B-01** : pas de test quand le daemon retourne HTTP 500. Le
  proxy wrape en `{"kind": "data", "status": 500}`, le comportement
  est defini mais non teste.

- **[P2] B-02** : pas de test assertant qu'un coordinator avec
  `visibility != "public"` ne publie PAS au boot. La logique est un
  simple `if visibility == "public"` guard (coordinator.py:387),
  correcte mais non testee.

- **[P3] B-03** : `_auto_publish` instancie son propre
  `httpx.AsyncClient(timeout=5.0)` au lieu du singleton partage.
  Correct (le serveur FastAPI n'est pas encore demarre), mais pattern
  inhabituel.

- **[P3] B-04** : `POST /project/publish` ne verifie pas
  explicitement que le coordinator est completement boote avant
  d'acceder a `coord.apps`. En pratique, `start()` doit etre
  complete avant que FastAPI serve des requetes.

---

## Track C — Default curators + auto-subscription

**Verdict** : PASS (0 P0, 0 P1, 2 P2, 2 P3)

`CuratorConfig` a `#[serde(default)]` : une config TOML sans
`[curator]` parse sans crash (teste). L'auto-subscribe au boot est
idempotent : verifie `subscribed_pubkeys_hex().contains()` avant
d'appeler `subscribe()` (teste). `GET /default-curators` retourne la
liste configuree (teste). 120 tests passent.

- **[P2] C-01** : `default_curators` est un `Vec<String>` sans
  validation hex au chargement de la config. Une valeur malformee
  est silencieusement ignoree au boot (warn log + skip) quand
  `parse_pubkey_hex` echoue. Pas de test du warn path.

- **[P2] C-02** : `GET /default-curators` retourne les strings brutes
  de la config sans re-validation. Un client qui passerait une
  valeur malformee a `POST /curators/subscribe` recevrait un 400.

- **[P3] C-03** : le boot log est silencieux quand tous les default
  curators sont deja abonnes (seulement `debug!`, pas `info!`).

- **[P3] C-04** : le commentaire dans `config.toml.example` dit
  "Populated after first curator list creation" mais la section
  est evaluee a CHAQUE boot (idempotent skip). Nit de doc.

---

## Track D — Browse full-screen UX

**Verdict** : CONCERN (0 P0, 1 P1, 0 P2, 1 P3)

`BrowsedProject.tsx` est bien structure : back link, loading states,
project-not-found, remote placeholder. Pas de XSS (le `projectId`
de `useParams()` est utilise uniquement en comparaison stricte `===`
ou passe dans `truncateHex()` qui slice puis rend en text child
React). `WebAppFrame.tsx` est un skeleton propre sans URL hardcodee.
Routes lazy-loaded correctement dans `App.tsx`.

- **[P1] D-01** : **`isLocal` est toujours `false` en production.**
  `BrowsedProject.tsx:101` : `const isLocal = healthQuery.data?.node_id === projectId`.
  `health.node_id` est le node_id iroh du **coordinator**
  (coordinator.py:164 : `self.state.node_id = self.state.node.node_id`
  cree avec `self._keypair.secret`). `projectId` est le node_id iroh
  du **daemon** (http.rs:423 : `project_id: state.node_id.clone()`
  cree par `create_node()` dans runtime.rs:141). Ce sont deux process
  iroh distincts avec des keypairs differentes — `isLocal` ne peut
  jamais etre `true`. En consequence, `LocalProjectApps` (qui rend
  TabView plein ecran) est du **dead code** en production. Tous les
  projets, y compris les locaux, affichent "Projet distant". Le
  test Vitest passe parce que le mock retourne le meme `node_id`
  pour les deux (line 118 + line 36).
  **Fix** : comparer le `projectId` contre les `project_id` des
  entries retournees par `/shell/discover` ou simplement contre le
  `node_id` du daemon (via un appel `/daemon/health` ou un champ
  ajoute a la reponse `/health` du coordinator).

- **[P3] D-02** : le chunk `BrowsedProject` partage le chunk
  `TabViewRenderer` (13.02 KB / 20 KB budget). Pas de tracking
  standalone dans size-limit. Minor.

---

## Track E — API schemas + backward compatibility

**Verdict** : PASS (0 P0, 0 P1, 0 P2, 1 P3)

Comparaison champ-par-champ Rust `BrowseEntry` (9 champs) vs Zod
`BrowseEntrySchema` (9 champs) : parfait alignement. Le champ
`source` est `#[serde(default)]` cote Rust (default `Curator`) et
`z.enum(...).optional()` cote Zod. Le test backward compat
`daemon.test.ts` verifie qu'un entry sans `source` parse sans
erreur. Aucun champ manquant ou excedentaire.

- **[P3] E-01** : asymetrie semantique de default : Rust
  `BrowseSource::Curator` vs TypeScript `undefined`. Si un
  composant fait `entry.source === "curator"`, il rate les entries
  d'un daemon v10 (pre-Sprint 11) ou `source` est `undefined`.
  Pas de crash, pas de test qui compare strictement.

---

## Track F — Deploy scripts correctness

**Verdict** : CONCERN (0 P0, 2 P1, 4 P2, 3 P3)

Les scripts sont fonctionnels dans leur premiere utilisation. Les
configs nginx (`provision.sh` inline vs `nginx-nexus.conf` standalone)
sont actuellement identiques en directives.

- **[P1] F-01** : `provision.sh:82` `ufw --force reset` detruit
  toutes les regles firewall existantes sur un VPS deja provisionne
  avant de les re-ajouter. Fenetre d'exposition securite entre le
  reset et les `ufw allow` suivants. Sur une VPS avec des regles
  personnalisees hors nexus-grid (ex: fail2ban), le reset les
  supprime silencieusement.
  **Fix** : remplacer `ufw --force reset` par des commandes
  idempotentes : `ufw allow ssh`, `ufw allow 80/tcp`, etc.
  directement, et `ufw --force enable` seulement.

- **[P1] F-03** : `deploy.sh` et `deploy-web.sh` utilisent
  `sudo systemctl reload/restart` en tant qu'utilisateur `nexus`,
  mais `provision.sh` n'installe jamais de regle sudoers NOPASSWD
  pour `nexus`. Les scripts de deploy echoueront ou bloqueront sur
  un prompt interactif sur le VPS.
  **Fix** : ajouter dans provision.sh :
  `echo "nexus ALL=(ALL) NOPASSWD: /usr/bin/systemctl" > /etc/sudoers.d/nexus`

- **[P2] F-02** : config nginx inline dans `provision.sh` (lines
  42-71) est une copie de `nginx-nexus.conf`. Aucune divergence
  actuelle, mais risque de drift futur. Le commentaire "edit
  deploy/nginx-nexus.conf upstream" est trompeur — ce fichier
  n'est jamais consomme par le script.

- **[P2] F-04** : `deploy-web.sh` et `deploy.sh --role web` font
  `rm -rf /opt/nexus-grid/web/*` puis `scp`. Si le `scp` est
  interrompu, le web root est vide et nginx retourne 404. Pas
  d'atomic swap (`web.new → web`).

- **[P2] F-05** : HTTP seulement (listen 80), pas de HTTPS/certbot.
  Scope cut declare (pas de custom domain), mais le traffic API et
  les apps sont servis en clair.

- **[P2] F-06** : `X-Forwarded-Proto $scheme` present dans le bloc
  `/api/` mais absent du bloc `/daemon/`. Inconsistance mineure.

- **[P3] F-07** : `$KEY` non-quote dans la composition des
  commandes SSH/SCP — un path avec espaces casserait le SSH.

- **[P3] F-08** : `StrictHostKeyChecking=accept-new` (TOFU) sans
  avertissement dans README.

- **[P3] F-09** : README ne documente pas la duplication nginx ni
  le setup TLS.

---

## Track G — Scope cuts verification

**Verdict** : PASS (0 P0, 0 P1, 0 P2, 2 P3)

Aucune fuite de scope cut detectee :
- Pas de blob upload UI (`grep -r "blob" web/src/` = 0 dans le code)
- Pas de branding SBFB dans le code user-facing
- Pas de cross-node fetch (BrowsedProject affiche placeholder "distant")
- Pas d'iframe avec contenu reel (WebAppFrame est dead code, jamais importe)

- **[P3] G-01** : `WebAppFrame.tsx` est fonctionnellement complet
  (accepte `blobUrl` et rend un `<iframe>`), pas vraiment un
  "skeleton". Dead code en production mais risque latent si
  `blobUrl` est plumbe accidentellement.

- **[P3] G-02** : le doc comment dit "skeleton" mais le composant
  implemente completement le rendu iframe. Surestimation dans les
  commentaires.

---

## Track H — Test coverage quality

**Verdict** : CONCERN (0 P0, 1 P1, 3 P2, 1 P3)

Les tests Rust (publish.rs, browse.rs) sont significatifs : roundtrips
serde, discrimination d'erreurs, TTL avec `thread::sleep`, integration
2-node avec vrais noeuds iroh. Les +12 tests supplementaires vs plan
sont du vrai signal (tests direct_entry, schema, probe).

- **[P1] H-04** : les responses mock dans `test_daemon_proxy.py`
  pour `/browse` omettent le champ `source` ajoute en Sprint 11.
  Le mock ne reflete pas le format wire reel — un bug de
  backward compat cote daemon serait invisible ici.
  **Fix** : ajouter `"source": "curator"` dans les canned responses
  des tests browse du daemon proxy.

- **[P2] H-01** : `BrowsedProject.tsx` (421 LOC, le composant
  principal du Sprint 11) est exclu de `vitest.config.ts`
  `coverage.include`. Sa couverture branches/lines est invisible
  au seuil 85%/78%. Regressions de coverage non detectees par CI.

- **[P2] H-02** : les 3 specs Playwright `browse-click-project.spec.ts`
  testent uniquement les chemins d'erreur (daemon offline, project
  not found). Le happy path (projet local → sidebar + TabView) n'a
  pas de couverture E2E automatisee. Consequence directe de D-01
  (le feature est casse, donc non testable en E2E).

- **[P2] H-03** : `aggregate_flattens_curator_lists_with_cached_status`
  dans browse.rs est quasi-creux : l'entry curator est creee,
  discardee avec `let _ = entry`, et le test asserte
  `out.is_empty()` (identique au test precedent). Le scenario
  qu'il pretend couvrir (flattening avec cache) n'est pas exerce.

- **[P3] H-05** : 7 Vitest BrowsedProject (5 page + 2 WebAppFrame)
  vs 12 planifies — le delta est absorbe par les tests schema/daemon
  cote Phase A Rust.

---

## Track I — SPDX + hygiene

**Verdict** : PASS (0 P0, 0 P1, 0 P2, 2 P3)

Tous les checks passent :
- `bash scripts/check-spdx.sh` → 209 fichiers conformes
- `cargo fmt --all --check` → exit 0
- `cargo clippy --workspace --all-targets --locked -- -D warnings` → 0 warning
- `bash web/scripts/scan-en-strings.sh` → "French-only, clean"
- `npm audit --audit-level=high` → 0 vulnerabilites
- Tous les deploy scripts (.sh, .conf, .toml) ont les headers SPDX

- **[P3] I-01** : `web/tests/browse-click-project.spec.ts` n'a pas de
  header SPDX — coherent avec la convention du projet (`web/tests/`
  exclu de `check-spdx.sh`), mais gap de couverture non documente.

- **[P3] I-02** : `runtime.rs:86` et `:282` portent
  `#[allow(dead_code)]` avec commentaire "Phase D will reach into it"
  — reference de phase stale (Phase D = deploy, pas filtre browse).
  Pre-existant Sprint 7. Pas d'action.

---

## Table recapitulative des findings

| ID | Track | Severity | Description |
|---|---|---|---|
| D-01 | D | **P1** | `isLocal` toujours false — LocalProjectApps dead code en production |
| F-01 | F | **P1** | `ufw --force reset` detruit les regles existantes |
| F-03 | F | **P1** | Pas de sudoers pour user `nexus` — deploy scripts bloquent |
| H-04 | H | **P1** | Mock daemon proxy omet `source` — wire drift masque |
| A-01 | A | P2 | Pas de test message gossip tronque |
| A-02 | A | P2 | `node_id` non valide dans ProjectAnnouncement |
| B-01 | B | P2 | Pas de test daemon retourne HTTP 500 |
| B-02 | B | P2 | Pas de test auto-publish private |
| C-01 | C | P2 | `default_curators` pas valide hex au config load |
| C-02 | C | P2 | `GET /default-curators` retourne strings brutes |
| F-02 | F | P2 | Config nginx inline diverge potentiellement de nginx-nexus.conf |
| F-04 | F | P2 | deploy-web.sh rm-rf sans rollback atomique |
| F-05 | F | P2 | Pas de HTTPS/certbot |
| F-06 | F | P2 | X-Forwarded-Proto manquant dans /daemon/ |
| H-01 | H | P2 | BrowsedProject.tsx hors coverage.include |
| H-02 | H | P2 | Playwright specs = error paths only, pas de happy path |
| H-03 | H | P2 | Test aggregate_flattens quasi-creux |
| E-01 | E | P3 | Asymetrie default source Rust vs TS |
| G-01 | G | P3 | WebAppFrame fonctionnel pas skeleton |
| G-02 | G | P3 | Doc comment "skeleton" surestimee |
| I-01 | I | P3 | web/tests/ sans SPDX (convention) |
| I-02 | I | P3 | allow(dead_code) commentaire Phase D stale |
| + | A,B,C,F | P3 | Nits divers (double parse, imports, quotes, doc) |

---

## Commits fix attendus (CONDITIONAL PASS)

Avant Sprint 12 Phase A, 2-3 commits `fix(sprint11): ...` :

1. **`fix(sprint11): isLocal check uses daemon node_id instead of coordinator`**
   - `web/src/pages/BrowsedProject.tsx` : remplacer la comparaison
     `health.node_id === projectId` par une logique qui determine
     la localite via le daemon (ex: `entry.project_id` present dans
     les entries retournees par `/shell/discover` avec status local,
     ou comparer contre le `node_id` du daemon via un nouvel endpoint
     ou un champ ajoute a `/health`)
   - Mettre a jour le test mock pour refleter le vrai contrat
   - Tester manuellement : un projet local doit afficher TabView,
     pas "Projet distant"

2. **`fix(sprint11): provision.sh idempotent ufw + sudoers for nexus`**
   - Supprimer `ufw --force reset`, garder les `ufw allow` directs
   - Ajouter `echo "nexus ALL=..." > /etc/sudoers.d/nexus`
   - Ajouter `"source": "curator"` dans les mock responses de
     `test_daemon_proxy.py` (H-04)

---

## P2 a logger en tech debt

Les items P2 suivants doivent etre loggues dans
`docs/shell/PATTERNS.md` et/ou `docs/rust/PATTERNS.md` :

- **T28** : validation `node_id` hex dans `ProjectAnnouncement::from_gossip_bytes()` (A-02)
- **T29** : test truncated gossip message (A-01)
- **T30** : test daemon 500 + auto-publish private (B-01, B-02)
- **T31** : validation hex `default_curators` au config load (C-01)
- **T32** : DRY nginx config provision.sh → `cp nginx-nexus.conf` (F-02)
- **T33** : atomic swap deploy web (F-04)
- **T34** : HTTPS/certbot pour VPS (F-05)
- **T35** : BrowsedProject.tsx dans vitest coverage.include (H-01)
- **T36** : test browse aggregate_flattens non-creux (H-03)

---

## P3 laisses sans action

- E-01, G-01, G-02, I-01, I-02, A-03, A-04, B-03, B-04, C-03, C-04,
  D-02, F-07, F-08, F-09, H-05 — nits optionnels.

---

## Notes on audit completeness

- Les tracks A-I ont ete joues par 7 agents paralleles avec
  verification croisee manuelle du finding D-01 (lecture directe
  de la chaine `project_id` dans http.rs:423, coordinator.py:164,
  et BrowsedProject.tsx:101).
- Les tests Rust (331), Python (302+1), Vitest (173) et Playwright
  (30) ont ete executes par les agents d'audit.
- Le finding D-01 est le plus critique : il invalide le happy path
  du goal Sprint 11 ("un nouveau noeud clique et voit l'app en
  plein ecran"). Le test Vitest passe grace aux mocks identiques,
  mais le comportement production est casse.
- Le Track F contient beaucoup de findings parce que c'est le
  premier sprint avec des deploy scripts commites — normal pour
  un premier pass ops.
