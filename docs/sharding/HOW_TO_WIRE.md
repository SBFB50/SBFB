# Comment câbler un projet pour le sharding

*Guide pratique (Diátaxis). Pour le pourquoi, voir
[`EXPLANATION.md`](./EXPLANATION.md) ; pour les types exacts, voir
[`REFERENCE.md`](./REFERENCE.md).*

> **Statut : PROVISIONAL.** Lire cette bannière d'honnêteté **avant** de câbler
> quoi que ce soit.

## Ce qui existe vraiment aujourd'hui (et ce qui n'existe pas)

- Les **primitives wire** (plan signé, `RunProof`, vérification N0→N3), le
  **data-plane** `sbfb/shard/1` et le **panneau front** `/compute` sont livrés.
- **Il n'y a pas de store de session live.** L'**orchestrateur de session
  in-vivo** — celui qui pilote la génération, peuple un registre de sessions et
  émet un `RunProof` signé — est un **carry Sprint 78**. Toute la surface décrite
  ici est donc soit du **contrat wire**, soit un **état-vide honnête**.
- **`sbfb-bridge.js` n'expose AUCUNE méthode shard.** Le bridge postMessage des
  apps sandboxées a une whitelist de méthodes (soumission/résultat de tâche,
  stockage, rédaction PII) — **aucune** ne touche le sharding. Une app dans son
  iframe **ne peut pas** démarrer ou rejoindre une session. **Le point d'entrée
  est le panneau shell `/compute`**, pas un appel bridge. Le nom et la forme d'une éventuelle méthode bridge shard sont **figés
  Sprint 78** — ne rien pré-câbler.

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
Aujourd'hui c'est un **texte explicatif** : il décrit la capacité, sans
orchestrateur derrière (carry S78). Quand l'orchestrateur atterrira, c'est
l'initiateur qui construira et **signera** le `ShardedSessionManifest`
(`nexus-shard-plan-v1`) décrivant le plan de pipeline. Côté protocole, rien à
câbler dans une app : l'initiation est une action **privilégiée du nœud**, pas une
méthode bridge.

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

**Sans store de session live (carry S78), la route répond de façon déterministe
`200 {found:false, session:null}` pour tout `id`** — un 200 avec des valeurs par
défaut honnêtes (jamais un 404), pour que le parsing front réussisse sur un état
vide. Quand une session sera peuplée, `session` exposera **uniquement** un
agrégat `member_count` — **jamais** un `worker_pubkey` ni l'`initiator` (privacy
SI-3/SI-4). Côté front, le helper est
[`web/src/api/daemon.ts`](../../web/src/api/daemon.ts) (`getShardSession`), avec
une enveloppe Zod `.strict()` qui parse l'état vide comme un succès, pas une
erreur de transport.

## Récapitulatif du câblage

| Rôle | Surface | État réel |
|---|---|---|
| START | panneau `/compute` (action nœud) | texte explicatif ; orchestrateur = carry **S78** |
| JOIN | `id` de session hors-bande + admission `is_member` au data-plane | admission câblée ; pas de découverte |
| OBSERVE | `GET /api/daemon/shard-session/{id}` | stub `{found:false,session:null}` ; agrégat `member_count` quand peuplé |

Les types, caps et seuils exacts manipulés par ces surfaces sont dans
[`REFERENCE.md`](./REFERENCE.md).
