# Sprint 77 — Phase J preflight (G8)

> Produit par Workflow ultracode (5 scans factuels Opus-4.8 + synthèse adversariale Opus-4.8).
> Phase front : panneau session shard + route daemon read-only + E2E hermétique.
> Vérifications croisées re-confirmées contre le code (http.rs:75-196/2399-2453, daemon.ts:231-528,
> shard_plan.rs:142-179, compute-tester.spec.ts:34-42, package.json:14).

## Verdict: PLAN-ADAPT

Le plan §13 reste **valide tel quel** dans son scope, son commit cible, son delta tests
(Rust +0 / Vitest +3 dont 1 E2E hors-count) et ses critères d'acceptation. **Aucune décision
Day-0 figée (D1..D5) n'est contredite** (route additive read-only, 0-bump wire, 0-dep, auth
loopback réutilisée, UX intentions PO-9, scope cut #8 mode-public NON violé) → ce n'est PAS un
DESIGN-CONFLICT. Mais l'**approche de la route doit être précisée et durcie** sur trois points
non spécifiés par le plan, chacun appuyé sur une évidence-code concrète :

1. **Absence totale de source de vérité** (S4, vérifié) : `DaemonHttpState` (http.rs:75-196) ne
   contient **aucun** store de session shard, et le crate `nexus-shell-daemon/src` a **0 référence
   shard** (grep vide). La route `GET /api/daemon/shard-session/{id}` est donc un **stub read-only
   déterministe à état vide** pour Phase J. Le plan dit « → statut » sans nommer la source ni le
   comportement id-inconnu → à documenter explicitement.
2. **Whitelist de champs + ZÉRO pubkey exposée** (S3, durcissement) : le plan §13.2 écrit
   « membres ». La projection NE DOIT JAMAIS sérialiser les `worker_pubkey`/`initiator` du
   `ShardedSessionManifest` signé (composition du groupe privé, surface SI-3/SI-4). On expose un
   **`member_count` agrégé** (un entier), pas la liste des identités.
3. **Contrat état-vide = enveloppe 200 stable, PAS 404** (S4 + précédent `seed_count`) : pour que
   le test api Zod `.strict()` valide une enveloppe stable et que le panneau « état vide » soit un
   **parse réussi** (et non un 404 traité comme erreur transport), la route renvoie `200
   {found:false, session:null}`. Précédent direct : `seed_count` (http.rs:2446-2453) renvoie
   toujours un `200` avec des défauts honnêtes (`peer_count:0`), jamais un 404.

Les durcissements ne touchent aucune Day-0 ; ils corrigent l'**approche** d'une route que le plan
décrit trop sommairement.

## Scans (résumé)

### S1a Page-mirror / test threshold
Signal **PLAN-ADAPT**. La page se mirrore exactement sur `Nodes.tsx` (useQuery + `DaemonResult`
union + `export const Component` lazy router) et le client sur `callDaemon<T>(baseUrl, path,
ZodSchema)` (daemon.ts:231-299). **ADAPTATION 1** : route shard-session sans état backing dans
`DaemonHttpState` (grep 0 match) → statut honnête vide via fn pure de projection + handler
`Path(id)`, **PAS un registre inventé**. **ADAPTATION 2** : gating E2E — le `test:e2e` par défaut
fait `--grep-invert @compute` (package.json:14), donc la partie hermétique de
`compute-shard.spec.ts` **sans tag `@shard`** tourne GREEN par défaut (bloquante) ; la partie
cross-machine est gardée par `test.skip(!ENABLED)` sur `SBFB_E2E_SHARD`, miroir exact de
`compute-tester.spec.ts:34-42` — **pas** par grep-invert (sinon CI hermétique cassée). CTA FR
exactes, data-testid anglais.

### S1b Deps / CVE / budget
**EXECUTE.** ZÉRO nouvelle dep npm (React 19 / react-query / zustand / zod / tailwind / Base
UI-shadcn / lucide-react / @playwright/test tous présents) ET ZÉRO nouvelle dep Rust (axum /
serde / serde_json / hex / `nexus-core-rs` path-dep tous présents dans `nexus-shell-daemon`).
Budget size-limit OK : les pages sont **lazy** (chunks per-route NON surveillés par les 6 buckets
size-limit) ; seul `vendor-query` est touché indirectement (le schéma Zod) avec **17,5 KB de
marge**, largement suffisant. Risques **P3 cosmétiques** seulement : marge css 1,24 KB et
vendor-ui 6,9 KB → réutiliser les utilitaires Tailwind/shadcn existants comme `Nodes.tsx` (l'atomic
CSS ne ré-émet pas une classe déjà présente). Aucune CVE introduite (surface dep inchangée).

### S2 Décisions historiques
**EXECUTE** (aucune contradiction figée). (1) UX intentions/PO-9 = Day-0 D5 respectée par le plan
§13.1 ; (2) la route read-only additive suit le **pattern FIGÉ** S73(/search `9472085`)/S75(/nodes
`1486fc9`) : tier auth loopback bearer+Host+Origin, Zod `.strict()`, 0-bump wire ; (3) **scope cut
#8** (mode public interdit, R-iroh-audit P0) NON violé — projection read-only d'un groupe **déjà
privé**, sur loopback, aucune découverte publique ni route mutative ; (4) `ComputeGroup` =
contrôle d'**ADMISSION** pas de **confidentialité** (THREAT_MODEL §16:1006-1009), les membres se
connaissent déjà (design_review:241). Net-new confirmé : grep `shard-session|ShardSession|
compute-shard` ne matche QUE les docs de planning. Items de vigilance (non-bloquants, → review) :
emplacement route dans le tier authentifié, enveloppe `.strict()` épinglée par un test Rust,
langage de surface sans claim « privé = chiffré ».

### S3 Threat model
Signal **PLAN-ADAPT** (auth intégralement réutilisée, mais 2 durcissements manquent au plan). La
route, **ajoutée dans `authed_routes`**, hérite de `auth_required` (auth.rs:395-455 :
bearer+Host+Origin, T0) — **aucun code d'auth à écrire**. Aucun invariant §16 violé (SI-1/SI-4
inchangés, control-plane only). Durcissements à fixer : **(a) WHITELIST de champs** — exposer un
statut agrégé (`session_id`, `member_count`, statut pipeline enum, niveau vérif), **JAMAIS** la
sérialisation brute du manifest (fuite `worker_pubkey`/`initiator` = SI-3/SI-4) ; **(b) DÉCISION
store-vs-404** — la route sans store est l'option minimale conforme au scope ; si un store éphémère
est un jour alimenté par ingest (Phase K+), gater par **vérif-signature `DOMAIN_SHARD_PLAN_V1` +
`is_member` AVANT insert** (discipline `authorize_claim` F2). Path-traversal/SQLi structurellement
adressés par le pattern existant (`Path<String>` → lookup in-memory, jamais d'interpolation
FS/SQL ; `SESSION_ID_MAX=128` déjà borné). Carry honnête à **maintenir** : **SI-9 withholding**
(timeout/fallback renvoyé « Phase J/data-plane » par le threat model) reste **non câblé** par le
scope §13 (control-plane only) → ne pas le marquer fermé au wrap-up.

### S4 Wire format / source de données
**PLAN-ADAPT honnête.** AUCUN registre de session shard vivante n'existe côté daemon (vérifié :
`DaemonHttpState` http.rs:75-196 sans champ shard ; 0 ref shard dans le crate daemon ; les seuls
`session_id` côté coordinator sont des **paramètres de fonctions pures** placement.rs/rerun.rs, pas
un store). Le data-plane `sbfb/shard/1` n'est PAS câblé à un store HTTP-lisible. → La route est un
**STUB read-only déterministe à état vide** pour tout id — ce qui est **exactement** le critère de
test « panneau état vide » du plan §13.3/§13.4. Le DTO HTTP n'est PAS un format wire signé (ne
touche ni `canonical_bytes` ni `DOMAIN_*`) → `SHARD_PLAN_FORMAT_VERSION`/`RUN_PROOF_FORMAT_VERSION`
restent à 1, **0-bump confirmé**. Le DTO ne doit JAMAIS exposer le `ShardedSessionManifestEntry`
signé brut.

## Recoupement adversarial

**Un scan a-t-il raté un conflit ou sur-revendiqué un verdict ?**

- **S1a/S3 « exposer les membres » vs S4 « stub état-vide » + S3 « whitelist »** : tension réelle,
  résolue. S4 prouve qu'**aucune donnée n'existe** pour peupler une liste de membres en Phase J →
  la route renvoie `found:false, session:null` pour tout id **aujourd'hui**. Le DTO `member_count`
  (et NON la liste de pubkeys) est défini pour le jour où un store sera câblé (Phase K+). Donc
  « exposer les membres » se **dégrade en « exposer un compte agrégé »** : compatible avec S3
  (privacy, jamais de pubkey) ET S4 (état-vide). **Les scans S1a/S3 qui parlaient de « membres »
  sont re-qualifiés** : `member_count:usize`, pas `Vec<pubkey>`.

- **S3 verdict PLAN-ADAPT vs S2 verdict EXECUTE** : non contradictoires. S2 (décisions figées)
  conclut justement qu'aucune Day-0 n'est contredite → pas de DESIGN-CONFLICT. S3 (threat) ajoute
  deux durcissements d'**approche** (whitelist + store-vs-404) absents du plan → PLAN-ADAPT. La
  règle « ≥1 scan PLAN-ADAPT étayé → global PLAN-ADAPT » s'applique : S3 ET S4 ET S1a sont tous
  PLAN-ADAPT étayés par évidence-code. S1b et S2 EXECUTE ne les annulent pas.

- **S3 « ajouter dans authed_routes » vs scope cut #8** : convergent. La route est loopback-only
  (T0), donc l'appelant est déjà local — aucune découverte publique de groupe. Le panneau est de
  l'**UI shell** (authFetch même-origine, x-sbfb-token), PAS une app iframe untrusted : pas de
  postMessage/CSP à gérer. (Note : ne PAS exposer la route via le bridge postMessage, qui est
  whitelisté à 3 méthodes ; le panneau passe par `callDaemon`, pas par le bridge.)

- **Précédent `seed_count` (vérifié http.rs:2446-2453)** : confirme byte-pour-byte le contrat
  état-vide retenu — un read-only renvoie un `200` avec défauts honnêtes (`peer_count:0`), jamais
  un 404. Le `{found:false, session:null}` de shard-session est le strict miroir.

**BLOCKER réel ?** Non. **Faux-positifs ?** Aucun fait n'est erroné ; le seul risque serait
d'implémenter la route en (a) inventant un `ShardSessionRegistry` (sur-ingénierie hors-scope front)
ou (b) sérialisant le manifest brut (fuite groupe privé). Le PLAN-ADAPT neutralise les deux.

## Approche d'implémentation corrigée

### Fichiers à créer / éditer

**Daemon (Rust)**
- `crates/nexus-shell-daemon/src/http.rs` (ÉDIT) :
  - nouvelle route `GET /api/daemon/shard-session/{id}` enregistrée **dans `authed_routes`**
    (bloc bearer+Host+Origin, à côté de `list_nodes`/`search`/`seed_count`), **JAMAIS** dans les
    Public routes ;
  - handler `async fn shard_session(State<Arc<DaemonHttpState>>, Path(id): Path<String>)` ;
  - fn **pure** de projection testable (mirror `nodes_response()`/`seed_count`) : renvoie toujours
    l'enveloppe état-vide (Phase J n'a pas de store) ;
  - DTO `ShardSessionStatusResponse` + `ShardSessionView` (`Serialize`) ;
  - test Rust d'enveloppe `shard_session_pins_envelope` (mirror
    `nodes_response_pins_envelope_and_grouping`) + test `shard_session_unknown_id_returns_empty`.

**Front (TS/React)**
- `web/src/api/daemon.ts` (ÉDIT) : `ShardSessionViewSchema` + `ShardSessionStatusResponseSchema`
  (enveloppe `.strict()`) + `getShardSession(baseUrl, id)` via `callDaemon`.
- `web/src/pages/ShardSession.tsx` **ou** `web/src/components/ShardSessionPanel.tsx` (CRÉER) :
  panneau read-only mirror `Nodes.tsx` (useQuery + `DaemonResult` union + bannière offline). Accès
  utilisateur (route lazy `App.tsx` + entrée nav, ou montage dans une page existante) = décision de
  **code non bloquante** pour le preflight (carry P2 review).
- `web/src/api/daemon.test.ts` (ÉDIT) **ou** `web/src/api/shard-session.api.test.ts` (CRÉER) : test
  Vitest `shard-session.api` — `GET` mocké, Zod `.strict()` rejette champ inconnu, état-vide
  parse OK.
- `web/src/components/ShardSessionPanel.test.tsx` (CRÉER) : Vitest rendu + état vide + intentions FR.
- `web/e2e/compute-shard.spec.ts` (CRÉER) : T1 hermétique (rendu FR + état vide) sans tag +
  `describe @shard` env-gated cross-machine.

### Forme EXACTE du DTO de réponse + schéma Zod

DTO Rust (additif, projection PROUVABLE depuis `ShardedSessionManifest`, **jamais** le manifest
signé brut ; enveloppe 200 état-vide stable) :

```rust
// Named-const enum mirror — closed enums, discipline §P (no magic strings)
#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum PipelineStatus { Forming, Running, Completed, Failed }

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum ShardVerificationLevel { N0, N1, N2, N3, Advisory }

#[derive(Serialize)]
struct ShardSessionView {
    session_id: String,                        // borné SESSION_ID_MAX=128
    member_count: usize,                       // = plan.assignments.len() — JAMAIS les pubkeys
    pipeline_status: PipelineStatus,           // enum fermé
    verification_level: ShardVerificationLevel,// niveau ATTEINT, dérivé verification.rs
}

#[derive(Serialize)]
struct ShardSessionStatusResponse {
    found: bool,                               // false pour id inconnu (état vide déterministe)
    session: Option<ShardSessionView>,         // None quand !found ; la clé est TOUJOURS sérialisée
}
```

Schéma Zod miroir (`daemon.ts`) — enveloppe `.strict()`, row tolérante (règle S73/S75) :

```ts
const ShardSessionViewSchema = z.object({
  session_id: z.string(),
  member_count: z.number().int().nonnegative(),
  pipeline_status: z.enum(["forming", "running", "completed", "failed"]),
  verification_level: z.enum(["n0", "n1", "n2", "n3", "advisory"]),
}); // row : PAS .strict() (additif-tolérant comme CatalogAppSchema)

export const ShardSessionStatusResponseSchema = z
  .object({
    found: z.boolean(),
    session: ShardSessionViewSchema.nullable(), // .nullable() PAS .optional() (clé toujours sérialisée, règle S73-E)
  })
  .strict();

export function getShardSession(
  baseUrl: string,
  id: string,
): Promise<DaemonResult<z.infer<typeof ShardSessionStatusResponseSchema>>> {
  return callDaemon(
    baseUrl,
    `/api/daemon/shard-session/${encodeURIComponent(id)}`,
    ShardSessionStatusResponseSchema,
  );
}
```

### Comportement id-inconnu / absence de registre

**Phase J : la route renvoie TOUJOURS `200 {found:false, session:null}`** (état vide déterministe)
car aucun store n'existe (`DaemonHttpState` sans champ shard). C'est `200` + enveloppe stable, **PAS
404** — précédent `seed_count` (http.rs:2446-2453 renvoie un `200` avec défauts honnêtes). Le test
api Zod `.strict()` valide ainsi un parse RÉUSSI, et le panneau « état vide » est un succès, pas une
erreur transport. **NE PAS inventer de `ShardSessionRegistry`** (sur-ingénierie hors-scope front ;
le câblage in-vivo data-plane est Phase K+). Quand un store sera ajouté (Phase K+), son chemin
d'ingest devra gater par **vérif-signature `DOMAIN_SHARD_PLAN_V1` + `is_member` AVANT insert**.

### CTA FR exactes (intentions, 0 jargon)

- **« Rejoindre un groupe de calcul »**
- **« Lancer un gros modèle en réseau »**

Aucun `shard` / `ALPN` / `ComputeGroup` / `sbfb` / `RunProof` visible en surface (PO-9/UX).
`data-testid` en anglais (ex. `shard-session-panel`, `cta-join-compute-group`,
`cta-launch-large-model`). Labels état : « Aucune session active » (état vide), « Membres », statut
pipeline traduit FR. **Ne JAMAIS écrire « privé = chiffré »** (le groupe est une admission, pas une
confidentialité — caveat S3). `scan-en-strings.sh` (scanne `web/src/`) doit rester vert.

### Structure du @shard env-gating dans `compute-shard.spec.ts`

Mirror byte-pour-byte de `compute-tester.spec.ts:34-42` :

```ts
// Partie HERMÉTIQUE — sans tag @shard, tourne GREEN par défaut (test:e2e --grep-invert @compute
// ne l'exclut PAS), BLOQUANTE wrap-up + CI :
test.describe("shard session panel (hermetic)", () => {
  test("renders the shard session panel with FR intents and empty state", async ({ page }) => {
    // charge le panneau contre le VRAI daemon Playwright, assert CTA FR byte-exact + état vide
  });
});

// Partie CROSS-MACHINE — taguée @shard, env-gated (skip si flag absent) :
const SHARD_ENABLED = process.env.SBFB_E2E_SHARD === "1";
test.describe("@shard cross-machine pipeline", () => {
  test.skip(!SHARD_ENABLED, "gated: set SBFB_E2E_SHARD=1 (multi-machine compute group)");
  // scénario cross-machine
});
```

Le `test:e2e` par défaut (`--grep-invert @compute`) NE filtre PAS `@shard` → la partie hermétique
tourne ; **NE PAS** gater la partie cross-machine par grep-invert (cela casserait la CI hermétique),
uniquement par `test.skip(!SHARD_ENABLED)`.

### Noms de tests

**Vitest (+3 attendus, plan §15)**
1. `ShardSessionPanel.test.tsx` → `renders FR intents and empty state` (+ variante CTA présents).
2. `shard-session.api` (dans `daemon.test.ts` ou fichier dédié) →
   `getShardSession parses empty-state envelope` + `rejects unknown field (.strict())`.
3. *(le 3e est l'E2E, hors-count Vitest — voir ci-dessous).*

**E2E Playwright (hors-count)**
- `web/e2e/compute-shard.spec.ts` :
  `renders the shard session panel with FR intents and empty state` (hermétique, GREEN par défaut).

**Rust (delta +0 au count, mais tests d'enveloppe recommandés au review)**
- `shard_session_pins_envelope` (mirror `nodes_response_pins_envelope_and_grouping`).
- `shard_session_unknown_id_returns_empty_envelope`.
- *(Si comptés, le delta Rust passe de +0 à +2 — carry P2 honnête à signaler au review ; le plan
  §15 annonce Rust +0, à reconcilier.)*

### Confirmation invariants

- **0 bump wire** : route additive HTTP, DTO non-signé (ne touche ni `canonical_bytes` ni
  `DOMAIN_*`) ; `FEED_FORMAT_VERSION`/`SHARD_PLAN_FORMAT_VERSION`/`RUN_PROOF_FORMAT_VERSION` =1
  intouchés. ✓
- **0 nouvelle dépendance** : npm (React 19/react-query/zustand/zod/tailwind/Base UI/lucide/
  @playwright/test) ET Rust (axum/serde/serde_json/hex/`nexus-core-rs` path-dep) tous présents. ✓
- **scan-en-strings FR** : CTA + labels en français, code/identifiants/erreurs/data-testid en
  anglais. ✓
- **auth réutilisée** : route dans `authed_routes` → `auth_required` (bearer+Host+Origin, T0),
  0 code d'auth. ✓
- **Day-0 D1..D5 intactes** : aucune touchée. ✓

## Risques consolidés

| # | Sév | Risque | Mitigation |
|---|-----|--------|------------|
| 1 | **P1** | Sur-ingénierie : câbler un vrai `ShardSessionRegistry`/store vivant en Phase J dépasse le scope front (data-plane = Phase K+). | Route = **stub état-vide déterministe** (`200 {found:false, session:null}`), exactement le critère de test du panneau. Ne PAS ajouter de champ shard à `DaemonHttpState`. |
| 2 | **P1** | Gating E2E cassé : taguer la partie hermétique `@shard` ou gater le cross-machine par grep-invert ferait disparaître la couverture de la CI hermétique. | Partie hermétique **sans tag** (tourne sous `--grep-invert @compute`) ; cross-machine en `@shard` + `test.skip(!SBFB_E2E_SHARD)`, miroir `compute-tester`. |
| 3 | **P2** | Fuite composition groupe privé : sérialiser `worker_pubkey`/`initiator` du manifest exposerait QUI compose le groupe (SI-3/SI-4). | DTO = **`member_count:usize` agrégé**, JAMAIS de pubkey ; jamais le `ShardedSessionManifestEntry` brut. Doc-comment caveat loopback (modèle §15.3). |
| 4 | **P2** | Emplacement route : si ajoutée dans les Public routes au lieu de `authed_routes` → régression d'auth. | Slot = bloc `list_nodes`/`search`/`seed_count` (tier bearer+Host+Origin). Test d'enveloppe + review. |
| 5 | **P2** | Drift Zod/Rust : un champ ajouté côté Rust ferait échouer l'enveloppe `.strict()` côté front. | Enveloppe `.strict()` épinglée par un test Rust (`shard_session_pins_envelope`) ; row tolérante. Test `shard-session.api`. |
| 6 | **P2** | Contrat état-vide flou (404 vs envelope) non figé. | **200 `{found:false, session:null}`** (précédent `seed_count` http.rs:2446-2453), pas 404 ; `.nullable()` pas `.optional()` (clé toujours sérialisée). |
| 7 | **P2** | Delta tests : le plan §15 annonce Rust +0, mais les tests d'enveloppe d'une route Rust valent +2 si comptés. | Soit garder Rust +0 (tests d'enveloppe en `web` only) et l'assumer, soit reconcilier le delta au commit body (honnêteté §P). À trancher au review. |
| 8 | **P3** | SI-9 withholding (timeout/fallback renvoyé « Phase J/data-plane » par le threat model) NON câblé par le scope §13 (control-plane only). | **Garder SI-9 carry honnête** au wrap-up ; ne PAS le marquer fermé (le câblage réel = Phase K/data-plane). |
| 9 | **P3** | Marges size-limit serrées (css 1,24 KB, vendor-ui 6,9 KB). | Réutiliser utilitaires Tailwind/shadcn existants (`glass-card`, etc.) comme `Nodes.tsx` ; pas de nouveau primitive Base UI/Radix lourd. Vérifier `npm run size` en clôture. |
| 10 | **P3** | Langage de surface laissant croire à une garantie de confidentialité du groupe. | Le panneau n'affiche que statut/appartenance/compte ; **jamais** « privé = chiffré ». |

Aucun P0. Aucun DESIGN-CONFLICT (pas de pivot A/B/C requis).
