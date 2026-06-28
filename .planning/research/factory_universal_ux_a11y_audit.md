> Statut : note d'audit hors-sprint (2026-06-28). Produite par un Workflow
> ultracode (6 agents Opus 4.8 1M, ~624k tokens). Pilote la refonte du front
> Factory Operator vers un outil universel/simple/accessible (WCAG 2.2 AA).
> Réaligne sur le Day-0 « intentions-pas-jargon » qui avait dérivé vers un
> cockpit dev. Aucun lot n'enfreint les invariants porteurs (0-verdict-UI,
> anti-PASS, restitution, CSP self, 0-dep).

# Audit UX/A11y Factory — vers un Factory universel

## Résumé exécutif

Le front Factory Operator est aujourd'hui un **cockpit d'ingénierie dense** — mono partout, texte sous 14px à ~99 %, encres `tx3`/`tx4` sous le seuil WCAG, jargon de procédé (`STEER`/`VERIFY`/`gates`/`préflight`/`diff`) exposé en façade, aucune structure de titres ni repère `main`, sens porté par la seule couleur, et **aucune porte d'entrée orientée but** (l'intention « Créer une app » n'existe même pas). Il échoue plusieurs critères WCAG 2.2 AA et s'adresse de fait à un mainteneur SBFB, pas à « tous ». La direction de réalignement est triple et **n'enfreint aucun invariant porteur** : (1) rendre la couche de restitution **lisible et audible** (taille ≥16px corps, contraste AA, sémantique HTML, libellés hors-couleur, `aria-live`) ; (2) **traduire le jargon** en langage universel via les points-source uniques (`catalog/*.ts`, `verdict.ts`, `gateStatus.ts`) ; (3) ajouter un **Home orienté intentions** (Créer / Vérifier / Reprendre / Voir où j'en suis) qui repousse toute la télémétrie git/gates/procédé derrière un « Mode avancé ». Le seul point fort déjà exemplaire est la gestion `prefers-reduced-motion` (`index.css:104-110`) — à conserver tel quel.

## Constats par axe

### a11y (WCAG 2.2 AA — échecs structurels)
- **Aucun titre h1–h6 dans toute l'app** (1.3.1) : tous les intitulés sont des `span`/`div` stylés — `SteerScene.tsx:15-22`, `VerifyScene.tsx:108-110`, `SurfaceHost.tsx:42`, `OrientationBar.tsx:71`, `Mur.tsx:45`. Navigation par titres impossible.
- **Aucun `<main>`** (1.3.1 / repères) : le panneau focal est une `div` (`App.tsx:79`) ; `header` et `nav` existent mais plusieurs `section[aria-label]` créent des régions sans repère principal qui les chapeaute. Pas de lien d'évitement alors que le rail précède le contenu.
- **Sens porté par la seule couleur** (1.4.1) : marqueurs `+/-/espace` du diff masqués `aria-hidden`, distinction ajouté/supprimé = couleur de fond seule (`DiffViewer.tsx:138-142,180,194`) → lignes indistinguables au lecteur d'écran ; idem frise verdicts (`ProcedeSurface.tsx:588-593`), scope cuts (`:651`), pouls gates (`OrientationBar.tsx:41-48`).
- **Messages de statut non annoncés** (4.1.3) : aucune région `aria-live`. Statut du tour (`Atelier.tsx:60-62`), état VERIFY (`VerifyScene.tsx:172-178`), confirmation « copié » (`ContextPackInspector.tsx:90,99`) restent muets ; le curseur `▌` (`Atelier.tsx:89`) sera lu comme caractère.
- **Raccourci une-lettre `s`/`v` global** (2.1.4 A) sans désactivation/remap/limitation au focus (`useFocalKeys.ts:12-22`).
- **Focus invisible** sur 3 champs (`Composer.tsx:87,98`, `ProcedeSurface.tsx:609`) et la zone diff (`DiffViewer.tsx:343-350`) via `outline-none` (2.4.7/2.4.11) ; pourtant le ring global est bon (`index.css:74-77`, 7.3:1).
- **Cibles tactiles < 24px** (2.5.8) : ~10 commandes secondaires à `py-0.5` (~15px) — `OrientationBar.tsx:115`, `GatesPanel.tsx:86,96`, `VerifyScene.tsx:116`, `DiffViewer.tsx:280,291,299,383`, `SessionsSurface.tsx:107`, `ContextPackInspector.tsx:97`.
- **`aria-expanded` manquant** sur le toggle de phase (`ProcedeSurface.tsx:84-101`) alors que ses voisins l'ont (`:278,415`).
- **Acquis à garder** : `lang="fr"`, titre de page, `role="alert"` (`Mur.tsx:28`), `role="status"` (`App.tsx:56`), `reduced-motion` complet.

### contraste (1.4.3 / 1.4.11 — quantifié, tokens vérifiés `index.css:35-46`)
- **`tx3` (L 0.540)** sur s0/s1/s2/s3 = **3.81 / 3.54 / 3.21 / 2.79:1** → FAIL texte normal partout (seuil 4.5). Porte statuts, libellés de phase, chemins, valeurs.
- **`tx4` (L 0.450)** = **2.59 / 2.41 / 2.18 / 1.90:1** → FAIL total (même 3:1). Le pire : badge `bg-s3 text-tx4 text-[8px]` (`VerifyScene.tsx:40`) à **1.90:1**. Omniprésent sur en-têtes de section, captions, timestamps, SHA, chemins.
- **`bad`/`bad-bg`** = **4.30:1** → FAIL normal de justesse (`App.tsx` bannière d'erreur).
- **Bordures de champ** : `bd2`/s1 = **2.0–2.4:1**, `bd`/s0 = **1.6:1** → seul indicateur du champ, viole 1.4.11.
- **`neu` (L 0.540)** = clone de tx3, **3.54:1** si routé en texte (`verdict.ts`).
- **Conformes (à ne pas toucher)** : `tx`, `tx2`, `ok`, `warn`, `bad`/surfaces, `info`, `mur`.

### jargon (parle dev, viole « intentions-pas-jargon » hors du seul CTA)
- **Modes en anglais cryptique exposés en gros** : `STEER`/`VERIFY` (~10 fichiers + clavier `s`/`v`), `mode focal`, `inspecteurs`, `gates`, `change-set`, `hunks`, `loopback`, `PTY`, `head`, `run@rev`, `backend`, `buffer`.
- **Métalangage de procédé brut** : `préflight`, `review`, `codex`, `verdict`, `scope cut`, `dette portée`, `frise`, `wrap-up`, `artefact .planning/`, `restitué`, `agrégat`, `Δ Rust/Vitest`, `lint/err/warn`, `chore`, `findings`.
- **Codes de chantier dans l'UI** : `S81`, `dégradé S81`, `§1`, `verification.md`, `.planning/`, `marqueur de gate par fichier · dégradé S81` (`VerifyScene.tsx:37,42`, `DiffViewer.tsx:337`).
- **Glyphe `◦` surchargé 3 sens** (scope respecté / dérive fichier / diff tronqué) — à dissocier.
- **Acquis** : « où on en est », « barrière de gouvernance », « Connaissance consultative », messages d'erreur clairs → à généraliser, pas réinventer.

### simplicité / charge cognitive
- **5 destinations, 2 grammaires de nav** : STEER/VERIFY se *togglent* (`Rail.tsx:81-88`), les 3 inspecteurs *s'ouvrent par-dessus* (`SurfaceHost.tsx:30-44`) — incohérence en soi.
- **~18 atomes de chrome télémétrique permanent** avant tout contenu : barre d'orientation (~11 atomes, `OrientationBar.tsx:63-132`) + rail (`Rail.tsx:69-130`). ~25 éléments dès l'arrivée sur STEER vide.
- **`ProcedeSurface.tsx` = dashboard mainteneur complet** (frise EXECUTE/PLAN-ADAPT, bilan tests Rust/Vitest, Δ tests, codex `✓~⚠`, table vérification §1, timeline commits, scope cuts, dette, légende glyphes — `:231-392,580-661`) : zéro pertinence « créer une app ».
- **Aucune intention « Créer une app »** : les 3 presets (`catalog/intentions.ts:25-44`) sont `preflight`/`phase-review`/`handoff` — les verbes du process dev, pas des buts utilisateur. Écart le plus grave vs la directive.
- **Bascule STEER→VERIFY guidée par un point de 6px** (`Rail.tsx:42-49`) : le non-technique reste bloqué sans savoir que la suite est dans VERIFY. (Invariant « jamais d'auto-switch » sain — il manque le **guidage explicite**.)
- **Aucun écran d'accueil/premier-lancement** (`main.tsx:11-17` rend le shell bi-focal direct).

### typographie (le cockpit dans la fonte)
- **Aucune échelle tokenisée** : `index.css:50-52` ne définit que les familles. Chaque composant écrase via `text-[Npx]` arbitraire (189 instances).
- **~85 % du texte < 12px ; ~99 % < 14px ; 0 % du texte de lecture n'atteint 16px.** 33 instances à **8/8.5px**. Champ de saisie d'intention à **13.5px** (`Composer.tsx:87`, sous 16px → zoom auto iOS). Boutons d'intention (le geste produit central) à **12.5px** (`Composer.tsx:63`).
- **Contrat de fonte inversé** : `font-mono` 127× sur 20 fichiers ; le commentaire `index.css:50` dit pourtant « sans = intention, mono = preuve ». ~80 usages mono abusifs (statuts humains, prose, labels) → look terminal anxiogène.
- **Pattern « micro-eyebrow » toxique** (cumul 4 pénalités) : `text-[8.5px]` + `uppercase` + `tracking-[0.14–0.16em]` + `text-tx4` — `Rail.tsx:75,96`, `ProcedeSurface.tsx` (10×), etc. Le letter-spacing positif à 8.5px détruit la reconnaissance des glyphes.

## Glossaire clair

Appliquer mécaniquement via les points-source uniques (`catalog/intentions.ts`, `catalog/surfaces.ts`, `verdict.ts`, `gateStatus.ts`). Colonne **Surface** : `Principale` = visible par tous · `Avancé` = sous disclosure/survol (traçabilité préservée) · `Supprimer` = note de dev hors UI utilisateur.

| Terme actuel | Langage universel | Surface |
|---|---|---|
| `STEER` | **Travailler** (« Demander à l'agent ») | Principale |
| `VERIFY` | **Vérifier** (« Examiner le travail ») | Principale |
| `mode focal` / `bascule bi-focal` | onglets simples, libellé retiré | Principale |
| `prêt à examiner` (point 6px) | **Travail terminé — prêt à vérifier** (bandeau guidé) | Principale |
| `inspecteurs` | **Consulter** | Principale |
| `Procédé` | **Historique du travail** | Principale |
| `Sessions` | **Journal & enregistrements** | Principale |
| `Knowledge` | **Connaissances** / **Documents de référence** | Principale |
| `gates` / `pouls des gates` | **contrôles** / **état des contrôles** | Principale |
| `diff` / `change-set` / `hunks` | **changements** / **fichiers modifiés** / **blocs de modification** | Principale |
| `intention` (la demande) | **demande** | Principale |
| `composeur d'intention` | **Votre demande** | Principale |
| `Lancer l'intention` | **Démarrer** | Principale |
| `atelier observable` / `tour` | **Travail en direct** / **réponse** | Principale |
| `barrière de gouvernance` / `MUR` | **Barrière de sécurité** | Principale |
| `intention sensible détectée` | **action protégée** | Principale |
| `verdict` / `verdict auto-clos` | **conclusion** / **aucune conclusion automatique** (jamais « PASS ») | Principale |
| `tenue` (état gate) | **réussi** (état de contrôle, jamais « validé/approuvé » global) | Principale |
| `hors périmètre` (gate) | **non concerné** | Principale |
| `committée` / `indexés` | **enregistrée** / **prêts à enregistrer** | Principale |
| `loopback` / `Operator` / `nœud` | **connexion locale** / **votre nœud** | Principale |
| `agent : claude · cloud` | **moteur : Claude (en ligne)** | Principale |
| `pack scellé` | **dossier verrouillé à transmettre** | Principale |
| `frise` | **vue d'ensemble** | Principale |
| `préflight` / `review` | **préparation** / **revue** | Avancé |
| `codex` | **revue externe** | Avancé |
| `EXECUTE / PLAN-ADAPT / DESIGN-CONFLICT / CONCERN / FAIL` | **feu vert / plan ajusté / conflit de conception / réserve / échec** (mot brut en survol) | Avancé |
| `scope cut` / `dette portée` | **hors périmètre (assumé)** / **points à reprendre** | Avancé |
| `kind` / `provider` | **type de procédé** / **moteur** | Avancé |
| `Δ +Rust · +Vitest` | **tests : +N (Rust) · +N (interface)** | Avancé |
| `run@rev` / `head` / SHA | **version <id>** / **version actuelle** | Avancé |
| `empreinte blake3` / `inliné` / `non-autoritaire` | **empreinte numérique** / **jamais recopié** / **à titre indicatif** | Avancé |
| `lint planning · err · warn` | **contrôle des documents · erreurs · avertissements** | Avancé |
| `Override` / `Bypass` / `no-op` / `buffer` | **contournement** / **passage en force** / (retirer) / (retirer) | Avancé/Supprimer |
| `S81` / `dégradé S81` / `§1` / `verification.md` / `.planning/` / `PTY` / `.cast` / `WS`/`SSE` | (notes de dev) | Supprimer |
| `marqueur de gate par fichier · dégradé` | (fonction non finie) | Supprimer |
| glyphes seuls `⊢ ≣ ◇` | icône **+ libellé texte visible** | Principale |
| `◦` (3 sens : respecté/dérive/tronqué) | dissocier en 3 libellés distincts | Principale |

## Échelle typographique + tokens contraste corrigés

### Tokens contraste (drop-in `index.css`, esthétique sombre préservée)
Le problème est **structurel** : 4 gris ne peuvent pas tous être à la fois lisibles à 4.5:1 sur fond sombre ET visuellement distincts. Réponse alignée « universel & simple » → **3 tiers d'encre lisibles + 1 ghost explicitement décoratif + 1 token bordure de champ**.

```css
/* Ink — 3 niveaux LISIBLES (AA normal sur s0–s2) + 1 ghost déco */
--color-tx:  oklch(0.930 0.004 260);  /* inchangé — 11.5–15.7:1 */
--color-tx2: oklch(0.720 0.005 260);  /* inchangé — 5.7–7.8:1   */
--color-tx3: oklch(0.700 0.006 260);  /* 0.540 -> 0.700 : s0 7.21 · s1 6.71 · s2 6.07 · s3 5.28 (PASS) */
--color-tx4: oklch(0.630 0.006 260);  /* 0.450 -> 0.630 : s0 5.3 · s1 4.9 · s2 4.5 ;
                                          RÔLE RESTREINT : décor non-informatif SEUL
                                          (aria-hidden, gouttières, séparateurs · ▸).
                                          Tout texte PORTEUR aujourd'hui en tx4 -> tx3. */
--color-neu: oklch(0.700 0.006 260);  /* aligné sur tx3 (était 0.540) */

/* Bordure de CONTRÔLE (input/textarea/bouton) — non-text 3:1 */
--color-field: oklch(0.550 0.008 260); /* nouveau : s0 3.97 · s1 3.69 · s2 3.34 (PASS 3:1).
                                          bd (0.330) / bd2 (0.430) RESTENT pour séparateurs
                                          de panneaux décoratifs (exemptés WCAG). */

/* Alerte */
--color-bad-bg: oklch(0.27 0.055 27);  /* 0.31 -> 0.27 : bad/bad-bg = 4.90:1 */
```
**Migration `text-tx4`** : `grep` exhaustif → déco (aria-hidden, séparateurs, gouttières `select-none`) reste `tx4` ; tout porteur (timestamps, chemins, SHA, captions, en-têtes de section) → `tx3`. **Bordures** : `border-bd`/`border-bd2` → `border-field` sur les contrôles de saisie/boutons uniquement.

### Échelle typographique (tokeniser `@theme` dans `index.css` — base 16px)
Min absolu de lecture = **13px** ; corps = **16px** ; champ de saisie ≥ **16px** (obligatoire, évite le zoom iOS) ; **abolir** le pattern caps tracké 8.5px.

| Rôle | Taille | Fonte / poids | Usage |
|---|---|---|---|
| Titre de scène | **20px** | sans semibold | h1 STEER/VERIFY, titre Mur |
| Titre de carte | **18px** | sans semibold | h2 Conformité / Knowledge / inspecteurs |
| **Corps (défaut)** | **16px** | sans regular | prose, **intentions/boutons**, statuts humains, **champ de saisie** |
| Secondaire | **14px** | sans | libellés secondaires, légendes |
| Légende / méta | **13px** | sans | compteurs, horodatages (plancher, jamais en dessous) |
| Eyebrow / section | **12px** | sans medium, **tracking ≤ 0.02em, sentence-case** (ou small-caps via `font-feature-settings`, PAS `uppercase`+letter-spacing), couleur ≥ tx3 | remplace TOUT le pattern 8.5px caps tracké |
| Code / diff / terminal | **13px** | **mono** | plancher mono lecture |
| Puce technique (sha/hash/path/gate-id) | **12px** | **mono** | tokens inline |

**Mapping mécanique des 189 classes** : `8/8.5px`→12px (déco) ou 13px (porteur), supprimer `uppercase`+tracking ; `9–9.5px`→12–13px ; `10–10.5px`→13px (méta) ; `11–11.5px`→14px ; `12/12.5px`/`text-xs`→16px corps (intentions, statuts) ; `13/13.5px`→16px ; `15px`→20px ; `text-base` (glyphe `Mur.tsx:39`) conservé. **Reclassement fonte conjoint** : garder mono pour diff/terminal/SHA/path/gate-id/`tabular-nums` ; basculer sans pour ~80 statuts/prose/labels humains.

## Architecture proposée

### Porte d'entrée : un Home orienté intentions AVANT le cockpit
`App` rend par défaut un **Home** ; le bi-focal STEER/VERIFY devient un niveau *interne* atteint via une intention. Le chrome télémétrique (barre git, rail STEER/VERIFY) **n'apparaît pas** sur le Home.

**4 grandes cartes-but** (gros boutons, langage humain, icône **+ libellé visible**, police ≥16px, encre `tx`/`tx2`, fond `s1`/`s2`) :
1. **Créer une app** → session d'authoring sur le kind `app-authoring` (déjà côté backend S79), pas `preflight`. *Manquant aujourd'hui — à ajouter au catalogue.*
2. **Vérifier mon travail** → VERIFY (changements + contrôles restitués), libellé humain.
3. **Reprendre** → reprise de session (déjà câblé `SessionsSurface.tsx:77-131`), promu au rang de but au lieu d'être enfoui.
4. **Voir où j'en suis** → vue *condensée* « où on en est » (réutilise `LiveProcessBanner`, `ProcedeSurface.tsx:191-229`), **pas** le dashboard complet, **jamais** un PASS/score.

### Flux guidé (ex. « Créer une app »)
Home → carte → **un seul champ** « Décrivez l'app que vous voulez » (variante grande de `Composer`, sans les 3 chips de process ni le select moteur en façade) → **Démarrer** → Travail en direct (`Atelier`, fil clair « l'agent travaille / terminé ») → en fin de tour, **bandeau guidé explicite** « C'est prêt — vérifier le résultat ▸ » (remplace le point 6px ; bascule **manuelle** par un vrai bouton, invariant D6 respecté).

### Ce qui reste en façade
Home 4 cartes · le champ de demande · le Travail en direct · le bandeau guidé · la **Barrière de sécurité** (re-titrée, gravité conservée, **zéro** affordance Forcer/Override, `Mur.tsx:53-55`) · l'honnêteté « aucune conclusion automatique » (`verdict.ts` `VERIFY_ETAT`, déjà non-PASS).

### Ce qui se replie (« Mode avancé » / disclosure — accessible en 1 clic, rien supprimé)
Branche git + modifiés/indexés (`OrientationBar.tsx:75-103`) · pouls gates par statut (`:104-105`) · Sprint/phase/`run@rev`/`head` · **tout le dashboard Procédé** (frise, bilan tests, Δ, codex, table §1, timeline, scope cuts, dette, légende — `ProcedeSurface.tsx:231-392,580-661`) en gardant seulement « où on en est » en façade · sha/hunks/word-diff/minimap (VERIFY garde « X fichiers changés » + « Voir le détail ») · kind/provider/prompt assemblé (déjà replié — bon) · empreintes blake3 / context-pack · journal/casts/terminal (sauf « Reprendre » promu). **Supprimer (pas replier)** les onglets morts « à venir (S81) » (`VerifyScene.tsx:127-128`) — réintroduire quand livrés.

## Plan priorisé

**P0 = empêche un usage universel aujourd'hui** (taille, contraste, langage, sémantique, sens hors-couleur). Effort : S ≈ ≤0,5 j · M ≈ 1–2 j · L ≈ 3–5 j.

| Prio | Lot | Effort | Impact |
|---|---|---|---|
| **P0** | **L1 — Tokens contraste** : `tx3`→0.700, `tx4`→0.630 (rôle déco), `field` nouveau, `bad-bg`→0.27 ; migration `text-tx4` déco↔porteur ; bordures contrôles→`border-field` | M | WCAG 1.4.3 + 1.4.11 résolus ; lisibilité globale. Le plus rentable. |
| **P0** | **L2 — Échelle typo tokenisée** : `@theme --text-*`, plancher 13px, corps 16px, **champ saisie ≥16px**, bannir 8/8.5px porteur, supprimer caps tracké ; balayage des 189 classes | L | Sort ~99 % du texte du sous-14px ; cœur de l'accessibilité basse-vision. |
| **P0** | **L3 — Sémantique HTML** : h1 par scène + h2 par section/inspecteur, `<main>`, lien d'évitement | M | WCAG 1.3.1 + 2.4.1 ; navigation lecteur d'écran possible. |
| **P0** | **L4 — Sens hors-couleur** : libellés texte/`sr-only` « ligne ajoutée/supprimée », gates, frise, scope cuts ; `aria-expanded` manquant ; dissocier `◦` (3 sens) | M | WCAG 1.4.1 + 1.1.1 ; daltonisme + lecteur d'écran. |
| **P0** | **L5 — États vivants** : `aria-live` polite sur statut du tour + état VERIFY + « copié » ; masquer le curseur `▌` | S | WCAG 4.1.3 ; tient la promesse « atelier observable » à l'oral. |
| **P0** | **L6 — Glossaire universel** : renommer STEER/VERIFY + glossaire central (`catalog/*.ts`, `verdict.ts`, `gateStatus.ts`) ; supprimer codes chantier (S81/§1/.planning) | L | Compréhension par tous ; honore « intentions-pas-jargon » partout, pas que le CTA. |
| **P0** | **L7 — Focus + raccourcis** : restaurer ring visible (3 champs + zone diff) ; borner/désactiver `s`/`v` au focus | S | WCAG 2.4.7 + 2.1.4. |
| **P0** | **L8 — Cibles tactiles** : ~10 commandes secondaires ≥ 24×24px | S | WCAG 2.5.8 ; usage tactile/motricité. |
| **P1** | **L9 — Home orienté intentions** : 4 cartes-but + ajout intention « Créer une app » (kind `app-authoring`) + bandeau guidé STEER→VERIFY | L | Débloque le parcours « créer une app » inexistant ; point d'entrée pour non-techniques. |
| **P1** | **L10 — Progressive disclosure « Mode avancé »** : replier télémétrie git/gates + dashboard Procédé ; garder « où on en est » en façade ; supprimer onglets morts | L | Réduit ~25→~6 éléments à l'arrivée ; lève la charge cognitive du cockpit. |
| **P1** | **L11 — Mono→sans** : ~80 usages langage humain (statuts, prose, labels) ; sans façade, mono réservé preuve | M | Sort du registre « console » ; honore le token `index.css:50`. |
| **P2** | **L12 — Reflow/zoom** : vérifier 320px / 400 % ; corriger rail fixe 158px + `overflow-hidden` + colonnes fixes | M | WCAG 1.4.10 + 1.4.4. |
| **P2** | **L13 — Légende glyphes toujours visible** : hors du seul Procédé ; `aria-label`/`title` sur chaque symbole | S | Lecture rapide + lecteurs d'écran. |
| **P2** | **L14 — Traduction verdicts bruts en survol** : EXECUTE→feu vert, etc. (mot brut conservé en survol pour traçabilité) | S | Lisibilité du procédé sans perte de traçabilité. |

**Séquencement conseillé** : L1+L2 d'abord (tokens + échelle, transverses, débloquent tout le reste visuellement), puis L3–L8 en parallèle (sémantique/a11y atomiques), puis L6/L11 (langage, sur les points-source), enfin L9/L10 (architecture) et P2.

## Invariants préservés

Aucun lot ne touche aux invariants porteurs — la simplification = **langage + lisibilité + guidage + progressive disclosure**, jamais le sens :
- **0-verdict-calculé-UI** : la couche de restitution est rendue lisible/audible, jamais calculatrice ; « Voir où j'en suis » restitue l'état observable, ne le fabrique pas.
- **anti-PASS** : `tenue`→« réussi » reste un **état de contrôle restitué**, jamais un « validé/approuvé/PASS » global ; aucun mot PASS introduit ; les verdicts traduits gardent leur mot brut en survol.
- **restitution (la UI restitue, ne fabrique pas)** : gates/verdicts restent restitués 1:1, jamais agrégés (clé `(gate,status)`) ; masquer la télémétrie ≠ mentir (tout reste consultable en 1 clic « Mode avancé »).
- **CSP self** : aucun CDN/origine externe ajouté ; Geist sans/mono déjà vendorées (`index.css:8-9`).
- **0-dep lourde** : aucun ajout — changements purement tokens/classes/fonte/sémantique HTML/`aria-*`.
- **D6 (bascule manuelle, jamais auto-switch)** : le bandeau guidé reste un **bouton manuel**, pas un auto-switch.
- **`prefers-reduced-motion`** : déjà exemplaire (`index.css:104-110`) — conservé tel quel.
