# Sharding — exécuter un gros modèle éclaté sur plusieurs machines

Le sharding d'inférence permet de faire tourner un modèle de langage **trop
gros pour la VRAM d'une seule machine** en le découpant en blocs de couches
contigus répartis sur les membres d'un **groupe de calcul privé**, reliés en
**pipeline** sur le réseau P2P SBFB (ALPN `sbfb/shard/1`).

Ce dossier est le **hub de documentation** du sous-système. Il suit le modèle
[Diátaxis](https://diataxis.fr/) : quatre types de documents pour quatre
besoins distincts.

---

## Statut : PROVISIONAL

Le **cœur du pipeline est livré et testé hermétiquement** (Sprint 77, phases
A→K : primitives wire signées, placement, routage, churn, fork llama.cpp,
data-plane `sbfb/shard/1`, vérification graduée N0→N3, panneau front). Mais la
feature reste **PROVISIONAL** : le **benchmark live cross-machine** (T2) est
`RIG-ABSENT` — il n'existe pas encore d'**orchestrateur de session in-vivo**
qui pilote la génération et émet un `RunProof` signé, et le rig 2 machines
(RTX 5080 + Mac M2) n'a pas été branché. L'orchestrateur de session + le
benchmark live sont un **carry P1 vers Sprint 78**.

Concrètement : tout ce qui est décrit ici concerne le **contrat wire** et les
**primitives** réellement présentes dans le code. Aucune session shardée ne
tourne en production aujourd'hui. Les preuves de vie hermétiques sont
[`scripts/acceptance/b3_shard_pipeline.sh`](../../scripts/acceptance/b3_shard_pipeline.sh)
et [`web/e2e/compute-shard.spec.ts`](../../web/e2e/compute-shard.spec.ts).

### Caveat cardinal — admission ≠ confidentialité

> **L'admission dans un groupe de calcul n'est PAS de la confidentialité.**
> L'allowlist `ComputeGroup` (signée Ed25519) contrôle **qui peut participer**
> à un pipeline, **pas** le secret des activations : celles-ci circulent **en
> clair** entre les machines (aucun TEE GPU grand public en 2026), et
> l'allowlist ne garantit pas une majorité honnête. La posture est
> *honest-but-curious*. En conséquence, **« admission ≠ confidentialité »** :
> **aucun secret applicatif ne doit transiter par les prompts d'une session
> shardée** — un membre admis mais curieux voit les activations de son
> segment. Le sharding sert à exécuter un **gros modèle public éclaté**, pas à
> traiter des entrées confidentielles.

Détail des surfaces (SI-1..SI-11), de l'échelle de vérification N0→N3 et de
l'incitation : voir le modèle de menace,
[`docs/security/THREAT_MODEL.md`](../security/THREAT_MODEL.md) §16. Cette doc
**renvoie** au modèle de menace, elle ne le duplique pas.

---

## Les quatre documents

| Type Diátaxis | Document | Pour | Audience |
|---|---|---|---|
| **Explication** | [`EXPLANATION.md`](./EXPLANATION.md) | comprendre *comment ça marche* (pipeline-parallel, signatures, vérification graduée) | humain |
| **Guide pratique** | [`HOW_TO_WIRE.md`](./HOW_TO_WIRE.md) | *comment câbler un projet* du protocole (rôles START / JOIN / OBSERVE) | humain |
| **Référence** | [`REFERENCE.md`](./REFERENCE.md) | les types wire, caps et seuils exacts (jumeau humain des schémas générés) | humain + agent |
| **Tutoriel** | _(différé Sprint 78)_ | un walkthrough end-to-end runnable | — |

> **Pourquoi pas de tutoriel ?** Un tutoriel promet un parcours qui *marche du
> premier coup*. Tant que l'orchestrateur de session in-vivo est un carry S78
> (statut PROVISIONAL ci-dessus), un walkthrough end-to-end sur-promettrait. À
> la place, les deux harness ci-dessus sont la preuve-de-vie exécutable.

---

## Contrat machine-lisible

La **source de vérité machine** du sous-système est
[`docs/protocol/SHARD_PROTOCOL_SPEC.md`](../protocol/SHARD_PROTOCOL_SPEC.md)
(+ les schémas JSON générés `*.schema.json` dans
[`crates/nexus-core-rs/src/schemas/`](../../crates/nexus-core-rs/src/schemas/)).
La [`REFERENCE.md`](./REFERENCE.md) de ce dossier en est le jumeau humain ; en
cas de divergence, **le spec machine + les structs Rust font foi**.
