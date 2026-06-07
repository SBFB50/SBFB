# Sprint 74 Phase A — Review (multi-dimension adversariale)

## Verdict: PASS

Date: 2026-06-07
HEAD (pré-commit): `6acf638` (working tree, Phase A staged)
Méthode: Workflow adversarial 5 agents indépendants (1M ctx chacun) — fallback
de `nexus-phase-review-deep` (non enregistré). ~595k tokens, 128 tool-uses.
Verdict final: **PASS** (PASS-PENDING promu PASS après réconciliation Codex — voir §Codex reconciliation).

## Dimensions & verdicts (avant résolution)

| Dimension | Verdict | Findings |
|---|---|---|
| CORRECTNESS | CONCERN | 2 P2 + 1 P3 |
| SECURITY | PASS | 1 P2 (garde non testée) |
| SCOPE & DESIGN-FIDELITY | PASS | 2 P3 |
| TESTS | CONCERN | 2 P2 + 3 P3 |
| UX & A11Y & HONESTY | CONCERN | 1 P1 + 2 P2 + 1 P3 |

## Findings & résolutions (toutes traitées en-phase)

### P1 — Toggle « Garder en ligne » faux-actif (UX/HONESTY, verrou §8(5))
Le toggle lecture-seule était un `<button>` base-ui focusable, avec hover/focus,
mais `onPressedChange={() => {}}` avalait silencieusement les clics — exactement
l'anti-pattern « jamais un faux bouton actif » que la phase doit incarner.
**Résolu** : rendu `disabled` (toggleVariants applique `disabled:pointer-events-none`
→ plus de hover/focus/clic, hors tab-order) + `opacity-100` pour rester visiblement
ON + microcopy « Bientôt configurable ». Test renforcé : assert `toBeDisabled()` +
`aria-pressed="true"`. `AvailabilitySheet.tsx:209-228`.

### P2 — Label de fraîcheur sur-revendique pour les apps perso (UX, PO Q2)
Le panneau affichait « En ligne — joignable par tous » même quand `reachable`
vient du raccourci self→Reachable (faux-vert NAT). **Résolu** : pour `isOwn &&
online` → « En ligne (vu de ton noeud) » (arbitrage PO Checkpoint §11(2)).
`AvailabilitySheet.tsx`. Test : `availability_state_maps_*` couvre les 2 variantes.

### P2 — Bouton « Disponibilite » mal-libellé pour l'état `unknown` (UX)
`isOffline` ne distinguait pas `unreachable` de `unknown` → app jamais sondée
stylée « en ligne ». **Résolu** : bouton tri-state (reachable/unreachable/unknown)
+ `aria-label` d'état + `availabilityShortLabel()` partagé + StatusDot title
tri-state. `BrowsedProject.tsx`.

### P2 — Rappel hors-ligne lu une seule fois au mount (CORRECTNESS)
La route est `lazy()` sans `key` → pas de remount sur changement de `:projectId`.
**Résolu** : la dérivation de la dismission lit sessionStorage PENDANT le rendu
(clé par `project_id`) + `useReducer` force-recheck au dismiss — plus de
setState-in-effect (corrige aussi un lint error « cascading renders »).
`BrowsedProject.tsx`. NB : sous le modèle d'ownership Phase A (`isOwn = isLocal`,
un seul app « perso » possible car `project_id == node_id`), le scénario de drift
inter-app n'est pas atteignable ; le fix est correct et load-bearing pour la
Phase D (ownership précis multi-app).

### P2 — Rename incomplet : CommandPalette « Deployer » (SCOPE, PO Q8)
La commande `/deploy` de la palette restait « Deployer » alors que la nav et le
titre disent « Publier ». **Résolu** : `CommandPalette.tsx:186` → « Publier ».

### P2 — Garde XSS greffe-D non testée (SECURITY, defense-in-depth)
`isHttpsUrl(entry.repo_url)` correcte mais sans test. **Résolu** : 2 cas
BrowsedProject (`javascript:` → remote-placeholder, pas de redeploy ; `https://`
→ redeploy-fallen-app avec href /deploy encodé).

### P2 — Test toggle partiellement false-green (TESTS)
N'assertait que « no fetch ». **Résolu** : assert `toBeDisabled()` (la honnêteté
read-only, pas juste l'absence de mutation réseau).

### P2 — Rename AddCoordinatorDialog non testé (TESTS, surface la plus dense)
**Résolu** : `AddCoordinatorDialog.test.tsx` NEW assert « Se connecter a un
noeud » / « URL du noeud » / 0 « coordinateur ».

### P3 (traités)
- greffe-D mal-présentée sur erreur daemon-info d'une app archivée → branche gatée
  sur `!entry.archive_hash` (`BrowsedProject.tsx`).
- Copie d'invariant auteur + « Signature verifiee » conditionnel → assertions
  ajoutées (`AvailabilitySheet.test.tsx`).
- Chemin Reverifier (browsePull) non testé → test ajouté.
- aria-label du bouton Disponibilité → ajouté avec l'état.

### P3 (documentés, sans action — choix de conception assumés)
- Fallback « Pas encore verifie » (null `last_probed_at`) hors table §6 :
  gestion gracieuse d'un état non spécifié, acceptable.
- Icône SignalZero vs puce `(•)` en état offline : langage d'icônes cohérent
  (bouton top-bar + greffe-D + panneau), plus clair qu'une puce nue.
- Rename large (9 fichiers) sans assertions positives dédiées : couvert par le
  gate grep source-wide (0 « coordinateur » hors commentaires/tests) + les 2
  négatifs `queryByText(/coordinateur/i)` + AddCoordinatorDialog/empty-wall tests.

## Suites (toutes vertes après résolution)
tsc 0 · eslint 0 erreur (5 warnings pré-existants ui/) · scan-en-strings clean ·
Vitest **309** (28 fichiers, +15 vs 294) · build OK · size 6/6.

## Codex reconciliation

Codex (GPT 5.5, `codex exec -o sprint74_phase_a_codex_review.md`) a audité les 11
livrables : **10 CONFIRME, 1 GAP**. Codex a aussi exécuté les 5 fichiers de test
(36 passés) et vérifié les invariants (0 `.rs`, 0 route keep-online, 0 chaîne
visible « coordinateur » hors commentaires/tests, identifiants/persist key
conservés).

**GAP unique (livrable 4) — FAUX POSITIF / divergence délibérée** : Codex note
que le mapping `reachable` pour une app perso affiche « En ligne (vu de ton
noeud) » au lieu du « En ligne — joignable par tous » de la table §6. C'est
**intentionnel** : le label NAT honnête a été introduit en réponse au finding UX
P2 de la review (sur-revendication du faux-vert NAT self→Reachable) et correspond
à l'**arbitrage PO Q2 Checkpoint §11(2)** + design §9 (« vu de ton noeud » /
« visible seulement par toi pour l'instant »). Le prompt Codex citait la table §6
littérale (antérieure à l'arbitrage NAT), d'où le faux GAP. La table §6 est
respectée pour les apps **distantes** (« joignable par tous »), confirmée par
Codex. **Aucun changement de code** ; comportement plus honnête conservé.

**Verdict final : PASS** (10/11 CONFIRME ; 1 GAP réconcilié comme divergence
PO-arbitrée). Promu de PASS-PENDING à PASS.
