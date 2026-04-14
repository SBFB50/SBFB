# Sprint 14 — Audit findings (Sprint 15 Phase 0 gate)

**Auditeur** : Claude Code session fraiche (Sprint 15 Phase 0)
**Date** : 2026-04-14
**Tip audite** : `f3f49be` (docs Sprint 14 verification + audit plan)
**Commit stack Sprint 14** :
- `407af60` Phase A — deploy from repo with Keyoxide + SLSA L1 provenance
- `328ef15` Phase B — ProjectAnnouncement v4 with provenance hash
- `ae7d6ea` Phase C — verified badge + P2 tech debt Sprint 13
- `3dc8ff2` Phase D — deploy public redirect to deploy-from-repo
- `f3f49be` Phase E (docs) — verification + audit plan for Sprint 15

**Timebox observe** : ~1h30.

---

## Verdict global : **CONDITIONAL PASS**

- **P0** : 0
- **P1** : 1 (A-1, feature cassee)
- **P2** : 8 (loggees en tech debt)
- **P3** : 7 (laisses sans action)

Sprint 15 Phase A est bloque tant que A-1 n'est pas resolu en commit
`fix(sprint14): ...` atterri sur master.

---

## Track A — Securite du clone git — **CONCERN**

Le subprocess git clone est globalement defendu en profondeur
(--depth 1 --single-branch, timeout 30s double, tmpdir cleanup en
finally, pas de --recursive). Les protections path traversal et
symlinks sont implementees dans `_zip_directory`. Mais la feature
`commit_sha` promise par le kickoff D1 est cassee.

### A-1 (P1) — `commit_sha` passe a `git clone --branch` ne supporte pas les SHA

**Localisation** : `packages/nexus-coordinator/src/nexus_coordinator/api/deploy.py:305-308`

```python
cmd = ["git", "clone", "--depth", "1", "--single-branch"]
if ref:
    cmd += ["--branch", ref]
```

`git clone --branch <ref>` n'accepte QUE des noms de branches ou de
tags, pas des SHA arbitraires. Verifie localement :

```
$ git clone --depth 1 --single-branch --branch abc123def456 \
    https://github.com/octocat/Hello-World.git
Cloning into 'Hello-World'...
fatal: Remote branch abc123def456 not found in upstream origin
```

Or :
- Le kickoff D1 promet `{repo_url, commit_sha?}` — champ documente
  comme SHA
- Le body Pydantic nomme le champ `commit_sha: str | None`
- Le `test_deploy_from_repo_happy_path` ne couvre pas ce cas (il
  n'envoie pas de commit_sha, et `_clone_repo` est mocke)

**Consequence** : tout client qui envoie un SHA reel dans
`commit_sha` recoit une 400 "fatal: Remote branch ... not found".
Le happy path test passe parce que le mock ne verifie rien, et le
bug est donc latent.

**Severite P1** :
- Pas un probleme de securite (l'erreur est propre, pas de crash)
- Mais feature documentee et exposee dans l'API, ne fonctionne pas
- Aucun test ne le detecte
- Bloque Sprint 15 Phase A jusqu'a resolution

**Fix propose** : deux options, au choix utilisateur.

**Option 1 (minimale, preferee)** : renommer `commit_sha` en `ref`
dans le body, clarifier que c'est un branch/tag (pas un SHA). Le
champ reste optionnel. Endpoint est Sprint 14 donc pas d'utilisateur
externe. Modifier en consequence le kickoff (retroactivement via
note) et les tests.

**Option 2 (complete)** : implementer le SHA pinning correctement.
`git clone --depth 1` sans --branch, puis
`git fetch --depth 1 origin <sha> && git checkout FETCH_HEAD`. Plus
de code, plus de tests, mais honore le nom du champ.

Recommendation auditeur : Option 1. Le SHA pinning n'est pas
testable facilement (necessite un repo git local), et la valeur
ajoutee est faible vs cout (la verification cryptographique est deja
dans le provenance — le SHA pinning est de l'over-engineering pour
un MVP).

**Fix attendu** : `fix(sprint14): rename commit_sha to ref in
deploy-from-repo body`.

---

## Track B — Provenance signing correctness — **PASS** (avec nits)

La signature provenance est correcte et non-rejouable :
- Domain `nexus-provenance-v1\x00` match `DOMAIN_PROVENANCE_V1`
  cote Rust (`crates/nexus-core-rs/src/canonical.rs:104`)
- Le champ `signature` est bien exclu du payload signe
  (`_signable_payload` construit explicitement le dict sans)
- Tests `test_rejects_tampered_hash` + `test_rejects_wrong_key` +
  `test_rejects_garbage_json` couvrent la non-rejouabilite

### B-1 (P2) — `provenance.py` utilise `json.dumps` au lieu de `jcs` PyPI

**Localisation** :
`packages/nexus-coordinator/src/nexus_coordinator/provenance.py:181-195`

Le commentaire admet la divergence :
```python
# We use json.dumps(sort_keys=True, separators=(',', ':'))
# which is equivalent to JCS for our flat string/int schema.
```

C'est vrai pour le schema v1 actuel (tous les champs sont ASCII :
URL, hex hash, ISO timestamp). Mais la convention projet est
`serde_jcs` cote Rust et `jcs` PyPI cote Python (cf. canonical.rs
§"Python side: `jcs` PyPI package").

**Impact** : aucun bug maintenant. Mais si un futur schema ajoute un
champ texte non-ASCII (ex: `project_description: str`),
`json.dumps(ensure_ascii=False)` peut diverger de JCS pour les
escapes Unicode.

**Fix suggere** : switcher a `import jcs; jcs.canonicalize(payload)`.
Cout faible (1 import, 1 remplacement). Logger en tech debt si pas
fait maintenant.

### B-2 (P2) — `verify_provenance` ne verifie pas `schema_version`

**Localisation** : `provenance.py:104-138`

```python
payload = _signable_payload(
    schema_version=data["schema_version"],
    ...
)
```

`verify_provenance` accepte n'importe quelle valeur de
`schema_version` (int arbitraire). Si un v2 est introduit avec des
champs differents, le verifier v1 continuera a valider silencieusement
les payloads v2 si les field names communs suffisent. Pas un replay
cross-domain (le domain tag protege), mais un cross-version trap.

**Fix suggere** : ajouter en tete de `verify_provenance` :

```python
if data.get("schema_version") != PROVENANCE_SCHEMA_VERSION:
    return False
```

### B-3 (P3) — signature en hex, pas base64

**Localisation** : `provenance.py:91`, `signature_hex = sig_bytes.hex()`

Convention projet ailleurs : le kudos ledger et les curator lists
serialisent les signatures en hex aussi (cf. canonical.rs). Donc
coherent — **pas un finding**. Laisse comme P3 note de confirmation.

---

## Track C — SBFB.json verification — **PASS**

Verification de propriete via SBFB.json est robuste :
- `_read_sbfb_json(clone_dir)` lit uniquement depuis le clone
  post-clone — **pas de TOCTOU** (le audit plan Track C listait un
  risque TOCTOU URL→clone, absent ici)
- Comparaison `sbfb["node_id"] != daemon_state.node_id` est stricte
  (bytes-for-bytes, case-sensitive) — un SHA de longueur 63 ou en
  upper-case serait rejete
- Le test `test_deploy_from_repo_wrong_node_id` couvre le cas

### C-1 (P3) — pas de validation format node_id dans SBFB.json

`_read_sbfb_json` verifie `"node_id" not in data` mais pas la
longueur (64 hex chars) ni le format. En pratique, la comparaison
stricte avec `daemon_state.node_id` (qui est garanti bien formate)
donne la validation de facto : un SBFB.json avec `node_id: "abc"`
sera rejete par le comparaison.

**Action** : aucune. La comparaison stricte est suffisante.

### C-2 (P3) — champs supplementaires SBFB.json acceptes silencieusement

`_read_sbfb_json` appelle `json.load` sans validation de schema. Un
SBFB.json avec des champs futurs (ex: `required_version: "2.0"`) est
accepte comme s'ils n'existaient pas. C'est le comportement Python
standard — **pas un bug**, mais a noter si jamais le schema SBFB
grows des champs semantiques.

**Action** : aucune maintenant. Si SBFB.json grows, ajouter un
pydantic model.

---

## Track D — Backward compat PA v4 — **PASS**

Les tests unitaires prouvent la compatibilite descendante :
- `publish.rs::v3_announcement_parses_without_provenance_hash` ✓
- `publish.rs::v4_announcement_without_provenance_hash_omits_field` ✓
- `publish.rs::v4_announcement_with_provenance_hash_round_trips` ✓
- `publish.rs::v2_announcement_parses_without_repo_url` ✓ (regression)
- `publish.rs::v1_announcement_parses_without_archive_ticket` ✓ (regression)
- Zod daemon.ts:207 `provenance_hash: z.string().optional()` ✓
- 113 tests nexus-shell-daemon-core verts en local

### D-1 (P2) — bump version 4 casse forward-compat pour un champ purement additif

**Localisation** : `crates/nexus-shell-daemon-core/src/publish.rs:131-136`

```rust
if ann.v == 0 || ann.v > PROJECT_ANNOUNCEMENT_VERSION {
    return Err(ProjectAnnouncementError::Version { got: ann.v, expected: ... });
}
```

Un v3 daemon (PROJECT_ANNOUNCEMENT_VERSION=3) qui recoit un v4
message le rejette — meme si le seul delta v3→v4 est un champ
optionnel ignorable. Pattern sub-optimal pour des additions non
breaking.

Design call explicite du kickoff D3 ("bump version a 4") — **hors
scope de re-debattre** pour cet audit. Mais a noter : si PA v5+
n'ajoute que des champs optionnels, envisager de ne plus bumper la
version et utiliser serde's default behavior pour ignorer les
unknowns. Sinon chaque bump fragmente le reseau temporairement.

**Action** : logger en tech debt dans PATTERNS.md, pas de fix.

### D-2 (P3) — doc commentaire obsolete

**Localisation** : `publish.rs:30`

```rust
/// Wire format version. Always 1 for Sprint 11.
pub v: u32,
```

Ce commentaire date de v1. La version est maintenant 4. Nit.

**Action** : corriger en tech debt, ou ignorer (laisser le comment
juste sur PROJECT_ANNOUNCEMENT_VERSION qui est a jour).

---

## Track E — Badge UI conditionnel — **PASS**

- Badge conditionnel au champ `entry.provenance_hash` dans
  `Browse.tsx:250-258` et `BrowsedProject.tsx:259-267` ✓
- Data-testid `verified-badge` ✓
- Tests Vitest : "renders verified badge when provenance_hash is
  present" + "does not render ... when absent" — les deux passent ✓
- Texte "Verifie" sans accent = convention projet (cf. Network.tsx
  "verifie", ProjectDetail.tsx "verifie" — accents systematiquement
  absents du code utilisateur)
- Icone `ShieldCheck` de lucide-react dans les deux endroits
- scan-en-strings.sh : pas verifie explicitement dans cet audit
  mais code source ne contient pas de strings anglaises visibles

### E-1 (P3) — pas d'aria-label sur le badge Verifie

Le span du badge contient uniquement "Verifie" + icone. Un
aria-label explicite "Application verifiee depuis le repo source"
ameliorerait l'accessibilite pour screen readers. Mais le texte
visible est lu correctement — pas un bloquer.

**Action** : aucune. Nit.

### E-2 (P3) — badge Verifie et badge P2P partagent le meme style emerald

**Localisation** : `Browse.tsx:245-258`

```tsx
{entry.archive_hash && (
  <span className="... bg-emerald-500/15 ... text-emerald-400">P2P</span>
)}
{entry.provenance_hash && (
  <span className="... bg-emerald-500/15 ... text-emerald-400" data-testid="verified-badge">
    <ShieldCheck /> Verifie
  </span>
)}
```

Deux badges verts cote a cote (P2P + Verifie) peuvent paraitre
redondants. Le badge Verifie pourrait etre une couleur plus
distinctive (cyan ? bleu ?) pour amplifier son poids semantique
("garantie cryptographique" vs "disponible P2P").

**Action** : nit UX, ignore sauf si FlowUP veut polish.

---

## Track F — Deploy public redirect — **PASS**

- Check `coord.config.network.visibility == "public"` fait en tete
  de `deploy_project` (ligne 147) — AVANT `await archive.read()`
  donc le body n'est pas lu en memoire
- Message d'erreur contient `deploy-from-repo` (actionnable)
- Test `test_deploy_public_without_repo_url_rejected` et
  `test_deploy_public_with_repo_url_also_rejected` couvrent le cas
- Test `test_deploy_private_without_repo_url_accepted` verifie la
  non-regression private

### F-1 (P3) — le body multipart est quand meme streame par le client

Note technique : FastAPI parse les headers multipart avant d'entrer
dans la fonction route. Le check visibility est avant
`await archive.read()` donc le contenu n'est pas buffere cote
serveur. Mais le client a deja pousse les bytes sur le socket.
Gaspillage bande passante cote client uniquement.

Fix theorique : check visibility via middleware avant multipart
parsing. Cout : haut. Benefice : nul. **Ignorer.**

---

## Track G — Tests et couverture — **CONCERN**

### Couverture par module

| Module | Tests | Notes |
|---|---|---|
| `forge.py` | 15 tests (TestDetectForge + TestRawFileUrl + TestNormalizeCloneUrl) | Couvre GitHub, GitLab, Codeberg, Gitea, unknown |
| `provenance.py` | 7 tests (generate + verify + blake3) | Tampered hash + wrong key + garbage JSON |
| `deploy.py` endpoint | 7 tests deploy-from-repo + 5 deploy upload | Happy path + 5 erreurs + public/private + oversized |
| Rust `publish.rs` PA v4 | 3 tests v4 + 2 tests v3 regression | Round-trip + backward + field omission |
| Vitest badge | 2 tests (present/absent) | BrowsedProject.test.tsx:379-416 |

Total : 34 tests nouveaux + ajustements. Dans le tableau du kickoff
§9 : estimation 260 LOC de tests. Livre ~450 LOC. **Au-dessus du
plan** en volume.

### G-1 (P2) — protections D4 clone sans tests dedies

Le D4 kickoff liste 7 protections clone. Aucune de ces protections
n'a de test end-to-end qui verifie son activation :

| Protection | Teste ? |
|---|---|
| `--depth 1 --single-branch` | Implicit (happy path) |
| Taille max 500 MB | **Non** — aucun test `test_deploy_oversized_repo` |
| Timeout 30s | **Non** — aucun test `test_deploy_clone_timeout` |
| Pas de .git/ | ✓ `test_deploy_from_repo_provenance_in_zip` (assert `not any(n.startswith(".git/"))`) |
| Path traversal rejet | **Non** — aucun test de repo avec `../` dans un path |
| Symlink exclusion | **Non** — aucun test de repo avec symlinks |
| Pas de submodules | Implicit (pas de --recursive) |

4 des 7 protections sont **non testees**. Le code les implemente
(visible dans `_zip_directory`), mais aucun test ne verifie que la
protection fonctionne. Un regression silencieuse (ex: suppression du
check `if ".." in arcname`) ne serait pas detectee.

**Action** : logger en tech debt. Ajouter des tests au Sprint 15
ou 16. Ne pas bloquer Sprint 15 sur ces tests (le code est
correct, seule la validation automatique manque).

### G-2 (P2) — clone reel jamais teste

`_clone_repo` est systematiquement mocke par `_make_mock_clone` qui
fait un simple `shutil.copytree`. Aucun test ne reellement appelle
`git clone` subprocess. Consequence : le bug A-1 (commit_sha →
--branch) etait invisible aux tests.

**Action** : ajouter un test `tests/integration/test_deploy_real_clone.py`
qui clone un mini repo Git local (creer avec `git init` + commit
dans tmp_path), marque `@pytest.mark.integration` pour skip par
defaut, exercice le vrai subprocess. Sprint 15 ou 16. Pas un
blocker.

---

## Track H — P2 tech debt Sprint 13 — **PASS**

Les 3 items sont correctement fermes :

### T41 — repo_url XSS via javascript: protocol — SUPERSEDED ✓

`docs/shell/PATTERNS.md:1503-1511` : marque SUPERSEDED par le deploy
verifie. Rationale claire : le `repo_url` n'est plus un champ trust-
based, il est verifie par SBFB.json + provenance. Pas de code fix
mais decision documentee.

### T42 — text-white/30 contrast — CLOSED ✓

`docs/shell/PATTERNS.md:1513-1519`. Grep confirmed :
- `BrowsedProject.tsx:282` utilise maintenant `text-white/40` ✓
- `ProjectDetail.tsx:131` utilise `text-white/40` ✓
- Aucune instance de `text-white/30` sur un texte 11px (les
  instances residuelles dans AppShell.tsx:136/242 sont des icons,
  pas du texte 11px — hors scope T42)

### T43 — _SVG_PAD_R = 16 → 32 — CLOSED ✓

`docs/shell/PATTERNS.md:1521-1524`. Grep confirmed :
- `packages/nexus-sdk/src/nexus_sdk/html_render.py:195` :
  `_SVG_PAD_R = 32` ✓

---

## Tableau recapitulatif des findings

| ID | Track | Severite | Titre | Action |
|---|---|---|---|---|
| A-1 | A | **P1** | `commit_sha` → `git clone --branch` ne supporte pas les SHA | **FIX avant Sprint 15 Phase A** |
| A-2 | A | P2 | `_dir_size` apres clone, pas pendant | Tech debt |
| A-3 | A | P2 | `_git_rev_parse` sans timeout | Tech debt |
| A-4 | A | P2 | `startswith("http")` accepte http:// | Tech debt |
| B-1 | B | P2 | `json.dumps` au lieu de `jcs` PyPI | Tech debt |
| B-2 | B | P2 | `verify_provenance` ne check pas `schema_version` | Tech debt |
| B-3 | B | P3 | signature hex (convention OK) | Rien |
| C-1 | C | P3 | pas de validation format node_id | Rien |
| C-2 | C | P3 | champs supplementaires SBFB acceptes | Rien |
| D-1 | D | P2 | bump v4 casse forward compat pour champ additif | Tech debt |
| D-2 | D | P3 | doc commentaire obsolete `publish.rs:30` | Nit |
| E-1 | E | P3 | pas d'aria-label sur badge | Nit |
| E-2 | E | P3 | badge Verifie et P2P meme style | Nit UX |
| F-1 | F | P3 | multipart streame par le client | Ignore |
| G-1 | G | P2 | 4/7 protections D4 sans tests dedies | Tech debt |
| G-2 | G | P2 | clone reel jamais teste (mock systematique) | Tech debt |

---

## Commits fix attendus (avant Sprint 15 Phase A)

**1 commit obligatoire** :

- `fix(sprint14): rename commit_sha to ref in deploy-from-repo body`
  (A-1). Option 1 du fix, minimale. Renommer le champ dans
  `DeployFromRepoBody`, propager au appel `_clone_repo(ref=...)`,
  ajuster le message d'erreur si le ref n'existe pas, mettre a jour
  le test happy path pour confirmer le renaming. ~20 LOC.

---

## P2 a logger en tech debt PATTERNS.md

A ajouter dans `docs/shell/PATTERNS.md` section Sprint 14 audit :

- **T44** — `_dir_size` post-clone (A-2). Mitigation : timeout 30s
  limite deja la taille transferable. Fix propre : streamer git
  clone avec `--progress` et surveiller la taille.
- **T45** — `_git_rev_parse` sans timeout (A-3). Post-clone, local
  disk uniquement — risque faible.
- **T46** — `startswith("http")` (A-4). `http://` accepte. Suggestion :
  regex strict `^https?://` ou forcer `https://` pour les forges
  publiques.
- **T47** — `json.dumps` vs `jcs` PyPI (B-1). Switcher a la prochaine
  occasion, documenter que pour schema ASCII flat c'est equivalent.
- **T48** — `verify_provenance` sans check `schema_version` (B-2).
  Ajouter guard si v2 introduit.
- **T49** — bump PA version 4 casse forward-compat (D-1). Pattern a
  reconsiderer pour additions non-breaking futures.
- **T50** — protections D4 clone sans tests dedies (G-1).
  path traversal, symlinks, oversized, timeout — ajouter tests
  Sprint 15/16.
- **T51** — clone reel jamais teste (G-2). Ajouter un test
  integration qui exerce le vrai subprocess git.

Total : 8 items tech debt ajoutes en Sprint 14 audit.

---

## P3 laisses sans action

- B-3 : signature hex (convention projet OK)
- C-1, C-2 : validations SBFB.json (comparison stricte suffit)
- D-2 : doc commentaire publish.rs:30 (nit)
- E-1, E-2 : accessibilite + style badge (nits UX)
- F-1 : bande passante multipart (cout-benefice defavorable)

---

## Notes on audit completeness

**Skippe par timebox** :
- scan-en-strings.sh non execute (manual inspection suffisait :
  pas de strings anglaises visibles dans Browse.tsx / BrowsedProject.tsx
  lignes 240-270)
- Playwright test badge non verifie (pas dans audit plan, mais
  noter qu'aucun test E2E ne visualise le badge dans un vrai browser)
- Performance du clone sur un repo reel de 500 MB non mesuree
- Pas verifie si `DeployFromRepoBody` a un max-length sur `repo_url`
  (pydantic par defaut n'a pas de limit)

**Checks joues** :
- Lecture integrale : forge.py, provenance.py, deploy.py, publish.rs,
  browse.rs, canonical.rs, daemon.ts (sections pertinentes),
  Browse.tsx + BrowsedProject.tsx (sections badge), PATTERNS.md
  (sections T41-T43)
- `cargo test -p nexus-shell-daemon-core --lib` : 113 passed
- `pytest test_deploy.py test_provenance.py test_forge.py -q` : 36 passed
- `vitest run BrowsedProject.test.tsx Browse.test.tsx` : 14 passed
- Verification locale : `git clone --branch <sha>` echoue (confirme A-1)
- Grep exhaustif : `text-white/30`, `_SVG_PAD_R`, `provenance_hash`,
  `verified-badge`, `T41|T42|T43`

**Ouvertures pour Sprint 15** :
- Hors scope audit mais a considerer dans kickoff Sprint 15 :
  re-evaluer si le champ `ref`/`commit_sha` est utile (commenter le
  pattern de pinning), envisager un test integration avec un vrai
  git local, polish du badge Verifie (couleur distincte, aria-label)
