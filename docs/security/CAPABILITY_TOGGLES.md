---
written: 2026-04-20  # S22 hors-sprint post Phase B `e9530c2`
last_validated: 2026-04-20
status: implemented (Sprint 25 Phase D)
triggers_revalidate:
  - "microsoft/sudo Settings UI change"
  - "Tool-calling sandbox reactivation S25+ effective"
  - "MCP spec revision Anthropic (impacte mcp_server_expose capability)"
  - "Integrity Level API breaking change Windows"
---

# Capability toggles — opt-in OS-level pattern

## 1. Scope et motivation

Ce document spécifie le pattern **capability toggles gate-off-by-
default** adopté de `microsoft/sudo` (entièrement OFF jusqu'à
activation explicite Settings → System → Advanced → Developer
Features). Feature D5 du cluster D dans
`.planning/research/S23_to_S29_agents_sudo_integration_matrix.md`.

**Principe** : toute capability à surface attaque augmentée est
désactivée par défaut. Activation requiert **admin privilege
OS-level** (pas juste edit filesystem) via binaire séparé
`nexus-admin`. Le daemon vanilla ne sert **aucune** capability
optionnelle tant qu'activation explicite.

**Symétrie vs toggle applicatif** : un flag config dans un fichier
éditable user-land (`~/.sbfb/config.toml`) peut être modifié par
malware user-mode. Un toggle `capabilities.toml` édité via `nexus-
admin` avec check admin privilege requiert escalade OS (runas
admin / sudo / elevated console) = malware user-mode sans privilege
escalation ne peut pas bypass.

## 2. Inventaire capabilities

Table des capabilities actuellement identifiées. Statut par défaut
**tout OFF pre-v1.0** sauf indication.

| Capability | Consumer sprint | Default | Justification gate-off |
|---|---|---|---|
| `tool_calling` | S25 RAG + tool registry | OFF | Surface OWASP LLM06:2025 Excessive Agency. Déferré post-S25 per S22 scope cuts. |
| `streaming_bridge` | S25 C5 bridge `task_submit_streaming` | OFF | Nouveau wire P24 méthode whitelist, testing multi-browser requis |
| `mcp_server_expose` | S25 B2 MCP server | OFF | Expose bridge vers agents LLM externes = expansion canal d'accès mais surface MCP spec-evolution dep |
| `federation_canary` | S30 FROST Niveau 1 | OFF | Cosign cross-juridiction, requiert partnership OpSec S30+ |
| `rag_retrieval` | S25 RAG sanitization | OFF | Ingestion sources externes = injection vecteur |
| `biometric_gate` (T2 loopback) | LT-4 D4 | OFF | Cross-platform biometric Windows Hello / TouchID / polkit |
| `ephemeral_workers` | S23 | OFF (opt-in per-app AppManifest) | Restart after N tasks = overhead cold-start Ollama ~10-30s, acceptable uniquement T3+ apps |
| `task_scoped_sandbox` (C4) | S28 | OFF (opt-in per-app AppManifest) | Fresh iframe per-task = overhead Pyodide boot ~2s, acceptable T3+ apps |

Les capabilities `ephemeral_workers` et `task_scoped_sandbox` sont
aussi **opt-in per-app AppManifest** (un publisher d'app T3+ peut
spécifier `requires_ephemeral_worker: true` ou `requires_task_scoped_
sandbox: true`). Le worker refuse alors la task si la capability
n'est pas activée côté consumer.

## 3. Format `capabilities.toml`

Fichier canonical : `~/.sbfb/capabilities.toml` (perm 0600 user-only).

```toml
# SBFB capabilities gate — gate-off by default pre-v1.0.
# Edit via `nexus-admin capability enable/disable <name>`.
# Direct edit = ignored by daemon (checksum SHA-256 mismatch =
# reject loading + log warn + fallback all-OFF).

version = 1  # pre-launch stable schema

[capability.tool_calling]
enabled = false
enabled_at = ""          # ISO 8601 UTC quand activée (trace audit)
enabled_by = ""          # OS username qui a activé (trace audit)

[capability.streaming_bridge]
enabled = false
enabled_at = ""
enabled_by = ""

[capability.mcp_server_expose]
enabled = false
enabled_at = ""
enabled_by = ""

[capability.federation_canary]
enabled = false
enabled_at = ""
enabled_by = ""

[capability.rag_retrieval]
enabled = false
enabled_at = ""
enabled_by = ""

[capability.biometric_gate]
enabled = false
enabled_at = ""
enabled_by = ""

# Checksum SHA-256 du fichier sans cette ligne (anti-tamper).
# Calculé par nexus-admin à chaque mutation, vérifié par daemon au load.
integrity_hash = "sha256-..."
```

**Anti-tamper simpliste** : le `integrity_hash` est un SHA-256 du
fichier sans la ligne hash. `nexus-admin` recalcule à chaque
mutation. Le daemon vérifie au load ; mismatch → reject + log +
fallback all-OFF. Un malware user-mode qui edit direct
`capabilities.toml` sans passer par `nexus-admin` sera détecté.

**Notes limites** : malware avec privilege escalation OS peut aussi
invoquer `nexus-admin` directement (admin privilege = game over).
Le gate protège contre user-mode malware sans priv-esc, pas contre
root/admin déjà compromis.

## 4. Binaire `nexus-admin` (Typer CLI Python)

Nouveau binaire Python séparé (pattern `sbfb quarantine` S21 Phase D
`f830579`) :

```
nexus-admin capability list              # affiche statut 8 capabilities
nexus-admin capability enable <name>     # active (requiert admin privilege OS)
nexus-admin capability disable <name>    # désactive
nexus-admin capability info <name>       # description + threat note + consumer sprint
nexus-admin audit-trail                  # print events capability_changed depuis install
```

### 4.1 Check admin privilege par OS

**Unix (Linux/macOS)** :

```python
import os
def require_admin_unix():
    if os.geteuid() != 0:
        raise PermissionError(
            "nexus-admin requires root privilege. Run with sudo."
        )
```

**Windows** :

```python
import ctypes
def require_admin_windows():
    is_admin = ctypes.windll.shell32.IsUserAnAdmin()
    if not is_admin:
        raise PermissionError(
            "nexus-admin requires elevated command prompt. Run as Administrator."
        )
    # Check Mandatory Integrity Level High (S-1-16-12288)
    # via GetTokenInformation(TokenIntegrityLevel) pour blocker
    # les ops depuis Medium IL quand même élevé (defense-in-depth).
    import win32security, win32con
    token = win32security.OpenProcessToken(
        win32security.GetCurrentProcess(),
        win32con.TOKEN_QUERY,
    )
    sid, _ = win32security.GetTokenInformation(
        token, win32security.TokenIntegrityLevel
    )
    rid = win32security.GetSidSubAuthority(sid, 0)
    if rid < 0x3000:  # SECURITY_MANDATORY_HIGH_RID
        raise PermissionError(
            "nexus-admin requires High Mandatory Integrity Level."
        )
```

## 5. Enforcement PR-block (Semgrep custom rule)

Préréquis pour garantir qu'aucune PR n'expose un endpoint sans
passer par le capability gate. Règle Semgrep custom dans
`.semgrep/capability_gate.yml` :

```yaml
rules:
  - id: require-capability-check
    patterns:
      - pattern-either:
          - pattern: |
              @app.post("/tool/$X")
              $FUNC
          - pattern: |
              @app.post("/rag/$X")
              $FUNC
          - pattern: |
              @app.post("/mcp/$X")
              $FUNC
      - pattern-not: |
          @app.post(...)
          @require_capability($CAP)
          $FUNC
    message: |
      Endpoint risqué sans `@require_capability(...)` decorator.
      Voir `docs/security/CAPABILITY_TOGGLES.md §5`.
    severity: ERROR
    languages: [python]
```

Le decorator Python `@require_capability("tool_calling")` :

```python
from functools import wraps
from fastapi import HTTPException

def require_capability(cap_name: str):
    def decorator(func):
        @wraps(func)
        async def wrapper(*args, **kwargs):
            if not capabilities_store.is_enabled(cap_name):
                raise HTTPException(
                    403,
                    detail=f"capability '{cap_name}' is disabled. "
                           f"Run `nexus-admin capability enable {cap_name}` to activate.",
                )
            return await func(*args, **kwargs)
        return wrapper
    return decorator
```

## 6. Intégration audit channel (A3 consumer)

Chaque mutation de capability émet un event via `nexus-events-core`
(A3 OS audit channel, S25 co-landing) :

```
event: capability_changed
capability: "tool_calling"
previous: false
new: true
actor_os_username: "alice"
actor_privilege_level: "root" | "admin" | "high_il"
timestamp: 2026-05-XX-...
integrity_hash_pre: "sha256-abc..."
integrity_hash_post: "sha256-def..."
```

Event visible :
- Windows : ETW provider `org.nexus-grid.sbfb` channel Security
- Linux : systemd-journald priority NOTICE unit `sbfb-daemon.service`
- macOS : unified logging subsystem `org.nexus-grid.sbfb` category
  `capability`

SIEM entreprise (Splunk/Sentinel) peut alerter sur pattern
`capability_changed` + `actor_privilege_level != root` (impossible
normalement, indique compromise admin).

## 7. Trigger d'intégration

**S22 Phase F** (maintenant, ce commit) : création de ce document
+ mention dans HARDENING §3 S23 amendement.

**S23 chore hors-sprint** (optionnel, pattern `88eee23`) : si
arbitrage user favorable, code `nexus-admin` skeleton + stub
`require_capability` decorator sans enforcement (no-op pour
préparer consumers S25).

**S25** (amendement HARDENING §3 S25) : implémentation complète
(Typer CLI + admin privilege check cross-OS + Semgrep rule +
A3 event emission).

**Consommateurs** :
- S25 tool_calling réactivation + MCP server expose + streaming
  bridge + RAG retrieval = tous gated `@require_capability(...)`.
- S30 federation canary FROST Niveau 1 = gated.
- LT-4 biometric gate = capability propre + consumer T2 loopback
  tier.

## 8. Références

- `.planning/research/S23_to_S29_agents_sudo_integration_matrix.md §1 Cluster D`
- `docs/security/HARDENING_ROADMAP.md §3 S25` (prérequis tool-calling)
- `docs/security/THREAT_MODEL.md §ADVERSARIES AD2` (malware user-mode)
- `docs/security/LOOPBACK_ENDPOINTS_TRUST_TIERS.md` (capability
  `biometric_gate` consumer T2)
- `packages/nexus-coordinator/src/nexus_coordinator/api/` (pattern
  FastAPI endpoints existant, base decorator)
- Source externe : [Microsoft Learn Sudo for Windows §Configuration](
  https://learn.microsoft.com/en-us/windows/sudo/#how-to-configure-sudo-for-windows)
  (pattern 3 modes + admin console requirement)
