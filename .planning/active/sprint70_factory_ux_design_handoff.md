# Handoff UX Design — Factory Viewer + Factory Operator

## Waiver Claude Design

Claude Design non utilisé — le développeur a décidé que l'agent
Claude Code, avec accès complet au code source, au design system
existant et aux contraintes sécurité, produirait un résultat plus
cohérent qu'un prompt isolé dans claude.ai/design.

Ce handoff contient les décisions de design et le wireframe
repo-visible en remplacement du prototype Claude Design.

## Décisions de design

### Palette (héritée du shell SBFB + apps existantes)

| Token | Valeur | Usage |
|---|---|---|
| `--bg` | `#0d1117` | Fond principal |
| `--bg-surface` | `#161b22` | Sidebar, status bar |
| `--bg-card` | `#1c2128` | Cartes, panneaux |
| `--border` | `#30363d` | Bordures |
| `--text` | `#ffffff` | Texte principal |
| `--text-muted` | `#8b949e` | Texte secondaire |
| `--accent` | `#58a6ff` | Liens, accents |
| `--green` | `#3fb950` | Succès, validé |
| `--red` | `#f85149` | Erreur, échoué |
| `--yellow` | `#d29922` | Warning, attention |

### Layout Operator

- Sidebar gauche 240px (collapsible 64px sous 768px)
- Zone contenu flex-1 avec padding 24px
- Status bar bas 32px : HEAD sha + sprint number
- Icônes : lucide-react (cohérent avec web/)

### Layout Viewer

- Top bar sticky 56px (cohérent avec sbfb-explorer)
- Contenu centré max-width 960px
- Pas de sidebar (app simple 2 écrans)

### Composants partagés readonly

- `StatusBadge` : dot coloré + label FR (4 états)
- `VerdictChip` : icône + label FR (4 verdicts)
- `ProofCard` : carte proof dl avec commit/hash/signataire/verdict
- `SprintTimeline` : grille phases avec StatusBadge
- `PreviewList` : grille apps avec metadata
- `ChangelogPanel` : timeline verticale versions

### UX Operator — intentions métier

Les 6 actions de l'assistant de phase :
1. "Préparer la phase" (preflight)
2. "Relire la phase" (phase-review)
3. "Vérifier avant validation" (phase-auditor)
4. "Préparer le message de commit" (commit-body)
5. "Transmettre à un autre agent" (handoff)
6. "Auditer le sprint" (audit-gate)

Commandes techniques dans `<details>` repliable uniquement.

### Responsive

- `>= 1024px` : sidebar étendue + contenu
- `768px — 1023px` : sidebar icônes seules + contenu
- `< 768px` : sidebar hamburger overlay + contenu plein écran
