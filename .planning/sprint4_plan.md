# Sprint 4 — Plan détaillé (Coordinateur Python + SDK + app gov)

**Statut** : DRAFT — en attente de validation user avant tout code.
**Date** : 2026-04-10
**Basé sur** : `.planning/sprint4_kickoff.md`,
`magical-marinating-phoenix.md` (lignes 715-818, 927-951, 1046-1051),
`docs/rust/PATTERNS.md` tech debt section, lecture in-extenso de
`task.rs`, `gossip.rs`, `docs.rs`, `invite.rs`, `runtime.rs`,
`nexus-core-py/src/lib.rs`.

**Règle or** : aucune ligne de code avant que le user ait explicitement
validé §2 (les 5 décisions critiques). Les 4 phases sont structurées
pour être arrêtables à chaque critère de fermeture.

---

## 1. État vérifié à l'entrée

- Branche `master`, HEAD `f68d997` (docs Sprint 3 verification). Working
  tree clean sauf `.planning/audit_sprint2/` (9 shards untracked, hors
  workspace) et ce plan (`.planning/sprint4_plan.md`) + le kickoff.
- Sprint 0/1/2/3 fermés. Sprint 3 e2e test vert, 161 tests workspace.
- Crates existantes : `nexus-core-rs`, `nexus-core-py`,
  `nexus-worker-core`, `nexus-worker`. Workspace uv déclare
  `crates/nexus-core-py` + `packages/*` mais `packages/` est vide.
- **P1 bloquants confirmés** (voir §2 décision A) :
  - `task.rs:309` `canonical_bytes` = `serde_json::to_vec` en ordre de
    déclaration ; commentaire ligne 307 affirmant la compatibilité avec
    `json.dumps(sort_keys=True)` est **faux**.
  - `invite.rs:149` `InvitePayload::canonical_bytes` a le même bug.
  - `gossip.rs:105` `GossipClient<'a>` avec `&'a Gossip`.
  - `docs.rs:56` `DocsClient<'a>` avec `&'a Docs` — même pattern, non
    listé par le kickoff mais candidat évident au même fix, à
    trancher par le user (voir §2 décision B).
- **P1 additionnel découvert** : `runtime.rs:347-401` `Engine::tick`
  ne fait aucune tentative de claim/execute/result ; le `TODO(W9.1)`
  est à la ligne 383 et attend explicitement un `DocTicket` dans
  l'invite. Confirmé.
- **Ce qui est déjà là et qu'on ne reconstruit pas** :
  - `Task`, `TaskEntry::sign/verify`, `ResultPayload`,
    `ResultEntry::sign/verify`, `Claim::new` (mais **pas**
    `Claim::sign/verify`, voir §2 décision D).
  - `Verifier::verify` avec layers signature + digest + logprobs dans
    `nexus-core-rs` (exposé via `PyVerifier`).
  - `DocsClient::import_and_subscribe`,
    `DocHandle::get_many_by_prefix`, `DocHandle::subscribe` ← le
    coordinateur validator consomme ça directement.
  - `BlobsClient::fetch_ticket`, mint ticket via
    `DiscoveryClient::my_endpoint_addr`.
  - Bindings PyO3 `sign_task`/`verify_task_entry`/`sign_result`/
    `verify_result_entry` en JSON string in/out. Fonctionnels mais
    hériteront automatiquement du fix JCS.
  - Format invite `nx1` base32 avec `version=1`, `scope`, `expires_at`,
    signature Ed25519. Utilisable, à étendre proprement (§2 décision C).

---

## 2. Décisions critiques Day 1 (bloquent Phase A)

Ces 5 décisions doivent être approuvées explicitement par le user
avant tout code. Les recommandations sont basées sur les lectures
ci-dessus + kickoff §4-§6.

### Décision A — Format canonique cross-langue

**Options vérifiées via context7/web** :

| Option | Rust | Python | Décision |
|---|---|---|---|
| JCS (RFC 8785) | `serde_jcs` v0.2.0 publié 2026-03-25, actif, 688k downloads | `jcs` 0.2.1 PyPI (titusz), stable depuis 2022-04-10, RFC stable | **RECOMMANDÉ** |
| CBOR canonique | `ciborium` avec canonical encoding | `cbor2` | fallback |
| MessagePack | `rmp-serde` | `msgpack` | fallback |
| Manual sort | custom serde writer | `json.dumps(sort_keys=True)` | fragile, rejeté |

**Recommandation : JCS.** Standard IETF stable, deux implémentations
conformes dans les deux langues, lexicographic sort garanti par
construction à tous les niveaux, nombres canoniques sans flottants
(on n'en a pas dans nos payloads, mais la garantie est dans le spec).
Le fait que `jcs` (Python) n'ait pas bougé depuis 2022 est un
indicateur de maturité, pas d'abandon — RFC 8785 est figée.

**Impact** :

- Remplace `serde_json::to_vec` par `serde_jcs::to_vec` dans trois
  endroits :
  1. `crates/nexus-core-rs/src/task.rs:309` `canonical_bytes<T>`
  2. `crates/nexus-worker-core/src/invite.rs:149`
     `InvitePayload::canonical_bytes`
  3. Tous les autres `serde_json::to_vec` qui produisent des bytes
     signés (à auditer : `grep "serde_json::to_vec" crates/`)
- Corrige les doc-comments faux : `task.rs:300-317`, `invite.rs:137-148`.
- Python : ajoute `jcs` aux deps du coordinateur, jamais de
  `json.dumps(sort_keys=True)` manuel pour du contenu signé.
- Tests cross-langue : fixture JSON canonique produite par Python jcs
  (script one-shot checké dans le repo) + test Rust qui rejoue
  `canonical_bytes(&sample)` et compare byte-pour-byte.
- Bonus audit P3 item 3 — domain separation prefix. Ajout en même
  temps pour éviter de re-toucher `canonical_bytes` plus tard :
  `canonical_bytes::<T: Serialize>(value, domain: &'static [u8])`
  prépend `domain || b"\0"` avant `serde_jcs::to_vec(value)`.
  Constantes : `DOMAIN_TASK_V1 = b"nexus-task-v1"`,
  `DOMAIN_RESULT_V1 = b"nexus-result-v1"`,
  `DOMAIN_CLAIM_V1 = b"nexus-claim-v1"`,
  `DOMAIN_INVITE_V1 = b"nexus-invite-v1"` (invite aussi, puisqu'on
  touche `invite.rs` dans le même commit).
- **Breakage** : les signatures existantes (Rust-only, produites dans
  les tests unitaires) deviennent invalides après le fix. Aucune
  signature externe n'a jamais été persistée (Sprint 3 est
  Rust-to-Rust en test). Zéro dette externe.

> **Question user** : OK pour JCS + domain prefix ? Si fallback CBOR
> souhaité, trancher maintenant.

### Décision B — Scope du fix lifetime

Le kickoff liste uniquement `GossipClient<'a>`. `docs.rs:56` a la même
structure `DocsClient<'a> { inner: &'a Docs }`.

**Options** :

| Option | Portée Day 0 | Risque |
|---|---|---|
| Fix GossipClient seul | 1 fichier, ~10 LOC | DocsClient reste une dette dans 3 mois |
| Fix GossipClient + DocsClient | 2 fichiers, ~20 LOC, même commit pattern | rien |

**Recommandation : les deux dans le même commit** (`fix(core-rs): P1
lifetime removal across GossipClient and DocsClient`). Le coût marginal
est nul, la cohérence maximale, les deux tests wrappers existants
couvrent les deux. Note explicite dans le message de commit que
DocsClient est proactif (pas encore un vrai blocker) pour éviter la
confusion en revue.

> **Question user** : Fixer les deux ou seulement GossipClient ?

### Décision C — Format invite v2

**Contrainte** : W9.1 drop-in (Phase D) nécessite qu'un invite
embarque le `DocTicket` du tasks doc, sinon le worker ne sait pas
quel doc importer. Le format `nx1` existant (`INVITE_VERSION = 1`) n'a
pas ce champ.

**Options** :

| Option | Compat v1 | Effort |
|---|---|---|
| (a) Hard bump v2, refus v1 par `decode` | v1 rejeté ferme | simple — un bump de constante + un nouveau field obligatoire |
| (b) v2 avec field optionnel, `decode` accepte `1 ≤ v ≤ 2` | v1 accepté (sans doc_ticket, mode observer de fait) | un peu plus complexe — plage de versions valides |
| (c) Nouveau format `nx2` séparé | incompat totale | rejeté, coûteux |

**Recommandation : (a) hard bump v2**. Justifications :
- Sprint 3 n'a jamais été distribué externally ; aucune invite v1
  n'existe "in the wild".
- Le coût de support de deux versions est non nul (code path
  supplémentaire, tests × 2, surface d'attaque élargie).
- Mieux vaut un format strict maintenant qu'une dette permanente
  jusqu'en v1.x.

**Payload v2** (nouvelle définition de `InvitePayload`) :

```rust
pub const INVITE_VERSION: u8 = 2;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InvitePayload {
    pub version: u8,
    pub project_id: String,
    pub project_name: String,
    #[serde(with = "hex_bytes32")]
    pub coordinator_pubkey: [u8; 32],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub coordinator_addr: Option<String>,
    /// NEW v2: DocTicket (string-encoded) pour le tasks doc. Le
    /// worker l'importe via docs_client.import_and_subscribe dès
    /// l'enrollment. Obligatoire pour scope=Worker, peut rester
    /// None pour scope=Observer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tasks_doc_ticket: Option<String>,
    pub scope: InviteScope,
    pub expires_at_unix: u64,
}
```

**Règles de cohérence** :
- `scope == Worker && tasks_doc_ticket.is_none()` → erreur au mint.
- `scope == Observer && tasks_doc_ticket.is_some()` → OK (le worker
  ignore le ticket si scope est observer).
- `decode` exige `version == INVITE_VERSION` (déjà le cas, pas de
  changement).

**Kickoff §5 Phase C** mentionne aussi un alias `nx://...` URL. Je
propose de **ne pas** implémenter l'alias URL en Sprint 4 — c'est de
la cosmétique UX, ça peut attendre Sprint 5 quand le frontend React
gérera le copy-paste depuis le navigateur. Le CLI accepte `nx1...`
brut.

> **Question user** : Hard bump v2 ou compat v1+v2 ? Alias `nx://`
> maintenant ou différé Sprint 5 ?

### Décision D — `Claim::sign/verify` + domain prefix

Audit P2 item 2 : `Claim` a `new()` mais pas `sign()`/`verify()`. La
course LWW documentée dans `task.rs:254-262` exige que le coordinateur
vérifie la signature sur chaque claim pour départager les conflits,
mais le type actuel ne peut même pas en porter une.

**Décision** : ajouter `ClaimEntry { claim, worker_pubkey, signature }`
sur le même pattern que `TaskEntry`/`ResultEntry`, avec
`ClaimEntry::sign(claim, &keypair)` et `verify_signature()`. Pas
optionnel pour Phase D (W9.1 drop-in en dépend directement). Peut
être soit Day 0 (avec le fix canonical_bytes), soit Phase D tôt.

**Recommandation : Day 0**. On touche déjà `task.rs` pour le fix JCS,
autant grouper tous les changements crypto dans un seul tir. Commits
séparés mais même fenêtre.

> **Question user** : ClaimEntry Day 0 ou Phase D ?

### Décision E — Layout docs coordinateur

**Options** (kickoff §5 Phase A) :

| Option | Structure | Permission |
|---|---|---|
| (1) Deux docs séparés | `tasks` (write-single) + `results` (write-multi) | propre |
| (2) Un seul doc avec préfixes `task:*` / `claim:*` / `result:*` | un DocTicket partagé | simple |

**Recommandation : (1) deux docs** — aligné sur le phoenix plan,
permet une permission model plus sûre (le tasks doc n'accepte pas les
writes des workers), et la logique de claim (workers écrivent `claim:*`
dans le tasks doc ?) peut rester cohérente si on autorise les workers
à écrire dans le tasks doc **pour les claims seulement** avec un write
ticket partagé mais un prefix strict enforcement côté validator.

**Subtilité** : iroh-docs n'a pas de permission per-prefix native.
Donc "single-writer tasks doc" signifie en pratique que le ticket
du tasks doc est `ShareMode::Read` pour les workers (via
`DocHandle::share_read()`), et un deuxième doc `claims` est partagé
en `Write` pour les workers. **Donc trois docs, pas deux** :
- `tasks` : write-single (coord), read-many (workers)
- `claims` : write-multi (workers), read-many (coord)
- `results` : write-multi (workers), read-many (coord)

L'invite v2 embarque alors 3 DocTickets plutôt qu'un. Ou bien un seul
ticket "bundle" qui contient les 3. Le kickoff parle de 2 docs mais
je pense que 3 est la bonne lecture de l'architecture actuelle.

**Alternative plus simple** : un seul doc `project-<id>` avec préfixes
`task:*` / `claim:*` / `result:*`, partagé en `ShareMode::Write` aux
workers. Le coordinateur validator ignore tout `task:*` write qui
n'est pas signé par sa propre pubkey. On perd la garantie structurelle
mais on gagne un DocTicket unique dans l'invite, un canal de sync
unique, et une sémantique LiveEvent plus simple.

**Recommandation révisée : option (2) un seul doc**. Justifications :
- Un seul `import_and_subscribe` à gérer, un seul stream LiveEvent,
  un seul ticket dans l'invite v2.
- La vérification "ce task: write est-il bien de la coord pubkey ?"
  est déjà couverte par `TaskEntry.verify_signature()` — c'est
  redondant d'avoir ça ET une permission doc au niveau iroh.
- Sprint 5 / v1.1 pourra migrer vers le modèle 3-docs si on a des
  raisons empiriques, sans changer le format d'invite (un bundle
  est une surextension).

> **Question user** : 1 doc avec préfixes, ou 2/3 docs séparés ?

---

## 3. Day 0 — Plan de commits (avant Phase A)

Dans l'ordre strict. Chacun doit compiler + tests verts avant
d'enchaîner.

### Commit 1 — `fix(core-rs): P1 canonical_bytes cross-language via RFC 8785 JCS + domain separation`

**Fichiers modifiés** :

- `Cargo.toml` (workspace) : ajouter `serde_jcs = "0.2"` dans
  `[workspace.dependencies]`.
- `crates/nexus-core-rs/Cargo.toml` : `serde_jcs = { workspace = true }`.
- `crates/nexus-core-rs/src/task.rs` :
  - Réécrire doc module-level (lignes 9-27) : JCS au lieu de
    "serde_json declaration order".
  - Ajouter constantes domain : `DOMAIN_TASK_V1`, `DOMAIN_RESULT_V1`,
    `DOMAIN_CLAIM_V1` au niveau module.
  - Réécrire `canonical_bytes<T>(value: &T, domain: &[u8]) -> Result<Vec<u8>>`
    (nouvelle signature avec domain prefix).
  - Mettre à jour `TaskEntry::sign/verify_signature` pour passer
    `DOMAIN_TASK_V1`.
  - Mettre à jour `ResultEntry::sign/verify_signature` pour passer
    `DOMAIN_RESULT_V1`.
  - Corriger le doc comment `canonical_bytes` (lignes 295-317).
- `crates/nexus-worker-core/src/invite.rs` :
  - Ajouter `const DOMAIN_INVITE_V1: &[u8] = b"nexus-invite-v1"`
    (ou exporter depuis `nexus_core_rs::task`).
  - Réécrire `InvitePayload::canonical_bytes` pour utiliser
    `serde_jcs::to_vec` avec le prefix.
  - Corriger le doc comment ligne 137-148 (supprimer la mention
    "declaration order caveat").
- `tests/fixtures/canonical_task.json` (nouveau, à la racine du repo
  ou dans `crates/nexus-core-rs/tests/fixtures/`) : payload de
  référence pour le test cross-langue.

**Tests modifiés** : les 8 tests existants dans `task.rs` doivent tous
rester verts après re-sign (c'est-à-dire que le canonical changé n'est
pas observable par les assertions structurelles ; seul le test
`version_field_is_present_in_canonical_output` ligne 434-443 pourrait
casser si on n'inclut plus `"version":1` dans la sortie, mais JCS le
préserve).

### Commit 2 — `fix(core-rs): P1 remove lifetime from GossipClient and DocsClient`

**Fichiers modifiés** :

- `crates/nexus-core-rs/src/gossip.rs` :
  ```rust
  #[derive(Debug, Clone)]
  pub struct GossipClient {
      inner: Gossip,
  }
  impl GossipClient {
      pub fn new(inner: &Gossip) -> Self {
          GossipClient { inner: inner.clone() }
      }
  }
  ```
  (`Gossip` est `Clone` via son `Arc` interne — vérifier dans le
  registry local iroh-gossip-0.97.0 avant commit).
- `crates/nexus-core-rs/src/docs.rs` : même pattern pour
  `DocsClient { inner: Docs }`.
- `crates/nexus-core-py/src/lib.rs` : les sites qui faisaient
  `RsGossipClient::new(n.gossip())` et `RsDocsClient::new(n.docs())`
  continuent de fonctionner (la signature `fn new(inner: &X)` ne
  change pas côté appelant). Ajouter un smoke test PyO3 qui stocke
  un `GossipClient` dans une variable locale across un `await` pour
  prouver que ça compile sans lifetime.

**Tests modifiés** : aucun changement nécessaire aux tests existants
(le pattern `DocsClient::new(node.docs())` reste valide). Les tests
Rust existants sont la non-régression.

### Commit 3 — `feat(core-rs): Claim signed envelope (audit P2 #2)`

**Fichiers modifiés** :

- `crates/nexus-core-rs/src/task.rs` : ajouter
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
  pub struct ClaimEntry {
      pub claim: Claim,
      pub worker_pubkey: [u8; PUBLIC_KEY_LENGTH],
      #[serde(with = "BigArray")]
      pub signature: [u8; SIGNATURE_BYTES],
  }

  impl ClaimEntry {
      pub fn sign(claim: Claim, keypair: &KeyPair) -> Result<Self> { ... }
      pub fn verify_signature(&self) -> Result<()> { ... }
  }
  ```
  Utilise `canonical_bytes(&claim, DOMAIN_CLAIM_V1)`.
- Tests : `claim_entry_sign_then_verify`,
  `claim_entry_rejects_tampered_claim`,
  `claim_entry_rejects_wrong_signer` sur le modèle de TaskEntry.

### Commit 4 — `feat(core-py): expose sign_claim/verify_claim_entry + dict API`

**Fichiers modifiés** :

- `crates/nexus-core-py/src/lib.rs` :
  - Ajouter `sign_claim(claim_json: &str, secret: &PyBytes) -> PyResult<String>`.
  - Ajouter `verify_claim_entry(entry_json: &str) -> PyResult<()>`.
  - **Ergonomique bonus** : ajouter `canonical_bytes_task(task_dict)`,
    `canonical_bytes_result(result_dict)`, `canonical_bytes_claim(claim_dict)`
    qui prennent un `Bound<'_, PyDict>` plutôt qu'un JSON string et
    retournent `PyBytes`. Utile pour les tests Python qui veulent
    comparer byte-pour-byte avec la fixture cross-langue.
  - Enregistrer dans `nexus_core` module.

### Commit 5 — `test(core-rs): cross-language canonical bytes fixture`

**Fichiers modifiés** :

- `crates/nexus-core-rs/tests/cross_lang_canonical.rs` (nouveau) :
  charge `tests/fixtures/canonical_task.json` + `canonical_task.hex`,
  recompose le `Task` depuis le JSON, calcule
  `canonical_bytes(&task, DOMAIN_TASK_V1)`, compare au hex attendu.
- `tests/fixtures/gen_canonical_fixture.py` (nouveau) : script
  one-shot Python qui utilise `jcs.canonicalize` + le même domain
  prefix pour produire le hex de référence. Checké dans le repo ; le
  test Rust compare seulement, il n'invoque pas Python.
- `crates/nexus-core-rs/tests/fixtures/canonical_task.hex` : sortie
  du script ci-dessus, checké.

> **Note importante** : ce commit est facultatif si user dit non JCS.
> Mais si JCS est accepté, c'est la garantie anti-régression, donc
> obligatoire.

**Verification globale Day 0** :

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo nextest run --workspace --exclude nexus-core-py --profile ci --locked
cd crates/nexus-core-py && maturin develop --release  # rebuild wheel
uv run python -c "import nexus_core; nexus_core.sign_claim"
```

Tous verts → Phase A peut démarrer.

---

## 4. Phase A — Coordinateur core (~2 jours)

**Objectif** : un process `nexus-coordinator` qui charge une clé,
boote un `Node` iroh, crée le doc projet, et expose un `/health` HTTP.

### Fichiers à créer

```
packages/nexus-coordinator/
├── pyproject.toml                         # uv workspace member
├── README.md                              # (1 page, install + run)
├── src/nexus_coordinator/
│   ├── __init__.py                        # version + re-exports
│   ├── config.py                          # CoordinatorConfig pydantic-settings
│   ├── paths.py                           # ~/.nexus-grid/projects/<name>/
│   ├── keystore.py                        # load/save Ed25519 via coord.key
│   ├── coordinator.py                     # Coordinator: boot Node + doc + API
│   ├── api/
│   │   ├── __init__.py
│   │   ├── app.py                         # FastAPI factory + lifespan
│   │   ├── health.py                      # /health, /project
│   │   └── dependencies.py                # DI: Coordinator singleton
│   └── cli/
│       ├── __init__.py
│       ├── main.py                        # Typer root app (empty stub Phase A)
│       └── commands/
│           ├── __init__.py
│           ├── init.py                    # `nexus-coordinator init <name>`
│           └── start.py                   # `nexus-coordinator start <name>`
└── tests/
    ├── __init__.py
    ├── conftest.py                        # async event_loop + tempdir
    ├── helpers/
    │   ├── __init__.py
    │   └── iroh_fixture.py                # spin real Node in-process
    ├── test_config.py                     # pydantic roundtrip
    ├── test_keystore.py                   # load_or_generate + perm 600
    ├── test_coordinator_boot.py           # init → boots Node → /health 200
    └── test_cli_init_and_start.py         # subprocess smoke test
```

### Dépendances

`packages/nexus-coordinator/pyproject.toml` :

```toml
[project]
name = "nexus-coordinator"
version = "0.1.0"
requires-python = ">=3.13"
dependencies = [
    "nexus-core",                # via uv workspace path dep
    "pydantic>=2.6",
    "pydantic-settings>=2.13",
    "fastapi>=0.111",
    "uvicorn>=0.30",
    "typer>=0.12",
    "rich>=13",
    "structlog>=25.5",
    "aiosqlite>=0.20",
    "platformdirs>=4.3",
    "jcs>=0.2.1",                # cross-lang canonical bytes
    "httpx>=0.27",               # client dans les tests
]

[project.scripts]
nexus-coordinator = "nexus_coordinator.cli.main:app"

[project.entry-points."nexus.coordinator"]
# empty for now, populated by gov in Phase D

[tool.uv.sources]
nexus-core = { workspace = true }
```

### Points clés du code

- **`keystore.py`** : enveloppe autour de
  `nexus_core.load_or_generate_secret(path)` (déjà exposé dans
  `nexus-core-py/src/lib.rs:267`). Enforce perm 600 sur Unix, ACL
  restrictive sur Windows (via `os.chmod` + `icacls` hint). Pas de
  keyring en v1.0 (aligné sur Sprint 3 worker). Path :
  `~/.nexus-grid/projects/<name>/coord.key`.
- **`config.py`** : `CoordinatorConfig(BaseSettings)` avec prefix
  `NEXUS_COORD_` et `env_nested_delimiter="__"`. Sections :
  `identity`, `network` (api_host, api_port, public_visibility),
  `paths` (data_dir), `policy` (claim_timeout_secs,
  max_pending_tasks). Persistence TOML dans
  `~/.nexus-grid/projects/<name>/coordinator.toml`. Bootstrapping via
  `nexus-coordinator init <name>` qui écrit le TOML par défaut si
  absent.
- **`coordinator.py`** : classe `Coordinator` avec méthodes `async
  start() / stop() / is_running()`. `start()` :
  1. Load keypair via keystore.
  2. `await nexus_core.create_node_with_secret(secret)` →
     `self.node`.
  3. `author_id = await self.node.docs_author_create()` (persisté
     dans `coordinator.toml` → `identity.author_id` à la première
     exécution).
  4. Crée ou réouvre le doc projet (décision E). Si premier run,
     `doc = await self.node.docs_create()`, stocke son
     namespace_id dans `coordinator.toml`. Sinon `docs_open(id)`.
     **NB** : l'API Python actuelle n'expose pas `docs_open` — à
     ajouter dans `nexus-core-py/src/lib.rs` comme part of Phase A.
  5. Mint le `tasks_doc_ticket` via `doc.share_write()` pour le
     stocker en mémoire (sera embarqué dans les invites en Phase C).
  6. Démarre uvicorn programmatiquement en arrière-plan sur
     `127.0.0.1:<api_port>` avec la FastAPI factory.
- **`api/app.py`** : FastAPI factory avec `lifespan` async qui prend
  un `Coordinator` injecté. Routes montées : `/health`, `/project`.
  CORS restrictif (local only).
- **Tests d'intégration** :
  - `test_coordinator_boot.py` : spawn un `Coordinator` in-process
    avec un tempdir, attend `start()`, fetch `GET /health` via
    `httpx.AsyncClient`, vérifie 200 + project name + node_id.
    Aucun mock iroh. Pattern dans `conftest.py::iroh_coord_fixture`.
  - `test_cli_init_and_start.py` : `subprocess.run(["uv", "run",
    "nexus-coordinator", "init", "test", ...])`, vérifie les fichiers
    créés, lance `start` en background, curl `/health`, kill.

### Dépendance Rust ajoutée en Phase A

- `docs_open(namespace_id: str)` à ajouter dans
  `crates/nexus-core-py/src/lib.rs` (wrapper autour de
  `DocsClient::open_doc` qui existe déjà dans
  `nexus-core-rs/src/docs.rs:146`). 1 commit,
  `feat(core-py): expose Docs::open for coordinator reboot flow`.

### Critère de fermeture Phase A

1. `uv run nexus-coordinator init test --visibility private` crée
   `~/.nexus-grid/projects/test/coord.key` (perm 600) +
   `coordinator.toml`.
2. `uv run nexus-coordinator start test --port 8765` boote ; les
   logs structlog montrent `iroh endpoint ready node_id=...` et
   `doc ready id=...`.
3. `curl 127.0.0.1:8765/health` → 200 avec body
   `{"status": "ok", "project": "test", "node_id": "...", "doc_id":
   "..."}`.
4. `uv run pytest packages/nexus-coordinator/ -v` → all green.
5. `cargo nextest run --workspace --exclude nexus-core-py --profile
   ci --locked` → encore vert.
6. Commit atomique :
   `feat(coordinator): Sprint 4 Phase A — Node + doc + FastAPI
   /health`.

---

## 5. Phase B — Dispatcher + Validator + Kudos (~3 jours)

### Fichiers à créer

```
packages/nexus-coordinator/src/nexus_coordinator/
├── dispatcher.py                          # submit + track task_state
├── validator.py                           # subscribe + Verifier + kudos trigger
├── kudos.py                               # aiosqlite hash-chain ledger
├── db/
│   ├── __init__.py
│   ├── migrations.py                      # aiosqlite schema bootstrap
│   └── schema.sql                         # task_state + kudos_ledger
└── api/
    ├── tasks.py                           # POST /tasks/submit, GET /tasks
    ├── results.py                         # GET /results
    └── kudos.py                           # GET /kudos, GET /kudos/verify

packages/nexus-coordinator/tests/
├── test_dispatcher.py                     # submit 10 tasks → tasks doc
├── test_validator.py                      # feed results → kudos credited
├── test_kudos_hash_chain.py               # append 10 → verify → flip 1 byte → invalid
└── test_full_loop.py                      # dispatcher + validator + kudos in-process
```

### Schema DB (`schema.sql`)

```sql
CREATE TABLE IF NOT EXISTS task_state (
    task_id TEXT PRIMARY KEY,
    state TEXT NOT NULL CHECK (state IN ('pending','claimed','completed','failed','timed_out')),
    task_json TEXT NOT NULL,                -- full TaskEntry JSON
    submitted_at INTEGER NOT NULL,
    claimed_by_pubkey BLOB,                 -- 32 bytes
    claimed_at INTEGER,
    completed_at INTEGER,
    result_hash BLOB                        -- 32 bytes
);

CREATE INDEX IF NOT EXISTS task_state_by_state ON task_state(state);

CREATE TABLE IF NOT EXISTS kudos_ledger (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    worker_pubkey BLOB NOT NULL,            -- 32 bytes
    task_id TEXT NOT NULL,
    tokens INTEGER NOT NULL,
    quality_factor REAL NOT NULL,
    trust_multiplier REAL NOT NULL,
    amount REAL NOT NULL,                   -- = tokens * quality * trust
    awarded_at INTEGER NOT NULL,
    prev_hash BLOB NOT NULL,                -- 32 bytes, zeros for id=1
    entry_hash BLOB NOT NULL,               -- sha256(prev_hash || canonical_bytes(entry))
    entry_sig BLOB NOT NULL                 -- Ed25519 sig over entry_hash by coord key
);

CREATE INDEX IF NOT EXISTS kudos_by_worker ON kudos_ledger(worker_pubkey);
```

### Logique clé

- **Dispatcher** :
  - `async submit(task: Task) -> str (task_id)`. Signe via
    `nexus_core.sign_task(task_json, coord_secret)`, écrit dans le
    doc projet sous la clé `task:{task_id}`, insère dans
    `task_state` avec state=`pending`.
  - Task retry loop async (`asyncio.create_task`) qui scanne
    `claimed` tasks avec `claimed_at + claim_timeout_secs < now`,
    passe en `timed_out` et remet en `pending` pour re-dispatch.
  - Route `POST /tasks/submit` accepte un `TaskCreateRequest`
    pydantic, appelle `dispatcher.submit()`.
- **Validator** :
  - `async def run(self)` = boucle qui consomme
    `doc.subscribe()` (via `nexus_core.Doc.subscribe` — **à ajouter
    dans les bindings PyO3 Phase B si pas encore là**. Spoiler :
    c'est déjà là côté Rust ligne 309 mais **pas exposé dans
    `nexus-core-py/src/lib.rs`**. Commit Phase B :
    `feat(core-py): expose Doc::subscribe stream for validator loop`).
  - Sur chaque `LiveEvent::InsertRemote` avec key `result:*` :
    1. Parse `ResultEntry` du blob via `doc.get_content(hash)`.
    2. Load `TaskEntry` depuis `task_state.task_json`.
    3. `report = verifier.verify_entries(task_json, result_json,
       calibration_prompt_id)` via `nexus_core.Verifier`.
    4. Si `report["passed"]` → `kudos.credit(worker_pubkey, task,
       result)` → update `task_state.state = 'completed'`.
    5. Sinon → log `trust_delta`, laisse `task_state.state = 'pending'`.
  - Sur `claim:*` → update `task_state.state = 'claimed'`.
- **Kudos** :
  - `async credit(worker_pubkey, task_entry, result_entry) -> int
    (ledger_id)`.
  - Formule : `tokens = result.tokens_generated`, `quality =
    quality_factor(report, task.task_type)` (1.0 par défaut),
    `trust = current_trust_multiplier(worker_pubkey)` (1.0 pour
    v1.0 sans historique).
  - Hash chain : `entry_hash = sha256(prev_hash ||
    canonical_bytes(entry_dict, DOMAIN_KUDOS_V1))`. Ajouter
    `DOMAIN_KUDOS_V1 = b"nexus-kudos-v1"` (et exposer le domain prefix
    depuis core-py ; sinon recalculer en pur Python via `jcs` +
    prepend manuel, équivalent). Signature par `coord_secret` sur
    `entry_hash`.
  - `async verify_chain_integrity() -> (bool, Optional[int])` :
    replay depuis id=1, recompute chain, compare chaque
    `entry_hash`. Retourne `(True, None)` si OK, `(False, id)` à
    l'id du premier mismatch.

### Critère de fermeture Phase B

1. `pytest packages/nexus-coordinator/tests/test_full_loop.py` :
   - Spawn un coordinator in-process.
   - Un worker **simulé** (pas le binaire Rust encore) écrit 10
     `claim:*` + 10 `result:*` directement dans le doc via une
     deuxième instance `nexus_core.Doc` qui a importé le write
     ticket.
   - Le validator consomme les 10 results, crédit 10 kudos, met
     à jour `task_state`.
   - `verify_chain_integrity()` retourne `(True, None)`.
   - Flip 1 byte dans la base (UPDATE sur la ligne 5) → retourne
     `(False, 5)`.
2. `curl POST /tasks/submit -d '...'` → task_id + 200.
3. `curl GET /kudos` → liste les 10 entrées.
4. Commit : `feat(coordinator): Sprint 4 Phase B — dispatcher +
   validator + kudos chain`.

---

## 6. Phase C — Invite v2 + CLI Typer (~2 jours)

### Fichiers à créer / modifier

```
crates/nexus-worker-core/src/invite.rs              # MODIFIER: v2 + tasks_doc_ticket
crates/nexus-worker-core/src/allowlist.rs           # MODIFIER: store doc_ticket
crates/nexus-core-py/src/lib.rs                     # MODIFIER: expose invite mint/decode
packages/nexus-coordinator/src/nexus_coordinator/
├── invite.py                               # wrapper Python around nexus_core bindings
├── api/invites.py                          # POST /invite/create, DELETE /invite/{id}, GET /invite/list
└── cli/commands/
    ├── invite.py                           # Typer sub-app: create/revoke/list
    ├── configure.py
    ├── stop.py
    └── stats.py

tests/e2e/                                   # NEW directory, Python pytest
├── __init__.py
├── conftest.py                              # worker binary discovery
└── test_invite_roundtrip.py                 # coord mints → nexus-worker parses
```

### Changements Rust invite v2

- `INVITE_VERSION: u8 = 2` (bump dur).
- Add `tasks_doc_ticket: Option<String>` avec `#[serde(default,
  skip_serializing_if = "Option::is_none")]`.
- `Invite::mint(...)` signature : ajouter `tasks_doc_ticket:
  Option<String>`. Règle cohérence : `scope == Worker && ticket.is_none()`
  → return `InviteError::InvalidPayload("Worker scope requires
  tasks_doc_ticket".into())`.
- `canonical_bytes` déjà réécrit via JCS + domain prefix en Day 0.
- Tests : ajout `mint_worker_without_ticket_rejects`,
  `decode_v2_with_ticket_roundtrip`, `decode_v1_refused`.
- `crates/nexus-worker-core/src/allowlist.rs` : ajouter colonne
  `tasks_doc_ticket TEXT` à la table `projects` via
  `rusqlite_migration`. `NewProject` struct gagne `tasks_doc_ticket:
  Option<String>`. `join` flow (`cli.rs::join_cmd`) extrait le ticket
  de l'invite v2 et l'écrit dans l'allowlist.

### Bindings PyO3 invite

Ajouter dans `nexus-core-py/src/lib.rs` :

```rust
#[pyfunction]
fn mint_invite(
    coord_secret: &Bound<'_, PyBytes>,
    project_id: String,
    project_name: String,
    coordinator_addr: Option<String>,
    tasks_doc_ticket: Option<String>,
    scope: String,                  // "worker" | "observer"
    expires_at_unix: u64,
) -> PyResult<String> { /* returns nx1... */ }

#[pyfunction]
fn decode_invite(wire: &str, now_unix: u64) -> PyResult<Bound<'_, PyDict>> {
    /* returns dict with all fields or raises PyValueError */
}
```

Ça transforme `nexus_coordinator.invite.py` en un wrapper mince au
lieu de dupliquer la logique base32/JCS en Python.

### CLI Typer

- `nexus-coordinator init <name> [--public] [--description]`
- `nexus-coordinator configure <name>` (lance `$EDITOR` sur le
  coordinator.toml)
- `nexus-coordinator start <name> [--port 8765]`
- `nexus-coordinator stop <name>` (écrit `stop.flag` dans le data_dir,
  daemon loop le ramasse dans la boucle lifespan)
- `nexus-coordinator invite create <project> --scope worker --expiry
  7d [--max-uses 10] [--note "recruitment batch 1"]` → mint via
  `nexus_core.mint_invite`, persiste un enregistrement dans une
  nouvelle table `invites` (id, wire, scope, expires, max_uses,
  uses_count, revoked_at, note), imprime le `nx1...` string.
- `nexus-coordinator invite revoke <invite_id>`
- `nexus-coordinator invite list <project>`
- `nexus-coordinator stats <project>`
- `nexus-coordinator kudos verify <project>`

Note : `max_uses` est tracké côté coordinator DB, pas dans le payload
signé (pas de state server-side vérifiable pour un invite v2 sans
compromis). La révocation = `INSERT INTO invites SET revoked_at`.
Serveur rejette au runtime lors du first-contact RPC. L'expiration
reste dans le payload signé pour que le worker puisse valider
offline.

### Critère de fermeture Phase C

1. `cargo nextest run -p nexus-worker-core invite` tous verts, y
   compris les 3 nouveaux tests v2.
2. `uv run nexus-coordinator invite create test --scope worker
   --expiry 7d` imprime un `nx1...` non vide.
3. `nexus-worker join nx1...` (binaire Sprint 3) parse l'invite,
   l'allowlist SQLite contient une ligne avec `tasks_doc_ticket` non
   null, `nexus-worker projects list` l'affiche.
4. `tests/e2e/test_invite_roundtrip.py` : coord mint → subprocess
   `nexus-worker join ...` → vérifie le rowcount dans la DB
   worker.toml via `sqlite3` CLI.
5. Commit : `feat(coordinator,worker): Sprint 4 Phase C — invite v2
   + Typer CLI`.

---

## 7. Phase D — SDK + gov migration + W9.1 drop-in + e2e final (~4-5 jours)

### Fichiers à créer

```
packages/nexus-sdk/
├── pyproject.toml
├── src/nexus_sdk/
│   ├── __init__.py
│   ├── app.py                              # NexusApp ABC + AppManifest
│   ├── decorators.py                       # @nexus_route, @nexus_worker, @nexus_tab
│   ├── compute_client.py                   # ComputeClient.submit_task()
│   ├── loader.py                           # importlib.metadata entry_points
│   └── registry.py                         # per-process registry used by decorators
└── tests/
    ├── test_app_manifest.py
    ├── test_decorators.py
    ├── test_compute_client.py              # against a live in-process coordinator
    └── test_loader.py

packages/nexus-app-gov/
├── pyproject.toml                          # entry_point nexus.apps = gov_app
├── src/nexus_app_gov/
│   ├── __init__.py
│   ├── app.py                              # GovApp(NexusApp)
│   ├── manifest.py                         # version, name, tabs, workers
│   ├── prompts.py                          # POLITICAL_CONTRADICTION_PROMPT (from nexus/engine/)
│   ├── routers/                            # re-export existing nexus/gov routers
│   │   └── __init__.py
│   └── workers/                            # wrap existing gov workers as @nexus_worker
└── tests/

examples/hello-world-app/
├── pyproject.toml                          # entry_point nexus.apps = hello_app
├── README.md
└── hello_world_app.py                      # < 100 LOC total

tests/e2e/
├── test_coordinator_dispatches_10_tasks.py # THE acceptance test
└── fixtures/
    └── worker_config.toml                  # test worker config

docs/COORDINATOR.md                          # install + usage + architecture
```

### SDK design résumé

- `NexusApp` ABC :
  ```python
  class NexusApp(ABC):
      manifest: AppManifest  # class attr

      @abstractmethod
      async def on_start(self, ctx: AppContext) -> None: ...

      @abstractmethod
      async def on_stop(self) -> None: ...

      def routes(self) -> list[APIRouter]: return []
      def workers(self) -> list[WorkerDescriptor]: return []
      def tabs(self) -> list[TabDescriptor]: return []
  ```
- `@nexus_route(path, methods=["GET"])` : decorator qui enregistre
  dans un `registry._routes` keyed by module, récupéré par
  `NexusApp.routes()` via reflection au boot.
- `@nexus_worker(name, model)` : decorator qui produit un
  `WorkerDescriptor(name, model, fn)`. La fonction est async, reçoit
  un `TaskContext`, appelle
  `ctx.compute.submit_task(prompt=...)` si elle a besoin d'une
  inférence distante.
- `ComputeClient(coordinator_url)` : wrap
  `httpx.AsyncClient.post("/tasks/submit")`, retourne un
  `asyncio.Future[ResultPayload]` résolu par polling de `GET
  /results/{task_id}` (simple v1.0) ou par websocket (v1.1).
- `loader.discover_apps()` :
  `importlib.metadata.entry_points(group="nexus.apps")`, instancie
  chaque `NexusApp` et le retourne.
- **Coordinator intègre** le SDK loader : au `start`, après avoir
  booté le Node + doc + API, il appelle `loader.discover_apps()`,
  `await app.on_start(ctx)` pour chaque, monte les routes sous
  `/app/{name}/...`, ajoute les workers à un dispatcher scheduler
  interne.

### Migration gov

Point critique phoenix.md ligne 960 :
`nexus/engine/__init__.py:POLITICAL_CONTRADICTION_PROMPT`. Ce prompt
doit migrer dans `packages/nexus-app-gov/src/nexus_app_gov/prompts.py`
et NE PLUS être exporté depuis `nexus/engine/`. Le SDK reste
généraliste.

Scope minimum migration pour "fermer" Phase D : 1 route gov + 1
worker gov + 1 tab déclaré dans manifest. **Pas** une migration
intégrale des 19 tabs / 31 workers — ça peut déborder sur Sprint 4.5
ou v1.1. Le critère de succès phoenix ligne 816 est "au moins l'app
gov est migrée comme démonstration" pas "intégralement portée".

> **Question user** : migration minimale (1 route/worker/tab) ou
> intégrale (19 tabs / 31 workers) ? Je recommande minimale pour
> garder Sprint 4 à 14 jours.

### W9.1 drop-in

Fichier : `crates/nexus-worker-core/src/engine/runtime.rs`.

Modifications :

1. Ligne 83-96 `Engine` struct : ajouter
   ```rust
   task_docs: HashMap<String, (DocHandle, BoxStream<LiveEvent>)>,
   docs_client: DocsClient,  // owned (thanks Day 0 commit 2)
   ```
2. Ligne 121 `Engine::new` : pour chaque projet enabled avec
   `tasks_doc_ticket` non null, `docs_client.import_and_subscribe`
   → stocker dans `task_docs`.
3. Ligne 382-396 `WorkerState::Processing` branch : remplacer le
   `TODO(W9.1)` par :
   ```rust
   for (project_id, (doc, _stream)) in &self.task_docs {
       let tasks = doc.get_many_by_prefix(b"task:").await?;
       for entry in tasks {
           let task_entry: TaskEntry =
               serde_json::from_slice(entry.content().unwrap_or(&[]))?;
           if !task_already_claimed(doc, &task_entry.task.task_id).await? {
               let claim = Claim::new(
                   task_entry.task.task_id.clone(),
                   self.keypair.public_bytes(),
                   now_unix(),
               );
               let claim_entry = ClaimEntry::sign(claim, &self.keypair)?;
               doc.set(
                   author,
                   format!("claim:{}", task_entry.task.task_id),
                   serde_json::to_vec(&claim_entry)?,
               ).await?;

               // Execute
               let resp = self.ollama.generate(GenerateParams {
                   model: task_entry.task.model.clone(),
                   prompt: task_entry.task.prompt.clone(),
                   system_prompt: task_entry.task.system_prompt.clone(),
                   ..Default::default()
               }).await?;

               // Sign result
               let result = ResultPayload { ... };
               let result_entry = ResultEntry::sign(result, &self.keypair)?;
               doc.set(
                   author,
                   format!("result:{}", task_entry.task.task_id),
                   serde_json::to_vec(&result_entry)?,
               ).await?;
           }
       }
   }
   ```
4. Tests worker : adapter les tests existants
   `engine_transitions_to_processing_when_project_is_enrolled` pour
   ne pas casser (le doc est vide, aucune task, comportement
   identique). Ajouter `engine_claims_and_executes_task_when_present`
   qui monte un doc via `nexus_core_rs::DocsClient` in-process, écrit
   un `TaskEntry`, attend qu'un `result:*` apparaisse.

### E2E test final

`tests/e2e/test_coordinator_dispatches_10_tasks.py` :

```python
@pytest.mark.asyncio
async def test_coordinator_dispatches_10_tasks_to_external_worker(tmp_path):
    # 1. Spawn coordinator (in-process for easier debug, or
    #    subprocess for stricter isolation).
    coord = Coordinator(name="e2e-test", data_dir=tmp_path / "coord")
    await coord.init(public=False)
    await coord.start(port=find_free_port())

    # 2. Mint invite for worker.
    invite_wire = await coord.mint_invite(scope="worker", expiry_sec=3600)

    # 3. Spawn nexus-worker binary as subprocess.
    worker_dir = tmp_path / "worker"
    worker_proc = subprocess.Popen([
        "cargo", "run", "-q", "-p", "nexus-worker", "--",
        "--data-dir", str(worker_dir),
        "join", invite_wire,
    ], stdout=subprocess.PIPE)
    worker_proc.wait(timeout=30)
    assert worker_proc.returncode == 0

    worker_start = subprocess.Popen([
        "cargo", "run", "-q", "-p", "nexus-worker", "--",
        "--data-dir", str(worker_dir),
        "--headless", "start",
    ], stdout=subprocess.PIPE)

    try:
        # 4. Dispatch 10 tasks via coord API.
        task_ids = []
        async with httpx.AsyncClient(base_url=coord.api_url) as client:
            for i in range(10):
                r = await client.post("/tasks/submit", json={
                    "task_type": "analysis",
                    "prompt": f"Echo task {i}",
                    "model": "stub-model:latest",
                    "priority": 5,
                })
                assert r.status_code == 200
                task_ids.append(r.json()["task_id"])

        # 5. Wait for 10 results (timeout 120s).
        start = time.monotonic()
        while time.monotonic() - start < 120:
            completed = await coord.count_completed_tasks(task_ids)
            if completed == 10:
                break
            await asyncio.sleep(1)
        else:
            pytest.fail(f"only {completed}/10 tasks completed in 120s")

        # 6. Verify all signatures.
        for tid in task_ids:
            result = await coord.get_result_entry(tid)
            nexus_core.verify_result_entry(result.json_encoded)  # raises on failure

        # 7. Verify kudos chain.
        ok, bad = await coord.kudos.verify_chain_integrity()
        assert ok, f"kudos chain broken at id={bad}"
    finally:
        worker_start.terminate()
        await coord.stop()
```

**Pré-requis runtime** : Ollama doit tourner localement avec un
modèle (le test peut utiliser `llama3.2:1b` ou un stub si un mock
Ollama backend est ajouté — v1.0, on exige Ollama réel). Ou bien on
ajoute un mode `--stub-ollama` dans `nexus-worker` qui fait tourner
un `StubOllama` deterministe (déjà existant dans les tests Rust
`runtime.rs:430`). Je **recommande** d'ajouter ce flag comme part
du drop-in W9.1 pour rendre le test hermétique.

### Critère de fermeture Phase D

1. Tous les tests unitaires Rust + Python verts.
2. `grep TODO\(W9.1\) crates/nexus-worker-core/src/engine/runtime.rs`
   → 0 match.
3. `uv run nexus-coordinator start gov` boot l'app gov, `curl
   /app/gov/manifest` retourne au moins 1 tab, 1 worker, 1 route.
4. `wc -l examples/hello-world-app/src/**/*.py` ≤ 100.
5. `pytest tests/e2e/test_coordinator_dispatches_10_tasks.py` vert en
   < 120s.
6. Commits atomiques :
   - `feat(sdk): Sprint 4 Phase D — NexusApp ABC + decorators + loader`
   - `feat(worker): Sprint 4 Phase D — W9.1 runtime drop-in (claim + execute + result)`
   - `feat(app-gov): Sprint 4 Phase D — migrate nexus/gov to nexus-app-gov`
   - `feat(examples): Sprint 4 Phase D — hello-world-app < 100 LOC`
   - `test(e2e): Sprint 4 Phase D — coordinator dispatches 10 tasks to nexus-worker`

---

## 8. Tableau fail-fast Sprint 4 (livrable final)

À produire en `.planning/sprint4_verification.md` en fin de sprint :

| # | Check | Commande | Critère |
|---|---|---|---|
| 1 | canonical_bytes cross-lang | `cargo test -p nexus-core-rs cross_lang_canonical` | green |
| 2 | GossipClient owned | `grep -n "GossipClient<'" crates/nexus-core-rs/src/gossip.rs` | 0 match |
| 3 | DocsClient owned | `grep -n "DocsClient<'" crates/nexus-core-rs/src/docs.rs` | 0 match |
| 4 | ClaimEntry sign | `python -c "import nexus_core; nexus_core.sign_claim"` | no AttrError |
| 5 | Coordinator boot | `uv run nexus-coordinator start test --port 8765` | logs "iroh endpoint ready", /health 200 |
| 6 | Dispatcher → doc | test_full_loop.py phase B | green |
| 7 | Validator → kudos | test_full_loop.py phase B | green |
| 8 | Kudos integrity | flip 1 byte → `verify_chain_integrity` false | green |
| 9 | Invite v2 roundtrip | test_invite_roundtrip.py | green |
| 10 | Invite v1 refused | `cargo test -p nexus-worker-core decode_v1_refused` | green |
| 11 | SDK hello-world | `wc -l examples/hello-world-app/src/**/*.py` | <100 LOC |
| 12 | gov migration | `curl /app/gov/manifest` via running coord | 200 + ≥1 tab |
| 13 | W9.1 drop-in | `grep TODO\\(W9.1\\) crates/nexus-worker-core/src/engine/runtime.rs` | 0 match |
| 14 | E2E 10 tasks | `pytest tests/e2e/test_coordinator_dispatches_10_tasks.py` | 10 results <120s |
| 15 | Format | `cargo fmt --all --check` + `uv run ruff check packages/` | exit 0 |
| 16 | Rust tests | `cargo nextest run --workspace --exclude nexus-core-py --profile ci --locked` | all green (≥161) |
| 17 | Python tests | `uv run pytest packages/ tests/e2e/ -v` | all green |

---

## 9. Questions ouvertes à trancher avec le user

Synthèse des `> Question user` disséminées dans le document :

1. **Décision A** — JCS validé pour canonical format, ou fallback
   CBOR/msgpack à considérer ?
2. **Décision B** — Fix lifetime sur `GossipClient` **et** `DocsClient`
   dans le même commit, ou seulement `GossipClient` comme strictement
   spécifié par le kickoff ?
3. **Décision C** — Invite v2 en hard bump (refus v1), ou support
   v1+v2 pour compat ascendante ? Alias `nx://` URL maintenant ou
   différé Sprint 5 ?
4. **Décision D** — `ClaimEntry::sign/verify` en Day 0 (avec le fix
   JCS) ou en Phase D (avec le W9.1 drop-in) ?
5. **Décision E** — Layout doc projet : 1 doc unique avec préfixes
   `task:*` / `claim:*` / `result:*`, ou 2-3 docs séparés (tasks
   read-only + claims write-multi + results write-multi) ?
6. **Phase D gov scope** — Migration minimale (1 route / 1 worker /
   1 tab) pour valider le SDK, ou migration intégrale des 19 tabs /
   31 workers ?
7. **`.planning/audit_sprint2/`** — Les 9 shards untracked : commit,
   gitignore, déplacer hors repo, ou supprimer ?
8. **Ollama dans le test e2e** — Exiger Ollama réel local (pollue
   CI GitHub Actions), ou ajouter `--stub-ollama` au binaire
   `nexus-worker` pour un test hermétique (petite surface
   supplémentaire, plus de flake-free e2e) ?

**Réponses attendues sous forme** :
```
Décision A : [réponse]
Décision B : [réponse]
...
```

Une fois les 8 réponses reçues, je peux commencer le Commit 1 de
Day 0 immédiatement. Ce plan est volontairement verbose pour que le
user puisse y revenir sans relire tout le repo.

---

## 10. Ce que le plan NE fait PAS (scope explicitement hors Sprint 4)

- Aucune modification de `nexus/` legacy Python sauf
  `nexus/gov/` en Phase D, et `nexus/engine/__init__.py` pour
  extraire `POLITICAL_CONTRADICTION_PROMPT`.
- Aucune modification de `web/` (Sprint 5).
- Pas de pkarr publish (audit S8 P3, différé Sprint 4.5 ou v1.1 —
  j'ai relu l'audit et le kickoff, et rien dans Sprint 4 n'en dépend
  fonctionnellement tant que l'invite v2 embarque un
  `coordinator_addr`).
- Pas de curator list flow via gossip (différé post-Sprint 4, non
  listé dans phoenix Sprint 4).
- Pas de `AmdRocmBackend` / `AppleMetalBackend` pour GPU worker
  (différé v1.1).
- Pas de migration `nexus/core/`, `nexus/recon/`, `nexus/vision/`,
  `nexus/forensics/` → `packages/nexus-app-coldcase/` (phoenix Jour
  13, différable v1.1 selon le plan lui-même).
- Pas de refactor frontend (Sprint 5).
