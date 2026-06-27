# Audit de finalisation — wireframes Factory Operator (S80)

> Vérification ultracode 2026-06-27 (Workflow `wf_e4ce35b6-30e`, 5 agents Opus 4.8 1M,
> ~516K tokens) déclenchée par le doute PO « les wireframes ne sont pas finalisés ».
> Artefact audité : `wireframes_factory_operator/Factory Operator - wireframes.dc.html`
> (971 lignes). **Live claude.ai/design == committé** (contenu identique, vérifié via MCP
> DesignSync ; seule différence = fins de ligne). Projet : `ee6278bc-…` « Factory Operator
> dev local ».

## Verdict : NON finalisés — PARTIEL (le PO a raison)

Deux axes à distinguer :

- **Couverture / inventaire low-fi : ~95-100 % COMPLET.** ~19 maquettes cadrées (Shell ;
  STEER 5 états ; MUR ; VERIFY 6 frames ; Terminal 2 ; secondaires 3) + diagramme d'états
  bascule + planche inventaire 16 vignettes + légende 18 atomes + variantes 2+2 + 8 états
  de gate honnêtes aux libellés exacts. Vrai contenu, 0 lorem/TODO, doctrine impeccable
  (0 « PASS », 0 coche verte, 0 bouton Approve/Merge/bypass).
- **Fidélité aux décisions DÉJÀ VERROUILLÉES du brief : ~60 % — NON finalisé.** Le dessin
  **contredit 3 décisions Q « validées PO 2026-06-26 »**, sur les **surfaces porteuses** :

| Décision Q (brief) | Dessin actuel | Statut |
|---|---|---|
| **Q4** barre d'orientation fine en HAUT | tout dans le rail gauche 212px, aucune barre haute | CONTREDIT |
| **Q5** 3 onglets [Diff·Aperçu·Preuve] + bande gates/ÉTAT permanente en BAS + marqueur gate par fichier | Gates = 4e onglet (interdit) ; seul bandeau permanent = ÉTAT en HAUT ; pas de marqueur par fichier | CONTREDIT |
| **Q6** MUR = barrière en-flux pleine largeur (pas un modal) | carte centrée 440px sur scrim = forme modale | CONTREDIT |
| Thème **Dark** (brief l.81) | tout en clair crème `#f6f6f4` (seul le terminal est sombre) | ABSENT |
| Variantes **B/B = les RETENUES** dessinées au niveau de A | STEER-B sans l'état vide-composeur-grand ; VERIFY-B = 1 frame vs A = 5 | PARTIEL |

**Aggravant** : les variantes **B/B retenues pour le build sont les moins développées** ;
tout le détail riche vit dans les variantes A écartées → la direction choisie est la moins
instruite.

**Hi-fi = étape séparée, entièrement absente** (normal pour un low-fi, mais reste à
produire) : 0 token oklch, 0 rampe typo, 0 spacing système, 0 état hover/focus/disabled,
0 motion.

## Manques bloquants avant de coder le FRONT (priorisés)

BLOQUANT (surfaces porteuses, à trancher avant la structure front) :
1. **Shell Q4** — barre haute (brief) vs tout-rail (dessin). Figer.
2. **VERIFY-B canonique Q5** — 1 frame conforme : change-set de fichiers repliable +
   **marqueur gate par fichier** + 3 onglets + **bande gates/ÉTAT permanente en BAS pleine
   largeur**. Point dur (porte « 0 verdict auto-clos non masquable »).
3. **MUR Q6** — barrière pleine largeur (brief) vs modal (dessin). Trancher.
4. **STEER-B** — état vide = composeur en grand (mitigation PO, absente).

NON bloquant (peut démarrer en parallèle) : auth (Phase A **DONE** `a5ace8d`), scaffold
layout, constantes copy/intentions/libellés d'états, câblage des 2 routes VERIFY
(`GET /api/git/diff`, `GET /api/gates`).

Différable (hi-fi, le brief l'autorise) : palette dark oklch, typo, spacing, états
d'interaction, 5 signatures motion → feuille de tokens, pas un pixel-perfect.

## Claude Design vs tout-Claude-Code (réponse PO)

Frontière nette, tenue par le verrou D2/D5 (sortie design re-thémée oklch ; maquette ni
câblée ni testée) :

- **Phase DIVERGENTE (forme : wireframe + hi-fi + tokens dark) → Claude Design.** Itération
  visuelle rapide, 0 coût d'intégration. Le patch de réconciliation 4 points + la feuille de
  tokens dark vivent ici.
- **Phase CONVERGENTE (build wiré + testé oklch + API réelle, React 19/Tailwind v4/Base UI,
  T1/T2) → exclusivement Claude Code.** La maquette est un **input de spec**, jamais un
  livrable de build.
- **Pas 100 % Design** : sortie clair-hex-inline, non oklch, non câblée, non testable →
  violerait D2/D5 + reproduirait les 3 écarts Q dans le code.
- **Pas 100 % Code** : coder la forme « extrêmement poussée » sans exploration visuelle ferait
  improviser au dev les surfaces disputées (shell, MUR, VERIFY-B) + le système dark — pile où
  le PO doute.

## Recommandation de séquencement

1. **NE PAS refaire les wireframes de zéro** — la largeur est acquise et solide.
2. **Patch de réconciliation (4 points) + feuille de tokens dark** en Claude Design (le brief
   le prescrit lui-même : « à verrouiller au preflight », l.179). Arbitrage de 4 points, pas
   une refonte. Alternative : acter par décision écrite que Q4/Q5/Q6 priment sur le dessin et
   coder selon Q — mais l'écrire, sinon le dev improvise.
3. **Démarrer en // les phases non-disputées** (scaffold, constantes copy/états, câblage
   routes VERIFY) — indépendantes du patch.
4. **Build en Claude Code** contre la spec réconciliée.
