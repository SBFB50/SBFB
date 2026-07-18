# Workflow Agents — vue d'ensemble (GUIDE)

> **Statut** : GUIDE **amorcé** (Sprint 83 Phase A). La clôture docs-contrat
> complète (couche REFERENCE des schémas remplis + entrée `llms.txt` +
> `THREAT_MODEL` vNN) est finalisée en fin de chantier (AJ1). Ce document fixe
> l'architecture d'autorité et nomme les contrats ; il grandit à chaque phase.

Le chantier **Workflow Agents** ajoute à SBFB la capacité de **publier,
distribuer, exécuter et vérifier** des workflows multi-agents portables et
signés — sans serveur central, dans le respect des invariants du réseau
(héberger ≠ publier, aucun LLM ne détient d'autorité de contrôle).

## 1. Architecture d'autorité — 4 couches (D1)

1. **Workflow Hub** — app SBFB **sandboxée** plein écran (iframe, CSP
   `connect-src 'none'`). Elle pré-valide, édite et affiche. Elle ne détient
   **jamais** de credential provider, de clé privée, de base autoritaire des
   runs, d'exécution privilégiée ni de scheduler distribué. Elle ne fabrique
   jamais un succès local.
2. **Shell hôte de confiance** — accorde les grants par app/release/pack, rend
   et signe le **HITL** dans le chrome hôte (lié au digest exact), détient les
   secrets et les quotas.
3. **Workflow Runtime Rust (hôte)** — **seule autorité** : compilateur,
   reducer, scheduler local, event store, inbox/outbox, checkpoints, reprise,
   annulation honnête, provenance, évaluation. Hébergé par le daemon hôte
   (routes loopback à l'inventaire D7 + auth complète). Le `FactoryAdapter`
   privilégié (repo/git/shell/PTY/Claude Code/Codex) vit dans un **companion
   local séparé** (extension `sbfb-factory`), hors TCB réseau.
4. **Protocole SBFB** — distribution content-addressed (iroh-blobs + feed) +
   feuilles Task/Result signées bornées. L'autorité d'un run reste le **nœud
   initiateur**.

**Invariant de contrôle** (Plan-Then-Execute / CaMeL) : le graphe déterministe
signé **est** le plan. Aucun LLM n'étend le graphe, n'accorde une capacité, ne
change une destination ni ne produit un verdict process autoritaire au runtime.
Un LLM peut influencer une **sortie** de modèle, jamais la
topologie/outils/destinations/grants/classification/HITL/budgets.

## 2. Les deux crates du cœur (D3)

| Crate | Rôle | Contraintes |
|---|---|---|
| `sbfb-workflow-core` | **pur** : schémas, canonicalisation + digests, validation statique, compilation du plan immuable, fixtures golden | sans Tokio / réseau / DB — déterministe, testable en isolation |
| `sbfb-workflow-engine` | hôte-neutre : reducer + machine d'état, scheduler local, retries/timers/joins/boucles bornées, inbox/outbox + idempotence, checkpoints + replay | traits `store/clock/activity/artifact/signal/secret/policy`, aucun couplage UI |

Le moteur durable est de la famille **mémoïsation-de-steps + journal append-only**
(pas de replay-total, pas de WASM). Le journal (`RunEvent.v1`, PK
`(run_id, sequence)`) et l'outbox d'effets partagent la **même transaction** ;
une clé d'idempotence `(run_id, node_instance_id, effect_id)` garantit
kill/restart/replay sans double effet. Un effet non idempotent sans reçu devient
`outcome_unknown` — jamais un succès inventé.

## 3. Contrats versionnés (identités de frontière)

Les sept identités de contrat sont réservées dans `crates/sbfb-workflow-core`
(module `contracts`). En Phase A ce sont des **frontières nommées** (types
`schemars`, JSON Schema draft 2020-12) dont le corps est rempli dans la phase
dédiée à chacune.

| Contrat | Rôle |
|---|---|
| `SBFBPromptAgentPack.v1` | racine du pack portable signé (graphe, agents, prompts, schémas pinnés, capacités **demandées**, evals, provenance) |
| `SBFBWorkflowGraph.v1` | graphe data-flow borné statiquement (ports typés, `repeat` borné, subworkflows, `human.review`) |
| `SBFBAgentDefinition.v1` | un îlot d'agent borné (prompt, policies, budgets) ; `handoff` ≠ `agent.as_tool` |
| `FactoryProcessProfile.v1` | contrat process Factory machine-lisible (rôles, modes, gates, hooks) lié aux digests sources ; mismatch → `PROFILE_STALE` |
| `RunEvent.v1` | une entrée du journal append-only |
| `CompiledPlan.v1` | plan immuable produit par le compilateur Rust autoritaire |
| `WorkflowPublicProjection.v1` | projection publique allowlistée consommée par la RRV |

**Signature** (D2) : JCS RFC 8785 (`serde_jcs`) + Ed25519 + constantes de
domaine dédiées, ajoutées au pattern `DOMAIN_*_V1` existant **quand chaque
contrat est rempli** (le census de familles reste gelé tant que les corps sont
vides). Emprunt in-toto minimal : `subject[] = [{name, digest:{blake3}}]`.

## 4. Preuves et fermeture

- **Matrice de gates** : `.planning/active/sprint83_gate_matrix.md` — 47 gates
  §23 + WA-T1..T6 + HUB-T1..T10, chacun avec sa phase propriétaire, son
  mécanisme d'enforcement et son statut EXISTANT/ÉTENDU/NEUF.
- **Acceptance** : `.planning/active/sprint83_t2_spec.md` — schéma versionné de
  `sprint83_t2_acceptance.json` (vocabulaire fermé PASS/BLOCK/RIG-ABSENT ;
  paliers ACTED/MIXED/NOT-RUN).
- **Grammaire de phase** : `scripts/agent/test_agentctl.py` prouve que les
  labels bi-lettres (A1, W1, AA, AI2, AJ1) parsent identiquement dans agentctl
  et les deux hooks Git.

## 5. Périmètre (hors MVP)

Scheduler DAG distribué, réplication live du run state privé, co-édition
temps réel, node code-exécuté / plugins tiers, marketplace centrale, auto-update
des packs, A2A adapter, cancel fort d'une Task réseau, import exécutable total
Flowise/n8n/LangGraph, mémoire personnelle P2P. Cf. `sprint83_kickoff.md §7`
(14 scope cuts).
