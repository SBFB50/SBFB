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
