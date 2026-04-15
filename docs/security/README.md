# Security — nexus-grid / SBFB

Ce dossier documente le modele de menace et la roadmap
d'isolation runtime du projet nexus-grid.

Ecrit en Phase E du Sprint 16 (2026-04-14), une fois les
mitigations A-D livrees. Les references `commit <sha>` pointent
vers le code reel, pas vers un wish-list.

---

## Index

| Document | Contenu | Sprint livre |
|---|---|---|
| [`THREAT_MODEL.md`](THREAT_MODEL.md) | Assets, adversaires, DFD, STRIDE par composant, LINDDUN par flux, mitigations livrees + residuals | S16 Phase E |
| [`RUNTIME_ISOLATION.md`](RUNTIME_ISOLATION.md) | Roadmap VM invisible (WSL2 / Virtualization.framework / systemd-nspawn) pour Sprint 17+ | S16 Phase E |
| [`ADVERSARIES.md`](ADVERSARIES.md) | Taxonomie 6 tiers T0-T5 (user misconfig → state targeted), rationale, mapping app-risk, glossaire | S17 Phase A |
| [`adversaries/`](adversaries/) | 6 fiches detaillees T0-T5 : capabilites, budget, tactiques, mitigations par tier | S17 Phase A |
| [`ATTACK_SCENARIOS.md`](ATTACK_SCENARIOS.md) | 12 scenarios concrets T1-T5 (CSP bypass, supply chain, dragnet, checkpoint seize, etc.) avec chain + mitigation status | S17 Phase A |
| [`P2P_THREATS.md`](P2P_THREATS.md) | 7 vecteurs reseau P2P (Sybil, Eclipse, gossip, DHT, routing/BGP, traffic analysis, ISP block) — etat SBFB + mitigations sequencees | S17 Phase B |
| [`COMPUTE_THREATS.md`](COMPUTE_THREATS.md) | 7 classes menace GPU compute-sharing (prompt leakage, result spoofing, compute theft, model extraction, prompt injection, side-channel GPU, DoS task flood) — etat SBFB + mitigations sequencees + refs 2020-2026 | S17 Phase C |
| [`HARDENING_ROADMAP.md`](HARDENING_ROADMAP.md) | Matrice 27 threats × mitigations + framework prioritization (I×L/E) + roadmap Sprint 18-30 sequencee + quick-wins + big-rocks + dependency graph + gates 1-4 debloquage | S17 Phase D |
| [`VALIDATED_BLUEPRINT.md`](VALIDATED_BLUEPRINT.md) | Vision long-terme maximaliste 13 couches (host/identity/transport/overlay/sybil/storage/compute/runtime/deploy/trust/opsec/formal-verif/research), chaque brique OSS validee contre docs officielles 2026 + advisories + CVE. Positionnement vs Signal/Tor/Briar/SecureDrop/Mozilla/Bytecode Alliance | S17 session recherche |
| [`RELEASE_GATES.md`](RELEASE_GATES.md) | Stub pointeur Phase E scope-cut — redirige vers `HARDENING_ROADMAP §7` pour mapping Gate→Sprint | S17 Phase F (stub) |
| [`PARTNERSHIPS.md`](PARTNERSHIPS.md) | Stub pointeur Phase E scope-cut — redirige vers `VALIDATED_BLUEPRINT Couche 10` pour partenariats OTF/NLnet/OpenSSF/ISRG/HackerOne + ONG cibles par gate | S17 Phase F (stub) |
| [`DISCLOSURE.md`](DISCLOSURE.md) | Stub pointeur Phase E scope-cut — redirige vers `VALIDATED_BLUEPRINT Couche 10` pour pattern security.txt + PGP + 90d embargo + CVE workflow | S17 Phase F (stub) |

## Mitigations livrees Sprint 16 (pointeurs rapides)

| Phase | Commit | Surface | Livre |
|---|---|---|---|
| A | `d7c265a` | Loopback HTTP | X-SBFB-Token 256-bit + Host allowlist + Origin check (mitigation CVE-2025-49596) |
| B | `1cfde89` | UDS + Named Pipes | SO_PEERCRED (Unix) + DACL user-only (Windows) |
| C | `3247e88` | Consent GPU + caps | Dialog 4 niveaux + whitelist L3 + watts/VRAM/heures enforced worker-side |
| D | `10bbc63` | ProjectAnnouncement v5 | Flag `is_open_source` derive auto par coordinator (non-user-settable) |

## Supply chain CI (Sprint 18 Phase A)

Trois garde-fous CI bloquent toute PR introduisant un CVE critical
upstream avant landing. Configures dans
[`deny.toml`](../../deny.toml) (Rust),
[`web/audit-ci.json`](../../web/audit-ci.json) (npm),
[`.github/workflows/supply-chain.yml`](../../.github/workflows/supply-chain.yml)
(declenche sur `pull_request` + cron weekly Monday 08:00 UTC).

| Outil | Surface | Politique |
|---|---|---|
| `cargo-deny` | Rust workspace (RUSTSEC + bans + licenses + sources) | yanked=deny, unmaintained=workspace, deny `wasmtime <43.0.1` (CVE-2026-34941 + CVE-2026-34946, Bytecode Alliance 2026-04-09), allowlist licenses standard OSI + AGPL workspace + MPL-2.0 |
| `pip-audit` (>=2.9,<3) | 3 packages Python (`nexus-sdk`, `nexus-coordinator`, `nexus-app-gov`) | requirements materialises via `uv export --no-emit-workspace --no-editable` puis audit `--strict` |
| `audit-ci` (>=7.1) | npm `web/` | `critical: true` -> CI fail, threshold high+ remontes en S19 |

Smoke local : `bash tests/ci-smoke/supply-chain-green.sh` execute
les trois audits (cargo-deny + 3x pip-audit + audit-ci) et exit 0
sur master propre. Prereq : `cargo install cargo-deny --locked`.

Ignores documentes (cargo-deny `[advisories].ignore`) :

- **RUSTSEC-2026-0097** (`rand 0.8` unsound) — applique uniquement
  via `ThreadRng` + custom `log` logger ; SBFB utilise `OsRng`
  directement (cf. `crates/nexus-core-rs/src/crypto.rs` et
  `crates/nexus-shell-daemon-core/src/auth.rs`) sans logger custom.
  Path non-exploitable. Upgrade `rand 0.9.3` repousse Sprint 19+
  (cascade `ed25519-dalek 2.3` + `rand_core 0.9`).

---

## Matrice de severite

Les findings dans `THREAT_MODEL.md` sont classes selon :

| Severite | Impact | Exploitabilite | Action |
|---|---|---|---|
| **C**ritical | Compromission total / exfil key material | Attaquant non privilegie | Sprint immediat |
| **H**igh | Compromission partielle / data tampering | Attaquant local user-mode | Prochain sprint |
| **M**edium | DoS, info disclosure limitee | Chaine d'attaque specifique | Backlog dedie |
| **L**ow | Best practice, defense en profondeur | Hypothetique / theorique | Backlog evergreen |

Cette echelle est alignee CVSS 3.1 mais simplifiee : le projet
n'a pas d'equipe dediee, un findings H equivaut a un Sprint 17+
item prioritaire dans le kickoff suivant.

## Comment contribuer au threat model

1. **Nouveau component ou flux de donnees** : ajouter une ligne
   dans la table STRIDE (`THREAT_MODEL.md` §5) et un row dans
   le DFD ASCII (§4).
2. **Nouvelle mitigation livree** : mettre a jour la table
   mitigations (`THREAT_MODEL.md` §7) avec le commit hash et
   le fichier touche. Supprimer la ligne residuelle couverte
   dans §8.
3. **Nouveau risque residuel** : ajouter ligne dans §8 avec
   severite + sprint cible (ou `v2+` si long-terme).
4. **Mise a jour post-audit externe** : creer
   `THREAT_MODEL_v2.md` plutot que de re-ecrire en place. Les
   signatures provenance historiques restent auditables.

Chaque change du threat model est revu en Phase E du sprint
correspondant. Il n'y a pas de revue asynchrone separee — le
document vit au rythme des sprints.

---

## Hors scope ici

- **Code review / bug bounty** : couvert par
  [CONTRIBUTING.md](../../CONTRIBUTING.md) et la section
  "Security" du README racine.
- **Incident response** : pas de runbook pour l'instant (projet
  solo). Si une CVE critique est publiee sur iroh / axum /
  FastAPI, l'utilisateur pousse un hotfix avec `fix(security):`
  et met une note dans `SPRINT_LOG.md`.
- **Conformite reglementaire exhaustive** : le threat model
  couvre GDPR via l'angle LINDDUN (§6) mais ne constitue pas un
  DPIA formel.
