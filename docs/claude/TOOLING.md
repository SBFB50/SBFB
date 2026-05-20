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

## 1. Vue d'ensemble — 4 couches actives

Le process empile 4 couches actives. Chacune catch une classe
d'erreurs differente.

| # | Couche | Moment | Outil principal |
|---|---|---|---|
| 1 | Garde-fous automatiques | PostToolUse (chaque write) | `.claude/hooks/verify-on-write.sh` + Semgrep |
| 2 | Supervision continue + plan contexte | Session entiere + chaque gate | Agent Team teammate `nexus-process-supervisor` + task list + Stop hook |
| 3 | Skills qualite specialises | Sur demande Claude | Trail of Bits skills + `nexus-phase-review` |
| 4 | Subagent review intra-sprint | Pre-commit d'une phase | `nexus-phase-auditor` agent (inconditionnel) |

L'**audit gate inter-sprint** (cf. [`README.md`](README.md) §3) reste
la couche de reference. Ce tooling la complete sans la remplacer.

---

## 2. Installation

### 2.1 Prerequis

- Claude Code >= 2.1 (hooks `PostToolUse` / `PreToolUse`, `Stop`, `matcher`, stdin JSON)
- Claude Code avec Agent Teams si disponible (mode prefere). Le repo active
  `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` dans `.claude/settings.json`.
  Si la version locale ne supporte pas encore Agent Teams, le process bascule
  en consultation Agent gate-check a chaque gate.
- Bash >= 4 (Git Bash sur Windows fonctionne)
- `jq` dans le PATH **OU** `python3` (les hooks detectent auto lequel utiliser
  pour parser l'input JSON — fail-open silent si aucun des deux n'est present)
- `cargo`, `uv`, `npm` installes (pour les linters scope au langage)
- `cargo-nextest` installe : `cargo install cargo-nextest --locked`
  (test runner process-per-test, config [`.config/nextest.toml`](../../.config/nextest.toml),
  commandes standard cf. [`README.md`](README.md) §4.3 et §7.4)
### 2.2 Hooks local au repo (automatique)

Rien a installer. Le fichier `.claude/settings.json` est committe dans
le repo, donc toute session Claude Code ouverte dans nexus herite
automatiquement :

- de `CLAUDE_CODE_EXPERIMENTAL_AGENT_TEAMS=1` pour permettre le mode
  superviseur long-lived quand Claude Code le supporte ;
- de `teammateMode: in-process`, plus robuste sur Windows qu'une dependance
  tmux/split-pane ;
- du hook `PostToolUse` qui execute `.claude/hooks/verify-on-write.sh` ;
- du hook `Stop` qui execute `.claude/hooks/process-supervisor-stop.sh` ;
- des hooks `TaskCreated` / `TaskCompleted` qui executent
  `.claude/hooks/process-task-gate.sh` ;
- du hook `TeammateIdle` qui execute
  `.claude/hooks/process-teammate-idle.sh`.

Verifier que ca marche :

```bash
echo '{"tool_input":{"file_path":"docs/claude/README.md"}}' \
  | bash .claude/hooks/verify-on-write.sh && echo "hook OK"
```

Sortie attendue : `hook OK` (le hook no-op sur docs/).

Verifier le Stop hook :

```bash
echo '{"last_assistant_message":"Fait, worktree clean"}' \
  | bash .claude/hooks/process-supervisor-stop.sh
```

Si le worktree est sale, la sortie attendue est un JSON `decision: block`.
Si le worktree est propre ou que le message ne ressemble pas a une conclusion,
le hook no-op.

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

### 2.6 Superviseur long-lived et plan contexte

Le process prefere maintenant un superviseur permanent, mais seulement quand
la surface Claude Code le permet factuellement :

- **Mode prefere** : Agent Team avec teammate `supervisor` de type
  `nexus-process-supervisor`. Le teammate reste actif pendant le contexte,
  voit la task list partagee, peut communiquer avec le lead, et envoie un
  `BLOCK-*` proactif si le process derive.
- **Mode degrade** : subagent classique `Agent(...)` re-invoque a chaque gate.
  Ce mode est requis si Agent Teams est absent, si le teammate est `Done`, ou
  si la session ne peut plus lui envoyer de message.
- **Backstop automatique** : les hooks ne remplacent pas le superviseur, mais
  ils donnent du feedback mecanique si le modele oublie. `Stop` bloque les fins
  de tour qui ressemblent a un faux "termine" avec worktree sale ou a un debut
  Phase C/SBFB factory sans preflight G8. `TaskCreated` / `TaskCompleted`
  bloquent les tasks de gate terminees sans artefact. `TeammateIdle` garde le
  teammate `supervisor` actif tant que le worktree est sale.

Le bootstrap [`README.md`](README.md) impose aussi un plan sequentiel visible
dans le contexte principal (`TaskCreate`/`TaskUpdate`/`TaskList`, fallback
`TodoWrite`). Le superviseur surveille cette task list : exactement une tache
`in_progress`, aucune tache `completed` sans artefact, et gates dans l'ordre.

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

## 4. Couche 3 — Skills qualite

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

## 5. Couche 4 — Subagent review intra-sprint

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

### 5.2 Hook `phase-auditor-gate.sh` (audit inconditionnel)

**Fichier** : `.claude/hooks/phase-auditor-gate.sh` (committed)

**Role** : PreToolUse matcher `Bash` qui intercepte `git commit` et
refuse si le commit est un Phase commit (match `(feat|fix|docs|chore|test)
(sprint{N}).*Phase X`) ET le `sprint{N}_phase_{X}_review.md` n'existe
pas ou son verdict n'est pas `PASS`.

**Audit inconditionnel** : tout commit Phase déclenche la vérification.
L'amendement C1-C9 conditionnel (2026-04-20, `34dacdc`) a été retiré
après le faux négatif S23 P1 C-1 (`redundancy_factor` dans canonical
bytes, `task.rs` non matché par regex C1 sur `canonical|schemas/`).
Cf. `.planning/research/S24_process_review_2026-04-21.md §1.2`.

**Hook complementaire** `.claude/hooks/phase-precommit-lightcheck.sh`
(declare en 2eme position PreToolUse Bash apres auditor-gate) applique
3 verifications legeres systematiques sur tout `git commit` :

1. **Coherence staging (STRICT, BLOCK)** — pour chaque `+pub mod X;` Rust
   ajoute, le file `<dir>/X.rs` ou `<dir>/X/mod.rs` doit exister + etre
   staged ou tracked.
2. **Refs fichiers body (WARN)** — pour chaque path cite dans le commit
   body, verifier que le file existe dans le repo.
3. **LOC deviation (WARN)** — si body cite `~XXX LOC` et diff stat reel
   > 2.5x, demander mention explicite.

**Fail-open** pour :
- Commits hors scope sprint (chore(claude), hotfixes, Merge, Revert)
- cwd != nexus (check Cargo.toml + crates/nexus-core-rs)

**Fail-closed** (exit 2, bloque le commit) pour :
- Phase commit ET review.md absent OU verdict final non exactement PASS
- Review encore en PASS-PENDING ou Codex EN ATTENTE
- Scope sprint du commit different du sprint du titre
- `git diff --cached --check` non propre
- Lightcheck Check 1 staging incoherence detectee

Le bypass env `NEXUS_SKIP_PHASE_AUDITOR=1` a ete retire en S67. Un
`git commit --no-verify` manuel est un incident process a documenter et a
resoudre avant de declarer la phase propre.

**Flow recommande par phase** :

1. Implementer la phase
2. Invoquer le skill `nexus-phase-review` (couche 2) pour la
   verification §7.4 + format commit body
3. Invoquer l'agent `nexus-phase-auditor` (couche 3) pour la review
   independante — produit `sprint{N}_phase_{X}_review.md`
4. `git commit` — le hook verifie review.md + lightcheck

---

## 6. Quick reference — activer le tooling

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

## 7. Evolution

**S24 process review** (`466f826`, 2026-04-21) : drop couche 4
(multi-modele) et couche 5 (hooks memory/statusline supprimes
en `2438c59`). Renumerate 5→3 couches actives. Drop C1-C9
conditionnel, retour audit inconditionnel.

**TODO architecturales** :
- Semgrep rule `sbfb-canonical-bytes-jcs` (detection semantique
  structs participant a `canonical_bytes()`)
- Skill `challenge-d5` (pattern adversarial D1..D5 kickoff)
