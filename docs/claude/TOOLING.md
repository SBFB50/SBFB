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

- Claude Code >= 2.1 (hooks `PostToolUse`, `matcher`, stdin JSON)
- Bash >= 4 (Git Bash sur Windows fonctionne)
- `jq` dans le PATH (pour parser l'input stdin des hooks)
- `cargo`, `uv`, `npm` installes (pour les linters scope au langage)

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

Une fois les etapes 6-9 du roadmap livrees, `scripts/install-claude-tooling.sh`
sera l'entry point unique. En attendant, installer manuellement.

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

**Integration hook** (roadmap future) : `verify-on-write.sh` pourra
ajouter un step Semgrep scope au fichier modifie apres le linter
natif, si `semgrep` est dans le PATH. A implementer dans une phase
ulterieure.

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

## 5. Couche 3 — Subagent review intra-sprint (a venir, etape 7)

Gap actuel : l'audit gate review **entre** les sprints. Rien ne review
**entre les phases** A→B→C→D→E→F. Un blind-spot Phase B decouvert
Phase F coute 4 commits a defaire.

Livrable : agent `nexus-phase-auditor` (user-level) spawne 4 subagents
Task en parallele avant chaque `git commit feat(sprint{N}): Phase X` :
- security (Semgrep + path traversal + secrets)
- patterns (diff vs `docs/rust/PATTERNS.md` + `docs/shell/PATTERNS.md`)
- scope-cuts (grep diff vs `sprint{N}_kickoff.md` §6)
- tests-delta (delta annonce dans body commit vs delta reel)

Synthese dans `.planning/active/sprint{N}_phase_{X}_review.md`.
Hook PreToolUse sur `Bash(git commit...)` refuse si verdict != PASS.

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

- **2026-04-15** (`4f0306b..HEAD`) : Couche 1 hook verify-on-write +
  Couche 2 Trail of Bits skills + section TOOLING.md initiale
- Etapes a venir : Couche 1 Semgrep SBFB, Couche 2 nexus-phase-review,
  Couche 3 phase-auditor agent, Couche 5 memory hooks + statusline
