# Sprint 11 — Kickoff (P2P end-to-end : publish + discovery + render)

**Ecrit** : 2026-04-12 (corrige 2026-04-12 post-audit)
**Tip master d'entree** : `4d04ac4` (Sprint 10 audit findings landed)
**Phase 0 audit** : DONE. Sprint 10 audit CONDITIONAL PASS leve
dans `4d04ac4` (P1 V-1 fixe, T23-T25 logges). Gate verte.

---

## 1. Constat d'entree

### 1.1 Etat du repo

- Sprints 0-10 **CLOSED**. v1.0.0 released.
- Le flow P2P **consommateur** fonctionne : Browse page affiche
  les projets des curators abonnes, badges reachability live.
- Le flow P2P **publication** n'existe pas : aucun moyen pour
  un coordinator de s'annoncer, aucun moyen de publier une app
  web sur le reseau.
- L'UX apps est un accordeon avec "Invoquer" — pas de vue
  plein ecran accessible sans Ctrl+K ou URL manuelle.

### 1.2 Compteurs de tests a l'entree

| Suite | Count |
|---|---|
| Rust workspace | 312 |
| Python SDK | 167 |
| Python coordinator | 83 + 1 skipped |
| Python app-gov | 46 |
| Vitest unit | 161 |
| Playwright | 27 |
| size-limit | 7/7 |
| SPDX | 204/204 |

### 1.3 Le probleme

Un utilisateur qui telecharge nexus-grid, le lance, et ouvre
Browse voit : **rien**. Parce que :
1. Personne ne peut publier de projet sur le DHT
2. Meme si quelqu'un publiait, il faudrait connaitre le pubkey
   du curator et s'y abonner manuellement
3. Meme si un projet apparaissait, cliquer dessus montre un
   accordeon avec "Invoquer", pas une app web

Le produit est inutilisable pour un utilisateur non-dev.

---

## 2. Goal en une phrase

**Un coordinator public s'annonce sur le reseau, un nouveau
noeud le decouvre dans Browse, clique, et voit l'app web
rendue en plein ecran — pour n'importe quel framework
(React, Vue, HTML statique).**

---

## 3. Decisions Day 0 (D1..D5 gelees)

### D1 — Apps web = blobs statiques rendus dans iframe sandboxee

**Retenu** : une app web publiee sur nexus-grid est un
dossier `dist/` (output de n'importe quel build : React,
Vue, Svelte, HTML pur) uploade comme blob iroh-blobs. Le
shell React fetch le blob et le rend dans une `<iframe
sandbox="allow-scripts allow-same-origin">`.

**Rejete** :
- TabView only (trop restrictif, pas d'app custom)
- Micro-frontends / module federation (trop complexe)
- WebAssembly container (overkill pour du hosting statique)

**Implications** :
- Le coordinator a un nouvel endpoint `POST /app/{name}/publish`
  qui accepte un tarball/zip du `dist/`
- Le blob hash + metadata sont propages via gossip
- Le shell daemon fetch le blob a la demande
- Les apps TabView existantes (gov, hello) continuent de
  fonctionner — c'est un mode additionnel, pas un remplacement

### D2 — Self-publish via pkarr : le coordinator annonce ses projets publics

**Retenu** : quand un coordinator demarre avec
`visibility = public`, le shell daemon publie un record pkarr
contenant le node_id + project metadata. Les autres daemons
le trouvent via DHT lookup.

**Rejete** : publication manuelle (trop de friction).

**Implications** :
- `nexus-shell-daemon-core` gagne une fonction `publish_project`
- Le coordinator appelle le daemon au demarrage si visibility=public
- Le record pkarr contient : node_id, project_name, app_blob_hash

### D3 — Default curator FlowUP pre-configure

**Retenu** : tout nouveau noeud est pre-abonne au curator
FlowUP (hardcode du pubkey dans le config par defaut). Le
VPS EU cree automatiquement une curator list des apps
officielles au boot.

**Rejete** : zero default subscription (Browse vide pour
tout le monde au premier lancement).

**Implications** :
- `nexus-shell-daemon-core/src/config.rs` : default
  `attention` set avec le pubkey FlowUP
- Le VPS EU boot script cree + signe + broadcast la curator
  list

### D4 — Browse → clic → app plein ecran

**Retenu** : cliquer sur un projet dans Browse navigue vers
`/browse/{project_id}` qui affiche l'app dans une iframe
plein ecran (si app web statique) ou les onglets TabView
(si app SDK classique). Sidebar avec les infos du projet.

**Rejete** : garder l'accordeon/invoquer actuel.

### D5 — VPS EU = premier noeud live du reseau

**Retenu** : le VPS EU (`135.181.42.188`) fait tourner :
- `nexus-shell-daemon` (DHT bootstrap)
- `nexus-coordinator` + app gov (projet public)
- nginx sert le shell web build sur port 80

Un visiteur sur `http://135.181.42.188` voit le shell web,
Browse liste le projet gov, clic → app gov plein ecran.

---

## 4. Plan Phase outline A..E

### Phase A — Self-publish coordinator → pkarr DHT (1-2j)

- Endpoint coordinator `POST /project/publish` qui signale au
  daemon de publier le record pkarr
- `nexus-shell-daemon-core` : `publish_project()` publie un
  signed record sur le DHT avec node_id + metadata
- Le daemon browse aggregator reconnait les records publies
  directement (pas seulement via curator lists)
- Tests Rust + Python
- **Commit** : `feat(p2p): Sprint 11 Phase A — self-publish
  coordinator projects via pkarr DHT`

### Phase B — Auto-curator + default subscription (1j)

- Default attention set avec pubkey FlowUP hardcode dans
  `config.rs`
- Script `deploy/create-curator-list.sh` : genere + signe +
  broadcast une curator list sur le VPS EU
- Coordinator endpoint `GET /daemon/default-curators` pour
  que le shell puisse pre-peupler
- Tests
- **Commit** : `feat(p2p): Sprint 11 Phase B — default FlowUP
  curator + auto-subscription`

### Phase C — Browse → app plein ecran UX (1-2j)

- Nouvelle route `/browse/:projectId` avec layout plein ecran
- Si le projet a un app blob → iframe sandboxee
- Si le projet a des TabView apps → rendu TabView directement
  (plus d'accordeon)
- Sidebar : nom projet, curator, status, metadata
- Refacto `Browse.tsx` : les cards deviennent cliquables avec
  `navigate(/browse/${id})`
- Tests Vitest + Playwright
- **Commit** : `feat(web): Sprint 11 Phase C — Browse full-screen
  app rendering`

### Phase D — Deploy VPS EU live (1j)

- Build web shell + upload sur VPS nginx
- Install coordinator + app gov sur VPS
- Demarrer shell daemon + coordinator en systemd
- Creer la curator list FlowUP
- Smoke test : `http://135.181.42.188` affiche le shell,
  Browse montre gov, clic → app gov rendue
- **Commit** : `feat(deploy): Sprint 11 Phase D — VPS EU live
  with coordinator + shell web`

### Phase E — Verification + audit plan (0.5j)

- `.planning/sprint11_verification.md`
- `.planning/sprint11_audit_plan.md`
- Update memory + docs
- **Commit** : `docs(sprint11): verification + audit plan for
  Sprint 12`

---

## 5. Scope cuts

- **Pas de upload blob via UI** — publish est CLI-only Sprint 11
- **Pas de branding SBFB** — Sprint 12+
- **Pas de 2 VPS supplementaires** (US/Asia) — Sprint 12+
- **Pas de multi-writer iroh-docs** — Sprint 12+
- **Pas de monetisation / tokens** — hors scope
- **Pas de sandboxing CSP avance pour les iframes** — basic
  sandbox attrs suffisent pour v1
- **Pas de custom domain / DNS** — acces par IP

---

## 6. Risks

| # | Risk | Mitigation |
|---|---|---|
| R1 | pkarr publish API pas stable iroh 0.97 | Verifier dans le registry local avant de coder |
| R2 | iframe CORS bloque les assets relatifs | Servir le blob via un endpoint local du coordinator |
| R3 | VPS EU pas assez puissant pour coordinator + nginx + daemon | CX33 a 4 vCPU / 8 GB, largement suffisant |
| R4 | Le blob fetch est lent pour une grosse app | Limiter a 50 MB (meme cap que file upload) |

---

## 7. Checkpoint de validation

L'utilisateur confirme :
1. **D1** : apps = blobs statiques + iframe
2. **D2** : self-publish pkarr automatique
3. **D3** : default curator FlowUP hardcode
4. **D4** : Browse → clic → plein ecran
5. **D5** : VPS EU = premier noeud live
6. **Audit skip** : Phase 0 sautee, rattrapage Sprint 12
