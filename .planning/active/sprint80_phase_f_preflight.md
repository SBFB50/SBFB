# Sprint 80 Phase F Preflight

Date: 2026-06-27
HEAD: `e036f65`
Verdict: **EXECUTE**

## Resume (3-5 lignes)
La primitive Phase F (`GET /api/git/diff` = diff du working-tree, hunks JSON calcules en
Rust, vERITE repo unique) est **implementable telle quelle**, sans deviation de plan, sans
dep, sans bump wire. Tout le moteur de hunks existe DEJA et est prouve+teste in-repo :
`parse_unified_diff` + les structs `FileDiff`/`DiffHunk`/`DiffLine` (`sprint_history.rs:943-1084`,
test hermetique `:1096-1137`) servent deja `commit_diff_data` (`:972-989`) pour les commits
PASSES — F applique la MEME technique au working-tree (unstaged `git diff` + staged
`git diff --cached`). 0 dep (git2/gix ABSENTS du workspace, le projet SHELLE git partout),
0 wire format versionne (JSON de route HTTP, pas une enveloppe canonique), route additive
DANS le sous-routeur `authed` derriere `auth_required` (Day-0 #9 tenu, 0 route daemon).
Les 5 scans convergent : **EXECUTE**. Le preflight fige les decisions de conception
code-ready que le plan/kickoff ont explicitement deleguees ici (forme JSON, commande git
exacte, reuse `parse_unified_diff`, point de branchement, fixture de test) + 3 durcissements
non-bloquants adoptes DANS la phase (`--no-color` deterministe, borne de taille, contrat
Zod `.nullable()` pour le consommateur Phase H).

## Evidence Rules
- Claim policy: chaque affirmation cite un chemin/ligne, une sortie de commande, une URL/date,
  ou une hypothese explicite.
- Local sources read: `crates/sbfb-factory/src/operator_server.rs`, `crates/sbfb-factory/src/sprint_history.rs`,
  `crates/sbfb-factory/src/process.rs`, `crates/sbfb-factory/src/diff.rs`, `crates/sbfb-factory/src/auth.rs`,
  `crates/sbfb-factory/tests/operator_server.rs`, `crates/sbfb-factory/Cargo.toml`, `Cargo.toml`,
  `Cargo.lock`, `docs/security/THREAT_MODEL.md` (S14 Operator), `.planning/active/sprint80_plan.md`
  (§Phase F), `.planning/active/sprint80_kickoff.md` (Day-0 #9/#11, Arbitrage PO #1),
  `.planning/active/sprint80_phase_A_preflight.md`, `docs/rust/PATTERNS.md`,
  memory `feedback_approach.md` + `feedback_named_constants.md`.
- Commands run: `git rev-parse --short HEAD` -> `e036f65` ; `git log --oneline -- sprint_history.rs` ;
  `git log --all --grep="end-of-options|option injection|is_safe_git_rev"` ;
  Grep `^name = "(git2|gix|libgit2-sys|gix-diff|similar|diffy|imara-diff)"` sur `Cargo.lock`
  (seul `imara-diff 0.2.0` present, transitif) ; Grep `/api/git/diff|working_tree` sur le repo
  (0 hit en code, uniquement planning) ; `ls crates/sbfb-factory/src/` ;
  `grep "^mod" main.rs`.

## Scope
- Plan source: `.planning/active/sprint80_plan.md` §Phase F (lignes 101-109) + recap deps (l.173) ;
  kickoff §Scope F (l.126-128), Day-0 #9 (l.202-205), Invariant "diff = vERITE Rust" #11 (l.211),
  Arbitrage PO #1 (l.76-83).
- Target files:
  - `crates/sbfb-factory/src/operator_server.rs` — ajouter `.route("/api/git/diff", get(handle_git_diff))`
    dans le sous-routeur `authed` (`:170-207`, entre les routes existantes et `.fallback_service`/`.layer(auth_required)`),
    + le handler `handle_git_diff(State<OperatorState>)`.
  - `crates/sbfb-factory/src/sprint_history.rs` — ajouter `pub fn working_tree_diff_data(root: &Path) -> WorkingTreeDiff`
    + struct `WorkingTreeDiff`, a cote de `commit_diff_data` (`:972`), reutilisant la fn privee
    `parse_unified_diff` (`:991`) et les structs `FileDiff`/`DiffHunk`/`DiffLine` deja `pub` (`:950-970`).
  - `crates/sbfb-factory/tests/operator_server.rs` — test HTTP de forme (200 + enveloppe) sur le repo reel.
  - (unit hermetique : dans `sprint_history.rs` `#[cfg(test)] mod tests`, fixture temp git repo.)
- Deps/APIs/specs: **AUCUNE** (shell `git`, 0 crate ajoutee). `axum 0.8` + `tower-http 0.6` deja presents.
- Security/protocol surfaces: route HTTP loopback **lecture seule** derriere `auth_required`
  (host+origin+bearer/cookie). 0 wire format versionne. Pas de spawn, pas de write disque.
- Tests expected (plan §F + kickoff T1 sous-test 5):
  - unit hermetique : `working_tree_diff_data` sur un repo temp (init+commit+modif+stage) -> hunks deterministes.
  - HTTP : `GET /api/git/diff` -> 200 + enveloppe `{unstaged:[], staged:[], head}` (forme, pas contenu) ;
    + 401 sans auth (couvert par le middleware `authed` existant, peut etre asserte explicitement).
  - Phase H consommera la route (Playwright sous-test 5).

## S1a OSS Prior Art
- Domain: calcul d'un diff working-tree (unstaged + staged) en hunks structures, cote serveur Rust,
  expose en JSON a un front.
- Precedent INTERNE decisif (le plus fort) : `crates/sbfb-factory/src/sprint_history.rs` calcule
  DEJA un diff de **commit passe** par shell-git + parse maison :
  - `commit_diff_data` (`:972-989`) : `git diff -U3 --end-of-options <sha>^..<sha>` -> `parse_unified_diff`.
  - `parse_unified_diff` (`:991-1084`) : machine a etats sur la sortie unified-diff (`diff --git`,
    `--- a/`, `+++ b/`, `@@ -x,y +a,b @@`, lignes `+`/`-`/` `), produit `Vec<FileDiff>` avec
    `insertions`/`deletions` comptes + `old_lineno`/`new_lineno` par ligne.
  - Teste hermetiquement par fixture string (`:1096-1137`, `parse_unified_diff_classifies_line_kinds`).
  F est **le meme moteur, source = working-tree** au lieu d'une plage de revisions. La sortie
  `git diff` (working-tree) et `git diff <range>` ont un format unified-diff **identique** ->
  `parse_unified_diff` s'applique tel quel, sans modification.
- Sources OSS externes (cross-check de la technique shell-git-then-parse) :
  - GitHub Desktop / `dugite` (Electron) : shell `git diff` puis parse le texte unified plutot que
    lier libgit2 — meme philosophie shell-then-parse (https://github.com/desktop/dugite, consulte 2026-06-27).
  - VS Code git extension : invoque le binaire `git` et parse la sortie (pas de binding natif),
    pour la robustesse cross-version (https://github.com/microsoft/vscode, dossier `extensions/git`).
  - `git diff` cote serveur en hunks JSON est un pattern eprouve (cf. les "diff hunks" de l'API
    GitHub/GitLab) — la forme par-fichier/par-hunk/par-ligne est canonique.
- Lib externe alternative consideree et REJETEE : `git2` (libgit2) / `gix` exposent `Repository::diff_*`
  qui produit des deltas/hunks en memoire sans shell. **Rejet motive** : (1) le projet SHELLE git
  PARTOUT (`process.rs:56,204,213,223,240,256` ; `sprint_history.rs:712` ; option-injection deja
  durci `--end-of-options` S71 Phase D) -> introduire libgit2 serait une 2e voie d'acces a git,
  incoherente ; (2) `sbfb-factory` ne depend QUE de `sbfb-manifest` + `nexus-core-rs`
  (`Cargo.toml:11-12`) et evite deliberement les gros arbres (cf. duplication volontaire des
  helpers loopback `auth.rs:21-30`) ; `git2` tire `libgit2-sys` (C, build), `gix` tire un arbre
  Rust massif ; (3) reuse de `parse_unified_diff` = 0 LOC nouvelle de parsing, 1 fonction +
  2 commandes git. La memory `feedback_approach.md` (pick-deepest) ne pousse PAS vers libgit2 ici :
  le "plus pousse" = la coherence + le reuse du parser teste, pas l'ajout d'un binding C.
- Finding: **APPROACH-ALIGNED**. Le plan (shell-git, hunks calcules Rust, reuse possible de
  `parse_unified_diff`) matche exactement la pratique interne prouvee + l'art OSS externe.
- Impact: aucune adaptation. Decision de reuse `parse_unified_diff` = OUI (cf. §Decisions code-ready).

## S1b Dependencies, CVEs, Release Notes
- Scanned: `git2`, `gix`, `libgit2-sys`, `gix-diff`, `similar`, `diffy`, `imara-diff` (lock) ;
  `axum 0.8`, `tower-http 0.6` (deja la, Phase A).
- Commands/sources:
  - Grep `^name = "(git2|gix|libgit2-sys|gix-diff|similar|diffy|imara-diff)"` sur `Cargo.lock`
    -> **seul `imara-diff 0.2.0`** present (`Cargo.lock:3764`).
  - Reverse-dep `imara-diff` -> `tor-consdiff 0.41.0` (`Cargo.lock:9085-9094`), partie de l'arbre
    arti-client/Tor. `sbfb-factory` ne depend PAS du daemon ni de Tor (`Cargo.toml:11-12` =
    sbfb-manifest + nexus-core-rs ; aucun `nexus-shell-daemon*`) -> `imara-diff` est **inaccessible**
    a `sbfb-factory`. Pas de collision, pas de reutilisation involontaire.
  - `git2`/`gix`/`similar`/`diffy` : **ABSENTS** du lock et du `Cargo.toml` workspace.
  - Transitive depth (P2-PREFLIGHT-TRANSITIVE-DEPTH) : **N/A** — F n'ajoute NI ne bumpe AUCUNE dep,
    donc aucun graphe transitif a resoudre, aucun `cargo tree -d` requis. La seule "dep" est le
    binaire `git` (deja requis par `process.rs`/`sprint_history.rs` + les tests existants ;
    `repo_root_resolves` (`process.rs:909`) atteste sa presence en CI).
- Finding: **clean**. 0 dep ajoutee, 0 bump, 0 CVE applicable (aucune surface crypto/wire/network
  nouvelle ; le binaire git est deja la base de l'Operator).

## S2 Historical Decisions
- Commands:
  - `git log --oneline -- crates/sbfb-factory/src/sprint_history.rs` ->
    `5f2cc9a` (diff endpoint + inline code viewer + all sprints navigation),
    `f19ed83` (Sprint 71 Phase D — harden git rev injection : `--end-of-options` + `is_safe_git_rev`),
    `17ead31` (unbounded phase discovery), `e73c9fb`, `a8a273f`.
  - `git log --all --grep="end-of-options|option injection|is_safe_git_rev"` ->
    `f19ed83` (Sprint 71 Phase D), `bcfc155` (Sprint 74 Phase B fork).
- Decisions traversees :
  - **`f19ed83` (S71 Phase D, retro-Codex P1)** : le shell de git avec un rev USER (`{sha}`)
    pouvait etre detourne (`--output=<path>` ecrit un fichier arbitraire). Mitigation en place :
    `is_safe_git_rev` (`operator_server.rs:382-386`, rejette `-`/whitespace/control) + `--end-of-options`
    dans `commit_diff_data` (`:976,981`) et `audit_commit_data` (`process.rs:584`). Tests
    `operator_commit_diff_rejects_option_injection` + `operator_audit_rejects_option_injection`
    (`tests/operator_server.rs:1245-1268`). **Application a F** : F n'a **AUCUN input user**
    (working-tree, pas de rev, pas de pathspec user) -> la classe d'injection est ABSENTE par
    construction ; `is_safe_git_rev` est sans objet. La commande est 100% litterale (cf. §Decisions).
    Reverse-commit check : la regle anti-injection est ACTIVE (non revertee, tests verts) ->
    F la respecte trivialement (0 arg user) ; aucun conflit.
  - **Invariant kickoff #11 "diff = vERITE Rust"** (`sprint80_kickoff.md:211` ; plan §F l.104) :
    le diff working-tree est calcule EN Rust, jamais un diff JS divergent ; les actions de hunk
    sont des intentions routees a la session (re-applique sous gate), jamais des mutations directes.
    F l'IMPLEMENTE (producteur Rust) ; H consomme. Aucune decision anterieure ne fige "shell-git
    vs lib" autrement que par le precedent shell-git omnipresent -> coherent.
  - Recherche reseau S71/S73 (`/api/sprint-history`, FTS5) : sans rapport avec le working-tree diff,
    aucune contrainte heritee.
- Finding: **clean**. Aucun commit `DEVIATION|rejected|scope-cut` ne fige une approche contraire ;
  la seule decision pertinente (anti-injection S71 Phase D) est respectee par construction
  (0 input user). "diff = vERITE Rust" est un invariant POUR cette phase, pas contre.

## S3 Local Patterns And Threat Model
- Threats/contracts checked: §14 Operator surface de `docs/security/THREAT_MODEL.md`
  (T-OPERATOR-CSRF `:800`, T-OPERATOR-SPAWN `:854`, Residual risks `:906`). Tier loopback T0
  (`LOOPBACK_ENDPOINTS_TRUST_TIERS.md §3.1`, cite `:798`).
- Map de la primitive F : route **GET lecture seule**, **0 write disque**, **0 spawn de processus**.
  Elle est strictement moins risquee que les surfaces deja cataloguees :
  - vs T-OPERATOR-CSRF : F est derriere le MEME `auth_required` (host+origin+bearer, fallback
    cookie+`Sec-Fetch-Site` Phase A) via le sous-routeur `authed` (`operator_server.rs:170-207`,
    `.layer(auth_required) :203`). Pas de nouvelle surface entrante non-auth.
  - vs T-OPERATOR-SPAWN : F ne spawn rien ; le gate `SENSITIVE_ACTIONS` est hors-sujet (pas de
    chat, pas d'agent).
  - Divulgation d'information : F revele le contenu du working-tree (source + modifs non-commitees).
    MAIS le caller est deja loopback-authentifie au tier T0, et l'Operator expose DEJA :
    `/api/context` (dirty_files/staged_files **listes** + recent_commits, `process.rs:809-825`),
    `/api/sprint-history/diff/{sha}` (diff complet de commit, `:1289`), et un terminal xterm PTY
    (`/api/terminal/ws:193`) ou l'operateur tape `git diff` librement (kickoff §D, l.116-119).
    F n'ouvre **AUCUN nouveau tier de confiance ni nouvelle frontiere de divulgation** : il rend
    en JSON ce que le terminal rend deja en texte. Pas de regression T0-T5.
- HARDENING_ROADMAP : aucune pre-exigence S80/Phase F manquante (la primitive est read-only,
  hors des tracks crypto/transport/sandbox).
- Findings **non-bloquants** (durcissements adoptes DANS la phase, pas des carries) :
  - **P2 (robustesse, adopte)** : un utilisateur avec `git config --global color.ui always`
    injecterait des codes ANSI dans la sortie `git diff` -> `parse_unified_diff` casserait
    (les `@@`/`+`/`-` precedes d'ANSI ne matcheraient plus). Le `commit_diff_data` existant a la
    MEME faiblesse latente mais ne l'a jamais subie en CI. Decision F : forcer `-c color.ui=false`
    ET `--no-color` (deterministe, "vERITE Rust"). C'est un durcissement, pas un changement d'approche.
  - **P2 (DoS/UX, adopte)** : un working-tree enorme (bundle vendore, gros refactor) produirait
    un JSON multi-Mo qui figerait le diff-viewer auto-rendu (VERIFY). Ni `commit_diff_data` ni le
    plan ne bornent. Decision F : borne par-fichier (ex. tronquer au-dela de N lignes/hunk) +
    drapeau `truncated: true` par fichier, OU borne d'octets globale + `truncated` racine
    (constante nommee, cf. `feedback_named_constants`). Non-bloquant (precedent commit-diff non-borne),
    mais "pick-deepest" -> on borne.
  - **P3** : pas de `TraceLayer` sur l'Operator (`build_router` `:119-222` n'en pose aucun) ->
    le contenu du diff n'est pas logge. Rien a faire ; noter l'invariant "ne pas logger le diff".
- Finding: **clean** (aucune regression de menace couverte ; 2 durcissements P2 + 1 note P3 traites
  in-phase, 0 carry).

## S4 Protocol And Wire Invariants
- Wire/security files checked: `crates/nexus-core-rs/src/canonical.rs` (hors perimetre), structs
  `FileDiff`/`DiffHunk`/`DiffLine` (`sprint_history.rs:950-970`), enveloppe a creer `WorkingTreeDiff`.
- VERSION/domain/canonical status: **0 wire format versionne touche**. La reponse de `GET /api/git/diff`
  est un JSON de route HTTP (axum `Json`), PAS une enveloppe serialisee canonique (`FeedEntry`, `Task`,
  `*_ANNOUNCEMENT_VERSION`, `DOMAIN_*_V1`, JCS). Aucun bump, aucune redefinition canonique. Politique
  pre-launch respectee (aucun `_VERSION` n'entre en jeu).
- Day 0 status: **preserved**.
  - **Day-0 #9 "Factory hors daemon, 0 route daemon"** : F ajoute la route dans
    `operator_server.rs::build_router` (serveur autonome `127.0.0.1:{port}`, `run_server:224`), PAS
    dans `nexus-shell-daemon`. Garde G-REVIEW : aucun `use nexus_shell_daemon*` introduit ;
    `sbfb-factory` ne depend toujours que de `sbfb-manifest` + `nexus-core-rs` (`Cargo.toml:11-12`).
  - **Day-0 #11 "diff = vERITE Rust"** : honore (producteur Rust unique).
- **Trace producteur -> consommateur (P2-PREFLIGHT-WIRE-CONTRACT-DEPTH)** :
  - **Producteur** : `handle_git_diff` -> `Json(WorkingTreeDiff{ unstaged: Vec<FileDiff>, staged:
    Vec<FileDiff>, head: String })`. `FileDiff`/`DiffHunk`/`DiffLine` portent deja `#[derive(Serialize)]`
    (`sprint_history.rs:950,958,964`). Forme exacte par ligne :
    `DiffLine{ kind:String("add"|"del"|"ctx"), content:String, old_lineno:Option<u32>, new_lineno:Option<u32> }`.
  - **Point clE** : `old_lineno`/`new_lineno` sont `Option<u32>` **SANS** `#[serde(skip_serializing_if)]`
    (`sprint_history.rs:967-969`) -> serde serialise `None` en `"old_lineno": null` (**toujours present,
    jamais absent**). C'est EXACTEMENT la lecon S73 Phase E (SearchResult provenance null-toujours-present) :
    le consommateur Phase H doit utiliser Zod **`.nullable()`** (PAS `.optional()`) sur `old_lineno`/`new_lineno`,
    et `kind` est une string **toujours presente** dans `{add,del,ctx}`.
  - **Enveloppe vs nu** : la reponse est une **enveloppe** `{unstaged, staged, head}`, **PAS** un array
    nu (meme lecon S73 : `{results,total,took_ms}` pas un array). Un fichier partiellement stage apparait
    LEGITIMEMENT dans `unstaged` ET `staged` avec des hunks differents (semantique git correcte, pas un doublon).
  - **Consommateur** : Phase H (diff-viewer greenfield) — **n'existe pas encore** (Grep `/api/git/diff`
    = 0 hit en code, uniquement planning). Donc **0 drift possible aujourd'hui** : le contrat est FIXE ICI
    pour que H le cible. Aubaine : le diff-viewer H rendra commits-passes (Phase D, meme `FileDiff`) ET
    working-tree (Phase F) avec UN SEUL composant -> argument fort pour reutiliser les structs identiques.
- `kind` ("add"/"del"/"ctx") = valeur de domaine enumeree (`feedback_named_constants`). Elles sont
  **pre-existantes** dans `parse_unified_diff` (`:1041,1052,1063`, litterales). F **reutilise** le parser
  inchange -> ne PAS introduire de nouveaux magic strings ; si F ajoute un statut enumere NEUF (ex. un
  `file_status` rename/binary), ce sera une constante nommee. Pas de refactor opportuniste des litteraux
  existants (anti-pattern G8 #6).
- Finding: **clean**. Aucun wire versionne, Day-0 #9/#11 tenus, contrat H fige (Zod `.nullable()` +
  enveloppe), 0 dep, AGPL-3.0 (toutes deps deja permissives, shell-git natif).

## Plan Adaptation
N/A (verdict EXECUTE — pas de PLAN-ADAPT). L'approche du plan est inchangee ; le preflight ne fait
que figer les details d'implementation que le plan/kickoff ont delegues au preflight.

## Decisions de conception figees pour le code (prete a coder)
1. **Commande(s) git (litterales, 0 input user, deterministes)** dans `working_tree_diff_data(root)` :
   - unstaged : `git -C <root> -c color.ui=false diff --no-color -U3 --no-ext-diff`
   - staged   : `git -C <root> -c color.ui=false diff --no-color -U3 --no-ext-diff --cached`
   - provenance head : `git -C <root> rev-parse --short HEAD` (pour `head` / fraicheur `run@<rev>`).
   `-C <root>` cible `OperatorState.root` (`run_server:225` = `repo_root_pub()`) -> rend la fonction
   testable contre une fixture temp. `--no-color`/`-c color.ui=false` neutralise un `color.ui=always`
   global. `--no-ext-diff` neutralise un `diff.external` user. **Pas** de `--end-of-options` necessaire
   (aucun arg user a desambiguiser) ; **pas** de pathspec. Memes `-U3` que `commit_diff_data` (`:981`).
2. **Reuse `parse_unified_diff` : OUI.** Ajouter `working_tree_diff_data` DANS `sprint_history.rs`
   (meme module que `parse_unified_diff` prive `:991` + `commit_diff_data` `:972`) -> appel direct,
   **0 changement de visibilite**, 0 LOC de parsing dupliquee. NE PAS mettre dans `diff.rs` (c'est le
   diff de DRIFT template `diff_workspace` vs `template_engine::expected_files`, semantique differente,
   sans git — `diff.rs:28`). NE PAS extraire/deplacer `commit_diff_data` (refactor opportuniste interdit).
3. **Forme JSON (enveloppe, fige le contrat Phase H)** :
   ```json
   { "head": "e036f65",
     "unstaged": [ { "path": "...", "insertions": N, "deletions": M,
                     "hunks": [ { "header": "@@ -a,b +c,d @@ ...",
                                  "lines": [ { "kind": "ctx|add|del",
                                               "content": "...",
                                               "old_lineno": 12, "new_lineno": 12 } ] } ] } ],
     "staged": [ /* idem */ ] }
   ```
   `FileDiff`/`DiffHunk`/`DiffLine` reutilises tels quels. Nouvelle struct
   `WorkingTreeDiff { head: String, unstaged: Vec<FileDiff>, staged: Vec<FileDiff> }` (`#[derive(Serialize)]`).
   `old_lineno`/`new_lineno` = `Option<u32>` -> serialisees `null` quand absentes (contrat Zod `.nullable()` cote H).
   (Option borne-taille : ajouter `truncated: bool` par `FileDiff` + une constante nommee
   `MAX_DIFF_LINES_PER_FILE` — cf. S3 P2 ; si adopte, documenter le champ comme tolerance runtime.)
4. **Branchement de la route** : dans `build_router`, sous-routeur `authed`
   (`operator_server.rs:170-199`), ajouter `.route("/api/git/diff", get(handle_git_diff))` parmi les
   routes existantes (avant `.fallback_service` `:199` et `.layer(auth_required)` `:203`). Le handler :
   `async fn handle_git_diff(State(state): State<OperatorState>) -> Json<serde_json::Value>` (ou
   `impl IntoResponse`) -> `Json(serde_json::to_value(crate::sprint_history::working_tree_diff_data(&state.root)).unwrap())`.
   Derriere `auth_required` automatiquement (le `.layer` `:203` couvre toutes les routes du sous-routeur).
   Namespace `/api/git/` neuf et additif (les autres routes sont `/api/sprint-history/diff/{sha}`).
5. **Fichiers de test** :
   - **Hermetique (determinisme des hunks)** : dans `sprint_history.rs` `#[cfg(test)] mod tests`, creer
     un repo temp (`tempfile::tempdir`, deja dev-dep `Cargo.toml:38`), `git init`, `git -c user.email=t@t
     -c user.name=t commit`, modifier un fichier suivi (unstaged) + stager une autre modif (`git add`),
     appeler `working_tree_diff_data(tmp)` et asserter : `unstaged` contient le fichier modifie avec
     `kind` ctx/add/del corrects, `staged` contient l'autre, `head` non vide. Hermetique (repo isole).
   - **HTTP (cablage + forme)** : dans `tests/operator_server.rs`, `server.get("/api/git/diff")` -> 200,
     `body["unstaged"].is_array()`, `body["staged"].is_array()`, `body.get("head").is_some()`
     (forme, PAS contenu — le repo reel est dirty pendant le run, contenu non-deterministe). Modele :
     `operator_commit_diff_endpoint_returns_inline_code` (`tests/operator_server.rs:1202`).
   - NE PAS introduire de surcharge de root operateur via env pour un test HTTP-fixture : l'unit test
     hermetique satisfait deja l'intention "workspace git fixture" du plan §F (l.109) sans elargir la
     surface prod. (Si le PO exige le test HTTP contre fixture, un `SBFB_OPERATOR_ROOT` opt-in serait la
     voie, mais YAGNI + surface minimale -> recommande NON.)
6. **Limites documentees (semantique git, a noter dans le commit body)** :
   - Fichiers **non-suivis** (untracked, `??`) : absents de `git diff` -> hors scope, **coherent** avec
     `git_dirty_files()` qui filtre deja `?? ` (`process.rs:232`). Option cheap : array `untracked:[paths]`
     via `git status --porcelain` si le PO veut les surfacer (sans hunks). Defaut : hors scope.
   - **Renames** purs (0 contenu) : `git diff` (working-tree) ne detecte pas les renames par defaut ;
     `parse_unified_diff` laisserait un `path` vide sur un bloc rename-only (pas de `+++ b/`). Cas rare au
     working-tree ; documenter. Fichiers ajoutes/supprimes (suivis) OK (`--- a/`/`+++ b/` ou `/dev/null`
     geres par le parser `:1016-1025`).
   - **Binaires** : `git diff` emet "Binary files ... differ" (pas de hunk) -> `FileDiff` a `path` vide /
     0 hunk. Acceptable (le viewer affiche "binaire") ; ameliorable plus tard.

## Risks And Scope Cuts
- Blocking risks: **aucun**.
- Non-blocking risks (traites DANS la phase, 0 carry) : color.ui injection -> `--no-color` ;
  diff enorme -> borne+`truncated` ; untracked/renames/binaires -> limites documentees ;
  pas de TraceLayer -> invariant no-log-diff.
- Scope cuts honored (kickoff §Out, l.149-163) : F ne touche NI au publish-via-Operator (reste CLI),
  NI a l'editeur CM6, NI a la palette. F est strictement le backend producteur du diff working-tree ;
  le diff-viewer + actions de hunk = Phase H. Day-0 #9 (0 route daemon) + #11 (vERITE Rust) tenus.

## Action
- **EXECUTE**: implementer la Phase F telle que planifiee, avec les decisions code-ready ci-dessus.
  Commit body : citer ce preflight (G8 traceability), documenter les limites (untracked/renames/binaires),
  les durcissements (`--no-color`, borne taille), et le contrat wire fige pour Phase H
  (enveloppe `{head,unstaged,staged}`, Zod `.nullable()` sur `old_lineno`/`new_lineno`).

## Verdict: EXECUTE
