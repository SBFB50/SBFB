# Sprint 14 — Kickoff (Deploy verifie from source + provenance SLSA L1)

**Ecrit** : 2026-04-13
**Tip master d'entree** : `0253922` (Sprint 13 audit findings + P1 fix stale running.json)
**Phase 0 audit** : DONE. Sprint 13 audit CONDITIONAL PASS leve
dans `7745384` (1 P1 fixe : stale running.json detection via TCP
health check). Gate verte.

---

## 1. Constat d'entree

### 1.1 Etat du repo

- Sprints 0-13 **CLOSED**. v1.0.0 released + rendu universel +
  bridge postMessage + open source enforcement + launcher.
- Le flow P2P fonctionne de bout en bout : publication → distribution
  → rendu iframe isole → communication bridge
- **Manque critique** : le flow `POST /project/deploy` accepte un
  zip uploade par l'utilisateur. Aucune garantie que le zip correspond
  au repo source declare. Un publisher peut avoir un repo propre et
  deployer un zip different contenant du code malveillant.
- **Manque produit** : les apps publiques affichent un lien vers le
  repo source (`repo_url`), mais ce lien est un champ de confiance —
  n'importe qui peut mettre n'importe quelle URL. Le reseau ne
  verifie rien.
- **Manque securite** : `repo_url` accepte toute string non-vide
  (P2 B-1 Sprint 13 audit), y compris `javascript:` — XSS possible
  au clic dans Browse/BrowsedProject.
- 3 P2 Sprint 13 a logger : B-1 (repo_url XSS, remplace par ce
  sprint), D-1 (text-white/30 accessibilite, 2 instances), G-1
  (PAD_R 16 ≠ React 32).

### 1.2 Compteurs de tests a l'entree (tip `0253922`)

| Suite | Count |
|---|---|
| Rust workspace | 369 |
| Python SDK | 183 |
| Python coordinator | 99 + 1 skipped |
| Python app-gov | 46 |
| Vitest unit | 191 |
| Playwright | 30 |
| size-limit | 7/7 |
| SPDX | 220/220 |

### 1.3 Le probleme

Le reseau SBFB promet un "app store open source par construction".
Aujourd'hui cette promesse repose sur un champ `repo_url` declaratif
que personne ne verifie. C'est l'equivalent d'un app store ou le
developpeur ecrit "trust me, voici mon code source" sans qu'aucun
mecanisme ne le confirme.

Pour que la promesse soit reelle, le code distribue sur le reseau
doit etre **prouvablement identique** au code du repo source. Et
cette preuve doit etre **verifiable offline** par n'importe quel
noeud du reseau.

### 1.4 Vision sprint

Sprint 14 transforme le flow de publication publique : le
coordinateur clone le repo lui-meme, verifie la propriete via
SBFB.json (pattern Keyoxide), construit le zip, et signe une
attestation de provenance (SLSA L1). Le code sur le reseau = le
code du repo. Garanti cryptographiquement.

---

## 2. Goal en une phrase

**Les apps publiques sont deployees depuis le repo source par le
coordinateur, avec preuve de propriete Keyoxide et provenance
signee SLSA L1, verifiable offline par tout noeud du reseau.**

---

## 3. Phase 0 — Audit Sprint 13

DONE. Verdict CONDITIONAL PASS, 1 P1 fixe (stale running.json
detection via TCP health check, commit `7745384`). 3 P2 logges
(B-1 repo_url XSS, D-1 text-white/30, G-1 PAD_R). Gate verte.
Cf. `sprint13_audit_findings.md`.

---

## 4. Decisions Day 0 (D1..D5 gelees)

### D1 — Deploy from source remplace deploy upload pour le public

**Retenu** : nouveau endpoint `POST /project/deploy-from-repo`
qui accepte `{repo_url, commit_sha?}`. Le coordinateur :
1. Clone `git clone --depth 1 --single-branch` (max 500 MB, 30s timeout)
2. Verifie `SBFB.json` a la racine (node_id Ed25519 matche le daemon)
3. Verifie que le repo est public (HTTP HEAD sur URL forge)
4. Verifie `index.html` existe
5. Zip le contenu (exclut `.git/`, valide les chemins, pas de symlinks)
6. Genere `provenance.json` signe (repo_url, commit_sha,
   artifact_hash BLAKE3, node_id, timestamp, signature Ed25519)
7. Inclut `provenance.json` dans le zip
8. Deploie via le flow blob existant

**Rejete** : garder le flow upload zip pour le public. Le publisher
controle le contenu du zip — impossible de garantir qu'il correspond
au repo. Aussi rejete : hash comparison (builds web non-reproductibles),
F-Droid rebuilder (trop lourd pour des apps web statiques).

**Implications** :
- Nouveau module `forge.py` (parsers URL multi-forge)
- Nouveau module `provenance.py` (generation + signature Ed25519)
- `deploy.py` gagne le nouvel endpoint
- L'ancien `POST /project/deploy` (upload zip) reste pour les apps
  **privees** uniquement
- Le frontend redirige les apps publiques vers le nouveau flow

### D2 — Provenance signee SLSA L1 avec domain separation

**Retenu** : `provenance.json` est signe avec la cle Ed25519 du
coordinateur (la meme que pour les kudos). Le format suit SLSA L1
(auto-attestation signee). La signature utilise la convention
`canonical_bytes` du projet (JCS + domain tag
`nexus-provenance-v1`).

Le fichier `provenance.json` est inclus dans le zip deploy et
son hash BLAKE3 est propage dans l'annonce gossip.

**Rejete** : Sigstore (centralise, log de transparence externe).
Aussi rejete : provenance dans un champ gossip separe (complexifie
le parsing, le zip est l'unite atomique de distribution).

**Implications** :
- `provenance.json` dans chaque zip d'app publique
- Hash du provenance dans `ProjectAnnouncement` v4
- N'importe quel noeud peut verifier : telechager le blob, lire
  `provenance.json`, verifier la signature avec la cle publique
  du node_id annonce

### D3 — ProjectAnnouncement v4 + BrowseEntry avec provenance

**Retenu** : bump version a 4. Nouveau champ optionnel
`provenance_hash: Option<String>` (BLAKE3 hex du
`provenance.json`). BrowseEntry gagne le meme champ. Le frontend
affiche un badge "Verifie" (vert) quand `provenance_hash` est
present, "Non-verifie" (gris) sinon.

**Rejete** : ajouter un champ `verified: bool` dans le wire
format. La verification est une decision locale du noeud receveur,
pas un champ trust-based dans le message.

**Implications** :
- Backward compat : v3 sans provenance_hash parse normalement
  (serde default + skip_serializing_if)
- Zod schema : `provenance_hash: z.string().optional()`
- Badge conditionnel dans Browse cards + BrowsedProject

### D4 — Securite du clone : defense en profondeur

**Retenu** : 7 protections cumulees pour le `git clone` :

| Protection | Implementation |
|---|---|
| Clone minimal | `--depth 1 --single-branch` |
| Taille max | 500 MB, rejet si depasse (check `Content-Length` HEAD + taille post-clone) |
| Timeout | 30s via `asyncio.wait_for` |
| Pas de .git/ | Exclu du zip |
| Path traversal | Valider chaque chemin (`..` interdit, pas de symlinks) |
| Pas de submodules | Clone sans `--recursive`, ignore `.gitmodules` |
| MIME scan | Pas de binaires executables dans le zip (optionnel P2) |

**Rejete** : faire confiance au publisher pour la taille. Aussi
rejete : sandboxer le clone dans un container (overengineered pour
le MVP, le timeout + size limit suffisent).

**Implications** :
- Toutes les protections dans `deploy.py` autour du subprocess
  `git clone`
- Tmpdir automatique pour le clone, supprime apres zip
- Tests pour chaque protection

### D5 — P2 Sprint 13 fermes + tech debt logge

**Retenu** :
- D-1 : `text-white/30` → `text-white/40` sur 2 instances
  (BrowsedProject.tsx:271, ProjectDetail.tsx:131). ~2 LOC.
- G-1 : `_SVG_PAD_R` 16 → 32 pour aligner sur React. ~1 LOC.
- B-1 : remplace entierement par le deploy verifie (le champ
  `repo_url` declaratif n'est plus le mecanisme de confiance).
  Logger en tech debt avec note "superseded by verified deploy".
- Les 5 P3 Sprint 13 restent sans action.

---

## 5. Plan Phase outline

### Phase A — Python deploy-from-repo + forge + provenance

**Scope** :
- Nouveau module `packages/nexus-coordinator/src/nexus_coordinator/forge.py` :
  parsers URL multi-forge (GitHub, GitLab, Codeberg/Gitea), detection
  de forge, construction URL raw pour SBFB.json pre-check
- Nouveau module `packages/nexus-coordinator/src/nexus_coordinator/provenance.py` :
  generation `provenance.json` (schema_version, repo_url, commit_sha,
  artifact_hash BLAKE3, node_id, timestamp), signature Ed25519 via
  nexus_core bindings, verification
- Nouvel endpoint `POST /project/deploy-from-repo` dans `deploy.py` :
  clone → verify SBFB.json → verify public → verify index.html →
  zip → sign provenance → include in zip → blob store → publish
- Securite clone : subprocess git avec timeout 30s, tmpdir, size
  check, path validation, no .git/, no submodules
- Tests : happy path, missing SBFB.json, wrong node_id, missing
  index.html, private repo reject, oversized repo, path traversal
  reject

**Critere** : `POST /project/deploy-from-repo` avec un repo public
contenant SBFB.json + index.html → 200 + zip deploye avec
provenance.json signe. Tous les cas d'erreur → 400/422 avec message
explicite.

**Commit** : `feat(coordinator): Sprint 14 Phase A — deploy from repo with Keyoxide + SLSA L1 provenance`

### Phase B — Rust ProjectAnnouncement v4 + BrowseEntry provenance

**Scope** :
- `publish.rs` : bump `PROJECT_ANNOUNCEMENT_VERSION` a 4, ajouter
  `provenance_hash: Option<String>` avec serde skip
- `browse.rs` : `BrowseEntry` gagne `provenance_hash: Option<String>`
- `http.rs` : `PublishRequest` gagne `provenance_hash: Option<String>`,
  propage au BrowseEntry direct
- Tests Rust : backward compat v3→v4 round-trip, v4 avec provenance
  round-trip, v3 sans provenance parse OK
- `web/src/api/daemon.ts` : Zod schema update `provenance_hash`

**Critere** : un daemon v4 publie un announcement avec
`provenance_hash`, un daemon v3 le parse sans erreur (champ ignore).
Zod accepte les deux formats.

**Commit** : `feat(p2p): Sprint 14 Phase B — ProjectAnnouncement v4 with provenance hash`

### Phase C — Frontend badge "Verifie" + P2 tech debt

**Scope** :
- Badge "Verifie" (icone shield/check vert) dans Browse.tsx cards
  quand `entry.provenance_hash` present
- Badge "Verifie" dans BrowsedProject.tsx top bar
- Lien provenance : "Source" pointe vers le repo du provenance
  (commit exact si disponible)
- Fix D-1 : `text-white/30` → `text-white/40` (2 instances)
- Fix G-1 : `_SVG_PAD_R` 16 → 32
- Logger B-1/D-1/G-1 dans PATTERNS.md comme T41-T43
- Vitest tests pour le badge (present/absent selon provenance_hash)

**Critere** : une entry avec `provenance_hash` affiche le badge
vert. Une entry sans affiche rien. Les 2 fixes P2 sont appliques.
Vitest + scan-en-strings verts.

**Commit** : `feat(web): Sprint 14 Phase C — verified badge + P2 tech debt Sprint 13`

### Phase D — Tests integration + polish

**Scope** :
- Test integration Python : deploy-from-repo → blob stored → publish
  avec provenance_hash → browse retourne entry avec provenance
- Verification que le flow complet fonctionne avec le fake daemon
  des tests
- Playwright test si faisable (deploy-from-repo + badge visible)
- SPDX headers sur les nouveaux fichiers (forge.py, provenance.py)
- Nettoyage : s'assurer que `POST /project/deploy` (upload zip)
  rejette les apps publiques avec un message redirigant vers
  deploy-from-repo

**Critere** : tests integration verts. Le flow complet public
passe par deploy-from-repo. L'ancien endpoint redirige les publics.

**Commit** : `feat(coordinator): Sprint 14 Phase D — integration tests + deploy public redirect`

### Phase E — Docs (verification + audit plan)

**Scope** :
- `sprint14_verification.md` avec checklist fail-fast remplie
- `sprint14_audit_plan.md` pour Sprint 15 Phase 0
- Update `docs/shell/PATTERNS.md` (T41-T43, nouveaux patterns)
- Update `docs/rust/PATTERNS.md` si applicable (PA v4)

**Commit** : `docs(sprint14): verification + audit plan for Sprint 15`

---

## 6. Scope cuts (PAS dans ce sprint)

- CPU watchdog iframe → Sprint 15 (le bridge n'est pas assez mature,
  besoin de retour d'experience sur deploy verifie d'abord)
- Bridge push bidirectionnel → Sprint 15 (prerequis pour watchdog)
- Runtime templates (`sbfb publish --type python`) → Sprint 15
  (necessite deploy verifie fonctionnel d'abord)
- Re-publish automatique sur repo update → Sprint 15 (webhook ou
  polling, necessite deploy verifie)
- Branding SBFB (nom, logo, favicon) → Sprint 15
- Origin separee par subdomain blob-serve → Sprint 15+
- 2 VPS supplementaires (US/Asia) → Sprint 15
- MIME scan executables dans le zip → Sprint 15 (P2, la sandbox
  iframe + CSP suffisent pour le MVP)
- Builds reproductibles (hash comparison cross-nodes) → v1.2+
  (les builds web ne sont pas deterministes)
- Multi-writer iroh-docs → v1.1+
- Custom domain / DNS → v1.2+

## 7. Tracabilite scope (items differes des sprints precedents)

| Item | Origine | Sprint 14 |
|---|---|---|
| CPU watchdog | Sprint 13 D6, Sprint 14 scope cut | Differe Sprint 15 |
| Bridge push | Sprint 13 roadmap | Differe Sprint 15 |
| Runtime templates | Sprint 12, 13 scope cut | Differe Sprint 15 |
| Re-publish auto | Sprint 12, 13 scope cut | Differe Sprint 15 |
| Branding SBFB | Sprint 10, 12, 13 scope cut | Differe Sprint 15 |
| Origin subdomain | Sprint 12, 13 scope cut | Differe Sprint 15+ |
| VPS US/Asia | Sprint 12, 13 scope cut | Differe Sprint 15 |
| Deploy verifie | Sprint 13 D6 / decision 2026-04-13 | **Phase A-D** |
| PA v4 provenance | Decision 2026-04-13 | **Phase B** |
| Badge verifie | Decision 2026-04-13 | **Phase C** |
| P2 D-1/G-1 fixes | Sprint 13 audit | **Phase C** |
| P2 B-1 superseded | Sprint 13 audit | **Phase C** (log only) |

---

## 8. Audit gate pattern — rappel

Phase 0 Sprint 13 jouee et fermee (CONDITIONAL PASS leve).
Phase E de ce sprint produira `sprint14_audit_plan.md` pour que
Sprint 15 Phase 0 audite independamment.

---

## 9. Estimations LOC

| Phase | LOC estimee | Repartition |
|---|---|---|
| A — Deploy from repo | ~500 | 150 forge + 120 provenance + 130 deploy + 100 tests |
| B — Rust PA v4 | ~200 | 60 publish + 40 browse + 30 http + 70 tests |
| C — Frontend badge + P2 | ~200 | 80 Browse + 60 BrowsedProject + 30 fixes + 30 tests |
| D — Integration + polish | ~200 | 100 integration tests + 50 redirect + 50 SPDX |
| E — Docs | ~300 | verification + audit plan + PATTERNS |
| **Total** | **~1400** | |

---

## 10. Checkpoint de validation

Avant de passer au plan detaille, confirmer :

1. D1 (deploy from source remplace upload pour public) est valide
2. D2 (provenance signee SLSA L1, domain `nexus-provenance-v1`) est valide
3. D3 (PA v4 avec provenance_hash, badge conditionnel) est valide
4. D4 (7 protections clone) est valide
5. D5 (P2 D-1/G-1 fixes, B-1 superseded) est valide
6. L'ordre des phases (A Python → B Rust → C Frontend → D Integration → E Docs) est OK
7. Les scope cuts (CPU watchdog, templates, push, branding, VPS → Sprint 15) sont acceptes
