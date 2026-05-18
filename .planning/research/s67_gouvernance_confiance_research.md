# S67 Gouvernance De Confiance — Recherche exhaustive

**Date :** 2026-05-18
**Sprint :** 67
**Objectif produit :** Rendre la confiance lisible et pluraliste.

---

## 1. Etat actuel du systeme curator — analyse exhaustive du code

### 1.1 CuratorList : struct, signing, validation

**Fichier :** `crates/nexus-core-rs/src/curator.rs`

La struct `CuratorList` est le payload non-signe :

```rust
pub struct CuratorList {
    pub version: u16,                          // == 1
    pub curator_pubkey: [u8; 32],              // Ed25519 pubkey
    pub curator_name: String,                  // display name
    pub created_at: u64,                       // unix timestamp
    pub revision: u64,                         // monotonic counter
    pub entries: Vec<CuratorProjectRef>,        // max 256
}
```

Chaque entree `CuratorProjectRef` contient :
- `project_id` (pkarr node id hex, 64 chars, max 128 bytes)
- `project_name` (max 128 bytes)
- `category` (max 64 bytes)
- `description` (max 280 bytes)

**Signing :** `CuratorListEntry::sign(list, keypair)` produit une enveloppe avec signature Ed25519 sur `canonical_bytes(list, DOMAIN_CURATOR_LIST_V1)`. JCS RFC 8785 pour la serialisation canonique.

**Verification (5 checks) :**
1. `version == CURATOR_LIST_FORMAT_VERSION` (= 1)
2. `entries.len() <= 256` (DoS cap)
3. Per-field byte caps sur chaque entree
4. Attribution consistency : `list.curator_pubkey == envelope.curator_pubkey`
5. Ed25519 signature valide

**Verification avancee :**
- `verify_with_revocation(cache, now_ts)` : verifie contre le `RevocationCache`. Rejette si la cle est revoquee (transition expiree). Retourne `Ok(true)` si la cle est en transition (warning).
- `verify_with_contributor_registry(registry)` : Couche 2 governance-strong. Pour chaque entree dont le `project_id` est enrolled dans le registre, le curator_pubkey doit etre un contributeur verifie.

**Tests :** 21 tests couvrant sign/verify, tamper, attribution mismatch, DoS cap, field caps, domain separation, JSON roundtrip, revocation integration, contributor registry.

### 1.2 Distribution gossip des listes curator

**Fichier :** `crates/nexus-shell-daemon-core/src/iroh_runtime.rs`

Le `CuratorRuntime` gere :
- **Attention set** (`DashMap<[u8; 32], ()>`) : les curators auxquels l'utilisateur est abonne. Seules les annonces de curators dans ce set sont traitees.
- **Liste cache** (`DashMap<[u8; 32], CuratorListEntry>`) : les listes verifiees les plus recentes.
- **Rollback protection** : revision monotonique enforcee a l'insertion.
- **Backpressure** : semaphore limitant les `process_announcement_bytes` concurrents.
- **Persistance** : `subscriptions.json` ecrit atomiquement a chaque subscribe/unsubscribe.

**Flow de publication :**
1. Curator signe sa `CuratorList` via `CuratorListEntry::sign()`
2. Serialise en JSON, stocke comme iroh blob via `BlobsClient::add_bytes()`
3. Annonce un `CuratorAnnouncement` (JSON avec `curator_pubkey_hex` + `blob_ticket`) sur le topic gossip per-curator : `BLAKE3("nexus-grid/curator/" || curator_pubkey)[..32]`
4. Les subscribers fetchen le blob via le `BlobTicket`, parsent, verifient la signature, et stockent dans le `DashMap`.

**Annonce gossip :** `CuratorAnnouncement { curator_pubkey_hex, blob_ticket }`. Le runtime verifie :
1. L'annonceur est dans l'attention set
2. Le blob est fetchable
3. La signature de l'entree est valide
4. L'attribution envelope-payload est consistante
5. La revision est strictement superieure

### 1.3 Quarantine queue

**Fichier :** `crates/nexus-coordinator-rs/src/quarantine_queue.rs`

SQLite-backed. Stocke les messages gossip borderline :
- `topic`, `sender_pubkey_hex`, `payload_json`
- `received_at`, `rate_strikes`, `pow_status`
- `flush_status` : `pending` | `flushed` | `dropped`

**Operations :** `add()`, `list_pending()`, `flush()`, `drop_entry()`, `flush_expired()`, `pending_count()`.

**GAP CRITIQUE :** La quarantine n'a PAS d'interface utilisateur. C'est un systeme backend-only. L'utilisateur ne sait pas qu'un message est en quarantaine, ne peut pas voir la queue, ne peut pas decider de flusher ou dropper. Le background sweep loop est meme "deferred to Tier 5 wire-up".

### 1.4 Capability toggles

**Fichier :** `crates/nexus-coordinator-rs/src/capability_store.rs`

6 capabilities, toutes off-by-default :
`biometric_gate`, `federation_canary`, `mcp_server_expose`, `rag_retrieval`, `streaming_bridge`, `tool_calling`.

TOML store avec hash SHA-256 d'integrite. Tamper = fallback all-off. Pas de lien avec la gouvernance de confiance.

### 1.5 Endorsement — mecanisme actuel

**IL N'EXISTE PAS de mecanisme d'endorsement signe dans le code.**

Ce qui existe :
- Un curator "vouches" pour un projet en l'incluant dans sa `CuratorList`. C'est un endorsement implicite (presence = approbation).
- Le spec `PUBLIC_FEED_SPEC.md` definit un type `CuratorVouched` comme "future operation (Sprint 2+)" mais il n'est PAS implemente. Le enum `PublicFeedOperation` n'a que `ReleasePublished` et `SourceBecameStale`.

**Ce qui manque :**
- Pas d'endorsement granulaire (scope : securite, qualite, licence)
- Pas de timestamp de quand le vouch a ete fait
- Pas de moyen de revoquer un vouch sans retirer le projet entier de la liste
- Pas de dissent visible (un curator ne peut pas dire "je desapprouve ce projet")

### 1.6 Revocation

**Fichier :** `crates/nexus-core-rs/src/key_rotation.rs`

**Ce qui EXISTE :**
- **Key rotation** : `KeyRotationAnnouncement` signee par l'ancienne cle. Transition window (7-90 jours). `RevocationCache` en memoire (HashMap).
- **Curator list verification avec revocation** : `verify_with_revocation(cache, now_ts)` rejette les cles revoquees.
- Gossip topic dedie : `"nexus-grid/key-rotation/v1"`

**Ce qui MANQUE :**
- **Revocation d'un endorsement individuel** : impossible sans re-publier la liste entiere sans le projet
- **Revocation d'une app** : pas de mecanisme pour marquer une app comme dangereuse/revoquee
- **Propagation de la revocation** : le `RevocationCache` est in-memory seulement (persistence SQLite differee S26). Un restart perd tout. Les rotations doivent etre re-recues via gossip.
- **Pas de "blocklist" signee** : un curator ne peut pas publier "je bloque ce projet" — il peut seulement retirer le projet de sa liste (absence = silence)

### 1.7 Warrant canary & FROST DKG

**Fichier :** `crates/nexus-shell-daemon-core/src/canary/`

Systeme complet : canary mensuel signe (Ed25519 ou FROST K-of-N), dead-man-switch 45 jours, duress ack quotidien. Topic gossip dedie. Format `CANARY.txt` verifiable hors-ligne. Pas directement lie a la gouvernance curator.

### 1.8 Frontend Curators page

**Fichier :** `web/src/pages/Curators.tsx`

Page simple :
- Input pour coller une cle publique Ed25519 (64 hex)
- Bouton "S'abonner"
- Liste des curators suivis avec :
  - Nom du curator
  - Pubkey tronquee
  - Nombre de projets vouches
  - Numero de revision
  - Statut : "En attente d'une premiere annonce gossip..." ou badges
- Bouton "Retirer" pour unsubscribe

**Ce qui MANQUE :**
- Aucune information de confiance (depuis quand ce curator existe, combien de followers)
- Pas de scope/specialite du curator
- Pas de dissent visible
- Pas de freshness indicator ("derniere mise a jour il y a 3 jours")
- Pas de lien vers les projets vouches depuis cette page

### 1.9 Browse page — filtrage par confiance

**Fichier :** `web/src/pages/Browse.tsx`

Netflix-style grid. Chaque `BrowseEntry` a :
- `curator_pubkey` / `curator_name` — quel curator a vouch
- `source` : `"curator"` ou `"direct"` (self-publish)
- `status` : `reachable` / `unreachable` / `unknown`
- Badges : "Verifie" (provenance_hash), "P2P" (archive_hash), "Auto-publie" (source=direct), "Source" (repo_url)

**Ce qui MANQUE :**
- Si 2 curators vouche pour le meme projet, on voit 2 entrees separees (pas d'agregation)
- Pas de "trust score" agrege
- Pas d'indicateur de dissent (si un curator approuve et un autre desapprouve)
- Pas d'indicateur de freshness de la verification
- Pas de filtre "montrer uniquement les apps approuvees par 2+ curators"

### 1.10 Public Feed — SourceBecameStale

La seule notion de "source perimee" existe dans le feed : `SourceBecameStale` avec reasons `repo_unreachable`, `commit_diverged`, `manual`. Mais :
- C'est un evenement dans le feed, pas un statut sur le Browse
- Pas de detection automatique (le coordinator devrait periodiquement re-verifier les repos)
- Pas de lien avec la freshness de l'endorsement du curator

---

## 2. Gaps de gouvernance — ce qui manque

### 2.1 Multi-curator subscription

**PARTIELLEMENT EXISTANT.** L'utilisateur peut s'abonner a N curators via le `CuratorRuntime.attention` DashMap. Chaque curator est independant. Les listes sont fusionnees dans le `BrowseAggregator.aggregate()`.

**GAP :** Pas de concept de "scope" ou "role" d'un curator. Tous les curators sont egaux. Pas de facon de dire "je fais confiance a ce curator pour la securite, et a celui-la pour la qualite".

### 2.2 Trust choice utilisateur

**EXISTANT MAIS BINAIRE.** L'utilisateur subscribe ou unsubscribe. Pas de granularite intermediaire (confiance partielle, confiance conditionnelle).

**GAP :** Pas de poids de confiance par curator. Pas de "je fais confiance a X mais seulement pour les apps de categorie Y".

### 2.3 Source perimee (stale source)

**PARTIELLEMENT EXISTANT.** Le feed a `SourceBecameStale` mais :
- Pas de detection automatique (cron/timer pour re-verifier les repos)
- Pas de surface dans le Browse (le statut stale n'est pas affiche)
- Pas de "freshness" pour les endorsements curator ("cette liste a ete mise a jour il y a 6 mois")

### 2.4 Freshness de la verification

**MANQUANT.** Aucun indicateur de quand un projet a ete verifie pour la derniere fois. Le `ProvenanceRecord` a un `timestamp` mais il n'est pas compare au temps actuel pour afficher "verifie il y a 3 jours" vs "verifie il y a 6 mois".

### 2.5 Quarantine visible

**MANQUANTE.** La quarantine est backend-only. L'utilisateur ne sait pas qu'un message est en quarantine. Pas d'UI, pas d'API pour lister les entrees quarantine.

### 2.6 Endorsement signe et verifiable

**MANQUANT.** L'endorsement actuel est implicite (presence dans la liste). Pas de structure signee qui dit "curator X endorse project Y pour la raison Z a la date T".

### 2.7 Revocation propagee et visible

**PARTIELLEMENT EXISTANT.** La key rotation se propage via gossip. Mais :
- Le `RevocationCache` est in-memory (perdu au restart)
- Pas de revocation d'endorsement individuel
- Pas de surface dans l'UI

### 2.8 Trust score agrege

**MANQUANT.** Pas de calcul de confiance agrege. Pas de "cette app est approuvee par 3 curators, dont 2 specialistes securite".

---

## 3. Modeles de gouvernance decentralisee — recherche externe

### 3.1 F-Droid : repos tiers et anti-features

F-Droid est le modele le plus proche de SBFB :
- **Repos tiers** : chaque repo a une cle de signature unique. Le client verifie automatiquement que l'utilisateur a ajoute le vrai repo. L'utilisateur peut ajouter N repos independants.
- **Anti-features** : systeme de tags negatifs (Tracking, Ads, NonFreeNet, etc.) affiches sur les fiches apps. L'utilisateur peut masquer les apps avec certains anti-features dans les settings.
- **Pas de moderation centralisee pour les repos tiers** : F-Droid n'est pas responsable des repos tiers. La confiance est a l'utilisateur.

**Lecon pour SBFB :** Le concept d'anti-features est directement applicable. Un curator devrait pouvoir non seulement lister les projets qu'il approuve, mais aussi signaler des "anti-features" sur des projets (telemetrie, pas open source, dependances proprietaires, etc.).

### 3.2 Certificate Transparency : logs publics verifiables

CT fournit un modele de **non-equivocation** :
- Chaque certificat emis est enregistre dans un log public append-only
- Les monitors scannent les logs pour detecter les certificats malveillants
- Le gossip entre participants detecte les "split-view attacks" (un log malveillant montrant des vues differentes a differents clients)

**Lecon pour SBFB :** Le public feed est deja un log append-only signe. La prochaine etape est d'ajouter des **monitors** — des noeuds qui verifient la consistance du feed et signalent les anomalies. Le mecanisme `verify_chain()` est le primitif ; il faut un runtime qui le tourne periodiquement et alerte.

### 3.3 Web of Trust (PGP) vs TOFU

- **WoT** : confiance transitive — "je fais confiance a A, A fait confiance a B, donc je fais partiellement confiance a B". Complexe, peu utilise en pratique.
- **TOFU** : Trust On First Use — "la premiere fois que je vois cette cle, je l'accepte. Si elle change, je m'alarme." Simple, largement deploye (SSH known_hosts).

**SBFB actuel = TOFU.** L'utilisateur colle une pubkey curator et fait confiance. Pas de verification de la premiere introduction. Pas de web of trust.

**Lecon pour SBFB :** Rester TOFU pour la simplicite, mais ajouter des indicateurs de confiance secondaires (age du curator, nombre de projets, coherence avec d'autres curators).

### 3.4 Scuttlebutt (SSB) : confiance sociale subjective

SSB est le modele P2P le plus pertinent :
- **Subjectivite** : chaque noeud a sa propre vue du reseau, basee sur ses follows/blocks
- **Follow graph** : le contenu est replique via les follows a 2-3 hops
- **Blocking public** : bloquer quelqu'un est une action publique dans le feed. Cela empeche aussi la replication du contenu de la personne bloquee via vos intermediaires.
- **Trustnet** : systeme de moderation qui calcule un score de confiance base sur les blocks 2nd/3rd party. Decision binaire block/not-block basee sur le consensus social.

**Lecon pour SBFB :** Le modele SSB de "blocking public" est directement applicable. Un curator devrait pouvoir publier une "disendorsement" signee dans le feed, visible par tous ses followers. Le Trustnet est un modele avance pour plus tard.

### 3.5 Mastodon/ActivityPub : blocklists et defederation

- **Instance-level governance** : chaque instance definit ses regles
- **Blocklists partagees** : les admins partagent des listes de serveurs bloques
- **Defederation** : une instance peut bloquer une autre, rendant son contenu invisible pour ses utilisateurs
- **Pas d'autorite superieure** : aucune entite centrale ne peut bannir une instance

**Lecon pour SBFB :** La defederation de Mastodon = un curator qui retire un projet de sa liste. Mais Mastodon a le concept de "blocklists publiques" que SBFB n'a pas encore.

### 3.6 Radicle : delegates et seuil de confiance

Radicle est le modele le plus structurellement proche de SBFB :
- **Delegates** : chaque repo a des delegates identifies par leur NodeID
- **Threshold canonique** : la branche canonique est determinee par un seuil de signatures parmi les delegates (ex: 2/3 doivent pousser le meme commit)
- **Seed nodes** : varient en politique de seeding (public, communautaire, selectif)
- **Self-certifying repos** : les signatures cryptographiques des delegates sont enregistrees comme "signed refs"

**Lecon pour SBFB :** Le modele de threshold de Radicle est directement applicable aux endorsements. "Une app est consideree safe quand 2/3 des curators de securite l'ont approuvee."

### 3.7 Google Key Transparency : non-equivocation

- **Verifiable map** : chaque utilisateur a une entree unique dans une structure de donnees verifiable
- **Append-only log** : chaque changement est enregistre
- **Gossip** : detecte si un serveur malveillant sert des vues differentes
- **Non-equivocation** : meme un serveur malveillant ne peut pas inserer/retirer des cles sans laisser une trace permanente

**Lecon pour SBFB :** Le feed SBFB est deja un append-only log signe. L'ajout de monitors qui verifient la consistance entre noeuds serait l'equivalent de la non-equivocation CT/KT.

### 3.8 Trustix (Nix) : attestation multi-builders

- **Chaque builder** a une paire de cles et log ses resultats dans un ledger signe append-only
- **M-of-N vote** : un binaire est considere de confiance quand M builders sur N ont produit le meme hash de sortie
- **Detection automatique** des builders malveillants qui produisent des hashes differents

**Lecon pour SBFB :** Le `BuildQuorumReached` dans la spec du feed est exactement ce pattern. Quand on l'implementera, on pourra dire "cette app a ete buildee independamment par 3 noeuds et ils ont tous le meme hash".

### 3.9 TUF (The Update Framework) : delegation hierarchique

- **Roles hierarchiques** : root, targets, snapshot, timestamp
- **Delegation** : le role targets peut deleguer la confiance a d'autres roles (sous-ensembles de targets)
- **Threshold signatures** : chaque role requiert M-of-N signatures
- **Multi-level delegation** : les roles delegues peuvent eux-memes deleguer

**Lecon pour SBFB :** TUF est trop hierarchique pour SBFB (pas de "root role"). Mais le concept de delegation avec threshold est pertinent : un "meta-curator" qui regroupe les endorsements de N curators individuels.

---

## 4. Modele de confiance pluraliste propose

### 4.1 Multi-curator avec scope

**Proposal :** Chaque curator a un `scope` optionnel dans sa `CuratorList` :

```rust
pub struct CuratorList {
    // ... existant ...
    pub scope: Option<CuratorScope>,  // NOUVEAU
}

pub enum CuratorScope {
    General,           // pas de specialite
    Security,          // specialiste securite
    Quality,           // qualite du code / UX
    License,           // conformite licence FLOSS
    Accessibility,     // accessibilite
    // extensible via string tag pour post-v1.0
}
```

L'utilisateur voit dans le Browse : "Approuve par 2 curators (1x securite, 1x general)".

### 4.2 Endorsement signe dans le feed

**Proposal :** Nouveau type d'operation feed `CuratorVouched` :

```rust
pub enum PublicFeedOperation {
    ReleasePublished(ReleasePublishedPayload),
    SourceBecameStale(SourceBecameStalePayload),
    CuratorVouched(CuratorVouchedPayload),        // NOUVEAU
    CuratorDisendorsed(CuratorDisendorsedPayload), // NOUVEAU
}

pub struct CuratorVouchedPayload {
    pub project_id: String,        // hex-64
    pub curator_pubkey: String,    // hex-64
    pub scope: String,             // "security", "quality", etc.
    pub comment: Option<String>,   // max 280 chars
}

pub struct CuratorDisendorsedPayload {
    pub project_id: String,
    pub curator_pubkey: String,
    pub reason: String,            // "vulnerability_found", "license_violation", etc.
    pub comment: Option<String>,
}
```

Chaque endorsement/disendorsement est signe par l'auteur du feed entry (qui doit etre le curator lui-meme). Verifiable, datable, irrevocable (append-only).

### 4.3 Agregation de confiance

**Modele de decision pour l'utilisateur :**

1. L'utilisateur subscribe a N curators
2. Pour chaque app dans le Browse, on calcule :
   - `endorsements` : nombre de curators qui ont vouch (via liste OU feed entry CuratorVouched)
   - `disendorsements` : nombre de curators qui ont disendorse
   - `net_trust = endorsements - disendorsements`
   - `scope_breakdown` : { security: 1, quality: 2, general: 0 }
3. L'UI affiche :
   - Badge vert "3 curators" si `net_trust >= 2`
   - Badge jaune "1 curator, 1 objection" si dissent
   - Badge rouge "Quarantine" si `disendorsements > endorsements`
   - Pas de badge si aucun curator connu n'a d'avis

**PAS de trust score numerique.** On montre les faits (qui dit quoi), pas un nombre opaque. L'utilisateur decide.

### 4.4 Dissent visible

Quand curator A approuve et curator B disendorse la meme app :
- Le Browse montre les deux avis cote a cote
- Un badge "Avis partage" avec tooltip : "Approuve par X (securite), Objecte par Y (qualite)"
- L'utilisateur peut cliquer pour voir le detail (commentaires, dates)

### 4.5 Freshness

Chaque endorsement a un timestamp (via le feed entry ou le `CuratorList.created_at`). L'UI affiche :
- "Verifie il y a 3 jours" (vert)
- "Verifie il y a 3 mois" (jaune)
- "Verifie il y a plus d'un an" (gris, warning)

Un curator qui n'a pas mis a jour sa liste depuis > 90 jours est marque "inactive" dans la page Curators.

### 4.6 Stale detection

Le coordinator devrait periodiquement re-verifier les repos source :
1. Cloner le repo (git clone --depth 1)
2. Comparer le commit_sha du dernier deploy avec la HEAD du repo
3. Si diverge, emettre un `SourceBecameStale { reason: "commit_diverged" }`
4. Si repo unreachable, emettre `SourceBecameStale { reason: "repo_unreachable" }`

Le Browse affiche un badge "Source perimee" sur les apps avec un SourceBecameStale recent sans SourceRecovered subsequent.

### 4.7 Revocation propagation

**Existant :** La key rotation se propage via gossip. Le `RevocationCache` est in-memory.

**Necessaire pour S67 :**
- Persister le `RevocationCache` en SQLite (carry item S26)
- Ajouter un `CuratorDisendorsed` dans le feed (propagation signee)
- L'UI Curators affiche un warning si un curator suivi a une cle en transition

---

## 5. Plan de phases S67

### Phase A : Endorsement signe + CuratorVouched dans le feed

**Scope :**
1. Ajouter `CuratorVouched` et `CuratorDisendorsed` au enum `PublicFeedOperation`
2. Domain de validation pour les deux nouveaux types
3. Tests unitaires : insert, replay, verify_chain avec les nouveaux types
4. `DOMAIN_FEED_V1` couvre deja les nouveaux types (meme domain tag)

**Complexite :** Moyenne. Le feed est deja extensible par tagged union serde.

**Pre-requis :** Aucun (les primitifs existent).

**Tests :** ~8-10 nouveaux tests (validation, serde, adversarial forgery).

### Phase B : Multi-curator trust overlay + scope

**Scope :**
1. Ajouter `scope: Option<String>` a `CuratorList` avec `#[serde(default)]`
2. Propagate le scope dans `BrowseEntry` : `endorsement_scopes: Vec<String>`
3. Ajouter dans `BrowseAggregator.aggregate()` la logique de multi-curator overlay :
   - Pour chaque `project_id`, agreger les endorsements de tous les curators
   - Calculer le breakdown par scope
4. Endpoint daemon `/api/daemon/browse` retourne les donnees d'agregation

**Complexite :** Moyenne. Le `BrowseAggregator` fait deja la fusion des listes. Il faut ajouter la dimension scope.

**Pre-requis :** Phase A (les CuratorVouched entries sont la source de verite pour les endorsements signes).

**Tests :** ~6-8 nouveaux tests (agregation multi-curator, scope breakdown, dedup).

### Phase C : UX confiance visible (badges, timeline, dissent)

**Scope :**
1. **Page Curators** : ajouter freshness ("derniere mise a jour il y a X"), scope du curator, badge "inactif" si > 90 jours
2. **Page Browse** : badges de confiance agrege :
   - Nombre de curators qui endorsent
   - Breakdown par scope
   - Indicateur de dissent si CuratorDisendorsed existe
   - Freshness de la derniere verification
3. **Page BrowsedProject** : detail de confiance dans le dialog verification :
   - Timeline des endorsements/disendorsements
   - Commentaires des curators
   - Freshness de chaque endorsement

**Complexite :** Moyenne-haute (beaucoup de composants UI).

**Pre-requis :** Phase B (les donnees d'agregation doivent etre dans le BrowseEntry).

**Tests :** ~10-15 Vitest (composants, formatage, rendu conditionnel).

### Phase D : Stale detection + freshness indicators

**Scope :**
1. Timer coordinator qui re-verifie periodiquement les repos source des apps deployees
2. Emission automatique de `SourceBecameStale` quand un repo est unreachable ou diverge
3. Ajout de `SourceRecovered` au feed (quand un repo stale redevient accessible)
4. Surface dans le Browse : badge "Source perimee" avec date et raison
5. Freshness des endorsements : calcul de l'age depuis le dernier CuratorVouched ou CuratorList.created_at

**Complexite :** Moyenne. Le mecanisme d'emission est simple (timer + git clone + compare). La surface UI est dans Phase C.

**Pre-requis :** Phase A (pour SourceRecovered). Phase C (pour l'affichage).

**Tests :** ~8 tests (stale detection, recovery, freshness calcul).

### Phase E : Tests adversariaux gouvernance

**Scope :**
1. **Curator malveillant** : un curator signe un CuratorVouched pour un projet qu'il ne connait pas, puis un CuratorDisendorsed — les deux sont dans le feed, le verify_chain passe, mais l'UI montre le dissent.
2. **Split-brain** : deux curators avec le meme nom mais des cles differentes — l'UI doit les montrer comme deux curators distincts.
3. **Stale replay** : un attaquant rejoue un ancien CuratorVouched — le timestamp doit etre ancien, l'UI montre "verifie il y a 2 ans".
4. **Forgery CuratorDisendorsed** : un attaquant signe un disendorsement avec la cle d'un autre curator — verify_entry rejette.
5. **Flood disendorsement** : un curator malveillant spam des disendorsements — le rate limiter du feed (5/min) limite l'impact.

**Complexite :** Moyenne. Patterns adversariaux deja etablis par S64.

**Pre-requis :** Phases A-D.

**Tests :** ~8-10 tests adversariaux.

---

## 6. Risques et pitfalls

### 6.1 Scope creep

Le modele de confiance pluraliste est un sujet immense. S67 doit se limiter a :
- Endorsement signe dans le feed
- Aggregation basique multi-curator
- Surface UI pour la visibilite

Il ne faut PAS essayer d'implementer : Web of Trust, trust scoring numerique, delegation hierarchique TUF, Trustnet SSB. Ces concepts sont pour post-S67.

### 6.2 Backward compatibility

L'ajout de `scope` a `CuratorList` et de nouveaux types au feed doit utiliser `#[serde(default)]` pour ne pas casser les noeuds existants. Un noeud pre-S67 doit pouvoir deserialiser une CuratorList avec scope sans erreur.

### 6.3 Performance d'agregation

Si un utilisateur suit 10 curators avec 256 projets chacun, l'agregation multi-curator dans `BrowseAggregator.aggregate()` traite 2560 entrees. C'est genable mais pas critique. Attention si on ajoute des lookups dans le feed pour chaque projet.

### 6.4 UX overload

Trop d'indicateurs de confiance = confusion. La Phase C doit etre soigneusement designee pour montrer l'information essentielle (combien de curators, dissent oui/non) sans noyer l'utilisateur.

### 6.5 RevocationCache persistence

Le carry item P2-REVOCATION-CACHE-PERSIST (depuis S26) doit etre resolu AVANT ou PENDANT S67 pour que la revocation fonctionne apres un restart du daemon.

---

## 7. Decisions recommandees

### 7.1 Endorsement via feed, pas via CuratorList

**Decision :** Les endorsements granulaires (avec scope, commentaire, date) vont dans le feed comme `CuratorVouched` entries. La `CuratorList` reste le mecanisme de "voila les projets que je recommande" sans granularite.

**Rationale :** Le feed est deja un log append-only signe avec hash-chain. Y ajouter des types est trivial (tagged union serde). La CuratorList est optimisee pour le Browse rapide (une liste complete), pas pour l'historique.

### 7.2 Pas de trust score numerique

**Decision :** Montrer les faits (qui endorse, qui objecte, quand) et laisser l'utilisateur decider. Pas de nombre magique.

**Rationale :** Un trust score cree une fausse precision et une surface de gaming. Le modele SSB "subjectif" est plus sain : l'utilisateur voit qui dit quoi et fait son propre jugement.

### 7.3 Scope comme string libre

**Decision :** Le scope est un `String` libre, pas un enum ferme. Convention documentee ("security", "quality", "license"), mais extensible.

**Rationale :** Un enum ferme necessite un consensus sur les categories et un upgrade coordonne. Un string libre permet a chaque communaute de definir ses propres scopes.

### 7.4 Disendorsement = action publique

**Decision :** Un `CuratorDisendorsed` est une action publique dans le feed, visible par tous. Pas de disendorsement prive.

**Rationale :** Modele SSB : le blocking est public. La transparence est essentielle pour la confiance. Un disendorsement cache serait une moderation opaque, contraire a la philosophie SBFB.

---

## 8. Sources

### Code analyse
- `crates/nexus-core-rs/src/curator.rs` — CuratorList struct, signing, validation, 21 tests
- `crates/nexus-core-rs/src/key_rotation.rs` — KeyRotationAnnouncement, RevocationCache, 25 tests
- `crates/nexus-coordinator-rs/src/public_feed.rs` — FeedEntry, PublicFeedOperation, verify_chain, 30+ tests
- `crates/nexus-coordinator-rs/src/quarantine_queue.rs` — QuarantineQueue, 5 tests
- `crates/nexus-coordinator-rs/src/capability_store.rs` — CapabilityStore, 5 tests
- `crates/nexus-shell-daemon-core/src/iroh_runtime.rs` — CuratorRuntime, attention set, gossip processing
- `crates/nexus-shell-daemon-core/src/browse.rs` — BrowseAggregator, BrowseEntry, probe cache
- `web/src/pages/Curators.tsx` — Frontend curator management
- `web/src/pages/Browse.tsx` — Frontend app browser
- `web/src/api/daemon.ts` — Daemon API schemas (Zod)
- `docs/protocol/PUBLIC_FEED_SPEC.md` — Feed spec with future CuratorVouched type

### Modeles de gouvernance (recherche web)
- F-Droid Security Model : https://f-droid.org/docs/Security_Model/
- F-Droid Anti-Features : https://f-droid.org/docs/Anti-Features/
- Certificate Transparency (InfoQ) : https://www.infoq.com/articles/tls-certificate-transparency/
- CT Gossip Protocols (ResearchGate) : https://www.researchgate.net/publication/283531637
- Secure Scuttlebutt Protocol Guide : https://ssbc.github.io/scuttlebutt-protocol-guide/
- SSB (Wikipedia) : https://en.wikipedia.org/wiki/Secure_Scuttlebutt
- TUF Specification : https://theupdateframework.github.io/specification/latest/
- TUF Roles and Metadata : https://theupdateframework.io/docs/metadata/
- Trustix (Nix community) : https://nix-community.github.io/trustix/
- Trustix NLnet : https://nlnet.nl/project/Trustix-Nix/
- Radicle Protocol Guide : https://radicle.dev/guides/protocol
- Google Key Transparency Design Doc : https://github.com/google/keytransparency/blob/master/docs/design.md
- Mastodon Decentralized Moderation (ACM) : https://dl.acm.org/doi/fullHtml/10.1145/3614419.3644016
- Mastodon Defederation (Carnegie) : https://carnegieendowment.org/research/2025/03/fediverse-social-media-internet-defederation
- Key Transparency at Meta : https://engineering.fb.com/2025/11/20/security/key-transparency-comes-to-messenger/
