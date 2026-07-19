# Workflow Hub — amorce (Sprint 83 Phase B)

App SBFB **auto-portante** et **scellée** (0 CDN, 0 fetch, 0 worker ; tourne
sous la CSP du bac à sable blob-serve, `connect-src 'none'`). Cette amorce pose
la **fondation d'accessibilité** du chantier Workflow Agents ; le vrai atelier
d'autorat (React + xyflow) est scaffoldé plus tard (kickoff Q8=A, via
app-authoring). DoD = **fondation + logistique, pas conformité** (AM-8).

## Ce que l'amorce livre

| Fichier | Rôle |
|---|---|
| `preboot.js` | Preboot **anti-flash needs-free** : pose les 15 axes-effet `data-*` avant le premier paint (**defaults + signaux OS uniquement** — aucun transport hash `#a11y` app-lisible). **N'écrit JAMAIS `data-needs`**. Expose `window.SbfbHubA11y`. |
| `a11y.js` | Canal **live** : applique le profil résolu poussé par l'hôte via l'event bridge `a11y-profile`, **sans reload**. **Rejet strict** : tout payload incomplet, à valeur invalide, à clé supplémentaire (`needs`), ou dont la source n'est pas le `parent` est **entièrement rejeté**. |
| `tokens.css` | **Design tokens oklch** (thème dark de base + rampe `data-scale`). |
| `a11y.css` | Règles d'effet génériques pilotées par les `data-*` (motion, police, espacement, focus, transparence, pointeur, monochrome). |
| `styles.css` | Mise en page minimale en **propriétés logiques** (RTL par bascule `dir`). |
| `mode-guide.json` | **Contrat déclaratif du mode guidé** : 3 projections (canevas / plan / guidé) d'un **même modèle de commandes** (même digest sémantique). Projections implémentées ultérieurement. |
| `index.html` / `app.js` | Amorce visible (français) : lecture du profil résolu, contrat du mode guidé, bascule LTR/RTL. |
| `i18n/locales/{fr,en,ar}.po` + `lingui.config.ts` | Socle i18n (Lingui, source fr + en + ar RTL). |
| `scripts/check-i18n-*.sh`, `scan-hub-strings.sh` | Gates de discipline i18n (parité clés / anti-verdict cross-locale / no-literal). |

## Invariant cardinal — `needs[]` ne quitte jamais l'hôte

Le profil projeté (`ResolvedAccessibilityProfile.v1`) ne porte que l'**effet**
(`motion=reduced`, `contrast=high`…), jamais la **cause** (les diagnostics de
l'opérateur). Ni le preboot ni le canal live n'écrivent `data-needs`. C'est
volontairement **plus strict** que l'outil-hôte Factory Operator (surface hôte
first-party privilégiée : il écrit `data-needs` à son propre DOM local, sous sa
propre CSP — jamais dans le bac à sable blob-serve). Contrat : `sbfb-workflow-core`
(Rust) + `web/src/a11y` (TS) ; garde : `web/scripts/check-a11y-needs-free.sh`.

## Servir

Comme toute app SBFB : publiée en archive puis rendue par le daemon blob-serve
dans un iframe sandboxé, via le shell Browse. **Ne jamais** ouvrir blob-serve
en onglet direct.
