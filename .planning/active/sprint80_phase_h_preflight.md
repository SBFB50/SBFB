# Sprint 80 â€” Phase H â€” Preflight (deep, 6 scans + 3 adversariaux, finalisÃ© G8)

**Date** : 2026-06-28
**Sprint / Phase** : 80 / H â€” VERIFY-plein front (focale VERIFY variante B du front Factory Operator greenfield) : diff-viewer React MAISON sur les hunks JSON Rust + panneau gates 1:1 + slot Ã‰TAT machine d'Ã©tats nommÃ©e + bascule bi-focal STEERâ†”VERIFY. Front-pur (backends F `bb35d39` + G `ed00b4a` dÃ©jÃ  livrÃ©s). 0 dep runtime nouvelle.
**Verdict** : **PLAN-ADAPT**

> Le plan littÃ©ral est exÃ©cutable et compatible Day-0, mais corrigÃ© par 5 adaptations d'approche Ã©tayÃ©es par le code, sans toucher aucun invariant : (1) V4 ne restitue QUE les 5 `GateStatus` rÃ©els â€” PROVISIONAL/Not-evidenced/RIG-ABSENT relÃ¨vent de l'acceptance T2, les dÃ©river de `/api/gates` fabriquerait un verdict (viol cardinal) ; (2) Â« Zod `.nullable()` Â» = contrat nullable documentÃ© serveur, PAS une dep (zod absent) â†’ garder l'interface-cast `as T` ; (3) `run@<rev>` + `â—¦ obsolÃ¨te` DÃ‰RIVÃ‰S front depuis `diff.head` (GatesView ne porte ni rev ni timestamp) ; (4) bascule MANUELLE (D6) dont seule la DISPONIBILITÃ‰ devient Ã©tat-driven, jamais un auto-switch arrachÃ© au stream ; (5) budget : extraire le diff-viewer hors du chunk `VerifyScene` (~5,9 KB de marge SI) + nouvelle entrÃ©e `.size-limit.json`. La dimension V5/V6 est **SCOPE-CUT-CONSISTENT** (dÃ©grader/carry S81, chemin dÃ©jÃ  provisionnÃ© Â§5.1) englobÃ©e dans le PLAN-ADAPT global. Aucune Day-0 rÃ©futÃ©e â†’ pas de DESIGN-CONFLICT.

> Finalisation : brouillon rÃ©conciliÃ© avec 3 lentilles adversariales (exactitude wire + V5/V6 ; deps + budget ; cardinal + scope + threat). Faits load-bearing **re-vÃ©rifiÃ©s en main-thread** : `GateStatus` enum 5 valeurs (gates.rs:75-89) ; `GateIssueView{message,file:Option<String>,line:Option<u32>}` avec `line: None` en dur (gates.rs:96-101,160) + test fige `is_none()` (gates.rs:805-808) ; `GatesView{gates}` sans champ racine (gates.rs:111-121) ; `WorkingTreeDiff{head,unstaged,staged,truncated}` + `MAX_DIFF_LINES=20000` (sprint_history.rs:991,1003-1009) ; `App.tsx:63` rend `<VerifyScene />` SANS prop `op` ; `verdict.ts:21-25` `VERIFY_ETAT` machine nommÃ©e ; CSP `default-src 'self'; connect-src 'self'` sans `style-src` (operator_server.rs:354). **Litige budget tranchÃ© EMPIRIQUEMENT** : `npx size-limit` 12.1.0 (bytes-iec) affiche Â« 86.07 kB Â» pour 86072 B â‡’ base **SI/dÃ©cimale** â‡’ limit Â« 92 kB Â» = 92000 B â‡’ marge = **5928 B â‰ˆ 5,9 KB** (les correctifs adversariaux Â« 8136 B / 7,9 KB base 1024 Â» supposent le package legacy `bytes`, FAUX ; conclusion inchangÃ©e mais nombre corrigÃ© en faveur du brouillon).

---

## 1. SynthÃ¨se des 6 scans

| Scan | Objet | Verdict | Apport load-bearing (evidence) |
|---|---|---|---|
| **S0** Ã‰tat-front | Surfaces existantes + points de cÃ¢blage | EXECUTE-pur | `VerifyScene` rendu SANS prop `op` (App.tsx:63 ; SteerScene/SurfaceHost reÃ§oivent `op`) â†’ threader `op`. `VERIFY_ETAT` machine nommÃ©e Ã  Ã©tendre (verdict.ts:21-25). `toneBg/toneText` = classes Tailwind LITTÃ‰RALES (verdict.ts:68-83). 0 client `/api/git/diff` ni `/api/gates`. 0 dangerouslySetInnerHTML/innerHTML/eval (grep src vide). |
| **S1a** OSS prior-art | Diff-viewer + word-diff maison | EXECUTE-pur sous D2 | Word-diff = mini-LCS de 2 lignes appariÃ©es (del.content vs add.content), pas un re-diff JS ; Rust a dÃ©jÃ  classÃ©+strippÃ© chaque ligne (sprint_history.rs:1116-1146 ; DiffView.tsx:38-57). Virtualisation maison : collapse-par-fichier + CSS `content-visibility:auto` (0 JS) ; jamais jsdiff/@tanstack/virtual (D2). |
| **S1b** Deps / budget | CoÃ»t runtime + size-limit | EXECUTE deps / **CONCERN budget** | **0 dep runtime ajoutÃ©e** (zod/jsdiff/@tanstack absents). Budget MESURÃ‰ : `VerifyScene-DSqtvLQa.js` = 86072 B Â« 86.07 kB Â» / limit Â« 92 kB Â» SI = **5928 B â‰ˆ 5,9 KB** headroom (bytes-iec, base 1000). Insuffisant statiquement â†’ extraction + nouvelle entrÃ©e. |
| **S2** DÃ©cisions / Day-0 | ConformitÃ© D2/D3/D5/D6/cardinal | (recommande PLAN-ADAPT) | D2 (0-dep) tenu, D3 (motion confinÃ©e Phase E), D5 (oklch), D6 (bascule manuelle App.tsx:8) intacts. Cardinal 0-verdict-UI tenu (GatesView sans agrÃ©gat). Aucune Day-0 contredite. |
| **S3** Threat | XSS / CSP / MUR / scellÃ© | EXECUTE sous garde-fous | XSS : word-diff en noeuds texte React (interdire dangerouslySetInnerHTML). CSP self-origin sans `style-src` (operator_server.rs:354) â†’ classes littÃ©rales. Intentions de hunk soumises au MUR (requires_gate operator.ts:72-79). Onglets scellÃ©/Preuve disabled (0 fetch). |
| **S4** Wire / invariants | Shape F+G exacte | EXECUTE (miroir strict) | `WorkingTreeDiff{head,unstaged,staged,truncated}` (sprint_history.rs:1003-1009). `GatesView{gates:[GateEntryView]}` sans racine (gates.rs:111-121). `GateStatus` 5 valeurs (gates.rs:75-89). `GateIssueView.line==None` + `file`=basename .planning (gates.rs:96-101,157-161). **Condition V5/V6 Â§5.1 NON remplie**. |

---

## 2. Verdicts adversariaux intÃ©grÃ©s

| Lentille | Verdict | RÃ©conciliation |
|---|---|---|
| Exactitude wire + dÃ©cision V5/V6 (re-dÃ©rivÃ©e du code) | **CONFIRME** le brouillon | Chaque claim wire vÃ©rifiÃ© 1:1. RENFORCEMENT V6 : `file` des issues = basename nu (process.rs:458) vs path complet du diff (sprint_history.rs:1095) â†’ jointure encore PLUS faible. Seule correction proposÃ©e = marge budget (cf. ci-dessous). |
| Deps + budget size-limit | **CONFIRME** PLAN-ADAPT | 0 dep ajoutÃ©e vÃ©rifiÃ©e. Budget chiffrÃ© empiriquement par `npx size-limit` : 86.07 kB / 92 kB. Nuance : Â« Zod Â» est aussi dans le doc-comment Rust (sprint_history.rs:1001-1002) â†’ renforce Â« contrat nullable, pas une dep Â». |
| Cardinal 0-verdict-UI + scope + threat | **CONFIRME** PLAN-ADAPT | GatesView sans agrÃ©gat (gates.rs:112). `/sprint-history/diff/{sha}` authed + sha validÃ© (operator_server.rs:192,1302 ; `--end-of-options`). Reclasse la fermeture scellÃ©/Preuve en Â« Ã  attester au review post-code Â». RÃ©fute le risque CSP/GateFlip (CSSOM non gouvernÃ© par style-src). |

**Litige inter-adversaires (budget base 1000 vs 1024) â€” TRANCHÃ‰ en main-thread** : `size-limit` 12.1.0 importe `bytes-iec` (create-reporter.js:1 ; get-config.js:1,219 `bytes.parse`). bytes-iec traite Â« kB Â» en **SI (1000)** et rÃ©serve Â« KiB Â» Ã  l'IEC (1024). Preuve d'exÃ©cution : 86072 B s'affiche Â« 86.07 kB Â» (= 86072/1000), 37163 B Â« 37.16 kB Â», 20644 B Â« 20.64 kB Â» â†’ base dÃ©cimale confirmÃ©e. **Marge = 92000 âˆ’ 86072 = 5928 B â‰ˆ 5,9 KB.** Le brouillon (5,9 KB) et la Lentille 2 ont raison ; les Lentilles 1 et 3 (Â« 8136 B / 7,9 KB base 1024 Â») ont **tort** (elles supposaient le package legacy `bytes`). Conclusion identique (extraction obligatoire), nombre corrigÃ©.

---

## 3. DÃ©cision load-bearing V5/V6 â€” DÃ‰GRADER + carry S81 (GO-DEGRADE)

### 3.1 Struct wire EXACTE (re-vÃ©rifiÃ©e, gates.rs:96-101)
```rust
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct GateIssueView {
    pub message: String,
    pub file: Option<String>,   // peuplÃ© UNIQUEMENT pour lint-planning (basename .planning/*)
    pub line: Option<u32>,      // TOUJOURS None en S80 (gates.rs:160 `line: None`)
}
```
- `GateStatus` = 5 variantes snake_case : `not_run` / `not_applicable` / `passed` / `informational` / `blocking` (gates.rs:75-89). PAS de 6e valeur.
- `GatesView { gates: Vec<GateEntryView> }` SANS champ racine agrÃ©gÃ© (gates.rs:111-121, Â« NO aggregate field at the root Â»).
- `lint-planning` Ã©met 2 entrÃ©es (`blocking` errors + `informational` warnings) quand il a les deux â†’ **keyer `(gate, status)`**, jamais `gate` seul (gates.rs:116-117,165-178).

### 3.2 Pourquoi DÃ‰GRADER (condition Â§5.1 NON remplie)
- **V5** (pouls-gate-gouttiÃ¨re par-ligne) : `GateIssueView.line` est codÃ© en dur `None` (gates.rs:160), commentaire Â« line is always None in S80 Â» (gates.rs:93-95), test Phase G fige `i.line.is_none()` (gates.rs:805-808) â†’ **aucune ancre ligne**.
- **V6** (filtre-par-gate sur le change-set) + **V4-marqueur-gate-PAR-FICHIER-du-change-set** : `GateIssueView.file` peuplÃ© UNIQUEMENT pour lint-planning (gates.rs:157-161, depuis `LintDiagnostic.file` = basename nu `.planning/active/*.md`, process.rs:443-486) ; FG4-8 + CSP ont `issues: Vec::new()` (gates.rs:142-153) â†’ **aucune jointure exploitable** avec `FileDiff.path` (chemin complet du change-set, sprint_history.rs:1093-1096). MÃªme un match basenameâ†”path serait fragile.

### 3.3 LivrÃ© S80 vs carry S81
- **LIVRÃ‰ S80** : V4-core (panneau gates 1:1, 5 GateStatus en Ã©tats distincts jamais aplatis, keyÃ© `(gate,status)`, `issues.message` + `issues.file` en libellÃ© brut, U2 provenance-de-verdict cliquable) + V1/V2/V3 diff-viewer maison complets (bi-mode inlineâ†”side-by-side, word-diff intra-ligne, nav clavier, minimap densitÃ©).
- **CARRY P1 S81 (explicite)** : refactor `GateResult.issues -> struct {path, line?, message}` + rattachement gateâ†”fichier-du-change-set (+ `head` additif 0-bump sur `GatesView` pour une ancre `run@<rev>` native) â†’ dÃ©bloque V5 + V6 + V4-per-fichier-du-change-set. Touche le coeur publish (pipeline.rs/atelier.rs/Display/tests, non-dÃ©lÃ©gable S79) â†’ hors pÃ©rimÃ¨tre front Phase H.
- **CohÃ©rence** : active le chemin conditionnel dÃ©jÃ  Ã©crit au plan Â§5.1 (Â« sinon dÃ©grader/carry S81 Â») â†’ dimension **SCOPE-CUT-CONSISTENT** dans le PLAN-ADAPT global.

---

## 4. Plan de budget (chunk + headroom)

| EntrÃ©e | Fichier | MesurÃ© | Limit (SI) | Headroom |
|---|---|---|---|---|
| verify-surface | `VerifyScene-*.js` = 86072 B | Â« 86.07 kB Â» | Â« 92 kB Â» = 92000 B | **5928 B â‰ˆ 5,9 KB** |
| app (hero) | `index-*.js` = 37163 B | Â« 37.16 kB Â» | Â« 40 kB Â» = 40000 B | ~2837 B (~2,8 KB) |
| (ref) ProcedeSurface | `ProcedeSurface-*.js` = 16666 B | â€” | (pas d'entrÃ©e dÃ©diÃ©e) | â€” |

- **Base TRANCHÃ‰E** : size-limit 12.1.0 â†’ bytes-iec â†’ Â« kB Â» = SI (1000). `.size-limit.json:16-22` verify-surface Â« 92 KB Â» â‡’ 92000 B. Le chunk porte dÃ©jÃ  ~67 KB hÃ©ritÃ©s (lib Motion confinÃ©e Phase E, App.tsx:17-27).
- **Diagnostic** : diff-viewer bespoke V1-V3 + panneau gates V4 > 5,9 KB RAW â‡’ NE TIENT PAS statiquement. Le gate `size` est BLOQUANT (D3).
- **Action OBLIGATOIRE** : (a) EXTRAIRE le diff-viewer en module partagÃ© importÃ© STATIQUEMENT par `VerifyScene` ET `ProcedeSurface` (fold V2/U7 â†’ rolldown hoiste vers un chunk commun HORS VerifyScene) OU manualChunk dÃ©diÃ© `diff-viewer` dans `vite.config.ts` (qui ne porte aujourd'hui que react/xterm). (b) AJOUTER une entrÃ©e `.size-limit.json` mesurant le chunk extrait â€” sinon angle mort (P2 Phase E). (c) MESURER dÃ¨s le 1er jet (`npm run size`) et VÃ‰RIFIER que l'extraction HOISTE rÃ©ellement hors VerifyScene (pas une duplication dans 2 chunks).
- Si aprÃ¨s extraction VerifyScene reste serrÃ© : bump documentÃ©+chiffrÃ© de `verify-surface` autorisÃ© (budgets per-entrÃ©e â‰  Day-0).

---

## 5. Adaptations PLAN-ADAPT (evidence)

1. **V4 â€” 5 GateStatus stricts** : constante-miroir TS de l'enum Rust (`passedâ†’âœ“`, `blockingâ†’âœ•`, `informational/not_runâ†’â€¢`, `not_applicableâ†’â€”`, `N issues=issues.len()`), keyÃ© `(gate,status)`. PROVISIONAL/Not-evidenced/RIG-ABSENT (vocabulaire acceptance T2, artefact distinct) RETIRÃ‰S de la bande `/api/gates` â†’ les dÃ©river fabriquerait un verdict. *Evidence : gates.rs:75-89,116-117,142-188.*
2. **Â« Zod `.nullable()` Â» â†’ contrat nullable** : typer `WorkingTreeDiff` + `GatesView` via interfaces TS `| null` (convention `getJson<T>`/`as T`), pas zod (absent package.json + 0 import src/, malgrÃ© le doc-comment Rust). Type-guard maison ~15 lignes si validation runtime voulue ; JAMAIS `npm i zod` (2e dep runtime, tension D2). *Evidence : operator.ts:25-33,204-215 ; sprint_history.rs:1001-1002 ; package.json (0 zod).*
3. **FraÃ®cheur DÃ‰RIVÃ‰E front** : co-rÃ©cupÃ©rer `/api/gates` ET `/api/git/diff` dans le mÃªme cycle, estampiller le panneau gates avec `diff.head`, marquer `â—¦ obsolÃ¨te` quand le head courant diverge. `run@<rev>` = restitution de 2 revs serveur, pas un horodatage par-gate fabriquÃ©. `head` natif sur GatesView = carry S81 0-bump. *Evidence : gates.rs:111-121 (sans rev) ; sprint_history.rs:1005,998-999.*
4. **Bascule MANUELLE, disponibilitÃ© Ã©tat-driven** : la bascule STEERâ†”VERIFY reste manuelle (altitudeShift View-Transition native, App.tsx:8 Â« MODE switch is manual, never arrachÃ©e au stream Â») ; seule sa DISPONIBILITÃ‰ devient Ã©tat-driven (CTA activÃ© quand turn terminal ET `diff.head==head` courant ET gates frais). Jamais d'auto-switch silencieux. *Evidence : App.tsx:8,60-64 ; kickoff:163,266-267.*
5. **CÃ¢blage** : `getWorkingTreeDiff()`/`getGates()` au pattern `getJson<T>` ; threader `op` en prop Ã  `VerifyScene` (rendu sans prop App.tsx:63) pour router les intentions de hunk via `POST /api/chat/{id}/send` + restitution du MUR (requires_gate). Champ `file` (PAS `path`) sur GateIssueView ; `GateStatus` = constante nommÃ©e 5-valeurs. RÃ©utiliser `FileDiff`/`DiffLine` â†’ composant prend `files: FileDiff[]` (dÃ©nominateur commun working-tree ET commit, fold V2/U7). *Evidence : operator.ts:72-79,204-222,296-298 ; gates.rs:96-121.*

---

## 6. Scope (confirmation des cuts)

**DANS le scope S80 Phase H** :
- Diff-viewer React MAISON (V1 bi-mode inlineâ†”side-by-side + word-diff intra-ligne en `<span>` texte ; V2/U7 bi-usage working-tree + commit passÃ© `/sprint-history/diff/{sha}` ; V3 nav clavier + minimap densitÃ©) sur les hunks JSON Rust, 0 dep runtime.
- Panneau gates 1:1 V4-core (5 GateStatus distincts, keyÃ© `(gate,status)`, U2 provenance cliquable).
- Slot Ã‰TAT = machine d'Ã©tats nommÃ©e (extension de `VERIFY_ETAT`), scan anti-PASS BLOQUANT.
- Intentions de hunk routÃ©es Ã  la session (POST chat/send), soumises au MUR ; 0 Approve/Merge/Commit.
- FraÃ®cheur `run@<rev>` / `â—¦ obsolÃ¨te` dÃ©rivÃ©e front (diff.head).
- Bascule bi-focal MANUELLE dont la disponibilitÃ© est Ã©tat-driven.
- 2 clients TS (`getWorkingTreeDiff`/`getGates`) + threading `op` â†’ VerifyScene.
- Extraction diff-viewer + nouvelle entrÃ©e `.size-limit.json`.

**HORS scope (cut cohÃ©rent / carry)** :
- V5 (pouls-gate-gouttiÃ¨re par-ligne) + V6 (filtre-par-gate change-set) + V4-marqueur-gate-PAR-FICHIER-du-change-set â†’ **DÃ‰GRADÃ‰S, carry P1 S81** (shape wire Â§3.2). Chemin conditionnel Â§5.1.
- Onglets Â« AperÃ§u scellÃ© Â» + Â« Preuve Â» = **disabled Â« Ã  venir (S81) Â»**, 0 fetch/rendu scellÃ© (Ã  attester au review post-code) â€” les coder rouvre le P1 app-authoring in-vivo.
- Refactor `GateResult.issues -> struct{path,line?,message}` + `head` additif GatesView â†’ carry P1 S81 (coeur publish non-dÃ©lÃ©gable).

---

## 7. Risques rÃ©siduels / cibles adversariales (Ã  re-vÃ©rifier EN Phase H)

1. **Budget** : re-builder + `npm run size` dÃ¨s le 1er jet du diff-viewer ; VÃ‰RIFIER que l'extraction HOISTE rÃ©ellement hors VerifyScene (pas une duplication dans 2 chunks) ET qu'une nouvelle entrÃ©e `.size-limit.json` mesure bien le code extrait (anti-angle-mort P2 Phase E).
2. **Wire mobile** : re-grep gates.rs:157-185 + `lint_planning_data` (process.rs:443-486) AVANT de figer la dÃ©gradation V5/V6 â€” si une phase future peuplait `file`(=source du change-set)/`line`, V5/V6 redeviendraient constructibles.
3. **D2** : re-confirmer 0 import zod APRÃˆS cÃ¢blage des 2 nouveaux clients.
4. **Cardinal** : le mapping V4 ne doit FABRIQUER aucun des 3 marqueurs T2 (PROVISIONAL/Not-evidenced/RIG-ABSENT) depuis l'enum 5-valeurs.
5. **Bascule** : garder MANUELLE (D6) â€” seule la disponibilitÃ© du CTA devient Ã©tat-driven ; jamais auto-switch arrachÃ© au stream.
6. **XSS/CSP** : T1 Playwright doit injecter une ligne `<script>`/onerror et asserter `textContent` littÃ©ral (word-diff en noeuds texte). NE PAS ajouter `style-src 'unsafe-inline'` : les styles inline React/Motion via CSSOM ne sont pas gouvernÃ©s par `style-src` (Phase E shippe dÃ©jÃ  sous ce CSP). PrÃ©fÃ©rer classes Tailwind littÃ©rales pour l'Ã©mission v4, pas pour le CSP.
7. **ScellÃ©/Preuve** : attester au review l'Ã©tat `disabled` visible + 0 import/fetch scellÃ© ni Proof Card.

---

## 8. Questions ouvertes PO

Aucune dÃ©cision PO requise pour dÃ©bloquer Phase H (pas de DESIGN-CONFLICT). Arbitrage dÃ©jÃ  tranchÃ© au plan, confirmÃ© ici : V5/V6 + V4-per-fichier-du-change-set sont DÃ‰GRADÃ‰S et CARRY P1 S81 (shape wire actuelle : `GateIssueView.line==None`, `file`=basename .planning hors change-set). Si le PO veut V5/V6 en plein dÃ¨s S80, cela impose en amont le refactor du coeur publish `GateResult.issues -> struct{path,line?,message}` (touche pipeline.rs/atelier.rs/Display/tests â€” non-dÃ©lÃ©gable S79), hors pÃ©rimÃ¨tre front de Phase H ; le dÃ©faut recommandÃ© reste DÃ‰GRADER+carry S81.

---

## Verdict: PLAN-ADAPT