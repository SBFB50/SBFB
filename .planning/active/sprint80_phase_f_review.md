# Sprint 80 Phase F Review

## Verdict: PASS

Driver-side deep review (independent agent) + Codex reconciliation.
Phase coherente, verifiee, 0 P0/P1, 1 P2 + 3 P3 (rigor signal G4 tenu).

### Codex reconciliation
Codex (`codex exec`, GPT 5.5) brut : `.planning/active/sprint80_phase_f_codex_review.md`.
Verdict : **7/7 livrables CONFIRME, 0 GAP, 0 PARTIEL**. Codex a relance les
tests (`working_tree_diff` 2 passed ; `--test operator_server operator_git_diff`
2 passed) et confirme independamment les invariants : 0 input user dans la
commande git, `parse_unified_diff` inchange, `truncated` au niveau enveloppe
(`FileDiff` intact), route sous `auth_required`, 0 route daemon / 0 `use
nexus_shell_daemon*` / 0 dep ajoutee.

**F1 (P2, branche `truncated=true` non testee) FERME in-phase** : test
`working_tree_diff_truncates_beyond_cap` ajoute (`sprint_history.rs`, fixture
> MAX_DIFF_LINES). Suites re-jouees apres ajout : fmt clean, tests cibles 4/4,
nextest workspace vert. **F2/F3/F4 = residus P3 documentes** (`git_cmd_in`
`unwrap_or_default` -> enveloppe vide indistinguable d'un arbre propre sur
erreur git ; `.trim()` peut rogner une ligne de contexte finale whitespace-only
[miroir de `commit_diff_data`] ; contrat Phase H : Zod `truncated: z.boolean()`
+ `.nullable()` sur `old/new_lineno`) -> routes vers `sprint80_verification.md`.

---

## Scope And Staging

Staging **propre et atomique**. `git status --short` :
- `M crates/sbfb-factory/src/operator_server.rs`
- `M crates/sbfb-factory/src/sprint_history.rs`
- `M crates/sbfb-factory/tests/operator_server.rs`
- `?? .planning/active/sprint80_phase_f_preflight.md` (planning, hors commit code)

Aucun fichier parasite. Le `M docs/security/THREAT_MODEL.md` du snapshot de boot
appartenait a Phase A (deja committe) — absent du working-tree courant. `git diff
HEAD --stat` = **3 fichiers, 181 insertions, 0 suppression**. 0 fichier `web/`.

Scope match plan §Phase F (`sprint80_plan.md:101-109`) : « `GET /api/git/diff`
retournant les hunks en JSON calcules en Rust ; gestion dirty/staged ; 0 route au
daemon ». Le diff livre exactement cela, rien de plus. Aucun refactor opportuniste
(`commit_diff_data`/`parse_unified_diff` intacts). Day-0 #9 (0 route daemon) et #11
(diff = verite Rust) tenus.

## Three-Block Verification

Bloc Rust **re-execute par l'auditeur** (pas de confiance aveugle aux chiffres fournis) :
- `cargo fmt --all --check` -> `FMT_OK` (rc=0).
- `cargo clippy -p sbfb-factory --all-targets --locked -- -D warnings` -> rc=0, 0 warning.
- `cargo nextest run -p sbfb-factory --locked` -> **214 passed, 0 skipped** (inclut les 3 nouveaux).
- `cargo nextest run -p sbfb-factory -E 'test(/git_diff/)'` -> **2 passed** :
  `operator_git_diff_requires_auth`, `operator_git_diff_endpoint_returns_envelope`.

Bloc Frontend : **non requis** — 0 fichier `web/` dans le diff (track backend pur).
Le contrat wire consomme par Phase H/J (front) est fige ici, non encore code cote web.

Bloc workspace complet (`--workspace` nextest 2008 + doctest + build release) :
**non re-execute integralement** par l'auditeur (perimetre crate isole, 0 dep,
0 surface partagee modifiee) ; les chiffres fournis sont coherents avec le delta
local (+3 = +1 unit +2 HTTP). Le filet Codex re-jouera le bloc complet.

## Delta Tests

| Suite | Avant | Apres | Delta | Note |
|---|---|---|---|---|
| nextest `sbfb-factory` (unit `bin`) | 213 | 214 | +1 | `working_tree_diff_separates_unstaged_and_staged` |
| nextest `operator_server` (HTTP) | — | +2 | +2 | `..._returns_envelope`, `..._requires_auth` |
| nextest workspace (fourni) | 2005 | 2008 | +3 | coherent |
| Frontend | — | — | 0 | **legitime** : 0 fichier web/ touche |

0-delta frontend justifie : la phase est strictement backend producteur.

## Modified-File Branch Coverage

`working_tree_diff_data` / `bounded_working_tree_diff` (nouveau code metier ~40 LOC) :
- **Chemin principal (non-truncated, unstaged+staged separes, kinds add/del/ctx)** :
  couvert par le test hermetique (repo temp, edit unstaged a.txt + stage b.txt) — asserte
  head non vide, `!truncated`, a.txt dans unstaged avec ins>=1 & del>=1, kinds {add,del,ctx}
  presents, b.txt dans staged, **a.txt absent de staged** (anti-fuite). Appels reels (vrai
  binaire git sur fixture isolee), assertions specifiques (pas `is_ok()`), inputs realistes.
- **Branche `truncated=true`** (`lines.len() > MAX_DIFF_LINES`, troncature + flag + OR
  `t_unstaged || t_staged`) : **NON exercee** par aucun test. -> Finding F1 (P2).
- `handle_git_diff` (wiring) : couvert par les 2 tests HTTP (forme 200 + 401 sans token).
- `git_cmd_in` : couvert transitivement par tous les chemins ci-dessus.

`parse_unified_diff` reutilise **inchange** (privee `:1068`, 0 changement de visibilite,
0 LOC de parsing dupliquee, pas deplace dans `diff.rs`). Sa couverture existante
(`parse_unified_diff_classifies_line_kinds`) reste valide.

## Security And Protocol

- **0 input user dans la commande git** (CONFIRME). `working_tree_diff_data(root: &Path)`
  et `bounded_working_tree_diff(root, cached: bool)` n'acceptent aucun rev/pathspec user.
  Tous les args sont litteraux (`vec!["-c","color.ui=false","diff","--no-color","-U3",
  "--no-ext-diff"]` + `--cached`). `git_cmd_in` cible le repo via `current_dir(root)` —
  **pas** de `-C <path>` ni de `format!` avec input. La classe d'injection git (S71 Phase D,
  `--end-of-options`/`is_safe_git_rev`) est **absente par construction** ; aucun `--end-of-options`
  necessaire. Le `root` provient de `OperatorState.root` (= `repo_root_pub()`), pas du caller.
- **Determinisme** : `-c color.ui=false` + `--no-color` (neutralise `color.ui=always` qui
  casserait `parse_unified_diff`) + `--no-ext-diff` (neutralise `diff.external`) — les 3 presents.
- **Auth** : `/api/git/diff` est DANS le sous-routeur `authed` (`operator_server.rs:197`)
  couvert par `.layer(from_fn_with_state(auth_state, auth::auth_required))` (`:208-211`).
  Verifie par `operator_git_diff_requires_auth` (Host loopback, **pas de token** -> 401, PASS).
  Le harness `TestServer::get` injecte `x-sbfb-token` -> le test envelope valide le 200 authentifie.
- **Divulgation** : la route revele le working-tree non-commite, mais le caller est deja
  loopback-auth T0 et l'Operator expose deja les memes octets via le terminal PTY (`git diff`)
  et `/api/sprint-history/diff/{sha}`. Aucun nouveau tier de confiance. Read-only, 0 spawn, 0 write.
- **Rouge-ligne DEEP** : pas d'`unsafe`/`#[allow(dead_code)]` nouveau, pas de crypto, pas de zip,
  pas de canonical/schema. Le seul `.unwrap()`-adjacent est `unwrap_or_default()` (graceful,
  miroir exact de `git_cmd` existant). `serde_json::json!(diff)` infaillible sur struct Serialize.
  Aucune ligne ne necessite l'audit complet « rouge-ligne » au-dela de ce qui precede.

## Research And G8

G8 preflight present, verdict **EXECUTE** (`sprint80_phase_f_preflight.md`). Les 6 decisions
code-ready sont suivies a la lettre :
1. Commandes git litterales -> CONFORME.
2. Reuse `parse_unified_diff` OUI, dans `sprint_history.rs`, 0 visibilite -> CONFORME.
3. Enveloppe JSON -> CONFORME, **avec** champ `truncated` (option durcissement §3 du preflight adoptee).
4. Branchement sous-routeur `authed` -> CONFORME.
5. Test hermetique + HTTP forme -> CONFORME (pas de surcharge env root, recommande NON par preflight).
6. Limites documentees -> doc-comment struct couvre untracked ; renames/binaires a citer au commit body.

0 dep ajoutee (Cargo.toml hors diff), 0 bump wire, 0 spec/crypto sans recherche. S1a (prior art
shell-git-then-parse : dugite/vscode) et le precedent interne `commit_diff_data` tracent l'approche.

## Patterns / Horizon

- `MAX_DIFF_LINES` = constante nommee (pas de magic number) — conforme `feedback_named_constants`.
- `git_cmd_in` = helper additif, miroir propre de `git_cmd` (DRY raisonnable, current_dir explicite).
- Horizon : enveloppe `{head,unstaged,staged,truncated}` + structs `FileDiff` partagees avec
  `commit_diff_data` -> le diff-viewer H rend commits passes ET working-tree avec UN composant.
  Tient a l'echelle (borne taille protege l'auto-render). `truncated` au niveau enveloppe
  (pas par-`FileDiff`) -> 0 impact sur les consommateurs de `CommitDiffResult`.

## Scope Cuts

Respectes (kickoff §Out) : F ne touche NI publish-via-Operator, NI editeur CM6, NI palette.
Strictement le backend producteur ; diff-viewer + actions de hunk = Phase H. Limites assumees
(non-bugs, a documenter au commit body) : untracked absents (coherent `git_dirty_files`),
renames purs (path vide), binaires (0 hunk).

## Codex verification

NON EXECUTE (driver-side). Le gate Codex (`codex exec`) doit re-jouer le bloc Rust complet
workspace + valider ; remplacer `PASS-PENDING` par `## Verdict: PASS` apres reconciliation,
ou `CONCERN`/`FAIL`. Security delta : nouvelle route GET loopback read-only derriere
`auth_required`, 0 input user, 0 nouveau tier de divulgation (equivalent au terminal/commit-diff
existants). 0 dep, 0 wire versionne, 0 route daemon.

## Commit Body Draft

```
feat(sbfb-factory): Sprint 80 Phase F — GET /api/git/diff working-tree (hunks Rust)

## Contexte
Diff du working-tree (unstaged + staged) en hunks JSON calcules en Rust =
verite repo unique (kickoff invariant #11), prerequis du diff-viewer VERIFY
(Phase H/J4). Reuse du moteur prouve parse_unified_diff (commit-diff). 0 dep,
0 route daemon, 0 wire versionne (Day-0 #9/#11).

## Fichiers
- crates/sbfb-factory/src/sprint_history.rs : git_cmd_in(root,args) (current_dir,
  0 arg user) ; const MAX_DIFF_LINES=20_000 ; struct WorkingTreeDiff
  {head,unstaged,staged,truncated} ; working_tree_diff_data(root) (reuse
  parse_unified_diff INCHANGE) ; bounded_working_tree_diff(root,cached)
  (git -c color.ui=false diff --no-color -U3 --no-ext-diff [--cached] + borne).
  +1 test hermetique (repo temp).
- crates/sbfb-factory/src/operator_server.rs : route /api/git/diff (get) dans
  sous-routeur authed + handler handle_git_diff(State).
- crates/sbfb-factory/tests/operator_server.rs : +2 tests HTTP (envelope, 401).

## Delta tests
nextest sbfb-factory 213->214 (+1 unit) ; +2 HTTP (operator_server) ;
workspace +3 (2005->2008). Frontend 0 (0 fichier web/).

## Verification
fmt --check OK ; clippy -D warnings rc=0 ; nextest sbfb-factory 214/0-skip ;
git_diff tests 2/2 ; doctest + build release fournis verts.

## Scope cuts
F = backend producteur seul. Diff-viewer + actions de hunk = Phase H. Limites
git assumees : untracked absents (coherent git_dirty_files), renames purs path
vide, binaires 0 hunk. truncation mid-hunk -> truncated=true (flag enveloppe).

## G8 traceability
Preflight EXECUTE (sprint80_phase_f_preflight.md), 6 decisions code-ready
suivies. Contrat Phase H fige : enveloppe {head,unstaged,staged,truncated},
Zod .nullable() sur old_lineno/new_lineno (Option<u32> -> null toujours present).

## Pre-launch protocol
JSON de route HTTP, PAS une enveloppe canonique versionnee — aucun _VERSION /
DOMAIN_*_V1 / JCS touche. Edition libre pre-tag, sans bump.

## Codex verification
Security delta : route GET loopback read-only derriere auth_required, 0 input
user (injection git S71-D absente par construction), 0 nouveau tier de
divulgation (equivalent terminal/commit-diff). [verdict Codex a inserer]

## Carry closure / Unblock
Debloque Phase H (diff-viewer) + J4. Carry P2 truncated-branch-untested ->
sprint80_verification.md + sprint81_audit_plan.md.
```

## Findings

- **F1 (P2) — branche `truncated=true` non testee.** `bounded_working_tree_diff`
  (`sprint_history.rs:~1043`) tronque a `MAX_DIFF_LINES` et propage `truncated`, mais
  aucun test n'exerce ce chemin. Correct par inspection (truncate a la frontiere de ligne,
  `join("\n")`, parse tolerant a l'input partiel — pas de panic), mais c'est du comportement
  neuf (durcissement) non verifie. Testable a faible cout (fixture d'un fichier > 20 000 lignes
  ajoute). **Owner** : driver. **Trigger** : Phase G/H ou sprint dette. **Exit** : 1 test
  hermetique assertant `truncated == true` + integrite du prefixe parse. Route vers
  `sprint80_verification.md` + `sprint81_audit_plan.md`.

- **F2 (P3) — erreur git / repo absent indistinguable d'un arbre propre.** `git_cmd_in`
  fait `unwrap_or_default()` (`:731`, miroir exact de `git_cmd`) : un echec git (binaire
  manquant, root non-repo) rend `{head:"",unstaged:[],staged:[],truncated:false}` en 200,
  sans signal d'erreur. Sans objet pour l'Operator (root = repo toujours), mais le viewer
  ne distingue pas « propre » de « erreur ». Coherent avec le pattern existant. Note
  d'observabilite, pas un bug.

- **F3 (P3) — `.trim()` peut perdre une ligne de contexte finale whitespace-only.**
  `git_cmd_in` trim la sortie (miroir de `git_cmd`) : une derniere ligne de contexte
  vide (prefixe espace, contenu vide) en fin de diff serait rognee. Edge de fidelite
  extreme, identique a `commit_diff_data`. Non-regression.

- **F4 (P3 / note Phase H) — contrat wire a refleter cote front.** L'enveloppe a **4
  champs** (`truncated` inclus, au-dela du `{head,unstaged,staged}` du preflight §3 base).
  Le Zod de Phase H doit donc inclure `truncated: z.boolean()` ET `.nullable()` (pas
  `.optional()`) sur `old_lineno`/`new_lineno` (`Option<u32>` sans `skip_serializing_if`
  -> `null` toujours present). Le test hermetique pourrait aussi asserter `b.txt` absent
  de `unstaged` (renforcement symetrique anti-fuite) — facultatif.

## Residual Risk

Faible. Surface read-only loopback authentifiee, 0 input user, 0 dep, 0 wire versionne,
0 route daemon, parser eprouve reutilise inchange. Le seul comportement neuf non couvert
(F1, truncation) est correct par inspection et sans impact sur le chemin commun. Aucun
P0/P1. La verite repo est calculee cote Rust comme exige (invariant #11). Codex requis
pour le verdict committable et le re-jeu du bloc workspace complet.
