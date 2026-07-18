# Sprint 83 — Spec T1/T2 (schéma d'acceptance versionné)

Cadrage AM-1 (kickoff §4.5). Fige **avant tout code de surface** : (1) le
**vocabulaire fermé** des verdicts ; (2) le **schéma versionné** de
`sprint83_t2_acceptance.json` ; (3) la liste des **specs T1 Playwright nommées**
et de leur job CI. C'est le contrat de fermabilité du sprint (DoD §4 (c)/(d)).
Vocabulaire ratifié S82 J.

---

## 1. Vocabulaire fermé (source unique — miroir des enums, §6.9)

### 1.1 Verdict T2 top-level (statut du run d'acceptance agrégé)
| Valeur | Sens |
|---|---|
| `PASS` | tous les paliers joués sont `ACTED` ; aucun P0/P1 en carry de confort |
| `BLOCK{diagnosis}` | un palier a échoué ; `diagnosis` = cause machine-lisible (jamais de la prose « différé ») |
| `RIG-ABSENT` | matériel/credential requis absent (2e worker, provider cloud, Ollama, AT physique) — l'absence est un fait, pas un échec |

### 1.2 Statut de palier `WA-T1..T6`
| Valeur | Sens |
|---|---|
| `ACTED{evidence}` | palier joué ; `evidence` = artefact(s) machine-lisible(s) (chemin JSON, digest, run id) |
| `MIXED` | palier partiellement joué (sous-étapes hétérogènes) — chaque sous-étape porte son propre statut |
| `NOT-RUN` | palier non joué (dépendance non disponible ou fenêtre ultérieure) |

### 1.3 Verdict T1 (spec Playwright hermétique par surface)
`GREEN` (`npm run test:e2e` vert) / `RED` / `N-A-no-frontend-change` (phase sans
surface front — ex. Phase A). Interdit : toute prose `DIFFERE-materiel` /
`DIFFERE-trace-user` (lightcheck Check 10 WARN + audit Track J = P1).

> **Sources uniques.** Ces trois familles de constantes sont l'autorité du
> vocabulaire ; toute suite/artefact les réutilise verbatim. Ne jamais
> introduire un 4e synonyme (« DEFERRED », « SKIP », « TODO ») — la conflation
> qui a fait taire S80 est interdite.

---

## 2. Schéma versionné de `sprint83_t2_acceptance.json`

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "sprint83_t2_acceptance.v1",
  "type": "object",
  "additionalProperties": false,
  "required": ["schema_version", "sprint", "status", "written_at_phase", "paliers"],
  "properties": {
    "schema_version": { "const": 1 },
    "sprint": { "const": 83 },
    "status": {
      "description": "verdict T2 top-level — vocabulaire fermé §1.1 ; BLOCK diagnosis = slug machine (pas de prose)",
      "type": "string",
      "pattern": "^(PASS|BLOCK\\{[a-z0-9][a-z0-9_-]*\\}|RIG-ABSENT)$"
    },
    "written_at_phase": {
      "description": "phase d'écriture (AI2 = agrégat initial ; AJ1 = finalisation T6)",
      "enum": ["AI2", "AJ", "AJ1"]
    },
    "paliers": {
      "type": "object",
      "additionalProperties": false,
      "required": ["WA-T1", "WA-T2", "WA-T3", "WA-T4", "WA-T5", "WA-T6"],
      "properties": {
        "WA-T1": { "$ref": "#/$defs/palier" },
        "WA-T2": { "$ref": "#/$defs/palier" },
        "WA-T3": { "$ref": "#/$defs/palier" },
        "WA-T4": { "$ref": "#/$defs/palier" },
        "WA-T5": { "$ref": "#/$defs/palier" },
        "WA-T6": { "$ref": "#/$defs/palier" }
      }
    }
  },
  "allOf": [
    {
      "$comment": "PASS exige que les 6 paliers soient ACTED (contrat non vacuous).",
      "if": { "properties": { "status": { "const": "PASS" } }, "required": ["status"] },
      "then": {
        "properties": {
          "paliers": {
            "properties": {
              "WA-T1": { "properties": { "status": { "pattern": "^ACTED\\{" } } },
              "WA-T2": { "properties": { "status": { "pattern": "^ACTED\\{" } } },
              "WA-T3": { "properties": { "status": { "pattern": "^ACTED\\{" } } },
              "WA-T4": { "properties": { "status": { "pattern": "^ACTED\\{" } } },
              "WA-T5": { "properties": { "status": { "pattern": "^ACTED\\{" } } },
              "WA-T6": { "properties": { "status": { "pattern": "^ACTED\\{" } } }
            }
          }
        }
      }
    }
  ],
  "$defs": {
    "palier": {
      "type": "object",
      "additionalProperties": false,
      "required": ["status"],
      "properties": {
        "status": {
          "type": "string",
          "pattern": "^(ACTED\\{.+\\}|MIXED|NOT-RUN)$"
        },
        "evidence": {
          "description": "artefacts machine-lisibles (chemins JSON / digests / run ids)",
          "type": "array",
          "items": { "type": "string" }
        },
        "rig_absent_reason": {
          "description": "présent si le palier 2-machines est RIG-ABSENT (WA-T2 / WA-T3 Babel)",
          "type": "string"
        },
        "substeps": {
          "description": "sous-étapes typées quand status = MIXED",
          "type": "array",
          "items": {
            "type": "object",
            "additionalProperties": false,
            "required": ["id", "status"],
            "properties": {
              "id": { "type": "string" },
              "status": { "type": "string", "pattern": "^(ACTED\\{.+\\}|NOT-RUN|RIG-ABSENT)$" }
            }
          }
        }
      },
      "allOf": [
        {
          "$comment": "ACTED exige au moins un artefact d'evidence (pas de succes sans preuve).",
          "if": { "properties": { "status": { "pattern": "^ACTED\\{" } }, "required": ["status"] },
          "then": { "required": ["evidence"], "properties": { "evidence": { "minItems": 1 } } }
        },
        {
          "$comment": "MIXED exige des sous-etapes typees (pas de MIXED opaque).",
          "if": { "properties": { "status": { "const": "MIXED" } }, "required": ["status"] },
          "then": { "required": ["substeps"], "properties": { "substeps": { "minItems": 1 } } }
        }
      ]
    }
  }
}
```

> **Non-vacuité (durcissement, réponse Codex round 1)** : le schéma **rejette**
> mécaniquement — vérifié Draft 2020-12 (`jsonschema` 4.26.0) — (a) `PASS` avec
> un palier non-`ACTED` ; (b) `ACTED{…}` sans `evidence` ; (c) `MIXED` sans
> `substeps` ; (d) `BLOCK{prose libre}` (seul un slug `[a-z0-9_-]` est accepté) ;
> (e) `written_at_phase` hors {AI2, AJ, AJ1}. Le vocabulaire fermé §1 n'est donc
> pas seulement lexical, il est **structurellement contraint**.

### 2.1 Règle de finalisation bi-temporelle (AM-1)
- **AI2** écrit l'agrégat initial : `WA-T1..T5` joués ;
  `WA-T6 = MIXED{substeps: [pre-cutover ACTED, cutover NOT-RUN]}`.
- **AJ / AJ1** mettent à jour le palier `WA-T6` (cutover joué) et **finalisent
  l'agrégat AVANT `sprint83_verification.md`** ; le commit **AJ1** porte la
  version finale (`written_at_phase: "AJ1"`).
- **Chemins `RIG-ABSENT`** attendus pour les paliers 2-machines : `WA-T2`
  (2e nœud / provider cloud), `WA-T3` (Babel 2 workers) — `rig_absent_reason`
  renseigné, jamais converti en `PASS`.

---

## 3. Specs T1 Playwright nommées + jobs CI (AM-1)

| Spec (hermétique) | Surface | Phase | Job CI (nommé) | Statut Phase A |
|---|---|---|---|---|
| `web/e2e/bridge-grant-intersection.spec.ts` | autorité bridge (intersection 7 couches) | **C** | `e2e-workflow` (rust-ci/ci.yml) | prévu C |
| `web/e2e/workflow-hub-author.spec.ts` | Hub — auteur (canvas/outline/guide) | **R/S/T** | `e2e-hub` (BLOQUANT-CI chaque push) | prévu R |
| `web/e2e/workflow-hub-run.spec.ts` | Hub — Centre des Runs | **V** | `e2e-hub` | prévu V |
| `web/e2e/workflow-hub-lifecycle.spec.ts` | Hub — publish/install/activate/fork/rollback | **X** | `e2e-hub` | prévu X |

- **Job CI** : `e2e-hub` est nommé et câblé **dès la Phase R** (ci.yml) et
  BLOQUANT à chaque push (AM-1). Les specs sont hermétiques (isolation projet
  Playwright `dependencies`, fix S81-A3-1) — pas de dépendance réseau.
- **Phase A (présent)** : **aucune surface front** → `T1 = N-A-no-frontend-change`.
  L'agrégat `sprint83_t2_acceptance.json` n'est **pas** écrit en A (il est écrit
  en AI2, finalisé AJ1 — plan §Phase A A.4).

---

## 4. Autorité de fermeture (rappel)
Le vocabulaire fermé ci-dessus est **le** contrat de testabilité §4. L'autorité
BLOQUANTE est la DoD §4 (c)/(d) + l'audit gate S84 **Track J** (T1/T2 manquant =
P1). Le hook lightcheck Check 10 est un **nudge WARN** non-bloquant (il ne porte
pas l'autorité, AM-1). `sprint84_audit_plan.md` (écrit en AJ1) inclut
explicitement Track J (testabilité) et Track K (docs-contrat).
