---
written: 2026-04-20  # S22 hors-sprint post Phase B `e9530c2`
last_validated: 2026-06-03  # S73 Phase A : §2.1 portee daemon+Operator + §3 GET /result reordonne + §8.1 couverture Operator (P2-TIER-MODEL, P2-RESULT-TEXT-GUARDRAIL-ORDER)
status: design-only T1/T2 (implementation S22 Phase F wrap-up + extension S25/S28/LT-4) ; §3.1 Operator = IMPLEMENTE+DURCI S71 C (`a0337c6`) ; Operator place dans le tier-model formel §2.1/§8.1 (S73 Phase A)
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

### 2.1 Portée : deux serveurs loopback (daemon + Operator)

Ce vocabulaire de tiers s'applique formellement aux **deux** serveurs
loopback du nœud, qui ont des postures de confiance **distinctes** :

| Serveur | Port | Auth | Peer-creds | Tiers présents |
|---|---|---|---|---|
| **Daemon** (`nexus-shell-daemon`) | dynamique | `X-SBFB-Token` + Host + Origin | **Oui** (UDS `SO_PEERCRED` / Named Pipe SDDL, S16) | T0 partout ; cibles T1/T2 par endpoint (§3) |
| **Operator** (`sbfb-factory`, §3.1) | `:3001` | `X-SBFB-Token` + Host + Origin + CORS épinglé (G7) | **Non** (TCP loopback uniquement) | **T0 uniformément** — pas de T1/T2 ; gate applicatif `SENSITIVE_ACTIONS` (G2) avant spawn |

Conséquence formelle : l'Operator est **entièrement T0** (aucun endpoint
T1/T2 à ce stade) et, contrairement au daemon, **ne bénéficie pas** de la
mitigation peer-creds AD3. Sa surface (write + spawn) est protégée par le
gate applicatif `SENSITIVE_ACTIONS` (§3.1, G2), pas par un tier de
confiance OS. Sa couverture threat model est tracée séparément en §8.1.

## 3. Inventaire endpoints loopback actuels + tier cible

| Endpoint | Origine | Tier actuel | Tier cible | Justification cible |
|---|---|---|---|---|
| `GET /health` | S16 Phase A | AUCUN (exception) | AUCUN | Liveness probe, pas d'action |
| `GET /api/daemon/curators` | S7 (browse aggregator), namespace S53 | T0 | T0 | Lecture seule, non-critique |
| `GET /api/daemon/browse` | S12, namespace S53 | T0 | T0 | Lecture seule |
| `POST /api/daemon/browse/pull` | S53 Phase G | T0 | T0 | Gossip browse_request, PoW envelope |
| `POST /api/v1/deploy` | S14, namespace S42 | T0 | T1 | Deploy sign coord = déléguer action non-réversible |
| `POST /api/v1/tasks/submit` | S13 (bridge), namespace S44 | T0 | T0 | Rate-limité S21 Phase A + guardrails S21/S22 |
| `GET /api/v1/tasks/{id}/result` | S72 Phase D (option A) | T0 | T0 | Lecture seule du `result_text` — persisté **uniquement APRÈS** passage de l'output guardrail (S73 Phase A, D5 : `default_output_chain` avant `set_task_result` sur les deux chemins HTTP + `validator_loop`). Un texte rejeté n'est jamais `completed`/lisible. Primitive lue par le bras NetworkProvider Operator pour rendre une réponse réseau dans le chat. 404 si pending/inconnu |
| `GET /api/v1/consent` | S16 Phase C, namespace S43 | T0 | T0 | Lecture only |
| `POST /api/v1/consent/set` tier escalade | S16 Phase C, namespace S43 | T0 | T1 | Escalade privilège GPU worker (expose plus de tasks acceptées) |
| `POST /api/v1/consent/set` tier other | S16 Phase C, namespace S43 | T0 | T0 | Descente ou tier équivalent — pas d'escalade |
| `POST /api/daemon/panic/wipe` | S20 Phase B, namespace S53 | T0 (+ Ctrl+Shift+Alt+W x5) | **T2** | Action destructive terminale, protéger contre malware browser-injected |
| `POST /api/daemon/publish` | S11, namespace S53 | T0 | T0 | Broadcast gossip project announcement |
| `POST /api/daemon/feed/insert` | S62, namespace S65 | T0 + `X-SBFB-Feed-Internal` header | T1 | Insert feed entry. S65 defense-in-depth: header check rejette callers externes (pas crypto, pas de nonce). Vrai T1 (CONFIRM_PROMPT + HMAC nonce temporal) programme post-pilote S69 |
| `GET /auth/token` | S16 Phase A | T0 (Host+Origin only) | T0 | Bootstrap bearer token |
| `POST /canary/cosign` FROST (S30 N1) | S30 futur | N/A | **T2** | Co-signer canary = engagement cryptographique plateforme, LT-4 consumer natif |
| `POST /quarantine/flush` | S21 Phase D CLI | T0 | T1 | Purge queue = perte d'évidence, validator humain recommandé |

## 3.1 Serveur Operator (sbfb-factory, port `:3001`) — surface write + spawn

Le **Factory Operator** (`crates/sbfb-factory/src/operator_server.rs`)
est un serveur HTTP loopback **distinct du daemon** : process séparé,
port `:3001` par défaut (`main.rs:161`), **TCP loopback uniquement** —
pas de UDS / peer-creds, un sous-ensemble token+Host+Origin du modèle
S16 (le daemon, lui, ajoute SO_PEERCRED Unix / SDDL Named Pipe). Il
**écrit des fichiers** et **spawn des sous-processus agent**
(`claude --permission-mode bypassPermissions`), ce qui en fait une
surface critique au même titre que les endpoints write du daemon. Livré
sans auth dans un bloc off-sprint, **durci S71 Phase C** (`a0337c6`,
G7 + G2). Inventaire à jour (P2-H-1, audit S71 Track H) :

| Endpoint | Origine | Tier actuel | Tier cible | Justification |
|---|---|---|---|---|
| `POST /api/artifacts/draft` (**write**) | off-sprint, durci S71 C | T0 | T0 | Écrit un artefact draft sur disque dans la frontière loopback durcie |
| `GET /api/chat/{id}/stream` (**spawn**) | off-sprint, durci S71 C | T0 + gate `SENSITIVE_ACTIONS` | T0 + gate | Spawn agent `bypassPermissions` ; gate `shell`/`commit`/`push`/`PASS` AVANT spawn (G2) |
| `POST /api/chat/{id}/send` | off-sprint, durci S71 C | T0 + gate `SENSITIVE_ACTIONS` | T0 + gate | Enregistre le message + déclenche le spawn ; même gate |
| `POST /api/actions/run` | S70 | T0 | T0 | Action allowlistée Operator (pas un shell libre) |
| `POST /api/context-pack` | S70 | T0 | T0 | Génère un context-pack depuis le repo |
| `GET /api/terminal/ws` | S70 | T0 | T0 | WebSocket terminal (lecture cast `.planning/terminal`, durci S71 D drive-prefix) |
| `GET /api/status` `…/lint` `…/audit/{rev}` `…/prompt/{kind}` `…/context` `…/providers` `…/actions/log` `…/chat/{id}/log` `…/sprint-history*` `…/terminal/sessions` | S70/S71 | T0 | T0 | Lecture seule sous le même middleware auth |

Gate **G7** (S71 Phase C `a0337c6`) : middleware `auth_required`
(`auth.rs:229`) sur chaque route data-bearing — `X-SBFB-Token`
(`constant_time_eq`) + `Host:` loopback + `Origin:` loopback/absent +
`CorsLayer` épinglé à `is_loopback_origin` (`operator_server.rs:103`,
plus de `allow_origin(Any)`). Gate **G2** : `SENSITIVE_ACTIONS`
(`const` ligne 34) dans `handle_chat_stream` AVANT le spawn (gate
`:866`, spawn `:898`). Token réutilisé depuis `~/.sbfb/auth_token`.
Détail + noms de tests : `docs/shell/PATTERNS.md §P35`. Menaces
catalogées : `THREAT_MODEL.md §14` (T-OPERATOR-CSRF / T-OPERATOR-SPAWN).

**Résidu** : un processus local hostile du même utilisateur peut lire
le token bearer et invoquer ces endpoints (frontière OS-sandbox
acceptée, même modèle que le daemon loopback — cf. §8 de ce document
(couverture threat model, table AD1-AD5) AD2 « Malware user-mode »,
résidu T0 : invocation silencieuse + `THREAT_MODEL.md §5.7`). Pas de tier T1/T2 sur l'Operator à ce stade ;
les actions destructives passent par le gate `SENSITIVE_ACTIONS`, pas
par un gate biométrique OS.

**NetworkProvider S72 (anticipation ProviderRouter)** : le bras
`Network` du ProviderRouter S72 (`provider_router.rs`) est un **client
sortant** de `POST /api/v1/tasks/submit` (daemon, §3 ligne 55, tier T0,
déjà inventorié + rate-limité S21) — **pas une nouvelle surface
entrante**. Le dispatch réseau S72 reste dans la frontière loopback
durcie ; le gate `SENSITIVE_ACTIONS` reste AVANT dispatch quel que soit
le provider (S72 Phase D). Aucun nouvel endpoint entrant Operator n'est
ajouté par le ProviderRouter.

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

### 8.1 Couverture Operator (`:3001`)

L'Operator étant uniformément T0 et sans peer-creds (§2.1), sa couverture
diffère du daemon sur AD2/AD3/AD4 :

| Threat class | Mitigation Operator | Résidu |
|---|---|---|
| AD2 — Malware user-mode lit le token bearer | Gate applicatif `SENSITIVE_ACTIONS` (G2) AVANT spawn `bypassPermissions` ; CORS épinglé `is_loopback_origin` (G7) | **T0** : invocation silencieuse des endpoints write/spawn possible (pas de T1/T2). Le gate bloque `shell`/`commit`/`push`/`PASS` mais le reste reste invocable. Pas de biométrie OS |
| AD3 — Multi-user OS (compte Windows partagé) | **Aucune** mitigation peer-creds (TCP loopback only, pas d'UDS/NP) | **T0** : un autre user local sur la même loopback n'est filtré que par le token bearer per-boot (`~/.sbfb/auth_token`). Écart explicite vs daemon (qui a `SO_PEERCRED`/SDDL) |
| AD4 — Compromise du shell/Viewer | `SENSITIVE_ACTIONS` indépendant du front ; shell/commit/push/verdict final passent par une vraie session agent + gates repo (pas l'Operator seul) | **T0** : si l'appelant local authentifié est l'attaquant, les endpoints non-gated restent invocables. Pas de tier OS indépendant du front |

Trigger de revalidation : tout ajout d'un endpoint **write/spawn** sur
l'Operator, ou l'introduction d'un tier T1/T2 côté Operator.

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
