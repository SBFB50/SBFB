# Process tooling Claude Code sur nexus-grid

Cette doc recense l'outillage qu'on ajoute au-dessus de Claude Code
vanilla pour durcir la qualite du code produit sur nexus-grid. C'est
un complement a [`README.md`](README.md) qui decrit le cycle sprint.

**Cible** : tout contributeur qui ouvre nexus-grid avec Claude Code
(terminal, VS Code, desktop) doit pouvoir installer ce tooling en
un script idempotent et heriter du meme process qualite que FlowUP.

**Non-objectif** : ce doc ne justifie PAS chaque choix d'outil. Le
choix initial et la comparaison d'alternatives sont dans les
discussions session de [`memory/`](../../../memory/) correspondantes.

---

## 1. Vue d'ensemble — 5 couches de qualite

Le process empile 5 couches independantes. Chacune catch une classe
d'erreurs differente. Tu peux activer chaque couche separement.

| # | Couche | Moment | Outil principal |
|---|---|---|---|
| 1 | Garde-fous automatiques | PostToolUse (chaque write) | `.claude/hooks/verify-on-write.sh` + Semgrep |
| 2 | Skills qualite specialises | Sur demande Claude | Trail of Bits skills + `nexus-phase-review` |
| 3 | Subagent review intra-sprint | Pre-commit d'une phase | `nexus-phase-auditor` agent |
| 4 | Multi-modele / second opinion | (non active sur nexus) | — |
| 5 | Observability + memory hygiene | PostCommit + statusline | hooks memory + statusline enrichi |

L'**audit gate inter-sprint** (cf. [`README.md`](README.md) §3) reste
la couche de reference. Ce tooling la complete sans la remplacer.

---

## 2. Installation

### 2.1 Prerequis

- Claude Code >= 2.1 (hooks `PostToolUse` / `PreToolUse`, `matcher`, stdin JSON)
- Bash >= 4 (Git Bash sur Windows fonctionne)
- `jq` dans le PATH **OU** `python3` (les hooks detectent auto lequel utiliser
  pour parser l'input JSON — fail-open silent si aucun des deux n'est present)
- `cargo`, `uv`, `npm` installes (pour les linters scope au langage)
- `node` (pour `nexus-statusline.js`)

### 2.2 Hooks local au repo (automatique)

Rien a installer. Le fichier `.claude/settings.json` est committe dans
le repo, donc toute session Claude Code ouverte dans nexus herite
automatiquement du hook `PostToolUse` qui execute
`.claude/hooks/verify-on-write.sh`.

Verifier que ca marche :

```bash
echo '{"tool_input":{"file_path":"docs/claude/README.md"}}' \
  | bash .claude/hooks/verify-on-write.sh && echo "hook OK"
```

Sortie attendue : `hook OK` (le hook no-op sur docs/).

### 2.3 Trail of Bits skills (user-level)

Les skills Trail of Bits s'installent au niveau user (`~/.claude/skills/`)
et sont partages entre tous les projets.

```bash
# Clone (idempotent avec pull si existe deja)
if [ -d ~/.claude/skills/trailofbits ]; then
  git -C ~/.claude/skills/trailofbits pull
else
  git clone --depth 1 https://github.com/trailofbits/skills.git \
    ~/.claude/skills/trailofbits
fi
```

Alternative via marketplace officiel Claude Code :

```
/plugin marketplace add trailofbits/skills
/plugin menu
```

### 2.4 Script tout-en-un

```bash
# Install interactive (prompts pour chaque composant optionnel)
bash scripts/install-claude-tooling.sh

# Install non-interactive (accepte les defaults, skip TDD Guard)
bash scripts/install-claude-tooling.sh --yes

# Install minimal (core uniquement, pas Semgrep ni TDD Guard)
bash scripts/install-claude-tooling.sh --minimal
```

Le script est idempotent (re-run safe) et auto-detecte ce qui est
deja installe. Composants couverts :
1. Git post-commit hook (memory updater)
2. Trail of Bits skills clone/pull
3. Semgrep via pip (optionnel)
4. TDD Guard via npm + cargo + pip (optionnel, opt-in)

Ne touche PAS les fichiers `.claude/` committed dans le repo (ils
sont deja actifs automatiquement).

### 2.5 Auto-detection au SessionStart

Un hook `SessionStart` (matcher `startup|resume`) tourne a chaque
ouverture de Claude Code dans le repo nexus. Il fait 2 choses :

1. **Auto-install leger** : si `.git/hooks/post-commit` n'existe pas
   et que `.claude/hooks/post-commit-memory.sh` est present, il ecrit
   automatiquement un wrapper dans `.git/hooks/post-commit`. Idempotent
   (ne touche rien si deja installe).

2. **Detection composants optionnels manquants** : verifie la presence
   de `jq` OU `python3`, de `~/.claude/skills/trailofbits/`, de
   `semgrep`. Si quelque chose manque, il emet un `additionalContext`
   JSON qui informe Claude :

```
[session-start] Composants process tooling manquants (optionnels) :
  - Trail of Bits skills : 'bash scripts/install-claude-tooling.sh' pour cloner
  - semgrep : 'pip install --user semgrep' (regles SBFB dans .semgrep/sbfb.yml ne tourneront pas)
[session-start] Install all-in-one : bash scripts/install-claude-tooling.sh
```

**Ne fait PAS** d'install externe automatique (npm/pip/cargo install).
Ces commandes peuvent prendre des minutes ou echouer — mauvaise
experience au SessionStart. Le user lit le signal et lance le script
install explicite quand ca l'arrange.

**Marker** : `.claude/_autoinstall_signaled.marker` est cree au premier
signal pour eviter de repeter le message a chaque session. Delete le
marker pour re-signaler (utile apres un `bash scripts/install-claude-
tooling.sh` partiel qui laisse des composants manquants).

---

## 3. Couche 1 — Garde-fous automatiques

### 3.1 Hook `verify-on-write.sh`

**Fichier** : `.claude/hooks/verify-on-write.sh`

**Declenchement** : PostToolUse matcher `Edit|Write`. Lance le linter
scope au fichier qui vient d'etre ecrit par Claude.

| Extension | Commande lancee |
|---|---|
| `.rs` | `cargo clippy -p <crate detecte> --lib --tests -- -D warnings` |
| `.py` | `uv run ruff check <file>` |
| `.ts/.tsx/.js/.jsx` (dans `web/`) | `cd web && npx eslint <file>` |
| autre | no-op |

**Early-exit** si :
- pas dans le repo nexus (absence de `Cargo.toml` + `crates/nexus-core-rs`)
- fichier dans `.planning/`, `docs/`, `target/`, `node_modules/`, `.venv/`,
  `dist/`, `build/`, `.git/`, `.claude/`

**Exit code** : 0 (clean), 2 (bloque avec message d'erreur visible par Claude).

**Rationale** : la verification §7.4 est une checklist humaine. Ce hook
la rend automatique sur chaque write -> zero clippy/ruff/eslint rate
avant commit.

### 3.2 Semgrep rules SBFB

**Fichier** : `.semgrep/sbfb.yml` (committed, partage)

**4 regles livrees** (high signal, low false-positive) :

| Id | Langue | Signal |
|---|---|---|
| `sbfb-no-todo-macros-rust` | Rust | `unimplemented!()`, `todo!()`, `panic!("not impl")` en production (hors tests/examples/benches). Incident de reference : ButtonBlock.task_submit stub Sprint 6 (§3.4 README). |
| `sbfb-no-placeholder-console-frontend` | TypeScript/JS | `console.*("not impl|todo|fixme|stub|placeholder|wip|coming soon|temporary...")` en production (hors tests). Incident meme pattern. |
| `sbfb-ignore-requires-reason-rust` | Rust | `#[ignore]` sans commentaire `// reason:` ou `// TODO: ref T-NN` precedant. Tie back docs/rust/PATTERNS.md tech debt. |
| `sbfb-ignore-requires-reason-python` | Python | `@pytest.mark.skip` et `skipif` sans argument `reason=`. |

**Rules architecturales TODO** (necessite `semgrep-rule-creator` ToB
pour generer avec le code en main, cf. §4.1) :

- `sbfb-canonical-bytes-jcs` : wire format serialise via JCS pas serde_json
- `sbfb-loopback-peer-creds` : route `/loopback/*` doit checker `PeerCredsVerified`
- `sbfb-iroh-endpoint-pin` : `iroh::Endpoint::builder()` avec pin discovery
- `sbfb-project-announcement-repo-url` : public -> `repo_url` obligatoire
- `sbfb-zip-path-traversal` : `zip::read_file` avec path validation

**Installation de Semgrep** (requis pour faire tourner les regles) :

```bash
# Option 1 : pip (recommande pour uniformite Python workspace)
pip install semgrep

# Option 2 : via uv (local a l'environnement nexus)
uv tool install semgrep

# Option 3 : brew (macOS)
brew install semgrep
```

**Invocation manuelle** :

```bash
# Scan complet (workspace)
semgrep --config .semgrep/sbfb.yml crates/ packages/ web/src/

# Scan d'un fichier unique (rapide, pre-commit)
semgrep --config .semgrep/sbfb.yml crates/nexus-core-rs/src/foo.rs

# Mode strict (exit 1 si findings)
semgrep --config .semgrep/sbfb.yml --error crates/ packages/ web/src/
```

**Integration hook** : `verify-on-write.sh` **lance automatiquement
Semgrep** sur le fichier modifie apres le linter natif, si `semgrep`
est dans le PATH et `.semgrep/sbfb.yml` existe. Seulement les findings
WARNING/ERROR bloquent (exit 2). Les findings INFO (comme
sbfb-iroh-endpoint-pin sur le code actuel) sont affiches mais non-
bloquants.

**Etat actuel du scan sur nexus master** :
- 0 finding WARNING (toutes les 6 regles WARNING respectees)
- 1 finding INFO sbfb-iroh-endpoint-pin sur
  `crates/nexus-core-rs/src/node.rs:243` — attendu, preventive,
  disparaitra quand Sprint 18 Phase C introduira le pin explicite

**7 regles livrees, 1 documentee en TODO** :

| ID | Langue | Severity | Status |
|---|---|---|---|
| sbfb-no-todo-macros-rust | Rust | WARNING | 0 findings |
| sbfb-no-placeholder-console-frontend | TS/JS | WARNING | 0 findings |
| sbfb-ignore-requires-reason-rust | Rust | WARNING | 0 findings |
| sbfb-ignore-requires-reason-python | Python | WARNING | 0 findings |
| sbfb-canonical-bytes-jcs | Rust | WARNING | 0 findings (avec exclusion `#[cfg(test)]`) |
| sbfb-zip-by-index-requires-validation | Rust | WARNING | 0 findings |
| sbfb-iroh-endpoint-pin | Rust | **INFO** | 1 finding preventive attendu |
| sbfb-loopback-peer-creds | Rust | — | TODO (necessite flow analysis axum middleware) |

**Regle retiree 2026-04-15** : `sbfb-project-announcement-repo-url`
produisait 6 faux positifs sur code correct (guard clauses HTTP +
builders chaines avec `with_provenance_hash` intermediaire). Le
coordinator Python + la validation daemon-side (http.rs:483-494)
enforcent deja la coherence. Documentee en TODO dans `.semgrep/sbfb.yml`
pour reouverture si flow analyzer Rust disponible (Creusot/Kani).

### 3.3 TDD Guard (optionnel, opt-in)

**Fichiers** :
- `.claude/hooks/tdd-guard-wrapper.sh` (committed) — wrapper gracieux
- `.claude/tdd-guard/data/config.json` (committed) — config nexus

**Role** : [nizos/tdd-guard](https://github.com/nizos/tdd-guard) est un
outil tiers qui enforce le cycle TDD (red-green-refactor) en bloquant
l'ecriture de code d'implementation si aucun test rouge n'existe
pre-alablement.

**Par defaut : DESACTIVE** (`guardEnabled: false` dans config.json).
Le wrapper no-op si `tdd-guard` n'est pas installe, donc zero friction
pour qui ne veut pas l'activer.

**Activation** (une fois par machine) :

```bash
# 1. Installer le CLI principal
npm install -g tdd-guard

# 2. Installer les reporters pour les test runners nexus
cargo install tdd-guard-rust                # Rust (cargo test)
pip install tdd-guard-pytest                # Python (pytest)

# 3. Dans Claude Code, activer pour la session :
#    (tape cette commande dans la conversation Claude)
/tdd-guard enable

# 4. (optionnel) Persister l'activation en editant config.json
#    .claude/tdd-guard/data/config.json : "guardEnabled": true
```

**Opt-out Phase A** (skeleton sans tests autorise) :

```
/tdd-guard disable    # au debut de Phase A
# ... code ...
/tdd-guard enable     # avant Phase B
```

**IgnorePatterns** nexus-specific deja configures :
`*.md, *.toml, .planning/**, docs/**, .github/**, scripts/**,
tests/ci-smoke/**, examples/**, **/migrations/**`

(liste complete dans `.claude/tdd-guard/data/config.json`)

**Hook config** : 4 hooks declares dans `.claude/settings.json` via
le wrapper :
- PreToolUse `Write|Edit|MultiEdit|TodoWrite` — valide avant write
- PostToolUse `Write|Edit|MultiEdit` — update state apres write
- UserPromptSubmit — intercepte `/tdd-guard enable|disable|status`
- SessionStart `startup|resume|clear` — clear transient data

Tous les 4 pointent sur le meme wrapper qui no-op si tdd-guard pas
installe, donc zero overhead pour qui ne l'active pas.

**Trade-off** : TDD Guard ajoute de la friction reelle (chaque
Edit/Write valide). Recommande seulement sur les phases impl
(Phase B-E) quand la discipline TDD vaut le cout. La couche 3
phase-auditor reste suffisante pour attraper les violations moins
aggressivement.

---

## 4. Couche 2 — Skills qualite

### 4.1 Trail of Bits — quand utiliser quoi

Une fois cloned (cf. §2.3), ces skills sont accessibles via le
Skill tool dans Claude Code.

#### Audit gate Phase 0 (cf. [`README.md`](README.md) §8)

| Skill | Usage |
|---|---|
| `differential-review` | Review security-focused du diff du sprint precedent (commit stack). Evidence-based, risk-first (auth/crypto/value transfer), genere un rapport markdown. |
| `static-analysis` | CodeQL + Semgrep + parsing SARIF pour triage. Utile quand l'audit track "outillage" demande un scan profond. |
| `supply-chain-risk-auditor` | Audit threat landscape des dependances (pertinent Sprint 18+ supply chain hardening). |
| `fp-check` | False positive verification systematique avec gate reviews. Evite de remonter un P0 imaginaire. |

#### Pendant un sprint (phases A-F)

| Skill | Usage |
|---|---|
| `semgrep-rule-creator` | Ecrire une regle Semgrep custom SBFB quand on detecte un anti-pattern recurrent. |
| `sharp-edges` | Identifier les API footgun. Utile quand on integre une nouvelle crate (iroh 0.98, wasmtime bump, etc.). |
| `insecure-defaults` | Detecte hardcoded creds, fail-open configs, defaults dangereux. Utile avant chaque release. |
| `spec-to-code-compliance` | Verifier que le code implemente bien le spec (ex: ProjectAnnouncement v5 matche les champs documentes). |

#### Research / deep-dive

| Skill | Usage |
|---|---|
| `audit-context-building` | Construire un contexte architectural profond pour un track d'audit lourd. |
| `variant-analysis` | Chercher des vulns similaires cross-codebase (pattern-based). Utile post-CVE iroh/wasmtime. |
| `property-based-testing` | Generer des tests PBT Hypothesis pour les parsers canoniques (`nexus-coordinator` JCS). |
| `mutation-testing` | Valider la solidite des tests existants via mutations. |

### 4.2 Skill nexus-phase-review (a venir, etape 3)

`~/.claude/skills/nexus-phase-review.skill/` (user-level). Orchestre la
checklist §7.4 avant commit d'une phase :
1. Rejoue toutes les suites pertinentes
2. Relit le draft commit body
3. Valide format `feat(scope): Sprint N Phase X — titre`
4. Verifie coherence delta tests annonce vs delta reel

---

## 5. Couche 3 — Subagent review intra-sprint

Gap actuel : l'audit gate review **entre** les sprints. Rien ne review
**entre les phases** A→B→C→D→E→F. Un blind-spot Phase B decouvert
Phase F coute 4 commits a defaire.

### 5.1 Agent `nexus-phase-auditor`

**Fichier** : `.claude/agents/nexus-phase-auditor.md` (project-level,
committed via `.claude/agents/`)

**Invocation** via Task tool — **template obligatoire** (l'omission de
la clause Write a déjà causé Sprint 19 Phase E un échec : agent a
produit un rapport en stdout sans écrire le fichier, hook a bloqué) :

```
Task(subagent_type="nexus-phase-auditor",
     prompt="Audit Sprint {N} Phase {X}.
             ECRIRE OBLIGATOIREMENT via Write tool dans
             .planning/active/sprint{N}_phase_{X}_review.md AVANT de
             retourner. Stdout ne suffit pas — le hook
             phase-auditor-gate.sh bloque sans le fichier sur disque.
             Si l'agent ne Write PAS, l'executeur n'a PAS l'autorisation
             de transcrire le rapport lui-meme (defait l'independance G4).
             Draft commit body: <coller body integral>")
```

**Ne JAMAIS utiliser** le template court "Audit Sprint X Phase Y" sans
la clause Write obligatoire — il replique la faiblesse historique.

L'agent review 5 dimensions en parallele sur le diff courant :
1. **Security** — Semgrep + patterns sensibles (secrets, path traversal,
   unsafe Rust, loopback sans peer-creds, wire format sans JCS canonique)
2. **Patterns** — diff vs `docs/rust/PATTERNS.md` + `docs/shell/PATTERNS.md`
3. **Scope-cuts** — grep diff vs `sprint{N}_kickoff.md` §6 (tout match = P1)
4. **Tests-delta** — delta annonce vs mesure reelle
5. **Research-grounding** — deps/APIs externes touchees par le diff
   sont tracees dans `sprint{N}_plan.md` §Research consulte via
   context7/WebSearch. Absence de trace sur API crypto ou spec
   standardisee = P0 (risque hallucination, CVE, API obsolete).

Produit `.planning/active/sprint{N}_phase_{X}_review.md` avec verdict
PASS | CONCERN | FAIL, listes P0/P1/P2/P3, et recommendation.

Convention (meme que audit gate Phase 0) : l'agent ne lit PAS
PATTERNS.md avant d'avoir forme son opinion sur chaque pattern — il
challenge, il ne ratifie pas.

### 5.2 Hook `phase-auditor-gate.sh`

**Fichier** : `.claude/hooks/phase-auditor-gate.sh` (committed)

**Role** : PreToolUse matcher `Bash` qui intercepte `git commit` et
refuse si le commit est un Phase commit (match `(feat|fix|docs|chore|test)
(sprint{N}).*Phase X`) ET le `sprint{N}_phase_{X}_review.md` n'existe
pas, ou son verdict n'est pas `PASS`.

**Fail-open** pour :
- Commits hors scope sprint (chore(claude), hotfixes, Merge, Revert)
- cwd != nexus (check Cargo.toml + crates/nexus-core-rs)
- Bypass d'urgence via env var : `NEXUS_SKIP_PHASE_AUDITOR=1 git commit ...`

**Fail-closed** (exit 2, bloque le commit avec message visible) pour :
- Phase commit sans review.md
- Phase commit avec review.md mais verdict != PASS

**Activation** : automatique via `.claude/settings.json` du repo qui
declare le hook PreToolUse Bash. Toute session Claude Code ouverte
dans nexus herite.

**Rationale** : l'audit gate inter-sprint catche les blind-spots en
post-hoc. Ce hook les catche en pre-commit. Les deux sont
complementaires — audit gate = sweep complet (3h+, session fraiche),
phase-auditor = sweep par-phase (20-30 min, dans la meme session).

**Flow recommande** par phase :
1. Implementer la phase
2. Invoquer le skill `nexus-phase-review` (couche 2) pour la
   verification §7.4 + format commit body
3. Invoquer l'agent `nexus-phase-auditor` (couche 3) pour la review
   independante 4 dimensions — produit `sprint{N}_phase_{X}_review.md`
4. `git commit -m "feat(sprintN): Sprint N Phase X — ..."` — le hook
   verifie le review et soit laisse passer, soit bloque

---

## 6. Couche 5 — Observability + memory hygiene

### 6.1 Post-commit memory updater

**Fichier** : `.claude/hooks/post-commit-memory.sh` (committed, partage)

**Role** : Git post-commit hook qui detecte les commits lies a un sprint
(`feat|fix|docs|chore|test(sprint{N})`) et met a jour en place :
- `memory/nexus_grid_pivot.md` frontmatter ligne `Tip \`<sha>\``
- `memory/MEMORY.md` ligne `SBFB pivot` (si la ligne mentionne le old tip)

**Ne fait pas** (volontaire, reste manuel) :
- Update du texte riche de la description (Phase livree, nouveaux tests,
  etc.) — le script ne fait QUE le SHA, la narration reste sous controle
  humain
- Ajout de lignes dans SPRINT_LOG.md
- Any update si commit != sprint scope (chore(claude), Merge, Revert)

**Idempotent** : re-run sur meme commit = no-op (old == new tip).

**Fail-safe** : si memory absente (nouveau clone, CI), warning silencieux,
exit 0. Ne bloque jamais le commit.

**Installation** : une seule commande par clone :

```bash
# Depuis la racine du repo
ln -sf "$PWD/.claude/hooks/post-commit-memory.sh" .git/hooks/post-commit
chmod +x .git/hooks/post-commit

# Ou si ln -s pose probleme sur Windows, wrapper direct :
cat > .git/hooks/post-commit <<'EOF'
#!/usr/bin/env bash
exec bash "$(git rev-parse --show-toplevel)/.claude/hooks/post-commit-memory.sh"
EOF
chmod +x .git/hooks/post-commit
```

**Test** :

```bash
# Simuler un commit sprint sans en faire un :
bash .claude/hooks/post-commit-memory.sh
# Exit 0 silencieux si le commit courant n'est pas sprint scope.
# Output "memory updated: Tip X -> Y (sprint-commit)" si c'etait
# un sprint commit et que le tip a change.
```

**Rationale** : l'etape §7.5 de README.md dit "mettre a jour la memory
avant de fermer la session". C'est une etape humaine oubliable. Ce hook
la rend automatique pour la partie la plus critique (tip SHA) sans
enlever le jugement humain sur la narration.

### 6.2 Statusline enrichi

**Fichier** : `.claude/hooks/nexus-statusline.js` (committed, partage)

**Role** : Override project-level du statusline Claude Code qui prefixe
le statusline GSD user-level avec le contexte sprint nexus :

```
[S18/B ⚠drift] <model> | <task> | <dirname> <context_bar>
  │   │   │
  │   │   └── warning jaune si memory tip != HEAD (drift)
  │   └────── phase courante detectee depuis le dernier commit Phase X
  └────────── sprint courant detecte depuis .planning/active/
```

**Composition** :
- Parse `.planning/active/sprint{N}_*.md` pour extraire N
- Grep `git log -20 --format=%s` pour trouver dernier `Phase X`
- Compare `HEAD` vs `memory/nexus_grid_pivot.md` Tip -> flag drift
- Delegue a `~/.claude/hooks/gsd-statusline.js` (spawnSync) pour le
  bloc model/task/dir/context bar existant

**Fallback** : si cwd n'est pas le repo nexus (absence Cargo.toml +
crates/nexus-core-rs), output = GSD statusline brut sans prefix.

**Activation** : automatique via `.claude/settings.json` du repo :

```json
"statusLine": {
  "type": "command",
  "command": "node \"${CLAUDE_PROJECT_DIR}/.claude/hooks/nexus-statusline.js\""
}
```

Claude Code fusionne les settings user + project, avec project qui
override sur les champs scalaires. Donc quand tu ouvres nexus, le
statusline bascule auto sur nexus-statusline.js. Quand tu ouvres
un autre projet, tu retombes sur gsd-statusline.js user-level.

**Rationale** : awareness permanent du contexte sprint. Evite de
confondre la phase courante ou de committer sans realiser qu'il y a
un drift memory a rattraper via update manuel post-sprint (§7.5
README.md).

---

## 7. Couche skippee — Multi-modele / second opinion

Decision utilisateur 2026-04-15 : **ne pas activer** de multi-modele
adversarial (Claude vs GPT/Gemini) sur nexus-grid. Cout eleve (3 API
simultanees) non justifie par le gain sur ce workflow sprint
discipline.

Alternative deja en place :
- Audit gate session fraiche (meme modele Claude, mais sans contexte
  historique -> effet "second opinion" natif)
- D1..D5 figes au kickoff (force la reflexion design en amont)

---

## 8. Quick reference — activer le tooling

```bash
# 1. Ouvrir nexus avec Claude Code
cd path/to/nexus
claude

# 2. Le hook verify-on-write est actif automatiquement via
#    .claude/settings.json committed dans le repo.
#    Verifier avec :
echo '{"tool_input":{"file_path":"docs/claude/README.md"}}' \
  | bash .claude/hooks/verify-on-write.sh && echo OK

# 3. Installer Trail of Bits skills (user-level, une fois par machine)
if [ -d ~/.claude/skills/trailofbits ]; then
  git -C ~/.claude/skills/trailofbits pull
else
  git clone --depth 1 https://github.com/trailofbits/skills.git \
    ~/.claude/skills/trailofbits
fi

# 4. Dans Claude Code, acceder aux skills :
#    /help         -> voir les skills disponibles
#    (les skills Trail of Bits s'invoquent via le Skill tool
#     avec le nom exact : "differential-review", etc.)
```

---

## 9. Evolution

Cette doc evolue avec chaque nouvelle couche. Historique des
ajouts :

**Session 2026-04-15** — meta-chantier process Claude Code v1
(hors cycle Sprint 18) :

| Commit | Scope | Livrable |
|---|---|---|
| `4f0306b` | chore(claude) | Couche 1 — hook `verify-on-write.sh` (Rust/Py/TS linter scoped) + .gitignore unignore .claude/ |
| `8769047` | chore(claude) | .gitignore simplifie + TOOLING.md initial (5 couches) |
| `db80335` | chore(claude) | Couche 2 — skill `nexus-phase-review` (verification §7.4 + format commit body) |
| `9f38367` | chore(claude) | Couche 5 — hook `post-commit-memory.sh` (bump tip SHA auto sur sprint commits) |
| `87512e9` | chore(claude) | Couche 5 — `nexus-statusline.js` enrichi (sprint/phase + drift warning) |
| `7ea8041` | chore(claude) | Couche 1 — 4 regles Semgrep SBFB (todo macros / placeholder console / ignore without reason) |
| `119de3a` | chore(claude) | Couche 3 — agent `nexus-phase-auditor` + hook `phase-auditor-gate.sh` (bloque Phase commit sans review PASS) |
| `0e8d49a` | chore(claude) | Couche 1 — TDD Guard opt-in (wrapper gracieux + config ignorePatterns + 4 hooks registered) |
| (ce commit) | chore(claude) | `scripts/install-claude-tooling.sh` idempotent + TOOLING.md §9 recap |

Couche 4 (multi-modele / second opinion) **skippee** par decision
utilisateur 2026-04-15 — cost eleve non justifie, audit gate session
fraiche couvre le besoin.

**TODO architecturales** (ouverture de nouvelles ameliorations
quand le besoin se presentera) :
- 5 regles Semgrep SBFB architecturales (JCS, PeerCreds, iroh pin,
  repo_url, zip path traversal) -> a generer via
  `trailofbits/semgrep-rule-creator` avec le code en main
- Integration Semgrep dans `verify-on-write.sh` (scan scope au
  fichier apres linter natif, conditionne a `semgrep` dans PATH)
- Skill `challenge-d5` (pattern adversarial sur D1..D5 kickoff)
  -> etape 8 originale de l'ultraplan reclassee Couche 4 skippee
- MCP `claude-context` Zilliz (recherche semantique cross-codebase
  71K LOC) -> etape 10 originale reclassee Couche 4 skippee
