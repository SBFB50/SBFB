# Sprint 12 — Plan d'audit pour Sprint 13 Phase 0

**Sprint audite** : Sprint 12 (rendu universel cross-node)
**Tip a auditer** : `bf3f009` (Phase E, apres planning commit Phase F)
**Auteur du plan** : session Sprint 12 (meme agent que le code — l'auditeur sera une session fraiche)

---

## Mode d'emploi pour la session fraiche

1. Lire dans l'ordre :
   - Memory : `MEMORY.md` → `nexus_grid_pivot.md` → `sprint_audit_gate.md` → `feedback_approach.md`
   - `docs/claude/README.md` (workflow source of truth)
   - Ce fichier (`sprint12_audit_plan.md`)
   - `sprint12_kickoff.md` (Day 0 D1-D7)
   - `sprint12_plan.md` (plan detaille phases A-F)
   - `sprint12_verification.md` (self-report, a challenger)

2. **NE PAS LIRE** `docs/shell/PATTERNS.md` P21-P23 avant d'avoir
   forme une opinion sur chaque track. L'auditeur doit challenger,
   pas ratifier.

3. **Timebox** : 2-3h. Le signal prime sur le volume.

4. **Delivrable** : `.planning/sprint12_audit_findings.md` avec
   verdict PASS / CONDITIONAL PASS / FAIL.

---

## Track A — Securite blob-serve

**Question** : le blob-serve est-il securise contre les archives malveillantes ?

**Methode** :
- Lire `blob_serve.rs` de bout en bout
- Verifier : path traversal (`../`, `\`, absolu), zip bomb (> 100MB), magic bytes
- Grep `unsafe` dans le crate
- Verifier que le CSP `connect-src 'none'` est bien injecte sur TOUTES les reponses
- Verifier que `sandbox="allow-scripts"` est SANS `allow-same-origin` dans `BrowsedProject.tsx`
- Tester manuellement : `GET /blob-serve/{hash}/../../../etc/passwd` — doit etre 400

**Signal** :
- P0 : contournement de path traversal ou zip bomb
- P1 : CSP manquant sur une reponse, allow-same-origin dans l'iframe
- P2 : content-type detection incomplete

## Track B — TabView pre-render fidelite

**Question** : le HTML pre-rendu est-il fidele au React shell ?

**Methode** :
- Comparer `html_render.py` block par block avec `web/src/components/app/tabview/blocks/`
- Verifier que les 12 block kinds sont geres (heading, text, kv, metric, table, badge_list, button, chart_line, chart_bar, empty, section, file_upload)
- Verifier le XSS : `html.escape` sur toutes les valeurs utilisateur
- Verifier les SVG charts : axes, labels, data points coherents
- Executer : `uv run python -c "from nexus_sdk.html_render import render_tabview_to_html; print(render_tabview_to_html({'blocks': [{'kind': 'heading', 'level': 1, 'text': '<script>xss</script>'}]}))"` — le `<script>` doit etre echappe

**Signal** :
- P0 : XSS exploitable (valeur non echappee dans le HTML)
- P1 : block kind non gere qui produit du HTML casse
- P2 : difference visuelle significative avec le React shell
- P3 : nits CSS cosmétiques

## Track C — Deploy endpoint securite

**Question** : `POST /project/deploy` est-il securise ?

**Methode** :
- Lire `api/deploy.py`
- Verifier : validation zip (index.html requis), taille max absente ?, authentification absente ?
- Verifier le flux : upload → validation → store blob → publish
- Chercher des race conditions ou des fuites de fichiers temporaires

**Signal** :
- P0 : execution de code arbitraire via l'upload
- P1 : absence de limite de taille sur l'upload
- P2 : pas d'authentification (acceptable pour loopback-only)

## Track D — Auto-publish integration

**Question** : le flow auto-publish genere-t-il correctement l'archive ?

**Methode** :
- Lire `coordinator.py` `_build_and_store_archive` et `_auto_publish`
- Verifier : chaque tab est pre-rendue, zip avec index.html redirects, archive_hash dans le payload
- Verifier backward compat : un coordinator sans apps ne crashe pas
- Verifier qu'un coordinator private ne publie PAS
- Lancer `uv run pytest packages/nexus-coordinator/tests/test_auto_publish_archive.py -v`

**Signal** :
- P1 : coordinator public sans apps crashe au boot
- P1 : archive_hash absent du payload publish
- P2 : redirects index.html manquants dans le zip

## Track E — Frontend cross-node rendering

**Question** : l'iframe remote fonctionne-t-elle correctement ?

**Methode** :
- Lire `BrowsedProject.tsx` RemoteProjectFrame
- Verifier : iframe sandbox attrs, banner "contenu tiers", fallback placeholder quand pas d'archive
- Verifier `daemon.ts` : BrowseEntrySchema archive_ticket + archive_hash, blobServeUrl(), daemonBaseUrlFromInfo()
- Lancer `npm run test:unit` et verifier que les tests BrowsedProject couvrent les 3 cas (local, remote avec archive, remote sans archive)
- Verifier la couverture : `BrowsedProject.tsx` est dans vitest coverage.include (T34)

**Signal** :
- P1 : allow-same-origin dans l'iframe
- P1 : blobServeUrl construit un mauvais path
- P2 : banner "contenu tiers" invisible ou mal positionnee

## Track F — Deploy infrastructure

**Question** : les scripts deploy sont-ils coherents et securises ?

**Methode** :
- Comparer `provision.sh` et `nginx-nexus.conf` — T32 : provision.sh doit utiliser cp pas heredoc
- Verifier `X-Forwarded-Proto` present sur les 3 blocs nginx (/api/, /daemon/, /blob-serve/)
- Verifier `provision-tls.sh` : certbot non-interactive, redirect HTTP→HTTPS
- Verifier que `/blob-serve/` proxy est present dans nginx

**Signal** :
- P1 : provision.sh toujours en heredoc (T32 pas ferme)
- P2 : X-Forwarded-Proto manquant sur un bloc

## Track G — Tech debt T28-T36

**Question** : les 9 items sont-ils reellement fermes ?

**Methode** :
- `grep CLOSED docs/shell/PATTERNS.md | wc -l` — doit etre >= 9
- Pour chaque item, verifier la correction :
  - T28 : `grep InvalidNodeId crates/nexus-shell-daemon-core/src/publish.rs`
  - T29 : `grep truncated crates/nexus-shell-daemon-core/src/publish.rs`
  - T30 : `grep 500 packages/nexus-coordinator/tests/test_daemon_proxy.py`
  - T31 : `grep retain crates/nexus-shell-daemon-core/src/config.rs`
  - T32 : `grep -c heredoc deploy/provision.sh` (doit etre 0)
  - T33 : `test -f deploy/provision-tls.sh`
  - T34 : `grep BrowsedProject web/vitest.config.ts`
  - T35 : `grep aggregate_flattens crates/nexus-shell-daemon-core/src/browse.rs`
  - T36 : `grep X-Forwarded-Proto deploy/nginx-nexus.conf | wc -l` (>= 3)

**Signal** :
- P1 : un item marque CLOSED qui n'est pas reellement corrige
- P2 : correction partielle (test ecrit mais code pas change)

## Track H — Tests et couverture

**Question** : les tests sont-ils suffisants et non-creux ?

**Methode** :
- `cargo test --workspace --locked` : >= 362
- `uv run pytest packages/nexus-sdk/tests/ -q` : >= 182
- `uv run pytest packages/nexus-coordinator/tests/ -q` : >= 95+1
- `npm run test:unit` : >= 180
- `npx playwright test` : >= 30
- Verifier que `test_html_render.py` couvre les 12 block kinds
- Verifier que `test_deploy.py` couvre valid zip, invalid zip, missing index
- Lire les nouveaux tests blob-serve dans `http.rs` — verifient-ils reellement les headers CSP ?

**Signal** :
- P1 : un test qui passe toujours quel que soit le code (creux)
- P2 : block kind non teste dans html_render
- P3 : couverture < 80% sur un nouveau module

## Track I — BrowseEntry backward compat

**Question** : les changements au BrowseEntry sont-ils backward-compatibles ?

**Methode** :
- Verifier serde : `archive_ticket` et `archive_hash` ont `#[serde(default, skip_serializing_if = "Option::is_none")]`
- Verifier Zod : `archive_ticket` et `archive_hash` sont `.optional()`
- Verifier que le test de backward compat (no source, no archive) passe toujours
- Simuler un daemon v1 (Sprint 11) qui envoie un BrowseEntry sans ces champs — le shell ne doit pas crasher

**Signal** :
- P0 : un champ non-optional casse le parsing des anciens daemons
- P1 : Zod strict() rejette les entries sans les nouveaux champs

---

## Verdict global attendu

- **PASS** : 0 P0, 0 P1 → Sprint 13 Phase A demarre direct
- **CONDITIONAL PASS** : 1-3 P1 fixables → Sprint 13 bloque tant que les `fix(sprint12): ...` ne sont pas landed
- **FAIL** : >= 1 P0 ou >= 3 P1 → re-conception partielle

## Out of scope pour l'audit

- Les decisions Day 0 D1-D7 ne sont pas rebattables
- Les scope cuts (branding, VPS, runtime templates) ne sont pas des findings
- Le pin iroh 0.97 / iroh-blobs 0.99 n'est pas remis en question
- L'absence de Playwright e2e remote iframe n'est pas un P1 (necessite 2 daemons)

## Livrable final attendu

`.planning/sprint12_audit_findings.md` avec :
1. Verdict global
2. Une section par track (A-I) avec findings
3. Table recap P0 → P3 triee par severite
4. Commits fix attendus si CONDITIONAL PASS
5. P2 a logger en tech debt
6. Notes on audit completeness
