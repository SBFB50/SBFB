# `.planning/` — Sprint planning workspace

Cette arborescence applique le **PARA pattern** (Projects / Areas /
Resources / **Archives**) au workflow sprint du projet. Le but :
garder le root court, separer le travail vivant de l'historique,
et permettre de scaler a 50+ sprints sans saturer le contexte.

---

## Layout

```
.planning/
├── README.md              # ce fichier
├── active/                # un seul sprint a la fois (kickoff + plan + audit du precedent)
│   └── sprint{N}_*.md
├── archive/               # sprints fermes, groupes par version livree
│   ├── v1.0/              # Sprints 0-13 — pivot + P2P + universal render + bridge + launcher
│   └── v1.1/              # Sprints 14-15 — verified deploy + bridge bidirectionnel + watchdog
├── codebase/              # cartographie codebase cross-sprint (snapshot 2026-04-06)
├── research/              # research notes consommees par les sprints (cross-sprint)
└── *_ROADMAP.md           # roadmaps thematiques evergreen (DISTRIBUTED_GPU, NEXUS_GOV, OPEN_SOURCE)
```

---

## Cycle de vie d'un sprint

### Pendant le sprint (Phase A → Phase E)

Tous les fichiers `sprint{N}_*.md` du sprint en cours vivent dans
`active/`. C'est le seul dossier que l'agent executeur regarde
pour ses livrables courants.

Fichiers attendus dans `active/` au cours du sprint :

| Fichier | Quand ecrit | Par qui |
|---|---|---|
| `sprint{N}_kickoff.md` | Phase 0/A entree | session sprint N |
| `sprint{N}_plan.md` | Phase 0/A entree | session sprint N |
| `sprint{N-1}_audit_findings.md` | Phase 0 (gate du sprint precedent) | session sprint N (fraiche) |
| `sprint{N}_verification.md` | Phase E sortie | session sprint N |
| `sprint{N}_audit_plan.md` | Phase E sortie | session sprint N |

A noter : `sprint{N-1}_audit_plan.md` (consomme par la Phase 0)
vit dans `archive/v{X}/` puisque le sprint N-1 est ferme. La
Phase 0 le **lit** depuis l'archive et **ecrit** son findings
dans `active/`.

### A la cloture du sprint N (= demarrage Sprint N+1)

Quand le Sprint N+1 ouvre sa Phase 0 :

```bash
# 1. Identifier la version cible (v1.x correspondant au sprint N)
TARGET=archive/v1.1   # exemple

# 2. Deplacer les 5 docs du sprint N (kickoff + plan + verification +
#    audit_plan + audit_findings) vers l'archive
git mv .planning/active/sprint{N}_*.md .planning/$TARGET/

# 3. Mettre a jour docs/claude/SPRINT_LOG.md (ajouter row)

# 4. Sprint N+1 ecrit ses propres docs dans active/
```

Les findings du sprint N sont donc archives **avec** les autres
docs du sprint N, pas avec ceux du sprint N+1.

---

## Regroupement par version (`archive/v{X}/`)

Les sous-dossiers d'archive ne sont pas par sprint individuel
mais par **version livree**. Cela permet de retrouver
"qu'est-ce qui a livre v1.0 ?" en lisant un seul dossier.

Mapping actuel :

| Version | Sprints | Theme dominant | Tip de cloture |
|---|---|---|---|
| **v1.0** | S0-13 | Pivot SBFB, P2P iroh, universal render, bridge postMessage, launcher | `08853ff` |
| **v1.1** | S14-15 | Verified deploy (Keyoxide + SLSA L1), bridge bidirectionnel, CPU watchdog, CLI scaffold | `4da0043` |
| **v1.2** | S16+ (en cours) | Security hardening (loopback auth + GPU consent + VM roadmap) | TBD |

Quand une version majeure est livree, le dossier est ferme et un
nouveau v1.x+1/ s'ouvre.

---

## Resources hors-sprint

Trois categories ne sont pas liees a un sprint specifique et
restent donc au root :

- `codebase/` — cartographie codebase (ARCHITECTURE.md,
  STRUCTURE.md, STACK.md, CONVENTIONS.md, etc.), snapshot
  2026-04-06, sert de reference cross-sprint
- `research/` — notes de recherche (cf. memory pour l'historique)
  consultees par plusieurs sprints
- `*_ROADMAP.md` — DISTRIBUTED_GPU, NEXUS_GOV, OPEN_SOURCE :
  documents thematiques long-terme qui transcendent les sprints

Ces dossiers ne sont jamais archives.

---

## Index global

Pour la table synthetique de **tous** les sprints (statut, tip,
nb commits, docs presents) voir
[`docs/claude/SPRINT_LOG.md`](../docs/claude/SPRINT_LOG.md).

Pour la **methodologie** (sprint lifecycle, audit gate pattern,
conventions commit), voir
[`docs/claude/README.md`](../docs/claude/README.md).
