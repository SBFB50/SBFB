# Comment marche le sharding d'inférence SBFB

*Document d'explication (Diátaxis). Pour le mode d'emploi par rôle, voir
[`HOW_TO_WIRE.md`](./HOW_TO_WIRE.md) ; pour les types exacts, voir
[`REFERENCE.md`](./REFERENCE.md).*

> **Statut : LIVE-PROVEN (S81 I/J)** — orchestrateur de session in-vivo +
> benchmark live 2 machines livrés (cf.
> [`README.md`](./README.md)). **Caveat cardinal : admission ≠ confidentialité**
> — les activations circulent en clair dans le groupe privé ; ne jamais faire
> transiter un secret par les prompts d'une session shardée.

---

## Le problème

Un modèle de 20–70 Go ne tient pas dans les 16 Go de VRAM d'une RTX 5080. Plutôt
que de l'abandonner, le sharding le **découpe en blocs de couches** répartis sur
plusieurs machines qui collaborent pour produire une réponse.

## Pipeline-parallel, pas tensor-parallel

SBFB utilise du **pipeline-parallel** : chaque worker possède un **bloc de
couches contigu** `[layer_start, layer_end)` (intervalle **demi-ouvert** : la
borne haute est exclue), exécute ce bloc, puis **transmet les activations de
frontière** au worker suivant. Le dernier worker émet la sortie.

Ce choix est délibéré. Le tensor-parallel découpe chaque couche et exige un
*all-reduce* à chaque étape : il est **bandwidth-bound** et s'effondre sur un
lien WAN. Le pipeline-parallel n'échange que les activations de frontière
d'un stage au suivant — relayées par le driver (topologie HUB : les workers ne
se parlent jamais entre eux, cf. [`REFERENCE.md`](./REFERENCE.md)) : il est
**latency-bound** et **survit au WAN** (l'objectif réaliste est
de l'ordre de 1–3 tokens/seconde sur un lien grand public, pas la vitesse d'un
GPU local).

L'ordre du `Vec` d'assignations **est** l'ordre du pipeline. La primitive wire
`ShardPlan::is_pipeline_contiguous` vérifie seulement la **contiguïté
structurelle** : chaque bloc est non vide et commence exactement là où le
précédent finit, sans trou ni recouvrement. Elle **ne vérifie pas** que le plan
couvre tout le modèle — un plan peut légitimement décrire un sous-intervalle. La
**couverture exacte de `[0, L)`** (premier bloc à la couche 0, dernier bloc à
`total_layers`) est validée séparément par le scheduler au moment du placement
(`covers_full_model`,
[`crates/nexus-coordinator-rs/src/placement.rs`](../../crates/nexus-coordinator-rs/src/placement.rs)),
pas par la primitive wire.

## Qui signe quoi

Deux signatures Ed25519, deux responsabilités distinctes :

- **L'initiateur signe le plan.** Le `ShardedSessionManifest`
  (tag `nexus-shard-plan-v1`) dit *qui calcule quel bloc, sur quel modèle*. Sa
  signature prouve seulement que **l'initiateur a autorisé ce plan**.
- **Le driver de session signe la preuve d'exécution.** Le `RunProof`
  (tag `nexus-run-proof-v1`) dit *ce que le run a exécuté* (métriques,
  `participants` = qui a réellement calculé, et le fingerprint d'activation N0
  du dernier step). Depuis S81 I/J c'est le **driver** (la tête qui a piloté la
  génération) qui le signe — un self-claim non-répudiable pour le driver, pas
  une vérification indépendante. Les preuves signées **par worker** distant sont
  re-routées **S82**.

**La frontière est une auto-attestation.** Une signature valide ne prouve **pas**
que le calcul est correct — seulement *qui* a parlé. Tant qu'un vérificateur
indépendant n'a pas **recalculé** un fingerprint, un `RunProof` est un
**self-claim**, pas une preuve. C'est exactement pour ça qu'existe la
vérification graduée ci-dessous.

## Pas de flottants dans un payload signé

Les `RunMetrics` sont **entièrement en entiers**. Un `f64` ne fait pas de
round-trip bit-à-bit identique entre plateformes ; un signataire (Rust) et un
vérificateur (par ex. un client Python) dériveraient des octets canoniques
divergents et la signature ne vérifierait pas. Les débits utilisent donc des
unités entières : `decode_milli_tokens_per_sec` = tokens/seconde × 1000, des
octets, des millisecondes.

## La vérification graduée N0 → N3

L'intégrité ne tient pas à la confiance dans les membres mais à une **échelle de
vérification** *hors-bande* (jamais transportée sur le data-plane ; dérivée des
`RunProof` signés après coup). Chaque étage est aujourd'hui une **primitive
câblée et testée hermétiquement** ; l'orchestrateur qui exécute la génération
réelle et signe le `RunProof` DRIVER est **livré (S81 I/J)** — les preuves
signées PER-WORKER des shards distants restent re-routées **S82**.

- **N0 — empreinte TOPLOC.** Un *commitment* BLAKE3 32 octets du top-k du dernier
  hidden state. Il **détecte un swap** de modèle ou de quantification (un worker
  qui exécute un GGUF différent produit un commitment différent), avec une
  probabilité proche de 100 %. Il ne **chiffre rien** et ne prouve pas la
  correction du calcul — c'est un détecteur de substitution.
- **N1 — spot-check par tirage vérifiable.** Un vérificateur est tiré pour
  re-exécuter ~1 % du prefill et recomparer le fingerprint de façon **tolérante**
  (jamais l'égalité, car les GPU hétérogènes flottent différemment). Le tirage
  est une **signature Ed25519 déterministe** hashée (tag `nexus-vrf-draw-v1`),
  **pas un ECVRF** : ni l'unicité ni l'imprévisibilité ne sont prouvées. C'est
  une **mitigation sous l'hypothèse one-honest-verifier pour un échantillon
  1–5 %**, pas une garantie.
- **N2 — redondance tolérante (clique).** Une tâche haute-criticité tourne sur
  plusieurs workers et n'est acceptée que si une **clique** d'entre eux s'accorde
  *deux-à-deux* sur le fingerprint (l'accord tolérant n'est pas transitif : on ne
  compte pas autour d'un pivot, on cherche le plus gros ensemble mutuellement
  d'accord). Ce chemin est **additif** et ne vote que sur des `RunProof` **signés**.
- **N3 — commit-reveal + SENTINEL (escalade de litige).** Deux primitives
  *orthogonales* : (a) un *commit-reveal* d'activation à la opML-style ancre
  *quel* fingerprint un worker assume ; (b) SENTINEL, une EMA entière
  *forward-only*, localise **directement (O(1) par frontière**, **ce n'est pas
  une bissection)** quelle frontière contester. N3 n'est **pas** la *soundness*
  d'un fraud-proof opML (SBFB n'a pas de VM déterministe bit-exacte) : il localise
  un litige, il ne prouve pas cryptographiquement la correction.

**Ce que l'échelle ne fait pas (honnêteté).** Le réseau **ne « vérifie » pas
chaque shard** aujourd'hui : les primitives sont câblées et testées, et
l'émission signée du `RunProof` DRIVER est **livrée (S81 I/J)** — mais le
re-exec GPU réel, le transport du sketch complet hors du slot 32 octets et les
preuves per-worker sont re-routés **S82**. Le chemin de
résultat *live* reste le **quorum exact-match sur `result_text`, INCHANGÉ**.
L'incitation est du **kudos non-monétaire** (réputation) : il n'y a **aucune
défense anti-vérificateur-paresseux** et **jamais** de slash/bond/burn (interdit
par décision PO). La garantie cryptographique forte (N4 zkML) est **hors-scope
S77**. La mitigation SI-5 (padding constant-rate contre le side-channel de
latence) dérive du benchmark réel — la baseline existe depuis S81 J, le padding
lui-même est re-routé **S82**.

## Posture de sécurité

Le modèle est **honest-but-curious**. L'allowlist borne *qui* participe (admission),
pas l'honnêteté ni la confidentialité (cf. le caveat cardinal en tête de page).
La confidentialité ne tient que si **au moins un worker du pipeline est honnête**
(SI-4). Le catalogue complet des surfaces (SI-1..SI-11), les sévérités et les
mitigations câblées vivent dans
[`docs/security/THREAT_MODEL.md`](../security/THREAT_MODEL.md) §16 — ce document
y **renvoie** sans le recopier. Les patterns d'implémentation correspondants sont
[`docs/rust/PATTERNS.md`](../rust/PATTERNS.md) §P64 à §P69.
