# Strategie de Bump de Version du Feed Public SBFB

**Date :** 2026-05-18
**Scope :** Versioning du `PublicFeedOperation` enum et `FEED_FORMAT_VERSION`
**Confiance globale :** HIGH (code source lu, spec lue, patterns externes verifies)
**Declencheur :** Feedback GPT 5.5 + roadmap S65-S75 qui ajoute 3+ nouveaux types d'operations

---

## 1. Etat actuel du feed — analyse exhaustive

### 1.1 Le enum `PublicFeedOperation`

```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op_type")]
pub enum PublicFeedOperation {
    ReleasePublished(ReleasePublishedPayload),
    SourceBecameStale(SourceBecameStalePayload),
}
```

**Serialisation :** internally tagged via `#[serde(tag = "op_type")]`.
Un `ReleasePublished` produit :

```json
{
  "op_type": "ReleasePublished",
  "project_id": "...",
  "repo_url": "...",
  ...
}
```

**Consequence critique :** un enum serde internally tagged avec des variants
nommes est **ferme par defaut**. Si un noeud recoit un JSON avec
`"op_type": "CuratorVouched"` et que son code ne connait que `ReleasePublished`
et `SourceBecameStale`, serde retourne une **erreur de deserialisation**.
Le JSON entier du `FeedEntry` echoue a parser. L'entry est rejetee.

### 1.2 Le `FeedEntry` et le `FeedEntryCanonical`

```rust
pub struct FeedEntry {
    pub version: u16,           // FEED_FORMAT_VERSION
    pub seq: u64,
    pub op: PublicFeedOperation,
    pub author_pubkey: String,
    pub timestamp: u64,
    pub entry_hash: String,
    pub prev_hash: String,
    pub signature: String,
    pub pow_nonce: Option<u64>,  // #[serde(default)]
}

pub struct FeedEntryCanonical {
    pub version: u16,
    pub op: PublicFeedOperation,
    pub author_pubkey: String,
    pub timestamp: u64,
    pub prev_hash: String,
}
```

Le `version` est ecrit dans chaque entry au moment de l'insertion
(`version: FEED_FORMAT_VERSION`). Il fait partie des canonical bytes
(il est dans `FeedEntryCanonical`), donc il affecte le `entry_hash`
et la `signature`.

### 1.3 Le `FEED_FORMAT_VERSION` et la spec §9

La spec `PUBLIC_FEED_SPEC.md` §9 dit explicitement :

> Adding a new `PublicFeedOperation` variant IS a breaking change
> (the enum is closed — unknown variants cause a deserialization
> error, not a silent skip)

Et :

> Each breaking change to the canonical format bumps the version

### 1.4 Le `verify_entry()` actuel

```rust
pub fn verify_entry(entry: &FeedEntry) -> Result<(), String> {
    let canonical = entry.to_canonical();
    let canonical_bytes = compute_feed_canonical_bytes(&canonical)?;
    let recomputed = compute_feed_entry_hash(&canonical)?;
    // ... compare hash, verify Ed25519 ...
}
```

`verify_entry()` ne verifie PAS le champ `version`. Il ne rejette pas
`version: 2` ni `version: 99`. Il recompute juste les canonical bytes
(qui incluent le version field) et verifie le hash + signature.

Mais attention : si `op` contient un variant inconnu, la deserialisation
echoue **avant** que `verify_entry()` soit appele. Le FeedEntry ne peut
meme pas etre construit depuis le JSON.

### 1.5 Le `verify_chain()` actuel

`verify_chain()` itere les entries par auteur et appelle `verify_entry()`
sur chacune. Si une entry ne peut pas etre deserialisee, elle n'est pas
dans le Vec et n'est pas verifiee. Cela **n'invalide pas** les entries
connues — mais le noeud a une vue incomplete du feed.

### 1.6 Le `ingest_doc_entry()` dans `feed_sync.rs`

```rust
let feed_entry: FeedEntry = match serde_json::from_slice(&content) {
    Ok(e) => e,
    Err(e) => {
        warn!(key = %key_str, error = %e, "invalid feed entry JSON");
        return;
    }
};
```

Un noeud qui recoit une entry avec un `op_type` inconnu log un warning
et **skip** l'entry. Il ne crash pas. Mais il ne l'insere pas non plus.
L'entry est perdue pour ce noeud.

### 1.7 L'`insert_feed_operation_inner()` et le match exhaustif

```rust
let op_type = match &op {
    PublicFeedOperation::ReleasePublished(_) => "ReleasePublished",
    PublicFeedOperation::SourceBecameStale(_) => "SourceBecameStale",
};
```

Ce match est exhaustif. Ajouter un variant au enum produit un
**compile error** a cet endroit (et dans `validate_feed_operation()`
et dans `ingest_doc_entry()` qui ont aussi des match exhaustifs).
C'est une propriete de securite : le compilateur force a gerer
chaque nouveau variant partout.

---

## 2. Impact sur le hash-chain — analyse cryptographique

### 2.1 Les entries existantes ne sont-elles PAS affectees ?

**OUI, les entries existantes sont intactes.** Voici pourquoi :

1. L'`entry_hash` est `BLAKE3(DOMAIN_FEED_V1 || 0x00 || JCS(FeedEntryCanonical))`.
2. Le `FeedEntryCanonical` contient `version`, `op`, `author_pubkey`, `timestamp`, `prev_hash`.
3. Les entries existantes ont `version: 1` et `op: ReleasePublished(...)` ou `SourceBecameStale(...)`.
4. Leur canonical bytes ne changent PAS si on ajoute un nouveau variant au enum.
5. Leur `entry_hash` ne change PAS.
6. Leur `signature` reste valide.

**Preuve :** les canonical bytes sont calcules a partir des **valeurs** de l'entry,
pas de la definition du type Rust. `JCS(FeedEntryCanonical{version:1, op: ReleasePublished{...}, ...})`
produit exactement les memes octets avant et apres l'ajout de `CuratorVouched` au enum.
Le hash BLAKE3 de ces octets est identique. La signature Ed25519 reste valide.

### 2.2 Un noeud ancien peut-il valider la hash-chain ?

**OUI, pour les entries qu'il connait. NON, pour les nouvelles.**

Scenario : noeud A (code v1, connait ReleasePublished + SourceBecameStale) recoit
un feed de noeud B (code v2, connait aussi CuratorVouched).

Le feed contient :
```
seq=1: ReleasePublished  (version:1) → A peut deserialiser, verifier
seq=2: SourceBecameStale  (version:1) → A peut deserialiser, verifier
seq=3: CuratorVouched     (version:2) → A ECHOUE a deserialiser
seq=4: ReleasePublished   (version:2) → A peut deserialiser, verifier (*)
```

(*) Pour seq=4, A peut deserialiser le JSON (c'est un ReleasePublished), mais
le `version: 2` dans les canonical bytes produit un `entry_hash` different de
ce que A calculerait s'il forcait `version: 1`. Cependant, `verify_entry()` ne
verifie pas que `version == FEED_FORMAT_VERSION`. Il recompute les canonical bytes
avec le `version` de l'entry elle-meme. Donc **seq=4 est verifiable par A** meme
si A ne connait que `FEED_FORMAT_VERSION == 1` dans son code.

Le probleme est seq=3 : A ne peut pas le deserialiser, donc il ne peut pas
verifier cette entry. Pour `verify_chain()`, si seq=3 est absente du Vec,
la chaine de l'auteur de seq=3 est cassee (broken linkage). Mais les chaines
des autres auteurs restent valides.

### 2.3 Le domain tag `DOMAIN_FEED_V1`

Le domain tag est `b"nexus-feed-v1"`. Il ne change PAS quand on bumpe
`FEED_FORMAT_VERSION` de 1 a 2. C'est correct : le domain tag est pour la
separation inter-types (feed vs task vs claim), pas pour le versioning
intra-type. Bumper le domain tag invaliderait TOUTES les signatures
existantes — c'est reserve aux "hard schema migrations" (cf. canonical.rs
docstring : "Bumping it changes the signature surface and invalidates every
existing signature").

---

## 3. Patterns externes — synthese de la recherche

### 3.1 Protocol Buffers (protobuf)

**Pattern :** Ajouter des valeurs a un enum est safe. Les anciens decoders
traitent les valeurs inconnues comme des entiers non reconnus et les
preservent lors de la re-serialisation. Bonne pratique : `UNKNOWN = 0` comme
valeur par defaut.

**Applicable a SBFB ?** Partiellement. Protobuf a un avantage que serde n'a
pas : les enum values protobuf sont des entiers, et un entier inconnu peut
etre preserve comme raw bytes. En serde JSON avec `#[serde(tag = "op_type")]`,
un tag string inconnu cause un **erreur de deserialisation**, pas une valeur
default.

### 3.2 Cap'n Proto

**Pattern :** Nouveaux enum values et union members sont forward-compatible
tant qu'ils ont des numeros superieurs. Les readers sont avertis de gerer
les valeurs inconnues dans les switch blocks.

**Applicable a SBFB ?** Meme logique que protobuf : fonctionne grace aux
representations binaires. JSON tagged unions n'ont pas cet avantage.

### 3.3 Avro

**Pattern :** Ajouter des symbols a un enum est backward-compatible (nouveau
reader lit ancien data). Mais PAS forward-compatible (ancien reader ne peut
pas lire nouveau data avec un symbol inconnu).

**Applicable a SBFB ?** Tres pertinent. La situation SBFB est exactement celle
d'Avro : ajouter un variant est backward-compatible (les anciennes entries
restent lisibles) mais pas forward-compatible (les anciens noeuds ne peuvent
pas lire les nouvelles entries).

### 3.4 Scuttlebutt (SSB)

**Pattern :** Le `type` d'un message est un string libre (3-53 chars). Les
implementations qui ne connaissent pas un type le stockent quand meme et le
repliquent. C'est le modele "store-and-forward even if unknown".

**Applicable a SBFB ?** Tres pertinent. SSB resout le probleme en ne deserialisant
PAS le contenu du message en un type Rust — chaque message est un JSON
opaque avec un champ `type`. L'application filtre par type. C'est exactement
le pattern "raw JSON + type tag" qui est l'alternative a un enum ferme.

### 3.5 ActivityPub / ActivityStreams

**Pattern :** Vocabulaire extensible par construction. Nouveaux types sont
ajoutes comme extensions JSON-LD. Les implementations qui ne comprennent pas
un type l'ignorent (pas de crash). Le processus d'extension est leger.

**Applicable a SBFB ?** Le pattern "ignore what you don't understand" est
directement applicable. Mais ActivityPub n'a pas de hash-chain — il peut
se permettre de dropper des messages inconnus sans consequences crypto.

### 3.6 Bitcoin SegWit (witness versions)

**Pattern :** Les scripts sont precedes d'un numero de version. Ajouter un
nouveau type de script = incrementer la version. Les anciens noeuds traitent
les nouvelles versions comme "anyone-can-spend" (forward-compatible par soft
fork). Les nouveaux noeuds appliquent les regles completes.

**Applicable a SBFB ?** Le concept de "les anciens noeuds acceptent sans
comprendre" est tres pertinent. En SBFB, un ancien noeud pourrait accepter
une entry avec un op_type inconnu en validant seulement le hash + signature
sans comprendre la semantique de l'operation.

### 3.7 Matrix (room versions)

**Pattern :** Chaque changement incompatible cree une nouvelle "room version".
Les rooms ne sont pas mises a jour — une nouvelle room est creee et l'ancienne
est fermee. Pas d'ordering hierarchique entre versions.

**Applicable a SBFB ?** Pas directement. Un feed est un log continu, pas une
room qu'on peut fermer et recreer. Le pattern de Matrix est trop disruptif
pour un append-only log.

### 3.8 Ethereum (transaction types)

**Pattern :** Chaque upgrade peut introduire de nouveaux types de transactions
(Type 1, Type 2, Type 3...). Les anciens noeuds qui ne comprennent pas un
type de transaction peuvent quand meme valider le block (grace au consensus).

**Applicable a SBFB ?** Le concept de "nouveaux types coexistent avec les
anciens dans le meme log" est directement applicable. Ethereum montre qu'un
log (blockchain) peut contenir des types heterogenes si chaque type est
auto-descriptif.

### 3.9 Synthese : le pattern dominant

| Protocole | Strategy | Forward-compatible? | Hash-chain? |
|-----------|----------|---------------------|-------------|
| Protobuf | Entiers inconnus preserves | OUI | Non |
| Cap'n Proto | Numeros superieurs | OUI | Non |
| Avro | Symbols inconnus = erreur | NON | Non |
| SSB | Type string libre, store-and-forward | OUI | OUI |
| ActivityPub | Ignore unknown types | OUI | Non |
| Bitcoin | Witness version, accept-without-understanding | OUI | OUI |
| Matrix | Nouvelle room, pas de migration | N/A | Non |
| Ethereum | Types heterogenes dans le meme log | OUI (via consensus) | OUI |

**Les protocoles avec hash-chain (SSB, Bitcoin, Ethereum) utilisent tous
le pattern "store-and-forward even if unknown".** C'est la seule approche
compatible avec un log append-only cryptographiquement lie.

---

## 4. Analyse des 4 options

### 4.1 Option A — Bump unique v1->v2 en S67

**Description :** Ajouter CuratorVouched + CuratorDisendorsed +
SearchManifestPublished + SourceRecovered + BuildQuorumReached en
un seul bump. `FEED_FORMAT_VERSION = 2` a partir de S67.

**Avantages :**
- Un seul bump, un seul decoder range (v1..v2)
- Tous les noeuds passent a v2 en une fois
- Pas de periode avec des noeuds v1 et v2 en parallele (puisque pas de noeuds tiers avant S69)

**Problemes :**
- **SearchManifestPublished n'est pas designe.** La recherche S70-S72 n'est pas
  encore faite. Figer le payload de `SearchManifestPublished` en S67 alors que
  S72 est a 10+ sprints d'ecart est premature. Le feedback GPT 5.5 suggere de
  batcher, mais batcher implique que le design est pret — ce n'est pas le cas.
- **BuildQuorumReached et SourceRecovered non plus.** La spec les mentionne
  comme "future" mais aucune recherche n'a defini leurs payloads.
- **Couplage temporel :** S67 devient un mega-sprint "tout le feed v2" au lieu
  d'un sprint focalise sur la gouvernance de confiance.

**Verdict : REJETE.** On ne fige pas un wire format pour un feature non designee.

### 4.2 Option B — Deux bumps separes (v1->v2 en S67, v2->v3 en S72)

**Description :** CuratorVouched + CuratorDisendorsed en v2 (S67).
SearchManifestPublished en v3 (S72). Eventuellement BuildQuorumReached et
SourceRecovered en v2 ou v3 selon leur timing.

**Avantages :**
- Chaque bump est informe par la recherche du sprint correspondant
- Pas de wire format premature

**Problemes :**
- Les noeuds doivent supporter v1+v2 puis v1+v2+v3. Decoders range croissant.
- Chaque bump est un breaking change qui force la mise a jour de tous les noeuds.
  Pour un reseau naissant (pre-launch), ce n'est pas grave. Pour un reseau avec
  des tiers (post-S69 pilote), c'est un frottement.
- Le feedback GPT 5.5 identifie correctement que v2 puis v3 est inelegant.

**Verdict : POSSIBLE mais sous-optimal.** C'est le choix le plus conservateur
si on reste dans le paradigme "enum ferme + version bump".

### 4.3 Option C — Pas de bump, `#[serde(other)]` pour les ops inconnues

**Description :** Ajouter un variant `Unknown` avec `#[serde(other)]` au enum.
Les anciens noeuds deserialisent les ops inconnues comme `Unknown` et les
stockent/propagent sans les comprendre.

**Avantages :**
- Forward-compatible : un noeud v1 peut recevoir, stocker, et propager une
  entry CuratorVouched sans la comprendre
- Pas de bump de version necessaire pour de nouvelles ops
- Pattern SSB/Bitcoin : store-and-forward

**Problemes CRITIQUES :**

1. **`#[serde(other)]` ne fonctionne que sur un unit variant.** Cela signifie :
   ```rust
   #[serde(tag = "op_type")]
   pub enum PublicFeedOperation {
       ReleasePublished(ReleasePublishedPayload),
       SourceBecameStale(SourceBecameStalePayload),
       #[serde(other)]
       Unknown,  // <-- UNIT VARIANT, pas de payload
   }
   ```
   Le payload de l'operation inconnue est **perdu**. On sait juste que c'est
   "quelque chose d'inconnu". On ne peut pas le re-serialiser, on ne peut pas
   le propager, on ne peut pas recalculer les canonical bytes.

2. **Verification crypto impossible.** Si le payload est perdu, les canonical bytes
   ne peuvent pas etre recalcules. L'`entry_hash` ne peut pas etre verifie. La
   signature ne peut pas etre verifiee. L'entry `Unknown` est un trou noir
   cryptographique — on sait qu'elle existe mais on ne peut rien prouver dessus.

3. **Serialisation brisee.** Si on re-serialise `Unknown` en JSON, on obtient
   `{"op_type":"Unknown"}`. Ce n'est pas le JSON original. Les canonical bytes
   changent. Le hash change. La propagation iroh-docs ne depend pas de la
   re-serialisation (elle propage le blob original), mais toute operation qui
   reconstruit l'entry depuis le Rust struct echoue.

**Verdict : REJETE dans sa forme pure.** `#[serde(other)]` perd le payload,
ce qui est incompatible avec la verification cryptographique du feed.

### 4.4 Option D — Feed version != operation version (hybrid)

**Description :** Le `FEED_FORMAT_VERSION` controle la structure de `FeedEntry`
(les champs du wrapper). Les operations sont extensibles independamment.
Ajouter une nouvelle operation ne bumpe pas la version du feed. Bumper
seulement si la structure FeedEntry change (nouveaux champs obligatoires).

**Avantages :**
- Separation claire : la "version" dit comment lire l'enveloppe, pas quel type
  d'operations sont possibles
- Ajouter un type d'operation est une extension, pas un breaking change
- Les anciens noeuds peuvent toujours valider les entries qu'ils connaissent

**Probleme :** Ca ne resout pas la deserialisation. Si un ancien noeud
recoit une entry avec un `op_type` inconnu, serde echoue quand meme a
deserialiser le `PublicFeedOperation` embedded dans `FeedEntry`. La separation
conceptuelle est bonne mais l'implementation bute sur le meme mur : l'enum
Rust est ferme.

**Verdict : BONNE DIRECTION mais necessite un mecanisme technique pour
gerer les variants inconnus.**

---

## 5. Option E — la strategie recommandee : RawOperation hybrid

### 5.1 Le pattern SSB applique a SBFB

La vraie solution est celle que SSB, Bitcoin et Ethereum utilisent :
**stocker l'operation comme du JSON opaque au niveau du transport, et
ne la parser en type Rust que quand on a besoin de la comprendre.**

### 5.2 Architecture proposee

```rust
/// Le feed entry tel que stocke et transmis.
/// L'operation est un JSON opaque (serde_json::Value) pour le transport.
/// Le parsing en PublicFeedOperation est fait a la demande.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FeedEntry {
    pub version: u16,
    pub seq: u64,
    pub op: serde_json::Value,  // <-- JSON opaque, pas un enum type
    pub author_pubkey: String,
    pub timestamp: u64,
    pub entry_hash: String,
    pub prev_hash: String,
    pub signature: String,
    #[serde(default)]
    pub pow_nonce: Option<u64>,
}

/// L'enum type pour les operations connues.
/// Utilise quand on veut interpreter l'operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op_type")]
pub enum PublicFeedOperation {
    ReleasePublished(ReleasePublishedPayload),
    SourceBecameStale(SourceBecameStalePayload),
    CuratorVouched(CuratorVouchedPayload),          // S67
    CuratorDisendorsed(CuratorDisendorsedPayload),  // S67
    // SearchManifestPublished(...)                  // S72
    // SourceRecovered(...)                          // S67 ou S68
    // BuildQuorumReached(...)                       // futur
}

impl FeedEntry {
    /// Try to parse the operation into a known type.
    /// Returns None if the op_type is unknown to this version.
    pub fn try_parse_op(&self) -> Option<PublicFeedOperation> {
        serde_json::from_value(self.op.clone()).ok()
    }

    /// Get the op_type string without parsing the full operation.
    pub fn op_type(&self) -> Option<&str> {
        self.op.get("op_type")?.as_str()
    }
}
```

### 5.3 Pourquoi ca marche avec la crypto

**Verification du hash :** Le `FeedEntryCanonical` contient `op: serde_json::Value`.
`serde_jcs::to_vec(canonical)` produit le JCS du JSON opaque. Puisque le JSON
original est preserve integralement dans `serde_json::Value`, les canonical bytes
sont **identiques** a ceux que le noeud emetteur a calcules. Le hash BLAKE3
match. La signature Ed25519 est verifiable.

**Crucial :** `serde_json::Value` preserve TOUTES les cles et valeurs du JSON.
JCS trie les cles lexicographiquement. Tant que le JSON original est valide
et que JCS est deterministe (ce qui est garanti par RFC 8785), les canonical
bytes sont reproductibles meme pour des operations inconnues.

### 5.4 Impact sur chaque composant

| Composant | Changement | Risque |
|-----------|------------|--------|
| `FeedEntry.op` | `PublicFeedOperation` -> `serde_json::Value` | LOW (le JSON est le meme) |
| `FeedEntryCanonical.op` | `PublicFeedOperation` -> `serde_json::Value` | LOW (JCS produit les memes bytes) |
| `compute_feed_entry_hash()` | Aucun | ZERO (JCS sur Value = meme bytes) |
| `compute_feed_canonical_bytes()` | Aucun | ZERO |
| `verify_entry()` | Aucun (recompute hash + verify sig) | ZERO |
| `verify_chain()` | Aucun | ZERO |
| `validate_feed_operation()` | Accepte `&serde_json::Value`, parse en enum, valide | LOW |
| `insert_feed_operation()` | Accepte `PublicFeedOperation`, serialise en Value | LOW |
| `ingest_doc_entry()` | Deserialise en FeedEntry (avec Value), hash/sig OK pour tous types | **GAIN** |
| `insert_feed_operation_inner()` | Le match pour `op_type` string utilise `entry.op_type()` | LOW |
| `replay_all()` | Retourne FeedEntry avec Value, le materializer parse les ops connues | LOW |
| `feed_sync.rs` ingest | Ne rejette plus les ops inconnues, les insere dans la DB | **GAIN** |
| DB schema `feed_entries` | Colonne `payload` reste TEXT (JSON) | ZERO |

### 5.5 Ce que les anciens noeuds font des nouvelles ops

Avec cette architecture, un noeud v1 (qui ne connait pas `CuratorVouched`) :

1. **Recoit** l'entry via iroh-docs sync
2. **Deserialise** le JSON en `FeedEntry` avec `op: serde_json::Value` -- **SUCCES**
3. **Verifie** `entry_hash` (recompute canonical bytes du Value) -- **SUCCES**
4. **Verifie** la signature Ed25519 -- **SUCCES**
5. **Verifie** le timestamp, le PoW, la dedup -- **SUCCES**
6. **Insere** dans la DB locale -- **SUCCES**
7. **Tente** `try_parse_op()` -> `None` (type inconnu)
8. **Ignore** l'operation pour la materialisation (pas de `BrowseEntry` mis a jour)
9. **Propage** via iroh-docs sync aux autres noeuds -- **SUCCES** (le blob original est intact)

La hash-chain reste intacte. Le noeud a une vue incomplete du feed (il ne
materialise pas les CuratorVouched) mais il n'a pas de trou dans la chaine.

### 5.6 Quand bumper `FEED_FORMAT_VERSION` ?

Avec cette architecture, `FEED_FORMAT_VERSION` ne bumpe que quand la
**structure de `FeedEntry` change**, pas quand on ajoute des operations.

Exemples de breaking changes qui justifient un bump :
- Ajouter un champ obligatoire a `FeedEntry` (ex: `merkle_root`)
- Changer le domain tag `DOMAIN_FEED_V1`
- Changer l'algorithme de hash (BLAKE3 -> autre)
- Changer le format de `FeedEntryCanonical` (ajouter/retirer un champ)

Exemples qui ne sont PAS des breaking changes :
- Ajouter `CuratorVouched` a l'enum `PublicFeedOperation`
- Ajouter `SearchManifestPublished` a l'enum
- Ajouter un champ optionnel a un payload existant (`#[serde(default)]`)

### 5.7 Verification de la faisabilite JCS

**Question critique :** Est-ce que `serde_jcs::to_vec(value)` produit les memes
bytes quand `value` est un `serde_json::Value` que quand `value` est un
`PublicFeedOperation` type ?

**Reponse : OUI.** JCS (RFC 8785) est defini sur le JSON, pas sur le type Rust.
`serde_jcs::to_vec(&value)` serialise un `serde_json::Value` en JSON
canonique. Le resultat est identique a `serde_jcs::to_vec(&typed_struct)` si
les deux representent le meme JSON.

**Preuve par construction :**
1. Le noeud emetteur serialise `PublicFeedOperation::CuratorVouched(payload)`
   via `serde_jcs::to_vec` -> bytes A
2. Le noeud recepteur deserialise en `serde_json::Value` puis
   re-serialise via `serde_jcs::to_vec` -> bytes B
3. bytes A == bytes B car JCS est deterministe et la representation intermediaire
   `serde_json::Value` preserve toutes les cles et valeurs.

**Caveat :** Il faut s'assurer que la deserialization `serde_json::from_slice`
preserve exactement les types numeriques. `serde_json::Value` represente les
entiers comme `Number(u64/i64/f64)`. Si le JSON original contient `42`,
`serde_json::Value` le lit comme `Number(42)`, et `serde_jcs::to_vec` le
re-ecrit comme `42`. Pas de probleme pour les entiers. Pour les floats, JCS
a des regles specifiques, mais le feed SBFB n'utilise pas de floats dans les
payloads d'operations.

---

## 6. Plan d'implementation concret

### 6.1 Migration du type `op` (S65 ou S67 Phase 0)

Cette migration est une refacto interne, pas un changement de wire format.
Le JSON sur le fil ne change pas. Les entries existantes en DB ne changent
pas. Le type Rust change.

**Avant :**
```rust
pub struct FeedEntry {
    pub op: PublicFeedOperation,
    // ...
}
```

**Apres :**
```rust
pub struct FeedEntry {
    pub op: serde_json::Value,
    // ...
}
```

**Changements requis :**

1. **`public_feed.rs`** : Changer le type de `FeedEntry.op` et
   `FeedEntryCanonical.op` en `serde_json::Value`. Ajouter
   `try_parse_op()` et `op_type()`.

2. **`insert_feed_operation()`** : Serialise l'op en Value avant
   de l'inserer dans le FeedEntry.

3. **`validate_feed_operation()`** : Accepte `&serde_json::Value`,
   parse en `PublicFeedOperation`, valide. Retourne erreur si le
   type est inconnu (pour les inserts locaux — on ne publie que
   des ops connues).

4. **`insert_feed_operation_inner()`** : Le match pour `op_type`
   string utilise `op_type()` sur le Value.

5. **`replay_all()`** : Parse le JSON de la DB directement en Value.

6. **`ingest_doc_entry()`** dans feed_sync.rs : Le FeedEntry se
   deserialise toujours. Le `validate_feed_operation()` est appele
   seulement si `try_parse_op()` retourne `Some`. Les ops inconnues
   sont validees au niveau crypto (hash + signature) mais pas au
   niveau semantique.

7. **Tests** : Adapter tous les tests qui construisent des FeedEntry
   avec un `op: PublicFeedOperation`. Ajouter un test "unknown op_type
   can be deserialized, verified, and stored".

### 6.2 Ajout de CuratorVouched et CuratorDisendorsed (S67)

Avec le pattern `serde_json::Value`, c'est un simple ajout de variants
a l'enum `PublicFeedOperation` + validation + tests. Pas de bump de
`FEED_FORMAT_VERSION`. Pas de decoder range. Les noeuds pre-S67
(s'il y en a apres le pilote S69) stockent et propagent les entries
CuratorVouched sans les comprendre.

### 6.3 Ajout de SearchManifestPublished (S72)

Meme pattern. Ajout de variant + validation + tests. Toujours pas
de bump de `FEED_FORMAT_VERSION`. Les noeuds pre-S72 stockent et
propagent.

### 6.4 Ajout futur de SourceRecovered, BuildQuorumReached

Meme pattern. Chaque nouveau type d'operation est un ajout non-breaking
au enum.

---

## 7. Reponse au feedback GPT 5.5

Le feedback disait :

> "Si SearchManifestPublished entre dans le feed, c'est un breaking change v2.
> A batcher avec les operations S67 si possible, pour eviter v2 puis v3
> inutilement."

**Ce feedback est correct dans le paradigme actuel** (enum ferme, chaque
nouveau variant = breaking change). Mais il est **depasse par l'Option E**
qui rend l'ajout de nouveaux types non-breaking par construction.

Avec l'Option E :
- CuratorVouched en S67 : pas de bump
- SearchManifestPublished en S72 : pas de bump
- Pas besoin de batcher
- Pas de dilemme "figer un format premature vs deux bumps"

---

## 8. Impact sur la politique pre-launch (CLAUDE.md)

### 8.1 Ce qui change

La section "Pre-launch protocol policy" de CLAUDE.md dit actuellement :

> `*_FORMAT_VERSION` / `*_ANNOUNCEMENT_VERSION` restent a 1 jusqu'au premier
> tag v1.0.

Avec l'Option E, cette politique s'affine :

1. `FEED_FORMAT_VERSION` reste a 1 non seulement pre-launch mais aussi
   **tant que la structure de FeedEntry ne change pas**. Ajouter des
   operations ne bumpe plus la version.

2. La spec §9 doit etre mise a jour : "Adding a new PublicFeedOperation
   variant is NOT a breaking change" (au lieu de "IS a breaking change").

3. La phrase "the enum is closed — unknown variants cause a deserialization
   error, not a silent skip" doit etre remplacee par "the enum at the
   transport layer is open (JSON Value) — unknown op_types are stored and
   propagated but not semantically interpreted".

### 8.2 Texte propose pour CLAUDE.md

```markdown
## Version policy — Feed operations

Le `FEED_FORMAT_VERSION` controle la structure de `FeedEntry` (enveloppe :
version, seq, op, author, timestamp, hashes, signature). Il bumpe
uniquement quand la structure de l'enveloppe change (nouveau champ
obligatoire, changement de hash algorithm, changement de domain tag).

Les operations (`PublicFeedOperation` variants) sont extensibles sans
bump de version. Le transport utilise `serde_json::Value` pour le champ
`op`, ce qui permet aux noeuds anciens de stocker, verifier (hash +
signature), et propager des operations qu'ils ne connaissent pas.
L'interpretation semantique est faite via `try_parse_op()` qui retourne
`None` pour les types inconnus.

Ajouter un nouveau variant a `PublicFeedOperation` n'est PAS un breaking
change. La verification cryptographique (BLAKE3 hash + Ed25519 signature)
fonctionne sur le JSON opaque, independamment du type d'operation.
```

---

## 9. Impact sur la spec PUBLIC_FEED_SPEC.md

### 9.1 Section §9 — Versioning policy (a recrire)

**Actuel :**
> Adding a new `PublicFeedOperation` variant IS a breaking change (the
> enum is closed — unknown variants cause a deserialization error, not
> a silent skip)

**Propose :**
> Adding a new `PublicFeedOperation` variant is NOT a breaking change.
> The transport layer stores operations as opaque JSON (`serde_json::Value`).
> Unknown op_types are stored, cryptographically verified (BLAKE3 hash +
> Ed25519 signature), and propagated to peers. Semantic interpretation
> (validation, materialization) is skipped for unknown types.
>
> Breaking changes that bump `FEED_FORMAT_VERSION`:
> - Adding/removing a required field in `FeedEntry` or `FeedEntryCanonical`
> - Changing the hash algorithm (BLAKE3 -> other)
> - Changing the domain tag (`DOMAIN_FEED_V1`)
> - Changing the canonical serialization format (JCS -> other)
>
> Non-breaking changes:
> - Adding a new variant to `PublicFeedOperation`
> - Adding an optional field to an existing payload (`#[serde(default)]`)

### 9.2 Nouvelle section §9.1 — Forward compatibility

> **Forward compatibility:** Nodes running older software versions MUST
> accept, verify, store, and propagate feed entries containing unknown
> operation types. This is achieved by deserializing the `op` field as
> `serde_json::Value` rather than a typed enum.
>
> For unknown operations:
> 1. `entry_hash` verification: PASS (canonical bytes computed from JSON Value)
> 2. `signature` verification: PASS (Ed25519 over canonical bytes)
> 3. `validate_feed_operation()`: SKIPPED (no semantic validation possible)
> 4. `materialize()`: SKIPPED (no state transition for unknown ops)
> 5. `verify_chain()`: PASS (hash-chain linkage is op-agnostic)

---

## 10. Risques et mitigations

### 10.1 Risque : perte de validation semantique pour les ops inconnues

Un noeud qui ne connait pas `CuratorVouched` ne peut pas valider que le
`project_id` est hex-64, que le `curator_pubkey` est hex-64, etc. Il
stocke l'entry "aveuglément" (mais cryptographiquement verifiee).

**Mitigation :** C'est acceptable. La validation semantique est faite par
les noeuds qui connaissent le type. Le materializer ignore les ops inconnues.
Un noeud qui ne comprend pas CuratorVouched ne produit pas de BrowseEntry
incorrect — il ne produit rien.

### 10.2 Risque : payload malveillant dans une op inconnue

Un attaquant pourrait creer un `op_type: "EvilOp"` avec un payload
JSON de 64KB contenant du spam. Le noeud stocke l'entry.

**Mitigation :** Le `MAX_OPERATION_JSON_SIZE = 65_536` bytes est deja
enforce **avant** le parsing de l'op type. On peut le verifier sur le
`serde_json::Value` brut (serialiser et mesurer la taille). Le rate
limiter (5/min/author) s'applique aussi. Le PoW 16-bit s'applique aussi.
Les 3 couches de defense sont op-agnostiques.

### 10.3 Risque : regression de la couverture de test

Le match exhaustif sur `PublicFeedOperation` dans `insert_feed_operation_inner()`
et `validate_feed_operation()` force le compilateur a gerer chaque variant.
Si on passe en `serde_json::Value`, on perd cette garantie.

**Mitigation :** Le code qui interprete les operations (materializer,
validation semantique, API responses) continue d'utiliser l'enum type via
`try_parse_op()`. Le match exhaustif reste pour l'interpretation. Il est
juste decouple du transport.

### 10.4 Risque : JCS determinism sur serde_json::Value

Si `serde_json::Value` perd une information lors du roundtrip (ex: un
nombre tres grand qui overflow un f64), les canonical bytes pourraient
differer.

**Mitigation :** Les payloads SBFB n'utilisent que des strings, des u64,
des booleans, et des optionals. Pas de floats, pas de grands nombres.
`serde_json::Value::Number` represente correctement tous les u64. Le risque
est theorique pour SBFB.

**Test preventif a ecrire :** Un test qui serialise un `PublicFeedOperation`
en `serde_json::Value`, recalcule les canonical bytes, et verifie qu'ils sont
identiques aux canonical bytes calcules directement depuis le type.

---

## 11. Verdict final

### Recommandation : Option E — `serde_json::Value` + enum pour l'interpretation

**C'est la seule option qui :**
1. Preserve la verification cryptographique pour TOUTES les entries (connues et inconnues)
2. Preserve la hash-chain sans trous
3. Permet l'ajout de nouveaux types d'operations sans bump de version
4. Est forward-compatible (les anciens noeuds gèrent les nouvelles ops)
5. Est backward-compatible (les nouveaux noeuds lisent les anciennes entries)
6. Suit le pattern dominant des protocoles a hash-chain (SSB, Bitcoin, Ethereum)
7. Elimine le dilemme "batcher vs bumps multiples" souleve par GPT 5.5

### Quand implementer ?

**Sprint 65 (go-live) ou au plus tard S67 Phase A (avant d'ajouter CuratorVouched).**

Justification :
- La migration `PublicFeedOperation` -> `serde_json::Value` dans `FeedEntry` est une
  refacto interne qui ne change pas le wire format
- Elle doit etre faite AVANT le premier ajout d'operation (S67 CuratorVouched)
  pour eviter un bump de version inutile
- S65 est le dernier sprint avant le pilote S69 — c'est le bon moment pour
  stabiliser l'architecture du feed avant que des tiers le consomment

### Ampleur du changement

- ~100-150 lignes de code modifiees dans `public_feed.rs`
- ~50 lignes dans `feed_sync.rs`
- ~20-30 lignes dans les endpoints HTTP
- ~10-15 tests a adapter + 3-5 nouveaux tests
- Spec §9 a recrire (~20 lignes)
- CLAUDE.md section pre-launch a mettre a jour (~10 lignes)

C'est une refacto de taille Phase B (une demi-journee), pas un sprint entier.

---

## 12. Timeline recommandee

```
S65 Phase X :  Migration FeedEntry.op -> serde_json::Value
               Spec §9 update. CLAUDE.md update.
               Tests : "unknown op_type can be stored, verified, propagated"
               Tests : "canonical bytes identical for Value vs typed struct"

S67 Phase A :  Ajouter CuratorVouched + CuratorDisendorsed au enum
               validate_feed_operation() pour les nouveaux types
               Tests unitaires + adversariaux
               FEED_FORMAT_VERSION reste a 1 ← pas de bump

S67+ :         Ajouter SourceRecovered si designe
               FEED_FORMAT_VERSION reste a 1

S72 Phase B :  Ajouter SearchManifestPublished au enum
               validate_feed_operation() pour le nouveau type
               FEED_FORMAT_VERSION reste a 1

Premier bump FEED_FORMAT_VERSION = 2 :
               Quand on change la structure de FeedEntry elle-meme
               (ex: ajout d'un champ merkle_root, changement de hash algo)
               Probablement post-v1.0 (pas prevu dans la roadmap S65-S75)
```

---

## 13. Sources

### Code source analyse
- `crates/nexus-coordinator-rs/src/public_feed.rs` — FeedEntry, PublicFeedOperation, verify_chain, 30+ tests
- `crates/nexus-shell-daemon/src/feed_sync.rs` — feed subscription, ingest, publish
- `crates/nexus-core-rs/src/canonical.rs` — JCS canonical bytes, domain separation
- `docs/protocol/PUBLIC_FEED_SPEC.md` — spec complete avec §9 versioning policy
- `.planning/codebase/protocol_wire_formats.md` — wire formats documentes
- `.planning/research/s67_gouvernance_confiance_research.md` — CuratorVouched/Disendorsed design
- `.planning/research/s70_s72_rrv_research.md` — SearchManifestPublished design

### Patterns de versioning externes
- [Protocol Buffers Schema Evolution Guide](https://jsontotable.org/blog/protobuf/protobuf-schema-evolution) — protobuf enum evolution
- [Cap'n Proto Schema Language](https://capnproto.org/language.html) — union member evolution
- [Avro Schema Evolution](https://laso-coder.medium.com/avro-schema-evolution-demystified-backward-and-forward-compatibility-explained-561beeaadc6b) — enum symbol compatibility
- [Serde Variant Attributes](https://serde.rs/variant-attrs.html) — #[serde(other)] documentation
- [Serde issue #912](https://github.com/serde-rs/serde/issues/912) — tagged enums + #[serde(other)]
- [SSB Specification — Feeds and Messages](https://spec.scuttlebutt.nz/feed/messages.html) — type string libre, forward-compatible
- [Scuttlebutt Protocol Guide](https://ssbc.github.io/scuttlebutt-protocol-guide/) — store-and-forward pattern
- [ActivityStreams 2.0 Extensions Policy](https://swicg.github.io/extensions-policy/) — vocabulary extension without breaking changes
- [Bitcoin SegWit](https://en.bitcoin.it/wiki/Segregated_Witness) — witness version, script versioning
- [Matrix Room Versions](https://spec.matrix.org/unstable/rooms/) — room version upgrade strategy
- [Ethereum EIP-1702](https://eips.ethereum.org/EIPS/eip-1702) — account versioning scheme
- [Ethereum Transaction Types](https://www.turnkey.com/blog/eip-4844-and-eip-7702-how-turnkey-supports-new-ethereum-transaction-types) — new tx types coexisting in same chain
