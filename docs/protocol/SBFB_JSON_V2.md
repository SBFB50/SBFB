# SBFB.json v2 — Manifest d'application

**Version** : 2.0 (spec, pas de code — implementation S67 Phase A).
**Auteur** : Sprint 65 Phase D.
**Prerequis** : TRUST_TAXONOMY.md (S65 Phase A), FACTORY_GATES.md
(S65 Phase D).

---

## Vue d'ensemble

Chaque app SBFB contient un fichier `SBFB.json` a la racine de son
archive zip. Ce manifest declare les metadonnees, les permissions
bridge, et les contraintes techniques de l'app.

La v2 enrichit la v1 avec des champs optionnels pour la Factory
(S67-S69), la recherche RRV (S70-S72), et l'affichage dans le
shell Browse. La v1 reste valide — tous les nouveaux champs sont
optionnels avec des defauts sensibles.

**Non-goal 2026-05-21 :** ce fichier reste un manifest d'application
SBFB. Il ne decrit pas encore un repo source externe, une librairie
GitHub generique, ni un corpus `@dev` source-only. Un futur mode
`source-only`/`source-index` devra avoir son propre contrat ou une
extension explicite, avec des labels separes de `verified SBFB app`.

---

## Schema

### Champs requis

| Champ | Type | Description |
|-------|------|-------------|
| `name` | `string` | Identifiant technique unique (kebab-case, `[a-z0-9-]+`, max 64 chars). Utilise comme slug dans les URLs et les references internes. |
| `display_name` | `string` | Nom affiche dans l'UI (max 128 chars, UTF-8). Peut contenir espaces, accents, emojis. |
| `description` | `string` | Description courte (max 500 chars). Affichee dans Browse et les resultats de recherche. |

### Champs optionnels

| Champ | Type | Defaut | Description |
|-------|------|--------|-------------|
| `schema_version` | `integer` | `1` | Version du schema manifest. `1` = v1 legacy, `2` = v2 enrichi. Le parser accepte les deux. |
| `category` | `string` | `null` | Categorie de l'app (`tool`, `game`, `social`, `productivity`, `education`, `other`). Utilisee pour le filtrage dans Browse. |
| `license` | `string` | `null` | Identifiant SPDX de la licence de l'app (ex : `MIT`, `GPL-3.0-only`, `AGPL-3.0-or-later`). Distinct de la licence SBFB (AGPL-3.0). |
| `lang` | `string` | `null` | Langue principale du contenu (`fr`, `en`, `ar`, etc., code ISO 639-1). Pour les apps multilingues, indiquer la langue par defaut. |
| `bridge` | `object` | `{}` | Configuration du bridge postMessage. |
| `bridge.methods` | `string[]` | `[]` | Methodes bridge requises. Sous-ensemble de `["task_submit", "storage_get", "storage_set"]`. |
| `bridge.events` | `string[]` | `[]` | Evenements bridge auxquels l'app s'abonne (reserve pour S67+). |
| `tech` | `object` | `{}` | Metadonnees techniques. |
| `tech.type` | `string` | `"static-html"` | Type technique : `static-html`, `react`, `pyodide`, `wasm`, `jupyterlite`. Utilise par Factory FG0 et par le shell pour adapter l'affichage. |
| `tech.build_command` | `string` | `null` | Commande de build (ex : `npm run build`). Utilisee par Factory FG8 pour la reproductibilite (future N4). Si `null`, l'app est servie telle quelle (pas de build step). |
| `requirements` | `object` | `{}` | Contraintes d'execution. |
| `requirements.min_bridge_version` | `string` | `null` | Version minimale du bridge SDK (semver). Si le daemon ne supporte pas cette version, l'app affiche un avertissement. |

---

## Exemples

### v1 minimal (retro-compatible)

```json
{
  "name": "hello-world",
  "display_name": "Hello World",
  "description": "Application de demonstration SBFB."
}
```

Le parser interprete ce manifest comme v1 (`schema_version`
implicitement `1`). Tous les champs optionnels prennent leur
defaut.

### v2 complet

```json
{
  "schema_version": 2,
  "name": "sbfb-ideas",
  "display_name": "Ideas Hub",
  "description": "Proposez et votez pour des idees. Stockage P2P via le bridge.",
  "category": "social",
  "license": "AGPL-3.0-or-later",
  "lang": "fr",
  "bridge": {
    "methods": ["storage_get", "storage_set"],
    "events": []
  },
  "tech": {
    "type": "static-html",
    "build_command": null
  },
  "requirements": {
    "min_bridge_version": null
  }
}
```

### v2 app React avec build

```json
{
  "schema_version": 2,
  "name": "sbfb-explorer",
  "display_name": "Protocol Explorer",
  "description": "Exploration interactive du protocole SBFB avec verification provenance.",
  "category": "tool",
  "license": "AGPL-3.0-or-later",
  "lang": "fr",
  "bridge": {
    "methods": ["storage_get"],
    "events": []
  },
  "tech": {
    "type": "react",
    "build_command": "npm run build"
  },
  "requirements": {}
}
```

---

## Strategie de versioning

### Parsing

Le parser de manifest suit cette logique :

1. Lire `schema_version`. Si absent, traiter comme `1`.
2. Si `schema_version == 1` : seuls `name`, `display_name`,
   `description` sont attendus. Les champs v2 sont ignores
   s'ils sont presents (forward-tolerant).
3. Si `schema_version == 2` : valider tous les champs declares
   contre le schema ci-dessus.
4. Si `schema_version > 2` : accepter le manifest (forward-
   tolerant), parser les champs connus, ignorer les inconnus.
   Avertissement dans les logs.

### `#[serde(default)]` et robustesse runtime

Tous les champs optionnels portent `#[serde(default)]` dans
l'implementation Rust. C'est une tolerance runtime, pas une
compatibilite historique : un client Python qui envoie un JSON
minimal a l'API daemon ne doit pas provoquer un 422 parse error
pour un champ optionnel absent.

### Forward-compatibility

Ajouter un champ optionnel au manifest n'est PAS un breaking
change. Le parser existant l'ignore (`#[serde(deny_unknown_fields)]`
n'est PAS utilise). Un bump de `schema_version` (2 → 3) n'est
necessaire QUE si un champ **requis** est ajoute ou si la
semantique d'un champ existant change.

### Pre-launch policy

Tant que le projet n'a pas de deploiement live (cf. CLAUDE.md
pre-launch protocol policy), `schema_version` reste librement
redefinissable. Apres le go-live, chaque breaking change bump
la version et le parser accepte un range.

---

## Relation avec les artefacts existants

| Artefact | Relation |
|----------|----------|
| `ProjectAnnouncement` | Contient `name`, `description`, `is_open_source` extraits du manifest. Le manifest v2 enrichit les metadonnees disponibles pour l'annonce. |
| `provenance.json` | Genere par Factory FG8. Le manifest declare `tech.build_command` utilise pour la reproductibilite. |
| Feed `ReleasePublished` | L'operation feed reference le manifest hash. |
| Bridge `sbfb-bridge.js` | Les methodes declarees dans `bridge.methods` sont les seules autorisees par le sandbox. |
| Shell Browse | Utilise `display_name`, `description`, `category`, `lang` pour l'affichage et le filtrage. |
