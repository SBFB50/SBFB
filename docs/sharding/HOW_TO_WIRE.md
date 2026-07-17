# Comment câbler un projet pour le sharding

*Guide pratique (Diátaxis). Pour le pourquoi, voir
[`EXPLANATION.md`](./EXPLANATION.md) ; pour les types exacts, voir
[`REFERENCE.md`](./REFERENCE.md).*

> **Statut : LIVE-PROVEN (S81 I/J).** Lire cette bannière d'honnêteté **avant**
> de câbler quoi que ce soit.

## Ce qui existe vraiment aujourd'hui (et ce qui reste ouvert)

- Les **primitives wire** (plan signé, `RunProof`, vérification N0→N3), le
  **data-plane** `sbfb/shard/1` et le **panneau front** `/compute` sont livrés.
- L'**orchestrateur de session in-vivo** — celui qui pilote la génération,
  peuple un registre de sessions live et émet un `RunProof` DRIVER signé — est
  **livré (S81 Phase I)** en surface **opérateur** (routes loopback
  `/api/daemon/shard-session/*`), et le **benchmark live 2 machines** a **PASS**
  (S81 Phase J). Depuis S81 Phase K, chaque stage-link exige l'**attestation du
  stage chargé** (fail-closed). **Reste ouvert (différé par S82 — sprint dette
  docs-contrat — vers le slot rig-chaud, roadmap v5) :** les preuves
  signées **per-worker** des shards distants + l'**arbitrage de litige in-vivo**
  (le RunProof actuel est celui du DRIVER, un self-claim — pas une vérification
  indépendante).
- **`sbfb-bridge.js` n'expose AUCUNE méthode shard.** Le bridge postMessage des
  apps sandboxées a une whitelist de méthodes (soumission/résultat de tâche,
  stockage, rédaction PII) — **aucune** ne touche le sharding. Une app dans son
  iframe **ne peut pas** démarrer ou rejoindre une session. **Le point d'entrée
  est le panneau shell `/compute`** + la surface opérateur, pas un appel bridge.
  Le nom et la forme d'une éventuelle méthode bridge shard restent **non figés**
  (S81 n'en a ajouté aucune) — ne rien pré-câbler.

> **Caveat cardinal — admission ≠ confidentialité.** Avant de soumettre quoi que
> ce soit : **« admission ≠ confidentialité »**. Les activations circulent en
> clair entre les membres ; **aucun secret applicatif ne doit transiter par les
> prompts d'une session shardée**. Voir
> [`docs/security/THREAT_MODEL.md`](../security/THREAT_MODEL.md) §16.

## Contrainte de cohorte

Le backend de sharding exige des modèles à **architecture llama uniquement**, et
**le même GGUF sur tout le groupe** (cohorte homogène). Un worker dont le modèle
n'est pas une architecture llama est refusé au backend ; un GGUF hétérogène
casserait la continuité du pipeline et la comparaison de fingerprints N0.

## Les trois rôles

### 1. START — « Lancer un gros modèle en réseau »

Dans le shell, l'onglet **`/compute`** présente l'intention *« Lancer un gros
modèle en réseau »* (panneau
[`web/src/components/ShardSessionPanel.tsx`](../../web/src/components/ShardSessionPanel.tsx)).
Le panneau reste un **texte explicatif** ; l'orchestrateur derrière est **livré
depuis S81 Phase I** en surface **opérateur** (routes loopback
`/api/daemon/shard-session/*` : mint du groupe signé, mount — placement +
signature du `ShardedSessionManifest` `nexus-shard-plan-v1` + readiness
barrier —, drive, résultat ; cf. `SHARD_PROTOCOL_SPEC.md` §6). Les corps de
requête exacts de ces routes — `ShardGroupMintRequest`, `MountSessionRequest`,
`ShardGenerateRequest` (S82 G) — sont spécifiés dans les tables de
`SHARD_PROTOCOL_SPEC.md` §6.1 et résumés dans [`REFERENCE.md`](./REFERENCE.md).
Côté protocole,
rien à câbler dans une app : l'initiation est une action **privilégiée du
nœud**, pas une méthode bridge.

### 2. JOIN — « Rejoindre un groupe de calcul »

L'intention *« Rejoindre un groupe de calcul »* permet de **consulter** une
session par son identifiant, transmis **hors-bande** (l'`id` n'est pas
découvrable ; il est partagé par l'initiateur). C'est aujourd'hui un **lookup
read-only** (voir OBSERVE). L'admission réelle d'un worker au data-plane est
gouvernée côté daemon : à l'ouverture d'un flux `sbfb/shard/1`, l'accepteur
vérifie `is_member` (la clé Ed25519 du pair appelant est sur l'allowlist
`ComputeGroup`) **avant** de lire le moindre octet ; un non-membre est fermé au
handshake.

### 3. OBSERVE — statut read-only d'une session

```
GET /api/daemon/shard-session/{id}
```

Route loopback-authentifiée (bearer + Host + Origin), dans `authed_routes` du
daemon ([`crates/nexus-shell-daemon/src/http.rs`](../../crates/nexus-shell-daemon/src/http.rs)).
Elle renvoie une enveloppe `ShardSessionStatusResponse` :

```json
{ "found": false, "session": null }
```

**Pour un `id` inconnu, la route répond de façon déterministe
`200 {found:false, session:null}`** — un 200 avec des valeurs par
défaut honnêtes (jamais un 404), pour que le parsing front réussisse sur un état
vide. Depuis S81 Phase I le statut est servi depuis le **registre live**
(`ShardSessionRegistry`, insert gaté signature + `is_member`) : pour une session
montée, `session` expose **uniquement** un
agrégat `member_count` — **jamais** un `worker_pubkey` ni l'`initiator` (privacy
SI-3/SI-4). Côté front, le helper est
[`web/src/api/daemon.ts`](../../web/src/api/daemon.ts) (`getShardSession`), avec
une enveloppe Zod `.strict()` qui parse l'état vide comme un succès, pas une
erreur de transport.

## Récapitulatif du câblage

| Rôle | Surface | État réel |
|---|---|---|
| START | panneau `/compute` (texte) + routes opérateur `/api/daemon/shard-session/*` | orchestrateur **livré S81 I** (mount/drive/result) ; UX produit du panneau = à venir |
| JOIN | `id` de session hors-bande + admission `is_member` au data-plane | admission câblée + attestation loaded-stage fail-closed (S81 K) ; pas de découverte |
| OBSERVE | `GET /api/daemon/shard-session/{id}` | registre live (S81 I) : `{found:false,session:null}` pour un id inconnu ; agrégat `member_count` pour une session montée |

Les types, caps et seuils exacts manipulés par ces surfaces sont dans
[`REFERENCE.md`](./REFERENCE.md).
