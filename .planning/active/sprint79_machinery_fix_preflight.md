# Sprint 79 — Fix machinery (phase detection) — Preflight

**Type** : `fix(sprint79)` autonome (hors séquence de phases A-I, qui restent les
livrables Factory). Décision PO : `fix(sprint79)` + rigueur complète (preflight +
review Workflow + Codex), `done` ⟺ `verification.md`.

**Méthode** : Workflow ultracode de balayage `w4u0woeqc` (6 agents Opus 4.8 1M,
429 K tokens, 110 tool-calls) — 5 balayages parallèles (caps Rust / regex-contrat /
casse cross-platform / sémantique-bornes / cohérence process+frontend) + synthèse
adversariale (10 faux positifs écartés). Ce balayage fait office de preflight approfondi.

## Verdict : EXECUTE

Le contrat de process (`docs/claude/README.md` §4 — phases NON plafonnées, token
`[A-Z]+[0-9]?`) était contredit par la machinerie de détection de statut/historique de
`sbfb-factory`. **12 incohérences confirmées, toutes dans `sbfb-factory`** (aucune autre
crate ; vérifié par grep). Le frontend `factory-ui` n'introduit aucun cap (hérite du
backend) → fix 100% backend.

### Familles confirmées
| Famille | Sites | Sév |
|---|---|---|
| Cap alphabet `['A'..'G']` | `process.rs` detect_current_phase + status_sprint_data (site neuf) ; `sprint_history.rs` build_sprint_summary + build_phase_histories + build_preflight_bilan | P1 |
| Regex bornée | `process.rs` PHASE_TITLE_RE `[A-Z][0-9]?` (mono-lettre, gate audit-commit) ; `sprint_history.rs` PHASE_RE `[A-G][0-9]?` (double cap) | P1 |
| **Bug casse cross-platform** | `process.rs` construit `phase_A` (MAJ) vs fichiers actifs `phase_a` (min) → marche Windows (FS insensible), CASSE Linux/CI/VPS (`.exists()` toujours faux → statut figé, faux « missing review/codex ») | P1 |
| Sémantique `done` | `process.rs:174` `Some('G')=>"done"` (S77=14 phases jamais "done") ; incohérent avec build_sprint_summary qui utilise déjà `verification.md` | P1 |
| Arithmétique lettres | `process.rs:176` `(p as u8+1) as char` — `'Z'+1='['`, jamais AA (latent) | P2 |
| Fixtures test majuscules | `process_cli.rs`/`operator_server.rs` — self-consistantes MAJ, masquaient le bug | P2 |

## Décision design (synthèse) : DÉCOUVERTE-PAR-FICHIERS, pas génération base-26
- **Nuance décisive** : l'archive est MIXTE (sprint65 + v1.2/v2.0 = MAJUSCULES 319+34
  fichiers, sprint66-77 + actif = minuscules). Un `to_ascii_lowercase()` naïf casserait
  la lecture de l'ancienne archive → le lookup DOIT être **case-insensible** retournant
  le **vrai chemin disque**.
- Helper partagé `crate::phase` (`discover_phase_artifacts`/`discover_phase_labels`/
  `find_phase_artifact`/`next_phase_label`/`phase_order_key`/`display_label`), réutilisé
  par les 8 sites — matérialise [[feedback_named_constants]] (1 source de vérité vs
  8 arrays/regex dupliqués). Génération base-26 conservée UNIQUEMENT pour `next_phase_label`.
- Ordre canonique = `(len, lexico)` (jamais sort string naïf : 'aa' < 'b' serait faux).
- Affichage MAJUSCULE préservé (contrat public : `letter`/`phase`/`current_phase`), lookups
  case-insensibles, labels internes minuscules.

## Décisions ouvertes tranchées
- `done` ⟺ `sprint{N}_verification.md` existe (cohérence interne avec build_sprint_summary). **PO confirmé.**
- Scope : une seule correction cohérente (helper partagé). **PO confirmé fix(sprint79).**
- Casse : scan case-insensible non-destructif, NE PAS renommer l'archive. **Décidé.**
- `Phase 0` (audit gate) : non suivi comme phase (convention chore non parsée). **Documenté.**
- Sous-phases `f1`/`f2` : labels distincts (regex `[A-Z]+[0-9]?` les capture). **Conservé.**

## Faux positifs écartés (discipline adversariale)
http.rs filler `vec![b'A';...]`, frontend SprintTimeline (hérite, 0 cap propre),
check-frontier PROMISE_RE (matcher de promesses, pas de phase), lightcheck/auditor greps
sprint-number, timestamps 'Z', `ctx.get("phase")` (valeur libre), charCodeAt checksums,
`list_active_artifacts` (= le PATTERN CORRECT à généraliser), sort sprints u32.

## Tests ajoutés (hermétiques)
- `phase::tests` : next base-26 (z→aa, az→ba, f1→g), ordre (aa après z), is_phase_label,
  découverte unbounded + case-insensible.
- `process::tests::detect_current_phase_is_unbounded_and_case_insensitive` (lowercase, >G,
  done⟺verification).
- `sprint_history::tests::sprint_summary_is_unbounded_and_case_insensitive` (lowercase +
  archive MAJ + >G).

## Gate
G8 = ce balayage (EXECUTE). Review Workflow + Codex à suivre. `check-frontier-contracts.sh`
+ suites Rust avant commit.
