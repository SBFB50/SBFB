---
written: 2026-04-20  # S22 hors-sprint post Phase B `e9530c2`
last_validated: 2026-04-20
status: design-only (implementation S22 Phase F wrap-up + extension S25/S28/LT-4)
triggers_revalidate:
  - "microsoft/sudo new elevation mode release"
  - "Nouveau endpoint loopback risky ajoute daemon"
  - "OS biometric API breaking change (Windows Hello / LocalAuthentication / polkit)"
---

# Loopback endpoints — trust tiers

## 1. Scope et motivation

Ce document étend le durcissement loopback HTTP livré S16 Phase A
(`d7c265a` : bearer token X-SBFB-Token 256-bit + Host allowlist +
Origin check CVE-2025-49596 mitigation + UDS SO_PEERCRED Unix + Named
Pipe SDDL Windows) vers un **modèle 3-tiers de confiance par
endpoint**, inspiré du pattern `microsoft/sudo` 3 modes (`forceNewWindow
/ disableInput / normal`) qui documente explicite le trade-off
UX/sécurité par mode.

Le modèle actuel est **uniforme** : tous les endpoints loopback
partagent le même gate (bearer + peer creds). Ce qui est problématique
pour les **endpoints critiques** dont la compromission = attaque
terminale (panic wipe, duress unlock, rotation token force, escalade
consent tier).

Le pattern cible = chaque endpoint loopback déclare son **trust tier**
(requis pour être invoqué) + son **threat note** (résidu documenté
par tier).

## 2. Les trois tiers

| Tier | Nom | Requirement | UX | Résidu threat |
|---|---|---|---|---|
| T0 | `AUTO` | Bearer + peer creds | Transparent (aucune interaction user) | Malware user-mode avec browser compromise peut invoquer |
| T1 | `CONFIRM_PROMPT` | T0 + prompt UI explicit (nonce + TTL 30s) | Utilisateur clique "Confirmer" dans UI shell | Malware avec DOM injection peut cliquer automatic (mitigé par nonce ré-entrant) |
| T2 | `BIOMETRIC_GATE` | T1 + OS biometric prompt (Windows Hello / TouchID / polkit) | Utilisateur fait biométrie OS | Malware browser-level ne peut pas déclencher (gate OS-level) |

T0 = comportement courant S16 Phase A (tous les endpoints).
T1 = ajout S22 Phase F (CONFIRM_PROMPT) pour ops intermédiaires.
T2 = LT-4 post-v1.0 (cf. `docs/release/ROADMAP_COMMITMENTS.md LT-4`
pour D4 OS biometric gate cross-platform).

## 3. Inventaire endpoints loopback actuels + tier cible

| Endpoint | Origine | Tier actuel | Tier cible | Justification cible |
|---|---|---|---|---|
| `GET /health` | S16 Phase A | AUCUN (exception) | AUCUN | Liveness probe, pas d'action |
| `GET /curator/*` | S7 (browse aggregator) | T0 | T0 | Lecture seule, non-critique |
| `GET /project/*` | S12 | T0 | T0 | Lecture seule |
| `POST /project/deploy` | S14 | T0 | T1 | Deploy sign coord = déléguer action non-réversible |
| `POST /task/submit` | S13 (bridge) | T0 | T0 | Rate-limité S21 Phase A + guardrails S21/S22 |
| `GET /consent/status` | S16 Phase C | T0 | T0 | Lecture only |
| `POST /consent/edit` tier `mes_projets→tous` | S16 Phase C | T0 | T1 | Escalade privilège GPU worker (expose plus de tasks acceptées) |
| `POST /consent/edit` tier other | S16 Phase C | T0 | T0 | Descente ou tier équivalent — pas d'escalade |
| `POST /panic/wipe` | S20 Phase B | T0 (+ Ctrl+Shift+Alt+W x5) | **T2** | Action destructive terminale, protéger contre malware browser-injected |
| `POST /unlock-duress` | S20 Phase B | T0 (+ PIN input) | **T2** | Bypass duress mode = exposer keypair réel, critique |
| `POST /auth/rotate-token` force | S18 Phase D | T0 | T1 | Rotation défensive = OK T1 ; rotation hostile par malware nécessite au moins UI confirm |
| `POST /canary/cosign` FROST (S30 N1) | S30 futur | N/A | **T2** | Co-signer canary = engagement cryptographique plateforme, LT-4 consumer natif |
| `POST /quarantine/flush` | S21 Phase D CLI | T0 | T1 | Purge queue = perte d'évidence, validator humain recommandé |

## 4. Format `consent.json` étendu

Le fichier `~/.sbfb/consent.json` actuel (S16 Phase C) :

```json
{
  "level": "mes_projets",
  "caps": {
    "watts_max": 250,
    "vram_mb_max": 12000,
    "hours_per_day": 6
  }
}
```

Extension proposée (S22 Phase F absorption) :

```json
{
  "level": "mes_projets",
  "caps": { "...": "..." },
  "level_threat_note": "Exposé aux apps que vous avez publiées. Défense-en-profondeur par kudos ledger + deploy verified S14. Zéro exposition apps tierces.",
  "residual_threats_acknowledged": ["T2-Sybil-kudos-farm"]
}
```

- `level_threat_note` : texte court (120 chars max) lisible UI
  launcher tooltip. Décrit factuellement ce que le tier expose.
- `residual_threats_acknowledged` : liste des threats résiduels
  acceptés par l'utilisateur (format `T<tier>-<category>-<subcategory>`
  référencé dans `THREAT_MODEL.md`). Permet DPIA/RGPD attestation
  formelle.

Les 4 niveaux S16 Phase C (`mes_projets / open_source_verifies /
whitelist_manuelle / tous`) acquièrent chacun leur `level_threat_note`.
Exemples :

- `mes_projets` : "Exposé uniquement aux apps que vous avez
  publiées via deploy verified. Zéro exposition tierce."
- `open_source_verifies` : "Exposé aux apps publiques dont le code
  source est vérifié (provenance SLSA L1 + commit signé). Exposition
  Sybil potentielle si contributeur malveillant publie app sourced."
- `whitelist_manuelle` : "Exposé aux apps listées explicitement par
  vous. Vous êtes responsable de la vérification initiale."
- `tous` : "Exposé à toute app acceptée par au moins un curator
  auquel vous souscrivez. Risque maximum de consommation ressources
  abusive. À utiliser uniquement si vous comprenez le modèle curator."

## 5. Implémentation S22 Phase F (doc-only)

Phase F absorption = **pur doc**, zéro code S22 :

1. Création de ce document (`LOOPBACK_ENDPOINTS_TRUST_TIERS.md`).
2. Enrichissement `consent.json` schema doc dans `docs/security/
   THREAT_MODEL.md §2` table A4 (colonne threat note par niveau).
3. Préparation stub Rust : `crates/nexus-worker-core/src/consent.rs`
   `ConsentLevel` enum ajout attribute `#[doc = "..."]` par variante
   (compilé en docstring, pas de struct runtime field).

## 6. Implémentation T1 CONFIRM_PROMPT (S25 co-landing D5 capability toggle)

Ajout côté loopback daemon :

```rust
pub enum TrustTier {
    Auto,
    ConfirmPrompt { nonce: [u8; 16], expires_at: Instant },
    BiometricGate { nonce: [u8; 16], expires_at: Instant },
}

pub trait TrustTierGate {
    async fn require(&self, endpoint: &str, tier: TrustTier) -> Result<(), TrustTierError>;
}
```

Workflow T1 :
1. Client requête endpoint T1 → daemon retourne 403 + `X-SBFB-
   Confirm-Nonce: <hex>` + `X-SBFB-Confirm-TTL: 30`.
2. Shell UI reçoit 403, affiche dialog "Confirmer : rotation token ?"
   avec nonce + TTL visible.
3. User clique "Confirmer" → shell POST `/confirm/tier-gate?nonce=
   <hex>` → daemon met le nonce en cache valide 30s.
4. Client re-requête endpoint T1 avec `X-SBFB-Confirm-Nonce: <hex>`
   → daemon accepte si nonce valide + non-consumé + TTL OK.

Mitigation malware DOM injection : nonce ré-entrant (un nonce =
un usage), TTL court (30s), scope endpoint-précis (nonce pour
`/auth/rotate-token` ne fonctionne pas pour `/panic/wipe`).

## 7. Implémentation T2 BIOMETRIC_GATE (LT-4 post-v1.0)

Cf. `docs/release/ROADMAP_COMMITMENTS.md LT-4` pour le détail
cross-platform. Résumé :

- Windows : Windows Hello via `windows-rs 0.58+` `Windows.Security.
  Credentials.UI` namespace.
- macOS : `LocalAuthentication` framework via `security-framework
  0.2+` ou crate dédiée `local-authentication-macos`.
- Linux : polkit via `polkit-agent` binding ou `zbus` D-Bus direct.

Workflow T2 = T1 + prompt biométrie OS entre étape 2 et 3. Si user
échoue biométrie 3 fois = gate verrouille endpoint 5 min.

## 8. Threat model coverage

| Threat class | Tier mitigé | Tier résiduel |
|---|---|---|
| AD1 — Remote attacker via network | T0 | — (bearer token + Host/Origin bloque) |
| AD2 — Malware user-mode avec browser compromise | T2 | T0 : invocation silencieuse possible ; T1 : auto-click DOM possible (mitigation nonce ré-entrant réduit) ; T2 : bloqué (biométrie OS non-forgeable) |
| AD3 — Multi-user OS (Windows shared account) | T0 (UDS/NP peer creds) | — |
| AD4 — Compromise du shell UI lui-même | T2 | T0/T1 : bypass possible si shell est l'attaquant ; T2 : biométrie OS indépendante du shell |
| AD5 — Debugger attaché au daemon | ? | Out-of-scope (attacker local avec debug privilege = game over, aucune mitigation applicative) |

## 9. Références

- `docs/security/THREAT_MODEL.md` (ADVERSARIES AD1-AD5 + STRIDE)
- `docs/security/ADVERSARIES.md` §3.1 T0-T5 adversary tiers
- `.planning/archive/v1.2/sprint16_plan.md §Phase A` (loopback
  hardening S16 Phase A `d7c265a` origine)
- `.planning/archive/v1.2/sprint20_plan.md §Phase B` (panic wipe +
  duress PIN S20 Phase B `c32ecb3` origine)
- `.planning/research/S23_to_S29_agents_sudo_integration_matrix.md`
  (mapping 18 features openai-agents-python + microsoft/sudo, cette
  extension = feature D1 du cluster D)
- `docs/release/ROADMAP_COMMITMENTS.md LT-4` (D4 biometric
  cross-platform, consumer T2 tier)
