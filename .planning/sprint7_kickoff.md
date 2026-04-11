# Sprint 7 — Kickoff (P2P Discovery Layer)

**Écrit** : 2026-04-11
**HEAD entrée** : `2926383` (master tip post audit gate Sprint 6)
**Auteur** : session de démarrage Sprint 7 après lecture de
`MEMORY.md` + `nexus_grid_pivot.md` + `sprint_audit_gate.md`,
`.planning/sprint6_kickoff.md` §3+§8, `.planning/sprint6_audit_findings.md`
(17 findings, verdict CONDITIONAL PASS LIFTED), `docs/shell/PATTERNS.md`
§T4+§T5, `docs/rust/PATTERNS.md` section Sprint 2 audit, et
reconnaissance du split `nexus-worker-core`/`nexus-worker` et du
pattern PyO3 `sign_claim` dans `crates/nexus-core-py/src/lib.rs`.

---

## 1. Constat d'entrée — Phase 0 audit gate DONE

Sprint 6 a été **audité en Phase 0 de Sprint 7** par une session
Claude Code fraîche jouant `.planning/sprint6_audit_plan.md`
(9 tracks A..I). Verdict initial : **CONDITIONAL PASS** (2 P1,
8 P2, 7 P3). Les 10 commits de gate `9239315..2926383` ont
atterri sur master avant tout commit Sprint 7 :

- `9239315` `docs(sprint6): audit findings from Sprint 7 Phase 0 gate`
  (`.planning/sprint6_audit_findings.md`, 534 lignes, 17 findings)
- `05c96c4` `fix(sprint6): ctrl-k case-insensitive` (F-1, P1)
- `cfb06f9` `fix(sprint6): cross-language canonical fixture` (A-3, P1)
- `b6e7a65` `fix(sprint6): zod discriminated union` (A-1, P2)
- `4b2fde4` `fix(sprint6): projectStore preserve active invariant` (C-1, P2)
- `9b76c9e` `fix(sprint6): legacy fallback TODO marker` (D-1, P2)
- `81b1a81` `fix(sprint6): tighten vendor-ui size budget` (E-2, P2)
- `e5c2028` `fix(sprint6): error boundary around route outlet` (G-4, P2)
- `db8795f` `fix(sprint6): coordinator legacy descriptor boot sweep` (D-3, P2)
- `8fbe07b` `docs(sprint6): log T4-T7 tech debt from audit findings`
- `2926383` `fix(sprint6): explicit types in RouteErrorBoundary test`
  (audit G-4 follow-up)

**CONDITIONAL PASS LEVÉ.** Les 2 P1 sont fixés, tous les P2 sont
soit fixés soit loggés en tech debt T4..T7 dans
`docs/shell/PATTERNS.md`. Les 7 P3 restent optionnels (F-2, F-3,
I-1 notamment).

### Test counts à l'entrée (verifiés par l'audit)

| Suite | Count | Delta vs Sprint 6 |
|---|---|---|
| Rust workspace | 193 (62+11+10+105+5) | 0 |
| Python (SDK + coord + app-gov) | 82 + 1 skipped | +3 (cross-lang fixture + projectStore invariant + legacy sweep) |
| Vitest web/ | 99 | +22 (10 useCommandPalette + 6 cross_lang + 1 projectStore + 5 RouteErrorBoundary) |
| Playwright | 10 | 0 |
| `npm audit` | 0 vulns | — |
| `size-limit` | 4/4 green (vendor-ui 31/50) | — |

Le working tree est clean à `2926383`. Aucun P1/P2 non-fixé ne
traîne au moment d'ouvrir ce kickoff.

## 2. Goal Sprint 7 (une phrase)

Livrer le premier binaire Rust P2P long-lived de SBFB
(`nexus-shell-daemon`), les primitives curator list signées
Ed25519 (Rust + PyO3), le wrapping iroh-gossip / BlobsClient
pour la souscription aux listes, la découverte pkarr DHT, et les
pages `/browse` + `/curators` câblées sur le daemon via un proxy
coordinator — sans jamais casser le shell coordinator-only qui
reste opérationnel pour les utilisateurs sans daemon.

## 3. Phase 0 — Audit gate de Sprint 6 (DONE)

Status : **terminé avant ce kickoff**. Références :
- `.planning/sprint6_audit_plan.md` (plan joué)
- `.planning/sprint6_audit_findings.md` (verdict + findings)
- `docs/shell/PATTERNS.md` §T4..§T7 (tech debt P2 loggée)

Phase 0 est **fermée** et ne consomme plus aucun commit
Sprint 7. Elle est listée ici uniquement pour la traçabilité du
pattern `sprint_audit_gate.md` — à partir de Sprint 8, Phase 0
jouera `.planning/sprint7_audit_plan.md` que **Phase F de
Sprint 7 doit écrire** (cf §5 Phase F).

## 4. Décisions Day 0 (D1..D5 — gelées)

### D1 — IPC shell ↔ daemon : HTTP loopback via proxy coordinator

**Retenu** : `nexus-shell-daemon` écoute sur un port éphémère
loopback-only (127.0.0.1), écrit son `running.json` (analogue à
celui du coordinator via `nexus_coordinator.registry`), et est
exposé au shell React **via un proxy coordinator** sur les
routes `/daemon/*`.

**Rejeté** :
- Unix socket / named pipe → incompatibilité Windows (named pipes
  ≠ Unix sockets, ajoute deux code paths à maintenir)
- Port fixe 8766 → collisions entre workers/coordinators multiples
  sur le même user
- Shell appelle directement le daemon en cross-origin → doublerait
  la config CORSMiddleware P5 et forcerait le shell à découvrir la
  bonne URL en bypass du `projectStore`
- IPC file-based (daemon lit commands/, écrit results/) → over-kill
  pour des POST /subscribe ponctuels, polling overhead inutile

**Implications** :
- Le daemon ajoute son propre `CORSMiddleware` regex loopback
  (même pattern P5 que le coordinator) pour rester cohérent
  même si le shell passe en direct un jour
- Le coordinator est enrichi de nouveaux handlers `/daemon/*` qui
  lisent `~/.nexus-grid/shell-daemon/running.json`, font un
  `httpx.AsyncClient` passthrough vers le daemon, et retournent
  la réponse verbatim (Zod validée côté shell)
- Si aucun daemon ne tourne, les handlers renvoient 503
  `{error: "shell-daemon not running"}` et les pages `/browse`
  + `/curators` affichent un bloc "démarrer le daemon" au lieu
  d'un spinner infini

### D2 — Singleton enforcé via `running.json` + pid check

Un seul `nexus-shell-daemon` par user à la fois. Au démarrage :

1. Si `~/.nexus-grid/shell-daemon/running.json` existe, parser
   son `pid`
2. Vérifier si le pid est live (`psutil`-free : sur Windows via
   `OpenProcess` équivalent Rust `sysinfo`, sur Unix via
   `kill(pid, 0)`)
3. Si live → sortir en erreur `"daemon already running (pid N)"`
4. Si stale → log warning "removing stale running.json",
   supprimer le fichier, continuer

**Rejeté** : multi-instance. Un second daemon écouterait sur un
autre port et subscriber-rait doublement aux curator topics → gossip
amplification. Le shell ne saurait pas auquel parler. Sprint 7 garde
singleton strict ; v1.2 reconsidérera si besoin.

### D3 — Curator list schema + domain + topic gelés

**Schéma Rust (source de vérité, dans `crates/nexus-core-rs/src/curator.rs` nouveau)** :

```rust
/// A signed curator list: a list of project endpoints a specific
/// curator vouches for. Consumed by workers to populate the
/// "Browse" page without trusting a central registry.
pub struct CuratorList {
    /// Canonical format version — MUST equal CURATOR_LIST_FORMAT_VERSION
    pub version: u16,

    /// Curator's Ed25519 public key (also signs this list)
    pub curator_pubkey: [u8; PUBLIC_KEY_LENGTH],

    /// Human-readable curator name (for display)
    pub curator_name: String,

    /// Unix timestamp when this revision was published
    pub created_at: u64,

    /// Monotonic revision counter, used by subscribers to pick
    /// the newest version when two tickets arrive out of order
    pub revision: u64,

    /// Vouched-for projects
    pub entries: Vec<CuratorProjectRef>,
}

pub struct CuratorProjectRef {
    pub project_id: String,      // pkarr node id hex
    pub project_name: String,
    pub category: String,        // free-form tag ("gov", "coldcase"…)
    pub description: String,     // short summary, ≤280 chars
}

pub struct CuratorListEntry {
    pub list: CuratorList,

    /// Redundant pubkey (must equal list.curator_pubkey) —
    /// catches attribution split-brain (same pattern as ClaimEntry)
    pub curator_pubkey: [u8; PUBLIC_KEY_LENGTH],

    #[serde(with = "BigArray")]
    pub signature: [u8; SIGNATURE_BYTES],
}
```

**Domain separation** (dans `crates/nexus-core-rs/src/canonical.rs`) :

```rust
/// Domain separation tag for CuratorList canonical bytes.
pub const DOMAIN_CURATOR_LIST_V1: &[u8] = b"nexus-curator-list-v1";
```

**Topic gossip** : **unique** `topic = blake3("nexus-grid/curator/v1")[..32]`.
Pas de namespace par curator pubkey en v1. Raisons :
- Un shell-daemon débutant ne connaît aucun curator → subscribe à
  un topic global donne de la portée
- La déduplication se fait côté daemon (map `curator_pubkey → latest_revision`)
- La charge gossip est bornée : une list = ~2 KB JSON × ~50 curators
  actifs = 100 KB total, rediffusés rarement (revision bump seulement)

Sprint 8+ pourra ajouter un topic namespacé `nexus-grid/curator/v1/<pubkey>`
pour des updates ciblées, le v1 global reste le baseline.

**Rejeté** : reuse du `CuratorList` doctest Sprint 2. Le doctest
n'existe pas concrètement (c'était un stub) et le vrai besoin est
un struct signé avec revision + attribution matching (pattern
`ClaimEntry`), pas un wrapper bytestring anonyme.

### D4 — T4 tranché : Option B (wire `AppContext.submit_task`) — signature gelée

**Décision** : retenu **Option B** (non-breaking, signature gelée
en Sprint 7, implémentation en Sprint 8 Phase A).

**Pourquoi Option B plutôt qu'Option A** :
- Retirer `ActionTaskSubmit` de v1 (Option A) bumperait
  `schema_version` → nouveau snapshot `tabview_schema.json` +
  régénération canonique Python + régénération Zod + un commit
  cross-lang breaking atomique à haut risque de casser un outil
  tiers
- La promesse implicite de Sprint 6 était que `button.task_submit`
  fonctionne. Un retrait ressemblerait à un aveu d'échec alors
  que le contrat est salvable
- Sprint 8 doit de toute façon étendre `AppContext` (storage,
  events, file upload, migration runner). Ajouter une méthode
  `submit_task` est trivial dans le même chantier
- Option B préserve la surface gelée du schéma et ajoute un
  comportement runtime Sprint 8 sans bump de version

**Signature SDK Python gelée (à implémenter Sprint 8 Phase A, **pas** en Sprint 7)** :

```python
# packages/nexus-sdk/src/nexus_sdk/app.py (extension Sprint 8)

class AppContext:
    """Runtime context injected into every @nexus_tab /
    @nexus_command handler."""

    # Sprint 7 freeze — implementation Sprint 8 Phase A
    async def submit_task(
        self,
        worker: str,
        payload: dict[str, Any],
        *,
        priority: int = 5,
        parent_task_id: str | None = None,
    ) -> str:
        """Submit a task to the coordinator's dispatcher.

        `worker` is a routing key the coordinator maps to a task
        type (e.g. "gov.contradiction_check"). `payload` is the
        JSON-serializable task body; the coordinator wraps it in
        a `Task` with the current project's keypair and writes
        it to the tasks doc. Returns the freshly-assigned
        `task_id`.

        Raises `CoordinatorUnreachableError` if no coordinator
        is reachable, `TaskRejectedError` if the worker key is
        unknown.
        """
        ...
```

**Wiring React gelé (Sprint 8 Phase A)** :

```tsx
// web/src/components/app/tabview/blocks/ButtonBlock.tsx

// Sprint 7 kickoff D4 freeze, impl Sprint 8 Phase A:
// - TabAppContext React context carries {projectName, coordinatorUrl, appName}
// - ButtonBlock reads it via useContext, resolves the submit_task endpoint to
//   POST /app/{appName}/tasks/submit on the active coordinator, body is the
//   ActionTaskSubmit payload verbatim
// - Coordinator proxies to dispatcher.submit_task(worker, payload)
// - 2xx → toast success + task_id; 4xx/5xx → toast error
```

La tech debt T4 dans `docs/shell/PATTERNS.md` est mise à jour en
Phase F Sprint 7 : "T4 status: signature gelée Sprint 7 Day 0,
implémentation Sprint 8 Phase A (blocking pour migration gov 19 tabs)".
Pas d'implémentation Sprint 7 — juste la doc + la signature.

### D5 — T5 tranché : `@nexus_command` signature gelée

**Décision** : le décorateur `@nexus_command` et ses types
miroirs Zod/Pydantic sont **designed et gelés ici**. L'implémentation
atterrit en Sprint 8 Phase A à côté des autres `AppContext` ext.

**Rationale** : sans cette signature figée, Sprint 8 aurait à la
débattre en cours de migration gov → risque de drift sur le contrat
shell/app qui est le seul verrou entre les deux sides.

**Signature Python gelée** :

```python
# packages/nexus-sdk/src/nexus_sdk/decorators.py (extension Sprint 8)
from nexus_sdk.commands import CommandDescriptor

def nexus_command(
    name: str,
    *,
    description: str,
    icon: str = "sparkles",
    group: str = "Actions",
) -> Callable[[F], F]:
    """Mark a coroutine on a NexusApp as a command palette entry.

    The decorated coroutine runs when the user selects the entry
    in the command palette. It takes the AppContext and returns
    either None (no UI change) or a NavigationIntent that the
    shell resolves to `navigate("/…")`.
    """

# packages/nexus-sdk/src/nexus_sdk/commands.py (new Sprint 8)
from pydantic import BaseModel, ConfigDict, Field

class CommandDescriptor(BaseModel):
    """Wire contract for a command palette entry, returned by
    the coordinator route GET /app/{name}/commands."""

    model_config = ConfigDict(extra="forbid", frozen=True)
    schema_version: Literal[1] = 1
    name: str = Field(..., min_length=1, max_length=64)
    description: str = Field(..., max_length=280)
    icon: str = Field("sparkles", max_length=32)
    group: str = Field("Actions", max_length=32)

class NexusApp(ABC):
    # Sprint 8 adds this next to tabs() / workers() / routes()
    def commands(self) -> list[Callable[..., Awaitable[Any]]]:
        """Collect @nexus_command-decorated methods via introspection."""
```

**Coordinator route gelée** (Sprint 8) :

```python
# packages/nexus-coordinator/src/nexus_coordinator/api/apps.py (ext Sprint 8)

@router.get("/app/{name}/commands", response_model=list[CommandDescriptor])
async def list_app_commands(name: str, request: Request) -> list[CommandDescriptor]:
    """Return the list of @nexus_command descriptors for the given app.
    Parses the return through CommandDescriptor.model_validate so a drift
    is caught at coordinator load time not at shell render time."""
```

**Zod mirror gelé** (Sprint 8) :

```ts
// web/src/api/coordinator.ts (ext Sprint 8)

export const CommandDescriptorSchema = z.object({
  schema_version: z.literal(1),
  name: z.string().min(1).max(64),
  description: z.string().max(280),
  icon: z.string().max(32).default("sparkles"),
  group: z.string().max(32).default("Actions"),
}).strict();

export type CommandDescriptor = z.infer<typeof CommandDescriptorSchema>;

export async function listAppCommands(
  coordUrl: string,
  appName: string,
): Promise<CommandDescriptor[]> { /* … */ }
```

**Wiring palette gelé** : `CommandPalette.tsx` gagne un 4e groupe
"App" qui merge toutes les `CommandDescriptor[]` renvoyées par
`listAppCommands()` pour chaque app enrôlée sur le coordinator
actif. Chaque entrée exécute sa `name` via
`POST /app/{appName}/commands/{commandName}/invoke` — endpoint
à geler aussi mais pas Sprint 7 scope ; Sprint 8 Phase A traite.

La tech debt T5 dans `docs/shell/PATTERNS.md` est mise à jour en
Phase F Sprint 7 : "T5 status: decorator + wire contract frozen
Sprint 7 Day 0, implémentation Sprint 8 Phase A".

## 5. Phase outline A..F (Sprint 7 proper)

### Phase A — `nexus-shell-daemon-core` + `nexus-shell-daemon` crates

Scope : créer le split headless/binary strictement copié du
pattern `nexus-worker-core`/`nexus-worker` :

- `crates/nexus-shell-daemon-core/` (library, UI-free)
  - `lib.rs` — doc module + re-exports + `VERSION`
  - `config.rs` — `ShellDaemonConfig` minimal (logging level +
    api_host par défaut `127.0.0.1`)
  - `paths.rs` — `nexus_grid_root` (réutilise helper déjà dans
    `nexus-worker-core::paths` — dédupliquer en montant dans
    `nexus-core-rs::paths` **non**, scope cut : duplication
    acceptée car les deux crates vivent côte à côte et la
    dédup créerait une dépendance qui n'existe pas aujourd'hui)
  - `state.rs` — `DaemonStateSnapshot` schema v1 (liste des
    curator lists actives, count entries, derniers heartbeats
    gossip)
  - `registry.rs` — `write_running_state` / `remove_running_state` /
    `check_stale_pid` (pattern `nexus_coordinator.registry` + pid
    liveness check via `sysinfo`)
- `crates/nexus-shell-daemon/` (binary)
  - `src/main.rs` — tokio + clap + subscriber wiring + ctrl-c
  - `src/cli.rs` — clap derive : `start`, `stop`, `status`, `config`
  - `src/http.rs` — axum router minimal (`GET /health`,
    `GET /info`, `GET /curators`, `POST /curators/subscribe`,
    `DELETE /curators/{pubkey}`, `GET /browse`) avec
    `CORSMiddleware` loopback regex (crate `tower-http` CORS)
  - `src/logging.rs` — tracing-subscriber (clone de
    `nexus-worker/src/logging.rs`)

**Critère Phase A acceptation** : `cargo test --workspace` passe,
`nexus-shell-daemon start` boot sans iroh (stub), écrit
`~/.nexus-grid/shell-daemon/running.json`, répond
`GET /health → 200 {"status":"ok","schema_version":1}`, ctrl-c
supprime `running.json`. **Aucune logique curator / pkarr encore
— juste le squelette runnable + tests headless.**

### Phase B — Curator list primitives Rust + PyO3

- `crates/nexus-core-rs/src/curator.rs` *(nouveau)* — struct
  `CuratorList`, `CuratorProjectRef`, `CuratorListEntry` + impl
  `sign` / `verify_signature` (exactement le pattern `ClaimEntry`
  dans `task.rs`, incluant le check d'attribution `curator_pubkey ==
  list.curator_pubkey`)
- `crates/nexus-core-rs/src/canonical.rs` — ajout
  `DOMAIN_CURATOR_LIST_V1`
- `crates/nexus-core-rs/src/lib.rs` — re-export du module
  `curator`
- `crates/nexus-core-py/src/lib.rs` — ajout `sign_curator_list`
  + `verify_curator_list_entry` PyO3 (pattern exact `sign_claim`
  lignes 878-897)
- Tests Rust : sign+verify, tampered list, attribution mismatch,
  wrong signer, domain separation (pattern copié de
  `task.rs::tests` 7 derniers tests)
- Tests Python : `packages/nexus-sdk/tests/test_curator.py`
  nouveau — roundtrip via `nexus_core.sign_curator_list` /
  `verify_curator_list_entry`

**Critère Phase B acceptation** : cross-language sign → verify
fonctionne, attribution mismatch détectée des deux côtés, aucun
drift Sprint 2 audit ne revient.

### Phase C — Shell-daemon iroh integration (gossip subscribe + fetch_ticket)

- `crates/nexus-shell-daemon-core/src/iroh_runtime.rs`
  *(nouveau)* — owns `nexus_core_rs::Node` + subscribe au topic
  `blake3("nexus-grid/curator/v1")`, boucle `next_event()` qui
  parse les messages en `{list_hash, ticket_str}`, appelle
  `BlobsClient::fetch_ticket(endpoint, memory_lookup, ticket_str)`,
  décode le body JSON en `CuratorListEntry`, verify signature,
  stocke dans un `DashMap<curator_pubkey, CuratorListEntry>` avec
  dédup par `revision`
- `crates/nexus-shell-daemon-core/src/state.rs` — expose
  `current_curator_lists()` qui snapshot le DashMap
- `crates/nexus-shell-daemon/src/http.rs` — `GET /curators` sérialise
  le snapshot en JSON (schema_version v1), `POST /curators/subscribe`
  ajoute le curator au set d'attention (filter les entries qu'on
  accepte), `DELETE /curators/{pubkey}` retire
- Tests : intégration 2-node (un publisher qui broadcast une
  liste, un daemon qui subscribe et la reçoit end-to-end)

**Critère Phase C acceptation** : un subscribe → fetch → verify
complet passe en 2-node test, `GET /curators` retourne bien la
liste après subscription, tampered list rejetée avec log warning.

### Phase D — Pkarr DHT browse

- `crates/nexus-shell-daemon-core/src/browse.rs` *(nouveau)* —
  résout les project_ids annoncés dans les curator lists via
  `Endpoint::lookup()` (iroh 0.97 presets::N0 publie et résout
  auto via pkarr), retourne `BrowseEntry { project_id,
  project_name, category, curator_pubkey, last_seen_at, status }`
- `crates/nexus-shell-daemon/src/http.rs` — `GET /browse` retourne
  l'agrégation
- Pas d'annonce pkarr sortante en Sprint 7 — on consomme seulement.
  Publish est scope Sprint 10 (release v1.0 public) pour éviter de
  polluer la DHT avec des nodes de dev
- Tests : stub pkarr lookup via `memory_lookup.add_endpoint_info`
  du pattern Sprint 2 audit S5

**Critère Phase D acceptation** : un project annoncé dans une
curator list et joignable par l'Endpoint local ressort dans
`GET /browse` avec `status: "reachable"`. Un project_id inconnu
ressort avec `status: "unreachable"`.

### Phase E — Coordinator proxy + Web pages câblées

- `packages/nexus-coordinator/src/nexus_coordinator/api/daemon.py`
  *(nouveau)* — router avec 5 routes proxy (`/daemon/info`,
  `/daemon/curators` GET + POST + DELETE, `/daemon/browse`) qui
  lisent `~/.nexus-grid/shell-daemon/running.json`, forwardent
  via `httpx.AsyncClient`, retournent verbatim. Sur absence
  running.json → 503 JSON `{error:"shell-daemon not running"}`
- `packages/nexus-coordinator/src/nexus_coordinator/paths.py` —
  nouveau helper `shell_daemon_registry_path()` → `~/.nexus-grid/shell-daemon/running.json`
- `web/src/api/daemon.ts` *(nouveau)* — Zod schemas
  `DaemonInfoSchema`, `CuratorListSchema` (liste), `BrowseEntrySchema`
  + helpers `getDaemonInfo`, `listCurators`, `subscribeCurator`,
  `unsubscribeCurator`, `listBrowse` — tous passent par
  `<coordinatorUrl>/daemon/*` (pas d'URL directe vers le daemon)
- `web/src/pages/Browse.tsx` — rewrite : React Query qui appelle
  `listBrowse()`, render en TabView-style cards, empty state
  "aucun project découvert", error state "daemon indisponible"
  avec CTA "Démarrer nexus-shell-daemon"
- `web/src/pages/Curators.tsx` — rewrite : listing des curators
  subscribés + bouton "Ajouter un curator" (Input `pubkey`
  hex 64 char) + bouton "Retirer" par entry + render TabView-style
- Playwright `curators-flow.spec.ts` + `browse-daemon-offline.spec.ts` —
  le premier spawn un daemon stub, le second teste l'état 503

**Critère Phase E acceptation** : le shell React affiche une curator
list après subscribe, survit au 503 daemon-offline avec UI non
cassée, les Zod schemas rejettent tout drift daemon → coordinator.

### Phase F — Sortie de sprint (obligatoire cf `sprint_audit_gate.md`)

**Livrables côte à côte** :

1. `.planning/sprint7_verification.md` — self-report fail-fast
   checklist (format Sprint 5/6, ≥24 rows)
2. `.planning/sprint7_audit_plan.md` — plan d'audit que la session
   fraîche de Sprint 8 Phase 0 jouera. **9 tracks minimum**
   (A crypto/canonical + B curator verify resilience + C daemon
   HTTP robustness + D singleton enforcement + E pkarr resolve
   correctness + F shell UX degraded states + G Sprint 8 risk
   assumptions + H deps audit + I doc coherence)
3. `docs/shell/PATTERNS.md` — ajout P9 (daemon pattern : HTTP
   proxy via coordinator, not direct) et mise à jour T4 + T5 avec
   "signature gelée Sprint 7, impl Sprint 8 Phase A" + closure
   tech debt (aucune nouvelle à logger si la phase est clean)
4. `docs/rust/PATTERNS.md` — ajout entry Sprint 7 canonical
   (domain `DOMAIN_CURATOR_LIST_V1` + attribution-match pattern +
   topic blake3 namespacing convention)

**Sans ces deux fichiers planning, Sprint 7 ne peut pas être
fermé.** C'est le point non-négociable de `sprint_audit_gate.md`.

## 6. Scope cuts (à respecter strictement)

- **Pas de bootstrap peers VPS FlowUP** — c'est Sprint 10 Phase
  release v1.0. Aucun code `bootstrap_nodes.toml` / `--seed-peer`
  / anything hardcoded VPS IP dans Sprint 7. Si le plan les
  réintroduit, c'est une erreur.
- **Pas de publish pkarr** — on consomme seulement la DHT en
  Sprint 7. Publish (nos propres projects annoncés) = Sprint 10
- **Pas d'implémentation `AppContext.submit_task`** — D4 a gelé
  la signature, Sprint 8 Phase A fait le code. Sprint 7 code
  reste inchangé côté `ButtonBlock.tsx` (le `console.warn` stub
  de Sprint 6 reste en place)
- **Pas d'implémentation `@nexus_command`** — D5 idem, Sprint 8
  Phase A
- **Pas de migration d'un tab gov** — Sprint 8 scope
- **Pas d'extension `AppContext.storage` / `.events`** — Sprint 8
- **Pas d'Unix socket / named pipe** — D1 figé
- **Pas de multi-instance daemon** — D2 figé
- **Pas de topic gossip namespacé par curator pubkey** — D3 figé
  (v1 global seulement)
- **Pas de re-signature cross-révision** — si un curator bump
  sa revision, c'est un nouveau `CuratorListEntry` complet. Pas
  de diff / delta.
- **Pas de multi-writer iroh-docs** — Sprint 10+ (mode 2/3 du
  plan phoenix)
- **Pas de persist SQLite des curator lists côté daemon** —
  Sprint 7 garde tout en RAM (DashMap). Redémarrage du daemon =
  re-subscribe via gossip. Ajouter SQLite = tech debt Sprint 9+
- **Pas de browse filter / search UI** — Sprint 8 ou 9 (listing
  brut suffit en Sprint 7, les curator lists resteront petites
  en early access)
- **Pas d'icônes dynamiques par curator** — icône fixe `<BookmarkPlus>`
  pour tous en Sprint 7

## 7. Traçabilité scope (Sprint 5 "What's NOT" — suite)

| Item Sprint 5 "What's NOT" | Sprint | Phase | Status |
|---|---|---|---|
| nexus-shell-daemon | **7** | A + C | Sprint 7 Phase A skeleton, Phase C iroh wiring |
| schema-driven tab rendering | 6 | A + B | DONE |
| curator list flow Ed25519 | **7** | B + C | Sprint 7 Phase B (primitives) + C (wiring) |
| DHT browse (pkarr) | **7** | D + E | Sprint 7 Phase D (daemon) + E (UI) |
| 19-tab `nexus-app-gov` migration v1.1 | 8 | A..F | pending |
| command palette Ctrl+K | 6 | C | DONE |
| Vitest unit tests (T3) | 6 | D | DONE (closed) |
| bundle size CI (T2) | 6 | D | DONE (closed) |
| worker HTTP API (axum) | rejeté | — | Sprint 5 D3 figé, pas de revisite |
| mobile responsive < 1280px | rejeté | — | idem |

## 8. Audit gate pattern — rappel

Sprint 7 est le **premier cycle complet** du pattern
`sprint_audit_gate.md` :
- Phase 0 a été jouée → `.planning/sprint6_audit_findings.md` +
  11 commits de gate (DONE avant ce kickoff)
- Phase F sera obligatoire → Sprint 7 doit livrer
  `.planning/sprint7_audit_plan.md` pour que Sprint 8 Phase 0
  puisse jouer son audit sur une session fraîche

Exception possible uniquement si l'utilisateur demande
explicitement de skipper l'audit — dans ce cas, noter
"Phase 0 audit skipped per user decision YYYY-MM-DD" et prévoir
un audit rétroactif Sprint 8.

## 9. Checkpoint de validation

Avant d'écrire le code Sprint 7 Phase A (premier commit
`feat(shell-daemon): Sprint 7 Phase A — …`), l'utilisateur doit :

1. Valider les **5 décisions Day 0 D1..D5** (ou les challenger)
2. Valider le **split Phase A..F** et l'ordre (A squelette →
   B primitives crypto → C iroh runtime → D pkarr → E shell →
   F sortie)
3. Confirmer les **scope cuts §6** : bootstrap peers VPS hors
   Sprint 7, publish pkarr hors Sprint 7, implémentation T4+T5
   hors Sprint 7
4. Valider que le plan détaillé `.planning/sprint7_plan.md`
   (commité atomiquement avec ce kickoff) reflète bien ces
   décisions avec la grille d'exécution et la checklist fail-fast

---

**État** : kickoff rédigé, 5 décisions Day 0 gelées en attente de
validation, plan détaillé dans `.planning/sprint7_plan.md`. Aucun
commit code Sprint 7 ne peut atterrir avant que ces deux docs
soient commités via `docs(sprint7): kickoff + plan`.
