Audit effectué sur le working tree réel via `git diff` + lecture fichiers. Tous les éléments demandés sont confirmés.

### Livrable 1 : helper anti-gaming
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/kudos_ledger.rs:37`
- Evidence :
```rust
37:pub const TOKENS_PER_MS_CEILING: u64 = 1_000;
44:pub fn sanity_bounded_tokens(tokens_generated: u64, generation_time_ms: u64) -> u64 {
45:    let ceiling = TOKENS_PER_MS_CEILING.saturating_mul(generation_time_ms.max(1));
46:    tokens_generated.min(ceiling)
```

### Livrable 2 : `credit()` paramétré et câblé aux 2 sites prod
- Statut : CONFIRME
- Fichier(s) : `kudos_ledger.rs:76`, `validator_loop.rs:109`, `http.rs:3465`
- Evidence :
```rust
81:    tokens_generated: u64,
82:    generation_time_ms: u64,
97:    let bounded_tokens = sanity_bounded_tokens(tokens_generated, generation_time_ms);
104:        amount: log_utility(bounded_tokens),
```
```rust
109:            if let Err(e) = kudos_ledger::credit(
114:                entry.payload.tokens_generated,
115:                entry.payload.generation_time_ms,
```
```rust
3465:            if let Err(e) = nexus_coordinator_rs::kudos_ledger::credit(
3470:                entry.payload.tokens_generated,
3471:                entry.payload.generation_time_ms,
```

### Livrable 3 : worker mesure la vraie durée d’inférence
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-worker-core/src/engine/runtime.rs:1102`
- Evidence :
```rust
1102:                let started_at = now_unix_secs();
1103:                let gen_start = Instant::now();
1104:                let generated = match self.llm.generate(params).await {
1111:                let generation_time_ms = gen_start.elapsed().as_millis() as u64;
1138:                    finished_at: now_unix_secs(),
```

### Livrable 4 : `ContributorSummary` / `get_contributor_summary`
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/kudos_ledger.rs:200`, `:236`
- Evidence :
```rust
215:pub struct ContributorSummary {
218:    pub effective_total: u64,
224:    pub tasks_served: u64,
226:    pub per_project: Vec<ContributorProject>,
```
```rust
241:    let entries = db.get_worker_entries(worker_node_id)?;
259:        let eff = effective_score(&project_entries, now_secs);
260:        let count = project_entries.len() as u64;
```

### Livrable 5 : `get_worker_entries()` via index M0
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/db.rs:50`, `:662`
- Evidence :
```rust
50:    CREATE INDEX IF NOT EXISTS idx_kudos_worker ON kudos (worker_node_id);
662:    pub fn get_worker_entries(
667:            "SELECT entry_id, worker_node_id, task_id, project_id, amount, created_at, prev_hash, entry_hash
668:             FROM kudos WHERE worker_node_id = ?1 ORDER BY created_at ASC, rowid ASC",
```
Aucune nouvelle migration/M20 dans le diff ; `db.rs` ajoute seulement la méthode et ses tests.

### Livrable 6 : route `GET /api/v1/contributor/{node_id}` authentifiée
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/kudos_api.rs:159`, `crates/nexus-shell-daemon/src/http.rs:252`, `:275`, `:454`
- Evidence :
```rust
159:pub async fn contributor_dashboard(
178:    match nexus_coordinator_rs::kudos_ledger::get_contributor_summary(&db, &node_id, now_secs) {
193:                Json(serde_json::json!({
195:                    "effective_kudos": summary.effective_total,
```
```rust
253:    let public_routes = Router::new()
254:        .route("/health", get(health))
275:    let authed_routes = Router::new()
454:        .route(
455:            "/api/v1/contributor/{node_id}",
456:            get(crate::kudos_api::contributor_dashboard),
```

### Livrable 7 : frontend dashboard contributeur
- Statut : CONFIRME
- Fichier(s) : `web/src/api/coordinator.ts:745`, `web/src/pages/Network.tsx:121`, `:426`
- Evidence :
```ts
745:export const ContributorSummarySchema = z
746:  .object({
748:    effective_kudos: z.number().int().nonnegative(),
750:    per_project: z.array(ContributorProjectSchema),
752:  .strict();
```
```tsx
128:  const contributorQuery = useQuery({
129:    queryKey: ["contributor", url, nodeId],
130:    queryFn: () => getContributorDashboard(url, nodeId as string),
```
```tsx
427:          <ContributorMetric label="Kudos effectifs (réputation, EMA)"
433:          <ContributorMetric label="Tâches servies (validées par quorum)"
438:            label="GPU-heures données par cette machine aujourd'hui (non attestées)"
```

### Livrable 8 : docs THREAT_MODEL §15.3 et PATTERNS §P61
- Statut : CONFIRME
- Fichier(s) : `docs/security/THREAT_MODEL.md:928`, `docs/rust/PATTERNS.md:3416`
- Evidence :
```md
928:### 15.3 Extension Sprint 76 Phase E — dashboard contributeur (D4)
942:| T | **Gonflage de kudos** ... sanity-bound `tokens <= TOKENS_PER_MS_CEILING * max(1, generation_time_ms)` ...
943:| T/D | **Forge coherente des deux champs** ... PAS une attestation anti-Sybil ...
944:| I | **Sur-promesse GPU-heures** ... JAMAIS repliquees ni agregees cross-nœud ...
```
```md
3416:## §P61 — Sprint 76 Phase E : sanity-bound plausibility-check
3431:1. **Asymmetric bound, not attestation.**
3435:   it as a plausibility-check, never as an anti-Sybil defense
3452:median-of-group re-scoped P2).
```

### Livrable 9 : tests delta annoncé
- Statut : CONFIRME
- Fichier(s) : `kudos_ledger.rs:520`, `db.rs:1612`, `http.rs:9263`, `dispatch_loop.rs:312`, `Network.test.tsx:142`
- Evidence :
```rust
520:    fn sanity_bound_clamps_implausible_token_claims() {
560:    fn credit_applies_sanity_bound_to_amount() {
583:    fn get_contributor_summary_aggregates_ema() {
636:    fn get_contributor_summary_empty() {
```
```rust
1612:    fn get_worker_entries_filters_by_node() {
1642:    fn contributor_query_uses_worker_index() {
1662:            plan.contains("idx_kudos_worker"),
```
```rust
312:            result.payload.generation_time_ms >= 1,
9263:    async fn contributor_dashboard_aggregates_node_credits() {
9302:    async fn contributor_dashboard_empty_for_unknown_node() {
```
```tsx
142:  it("rend les 3 métriques honnêtes ...", async () => {
167:    expect(screen.getByTestId("contributor-kudos")).toHaveTextContent("4200");
168:    expect(screen.getByTestId("contributor-tasks")).toHaveTextContent("7");
172:    expect(gpuHours).toHaveTextContent("2.5 h");
```

### Invariant : kudos non-monétaire
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/kudos_ledger.rs:4`, `web/src/pages/Network.tsx:418`
- Evidence :
```rust
4://! Kudos are non-monetary, non-transferable reputation scores tied to
```
```tsx
418:        Ce que cette machine a apporté au calcul du réseau. Kudos non
419:        monétaires, agrégés depuis le registre local de ce noeud.
```
Recherche diff ajoutée Rust+TS sur `cost|deposit|stake|burn|refund|escrow|currency|wallet|price` : `NO_MATCH`.

### Invariant : validator inchangé
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/validator.rs:117`
- Evidence :
```rust
117:    if task.redundancy_factor > 1 {
118:        return validate_quorum_pre_guardrail(
122:            &entry.payload.result_text,
123:            now,
```
`git diff -- crates/nexus-coordinator-rs/src/validator.rs` : 0 ligne.

### Invariant : 0 bump wire
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-core-rs/src/task.rs:61`, `:474`, `crates/nexus-core-rs/src/canonical.rs:74`
- Evidence :
```rust
61:pub const TASK_FORMAT_VERSION: u16 = 1;
474:    /// Number of tokens generated. Used for kudos accounting and
476:    pub tokens_generated: u64,
481:    pub generation_time_ms: u64,
```
```rust
74:pub const DOMAIN_TASK_V1: &[u8] = b"nexus-task-v1";
77:pub const DOMAIN_RESULT_V1: &[u8] = b"nexus-result-v1";
```
Recherche diff ajoutée `_VERSION|DOMAIN_|FORMAT_VERSION` : `NO_MATCH`.

### Invariant : self-view per-node, pas ranking global
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-coordinator-rs/src/kudos_ledger.rs:211`, `crates/nexus-shell-daemon/src/kudos_api.rs:150`
- Evidence :
```rust
211:/// [`effective_score`] verbatim ... It is a self-view
212:/// per node, NOT a network-wide ranking
```
```rust
150:/// Second aggregation view ... keyed on
151:/// `worker_node_id`
155:/// self-view, NOT a network-wide ranking.
```

### Invariant : sanity-bound non survendu anti-Sybil
- Statut : CONFIRME
- Fichier(s) : `docs/security/THREAT_MODEL.md:943`, `docs/rust/PATTERNS.md:3431`
- Evidence :
```md
943:| T/D | **Forge coherente des deux champs** ... PAS une attestation anti-Sybil ...
```
```md
3431:1. **Asymmetric bound, not attestation.**
3434:   It catches the bug and the naive over-claim, NOT the Sybil/forger
3435:   it as a plausibility-check, never as an anti-Sybil defense
```

### Vérifications exécutées
- `cargo test -p nexus-coordinator-rs sanity_bound --locked` : OK, 3 tests.
- `cargo test -p nexus-coordinator-rs contributor --locked` : OK.
- `cargo test -p nexus-coordinator-rs get_worker_entries --locked` : OK.
- `cargo test -p nexus-shell-daemon contributor_dashboard --locked` : OK.
- `cargo test -p nexus-shell-daemon dispatched_task_is_claimed_and_executed_by_worker_engine --locked` : OK.
- `cd web && npm run test:unit -- Network.test.tsx` : OK.

## Resume final
- Total livrables : 9 (+ 5 invariants)
- Confirmes : 14
- Gaps : 0
- Partiels : 0