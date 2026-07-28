# Preflight G8 — Sprint 83 Phase M (artefacts, ACL et provenance)

## Verdict

**Verdict : PLAN-ADAPT**

Le plan ne contredit aucune décision Day-0 figée (la « visibilité 2 états public/privé »
de `CLAUDE.md:612` porte sur les **projets publiés**, pas sur les artefacts de run —
objets disjoints, `publish_api.rs:87-89`, aucun `enum Visibility` en Rust). Mais quatre
livrables de M.2 sont **irréalisables tels qu'écrits**, chacun sur mesure de première main :
(1) le remède annoncé pour `nexus-trace-core` (« JCS garanti ») **régresserait** la
signature — `timestamp_unix_ns` est un `u64` en nanosecondes ~1,78e18, soit 198× au-dessus
de 2^53, et l'engine ne porte PAS la garde I-JSON que `sbfb-workflow-core` porte
(`hash.rs:357` vs `pack.rs:142`+`:505-521`) ; (2) l'invariant cardinal « digest ≠ droit de
lecture » ne peut PAS être tenu par une ACL de façade, car `/blob-serve` est monté dans
`public_routes` (`http.rs:272`) et `BlobsProtocol::new(&blobs_store, None)`
(`node.rs:442`) sert tout blob à tout dialeur ; (3) le crate cible ne peut pas SIGNER
(charte keyless, `activity.rs:36-37`, aucune dep de clé) ; (4) `M.4` ne lance jamais le
crate que la row 25 commande, et le filtre de la row 26 ne matche rien (mesuré :
`RUN_EXIT=4`). Le mot « audience » est de plus **déjà pris** dans le même crate avec un
autre sens (`capability.rs:543`, classe de menace `T-CAP-GRANT-AUDIENCE-CONFUSION`).

Aucun de ces écarts ne touche une décision figée ; chacun a un chemin corrigé concret
appuyé sur du code en dépôt. D'où PLAN-ADAPT et non DESIGN-CONFLICT.

## Methode et convention de preuve

Six scans (S1a prior-art OSS, S1b dépendances, S2 décisions historiques, S3 threat model,
S4 wire format, S5 fidélité doc↔code), chacun accompagné de sa réfutation adversariale.
Cette synthèse **n'arbitre au milieu d'aucune contradiction** : chaque point litigieux a
été re-mesuré ici, commande et sortie citées.

- **[M]** mesuré de première main dans cette synthèse.
- **[I]** inféré, avec le raisonnement explicite.
- **[B]** repris d'un document sans re-dérivation (marqué comme tel, jamais load-bearing).

**HEAD réel : `22a34082`**, et non `709ff4e2` comme annoncé par le mandat et supposé par
les **six** scans. [M] `git rev-parse HEAD` → `22a34082022de0cb4f800dbf5949cb3fb2ad6c6f` ;
`git log --oneline -2` → `22a34082 chore(docs): versionner le manuscrit nexus-v1 …` /
`709ff4e2 fix(keystore): …`. Le delta est docs-only ; **aucune ancre de code des scans
n'est invalidée** par cet écart, mais le commit de M doit citer `22a34082`.

Arbre de travail vérifié propre en fin de synthèse : [M] `git status --short
--untracked-files=all` → sortie vide. Une sonde temporaire
(`crates/nexus-trace-core/tests/zz_synth_probe.rs`) a été écrite, exécutée, puis
**supprimée** ; sa commande et sa sortie intégrale sont reproduites ci-dessous.

**Mesure de référence de cette synthèse** (sonde liant le VRAI crate :
`SignedCanaryProcessor::process` → JSONL → `serde_json::from_str` → `verify_signed_event`,
200 essais par palier, timestamp ns réaliste `1_785_000_000_123_456_789`) :

```
PROBE n=0: jsonl-roundtrip verify FAIL   0/200 | bytes-differ-after-roundtrip   0/200
PROBE n=1: jsonl-roundtrip verify FAIL   0/200 | bytes-differ-after-roundtrip   0/200
PROBE n=2: jsonl-roundtrip verify FAIL  86/200 | bytes-differ-after-roundtrip  87/200
PROBE n=3: jsonl-roundtrip verify FAIL 142/200 | bytes-differ-after-roundtrip 150/200
PROBE n=6: jsonl-roundtrip verify FAIL 197/200 | bytes-differ-after-roundtrip 198/200
PROBE n=8: jsonl-roundtrip verify FAIL 200/200 | bytes-differ-after-roundtrip 200/200
PROBE serde_json = {"trace_id":"aaaa…","span_id":"bbbb…",
  "timestamp_unix_ns":1785000000123456789,"name":"probe","service_name":"svc",
  "attributes":{"k0":0}}
```

## S1a — Prior art OSS

**Ce qui tient.** L'état de l'art sur « digest ≠ droit de lecture » est la séparation
**verify-cap / read-cap** de Tahoe-LAFS : le digest sert à vérifier l'intégrité et à lier
la provenance, jamais à déchiffrer ; la clé vit dans une capability distincte, et
l'identifiant de stockage est **dérivé** de la clé de lecture (sens unique). C'est
exactement ce qui réconcilie les deux exigences de M.1 (« content-addressed » ET « digest
≠ lecture »), qui se contredisent sans elle. **[B]** — documentation Tahoe-LAFS, non
re-dérivée ici ; l'argument de conception qui en découle est **[I]** et il est corroboré
en dépôt par le patron `SecretRef` (référence opaque qui NOMME sans porter la valeur,
`secret.rs:260-276`).

À **LAISSER** : le modèle Perkeep `ShareHaveRef` (« qui a la référence a l'accès ») —
c'est l'anti-invariant littéral de G-ARTIFACT-ACL **[B]** ; et l'idée de chercher un
modèle d'ACL chez Bazel REAPI, dont le proto est muet **[B]**.

**Ce qui tombe.** Le scan présentait « `verify_signed_event` échoue **dès** 2 attributs,
**y compris dans le même processus** » avec 6 exécutions à l'appui. **Les deux moitiés sont
fausses**, et le scan avait mesuré une **réplique** au lieu du crate. [M] Ma sonde sur le
vrai crate : 0/200 à n=0 et n=1, **86/200 (43 %)** à n=2, 200/200 à n=8. Le chemin sans
frontière serde vérifie toujours, parce que `sign_event` stocke `event.clone()`
(`signed.rs:51-53`) et que `HashMap::clone` préserve l'ordre d'itération. La conséquence
est opérationnelle : la prescription du scan (« test à ≥2 attributs ») produirait un bras
rouge **flaky à ~43 %** — un tirage, pas un gate.

**Constat négatif utile, confirmé.** `serde_jcs 0.2` est déjà dep de workspace
([M] `Cargo.toml:67`, déclaré par 4 crates), conforme RFC 8785, et **aucune migration de
canonicaliseur n'est justifiée**. La réputation d'abandon citée par des sources tierces est
périmée **[B]**.

**Porte durcie existante, non réutilisable en l'état.** [M] `grep -n 'fn
parse_rejecting_duplicate_keys' crates/sbfb-workflow-core/src/pack.rs` → `742:fn …`
(**privée**, non ré-exportée). `sbfb-workflow-engine` ne peut donc pas l'appeler
aujourd'hui, alors que M y écrit du parsing d'octets de provenance.

## S1b — Dependances et graphe transitif

**Verdict : la Phase M est faisable à 0 dep runtime neuve et 0 nœud de lock neuf.**

[M] Primitives déjà présentes et résolues : `serde_jcs` (`Cargo.toml:67`) ; `blake3`,
`ed25519-dalek`, `argon2`, `aes-gcm`, `secrecy`, `zeroize`, `keyring` — tous
`{ workspace = true }` et deps **directes** de `nexus-core-rs`
(`crates/nexus-core-rs/Cargo.toml:110-114`), déjà exploités par `vault.rs` en L1.

[M] **La façade blob coûte zéro** : `crates/nexus-shell-daemon/Cargo.toml:38` déclare
déjà `sbfb-workflow-engine = { path = … }` et `:87` `iroh-blobs` ; `BlobsClient`
(`blobs.rs:67`) expose `add_bytes`/`get_bytes`/`set_tag`/`delete_tag`. L'hôte a tout ;
l'engine n'a qu'à déclarer un trait.

[M] **L'engine n'a aucune source d'entropie** — deps exactes vérifiées :
`sbfb-workflow-core`, `serde`, `serde_json`, `serde_jcs`, `blake3`, `rusqlite`,
`rusqlite_migration`, `thiserror`, `zeroize` (+ `libc`/`windows` target-gated). Aucun
`rand`/`getrandom`, aucun `ed25519-dalek` direct, aucun `iroh-blobs`. AES-GCM exige un
nonce frais ⇒ **le scellement doit vivre derrière le trait hôte**.

**Correction due au scan.** Sa prescription centrale (« une ligne `serde_jcs` +
`serde_jcs::to_vec` à `signed.rs:60` ») est **insuffisante et régressive** : RFC 8785 fait
passer les nombres par un double IEEE-754. [M] `timestamp_unix_ns` est un `u64` en
**nanosecondes** (`lib.rs:37`), valeur mesurée `1785000000123456789` — 198× au-dessus de
2^53. Le dépôt **connaît déjà ce piège et l'a fermé ailleurs** : [M] `pack.rs:142`
`const MAX_SAFE_INTEGER: i64 = 9_007_199_254_740_991;` et `pack.rs:505-521` (`reject_nulls`
refuse « integer at {path} outside the I-JSON exact range collapses under JCS »), avec la
rationale mesurée `pack.rs:496-504` (« `serde_jcs` serialises 9007199254740993 as
9007199254740992 … une collision silencieuse dans une pré-image de signature ») et le test
discriminant `pack.rs:2080-2108`.

**Asymétrie décisive, sur le chemin exact de M** — angle manqué par le scan :
[M] `sed -n '357,376p' crates/sbfb-workflow-engine/src/hash.rs` montre que le
`reject_nulls` de l'ENGINE ne traite que `Null`, `Array`, `Object`, avec `_ => Ok(())`
pour `Number`. **Aucune garde `MAX_SAFE_INTEGER`, aucune mention I-JSON dans tout
`hash.rs`.** Or `canonical_bytes` (`hash.rs:343`) → `serde_jcs::to_vec` (`hash.rs:347`) est
la porte canonique de l'engine, et M.2 y place `ArtifactRef` + receipts, avec G-PROVENANCE
exigeant « mutation de chaque digest rejetée ». Un digest d'artefact portant une taille ou
un timestamp ns > 2^53 serait **collisionnable par la porte canonique de l'engine**.

**CI supply-chain.** Le carry CI-1 est mesurément **inversé** au HEAD local : `licenses ok`,
`advisories FAILED` (RUSTSEC-2023-0071 `rsa`, `enumset` yanked, via la feature optionnelle
`tor`) — et le gate local **n'est discriminant que sous `--all-features`**. À porter dans
la procédure de la phase. La lecture des runs GitHub cités par le scan n'a **pas** été
re-jouée ici : **[B]**, à traiter comme non établi.

## S2 — Decisions historiques traversees

**Aucun DESIGN-CONFLICT.** La provenance SLSA existante (S14 → port Rust S42) atteste une
**release d'app**, pas un run. [M] `DOMAIN_PROVENANCE_V1` est déclaré `canonical.rs:128` et
a **deux producteurs vivants** : `nexus-coordinator-rs/src/provenance.rs:11,119-120` et
`sbfb-factory/src/gates.rs:6,648-649`. M ne doit ni le réutiliser ni le renommer — le
réutiliser serait exactement la confusion de rôles que §17.1 interdit.

**Census gelé à 29, et les 8 rôles ne demandent pas 8 constantes neuves.** [M] Recompte à la
source par la méthode du gate → **29** ; `bash scripts/check-frontier-contracts.sh` →
EXIT 0, « DOMAIN census [29 frozen] ». [M] Trois tags sont **RÉSERVÉS 0-producteur** et
consommables tels quels : `DOMAIN_COMPILED_PLAN_V1` (`canonical.rs:395`),
`DOMAIN_RUN_EVENT_V1` (`:404`), `DOMAIN_PACK_INSTALL_RECEIPT_V1` (`:414`), tous documentés
« This constant alone confers NO acceptance ». Le rôle 1 est rempli
(`DOMAIN_WORKFLOW_PACK_V1`, `:382`), le rôle 7 depuis S7 (`:95`/`:98`/`:101`), le rôle 6 est
**routé Phase O** (`THREAT_MODEL.md:1836`), le rôle 8-humain est déjà tranché non-famille
(`hitl_approval.rs`, contexte local `sbfb-hitl-approval-v1`), le rôle 8-eval appartient à
`G-EVAL` **Phase W** (`gate_matrix.md:61`).

**Angle manqué dominant, que je porte au scope : la Phase L.** Aucun des scans classiques
n'a ouvert `capability.rs`, la phase de feature immédiatement précédente, dans le crate
même que M vise. [M] `capability.rs:211` `pub const ALL: [CapabilityClass; 17]` et `:543`
`pub enum GrantAudience { Release, Pack }`, avec le doc `:539-541` : « A release-audience
grant NEVER satisfies the pack-version layer and vice versa (T-CAP-GRANT-AUDIENCE-
CONFUSION) ». Le mot « audience » que M.1 annonce comme livrable **existe déjà dans le même
crate avec un autre sens, et il porte une classe de menace nommée**. Cinq des variantes de
`CapabilityClass` recouvrent de plus les 4 classes de M (`ReadPublic`, `ReadPrivateScoped`,
`SecretUse`, `NetworkFixedDestination`, `NetworkOpen`).

**Ce qui tombe.** Le scan affirmait un « patron canon sur 8 surfaces » pour
`roles_not_interchangeable`. [M] `grep -rn 'fn cross_domain_signature_rejected'
--include=*.rs crates/` → **2 hits seulement** (`shard_plan.rs:874`, `pack.rs:1767`) ; les
5 autres ancres portent des noms différents et **plusieurs ne prouvent que la moitié
nécessaire** (`assert_ne!` sur les octets, aucune vérification de signature). Un
implémenteur qui « réutilise le patron » depuis cette liste peut livrer un test qui ne
vérifie aucune signature. Le seul patron complet (nécessaire + suffisant + attente lue dans
un corpus) est `pack.rs:1767-1810`.

Tombe aussi : la « sonde temporaire non committée » signalée dans l'arbre par le scan
n'existe pas au HEAD réel — [M] `ls crates/nexus-trace-core/tests/` → « No such file or
directory », et `git status` est vide.

## S3 — Threat model (couverture prospective + fidelite retrospective)

**Prospectif — aucune section existante ne modélise la confidentialité d'un blob.** [M] Les
sections §19.x du sprint couvrent run store, effets, capacités, coffre ; la dernière
sous-section de menace est `### 19.4` et elle est suivie de `## 20`. Les surfaces suivantes
n'ont **aucune** entrée : digest devinable/énumération, confused deputy sur la façade blob,
rejeu d'un receipt d'artefact, substitution d'un nœud du graphe, fuite par métadonnée
(taille/audience/timing), **fuite par le lineage lui-même** (le graphe révèle l'existence
d'artefacts privés), suppression non effective, rétention contournée. M doit **ouvrir une
§19.5**, pas étendre.

**L'invariant cardinal est aujourd'hui contredit au fil et à la route.** [M]
`crates/nexus-shell-daemon/src/http.rs:272` : `// Public routes: no bearer, no Host check,
no Origin check.` suivi de `.nest("/blob-serve", blob_serve_routes)`. [M] La doc de
`build_router` (`:244-256`) porte l'hypothèse porteuse, à citer telle quelle dans le body :
« The blob content is **already public by construction** (anyone on the P2P network can
fetch the zip by hash), so exempting the route does not leak anything new. » [M] Au niveau
P2P : `node.rs:442` `.accept(BLOBS_ALPN, BlobsProtocol::new(&blobs_store, None))` — le
`None` résout sur `EventSender::DEFAULT`, dont le masque est `get: RequestMode::None` ⇒
aucune interception, aucun `AbortReason` possible. **Une ACL de façade en process ne peut
pas fermer G-ARTIFACT-ACL** : le chemin réseau la court-circuite.

Correction due au scan : sa « borne honnête » (« le daemon écoute en loopback durci ») est
**fausse pour cette route précise** — aucune des trois défenses ne s'y applique. Et son
ancre `THREAT_MODEL.md:262-265` (« encryption at rest différé Sprint 17+ ») est à la fois un
**non-sequitur** (elle traite du stockage de clés, pas de l'autorisation de lecture) et
**périmée** (le coffre AES-256-GCM a atterri en L1).

**Rétrospectif.** [M] `blobs.rs:130-133` : « Once a blob carries no tags the store MAY
garbage-collect it — **though no GC is scheduled today** … it does not free disk now
(Sprint 74 Phase D; the reaper is post-launch) ». Une claim « suppression » au sens octets
serait donc **fausse au HEAD**. Trois copies survivent à un `delete` : le blob (aucun
reaper), le cache LRU en clair de blob-serve (32 archives, aucune API d'invalidation), les
copies distantes chez tout pair ayant déjà tiré le hash.

**Verrou machine à laisser rougir.** [M] `secret.rs:634-636` : `assert_eq!(surfaces.len(),
5, "a surface was dropped from the canary sweep")`, et `THREAT_MODEL.md:1960` affirme « 16
items » pour §14.3 — liste dont **`ArtifactRef` est l'item 14**. Dès qu'`ArtifactRef`
existe, le canari doit passer à 6 surfaces, l'assert rougit, et THREAT_MODEL doit être
corrigé **dans le même commit**. C'est le seul verrou automatique du dépôt contre la dérive
doc de cette famille : ne pas le contourner.

**Carry qui borne toute claim « chiffré ».** [M] `.planning/active/sprint83_audit_plan.md:120-132`
(KEYSTORE-LAYER2-INERT) : `keyring 3.6` est déclaré **sans feature de backend**, donc
`keyring::default` résout vers le builder **MOCK**. La seule protection at-rest réelle est
Argon2id(PIN). Reprendre la formulation honnête déjà écrite à `vault.rs:22-31`.

## S4 — Wire format et frontieres

**0 bump atteignable et canoniquement légitime.** [M] `CLAUDE.md:682-689` (« `*_ANNOUNCEMENT_VERSION`
restent à 1 … Un sprint qui change le canonical **redéfinit la v1 courante** ») et `:695-697`
(« Tests “legacy decode” … **à supprimer immédiatement** »). [M] `sprint83_plan.md:29-30`
(AM-12) : « évolutions Task/Result/ArtifactRef **ADDITIVES par défaut** ». `ArtifactRef` est
un type **neuf** : [M] `grep -rn 'ArtifactRef|artifact_ref' --include=*.rs --include=*.ts
crates web/src` → **0 hit**.

**Recette canonique à respecter.** [M] `canonical.rs:435-442` : `serde_jcs::to_vec` puis
`domain || 0x00 || body`. Digest = blake3 hex **minuscule** strict (rationale `pack.rs:289`,
« hex has two spellings per digest »). Précédent de digest content-adressé **sans** famille
DOMAIN : `pack.rs:1345` `digest_for` construit `sbfb-pack-digest-<field>-v1` — 6 digests
domaine-séparés **sans toucher le census**. Précédent de contexte local : `hash.rs:26-36`
(« not a signing domain, never gossiped, so it deliberately does not follow the
`DOMAIN_*_V1` naming »), séparateur `0x1f`.

**Interdit dur.** [M] `Cargo.lock` : `serde_json 1.0.149` résout avec `indexmap` ⇒
`preserve_order` est unifié ON. Aucun digest, aucune arête de graphe, aucun receipt de M ne
doit passer par `serde_json::to_vec` — la rationale est déjà écrite dans
`crates/sbfb-workflow-engine/Cargo.toml:19-24`.

**Frontières neuves — c'est à ce scan de les nommer, le gate ne le fera pas.** [M] Le gate
ne boucle que sur les lignes portant déjà `// FRONTIER: `, et la couverture est bien de
**2 types tagués** (`shard_plan.rs:188`, `pack.rs:391`) — chiffre re-vérifié, non recopié,
corroboré par la sortie « frontier-tag coverage [2 tagged] ». Poser une étiquette machine
exige **trois** conditions (un `DOMAIN`, un `*_VERSION` résolvable, et un `schema_for!` ou
une exemption `// FRONTIER-NO-SCHEMA:`) — donc incompatible avec la posture 0-DOMAIN
recommandée.

**Réconciliation de gates, mesurée.** [M] `gate_matrix.md:59` (row 25) commande
`cargo nextest run -p sbfb-workflow-core provenance` ; `plan.md:1049-1050` (M.4) ne lance que
`-p sbfb-workflow-engine` + `-p nexus-trace-core`. [M] `gate_matrix.md:60` (row 26) commande
`cargo nextest run -p sbfb-workflow-engine artifact_acl`, filtre qui ne matche **aucun** des
noms de test annoncés en M.3. Mesure : `cargo nextest run -p sbfb-workflow-engine
artifact_acl --offline` → `Summary [0.001s] 0 tests run: 0 passed, 160 skipped` +
`error: no tests to run`, **`RUN_EXIT=4`**. **Ce n'est donc PAS un gate qui passe à vide**
(S5 réfuté sur ce point, S4 confirmé) : il échoue bruyamment.

**Contrainte d'implémentation du remède, mesurée.** Le filtre positionnel de nextest matche
les **modules de lib**, pas les noms de binaires d'intégration : [M] `cargo nextest list -p
sbfb-workflow-engine secret` → 4 lignes `secret::tests::…` ; `cargo nextest list -p
sbfb-workflow-engine run_store` → **0 ligne**. Créer un fichier `tests/artifact_acl.rs` ne
réparerait donc **pas** la row 26 ; seul un `mod artifact_acl` de lib le fait.

## S5 — Fidelite doc <-> code

**Le point le plus important de ce scan : §17.3 est VRAIE, ses trois ancres sont EXACTES à
HEAD, et elle est SOUS-ESTIMÉE.** [M] `lib.rs:31-41` = la struct `TraceEvent` avec
`attributes: HashMap<String, serde_json::Value>` (`:41`) ; `signed.rs:48-65` couvre
`sign_event` + `domain_message`, dont `:60` `serde_json::to_vec(event)` ; `otel.rs:22-31`
couvre `process`, dont `:25` `tracer.start(event.name.clone())` sans contexte parent. [M]
`grep -c serde_jcs crates/nexus-trace-core/Cargo.toml` → **0**. Ce n'est pas un risque
d'interopérabilité : c'est un **échec de vérification Ed25519** dès qu'un événement traverse
le JSONL avec ≥2 attributs (43 %) et de façon déterministe à 8 (200/200).

**Jumeau bloquant.** [M] `crates/nexus-core-rs/src/canonical.rs:8-10` — le fichier de
référence de la canonicalisation — affirme l'**inverse** : « (`nexus-trace-core` trace
events, and `sbfb-workflow-core` packs since S83 Phase F) rebuild the SAME `<domain> 0x00
<JCS>` shape locally ». Moitié pack VRAIE, moitié trace-core FAUSSE. Un agent qui ouvre ce
fichier conclut que la correction est déjà faite.

**Second jumeau, sur la primitive elle-même.** [M] `signed.rs:5-6` promet « + `0x00` +
**JSON canonical bytes**, producing a `SignedTraceEvent` that **auditors can verify
independently** ». Les deux moitiés sont mesurément fausses.

**Angle manqué décisif, que je porte au scope.** [M] `crates/nexus-trace-core/src/propagation.rs:12`
`pub const TRACEPARENT_KEY: &str = "_traceparent";` et `:83` `pub fn inject(ctx:
&TraceContext, carrier: &mut HashMap<String, serde_json::Value>)` — **le type exact de
`TraceEvent.attributes`**. La propagation de parent n'est donc pas « morte » (le scan l'avait
conclu en ne regardant que le champ `parent_span_id`) : elle est **vivante ailleurs et 0
fois câblée** ([M] 0 call-site de `TRACEPARENT_KEY`/`inject`/`extract` hors du module), et
**réimplémentée deux fois** (`nexus-executor/src/ipc.rs:19,57,147` et
`nexus-shell-daemon-core/src/ipc_broker.rs`). **Les deux défauts composent** : injecter le
traceparent + un seul autre attribut place l'événement à n=2, donc ~43 % de signatures
invérifiables.

**Correction de compte due au scan.** Il affirme « les **4** fixtures utilisent
`HashMap::new()` ». [M] Il y en a **3** (`signed.rs:130`, `lib.rs:110`, `batch_log.rs:115`) ;
la quatrième, `otel.rs:80`, est `HashMap::from([("key".into(), json!("value"))])`. La
conclusion (« la suite ne peut pas rougir ») survit ; le compte, non.

**Blast radius — la fenêtre de correction est gratuite.** [M] `grep -rn
'SignedCanaryProcessor|verify_signed_event|OtelProcessor' --include=*.rs crates/` hors
`nexus-trace-core` → **aucun résultat**. Seuls consommateurs : `BatchLogProcessor`
(`nexus-executor/src/main.rs:42`, `nexus-shell-daemon/src/runtime.rs:508`). Zéro signature
de production à préserver ⇒ **CORRIGER, ne pas wrapper** : l'option « wrapper » de M.2
laisserait le chemin cassé atteignable sous un vrai tag du census.

**Paliers d'acceptance — les référents n'avaient jamais été ouverts.** La source autoritaire
est le Research Pack **§22.5** (`:2304`), pas `sprint83_at_matrix.md` (qui est la matrice
AT/OS d'accessibilité de la Phase B). [M] `WA-T2` item **6** = « joindre les sorties dans un
ArtifactRef type » et item **7** = « consommer cet artefact depuis une autre app » : M.4
**conflate 6 et 7**. [M] `WA-T5` item **6** = « muter chaque classe de
**digest/receipt/lineage** et verifier le rejet » — **trois familles**, alors que M.3 ne
nomme qu'un test de digests. [M] `WA-T5` item **5** = « verifier quotas, backpressure,
**retention et suppression** » — le palier de `retention_enforced`, que M.4 ne cite pas.
[M] `grep -rn 'WA-T2\.6|WA-T5\.6' .planning/ docs/` → **une seule ligne dans tout le dépôt**,
`plan.md:1053`, celle qui les cite.

**Enfin, §17.4 « Ce que la provenance ne prouve pas »** (`research:1490-1501`) n'a été ouvert
par aucun scan : il énumère 7 non-garanties et conclut « **Elle prouve qui declare avoir
fait quoi** ». C'est le référent des bornes honnêtes que le body de M devra écrire.

## Affirmations tombees en refutation

| Affirmation | Scan | Pourquoi elle tombe | Ancre |
|---|---|---|---|
| « `verify_signed_event` échoue **dès 2 attributs, y compris dans le même processus** » | S1a | Les deux moitiés fausses : le chemin sans frontière serde vérifie toujours (`event.clone()` préserve l'ordre), et l'échec est probabiliste | [M] sonde : 0/200 à n=1, **86/200** à n=2, 200/200 à n=8 ; `signed.rs:51-53` |
| « 40 encodages distincts du **MÊME** événement » | S1a | Un même objet donne 1 encodage ; 40 exige 40 instances **fraîches** | [M] `bytes-differ-after-roundtrip 0/200` à n≤1 |
| « recalculé : 27 + keystore + trace-core = 29 » présenté comme la sortie du grep | S1a | Le grep rend **30** lignes ; le 4e site (`pack.rs`, miroir) est omis. Le 29 est juste, l'itemisation non | [M] `grep -rn 'pub const DOMAIN_…' crates/ \| wc -l` → 30 ; 4 fichiers |
| « comptes divergents 6 / 8 / 9 » comme contradiction | S1a | Trois **objets** différents : digests de pack / rôles de signature / nœuds du graphe | [M] `pack.rs:224` `ALL: [Self; 6]` ; §17.1 = 8 rôles ; §17.2 = 9 nœuds |
| « le correctif JCS coûte 0 dep : une ligne `serde_jcs` » | S1b | **Régressif** : `timestamp_unix_ns` u64 ns ≈1,78e18 > 2^53 s'écrase sous JCS ; l'engine n'a pas la garde I-JSON | [M] `lib.rs:37` ; `pack.rs:142`+`:505-521` vs `hash.rs:357-376` (`_ => Ok(())`) |
| « 133 groupes multi-versions » | S1b | 133 = **entrées de version**, pas groupes | [M] noms distincts ≪ 133 ; `deny.toml` documente « 72 », lui-même périmé |
| « CLAUDE.md : les deux moitiés sont fausses » | S1b | Sur-lecture : « fermer 4 RUSTSEC » et « il en reste 2 » sont compatibles ; « licenses rouge » est vrai de `origin/master` | [M] `ce99c974` non-ancêtre de `origin/master` |
| « patron canon sur 8 surfaces » pour `roles_not_interchangeable` | S2 | ≥4 formes distinctes ; 4 ancres ne prouvent que la moitié **nécessaire** (aucun `verify`) | [M] `grep 'fn cross_domain_signature_rejected'` → **2 hits** ; seul `pack.rs:1767` est complet |
| « rôle 5 (attestation de run) : AUCUN tag » | S2 | `DOMAIN_RUN_EVENT_V1` existe, RÉSERVÉ, même arrangement que les 2 autres | [M] `canonical.rs:404` |
| « sonde `zz_preflight_probe.rs` déposée dans l'arbre » | S2 | N'existe pas au HEAD réel ; arbre propre | [M] `ls crates/nexus-trace-core/tests/` → No such file ; `git status` vide |
| « le threat model n'a pas de mitigation opposable » (`THREAT_MODEL:262-265`) | S3 | Non-sequitur (stockage de clés ≠ autorisation de lecture) **et** périmé (coffre L1 livré) | [M] `vault.rs:17,35,120` AES-256-GCM |
| « `/blob-serve` : loopback durci, donc borne = processus local » | S3 | Aucune des trois défenses ne s'applique à cette route | [M] `http.rs:272` « no bearer, no Host check, no Origin check » |
| « le gate row 25 **passe à vide** » | S5 | nextest sort **4** avec `error: no tests to run` — échec bruyant | [M] `RUN_EXIT=4`, `0 tests run: 0 passed, 160 skipped` |
| « les **4** fixtures utilisent `HashMap::new()` » | S5 | Trois ; `otel.rs:80` porte 1 attribut | [M] `grep -rn 'attributes:' crates/nexus-trace-core/src/` |
| « mesuré à HEAD `709ff4e2` » | **les 6** | HEAD réel `22a34082` (delta docs-only : aucune ancre invalidée, mais l'estampille est non ancrée) | [M] `git rev-parse HEAD` |

## Ce que la Phase M doit faire differemment du plan

1. **`nexus-trace-core` : CORRIGER, jamais wrapper — et le fix n'est pas « ajouter
   `serde_jcs` ».** L'option « OU wrapper conforme » (M.2) doit être **retirée** : 0
   call-site de production ([M]), donc la correction est gratuite et sans migration, alors
   qu'un wrapper laisserait un signeur cassé atteignable sous `DOMAIN_TRACE_EVENT_V1`. Le
   fix doit combiner JCS **et** une garde I-JSON, ou changer la représentation du timestamp
   — sans quoi il transforme un bug de vérification en **collision de pré-image**.
2. **Le test `jcs_guaranteed` doit être déterministe, pas un aller-retour à ≥2 attributs.**
   [M] Un bras rouge à n=2 est vert 57 % du temps. Forme correcte : construire N événements
   **frais** à ≥6 attributs identiques et asserter que `domain_message` produit **exactement
   1 valeur distincte** — rouge avec probabilité ≈1 avant le fix, vert par construction
   après.
3. **Fermer l'asymétrie de gardes de l'engine.** `hash.rs` doit recevoir la garde
   `MAX_SAFE_INTEGER` que `pack.rs` porte déjà, **avant** qu'`ArtifactRef` et les receipts
   n'empruntent `canonical_bytes`. Fermer la classe, pas l'instance.
4. **L'ACL ne peut pas être une politique de façade : elle doit être structurelle.** Le clair
   d'un artefact privé/secret **ne doit jamais entrer** dans le store servi par
   `BLOBS_ALPN`. Structure à deux capabilities : `ArtifactRef` porte le digest (verify-cap :
   intégrité, lineage, dédup), la lecture exige une read-cap distincte détenue par
   l'audience. Le test `digest_is_not_read_right` doit porter un bras prouvant que
   `/blob-serve/{digest}` ne rend pas l'artefact privé — sinon il ne peut pas rougir sur le
   chemin qui compte.
5. **Renommer l'« audience » de M.** [M] `GrantAudience` (`capability.rs:543`) occupe déjà le
   mot dans le même crate, avec une classe de menace nommée. Choisir `read_audience` /
   `disclosure_scope`, ou documenter explicitement la disjonction. Coût nul si tranché avant
   le premier `pub enum`.
6. **La signature vit chez l'hôte.** [M] L'engine est keyless par charte (`activity.rs:36-37`)
   et n'a aucune dep de clé ; `sbfb-workflow-core` a `ed25519-dalek` en **vérification
   seule**. Répliquer le PLAN-ADAPT de L1 : types + trait hôte-neutre côté engine,
   vérification côté core, **signature côté `nexus-core-rs`**.
7. **« Suppression » = crypto-shredding + dé-tag, jamais effacement d'octets.** [M]
   `blobs.rs:130-133` : aucun GC. À énoncer dans le code **et** dans le body, sinon
   `retention_enforced` prouve une propriété que le réseau contredit. C'est aussi la
   justification du chiffrement **par artefact** (une clé globale rendrait le shredding
   impossible).
8. **Réconcilier les deux gates.** Row 26 : nommer un **module de lib** `artifact_acl` — [M]
   un fichier `tests/artifact_acl.rs` ne serait **pas** matché par le filtre positionnel.
   Row 25 : ajouter `-p sbfb-workflow-core` à M.4, ou déplacer l'objet du gate ; et amender
   la colonne évidence « 6 digests » qui étiquette un gate de **graphe** avec un compte de
   **pack**.
9. **Corriger les paliers cités par M.4.** `WA-T2.6` → **WA-T2.6 + WA-T2.7** ; `WA-T5.6`
   couvre **digest/receipt/lineage** (trois familles) ; ajouter **WA-T5.5** pour
   rétention/suppression. Source autoritaire = Research Pack **§22.5**, pas
   `sprint83_at_matrix.md`.
10. **0 famille DOMAIN neuve.** Consommer les 3 tags réservés + le patron `digest_for`
    (`pack.rs:1345`) et les contextes locaux (`hash.rs:26-36`). Toute croissance imposerait
    de ré-écrire ~31 à 36 affirmations « census 29 » réparties sur 11 fichiers `.rs`, 16
    lignes de `THREAT_MODEL.md`, 4 de `PATTERNS.md`, 1 de `README.md` — dont la plupart hors
    du diff de M. C'est la classe de défaut du post-mortem L1.

## Scope propose pour la Phase M

| # | Livrable | Fichier(s) cible(s) | Test qui le prouve | Gate |
|---|---|---|---|---|
| M-a | Correction `nexus-trace-core` : JCS **+** garde I-JSON entiers (ou timestamp ms/chaîne), redéfinition de la v1 courante de `DOMAIN_TRACE_EVENT_V1` | `crates/nexus-trace-core/{Cargo.toml,src/signed.rs,src/lib.rs}` | `signed::tests::jcs_guaranteed` (déterministe, N événements frais ≥6 attributs → 1 encodage) + un test de non-collision de timestamps voisins | pré-requis G-PROVENANCE |
| M-b | Propagation de parent : câbler `propagation::inject/extract` (0 call-site aujourd'hui) ou **scope-cut explicite** avec carry nommé | `crates/nexus-trace-core/src/{otel.rs,propagation.rs}` | test d'identité de trace/parent côté exporté (pas un `exported >= 1`) | §17.3 |
| M-c | Garde I-JSON dans la porte canonique de l'engine (fermer l'asymétrie avec `pack.rs`) | `crates/sbfb-workflow-engine/src/hash.rs` | test miroir de `pack.rs:2080-2108` (prouver l'écrasement **avant** de le refuser) | G-PROVENANCE |
| M-d | `ArtifactRef` v1 + trait `artifact` hôte-neutre (patron `SecretResolver` : réf. opaque, `Debug` rédacteur, fail-closed), relié à `ArtifactOutput.v1` | `crates/sbfb-workflow-engine/src/artifact_acl.rs` (module de lib) | `artifact_acl::tests::digest_is_not_read_right` | **G-ARTIFACT-ACL** |
| M-e | Classes public/privé/secret/réseau + `read_audience` (nom distinct de `GrantAudience`), ACL fail-closed | idem M-d | `artifact_acl::tests::{read_audience,encryption}_enforced` + bras `/blob-serve/{digest}` | **G-ARTIFACT-ACL** |
| M-f | Chiffrement **côté hôte** (clé par artefact, réutilise le patron `vault.rs`) + rétention (`Clock`) + suppression = crypto-shredding + dé-tag | `crates/nexus-core-rs/` (impl) + trait engine | `artifact_acl::tests::retention_enforced`, bornes honnêtes assumées | **G-ARTIFACT-ACL** / WA-T5.5 |
| M-g | Graphe de provenance : types + vérification (core), signature (hôte), 3 tags réservés consommés, 0 famille neuve | `crates/sbfb-workflow-core/src/provenance.rs` | `provenance::tests::mutation_of_each_{digest,receipt,lineage}_rejected` (itère un `ALL` compile-time) + `roles_not_interchangeable` sur le patron `pack.rs:1767` | **G-PROVENANCE** / WA-T5.6 |
| M-h | Réconciliation gates + paliers : M.4 ajoute `-p sbfb-workflow-core` ; row 25 évidence corrigée ; WA-T2.6+2.7 / WA-T5.5+5.6 | `.planning/active/sprint83_{plan,gate_matrix}.md` | les deux commandes de gate sélectionnent ≥1 test | rows 25 + 26 |
| M-i | Clôture docs-contrat : `canonical.rs:8-10`, `signed.rs:5-6`, THREAT_MODEL **§19.5** + canari 5→6 + `:1960`, `docs/workflow/llms.txt` | 4 fichiers de référence | `check-frontier-contracts.sh` exit 0 + canari 6 surfaces | DoD (d) §4 |

## Frontieres neuves creees par la Phase M

Le gate `check-frontier-contracts.sh` est **OPT-IN** : [M] sa boucle ne parcourt que les
lignes portant déjà `// FRONTIER: `. Une frontière jamais annotée n'est **jamais** détectée.
C'est donc ici qu'elles doivent être nommées.

Test-acteur — « qui LIT cette primitive, qui n'est pas le code Rust lui-même ? » :

1. **`ArtifactRef` v1 → un AUTRE NŒUD (wire).** [M] `plan.md:1161` (Phase P) : « Le **wire
   compute** : `Task`/`Result`/`ArtifactRef` **additifs 0-bump (AM-12)** ». **Frontière.**
2. **`ArtifactRef` v1 → une AUTRE APP (contrat).** [M] `plan.md:1685` (Phase Y) : « résolution
   `ArtifactRef` sous ACL (Phase M) » et `:1691` `handoff_artifactref_acl.rs` ; [M] §22.5
   WA-T2.7 « consommer cet artefact depuis une autre app ». **Frontière.**
3. **Le trait `artifact` hôte-neutre → un IMPLÉMENTEUR dans un autre crate** (le daemon).
   Frontière inter-crate, même process — traiter en **paragraphe test-acteur** sur le modèle
   `secret.rs:54-64`, pas en étiquette machine.
4. **Le graphe de provenance exporté**, s'il produit une forme consommable hors-process.

**Étiquette à produire DANS le commit.** Sous la posture 0-DOMAIN recommandée, une étiquette
machine `// FRONTIER:` est **impossible** : [M] le gate exige trois conditions (un `DOMAIN`,
un `*_VERSION` résolvable, et un `schema_for!` ou une exemption `// FRONTIER-NO-SCHEMA:`).
Les frontières 1, 2 et 3 doivent donc porter un **paragraphe test-acteur explicite** (section
« Pre-launch protocol / frontier » sur le modèle `secret.rs:54-64`) + une entrée dans
`docs/workflow/llms.txt`. **Ne pas** poser de `// FRONTIER:` sur un type non signé : le gate
rougirait en exigeant un domaine inexistant.

Deux surfaces gelées voisines à ne pas franchir en silence : [M] `bridge-parity [16 methods]`
(M ne doit ajouter **aucun** verbe bridge ; la fermeture réelle est le Vitest
`authority.test.ts`, pas ce leg) et `hitl-parity [15 fields]`.

## Invariants a ne pas casser

- **Census `DOMAIN_*_V1` = 29**, liste nominale gelée. [M] gate vert au départ ⇒ toute dérive
  sera imputable au diff de M.
- **0 bump** : `*_VERSION` restent à 1 ; `ArtifactRef` est purement additif (AM-12) ; tout
  test « legacy decode » de l'ancien encodage trace-core est à **supprimer**, pas à conserver.
- **0 dep runtime neuve, 0 nœud de lock neuf** — établi faisable ([M] S1b).
- **`sbfb-workflow-core` PUR** (0 Tokio/réseau/DB) ; **`sbfb-workflow-engine` KEYLESS et
  déterministe** (aucune entropie, aucun `iroh-blobs`).
- **Canonicalisation = `serde_jcs`, jamais `serde_json::to_vec`** ([M] `preserve_order` unifié
  ON), digests blake3 **hex minuscule** strict.
- **`DOMAIN_PROVENANCE_V1` intouché** : la chaîne SLSA de deploy reste byte-intacte.
- **Le canari doit rougir** en passant à 6 surfaces ; ne pas contourner
  `assert_eq!(surfaces.len(), …)`.
- **Aucune promesse future dans un commentaire de code** (leg anti-promise du gate).
- **Toute vérification supply-chain citée passe `--all-features`**, sinon elle est
  tautologique ([M]).
- **Muter avant de croire un test vert** : aucun gate n'est cité comme preuve sans avoir été
  vu rouge.

## Risques et questions ouvertes pour le PO

1. **Voie d'enforcement de l'ACL.** (a) chiffrement obligatoire avant `add_bytes` pour
   privé/secret — seule voie compatible avec un digest qui circule, et faisable à 0 dep ; ou
   (b) armer l'`EventSender` d'iroh-blobs (`node.rs:442`, `AbortReason::Permission`) — mais
   c'est un **livrable hôte non annoncé** dans M.2, qui élargit `NodeConfig` et n'a aucun test
   aujourd'hui. **Recommandation : (a).** Décision PO requise car (b) modifierait le câblage
   du nœud.
2. **Révocation d'une décision de sécurité S16 ?** [M] `http.rs:246-256` documente que « the
   blob content is already public by construction ». Les classes privé/secret invalident cette
   hypothèse porteuse. Soit M ne route **jamais** un artefact privé par blob-serve (voie (a)),
   soit la décision S16 est explicitement révoquée. À acter, pas à contourner.
3. **Périmètre de la propagation de parent (M-b).** Livrable annoncé en M.2, mais aucun test
   n'est prévu en M.3 et M.4 ne le mentionne pas — un livrable peut donc être expédié sans
   preuve tout en satisfaisant la condition binaire. Livrer, ou scope-cut avec carry nommé ?
4. **Représentation du timestamp trace-core.** Garde I-JSON (le timestamp ns reste exact mais
   toute valeur >2^53 est refusée) **ou** passage en millisecondes / chaîne décimale (~1,78e12,
   sous 2^53). Le second change la surface publique du type ; le premier peut refuser des
   événements légitimes. Arbitrage de conception.
5. **Bornes honnêtes du chiffrement.** [M] KEYSTORE-LAYER2-INERT : la couche 2 OS keyring est
   sur builder MOCK, donc la seule protection at-rest réelle est Argon2id(PIN). Confirmer que
   M reprend la formulation de `vault.rs:22-31` plutôt que de revendiquer une double couche.
6. **Le lineage comme oracle.** Un graphe qui référence des artefacts privés révèle leur
   **existence**, leur nombre et leur chronologie même sans droit de lecture. Projection
   rédigée par audience, ou résiduel documenté en §19.5 ?
7. **Classe `réseau`.** Que signifie « audience » pour une classe par définition diffusable, et
   la suppression y est-elle déclarée impossible ? Non précisé par le plan. Interagit avec
   `G-NETWORK` (Phase P) et `G-WIRE-COMPAT` (row 16, qui nomme `ArtifactRef` vNext).
8. **Deux jumeaux de canonicalisation de provenance non extraits** ([M] `provenance.rs:102`
   et `gates.rs:631`, corps logiquement identiques, non-JCS, dette T-NN+3 ouverte depuis S70).
   M en créerait un **troisième**. Harmoniser dans M, ou router la dette explicitement ?
9. **CI-1 hors périmètre de M.** [M] `advisories FAILED` subsiste (RUSTSEC-2023-0071 `rsa`
   sans patch amont, `enumset` yanked, via la feature `tor`). M n'ajoute aucune dep et
   n'aggrave rien, mais la condition de sortie « job VERT » du carry A1 n'est pas atteignable
   au push. `ignore` motivé, remontée d'`arti-client`, ou re-labellisation ?
