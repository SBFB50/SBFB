# Feature Landscape

**Domain:** P2P app factory, scaffolding, domain-specific app generation
**Researched:** 2026-05-18

## Table Stakes

Features users expect. Missing = product feels incomplete.

| Feature | Why Expected | Complexity | Notes |
|---------|--------------|------------|-------|
| CLI `sbfb create` | Tout ecosysteme a un scaffolder | Low | Pattern create-vite, degit |
| SBFB.json enrichi (permissions, category) | Le manifest doit declarer ce que l'app fait | Low | 3 champs actuel -> ~12 champs |
| Templates HTML pur + React | Les 2 stacks les plus courantes | Med | 2 templates minimum |
| Bridge SDK auto-copie | Ne pas forcer la copie manuelle | Low | Deja en place via sync-bridge-sdk.sh |
| Preview avant publish | Voir l'app avant de la publier | Low | blob-serve existant = preview |
| Publish gate (checklist) | Pas de publish sans index.html + manifest valide | Med | Extension de deploy.rs |
| Diff review avant modification | Standard AI code gen (bolt.new, v0) | Med | Broker + UI React |
| Audit log des actions Factory | Tracabilite = confiance | Low | JSONL simple |
| Template verification (hash) | Un template modifie = potentiel supply chain | Low | BLAKE3 du template |

## Differentiators

Features that set product apart. Not expected, but valued.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| factory.provenance.json | Tracer la chaine de creation (qui, quoi, quand, depuis quel template) | Med | Unique a SBFB |
| Domain packs | Template + fixtures + config domaine = app complete en 1 commande | High | Babel, Repair Notebook |
| Ed25519 provenance sur l'app generee | L'app generee est verifiable comme toute app SBFB | Low | Reutilise deploy-from-repo |
| Template lui-meme verifiable | Le template passe par le meme pipeline de verification | Med | Meta-verification |
| Broker permission model (Flatpak-like) | L'utilisateur autorise chaque action privilegiee | Med | Differencie du "vibe coding" |
| Safety scorecard | Badge de securite automatique base sur les declarations SBFB.json | Med | Inspire par Obsidian 2025+ |

## Anti-Features

Features to explicitly NOT build.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| AI code generation automatique dans l'iframe | L'iframe est sandboxee, pas de FS/shell/git | Broker dans le daemon, UI dans le shell |
| WebContainers dans le browser | 20+ MB WASM, overkill, SBFB a blob-serve | Utiliser blob-serve pour preview |
| Templates dynamiques avec logique serveur | Ajoute complexite, casse le modele offline | Templates statiques avec substitution simple |
| Publish automatique sans gate humain | Detruit le modele de confiance SBFB | Publish gate obligatoire avec checklist |
| Methodes bridge Factory-specifiques (factory_*) | Pollue le protocole neutre | Routes HTTP daemon /api/v1/factory/* |
| NLLB-200 dans l'iframe | Bloque par sandbox (IndexedDB, connect-src) | task_submit vers workers SBFB |
| Multi-step interactive wizard | UX complexe, hard to test | CLI flags + form simple dans le shell |
| Template marketplace reseau | Trop tot, zero utilisateur | Templates locaux/Git d'abord |

## Feature Dependencies

```
SBFB.json v2 -> Template Engine -> CLI sbfb create -> Template verification
                                       |
                                       v
                             Broker routes API -> Diff generation -> Review UI -> Publish gate
                                                                       |
                                                                       v
                                                       Domain pack format -> Babel reader
                                                                               |
                                                                               v
                                                                 Bridge integration -> Deploy verifie
```

## MVP Recommendation

Prioritize:
1. SBFB.json v2 (schema_version compat) — gate d'entree pour tout le reste
2. 3 templates (static-minimal, static-storage, react-vite) — couvrent 90% des cas
3. CLI `sbfb create` — experience developer immediate
4. Preview via blob-serve — zero nouveau code, reutilise l'existant
5. Publish gate — coherent avec le modele de confiance

Defer:
- Domain packs : concept nouveau, valider d'abord les templates simples (S73) avant d'ajouter des packs (S75)
- Safety scorecard : post-S75, quand il y a assez d'apps pour que le scorecard ait du sens
- Template marketplace reseau : post-v1.0, quand des tiers creent des templates

## Sources

- bolt.new (preview + AI generation): https://github.com/stackblitz/bolt.new
- Flatpak portals (broker pattern): https://docs.flatpak.org/en/latest/sandbox-permissions.html
- Obsidian safety scorecard: https://obsidian.md/blog/future-of-plugins/
- VS Code extension trust: https://code.visualstudio.com/docs/configure/extensions/extension-runtime-security
