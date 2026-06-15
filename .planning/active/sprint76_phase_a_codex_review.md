Audit source uniquement sur le working tree courant (`master`, fichiers modifiés/non committés visibles). Tests non exécutés.

### Livrable 1 : `ConsentSnapshot` additif
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-worker-core/src/engine/state_writer.rs:54`, `:91`, `:100`
- Evidence :
```rust
54: pub const SCHEMA_VERSION: u32 = 1;
91: #[serde(default, skip_serializing_if = "Option::is_none")]
92: pub consent: Option<ConsentSnapshot>,
100: pub struct ConsentSnapshot {
104:     pub level: u8,
```
Rétro-compat confirmée : le test désérialise un `state.json` sans `consent` et vérifie `None` (`state_writer.rs:424-438`).

### Livrable 2 : pompe vers snapshot
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-worker-core/src/engine/runtime.rs:1212`, `:1220`, `:1232`
- Evidence :
```rust
1212: let inputs = SnapshotInputs {
1220:     consent: self.consent_snapshot(),
1232: fn consent_snapshot(&self) -> Option<state_writer::ConsentSnapshot> {
1233:     let cfg = self.consent.as_ref()?.current().ok()?;
1241:     .and_then(|u| u.try_lock().ok().map(|mut g| g.hours_used_today()))
```
Le helper lit bien `ConsentWatcher`, lit `UsageTracker` via `try_lock`, puis transporte level/caps/heures (`runtime.rs:1243-1249`).

### Livrable 3 : enrolement worker co-localise D1
- Statut : CONFIRME
- Fichier(s) : `crates/nexus-shell-daemon/src/local_worker.rs:280`, `:329`, `:346`, `:368`; `crates/nexus-shell-daemon/src/http.rs:3243`
- Evidence :
```rust
329: let mut consent = ConsentConfig::default_for("local-worker");
334: consent.level = ConsentLevel::Whitelist;
335: consent.allowed_project_ids.insert(project_id);
346: if let Some((level, caps)) = user_public_consent(user_sbfb_home) {
347:     consent.level = level;
348:     consent.caps = caps;
```
`own_node_id` utilisateur n’est pas copié : seuls `level` et `caps` le sont. Le floor own-doc survit. Consent absent ne s’ouvre pas : `load_or_default` retombe sur `OwnProjects` (`consent.rs:192-203`), puis `user_public_consent` renvoie `None` hors `OpenSource/All` (`local_worker.rs:374-378`). Call-site confirmé avec `state.sbfb_home` (`http.rs:3240-3244`).

### Livrable 4 : routes consent `/api/v1/consent*`
- Statut : CONFIRME
- Fichier(s) : `web/src/api/consent.ts:117`, `:159`, `:166`, `:175`; `crates/nexus-shell-daemon/src/http.rs:423`
- Evidence :
```ts
117: const res = await authFetch(`${baseUrl}/api/v1/consent`, {
159: return consentPost(baseUrl, "/api/v1/consent/set", cfg);
166: return consentPost(baseUrl, "/api/v1/consent/whitelist/add", {
175: return consentPost(baseUrl, "/api/v1/consent/whitelist/remove", {
```
Daemon match exact : GET `/api/v1/consent`, POST `/api/v1/consent/set`, add/remove whitelist (`http.rs:423-431`). Pas de `/consent/get`.

### Livrable 5 : type front + schema
- Statut : CONFIRME
- Fichier(s) : `web/src/api/coordinator.ts:399`, `:408`, `:422`
- Evidence :
```ts
399: export const ConsentSnapshotSchema = z.object({
400:   level: z.number().int().min(1).max(4),
402:   hours_used_today: z.number().nonnegative(),
408: export const WorkerStateV1Schema = z.object({
422:   consent: ConsentSnapshotSchema.nullable().optional(),
```
Le schéma est non-strict par défaut (`z.object` sans `.strict()`), avec commentaire d’acceptation additive (`coordinator.ts:418-421`).

### Livrable 6 : page “offrir ma puissance”
- Statut : CONFIRME
- Fichier(s) : `web/src/pages/Network.tsx:110`, `:133`, `:231`, `:254`, `:281`
- Evidence :
```tsx
110: const liveConsent =
112:   ? query.data.state.consent ?? undefined
133: <OfferPowerCard consent={liveConsent}
240: const level = (consent?.level ?? fallbackLevel)
281: {consent && hoursCap !== null && (
```
CTA intentionnel confirmé : `Offrir ma puissance au réseau` (`Network.tsx:254`, `:268`). Jauge heures/jour confirmée (`Network.tsx:282-294`).

### Livrable 7 : double-confirmation L4
- Statut : CONFIRME
- Fichier(s) : `web/src/components/GpuConsentDialog.tsx:114`, `:168`, `:193`, `:360`
- Evidence :
```tsx
171: if (level === CONSENT_LEVEL.ALL && !confirmingAll) {
172:   setConfirmingAll(true);
173:   return;
175: void doSave();
196: setConfirmingAll(false);
```
Le premier clic L4 arme la confirmation sans POST. Changer de niveau désarme. Le second clic appelle la sauvegarde.

### Livrable 8 : tests
- Statut : CONFIRME
- Fichier(s) : `state_writer.rs:409`, `:442`; `local_worker.rs:519`, `:566`; `consent.test.ts:50`; `Network.test.tsx:102`; `GpuConsentDialog.test.tsx:144`
- Evidence :
```rust
417: let json = serde_json::to_value(&snap).unwrap();
420: json.get("consent").is_none(),
437: let back: WorkerStateSnapshot = serde_json::from_value(legacy).unwrap();
456: let c = snap.consent.clone().expect("consent carried into snapshot");
457: assert_eq!(c.level, 4);
```
Assertions utiles confirmées aussi pour worker co-localisé (`local_worker.rs:546-552`, `:593-598`), routes Vitest (`consent.test.ts:57-84`), jauge/CTA (`Network.test.tsx:115-137`) et double-confirm L4/direct save/disarm (`GpuConsentDialog.test.tsx:165-219`).

## Resume final
- Total livrables : 8
- Confirmes : 8
- Gaps : 0
- Partiels : 0