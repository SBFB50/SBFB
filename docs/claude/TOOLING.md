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

### 3.2 Semgrep rules SBFB (a venir, etape 6)

`.semgrep/sbfb.yml` + hook PostToolUse qui lance Semgrep scope au
fichier. Regles planifiees :

- `sbfb-canonical-bytes-jcs` : interdire `serde_json::to_string` pour
  wire format, exiger canonical bytes JCS
- `sbfb-loopback-peer-creds` : toute route loopback doit checker
  `PeerCredsVerified` marker
- `sbfb-iroh-endpoint-pin` : `iroh::Endpoint::builder()` doit utiliser
  pin discovery
- `sbfb-no-dead-code` : pas de `console.warn` / `unimplemented!()` /
  `todo!()` hors Phase F docs
- `sbfb-public-repo-url` : `ProjectAnnouncement` public -> `repo_url`
  obligatoire
- `sbfb-zip-path-traversal` : tout `zip::read_file` doit checker le
  path avant extraction

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

### 6.1 Post-commit memory updater (a venir, etape 4)

Git post-commit hook qui detecte `feat(sprint{N}) | docs(sprint{N}) | fix(sprint{N})`
et met a jour :
- `memory/nexus_grid_pivot.md` frontmatter `description:` -> nouveau tip
- `memory/MEMORY.md` ligne `SBFB pivot` -> resume court avec tip

Objectif : zero session future ne demarre avec memory stale (le
pre-flight §7 de README.md le detecte mais c'est trop tard).

### 6.2 Statusline enrichi (a venir, etape 5)

`~/.claude/hooks/gsd-statusline.js` etendu pour afficher :
- Sprint + Phase en cours (parse `.planning/active/`)
- Verdict audit gate precedent
- Drift memory-vs-HEAD (warning si memory stale)
- TODOs non landed du sprint courant

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
