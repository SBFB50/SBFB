# Sprint 83 Phase F — Preflight G8

**Sprint** : 83 — chantier « Workflow Agents » (43 phases A..AJ1).
**Phase** : F — Format portable et signé (`SBFBPromptAgentPack.v1`), plan canonique
`.planning/active/sprint83_plan.md:577-625` (F.1 scope :579-588, F.2 livrables :590-600,
F.3 tests :602-609, F.4 acceptation :611-617, F.5 commit :619-625).
**Date** : 2026-07-20. **HEAD** : `6f95cd2` (`git log --oneline -1`), arbre propre.
**Méthode** : mode ultracode Workflow — **5 agents de scan** (S1a prior-art OSS, S1b
dépendances/CVE, S2 décisions historiques, S3 couverture THREAT_MODEL, S4 invariants
wire/frontières), **1 passe adversariale** de réfutation sur l'intégralité des findings,
puis cette synthèse. Le synthétiseur a **rejoué à HEAD** les mesures porteuses : census
DOMAIN, exécution du gate frontier, suite `sbfb-workflow-core`, exit code nextest
zéro-match, présence/exécution de blake3 côté TS, absence de JCS côté TS, deps du crate
cible, ancres `canonical.rs`, `task.rs:316`, `hash.rs:225-300`, espace `§P`.
**Verdict** : **PLAN-ADAPT** (aucun DESIGN-CONFLICT). Signaux : S1a=PLAN-ADAPT,
S1b=PLAN-ADAPT, S2=PLAN-ADAPT, S3=PLAN-ADAPT, S4=PLAN-ADAPT.

---

## 1. Rationale du verdict

Aucune décision Day-0 ni aucun gel D2 n'est en tension avec le plan F. Vérifié sur disque :

- **D2 GELÉ et non ré-ouvert** : `sprint83_kickoff.md:285-301` — canonicalisation JCS
  RFC 8785 + Ed25519 + constantes `DOMAIN_*` dédiées, `subject[] = [{name, digest:
  {blake3}}]` à la in-toto, 6 digests séparés (§9.3), interdits durs (secret/endpoint
  credentialisé/code exécutable/grant/état live/mémoire), distribution zip + iroh-blobs +
  raw-op `prompt_agent_pack` **sans bump `FEED_FORMAT_VERSION`**. Les alternatives
  (DSSE/PAE complet, Sigstore, OCI/ORAS, format tiers autoritaire) sont REJETÉES au
  kickoff (:304-317). Le plan F n'en ré-ouvre aucune.
- **Politique pre-launch favorable** : un format NEUF n'est pas un bump ; `CLAUDE.md
  §Pre-launch protocol policy` autorise l'édition libre du canonical jusqu'au tag v1.0, et
  une nouvelle op raw-op ne bump pas `FEED_FORMAT_VERSION`.
- **Le stub cible existe et est prêt** : `crates/sbfb-workflow-core/src/contracts.rs:41-44`
  `pub struct SbfbPromptAgentPack {}` porte déjà `#[serde(deny_unknown_fields)]` et
  `#[schemars(title = "SBFBPromptAgentPack.v1")]` ; `schema_for!(SbfbPromptAgentPack)` est
  déjà appelé (`contracts.rs:99`), le pack est l'entrée **1/7** du registre.
- **Suite verte au départ** : `cargo nextest run -p sbfb-workflow-core --locked` →
  `22 tests run: 22 passed` (mesuré ce jour).
- **Gate frontier vert au départ** : `bash scripts/check-frontier-contracts.sh` →
  `clean (… + DOMAIN census [25 frozen] + …)`, EXIT=0 (mesuré).
- **Aucune zone rouge touchée** : R-iroh-audit P0 (0 surface iroh en F), R-wasmtime-cve P0
  (décision gelée « OS sandbox pour Factory, pas wasmtime »), R-libcrux-hax P2 (crypto SBFB
  = `ed25519-dalek`), R-pyodide-escape. Formule reprenable du body Phase C.
- **Supply-chain vert** : `cargo deny check advisories` et `… licenses` passent localement
  (cargo-deny 0.19.2) depuis `ce99c97` (Phase A1, allowlist Unlicense/MIT-0, job pip-audit
  supprimé). La mention « supply-chain rouge (CI-1) » de `CLAUDE.md` est **périmée** et ne
  doit pas être reconduite dans le body F.

**Pourquoi PLAN-ADAPT et pas EXECUTE** : le texte de F.2/F.3/F.4, tel qu'écrit, est
**mécaniquement infaisable en l'état** sur six points mesurés, et **sous-spécifié** sur
une dizaine d'autres. Les trois plus durs :

1. **F.4 est auto-contradictoire.** Il exige `bash scripts/check-frontier-contracts.sh`
   exit 0 alors que F.2 ajoute ≥ 4 familles `DOMAIN_*_V1`. Mesuré : census = **25**,
   `DOMAIN_CENSUS_FROZEN=25` (`scripts/check-frontier-contracts.sh:190`), comparaison
   stricte `-ne` à `:193`. Toute famille nette rend le gate ROUGE. Le plan F ne mentionne
   nulle part le census (0 hit `census` sur `sprint83_plan.md`).
2. **Le plan contredit la charte du crate sur l'emplacement des DOMAIN.** F.1 (:582) dit
   « append à `canonical.rs:77-332` » — unique `canonical.rs` du dépôt =
   `crates/nexus-core-rs/src/canonical.rs`, crate qui tire iroh + tokio non-optionnels —
   pendant que F.2 (:591-594) liste ces constantes sous la puce `crates/sbfb-workflow-core`,
   dont la charte écrite est « No Tokio, no network, no database » (`Cargo.toml:3`,
   `src/lib.rs:9`).
3. **La commande d'enforcement du gate que F ouvre ne matche aucun test de F.**
   `sprint83_gate_matrix.md:69` : `cargo nextest run -p sbfb-workflow-core import_hostile`.
   Mesuré ce jour : **EXIT=4** (`no tests to run`) — donc ROUGE mécanique, et aucun des
   quatre noms de F.3 (`pack::tests::six_digests_stable`, `jcs_cross_lang_parity`,
   `hostile_corpus_refused`, `signature_before_parse`) ne contient `import_hostile`.

---

## 2. Scan S1a — Prior art OSS (verdict de scan : PLAN-ADAPT)

### Faits établis

- **`serde_jcs 0.2.0` effondre les floats non finis IMBRIQUÉS sur `null`, sans erreur.**
  Mesuré en Phase E et documenté dans le dépôt :
  `crates/sbfb-workflow-engine/src/hash.rs:227-252` (tableau MESURED trois chemins :
  scalaire nu → `Err("invalid float value")` ; imbriqué → `Ok({"v":null})` ; via
  `to_value` → `Ok({"v":null})`). Conséquence : `NaN`, `+Inf`, `-Inf` et un `null`
  explicite partagent une forme canonique et un digest. Cause racine :
  `serde_json/src/ser.rs` `serialize_f64` intercepte `FpCategory::Nan|Infinite` en
  `write_null` avant que le garde de `JcsSerializer::serialize_f64` ne soit atteint.
  La porte corrective existe — `hash.rs:269` `canonical_bytes` + `hash.rs:283`
  `reject_nulls` — mais **uniquement dans `sbfb-workflow-engine`** ;
  `nexus-core-rs::canonical_bytes` (`canonical.rs:353-360`) appelle `serde_jcs::to_vec`
  sans filtre, et `sbfb-workflow-core` n'a **même pas la dépendance `serde_jcs`**
  (`Cargo.toml:11-19`, mesuré).
  *Réserve mesurée* : aujourd'hui `crates/sbfb-workflow-core/src/*.rs` ne contient
  **aucun** `f64`/`f32` et un JSON sur disque ne peut pas porter `NaN`/`Infinity`
  (RFC 8259) ⇒ le vecteur est P2 préventif tant que le pack ne déclare pas de flottant,
  pas un P0. Il devient P0 dès qu'un champ flottant entre dans le format signé.
- **`serde_jcs 0.2.0` route i64/u64 à travers `f64`** : `serde_jcs-0.2.0/src/lib.rs:301-307`
  (`write_i64` → `write_f64(value as f64)`), `:343-349` (`write_u64`). Mesuré :
  `9007199254740993u64` et `9007199254740992u64` produisent la **même** sortie canonique
  `9007199254740992` ; `1753000000123456789u64` devient `1753000000123456800`.
  En revanche i128/u128 échouent **bruyamment** (`Err("i128 is not supported")`) — pas de
  perte silencieuse sur 128 bits. RFC 8785 §3.2.2.3 borne les vrais entiers à
  [-9007199254740991, 9007199254740991] et §3.1 recommande la représentation en chaîne
  au-delà. **Aucun garde I-JSON n'existe dans le dépôt** (grep `9007199254740991|2^53|I-JSON`
  sur `*.rs` : 0 hit).
- **Précédent CVE du pattern « signature valide + payload non parsable = VERIFIED »** :
  GHSA-w6c6-c85g-mmv6 / CVE-2026-39395 sur `cosign verify-blob-attestation`, corrigé
  v3.0.6 / v2.6.3, CVSS 4.3 — « For new-format bundles, the predicate type validation was
  bypassed completely ». L'analogie est **structurelle, pas littérale** (le faux positif
  cosign est conditionné à un opt-out `--check-claims`, absent du design F).
- **Le tri des clés est ALIGNÉ** entre `serde_jcs` (`Utf16Key` construit par
  `.encode_utf16()`, `impl Ord` sur le `Vec<u16>`) et JS (`Object.keys().sort()`, unités
  de code UTF-16), conformément à RFC 8785. Piège classique NON présent — à condition de
  ne jamais substituer un `BTreeMap<String, _>` Rust (tri par code point, pas par unité
  UTF-16). Un vecteur à clé non-BMP est nécessaire comme témoin discriminant.
- **Clés JSON dupliquées : « dernier gagne » silencieux** pour `serde_json::Value`,
  `BTreeMap`/`HashMap` et `#[serde(flatten)]` ; un champ de struct `#[derive(Deserialize)]`
  erreure de lui-même (et ce n'est **pas** l'effet de `deny_unknown_fields`). Comme SBFB
  signe la re-sérialisation JCS d'une valeur déjà parsée, la fonction octets-reçus →
  octets-signés est non injective : `{"caps":{"k":1,"k":2},"a":1}` et `{"caps":{"k":2},"a":1}`
  produisent les mêmes octets signés. Remède disponible sans nouveau nœud de lock :
  `serde_with` 3.20.0 est déjà dans `Cargo.lock:7826` (`MapPreventDuplicates`).
  Nuance : `JSON.parse` de Node est aussi last-wins ⇒ pas de divergence de parité Rust/TS
  sur ce point.
- **Durcissement zip déjà fait et réutilisable** : `crates/sbfb-factory/src/fork.rs:68`
  `MAX_ARCHIVE_ENTRIES = 4096` (ajouté S75 Phase G `8b53c38`), garde zip-slip `:244`
  (introduite S74 Phase B `bcfc155`), test `:566`
  `fork_from_blob_rejects_zip_slip_all_vectors` (6 vecteurs : `../escape.txt`, `/abs.txt`,
  `\win.txt`, `a\b.txt`, `nested/../../escape.txt`, `C:/escape.txt`) — **mesuré vert**
  (`cargo nextest -E 'test(zip_slip)'` → 1 passed). Second parseur zip borné, PUBLIC et
  100 % en mémoire : `crates/nexus-shell-daemon-core/src/blob_serve.rs:95` `pub fn load`,
  `:190` `pub fn validate_zip_path`, caps `:39`/`:42`, garde anti-header-menteur `:126-130`.
  `zip` 8.6.0 au lock n'est **pas** affecté par CVE-2025-29787 (corrigé en 2.3.0).
  Axes NON couverts par le prior art local : casse Windows (`A.json`/`a.json`), noms
  réservés Windows (`CON`, `NUL`, `AUX`, `COM1`), Unicode confusables (UTS #39), ratio de
  compression (bombe) distinct de la taille absolue.
- **`serde_json` 1.0.149 applique par défaut une limite de profondeur** de récursion
  (`de.rs:38` `disable_recursion_limit`, `:215` le désactive explicitement). La borne de
  **taille** avant `from_slice` et la borne de **cardinalité** ne sont pas fournies.
- **Vecteurs SVG/HTML** : la taxonomie DOMPurify (wiki cure53 « Attack Classes & Bypass
  History ») est *une* source publique réutilisable, pas la seule (PortSwigger XSS cheat
  sheet 2026, PayloadsAllTheThings). Elle n'apporte **aucun** vecteur Markdown (mot absent
  de la page). Doctrine de test, en paraphrase fidèle : « Weak test: substring checks miss
  parser mutation and encoding. Better test: insert and inspect the resulting DOM. »

### Not evidenced (S1a)

- Le comportement de la bibliothèque TS effectivement retenue par le plan (`jcs`,
  `sprint83_plan.md:599`) sur les floats non finis : non mesuré (npmjs 403).
- Aucune divergence prouvée Rust/TS sur la normalisation Unicode NFC/NFD (ni l'un ni
  l'autre ne normalise ; RFC 8785 ne l'exige pas), faute de vecteur public.
- Aucun précédent OSS de « six digests sémantiques séparés sur un même artefact » ;
  in-toto/SLSA ne documentent le multi-digest que pour l'agilité d'algorithme —
  **INFERENCE** : le design F est sans prior art direct.
- Aucune suite OSS de fixtures ZIP hostiles clef-en-main (snyk/zip-slip-vulnerability est
  un registre de projets vulnérables, pas un corpus) ⇒ corpus à fabriquer en Rust.
- Contenu exact du §19.2 du Research Pack non lu par ce scan (lu par S3, cf. §5).

---

## 3. Scan S1b — Dépendances / CVE (verdict de scan : PLAN-ADAPT)

### Faits établis

- **Toutes les briques Rust existent déjà en `[workspace.dependencies]`** (mesuré) :
  `Cargo.toml:67 serde_jcs = "0.2"`, `:70 ed25519-dalek = "2.1"`, `:71 blake3 = "1.5"`,
  `:60 serde_json`, `:362 schemars = { version = "1.2", features = ["derive"] }`,
  `:206 zip = { version = "8.5", default-features = false, features = ["deflate"] }`.
  Lock : `serde_jcs 0.2.0` (`Cargo.lock:7760`), `ed25519-dalek 2.2.0` (`:2101`),
  `blake3 1.8.5` (`:716`), `schemars 1.2.1` (`:7560`), `zip 8.6.0` (`:11398`).
  **Aucune dépendance runtime Rust NOUVELLE n'est requise.**
- **Le crate cible ne déclare que 4 deps** (mesuré, `crates/sbfb-workflow-core/Cargo.toml:11-19`)
  : `serde`, `serde_json`, `schemars`, `blake3` ; `[dev-dependencies]` vide. Ni `serde_jcs`
  ni `ed25519-dalek`. Ajout = deux lignes `{ workspace = true }`, **0 nouveau nœud de lock**.
- **La charte du crate n'interdit pas ces deux deps** : `Cargo.toml:3` « No Tokio, no
  network, no database » et `src/lib.rs:9` « links no Tokio, no network stack and no
  database ». `ed25519-dalek` = calcul CPU sans I/O — même catégorie que le `blake3` déjà
  justifié par commentaire (`Cargo.toml:15-18`). Précédent : `crates/nexus-trace-core`
  embarque `ed25519-dalek` sans Tokio/réseau/DB.
- **Convergence de version sûre** : `ed25519-dalek 3.0.0-rc.0` existe au lock
  (`Cargo.lock:2117`) mais est tiré **uniquement par iroh** ; le requirement workspace
  `^2.1` ne peut pas résoudre sur une pre-release de majeur incompatible. Contrôle
  discriminant à jouer : `cargo tree -p sbfb-workflow-core | grep ed25519` doit afficher
  exactement `ed25519-dalek v2.2.0`. **Ne pas** utiliser `cargo tree -d -p
  sbfb-workflow-core` : mesuré `warning: nothing to print.` aujourd'hui, donc non
  discriminant (même sortie dans le cas bon et le cas mauvais).
- **Côté TS, aucune canonicalisation JCS n'existe** : mesuré
  `grep -cE '"(node_modules/)?(json-)?canonicalize"|node_modules/jcs'` →
  `web/package-lock.json:0` et `tools/factory-operator/package-lock.json:0` ; aucun
  canonicaliseur maison dans `web/src` ni `tools/factory-operator/src`.
- **En revanche blake3 EXISTE et TOURNE côté TS** : `@noble/hashes 1.8.0` est un nœud de
  **production** du lock `web/` (non flaggé `dev`), tiré par `shadcn` →
  `@dotenvx/dotenvx` → `eciesjs`. Fichier `web/node_modules/@noble/hashes/blake3.js`
  présent. **Mesuré par exécution depuis `web/`** :
  `blake3("abc") = 6437b3ac38465133ffb63b75273a8db548c558465d79db03fd359c6cd5bd9d85`
  (= vecteur BLAKE3 officiel, identique au Rust). `@noble/curves 1.9.7` expose aussi
  `ed25519`. Ces paquets ne sont **pas déclarés** dans `web/package.json` (mesuré :
  `has noble dep? false`) ⇒ les importer tels quels serait une dépendance fantôme.
- **Aucune contrainte de toolchain** : pas de `rust-toolchain.toml` ; `Cargo.toml:26`
  `rust-version = "1.91"`, CI `dtolnay/rust-toolchain@stable`, machine locale
  `rustc 1.95.0`. `serde_jcs` / `ed25519-dalek 2.x` / `blake3` / `schemars 1.2` sont
  compatibles 1.91.
- **`zip` n'est PAS dep de `sbfb-workflow-core`** (mesuré) et n'est déclaré que dans
  `nexus-shell-daemon`, `nexus-shell-daemon-core`, `nexus-test-harness`, `sbfb-factory`.
  Le plan F.2 ne demande **pas** de parseur ZIP dans le crate : il demande « Parseur borné
  + `deny_unknown_fields` sur le profil signé » (:595) — JSON/JCS. Le mot « zip » n'apparaît
  qu'à :600 comme **classe de fixtures** du corpus hostile.

### Not evidenced (S1b)

- La parité byte-identique Rust/TS sur les structures réelles du pack : aucune fixture
  golden cross-langage JCS n'existe aujourd'hui — hypothèse non prouvée.
- L'état réel du job supply-chain sur la CI distante après `ce99c97` : seule l'exécution
  LOCALE de cargo-deny 0.19.2 a été mesurée ; le job GHA utilise
  `EmbarkStudios/cargo-deny-action@v2` qui embarque sa propre version.
- Le coût bundle exact d'un import `@noble/hashes/blake3` : aucun `npm run size` exécuté.
- La stabilité dans le temps de la chaîne transitive `shadcn → @dotenvx → eciesjs →
  @noble/*` : tracée, non garantie.
- Aucune phase du plan ne porte le flip `multiple-versions = "warn" → "deny"`
  (`deny.toml` : « carry S83, external-blocked », sans phase cible).

---

## 4. Scan S2 — Décisions historiques traversées (verdict de scan : PLAN-ADAPT)

### Faits établis

- **Le census DOMAIN est gelé à 25 et la mesure à HEAD est exactement 25** (rejouée par le
  synthétiseur avec la commande verbatim du gate) : ACTIVATION_COMMIT, AGE_WITNESS, CLAIM,
  COMPUTE_GROUP, CONTRIBUTOR_ATTESTATION, CURATOR_LIST, DELEGATION_CERT, DURESS_ACK, FEED,
  INVITE, KEY_ROTATION, KEYSTORE, KUDOS, NODE_DIRECTORY, POW, PROVENANCE, RESULT,
  RUN_PROOF, SEED_REQUEST, SEED_RESPONSE, SHARD_PLAN, TASK, TRACE_EVENT, VRF_DRAW,
  WARRANT_CANARY. Gate câblé BLOQUANT : `.github/workflows/ci.yml:136`,
  `.woodpecker/ci-linux.yml:92`, `scripts/verify.sh:98-99`.
- **La recette de refresh est IMPRIMÉE par le gate lui-même**
  (`scripts/check-frontier-contracts.sh:194-197`, lu verbatim) : `DOMAIN_CENSUS_FROZEN`
  (`:190`) + « the census prose in this file (header (2), comment (2b)) » (`:44-47` et
  `:180-189`) + « docs/rust/PATTERNS.md §P70 ». Chacun de ces trois blocs de prose porte
  des nombres codés en dur (25 familles / 3 schématisées / 22 unschematised —
  `PATTERNS.md:4068`, `:4072`, `:4077`) que le seul bump du compteur laisserait FAUX en
  gate vert. **Le tripwire ne compare QUE le nombre** (`:193`).
- **Phase F serait le PREMIER bump du census de l'histoire du dépôt** :
  `git log -p --all -G"DOMAIN_CENSUS_FROZEN="` ne rend qu'une ligne `+DOMAIN_CENSUS_FROZEN=25`
  (commit `d2705b7`, S82 Phase G) et **aucune** ligne `-`. Pas de diff « refresh » copiable ;
  le modèle est le diff d'introduction (script + PATTERNS §P70 en lockstep, même commit).
- **Le NOMBRE de domaines à ajouter n'est pas tranché.** `sprint83_plan.md:592-593` nomme
  4 constantes suivies d'un « … » littéral ; `sprint83_kickoff.md:287-288` prescrit « un
  domaine par role §17.1 » ; la source primaire
  `.planning/research/workflow_agents_app_conception_ultradeep_2026-07-15.md:1444` écrit
  « Il faut separer **au moins**: » 8 rôles — 8 est un **plancher**, pas un cardinal.
  Contre-indication à l'hypothèse « F porte les 8 » : `sprint83_plan.md:984,991` attribue
  le graphe de provenance à 8 rôles domaine-séparés à la **Phase M**, et
  `sprint83_phase_e_preflight.md:144-145` ne route vers F que « le domaine du pack » et
  `DOMAIN_RUN_EVENT_V1`.
- **L'ancre `canonical.rs:77-332` est EXACTE à HEAD** : 23 constantes `pub const DOMAIN_*_V1`
  (`grep -c` = 23), première `:77 DOMAIN_TASK_V1`, dernière `:332 DOMAIN_ACTIVATION_COMMIT_V1`,
  `canonical_bytes` `:353`. Conventions non citées par le plan : valeur d'octets préfixée
  `nexus-` (jamais `sbfb-`), et la liste du module-doc `:50-65` doit être étendue.
- **`DOMAIN_RUN_EVENT_V1` est explicitement ROUTÉ vers F par le preflight E**
  (`sprint83_phase_e_preflight.md:144`) et le remplissage du stub `RunEvent.v1` est routé
  vers la **Phase I** (`:147-154` ; `sprint83_plan.md:733-735`). Il n'y a donc pas de
  tension avec la Phase E — seulement un séquencement domaine-en-F / schéma-en-I à
  documenter, la voie « motivated no-schema » étant explicitement sanctionnée par D8
  (`check-frontier-contracts.sh:195-196`).
- **Remplir le stub casse le golden `contracts.schema.json`** (schemars émet
  `properties`/`required` en plus) — précédent exact Phase D `e8180b8`
  (`contracts.schema.json +262/-1`). Refresh : `UPDATE_SNAPSHOTS=1 cargo test -p
  sbfb-workflow-core schema_snapshot_stub` (`contracts.rs:172-174`). Le registre doit
  rester à 7 : `contracts.rs:133 const CONTRACT_NAMES: [&str; 7]`, `:147
  assert_eq!(obj.len(), CONTRACT_NAMES.len(), "seven reserved identities")`. Le pack est
  l'entrée **1/7** (`contracts.rs:97-99`), pas une 8e identité. Ce `len == 7` est enforcé
  **uniquement par ce test Rust** — `check-frontier-contracts.sh` ne le vérifie pas.
- **La commande d'enforcement de `G-IMPORT-HOSTILE` ne matche aucun test de F.3**
  (`sprint83_gate_matrix.md:69` : `cargo nextest run -p sbfb-workflow-core import_hostile`).
  Mesuré ce jour : **EXIT=4**. nextest filtre sur le **nom de test**, jamais sur le
  binary-id ni le nom de fichier. EXIT 4 est un code non nul = **ROUGE** (la Phase E s'en
  servait comme signal positif de non-pollution, cf. body `6de49b1`).
- **La politique pre-launch AUTORISE F** : format neuf ≠ bump ; raw-op `prompt_agent_pack`
  sans toucher `FEED_FORMAT_VERSION` (kickoff :299-301) ; `contracts.rs:15-22` documente
  déjà que « a signing domain and format version are added when a SIGNED wire contract is
  filled ».
- **Le point PO ouvert autour de D2 n'est PAS `subject[]`** (retenu) mais l'interop
  supply-chain tierce DSSE/cosign, explicitement différée « SANS casse » (kickoff :290-292,
  question PO Q4 :874-876). Contrainte de conception : le payload canonicalisé doit rester
  séparable de la signature pour que la clause « sans casse » reste vraie.
- **Les gates ouverts par E ne pèsent pas sur F** : `G-AUTHORITY` (E→J) et `G-STORE` (E→I)
  portent sur `sbfb-workflow-engine`. F ouvre `G-IMPORT-HOSTILE` seul (fermé AG).
- **Espace `§P` — recensement à HEAD** : rust = P70, P71, P72, P73, P75 ; shell = P70, P74.
  **Prochain libre = `§P76`** (l'espace est COMMUN aux deux fichiers depuis §P74, finding
  C-1 de l'audit S82). Le pattern livré par F se rédige au wrap-up, pas en phase feature.

### Not evidenced (S2)

- Le §17.1 du Research Pack n'est pas citable comme liste CLOSE : la source dit « au
  moins » 8 rôles. La table rôles→domaines fermée n'existe nulle part.
- Aucune ratification PO écrite de Q3-Q7 postérieure au kickoff (CLAUDE.md n'enregistre
  que Q1, Q2, Q8=A, Q9). Aucun artefact daté tranchant Q4.
- Le corpus hostile §19.2 ne porte **aucune borne numérique** (tailles/profondeur/
  cardinalité exactes) : à choisir en F, pas à recopier.
- Aucune règle explicite « un contrat signé neuf DOIT être taggé `// FRONTIER:` » n'a été
  trouvée ; le bloc (2) du gate est décrit comme opt-in incrémental
  (`check-frontier-contracts.sh:41-47`). L'obligation vient du process AM-3, pas du gate —
  et la Phase B a déjà obtenu une exception PO ratifiée sur ce point
  (`sprint83_phase_b_preflight.md:82,96`).

---

## 5. Scan S3 — Couverture THREAT_MODEL (verdict de scan : PLAN-ADAPT)

### Faits établis

- **Le THREAT_MODEL courant est v21** (Sprint 83 Phase C, 2026-07-19, entrée `:1965`).
  Aucune v22 : les Phases D (`e8180b8`) et E (`6de49b1`) n'ont pas touché le fichier.
  Assets A1..A9 (`:55-67`, A9 = `needs[]` a11y, ajouté v20). Sections : §15 seed
  cross-nœud (`:977`, + 15.1..15.5), §16 sharding (`:1317`), §17 profil d'accessibilité
  (`:1669`), §18 revue et évolution (`:1698`, fin de fichier `:1999`). **Il n'existe pas de
  §19.**
- **Aucune section ni row ne modélise le pack** : 0 occurrence de `SBFBPromptAgentPack`,
  0 occurrence de « rug-pull » dans `docs/security/`, 0 occurrence de « SSRF », 0
  occurrence de « SVG », 0 occurrence de « Markdown », 0 occurrence de « zip-slip ».
- **Correction importante — le dépôt modélise DÉJÀ des imports d'artefacts tiers signés**,
  sous le vocabulaire « ingest » et non « import » : §10 Feed (`:544`, entries Ed25519
  tierces, T-FEED-FORGERY / T-FEED-SPAM, cap payload 64 KB + GCRA + PoW), §15/§15.1
  (`SeedRequest` authentifié + anti-replay nonce `:1004` ; `NodeDirectoryEntry` signé
  ingest subscription-gated `:1016-1017` ; flood Sybil borné cap 256 / TTL 48 h `:1037`),
  §12 ProofCard (`:690`). **F crée la première surface de sa CLASSE** (canal local,
  artefact = prompt exécutable), pas la première surface d'import signé hostile.
- **Le « §19.2 » cité par F.1 n'est PAS une section du THREAT_MODEL** : c'est
  `.planning/research/workflow_agents_app_conception_ultradeep_2026-07-15.md:1583`
  (« ### 19.2 Import hostile », §19 threat model `:1567`). Ses **9 clauses** littérales :
  (1) limite de taille, profondeur, cardinalité et **longueur des chaînes** ; (2) parse
  structuré + `deny_unknown_fields` pour le profil signé ; (3) aucune interpolation de
  chemin hors racine ; (4) aucun `eval`, script, template exécutable ou désérialisation
  native ; (5) digests et signatures vérifiés avant import actif ; (6) scan secrets et
  endpoints suspects ; (7) **schémas `$ref` locales ou pinnées** ; (8) rapport
  `mapped/approximated/refused` ; (9) **un import reste brouillon non signé jusqu'à revue
  et republication**. Les clauses **(7)** et **(9)**, plus la « longueur des chaînes » de
  (1), ne sont reprises **nulle part** dans le plan — ni en F, ni en AG (qui ferme pourtant
  `G-IMPORT-HOSTILE`, `:1898-1932`), ni en S.
- **Doctrine « crypto-avant-I/O » : précédent EXACT et câblé.**
  `crates/nexus-worker-core/src/engine/shard_claim.rs:271-294` (« Signature FIRST — a forged
  manifest never reaches membership / I/O »), documenté `THREAT_MODEL.md:1358-1367`.
- **NUANCE FACTUELLE cardinale — l'ordre réel du dépôt n'est PAS « signature avant parse ».**
  Mesuré : `iroh_runtime.rs:1071-1072`, `:1204-1205` et `:1321-1322` font
  `serde_json::from_slice(&body)` AVANT `verify_signed_list_ingest` ;
  `node_directory.rs:272-293` fait version → cap cardinalité → caps par champ → attribution
  → `canonical_bytes` → `crypto::verify`. Et cette déserialisation pré-signature **n'est pas
  bornée aujourd'hui** : ni `deny_unknown_fields` sur `CuratorListEntry` (`curator.rs:182`)
  ni sur `NodeDirectoryEntry` (`node_directory.rs:201`), ni cap de taille sur le corps du
  blob (`blobs.rs:103-112`, aucun `MAX_*` dans `nexus-shell-daemon-core/src`). Les caps de
  cardinalité sont INTERNES à `verify_signature`, donc post-parse. Un chemin parse même en
  `serde_json::Value` non typé avec `unwrap_or_default` (`gates.rs:664-686`).
  Corollaire : « jamais sur les octets bruts » ne vaut que pour les SIGNATURES — un digest
  blake3 EST vérifié sur les octets bruts avant tout parse (`fork.rs:148-155`).
- **Menaces spécifiques non couvertes**, avec leur analogue le plus proche :
  - *substitution de pack / rug-pull* → analogues : `THREAT_MODEL.md:200` (§5.3, pin
    `commit_sha` 40 hex signé Ed25519 — même forme que `PACK.lock`), `:1019` (floor
    anti-rollback `anchors.json`), §15.1 row S (`from_subscribed` catalog-backed).
  - *confusion de domaine de signature* → **couverte par construction ET par un corpus de
    tests canonique** : `canonical.rs:412` `different_domains_yield_different_bytes_for_same_value`,
    `node_directory.rs:549` (bytes) + `:566` `node_directory_cross_domain_signature_rejected`
    (signature refusée), `shard_plan.rs:874` et `:1090` (les deux sens), `age_witness.rs:372`.
    Le commentaire `node_directory.rs:551-554` énonce la doctrine : l'inégalité d'octets est
    NÉCESSAIRE, le test signature-refusée est SUFFISANT. Aucune row STRIDE n'existe.
  - *rejeu* → non applicable en réseau (artefact content-addressed immuable). Le rejeu
    pertinent est la re-installation ; la doctrine est déjà gelée
    (`prompt_agent_packs_protocol_research.md:377` « Pas d'auto-update silencieux. Chaque
    version est un nouveau hash. ») et l'enforcement est routé **Phase X**
    (`sprint83_plan.md:1567-1603`, gate `G-LIFECYCLE`).
  - *downgrade* → le seul précédent réellement opposable est **stateful** :
    `iroh_runtime.rs:471-478` (step 8, `list_revision() <= stored` → `RevisionRollback`).
    Le check `version != FORMAT_VERSION` de `node_directory.rs:273-278` est un gate de
    **compréhension de format**, pas une défense anti-downgrade — le fichier le dit
    (`:270-271` « Revision rollback protection is NOT checked here »).
  - *TOCTOU* → classe connue mais **ouverte** (SI-12, `:1645-1652`, durcissement re-routé
    slot rig-chaud) ; seul le getter-TOCTOU bridge est FERMÉ (`:1689`, S83 Phase C).
    F ne livre aucun chargeur filesystem ⇒ note P3, pas un adapts.
  - *secret embarqué* → un scanner existe déjà et est **relâchable** :
    `crates/sbfb-factory/src/secret_scanner.rs:19-31` (3 patterns : AKIA, ghp_/gho_/ghs_/ghr_,
    PEM), gate `gates.rs:270 run_gate_fg6_secrets`, `docs/factory/FACTORY_GATES.md:152,293`
    (« relâchables via `--skip-gates` »). Ces 3 patterns ne couvrent pas les RÉFÉRENCES
    nominatives de credentials exigées par kickoff:295-298.
  - *exfiltration / SSRF* → §13.1 (`:783-817`) couvre le scan statique publish-time des
    ASSETS d'app sous la CSP du bac à sable et **assume ses faux-négatifs** (`:808` « Le
    gate ne pretend PAS prouver l'absence d'exfiltration »). La défense SSRF réelle
    appartient à la **Phase P1** (`sprint83_plan.md:1172`, gate `G-MCP-SSRF`).
  - *digest partiel / signature sur un sous-ensemble* → **pattern maison déjà armé** dans
    le même crate : `factory_profile.rs:316-320` (`MissingSource`/`DuplicateSource`,
    « fail-closed construction: no silent first-match »), `:346-352` (`StaleReason::Malformed`
    = « not exactly `SourceId::ALL` in order (empty, partial, duplicated, extra, or
    reordered) … must never pass fail-closed »), `:571-591` (`extract()` itère
    `SourceId::ALL`, jamais la map désérialisée), tests `:857-900` et `:1113-1123`.
- **Aucun des 5 fichiers `docs/security/*` à front-matter `triggers_revalidate` ne se
  déclenche sur F** (F n'ajoute ni route loopback, ni dep upstream). En revanche
  `THREAT_MODEL.md` n'a pas de front-matter : sa règle d'évolution est §18 (`:1698-1712`),
  dont la cadence est le SPRINT (point 5), pas la phase.
- **Précédent in-sprint de report explicite** : Phase D (`e8180b8`) ne touche pas le
  THREAT_MODEL ; Phase E (`6de49b1`) écrit au body « THREAT_MODEL : aucune section neuve,
  report EXPLICITE (E est hermetique …) — un silence aurait ete un scope cut non declare ».
  La norme in-sprint est **déclarer explicitement**, pas « section obligatoire ».

### Not evidenced (S3)

- Aucun mécanisme de RÉVOCATION d'un pack ou d'une clé de publication de pack n'est trouvé,
  ni dans le plan F, ni dans D2, ni dans le §17 du research pack. `DOMAIN_KEY_ROTATION_V1`
  existe (`canonical.rs:182`) mais son applicabilité aux packs n'est pas vérifiée.
- Le modèle de CONFIANCE de la clé de publication d'un pack (PKI ? liste de curateurs
  signée ? TOFU ?) n'est evidencé nulle part ; le research pack §17.1 n'énonce qu'une
  séparation de rôles. Sans lui, la menace « substitution de pack » ne peut pas être
  entièrement évaluée.
- Le partage de responsabilité F/AG sur le corpus hostile n'est pas evidencé : AG prévoit
  `fuzz_pack_import.rs` + `malicious_pack_refused.rs` ; on ne peut pas déterminer si le
  corpus de F doit être fuzzé dès F ou seulement testé unitairement.

---

## 6. Scan S4 — Invariants wire / frontières (verdict de scan : PLAN-ADAPT)

### Faits établis

- **30 constantes de version sous le regex du dépôt**, toutes à 1 sauf
  `INVITE_FORMAT_VERSION = 2` (`nexus-worker-core/src/invite.rs:73`) et
  `BLOB_VERSION = 0x01` (`nexus-core-rs/src/keystore.rs:108`). La seule amorcée en Phase E
  est `RUN_EVENT_FORMAT_VERSION = 1` (`sbfb-workflow-engine/src/event.rs:33`, LOCALE) —
  vérifié : c'est l'unique constante de version des deux crates `sbfb-workflow-*`.
  Ce n'est pas un census exhaustif des versions wire : hors regex existent
  `JSONRPC_VERSION = "2.0"` (`nexus-executor/src/ipc.rs:10`,
  `nexus-shell-daemon-core/src/ipc_broker.rs:16`) et la version portée par l'ALPN
  (`nexus-core-rs/src/node.rs:70 SEED_ALPN = b"sbfb/seed/0"`). **F ne doit en bumper aucune.**
- **Le gel du census est un gate SHELL, pas un test Rust** : aucun test n'observe le census
  (les seules occurrences dans `crates/**/*.rs` sont des doc-comments). Conséquence :
  `cargo nextest` restera vert pendant que `check-frontier-contracts.sh` rougit.
  Le gate n'est **pas** câblé au commit (aucun hook `.claude/hooks/*` ni
  `.git/hooks/pre-commit` ne l'invoque) : il mord en CI et via le critère F.4.
- **Forme réelle d'une étiquette de frontière** : le parseur (`:133-146`) prend toute ligne
  contenant le préfixe `// FRONTIER: `, `name` = premier token, puis les tokens `domain=`
  et `version=` **dans un ordre quelconque**, tokens supplémentaires et préfixe de ligne
  tolérés. Les deux tokens sont obligatoires (`:148-152`). Chacun doit apparaître comme
  sous-chaîne littérale `const <NOM>` quelque part sous `crates/` (`:153`, `:157`, via
  `grep -rqF` sans filtre d'extension) — ce n'est **pas** une résolution de symbole.
  Il faut en outre un `schema_for!(<name>)` réel (`:161`) ou une exemption
  `// FRONTIER-NO-SCHEMA: <name>` (`:163`). **Piège mécanique** : F.2 ne nomme AUCUNE
  constante de version pour le pack ⇒ une étiquette
  `// FRONTIER: SbfbPromptAgentPack domain=DOMAIN_WORKFLOW_PACK_V1 version=…` échouerait
  à `:157`. Le volet schéma est déjà satisfait (`contracts.rs:99`).
  Étiquette unique existante : `crates/nexus-core-rs/src/shard_plan.rs:188` ; garde
  anti-suppression `:175-178` ; gate mesuré `[1 tagged]`.
- **État réel du crate cible après Phase D** : 3 modules, 8 fichiers, ~4547 lignes.
  `lib.rs:75-77` = `pub mod accessibility_profile; pub mod contracts; pub mod factory_profile;`.
  5 des 7 identités restent des stubs `{}` vides. **Point d'insertion mécanique** (précédent
  Phase D `e8180b8`) : corps riche dans un module NOUVEAU (`pub mod pack;`), et
  `contracts.rs:41-44` devient un `pub use crate::pack::SbfbPromptAgentPack;` — ce qui
  préserve l'entrée 1/7 et reste cohérent avec les noms `pack::tests::*` de F.3.
- **La charte est écrite à trois endroits** : `Cargo.toml:3` (description), `Cargo.toml:15-18`
  (commentaire blake3), `src/lib.rs:9` (module-doc) ; le volet FS est énoncé séparément
  (`factory_profile.rs:573` « this crate links no filesystem »). Un scanner pur sur octets
  et un parseur JSON borné ne la violent pas. Si un parseur ZIP devait entrer, le précédent
  de production à copier est `blob_serve.rs:103` ou `fork.rs:216`
  (`ZipArchive::new(Cursor::new(bytes))`) — mesuré : **aucun** `ZipArchive::new(File::open(..))`
  n'existe dans le dépôt, les 10 sites sont déjà tous en mémoire.
- **`schema_snapshot_stub` est FAIL-OPEN au bootstrap** (`contracts.rs:190-200` : self-write
  + `return` si `UPDATE_SNAPSHOTS` est set OU si le fichier est absent/vide), contrairement
  aux trois autres goldens du crate (`accessibility_profile.schema.json`,
  `accessibility_fixtures.json`, `factory_profile.golden.json`, cf. `factory_profile.rs:786-793`
  « STRICT (NOT fail-open … deleting the golden cannot self-heal »). Mesuré sur copie :
  supprimer `contracts.schema.json` laisse le test VERT et régénère le fichier. Aucun script
  CI ne référence `contracts.schema.json` ⇒ ce test est son SEUL gate. Nuance : le fail-open
  est le patron canonique hérité de `nexus-core-rs` (`task_response.rs:290`, `shard.rs:644`),
  revendiqué en doc-comment `contracts.rs:176-180` — le durcir est une divergence assumée.
- **Trois blocs de prose affirment aujourd'hui que le census reste intact** :
  `sbfb-workflow-core/src/lib.rs:47-49`, `sbfb-workflow-core/src/contracts.rs:20-22`,
  `sbfb-workflow-engine/src/lib.rs:105-107`. Seule la clause « the frozen `DOMAIN_*_V1`
  family census stays untouched » devient fausse ; les clauses voisines (« no `DOMAIN_*_V1`
  constant is declared here » / « the crate declares no `DOMAIN_*` signing family ») restent
  VRAIES si les constantes vont dans `nexus-core-rs`. **Aucun gate n'attrape cette classe**
  (le check (1) anti-promise ne vise que les promesses futures ; le census compte des
  consts, pas de la prose).
- **La parité JCS Rust/TS n'a aucun socle TS** : 0 canonicaliseur, 0 dep JCS (mesuré). En
  revanche blake3 est disponible et exécutable (cf. S1b). Les seuls stringifiers présents
  (`fast-json-stable-stringify`, `json-stable-stringify-without-jsonify`) ne sont **pas**
  RFC 8785 (tri de clés ≠ échappement Unicode + format numérique JCS).
- **Le miroir TS de la Phase C** (`web/src/bridge/authority.ts`, 176 l) ne fait aucune
  canonicalisation ni digest : algèbre d'ensembles + comparaison temporelle (`:57-60`).
  MAIS **le harnais de fixtures golden cross-langage EXISTE DÉJÀ et est vert** :
  `web/src/a11y/__tests__/profile_resolution.test.ts:228-265` lit l'oracle Rust partagé
  `crates/sbfb-workflow-core/src/accessibility_fixtures.json` via
  `pathResolve(process.cwd(), "../crates/…")`, et `:275` / `hub_scripts.test.ts:22` lisent
  le golden schemars `accessibility_profile.schema.json` (mesure Phase B : 56/56 PASS).
  Un gate de parité cross-langage avec **self-test de non-vacuité** existe aussi :
  `scripts/check-frontier-contracts.sh:272-311` (bridge-parity, `[16 methods]`).
  Le coût résiduel de F est donc **la seule primitive JCS TS**, pas le harnais.

### Not evidenced (S4)

- Le nombre exact de constantes `DOMAIN_*` que F ajoutera (cf. §4, « au moins » 8 rôles) ⇒
  la valeur cible de `DOMAIN_CENSUS_FROZEN` n'est pas déterminable avant arbitrage.
- Aucune mutation réelle du census n'a été jouée (interdiction d'écrire) : le passage au
  rouge est une lecture mécanique de `:190-199` (comparaison `-ne` sur un entier) —
  **INFERENCE** à confiance très élevée, non exécutée.
- La nature exacte des entrées ZIP hostiles attendues (§9.2 layout, §19.2 bornes) reste
  non chiffrée.

---

## 7. Passe adversariale — ce qui a été RÉFUTÉ

La passe adversariale a re-mesuré chaque finding. **8 findings survivent** tels quels ou
resserrés ; **28 sont réfutés** — presque tous pour sur-affirmation (règle 3 : vrai sur un
cas particulier, énoncé comme général) ou pour ancre de ligne fabriquée. Ces réfutations
sont la valeur d'audit de la phase : elles doivent empêcher le code de sur-corriger.

- **« Divergence float non fini ⇒ parité byte-identique structurellement impossible,
  BLOCKING »** → **RÉFUTÉ (déclassé P2)**. Le mécanisme est vrai et re-mesuré, mais (a) la
  lib TS visée par le plan est `jcs` (`plan:599`), pas `canonicalize` — le comportement de
  `jcs` sur le non-fini est Not evidenced ; (b) le cas est **inatteignable** aujourd'hui :
  `crates/sbfb-workflow-core/src/*.rs` ne contient aucun `f64`/`f32` (0 hit) et un JSON sur
  disque ne peut pas porter `NaN`/`Infinity`. Sévérité réelle : préventif, pas bloquant.
- **« `subject[]` in-toto porte une sémantique ANY ⇒ un pack au `content_digest` altéré
  passerait »** → **RÉFUTÉ (strawman)**. Les 3 citations in-toto sont exactes, mais aucun
  document d'autorité ne pose les 6 digests dans une DigestSet : le research pack §9.3
  (`:767-782`) les pose comme **champs de pack** (0 occurrence de « subject » dans tout le
  fichier), et D2 gèle déjà la forme sûre `subject[] = [{name, digest:{blake3}}]`
  (kickoff:289-291) — identique au précédent déjà implémenté
  `crates/nexus-core-rs/src/attestations/contributor.rs:128-140,157-158` (« Subject array
  (exactly 1 entry in v1) », « SBFB v1 uses a single entry "blake3" → … »). Reste utile :
  écrire noir sur blanc que la vérification SBFB est une CONJONCTION, divergente de la
  sémantique ANY, et que blake3 n'est pas un algo standardisé in-toto (« à la in-toto »,
  jamais « standard in-toto » — déjà acté `sprint83_phase_a_preflight.md:43-46`).
- **« `json-canonicalize` diverge de la RFC sur `undefined` dans les tableaux »** →
  **RÉFUTÉ**. RFC 8785 ne contient **pas** le mot « undefined » (0 occurrence mesurée) ;
  par défaut `json-canonicalize` fait exactement ce que fait `canonicalize`. Ne PAS écarter
  une lib sur ce motif. Reste valide : exclure `@tufjs/canonical-json` (OLPC, pas RFC 8785)
  et verrouiller le format par construction (pas de membre optionnel dans les structures
  signées).
- **« Aucun blake3 côté JS ⇒ 2 deps npm neuves / parité non réalisable à deps
  constantes »** → **RÉFUTÉ par exécution**. `@noble/hashes 1.8.0` est présent en
  production dans `web/`, expose `blake3.js`, et produit le bon digest. La parité complète
  (chaîne canonique ET digest) a été obtenue en mesure avec **zéro paquet npm nouveau**,
  via un canonicaliseur JCS first-party de ~12 lignes + `@noble/hashes/blake3`.
  L'arbitrage réel est : déclarer `@noble/hashes` en **devDependency** (0 nouveau nœud de
  lock, hors bundle, hors `npm run size` qui ne mesure que `dist/assets/*`) vs phantom-dep
  (à proscrire).
- **« `serde_jcs` route TOUS les entiers via f64 »** → **RÉFUTÉ en généralité** : vrai pour
  i64/u64 (mesuré), FAUX pour i128/u128 qui échouent bruyamment
  (`Err("u128 is not supported")` — `write_i128`/`write_u128` sont du code mort). Le risque
  à couvrir est celui des 64 bits.
- **« Un digest sémantique invariant au réordonnancement est aussi dur que l'isomorphisme
  de graphes »** → **RÉFUTÉ**. RDFC-1.0 conditionne explicitement la dureté à l'absence
  d'identifiants globalement uniques (« those that contain nodes that do not have globally
  unique identifiers pose a greater challenge »). Avec des node ids explicites, l'invariance
  se ramène à un tri déterministe O(n log n). Le « défaut 4000 » n'est pas un défaut
  standard : c'est celui d'une seule implémentation (zkp-ld/rdf-canon) ; l'impl de
  référence JS n'en a pas.
- **« DOMPurify est la seule source de vecteurs SVG/Markdown »** → **RÉFUTÉ** : la page ne
  contient **aucune** mention de Markdown ni de « canvas » (le vecteur
  `<svg><canvas></canvas><textarea></textarea></svg>` cité est fabriqué), et d'autres
  corpus publics existent (PortSwigger, PayloadsAllTheThings). Le volet Markdown reste
  Not evidenced.
- **« Phase F crée la 1re menace SVG/Markdown du dépôt »** → **RÉFUTÉ** : `blob_serve.rs:223`
  sert du SVG depuis le Sprint 12 (`32a1dca`), couvert par classe (iframe sandbox origin
  opaque + CSP `connect-src 'none'`, `THREAT_MODEL.md:19`, `:104-107`). Et F ne crée aucune
  surface de rendu : 0 `dangerouslySetInnerHTML` dans `web/src` et
  `tools/factory-operator/src`, 0 dep markdown/sanitizer, `plan:609` T1 =
  `N-A-no-frontend-change`. Le gap réel (rendu de ressource de pack sur la surface HÔTE
  first-party, hors CSP du bac à sable) est à router **R/S/T**, pas à bloquer F.
- **« Il n'existe aucune section d'import d'artefact tiers dans le THREAT_MODEL »** →
  **RÉFUTÉ** : §10 Feed, §15/§15.1, §12 ProofCard modélisent déjà des ingestions
  d'artefacts signés émis par des pairs hostiles. F crée la première surface **de sa
  classe**, pas la première surface tout court — et doit RÉUTILISER le patron canon
  (vérification avant usage, caps taille/nombre, gate d'admission).
- **« La confrontation THREAT_MODEL vs §19 n'est planifiée QU'en AH »** → **RÉFUTÉ** :
  elle est aussi en AJ1 (`plan:2126,2139,2291`) et le kickoff (`:145`, `:819`) la prescrit
  comme mécanisme **G8/G9 par phase** — ce préflight EST ce G8. `AM-3` (kickoff:536) route
  explicitement « THREAT_MODEL vNN + PATTERNS §P76+ **au wrap-up** ». Le plan est conforme.
- **« Phase F doit ajouter une §19 STRIDE au THREAT_MODEL »** → **RÉFUTÉ (confusion de
  document)** : le §19 visé est celui du research pack, il existe déjà. La règle §18 point 3
  (`THREAT_MODEL.md:1705-1707`) parle de « §5.x STRIDE + §6 LINDDUN + ligne §2 Assets +
  §4 DFD », jamais d'une §19.
- **« Aucune primitive anti-rejeu n'est prévue ⇒ le plan est muet »** → **RÉFUTÉ** : la
  doctrine est gelée (`prompt_agent_packs_protocol_research.md:377,531` — pas d'auto-update,
  chaque version est un nouveau hash), le format porte déjà `schema_version` + `version` +
  provenance `timestamp`, et l'enforcement du cycle est en Phase X (`G-LIFECYCLE`).
- **« Le rejet `version != 1` est une défense anti-downgrade »** → **RÉFUTÉ par le code
  lui-même** (`node_directory.rs:270-271`). C'est un gate de compréhension de format.
- **« Digest partiel : risque structurel PROPRE au design D2, non couvert nulle part »** →
  **RÉFUTÉ** : le même crate ferme déjà ce vecteur pour son autre jeu de 6 digests
  (`factory_profile.rs:316-320`, `:346-352`, `:571-591`, tests `:857-900`, `:1113-1123`,
  Phase D). L'action est de RÉPLIQUER un pattern existant, pas d'inventer.
- **« zip-slip : la seule row voisine est le coordinator »** → **NUANCÉ** : vrai pour le
  THREAT_MODEL, mais la garde code est vivante et testée (`fork.rs:244`, `:566` mesuré
  vert) ; `MAX_ARCHIVE_ENTRIES` date de S75 G (`8b53c38`), pas de S74 B.
- **« Le crate n'a AUCUNE dépendance crypto »** → **RÉFUTÉ** : `blake3` (`Cargo.toml:19`)
  est un hash cryptographique, utilisé `factory_profile.rs:419`. Et la charte n'interdit
  ni `serde_jcs` ni `ed25519-dalek` (pur CPU) — il n'y a **pas** d'« extension de charte à
  assumer », seulement une justification de portée à écrire.
- **« Plan et kickoff se contredisent sur l'emplacement des DOMAIN »** → **RÉFUTÉ entre
  documents** : `kickoff:316-322` sépare explicitement « DOMAIN → nexus-core-rs » et
  « schemas/digests/goldens → sbfb-workflow-core », et `plan:105` (D2) + `plan:582` (F.1)
  disent la même chose. La contradiction est **intra-plan** et typographique : F.2
  (`:591-594`) élide le chemin de crate d'un parenthétique « (`canonical.rs`) ». Reste un
  vrai point de placement à trancher (cf. §8, B2).
- **« Aucun gate mécanique n'attrape la prose de census »** → **RÉFUTÉ pour le compteur**
  (le tripwire voit un net-add ; `PATTERNS.md:4081-4085` ne concède l'aveuglement que pour
  un net-zero add+remove ou un rename), **CONFIRMÉ pour les 3 doc-comments** du crate.
- **« `cargo tree -d -p sbfb-workflow-core` vérifie l'atterrissage sur ed25519 2.2.0 »** →
  **RÉFUTÉ (tautologie)** : mesuré `warning: nothing to print.` aujourd'hui, et même sortie
  dans le cas mauvais. Contrôle discriminant : `cargo tree -p … | grep ed25519`.
- **« EXIT 4 = gate vert vacueux »** → **RÉFUTÉ (auto-contradictoire)** : EXIT 4 est un code
  non nul, donc ROUGE. C'est un garde-fou, pas un faux positif.
- **« Le plan F ne mentionne aucune option pour la parité TS »** → **RÉFUTÉ** : `plan:599`
  nomme déjà « (`serde_jcs` / `jcs`) ».
- **« La déserialisation pré-signature existante est BORNÉE »** → **RÉFUTÉ** (cf. §5,
  nuance factuelle). Écrire la garantie F comme un RENFORCEMENT introduit par F, jamais
  comme une description de l'existant — sinon F documente une propriété que le dépôt n'a pas.
- **« `deny_unknown_fields` cause l'erreur sur clé dupliquée »** → **RÉFUTÉ par mesure** :
  `Plain` et `Deny` erreurent identiquement ; c'est le `derive` qui rejette. Le vrai
  vecteur est sur les champs de type MAP / `Value`.
- **« Le précédent de test cross-domaine est `verify_rejects_attribution_mismatch`
  (node_directory.rs:431) »** → **RÉFUTÉ** : ce test porte sur l'ATTRIBUTION. Le précédent
  exact est `node_directory.rs:566` + sa paire `:549`, et `shard_plan.rs:874`/`:1090`.
- **« `DOMAIN_RUN_EVENT_V1` entre en tension avec la Phase E »** → **RÉFUTÉ** : le
  preflight E le route explicitement vers F (`:144`).
- **« La supply-chain est rouge (CI-1) »** → **RÉFUTÉ, périmé depuis `ce99c97`** :
  `cargo deny check advisories` et `… licenses` passent (mesuré).
- **« F.2 exige un parseur ZIP dans workflow-core »** → **RÉFUTÉ** : F.2 (`:595`) demande
  un parseur **borné + `deny_unknown_fields` sur le profil signé** (JSON/JCS) ; « zip »
  n'apparaît qu'à `:600` comme classe de fixtures.
- **« `ZipArchive::new(File::open(..))` est un anti-pattern présent dans deploy.rs:958 »** →
  **RÉFUTÉ** : `deploy.rs:956-958` utilise déjà un `Cursor` ; aucun `File::open` zip
  n'existe dans le dépôt.
- **« Aucun harnais de fixtures golden cross-langage n'existe »** → **RÉFUTÉ** :
  `profile_resolution.test.ts:228-265` (Phase B) le fait déjà, cwd-resolution comprise,
  et il est vert.

---

## 8. Contraintes mécaniques pour l'implémentation

Ordre à respecter. Le code suit **ce préflight**, pas le texte brut du plan.

### 8.1 Blocants — à trancher AVANT le premier `Edit`

- **B1 — Fermer la LISTE des `DOMAIN_*_V1` ajoutés par F, avant toute autre décision.**
  F.2 en nomme 4 suivies d'un « … » et le kickoff parle d'« au moins » 8 rôles §17.1 dont
  le plan attribue le graphe complet à la **Phase M** (`plan:984,991`). Écrire l'énumération
  close (recommandation par défaut : **les 4 nommées**, `DOMAIN_WORKFLOW_PACK_V1`,
  `DOMAIN_COMPILED_PLAN_V1`, `DOMAIN_RUN_EVENT_V1`, `DOMAIN_PACK_INSTALL_RECEIPT_V1`, avec
  route explicite des rôles restants vers M), et **une décision D8 écrite par famille**
  (schéma généré + snapshot, OU rationale no-schema motivée) dans le body.
- **B2 — Trancher l'emplacement des constantes.** Le plan est ambigu (F.1 `:582` renvoie à
  `canonical.rs:77-332` = `nexus-core-rs` ; F.2 `:591-594` les liste sous
  `crates/sbfb-workflow-core`). **Recommandation** : append dans
  `crates/nexus-core-rs/src/canonical.rs` conformément au kickoff `:316-322` (autorité
  déclarée `plan:100`), en **n'introduisant AUCUNE dep `nexus-core-rs` dans
  `sbfb-workflow-core`** (elle tirerait iroh + tokio et détruirait la charte). Corollaire :
  la vérification Ed25519 du pack est portée soit par un helper local de `workflow-core`
  déclarant ses propres domaines (précédent `nexus-trace-core/src/lib.rs:29`,
  `signed.rs:68`, crate sans dep `nexus-core-rs`), soit par le crate hôte. **Décider et
  écrire le choix dans le body** ; ne pas laisser les deux lectures cohabiter.
- **B3 — Rafraîchir le census dans le MÊME commit, 6 surfaces.** Le critère F.4 exige
  `check-frontier-contracts.sh` exit 0 ; le census est à 25 et le gate compare strictement.
  Éditer : (1) `scripts/check-frontier-contracts.sh:190` `DOMAIN_CENSUS_FROZEN` ;
  (2) prose header (2) `:44-47` (« 25 families, 22 without a generated schema ») ;
  (3) commentaire (2b) `:180-189` (25 / 3 schématisées / 22 unschematised) ;
  (4) `docs/rust/PATTERNS.md:4068,4072,4077` (mêmes trois nombres + `DOMAIN_CENSUS_FROZEN=25`) ;
  (5) `crates/sbfb-workflow-core/src/lib.rs:47-49` et (6) `…/src/contracts.rs:20-22` —
  la clause « the frozen `DOMAIN_*_V1` family census stays untouched » devient FAUSSE
  (les clauses voisines « no `DOMAIN_*_V1` constant is declared here » restent vraies sous
  B2). Si F ne touche pas `sbfb-workflow-engine`, `engine/src/lib.rs:105-107` doit néanmoins
  voir sa seconde proposition corrigée. **Les surfaces (5) et (6) sont invisibles au gate.**
- **B4 — Aligner le nommage des tests sur la gate matrix, OU amender la matrice.**
  `sprint83_gate_matrix.md:69` enforce `cargo nextest run -p sbfb-workflow-core import_hostile`
  ⇒ EXIT=4 (mesuré) avec les noms de F.3. Choix : nommer le module hostile pour que le
  filtre matche (ex. `pack::import_hostile::…`, en conservant les noms de tests du plan
  dans ce module), OU amender la ligne 69 dans le même commit. Dans les deux cas,
  **MESURER et inscrire dans le body le nombre de tests sélectionnés** (précédent Phase E
  pour `G-STORE`). Vérifier symétriquement qu'aucun test de F ne pollue un filtre d'une
  autre phase.
- **B5 — Installer une porte de canonicalisation UNIQUE dans `sbfb-workflow-core`.**
  Le crate n'a ni `serde_jcs` ni garde `null`. **Ne PAS hisser `reject_nulls` dans
  `nexus-core-rs::canonical_bytes`** : mesuré, `crates/nexus-core-rs/src/task.rs:315-316`
  expose `#[serde(default)] pub required_runtime: Option<RuntimeTuple>` **sans**
  `skip_serializing_if`, et `task_canonical_bytes` (`:39-50`) ne retire que
  `redundancy_factor` ⇒ tout `Task` sans cohorte signe aujourd'hui sur
  `"required_runtime":null` et **cesserait de signer**. Donc : porte locale à
  `workflow-core`, calquée sur `sbfb-workflow-engine/src/hash.rs:266-300` (une seule
  fonction `canonical_bytes` traversée par TOUT ce qui digère/compare), avec le test qui
  **DÉMONTRE** la collision (jamais la suppose) et qui exerce le chemin `serde_jcs`
  **direct**, pas seulement le chemin `to_value` (le seul test in-repo,
  `store_spike.rs:498-508`, n'exerce que ce dernier).
- **B6 — Ne poser une étiquette `// FRONTIER:` que si une constante de version existe.**
  Le gate exige que `const <VERSION_CONST>` apparaisse littéralement sous `crates/`
  (`check-frontier-contracts.sh:157`). F.2 n'en prévoit aucune. Deux voies : déclarer un
  `pub const PACK_FORMAT_VERSION: u16 = 1;` (cohérent avec « Versionné dès la v1, leçon
  n8n », kickoff:283, et avec la réservation du champ monotone, cf. §9) **et** poser
  l'étiquette ; ou satisfaire AM-3 comme les Phases B/D/E l'ont fait (commentaire de
  provenance en prose + golden strict) avec exception tracée. **Recommandation** : déclarer
  la constante et poser l'étiquette — le pack EST une frontière signée, et l'exception B
  était motivée par un contrat LOCAL, pas signé.

### 8.2 Adaptations obligatoires

- **A1 — Golden de schéma.** Remplir le stub casse `contracts.schema.json` : refresh via
  `UPDATE_SNAPSHOTS=1 cargo test -p sbfb-workflow-core schema_snapshot_stub`. Le pack reste
  l'entrée **1/7** — ne pas ajouter d'identité au registre (`CONTRACT_NAMES: [&str; 7]`,
  `contracts.rs:133`, assert `:147`). Conserver `deny_unknown_fields` sur la racine ET le
  poser sur **chaque type imbriqué neuf**. Arbitrage à rendre : durcir `schema_snapshot_stub`
  en STRICT (calque `factory_profile.rs:786-793`) — le golden que F va massivement grossir
  est le seul fail-open des quatre, et sa suppression le régénère en vert. Divergence
  assumée d'avec le patron `nexus-core-rs`, à écrire si retenue.
- **A2 — Placement du code.** Nouveau module `pub mod pack;` dans `lib.rs`, `contracts.rs`
  ne garde que la ré-export (`pub use crate::pack::SbfbPromptAgentPack;`). Précédent
  Phase D `e8180b8`. SPDX `AGPL-3.0-or-later` dans les 5 premières lignes de chaque `.rs`
  neuf (`scripts/check-spdx.sh:19,25,31`).
- **A3 — Deps.** Ajouter `serde_jcs = { workspace = true }` et, si la vérification vit dans
  le crate, `ed25519-dalek = { workspace = true }` — **0 nouveau nœud de lock**. Re-motiver
  chacune par commentaire au format du bloc blake3 (`Cargo.toml:15-18`). Contrôle
  discriminant après ajout : `cargo tree -p sbfb-workflow-core | grep ed25519` doit rendre
  exactement `ed25519-dalek v2.2.0` (jamais `3.0.0-rc.0`). Ne PAS ajouter `zip` (non requis
  par F.2).
- **A4 — Parité JCS/TS, à deps npm quasi nulles.** Écrire le canonicaliseur JCS en
  **first-party test-util TS** (tri `Object.keys().sort()` = unités de code UTF-16 =
  exigence RFC 8785 ; `JSON.stringify` pour les scalaires) et déclarer `@noble/hashes` en
  **devDependency** (déjà au lock en 1.8.0 via `eciesjs` — 0 nouveau nœud, hors bundle,
  hors `npm run size` qui ne mesure que `dist/assets/*`). **Ne PAS** l'importer sans le
  déclarer (phantom-dep). L'invariant projet visé est « 0 dep **runtime** ajoutée au
  daemon/shell » (`plan:31`) : une devDependency de test n'y touche pas — l'écrire
  explicitement dans le body. Réutiliser le harnais existant (`profile_resolution.test.ts:235`
  `pathResolve(process.cwd(), "../crates/sbfb-workflow-core/src/…")`) plutôt que d'en créer
  un. La parité doit être testée sur des vecteurs **adverses** RFC 8785 (clé non-BMP,
  emoji, échappements, `1e+21`, `1e-7`, `-0`), pas sur un golden nominal.
- **A5 — Formuler l'ordre verify/parse comme le dépôt le fait réellement.** Écrire :
  « déserialisation **BORNÉE** (cap de taille AVANT `from_slice` + `deny_unknown_fields` +
  caps de profondeur et de cardinalité) → vérification signature Ed25519 + digests → et
  SEULEMENT ensuite résolution, rendu, mapping, I/O », et la présenter comme un
  **renforcement introduit par F**, pas comme l'existant. Trois bornes distinctes à tester
  séparément : (i) taille avant parse — à implémenter, rien ne la fournit ; (ii) profondeur
  — héritée de `serde_json` (`de.rs:38`), à documenter et à ne **jamais** désactiver ;
  (iii) cardinalité — à implémenter. Le test `signature_before_parse` doit être
  **DISCRIMINANT** : inverser l'ordre doit le rendre rouge.
- **A6 — Test jumeau du mode de panne cosign.** Ajouter au corpus au moins une entrée
  portant une signature Ed25519 **VALIDE** sur un corps qui échoue au parse borné /
  `deny_unknown_fields` / au type de racine attendu ⇒ **REFUS TERMINAL**, jamais un statut
  « vérifié » dégradé ni un chemin de repli.
- **A7 — Test cross-domaine dans les DEUX sens.** Précédent exact : `node_directory.rs:549`
  (bytes diffèrent, condition nécessaire) **+** `:566`
  `node_directory_cross_domain_signature_rejected` (signature sous un autre DOMAIN refusée,
  condition suffisante), et `shard_plan.rs:874`/`:1090`. Copier aussi le commentaire
  nécessaire/suffisant (`node_directory.rs:551-554`).
- **A8 — Clés dupliquées : tester sur les champs de type MAP.** Le refus doit être prouvé
  sur les maps / objets libres du pack (capacités, digests, métadonnées), pas sur un champ
  de struct où le `derive` rejette déjà — sinon `hostile_corpus_refused` est vert pour une
  mauvaise raison. Remède sans nouveau nœud : `serde_with` 3.20.0 (`Cargo.lock:7826`),
  `MapPreventDuplicates`.
- **A9 — Entiers I-JSON.** Interdire par schéma tout entier hors
  [-9007199254740991, 9007199254740991] dans les structures signées (ou représentation en
  chaîne), + vecteur hostile `integer > 2^53` attendu en refus. Motif mesuré : deux valeurs
  distinctes partagent aujourd'hui une sortie canonique.
- **A10 — Exhaustivité et complétude des 6 digests, en répliquant le pattern maison.**
  Deux tests à ajouter à F.3 : (i) **exhaustivité** — un fichier ajouté à l'archive hors de
  tout digest fait échouer la vérification ; (ii) **complétude** — un pack dont un digest
  est absent/nul est REFUSÉ (pas de fail-open par champ optionnel). Modèle :
  `factory_profile.rs:571-591` (itérer l'énumération canonique, **jamais** la map
  désérialisée), `:346-352` (`Malformed` = vide/partiel/dupliqué/extra/réordonné),
  tests `:857-900` et `:1113-1123`.
- **A11 — `subject[]` : forme et sémantique explicites.** Entrées **nommées**, DigestSet
  **mono-algo blake3** (précédent `contributor.rs:128-140`, « exactly 1 entry in v1 ») ;
  les 6 digests sémantiques sont des **champs du pack**, pas des clés d'algorithme.
  Documenter que la vérification SBFB est une **CONJONCTION** (tous contraignants),
  divergente de la sémantique ANY d'in-toto, et que blake3 n'est pas standardisé in-toto
  (« à la in-toto »). Garder le payload canonicalisé **séparable** de la signature pour que
  la clause D2 « DSSE atteignable plus tard SANS casse » (kickoff:290-292) reste vraie.
- **A12 — `graph_semantic_digest` : identifiants explicites + ordre total.** Ne PAS définir
  un étiquetage canonique de graphe. Poser des node ids stables pinnés par le pack et un
  tri lexicographique déterministe des arêtes ; l'invariance au déplacement devient triviale
  et bornée. Cohérent avec la Phase H (`plan:689`, IDs d'occurrence uniques). Aucune limite
  de calcul ni vecteur « graphe poison » n'est alors nécessaire.
- **A13 — Corpus hostile : réutiliser, puis étendre.** Réutiliser les 6 vecteurs de
  `fork.rs:566` et les bornes `fork.rs:68` / `blob_serve.rs:39,42,126-130` plutôt que les
  réinventer. Axes à AJOUTER : casse Windows (`A.json`/`a.json`), noms réservés Windows
  (`CON`, `NUL`, `AUX`, `COM1`), séparateur `\` en nom d'entrée, Unicode confusables
  (UTS #39), **ratio** de compression distinct de la taille absolue, **longueur des chaînes**
  (clause §19.2(1) non reprise par le plan). Pour SVG/Markdown : n'importer que la LISTE de
  vecteurs (DOMPurify / PortSwigger / PayloadsAllTheThings), **jamais la logique** — le refus
  doit être un refus de **SCHÉMA** (média actif non admis), testé comme tel, et non un test
  de sanitizer. Si le pack v1 n'admet aucun média actif, l'écrire explicitement en F.2.
- **A14 — Clauses §19.2 non reprises : décider explicitement.** (7) « schémas `$ref` locales
  ou pinnées » et (9) « un import reste brouillon non signé jusqu'à revue et republication »
  ne sont couvertes ni par F, ni par AG, ni par S. Les expliciter en F ou les router
  nommément (AG) — jamais les laisser tomber en silence.
- **A15 — Scanner secrets/endpoints : statut d'autorité écrit.** Le poser comme
  **best-effort non autoritaire**, au même titre que FG6 qui est relâchable via
  `--skip-gates` (`FACTORY_GATES.md:152,293`), l'autorité fail-closed restant le compilateur
  §10.5 (Phase H). Le poser comme **déclaratif** côté endpoints (le pack DEMANDE, n'obtient
  rien — kickoff:281-282) et écrire en scope cut daté que la défense SSRF appartient à la
  **Phase P1** (`plan:1172`, gate `G-MCP-SSRF`). Noter que les 3 patterns existants
  (`secret_scanner.rs:19-31`) ne couvrent pas les RÉFÉRENCES nominatives de credentials
  exigées par kickoff:295-298.
- **A16 — THREAT_MODEL : une ligne explicite, jamais un silence.** Précédent Phase E
  (`6de49b1`). Deux options recevables : (a) **report explicite justifié** dans le body
  (« F est une lib de format : 0 route, 0 réseau, 0 rendu, import non actif ; la surface
  d'import vit en X/AG, le durcissement en AH, `THREAT_MODEL vNN` en AJ1 ») ; (b) si le
  préflight de review juge que la vérification d'un pack signé crée déjà une frontière de
  confiance, ajout conforme à la règle §18 point 3 (`:1705-1707` : §5.x STRIDE + §6 LINDDUN
  + ligne §2 Assets + §4 DFD), **sans** créer de « §19 ». Recommandation : (a), en nommant
  les 4 menaces non couvertes (substitution de pack, confusion de domaine, downgrade
  stateful, digest partiel) et leur phase propriétaire.
- **A17 — Réserver le champ monotone du pack.** Un champ de révision/version de pack doit
  exister **dès F** (sinon bump wire ultérieur) ; son enforcement stateful (floor
  anti-rollback, modèle `iroh_runtime.rs:471-478`) appartient à la phase qui installe
  (X, `DOMAIN_PACK_INSTALL_RECEIPT_V1`). Écrire ce partage comme scope cut daté.
- **A18 — Exclusion par le TYPE, pas par `skip`.** Aucun secret matérialisable dans le
  format (seul `secret_ref` opaque, kickoff:295-298) ; aucun champ ne peut porter de PII
  (champ **absent**, pas `#[serde(skip)]` — doctrine A9, `THREAT_MODEL.md:1683`). Interdire
  par schéma tout membre optionnel dans les structures signées, ou imposer
  `skip_serializing_if` systématique + refus des `null` (règle B5).
- **A19 — Prose : PROMISE_RE.** `check-frontier-contracts.sh:89` mord sur les formes
  anglaises (`Phase [A-Z0-9]+ (will|adds|ships)`, `lands in Phase`, `inert until Phase`) et
  scanne aussi `*.toml`. Les doc-comments Rust de ce dépôt sont en anglais : écrire
  « installing a pack is out of scope for this crate », jamais « Phase X will install ».
- **A20 — `§P76`** est le prochain numéro libre (espace commun rust+shell). Le pattern livré
  par F se rédige **au wrap-up du sprint**, pas dans le commit de phase.

### 8.3 Interdits

- Ne pas ajouter de dépendance `nexus-core-rs` à `sbfb-workflow-core`.
- Ne pas hisser `reject_nulls` dans `nexus-core-rs::canonical_bytes` (casse la signature
  des `Task` sans cohorte, `task.rs:315-316`).
- Ne pas réutiliser un `DOMAIN_*` existant pour une sémantique de pack (le gate dédoublonne
  par `sort -u` et laisserait passer ; c'est une violation de la doctrine `canonical.rs`).
- Ne pas bumper `FEED_FORMAT_VERSION` ni aucune des 30 constantes de version existantes.
- Ne pas ajouter d'identité au registre `CONTRACT_NAMES` (le pack est 1/7).
- Ne pas ajouter `zip` au crate (non requis par F.2), ni de dépendance runtime npm.
- Ne pas toucher `web/` au-delà des fixtures/tests de parité (T1 = `N-A-no-frontend-change`,
  `plan:609`).
- Ne pas écrire « signature vérifiée avant parse » sans le qualificatif « parse **riche** »
  et la borne préalable (A5), sous peine d'écrire une garantie fausse.
- Ne pas reconduire la claim périmée « supply-chain rouge (CI-1) ».

### 8.4 Invariants à vérifier (commande par invariant)

| Invariant | Commande | Attendu |
|---|---|---|
| census cohérent | `bash scripts/check-frontier-contracts.sh` | exit 0 (après B3) |
| census mesuré | commande verbatim `:191-192` | == nouveau `DOMAIN_CENSUS_FROZEN` |
| registre 7 | `cargo nextest run -p sbfb-workflow-core --locked` | exit 0, `len == 7` intact |
| gate F non vacueux | `cargo nextest run -p sbfb-workflow-core import_hostile --locked` | **≥ 1 test exécuté** (EXIT 4 = rouge) |
| charte crate | `grep -rn 'nexus-core-rs\|tokio' crates/sbfb-workflow-core/Cargo.toml` | vide |
| ed25519 stable | `cargo tree -p sbfb-workflow-core \| grep ed25519` | `ed25519-dalek v2.2.0` |
| 0 nœud lock neuf | `git diff Cargo.lock` | aucune version d'un paquet existant ne bouge |
| SPDX | `bash scripts/check-spdx.sh` | exit 0 |
| parité TS | `npm --prefix web run test:unit` | vert, fixtures golden lues depuis `crates/` |
| bundle intact | `npm --prefix web run size` | 6/6 (devDependency hors bundle) |
| docs-contrat | `bash scripts/check-factory-docs.sh` | exit 0 |
| dual-platform | `cargo fmt --all --check` ; `cargo clippy --workspace --all-targets --locked -- -D warnings` ; `cargo nextest run --workspace --locked` ; `cargo test --workspace --locked --doc` ; `cargo build -p nexus-shell-daemon --release` | verts Win natif **ET** Docker `sbfb-ci` |

---

## 9. Risques ouverts

- **R1 — Condition binaire non falsifiable en l'état.** F.4 exige « 100 % du corpus hostile
  refusé avant rendu » alors que (a) le corpus n'a pas de bornes numériques evidencées et
  (b) F ne livre aucun moteur de rendu. Reformuler la condition en « refusé avant toute
  résolution/mapping/rendu », et énumérer le corpus dans le commit pour que « 100 % » soit
  mesurable. Sinon la condition est vraie par construction et ne prouve rien.
- **R2 — Test vert pour une mauvaise raison.** Trois pièges mesurés : `hostile_corpus_refused`
  vert parce que le `derive` rejette (A8) ; `signature_before_parse` vert sans être
  discriminant (A5) ; `six_digests_stable` vert sans couverture d'octets (A10). Muter avant
  de croire chaque vert (memory `feedback_mutation_before_trust`).
- **R3 — Gate vert et prose fausse.** Le tripwire census ne compare qu'un entier : les
  surfaces B3(5) et B3(6) peuvent rester mensongères en commit vert. C'est exactement la
  classe `feedback_gate_discriminant`.
- **R4 — Nombre de domaines re-drifté.** Si B1 n'est pas fermé, le compteur re-drifte à la
  phase suivante — la dérive 21/22/23 que le tripwire existe pour tuer.
- **R5 — Parité TS non atteinte sur les cas limites.** La parité a été démontrée en mesure
  sur une fixture riche (clés Unicode, emoji, échappements, `1e+21`, `1e-7`, `-0`), pas sur
  les cas limites RFC 8785 (surrogates isolés, floats non finis, entiers hors f64). Ces cas
  doivent être **exclus par le schéma borné** du pack, sinon la parité est fragile.
- **R6 — Phantom-dependency npm.** Si `@noble/hashes` est importé sans être déclaré, le
  test casse au premier `npm prune` ou bump de `shadcn`/`eciesjs`.
- **R7 — Golden fail-open.** Tant que `schema_snapshot_stub` reste fail-open, supprimer
  `contracts.schema.json` régénère un golden vert et défait le drift-gate du contrat que F
  vient de geler.
- **R8 — Dual-platform.** Classe env connue : les tests iroh-networked et les tests HTTP
  loopback `operator_server` sont instables sous Docker-on-Windows ; passer
  `SBFB_TEST_HTTP_TIMEOUT_SECS=120` au run `sbfb-ci` local. F est platform-agnostique
  (structs + digests + fixtures) mais le bloc §7.4 complet reste obligatoire.
- **R9 — Modèle de confiance de la clé de publication non défini** (Not evidenced, §5).
  Tant qu'il ne l'est pas, la menace « substitution de pack » ne peut pas être entièrement
  évaluée ; ne pas écrire dans le body que F la ferme.

---

## 10. Verdict G8

**PLAN-ADAPT.**

Justification (§4.5.7) — aucune décision Day-0 ni aucun gel D2 n'est touché : JCS RFC 8785,
Ed25519, domaines dédiés, `subject[]` blake3 à la in-toto, 6 digests séparés, interdits
durs, distribution raw-op sans bump restent exactement ceux du kickoff `:285-301`. Le
verdict n'est **pas** DESIGN-CONFLICT car aucune alternative rejetée n'est ré-ouverte et
aucun arbitrage PO n'est requis : les six points bloquants sont des **défauts de
spécification et de mécanique de gate**, tous corrigeables à l'intérieur du périmètre gelé,
et chacun appuyé sur une **evidence code/OSS concrète et rejouée** (census mesuré 25 et
gate `:190-199` ; `EXIT=4` mesuré sur le filtre `import_hostile` ; `task.rs:315-316` qui
interdit le hoist de `reject_nulls` ; `check-frontier-contracts.sh:157` qui exige un
`const <VERSION>` ; `contracts.rs:133,147,190-200` ; `Cargo.toml:11-19` du crate cible).
Le verdict n'est pas EXECUTE car F.4 est, tel qu'écrit, **inatteignable** (le critère
`check-frontier-contracts.sh` exit 0 contredit F.2), et la commande d'enforcement du gate
que F ouvre sort ROUGE avec les noms de tests du plan. Il n'est pas
SCOPE-CUT-CONSISTENT car rien n'est retiré : le périmètre est conservé et **précisé**.

**Approche corrigée que le code suit** (autorité : ce préflight, pas le texte du plan) :

1. Fermer la liste des `DOMAIN_*_V1` de F (défaut : les 4 nommées ; rôles §17.1 restants
   routés Phase M) avec une décision D8 écrite par famille — **B1**.
2. Appender ces constantes dans `crates/nexus-core-rs/src/canonical.rs` (kickoff `:316-322`,
   convention `nexus-<kebab>-v1`, extension du module-doc `:50-65`), **sans** dépendance
   `nexus-core-rs` dans `sbfb-workflow-core` ; la vérification Ed25519 vit dans un helper
   local du crate pur (précédent `nexus-trace-core`) ou chez l'hôte — choix écrit au body —
   **B2**.
3. Rafraîchir le census sur les **6 surfaces** de B3 dans le même commit, dont 2 invisibles
   au gate.
4. Nommer le module hostile pour que `import_hostile` matche (ou amender la matrice), et
   inscrire le nombre de tests sélectionnés au body — **B4**.
5. Installer une porte de canonicalisation unique dans `sbfb-workflow-core` (calque
   `hash.rs:266-300`), avec test démontrant la collision sur le chemin `serde_jcs` direct ;
   **ne pas** hisser vers `nexus-core-rs` — **B5**.
6. Déclarer `PACK_FORMAT_VERSION` et poser l'étiquette `// FRONTIER:` complète — **B6**.
7. Livrer le format avec : `deny_unknown_fields` partout, bornes taille/profondeur/
   cardinalité/longueur de chaîne **testées séparément**, entiers I-JSON, aucun membre
   optionnel signé, champ de révision réservé, `subject[]` mono-algo nommé + conjonction
   documentée, `graph_semantic_digest` sur ids explicites et ordre total, exhaustivité et
   complétude des 6 digests répliquées de `factory_profile.rs` — **A5, A9, A10, A11, A12,
   A17, A18**.
8. Tests : les 4 du plan **plus** le jumeau cosign (signature valide + corps malformé →
   refus terminal), le cross-domaine dans les deux sens, et les clés dupliquées sur champs
   MAP — **A6, A7, A8**.
9. Parité TS : canonicaliseur JCS first-party test-util + `@noble/hashes` en
   **devDependency**, sur le harnais golden existant, avec vecteurs adverses — **A4**.
10. Body 9 sections conforme à F.5, en y ajoutant : census 25→N assumé avec D8 par famille,
    THREAT_MODEL en report explicite nommant les 4 menaces non couvertes et leur phase
    propriétaire, statut non autoritaire du scanner, scope cuts datés (installation/rollback
    → X, SSRF → P1, fuzz → AG, `RunEvent.v1` rempli → I), et « 0 bump wire, 0 dep runtime
    ajoutée ».
