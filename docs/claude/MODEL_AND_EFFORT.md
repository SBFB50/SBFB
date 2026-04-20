# Modèle & effort — discipline de session

Document de discipline optionnelle pour piloter la consommation
compute de Claude Code sur SBFB. Couvre deux leviers distincts :
(1) **effort level** (profondeur de raisonnement) et (2) **modèle**
(Opus 4.6 vs 4.7). Les deux impactent coût et qualité mais pas de
la même façon.

## 1. Effort level — toggle manuel par type de phase

### 1.1 Contexte

Le setting `effortLevel` dans `~/.claude/settings.json` est
actuellement à `"xhigh"` (global, toutes les sessions). Le mapping
qualitatif :

| Valeur | Profondeur | Latence relative | Cas d'usage typique |
|---|---|---|---|
| `medium` | Shallow multi-step | 1x | Docs sprint wrap-up, édits triviaux, scan artefacts |
| `high` | Deep multi-step | 2-3x | Phase A/F wrap-up, fix post-audit P2/P3, chore planning |
| `xhigh` | Deeper reasoning | 5-7x | Phase B/C/D implémentation code métier standard |
| `max` | Maximum reasoning | 10-15x | Phase 0 audit gate, Phase E federation/security, G1 Design Review Board, pivot G8 DESIGN-CONFLICT |

Défaut global `xhigh` inflate le coût sur les phases légères
(Phase A scope-cut simple, Phase F docs-only wrap-up, chore planning)
sans bénéfice qualité mesurable. Défaut global `max` ruine le budget
sur les phases simples.

### 1.2 Mapping recommandé par type de phase

Sélection à faire **manuellement** en début de phase via slash
command `/effort <level>` (commande Claude Code built-in) ou via
Config tool. Pas d'automatisation hook — trop fragile (hook qui
édite settings.json mid-session = race condition, et l'effort est
appliqué au prompt submit, pas au tool call).

| Type de phase | Effort recommandé | Rationale |
|---|---|---|
| **Phase 0** audit gate du sprint N-1 | `max` | Décisions de blocage P0/P1, 6-9 tracks en parallèle, responsabilité d'un verdict binaire. Jamais économiser ici |
| **Kickoff** Cas C nouveau sprint + plan | `max` | D1..D5 Day 0 figées à vie du sprint, Design Review Board G1 + Research G2 ne tolèrent aucune lacune |
| **Phase A** typiquement implém simple ou scope-cut | `high` | Solutions connues (carry finding, bump dep, wire primitive existante) |
| **Phase B/C/D** implém code métier | `xhigh` | Default. Balance deep-thinking vs coût |
| **Phase D/E** crypto, security, threat-model-critical, federation, wire protocol | `max` | Surface d'erreur catastrophique (CVE, backdoor, wire break pre-launch). Vaut le surcoût |
| **Phase F** wrap-up docs + verification + audit_plan | `high` | Mécanique répétitive, peu de décisions. `medium` acceptable si confiance haute |
| **Chore(planning)** split CRAFT pre-phase | `medium` | Stage + commit template, zéro décision nouvelle |
| **Fix post-audit** P0/P1 isolé | `high` | Root cause bien défini par le finding |
| **Hotfix Cas D** hors sprint | variable | Dépend de la criticité — un fix segfault iroh = `max`, un typo doc = `medium` |

### 1.3 Protocole de toggle

Début de phase, user ou Claude annonce :

```
Phase X ouverture — effort: <niveau>
```

Puis `/effort <niveau>` avant le premier tool call de la phase.
Fin de phase (post-commit), restaurer à `xhigh` par défaut :

```
/effort xhigh
```

**Ne jamais** descendre sous `high` sur une phase qui touche du
code sécurité ou wire format, même en wrap-up. `medium` est
réservé aux docs pures.

### 1.4 Anti-patterns

- **Bascule silencieuse** : changer d'effort mid-phase sans
  annonce — les commits perdent leur traçabilité. Un commit feat
  Phase C doit avoir été produit à un effort cohérent début-fin.
- **Économie sur Phase 0** : tenter d'économiser sur un audit gate
  = casse le contrat du pattern permanent (`docs/claude/README.md §3`).
- **`max` sur tout** : inverse du problème, on explose le budget
  sans raison sur des chore commits.

## 2. Baseline A/B Opus 4.6 vs 4.7 — protocole de mesure

### 2.1 Contexte

Opus 4.7 est sorti avec des gains benchmarkés (SWE-bench Verified
+6.8pp, SWE-Pro +10.9pp, CursorBench 58→70) mais une régression
MRCR confirmée system card (-32.7pp @256K / -46.1pp @1M). Pour un
projet comme SBFB qui charge ~40-50k tokens de contexte cross-
session (MEMORY.md + CLAUDE.md + docs/claude/* + .planning/active/),
la régression MRCR peut importer plus que le gain SWE-bench.

Avant de basculer définitivement, **mesurer** sur phases réelles
plutôt que faire confiance aux benchmarks génériques.

### 2.2 Protocole

Sprint 22 phases restantes (C Sybil, D NVML, E watermark, F wrap)
servent de cohorte test. Alternance :

- **Phase C** — Opus 4.6 (baseline)
- **Phase D** — Opus 4.7
- **Phase E** — Opus 4.6
- **Phase F** — Opus 4.7

Pour chaque phase, **logger** dans la table ci-dessous :

| Métrique | Source |
|---|---|
| Tokens input (total session) | Claude Code `/cost` ou telemetry |
| Tokens output (total session) | idem |
| Coût USD session | idem |
| First-pass success (commit passe §7.4 sans fix) | oui / non |
| Nombre de fixes post-commit nécessaires | `git log --oneline` after phase feat commit |
| Nombre de refus cyber (safeguard triggers) | notes manuelles |
| MRCR self-test score (`MRCR_SELFTEST.md`) | 0-3 |
| Temps wall-clock user→commit | notes manuelles |

### 2.3 Règle de décision post-S22

Après les 4 phases :

- Opus 4.7 gagne **si et seulement si** : coût ≤ 110% Opus 4.6
  **ET** first-pass success ≥ 4.6 **ET** 0 refus cyber sur
  phases sécurité **ET** MRCR self-test ≥ 2/3 moyen.
- Sinon : rester sur Opus 4.6 par défaut jusqu'au prochain tag
  Opus qui corrige MRCR.

Mi-chemin (après Phase D), si Opus 4.7 montre déjà un delta coût
> 130% ou 2+ refus cyber sur phase sécurité, **abandonner** le
test et repasser Opus 4.6 pour Phase E/F (priorité = livrer
Sprint 22, pas mesurer).

### 2.4 Table de mesures (à remplir)

| Phase | Modèle | Tokens in | Tokens out | Coût $ | First-pass | Fixes post | Refus cyber | MRCR | Temps | Notes |
|---|---|---|---|---|---|---|---|---|---|---|
| C | Opus 4.6 | — | — | — | — | — | — | — | — | — |
| D | Opus 4.7 | — | — | — | — | — | — | — | — | — |
| E | Opus 4.6 | — | — | — | — | — | — | — | — | — |
| F | Opus 4.7 | — | — | — | — | — | — | — | — | — |

### 2.5 Notes

- Ne pas comparer à travers des phases de nature très différente
  (Phase C = cryptographie distribuée, Phase F = docs wrap-up).
  L'alternance proposée couple chaque modèle à 1 phase lourde +
  1 légère pour neutraliser le biais.
- Le « refus cyber » observé S22 sur Opus 4.7 (anecdotes commu-
  nauté GitHub/HN, non-mesuré officiel) peut obliger à soumettre
  un CVP appeal. Candidater au besoin via le formulaire
  `claude.com/form/cyber-use-case` (délai 2 jours).
- Reboucler dans `docs/claude/README.md` §12 une note une fois
  la décision prise, avec le SHA de commit qui flipe le default.

## 3. Refs

- `docs/claude/MRCR_SELFTEST.md` — test binaire fidélité contexte
- `docs/claude/README.md §3` — pattern permanent audit gate (Phase 0 = `max` non négociable)
- `~/.claude/settings.json` — `effortLevel` global actuel
- Anthropic Opus 4.7 system card — chiffres MRCR / SWE-bench
