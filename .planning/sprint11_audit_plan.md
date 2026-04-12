# Sprint 11 — Audit plan pour Sprint 12 Phase 0

**Sprint audite** : Sprint 11 (5 commits `cea0c2b` → `999fec6`)
**Goal du sprint** : Un coordinator public s'annonce sur le reseau,
un nouveau noeud le decouvre dans Browse, clique, et voit l'app
rendue en plein ecran.

---

## 0. Mode d'emploi pour la session fraiche

### Ordre de lecture impose

1. Memory : `MEMORY.md` → `nexus_grid_pivot.md` → `sprint_audit_gate.md` → `feedback_approach.md`
2. `docs/claude/README.md` — la methode de travail
3. `git log --oneline cea0c2b..999fec6` — les commits du sprint
4. `.planning/sprint11_kickoff.md` — decisions D1-D5
5. `.planning/sprint11_plan.md` — plan detaille phases A-E
6. `.planning/sprint11_verification.md` — self-report (lire APRES avoir forme ton opinion)

### NE PAS LIRE avant d'avoir forme ton opinion

- `docs/shell/PATTERNS.md` (P18-P20, T26-T27) — lire seulement apres les tracks A-H pour comparer
- `docs/rust/PATTERNS.md` — pas touche Sprint 11, mais eviter le biais de confirmation

### Timebox

2-3h. Le signal prime sur le volume. 9 tracks ci-dessous,
chacune avec une question centrale et des methodes concretes.

### Delivrable final

`.planning/sprint11_audit_findings.md` avec :
- Verdict global (PASS / CONDITIONAL PASS / FAIL)
- Findings tries P0..P3
- Commits fix attendus si CONDITIONAL PASS

---

## Track A — Self-publish gossip correctness

**Question** : est-ce que `publish.rs` + `iroh_runtime.rs` +
`browse.rs` implementent correctement le self-publish via gossip,
et est-ce que le message format est robuste ?

**Methodes** :
1. Lire `crates/nexus-shell-daemon-core/src/publish.rs` : verifier
   que `ProjectAnnouncement` serde roundtrip est teste et que les
   champs sont valides
2. Lire `browse.rs` : verifier que `BrowseSource::Direct` est
   correctement traite dans `add_direct_entry()` et que la dedup
   par `project_id` fonctionne
3. Lire `iroh_runtime.rs` : verifier que `process_announcement_bytes()`
   branche correctement entre curator messages et project messages,
   et qu'un message malformed est rejete proprement (pas de panic)
4. `cargo test -p nexus-shell-daemon-core --locked` : verifier que
   les tests publish/browse passent et couvrent les edge cases
5. Grep `unwrap()` dans publish.rs / browse.rs : un unwrap sur un
   parse utilisateur serait un P0

**Signal** :
- P0 : panic sur message gossip malformed
- P1 : pas de validation du champ `v` (version drift silencieuse)
- P2 : pas de test de rejection sur message tronque
- P3 : naming/doc nits

---

## Track B — Coordinator auto-publish integration

**Question** : est-ce que le coordinator publie correctement au
demarrage quand `visibility=public`, et est-ce que le endpoint
`POST /project/publish` fonctionne ?

**Methodes** :
1. Lire `packages/nexus-coordinator/src/nexus_coordinator/coordinator.py`
   dans `start()` : verifier l'etape auto-publish (apres quel step,
   condition visibility, handling daemon unreachable)
2. Lire `packages/nexus-coordinator/src/nexus_coordinator/api/health.py` :
   verifier l'endpoint POST /project/publish
3. Lire les tests `tests/test_daemon_proxy.py` : verifier que les
   4 cas sont couverts (publish OK, daemon down, auto-publish public,
   auto-publish private)
4. Grep `httpx` dans le test : verifier que le mock daemon est
   correctement configure et ne masque pas un bug de URL

**Signal** :
- P0 : auto-publish bloque le boot si daemon down
- P1 : endpoint POST expose sans validation de body
- P2 : pas de test quand daemon retourne une erreur 500
- P3 : naming nits

---

## Track C — Default curators + auto-subscription

**Question** : est-ce que la config `[curator]` est parsee
correctement, l'auto-subscribe est idempotent, et le endpoint
`GET /default-curators` retourne les bonnes donnees ?

**Methodes** :
1. Lire `crates/nexus-shell-daemon-core/src/config.rs` : verifier
   `CuratorConfig` avec `#[serde(default)]`
2. Lire `crates/nexus-shell-daemon/src/runtime.rs` : verifier le
   loop auto-subscribe au boot (idempotence, log)
3. Lire `crates/nexus-shell-daemon/src/http.rs` : verifier
   `GET /default-curators`
4. `cargo test -p nexus-shell-daemon-core -p nexus-shell-daemon --locked`
   pour les tests config + subscribe
5. Lire `deploy/config.toml.example` : verifier que la section
   `[curator]` est presente et commentee

**Signal** :
- P0 : config sans `[curator]` fait crasher le parse
- P1 : auto-subscribe duplique les subscriptions a chaque reboot
- P2 : `default_curators` n'est pas valide comme hex pubkey
- P3 : doc/commentaire nits

---

## Track D — Browse full-screen UX

**Question** : est-ce que `/browse/:projectId` affiche correctement
le projet avec sidebar + TabView, et est-ce que le fallback distant
est propre ?

**Methodes** :
1. Lire `web/src/pages/BrowsedProject.tsx` : verifier la logique
   local vs distant, le rendu TabView, le fallback "noeud distant"
2. Lire `web/src/pages/Browse.tsx` : verifier que les cards sont
   cliquables et que le `project_id` est bien extrait
3. Lire `web/src/App.tsx` : verifier la route lazy-loaded
4. Lire `web/src/components/app/WebAppFrame.tsx` : verifier que
   c'est un skeleton sans URL hardcodee
5. Lancer `npx playwright test tests/browse-click-project.spec.ts`
   pour valider le flow E2E
6. Verifier `npm run size` : le chunk BrowsedProject doit etre
   sous budget

**Signal** :
- P0 : crash quand un projectId inexistant est dans l'URL
- P1 : XSS via projectId dans l'URL (injection dans le DOM)
- P1 : TabView non rendu (regression sur le flow existant)
- P2 : pas de loading state pendant le fetch
- P3 : styling nits

---

## Track E — API schemas + backward compatibility

**Question** : est-ce que le champ `source` de `BrowseEntry` est
backward-compatible (optionnel), et est-ce que les Zod schemas
sont alignes avec le Rust backend ?

**Methodes** :
1. Lire `web/src/api/daemon.ts` : verifier que `source` est
   `z.enum(...).optional()` (pas required)
2. Lire `web/src/api/coordinator.ts` : verifier `getProjectApps()`
3. Lire `crates/nexus-shell-daemon-core/src/browse.rs` :
   verifier que le `BrowseEntry` JSON contient bien le champ
   `source` et que le `#[serde(default)]` est present pour compat
4. Comparer les champs Zod frontend vs Rust backend : tout champ
   manquant ou en trop est un finding
5. Lire les tests `web/src/api/__tests__/daemon.test.ts` : verifier
   les cas avec et sans `source`

**Signal** :
- P0 : Zod parse en strict mode rejette un BrowseEntry sans source
- P1 : schema drift entre Rust et TypeScript (champ present dans un
  seul cote)
- P2 : pas de test pour le cas backward compat (daemon v10 sans source)
- P3 : naming convention inconsistante (camelCase vs snake_case)

---

## Track F — Deploy scripts correctness

**Question** : est-ce que les scripts de deploy sont corrects,
securises, et ne creent pas de regression sur l'existant ?

**Methodes** :
1. Lire `deploy/nginx-nexus.conf` : verifier la config (SPA
   fallback, proxy pass, pas de directory listing)
2. Lire `deploy/deploy-web.sh` : verifier le flow build + upload
3. Lire `deploy/deploy.sh` : verifier le nouveau `--role web`
   (pas de regression sur daemon/coordinator)
4. Lire `deploy/provision.sh` : verifier l'ajout nginx (steps 5/8,
   firewall 80/tcp, sites-enabled symlink, default removed)
5. Diff `deploy/provision.sh` vs `deploy/nginx-nexus.conf` :
   verifier que la config inline dans provision.sh est identique
   au fichier nginx-nexus.conf (sync risk)
6. Verifier `deploy/coordinator.toml.example` : syntaxe TOML valide
7. Lire `deploy/README.md` : verifier que la doc couvre les
   nouvelles etapes

**Signal** :
- P0 : provision.sh casse sur une VPS deja provisionee (idempotence)
- P1 : nginx config inline diverge de nginx-nexus.conf (sync risk)
- P1 : deploy-web.sh `rm -rf /opt/nexus-grid/web/*` est destructif
  sans rollback
- P2 : pas de HTTPS / certbot
- P3 : doc nits

---

## Track G — Scope cuts verification

**Question** : est-ce que le sprint respecte les scope cuts declares
dans le kickoff §5 ?

**Methodes** :
1. `grep -r "blob" web/src/` : aucun upload blob UI ne doit exister
2. `grep -r "SBFB" web/src/ packages/ crates/` : pas de branding
   (scope cut)
3. `grep -r "cross-node\|remote.*fetch\|distant.*fetch" web/src/` :
   le fetch cross-node ne doit pas etre implemente
4. Verifier que BrowsedProject.tsx affiche un placeholder pour les
   projets distants, pas un fetch reel
5. `grep -r "iframe" web/src/` : l'iframe doit etre un skeleton,
   pas un composant fonctionnel avec fetch blob

**Signal** :
- P1 : code qui depasse un scope cut (livraison non planifiee →
  surface non testee)
- P2 : placeholder pas assez clair pour l'utilisateur
- P3 : naming nit dans le placeholder text

---

## Track H — Test coverage quality

**Question** : est-ce que les 40 nouveaux tests sont significatifs
ou ceremoniels ?

**Methodes** :
1. Lire 3 tests Rust au hasard dans publish.rs/browse.rs : verifier
   qu'ils testent un comportement, pas juste la compilation
2. Lire les tests Playwright `browse-click-project.spec.ts` :
   verifier qu'ils assertent un contenu reel (pas juste "page loads")
3. `npm run test:coverage` : verifier que `BrowsedProject.tsx` a
   une couverture significative (branches + lines)
4. Lire `test_daemon_proxy.py` : verifier que les mocks ne masquent
   pas les vrais bugs (mock trop genereux)
5. Comparer le nombre de tests planifies (8+5+12+3 = 28) vs livres
   (19+6+12+3 = 40) : 12 de plus que prevu, verifier que c'est
   du signal et pas du padding

**Signal** :
- P1 : test qui ne teste rien (assert True, mock qui renvoie
  toujours le bon resultat)
- P2 : coverage significativement sous les seuils pour un
  nouveau module
- P3 : test nommage ou structure nits

---

## Track I — SPDX + hygiene

**Question** : est-ce que les nouveaux fichiers source ont les
headers SPDX, et est-ce que l'hygiene generale est maintenue ?

**Methodes** :
1. `bash scripts/check-spdx.sh` : doit couvrir les nouveaux .rs/.ts/.py
2. `cargo fmt --all --check` + `cargo clippy --workspace --all-targets --locked -- -D warnings`
3. `bash web/scripts/scan-en-strings.sh` : pas de strings anglais
   dans le code React cote utilisateur
4. Verifier que les deploy scripts (.sh) ont le header SPDX
5. `npm audit --audit-level=high` : 0 vulnerabilites

**Signal** :
- P0 : fichier source sans SPDX qui est passe entre les mailles
- P1 : string anglaise visible dans l'UI utilisateur
- P2 : warning clippy silenced sans raison
- P3 : formatting nit

---

## 1. Verdict global attendu

- **PASS** : 0 P0, 0 P1 → Sprint 12 Phase A demarre
- **CONDITIONAL PASS** : 1-3 P1 fixables → Sprint 12 Phase A
  bloque tant que les `fix(sprint11): ...` ne sont pas landed
- **FAIL** : >= 1 P0 ou >= 3 P1 → re-conception partielle

## 2. Out of scope pour l'audit

- Les D1-D5 gelees du kickoff : ne pas rebattre les decisions
  (apps web = iframe, gossip publish, default curator, plein ecran,
  VPS EU)
- Les scope cuts declares : ne pas critiquer l'absence de blob UI,
  branding, HTTPS, cross-node rendering
- Les pins de dependances (iroh 0.97, axum 0.7, etc.)
- Le deploiement reel sur le VPS (pas de SSH dans l'audit)

## 3. Livrable final attendu

Fichier `.planning/sprint11_audit_findings.md` avec :
1. Verdict global
2. Une section par track (A-I) avec verdict + findings
3. Table recap des findings tries P0..P3
4. Commits fix si CONDITIONAL PASS
5. P2 a logger en tech debt
6. P3 laisses sans action
