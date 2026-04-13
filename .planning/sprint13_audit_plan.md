# Sprint 13 — Audit plan pour Sprint 14 Phase 0

**Ecrit** : 2026-04-13
**Tip a auditer** : `72cf5ad` (Phase D) + commit Phase E docs
**Commit stack** : 5 commits Phase A-D + 1 planning + 1 docs

---

## Mode d'emploi pour la session fraiche

1. Lire dans l'ordre : memory → git log → kickoff → plan →
   verification → cet audit plan
2. **NE PAS lire** `docs/shell/PATTERNS.md` avant d'avoir forme
   une opinion track par track
3. Timebox suggere : 2-3h
4. Delivrable : `.planning/sprint13_audit_findings.md`

---

## Track A — Bridge securite

**Question** : le bridge postMessage peut-il etre exploite par
une app malveillante dans l'iframe ?

**Methodes** :
- Lire `web/src/bridge/protocol.ts` : le schema Zod est-il
  suffisamment restrictif ? Un champ `payload` de type
  `z.record(z.unknown())` permet-il l'injection ?
- Lire `web/src/bridge/useBridge.ts` : la validation de source
  (`event.source === iframe.contentWindow`) est-elle contournable ?
- Verifier que `sbfb-bridge.js` n'expose pas de methode
  permettant d'appeler des endpoints non-whitelistes
- Verifier qu'un postMessage depuis une autre fenetre (pas
  l'iframe) est bien ignore
- Tester : ouvrir la console dans le navigateur et envoyer un
  `window.postMessage(...)` — le bridge doit l'ignorer

**Signal** :
- P0 : methode non-whitelist accessible via le bridge
- P1 : source validation contournable
- P2 : payload trop permissif mais limite aux 3 methodes

---

## Track B — Open source enforcement

**Question** : la validation `repo_url` est-elle contournable ?

**Methodes** :
- Lire `deploy.py` : la validation se fait-elle bien cote
  coordinateur, pas seulement cote client ?
- Tester : `POST /project/deploy` avec visibility=public et
  pas de `repo_url` → doit retourner 400
- Verifier que le champ `repo_url` est bien propage dans
  l'announcement gossip (publish.rs → http.rs → gossip)
- Verifier backward compat : un BrowseEntry v2 sans repo_url
  parse correctement (Zod `.optional()` + serde `#[serde(default)]`)
- Verifier que le lien repo est bien visible dans le shell
  (Browse cards + BrowsedProject top bar)

**Signal** :
- P0 : validation contournable (public sans repo_url accepte)
- P1 : repo_url present en Rust mais pas propage au frontend
- P2 : validation trop permissive (URL vide acceptee)

---

## Track C — Launcher robustesse

**Question** : le launcher gere-t-il les cas d'erreur correctement ?

**Methodes** :
- Lire `crates/nexus-launcher/src/main.rs`
- Cas 1 : daemon deja running → le launcher doit l'utiliser, pas
  en spawner un second
- Cas 2 : daemon introuvable (pas dans PATH) → message d'erreur
  clair, pas de panic
- Cas 3 : running.json existe mais daemon mort (stale file) →
  le launcher tente de spawner et echoue proprement
- Cas 4 : Ctrl+C → le child process est bien arrete
- Verifier que le binary se compile : `cargo build -p nexus-launcher`
- Verifier `--help` : `cargo run -p nexus-launcher -- --help`

**Signal** :
- P0 : panic en production sur un cas d'erreur normal
- P1 : le launcher spawne un second daemon quand un tourne deja
- P2 : message d'erreur peu clair ou manquant

---

## Track D — UI glassmorphism accessibilite

**Question** : le redesign glassmorphism est-il accessible ?

**Methodes** :
- Inspecter le contraste des textes `text-white/40`, `text-white/50`
  contre `bg-[#0a0a0f]` → les ratios WCAG AA requierent 4.5:1
  pour le texte normal
- Verifier que tous les boutons ont un `aria-label` ou un texte
  visible
- Verifier que les liens repo ont `rel="noopener noreferrer"`
- Verifier que la navigation au clavier fonctionne (tab entre
  les elements, Enter pour activer)
- `scan-en-strings.sh` : toutes les strings utilisateur en francais

**Signal** :
- P1 : texte important illisible (ratio < 3:1)
- P2 : texte secondaire avec faible contraste mais lisible
- P3 : nit d'accessibilite (pas de skip link, etc.)

---

## Track E — Backward compat BrowseEntry v3

**Question** : les changements de schema cassent-ils les daemons
existants ?

**Methodes** :
- Verifier que `ProjectAnnouncement::from_gossip_bytes` accepte
  v1, v2, ET v3 (pas seulement v3)
- Verifier que le Zod `BrowseEntrySchema` a `repo_url` comme
  `.optional()` (pas required)
- Verifier que `serde(default)` est present sur les nouveaux
  champs Rust
- Grepper pour `skip_serializing_if = "Option::is_none"` sur
  les champs optionnels (evite de polluer le JSON pour les vieux
  clients)

**Signal** :
- P0 : un daemon v2 ne peut plus parser les announcements v3
- P1 : champ required dans le Zod schema qui casse le frontend
  avec des entries anciennes
- P2 : champ serialise meme quand None (verbeux mais pas cassant)

---

## Track F — Tests et couverture

**Question** : les nouveaux tests couvrent-ils les cas critiques ?

**Methodes** :
- Lister les tests ajoutes par Sprint 13 (`git diff --stat
  53a9e32..HEAD -- "*.test.*" "*.rs"`)
- Verifier qu'il y a un test pour chaque nouveau endpoint /
  methode bridge
- Verifier que les tests Python deploy couvrent les 3 cas
  (public sans repo → 400, public avec → 200, prive sans → 200)
- Verifier que les tests Vitest bridge couvrent la validation
  de source et le rejet de messages malformes
- Compter les tests totaux vs le plan (plan visait 369 Rust,
  99+1 coord, 187 Vitest)

**Signal** :
- P1 : cas critique non teste (ex: validation source bridge)
- P2 : nombre de tests inferieur au plan
- P3 : test present mais assertion faible

---

## Track G — Tech debt T37-T40 reellement fermes

**Question** : les 4 items tech debt sont-ils vraiment fixes ?

**Methodes** :
- T37 : `grep blob_serve_csp_middleware http.rs` — le middleware
  existe et est monte sur les routes blob-serve. Tester avec un
  404 blob-serve et verifier les headers CSP dans la reponse
  (test `blob_serve_error_responses_have_csp`)
- T38 : verifier les constantes SVG dans `html_render.py` —
  H=120, PAD_L=32, PAD_R=16, PAD_T=16, PAD_B=16 (line) / 24 (bar)
- T39 : `pytest -k test_render_file_upload` — le test existe
  et passe
- T40 : `grep X-Real-IP deploy/nginx-nexus.conf` — present dans
  les 3 location blocks

**Signal** :
- P1 : un item marque CLOSED mais pas reellement fixe
- P2 : fixe mais le test est trop faible pour le prouver
- P3 : fixe correctement, nit mineur

---

## Verdict global attendu

- **PASS** : 0 P0, 0 P1 → Sprint 14 Phase A demarre direct
- **CONDITIONAL PASS** : 1-3 P1 fixables → Sprint 14 bloque
  tant que les `fix(sprint13): ...` ne sont pas landed
- **FAIL** : >= 1 P0 ou >= 3 P1 → re-conception partielle

---

## Out of scope pour l'audit

- Les D1-D6 gelees (cf. kickoff §4) — ne pas les rebattre
- Les scope cuts (CPU watchdog, branding, etc.) — decisions de
  priorisation, pas des bugs
- Les pins de deps (iroh 0.97, axum 0.7, etc.)
- Le contenu des design docs dans `docs/apps/`
- Les fichiers non trackes (cc.json, site/, docs/apps/)

---

## Livrable final attendu

```
.planning/sprint13_audit_findings.md
```

Format : verdict global + une section par track avec findings
ventiles P0 / P1 / P2 / P3 + commits fix pour les P0/P1.
