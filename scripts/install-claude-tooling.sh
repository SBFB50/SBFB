#!/usr/bin/env bash
#
# scripts/install-claude-tooling.sh
#
# Installe tous les composants optionnels du process tooling Claude Code
# sur nexus-grid. Idempotent : re-run safe.
#
# Cf. docs/claude/TOOLING.md pour la description complete de chaque
# composant.
#
# Usage :
#   bash scripts/install-claude-tooling.sh             # interactive
#   bash scripts/install-claude-tooling.sh --yes       # skip prompts
#   bash scripts/install-claude-tooling.sh --minimal   # only core
#
# Que fait le script :
#   1. Install le git post-commit hook (memory updater)
#   2. Clone Trail of Bits skills en user-level (~/.claude/skills/)
#   3. (optionnel) Install Semgrep via pip
#   4. (optionnel) Install TDD Guard via npm + cargo + pip
#
# Ne touche PAS :
#   - .claude/settings.json (committed, auto-chargé par Claude Code)
#   - .claude/hooks/*.sh (committed, lances via settings)
#   - .claude/agents/*.md (committed)
#   - .claude/skills/*.md (committed)
#   - .semgrep/sbfb.yml (committed)
#   Ces fichiers sont deja dans le repo, rien a installer.

set -euo pipefail

# ------- Arguments -------
INTERACTIVE=1
MINIMAL=0
for arg in "$@"; do
  case "$arg" in
    --yes|-y) INTERACTIVE=0 ;;
    --minimal) MINIMAL=1 ;;
    --help|-h)
      sed -n '2,30p' "$0"
      exit 0
      ;;
  esac
done

# ------- Helpers -------
prompt_yn() {
  local msg="$1"
  local default="${2:-y}"
  if [ "$INTERACTIVE" -eq 0 ]; then
    [ "$default" = "y" ] && return 0 || return 1
  fi
  local suffix="[Y/n]"
  [ "$default" = "n" ] && suffix="[y/N]"
  read -r -p "$msg $suffix " ans
  ans="${ans:-$default}"
  [[ "$ans" =~ ^[Yy]$ ]]
}

info()  { echo -e "\033[36m[install-claude-tooling]\033[0m $1"; }
ok()    { echo -e "\033[32m[✓]\033[0m $1"; }
warn()  { echo -e "\033[33m[!]\033[0m $1"; }
error() { echo -e "\033[31m[✗]\033[0m $1" >&2; }

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Sanity check
if [ ! -f "$REPO_ROOT/Cargo.toml" ] || [ ! -d "$REPO_ROOT/crates/nexus-core-rs" ]; then
  error "Not in nexus repo root (no Cargo.toml + crates/nexus-core-rs)"
  exit 1
fi

info "Installing Claude Code process tooling for nexus-grid"
info "Repo root: $REPO_ROOT"
echo

# ------- Core (always) -------

# 1. Git hooks (Claude hooks in .claude/hooks/ are the active backstop)
info "Step 1/4: Git hooks check"
info "  Claude hooks in .claude/hooks/ are the active backstop."
info "  No git post-commit hook needed."
echo

# ------- Optional: Trail of Bits skills -------

info "Step 2/4: Trail of Bits skills (user-level)"

TOB_DIR="$HOME/.claude/skills/trailofbits"
if [ -d "$TOB_DIR" ]; then
  if prompt_yn "Update existing clone at $TOB_DIR?"; then
    (cd "$TOB_DIR" && git pull --rebase --quiet)
    ok "Updated Trail of Bits skills"
  else
    ok "Trail of Bits skills already installed (skipped update)"
  fi
else
  if prompt_yn "Clone Trail of Bits skills to $TOB_DIR?"; then
    mkdir -p "$HOME/.claude/skills"
    git clone --depth 1 https://github.com/trailofbits/skills.git "$TOB_DIR"
    ok "Cloned Trail of Bits skills"
  else
    warn "Skipped — couche 2 skill mapping not available (see TOOLING.md §4.1)"
  fi
fi
echo

if [ "$MINIMAL" -eq 1 ]; then
  info "Minimal mode: skipping optional tool installs (Semgrep, TDD Guard)"
  exit 0
fi

# ------- Optional: Semgrep -------

info "Step 3/4: Semgrep (custom SBFB rules)"

if command -v semgrep >/dev/null 2>&1; then
  SEMGREP_VER=$(semgrep --version 2>&1 | head -1)
  ok "Semgrep already installed ($SEMGREP_VER)"
else
  if prompt_yn "Install Semgrep via pip?"; then
    if command -v pip >/dev/null 2>&1; then
      pip install --user semgrep
      ok "Semgrep installed via pip"
    elif command -v pip3 >/dev/null 2>&1; then
      pip3 install --user semgrep
      ok "Semgrep installed via pip3"
    else
      error "Neither pip nor pip3 found — install Python first"
    fi
  else
    warn "Skipped — .semgrep/sbfb.yml rules are committed but won't run automatically"
  fi
fi
echo

# ------- Optional: TDD Guard -------

info "Step 4/4: TDD Guard (opt-in, discipline TDD stricte)"

if command -v tdd-guard >/dev/null 2>&1; then
  ok "TDD Guard already installed ($(tdd-guard --version 2>/dev/null || echo 'version unknown'))"
else
  if prompt_yn "Install TDD Guard (npm + reporters)?" "n"; then
    if command -v npm >/dev/null 2>&1; then
      npm install -g tdd-guard
      ok "Installed tdd-guard (npm)"
    else
      error "npm not found — install Node.js first"
    fi

    if command -v cargo >/dev/null 2>&1; then
      cargo install tdd-guard-rust
      ok "Installed tdd-guard-rust"
    fi

    if command -v pip >/dev/null 2>&1; then
      pip install --user tdd-guard-pytest
      ok "Installed tdd-guard-pytest"
    fi

    info "TDD Guard installe mais DESACTIVE par defaut (guardEnabled: false)."
    info "Pour activer dans une session : /tdd-guard enable"
    info "Pour persister : editer .claude/tdd-guard/data/config.json"
  else
    ok "Skipped — TDD Guard wrapper will no-op silently"
  fi
fi
echo

# ------- Summary -------

info "Install terminee. Recapitulatif :"
echo
echo "  Ce qui est actif automatiquement (via .claude/settings.json committed) :"
echo "    - Hook verify-on-write (Rust clippy / Python ruff / TS eslint)"
echo "    - Hook phase-auditor-gate (bloque Phase commit sans review PASS)"
echo "    - Statusline nexus (sprint/phase + drift memory)"
echo
echo "  Agent / skill disponibles via Task / Skill tool :"
echo "    - Agent nexus-phase-auditor (review intra-sprint 4 dimensions)"
echo "    - Skill nexus-phase-review (verification §7.4 + format commit body)"
echo
echo "  Git hook installe :"
echo "    - post-commit memory updater (bump tip SHA sur sprint commits)"
echo
if [ -d "$TOB_DIR" ]; then
  echo "  Trail of Bits skills disponibles : oui (~/.claude/skills/trailofbits/)"
  echo "    Usage : cf. docs/claude/TOOLING.md §4.1"
else
  echo "  Trail of Bits skills : NON installes (bash scripts/install-claude-tooling.sh pour y remedier)"
fi
if command -v semgrep >/dev/null 2>&1; then
  echo "  Semgrep : oui"
  echo "    Scan manuel : semgrep --config .semgrep/sbfb.yml crates/ packages/ web/src/"
else
  echo "  Semgrep : NON installe"
fi
if command -v tdd-guard >/dev/null 2>&1; then
  echo "  TDD Guard : oui (desactive par defaut, /tdd-guard enable pour activer)"
else
  echo "  TDD Guard : NON installe (wrapper no-op silent)"
fi
echo
ok "Done. Voir docs/claude/TOOLING.md pour les details."
