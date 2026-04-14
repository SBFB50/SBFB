# Sprint 14 — Audit plan pour Sprint 15 Phase 0

**Ecrit** : 2026-04-14
**Tip a auditer** : `3dc8ff2` (Phase D) + commit Phase E docs
**Commit stack** : 4 commits Phase A-D + 1 docs

---

## Mode d'emploi pour la session fraiche

1. Lire dans l'ordre : memory → git log → kickoff → plan →
   verification → cet audit plan
2. **NE PAS lire** `docs/shell/PATTERNS.md` avant d'avoir forme
   une opinion track par track
3. Timebox suggere : 2-3h
4. Delivrable : `.planning/sprint14_audit_findings.md`

---

## Track A — Securite du clone git

**Question** : le subprocess `git clone` est-il securise contre
les attaques connues (path traversal, symlinks, taille excessive,
timeout bypass) ?

**Methodes** :
- Lire `deploy.py` `_clone_repo()` : le timeout asyncio est-il
  robuste ? Un repo qui repond lentement mais reste sous timeout
  peut-il bloquer le coordinateur ?
- Verifier `_zip_directory()` : les chemins `..` sont-ils rejetes ?
  Les symlinks sont-ils exclus ? `.git/` est bien exclu ?
- Verifier `_dir_size()` : le check de taille est-il fait APRES
  le clone (pas avant) — un repo malveillant pourrait-il remplir
  le disque avant le check ?
- Tester : creer un repo avec un fichier `../../../etc/passwd` dans
  le nom — le zip doit l'exclure
- Verifier que `--depth 1 --single-branch` sont bien passes

**Signal** :
- P0 : path traversal non bloque (fichier hors zip)
- P1 : symlink non exclu (potential file read)
- P2 : timeout non robuste ou taille checkee trop tard
- P3 : .gitmodules pas explicitement ignore

---

## Track B — Provenance signing correctness

**Question** : la signature provenance est-elle correcte et
non-rejouable ?

**Methodes** :
- Lire `provenance.py` : la signature utilise-t-elle bien
  `canonical_bytes` avec domain `nexus-provenance-v1\x00` ?
- Verifier que le champ `signature` est exclu du payload
  signe (sinon il se signe lui-meme — circulaire)
- Verifier que `json.dumps(sort_keys=True, separators=(',', ':'))`
  est equivalent a JCS pour le schema plat (pas de floats)
- Tester : modifier un champ apres signature → verify echoue
- Tester : signer avec une cle, verifier avec une autre → echoue
- Verifier cross-language : si un noeud Rust voulait verifier,
  le domain tag `b"nexus-provenance-v1"` matche-t-il
  `DOMAIN_PROVENANCE_V1` dans `canonical.rs` ?

**Signal** :
- P0 : signature rejouable (meme signature valide pour deux payloads)
- P1 : domain tag mismatch Rust ↔ Python
- P2 : canonical bytes pas strictement JCS (mais fonctionne pour
  le schema actuel)
- P3 : signature hex au lieu de base64 (convention projet)

---

## Track C — SBFB.json verification (Keyoxide pattern)

**Question** : la verification de propriete via SBFB.json est-elle
robuste ?

**Methodes** :
- Lire `_read_sbfb_json()` : que se passe-t-il si le fichier
  contient des champs supplementaires (injection) ?
- Verifier que `node_id` est compare strictement (casse, longueur)
- Verifier que `SBFB.json` est lu depuis le clone (pas depuis
  une URL pre-check — pour eviter TOCTOU)
- Tester : SBFB.json avec un node_id de longueur 63 → rejet ?
- Tester : SBFB.json sans champ `project_name` → rejet ou accepte ?

**Signal** :
- P0 : node_id compare de maniere laxe (prefix match au lieu de strict)
- P1 : SBFB.json lu depuis URL raw avant clone (TOCTOU)
- P2 : pas de validation de format node_id (hex, longueur)
- P3 : champs supplementaires acceptes silencieusement

---

## Track D — Backward compat PA v4

**Question** : un daemon v3 (ancien) peut-il parser une annonce v4 ?
Un daemon v4 peut-il parser une annonce v3 ?

**Methodes** :
- `cargo test -p nexus-shell-daemon-core` : les tests v3/v4 passent
- Verifier `serde(default)` sur `provenance_hash` dans publish.rs
- Verifier `skip_serializing_if = "Option::is_none"` (v4 sans
  provenance ne pollue pas le JSON)
- Verifier Zod `.optional()` dans daemon.ts
- Simuler : envoyer un JSON v3 (sans provenance_hash) →
  deserialization OK avec None

**Signal** :
- P0 : un v3 daemon crash sur une annonce v4
- P1 : un v4 daemon rejette une annonce v3
- P2 : champ serialise meme quand None (verbeux)

---

## Track E — Badge UI conditionnel

**Question** : le badge "Verifie" est-il correctement conditionnel
et accessible ?

**Methodes** :
- Verifier Browse.tsx : badge present quand `provenance_hash` defini,
  absent sinon
- Verifier BrowsedProject.tsx : idem + auto-hide top bar
- Verifier le test data-testid="verified-badge"
- Verifier l'accessibilite : le texte "Verifie" est-il en francais ?
  Le contraste vert sur fond sombre est-il suffisant (WCAG AA) ?
- `scan-en-strings.sh` doit etre clean

**Signal** :
- P1 : badge affiche sans provenance (faux positif de securite)
- P2 : badge pas assez visible ou mal contraste
- P3 : nit d'accessibilite (pas d'aria-label)

---

## Track F — Deploy public redirect

**Question** : l'ancien `POST /project/deploy` bloque-t-il bien
les apps publiques ?

**Methodes** :
- Lire `deploy.py` : le check visibility se fait AVANT la lecture
  du body (pas de gaspillage de bande passante)
- Tester : `POST /project/deploy` avec visibility=public → 400
  avec message mentionnant deploy-from-repo
- Tester : `POST /project/deploy` avec visibility=private → 200
  (inchange)
- Verifier que le message d'erreur est actionnable (contient le
  nom du bon endpoint)

**Signal** :
- P0 : public peut toujours uploader un zip (bypass)
- P1 : message d'erreur pas clair (l'utilisateur ne sait pas quoi faire)
- P2 : check fait apres lecture du body (gaspillage)

---

## Track G — Tests et couverture

**Question** : les nouveaux tests couvrent-ils les cas critiques ?

**Methodes** :
- Compter les tests par module (forge, provenance, deploy-from-repo)
- Verifier qu'il y a un test pour chaque cas d'erreur du endpoint
  (missing SBFB.json, wrong node_id, missing index.html, private
  rejected, repo not public)
- Verifier que le test provenance couvre tampered hash + wrong key
- Verifier le test provenance-in-zip (le zip envoye au daemon
  contient bien provenance.json)
- Compter les tests totaux vs le plan

**Signal** :
- P1 : cas critique non teste (ex: path traversal)
- P2 : nombre de tests inferieur au plan
- P3 : test present mais assertion faible

---

## Track H — P2 tech debt resolution

**Question** : les P2 du Sprint 13 audit sont-ils correctement fermes ?

**Methodes** :
- T42 : `grep "text-white/30"` dans BrowsedProject + ProjectDetail →
  0 instances sur les lignes 11px identifiees
- T43 : `grep "_SVG_PAD_R = 32"` dans html_render.py → present
- T41 : grep T41 dans PATTERNS.md → marque SUPERSEDED
- Verifier que les Vitest BrowsedProject passent toujours

**Signal** :
- P1 : un P2 marque CLOSED mais pas reellement fixe
- P2 : fixe mais sans test
- P3 : log dans PATTERNS.md incomplet

---

## Verdict global attendu

- **PASS** : 0 P0, 0 P1 → Sprint 15 Phase A demarre direct
- **CONDITIONAL PASS** : 1-3 P1 fixables → Sprint 15 bloque
  tant que les `fix(sprint14): ...` ne sont pas landed
- **FAIL** : >= 1 P0 ou >= 3 P1 → re-conception partielle

---

## Out of scope pour l'audit

- Les D1-D5 gelees (cf. kickoff §4) — ne pas les rebattre
- Les scope cuts (CPU watchdog, templates, etc.)
- Les pins de deps (iroh 0.97, axum 0.7, etc.)
- Le test SDK flaky Windows (pre-existant)
- Les fichiers non trackes (cc.json, site/, docs/apps/)

---

## Livrable final attendu

```
.planning/sprint14_audit_findings.md
```

Format : verdict global + une section par track avec findings
ventiles P0 / P1 / P2 / P3 + commits fix pour les P0/P1.
