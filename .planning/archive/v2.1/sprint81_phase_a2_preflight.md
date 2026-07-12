# Sprint 81 Phase A2 — Préflight G8 (Workflow ultracode)

> **Verdict : PLAN-ADAPT** — la DIRECTION du plan (fail-fast root-cause ×2) est ratifiée et
> exécutable, mais sa LETTRE repose sur une prémisse factuellement fausse pour iroh-docs 0.98 :
> le discriminant n'est pas `Ok(None)` vs `Err`, c'est `Err(NotFound)` vs `Err(autre)`.
> Workflow `wf_a2e0db35-50c` (2026-07-02) : 5 scans + 5 vérifications adversariales + synthèse,
> 11 agents Opus 4.8 1M, ~882k tokens, 211 tool uses. Faits décisifs re-vérifiés de première
> main par le main thread (§5).

## 1. Rappel de la lettre du plan (sprint81_plan.md §Phase A2 + kickoff C2/C5)

Sur les 2 sites self-heal destructeurs (`boot_storage_namespace` runtime.rs:2456-2549,
recreate :2518 ; miroir `boot_feed_namespace` :2555-2633, recreate :2606) : `Err` →
fail-fast diagnostiquable (plus jamais `warn` + recreate) ; seul `Ok(None)` (cas légitime :
DB importée d'un autre data-dir) recrée un namespace neuf. Delta +2..4 tests. 0-bump,
indépendant d'iroh, commit séparé.

## 2. Pourquoi PLAN-ADAPT (prémisse fausse, évidence code upstream)

1. **`open_doc` ne rend JAMAIS `Ok(None)` en 0.98.** L'API upstream `Docs::open`
   (`iroh-docs-0.98.0/src/api.rs:262-265`) fait `rpc(OpenRequest).await??` puis retourne
   **inconditionnellement** `Ok(Some(Doc::new(...)))` sur succès. Notre wrapper
   (`nexus-core-rs/src/docs.rs:155-161`) ne fait que mapper l'`Option` — la branche
   `Ok(None) ⇒ recreate` du plan est **inatteignable** et le doc-comment `docs.rs:153-154`
   (« Returns Ok(None) if the document is not present ») est **faux**.
2. **Le cas LÉGITIME visé arrive en `Err`, pas en `Ok(None)`.** Une replica absente du store
   surface en `Err(OpenError::NotFound)` (`store/fs.rs:323` : `Ok(None) => return
   Err(OpenError::NotFound)` ; Display exact `"Replica not found"`, `store.rs:24-27`).
   C'est PRÉCISÉMENT le scénario « store reset / DB héritée d'un autre data-dir » que le
   hotfix introducteur `6ca9702` (2026-06-08) a observé LIVE et corrigé. Un fail-fast
   indiscriminé sur TOUT `Err` régresserait cet incident (`storage namespace not
   initialized` pour toute la session).
3. **Le scope littéral n'atteint pas le « crash diagnostiquable ».** Les 2 call-sites avalent
   déjà tout `Err` remonté en `warn` + dégradation (storage `runtime.rs:727-729` boucle
   per-app ; feed `:762-765` → `(None, None)`). Sans propagation aux call-sites, le fix
   resterait dans la classe « perte silencieuse warn-only » qu'A2 doit fermer.
4. **Contrainte d'implémentation : le variant typé est ERASÉ.** `OpenError::NotFound` traverse
   la couche RPC iroh-docs en erreur string-based ; `docs.rs:160` aplatit en
   `NexusError::Docs(format!("open failed: {e}"))`. Le discriminateur DOIT être
   `.contains("Replica not found")` (jamais `==` — le message est préfixé).

Ce n'est **pas** un DESIGN-CONFLICT : S2 confirme qu'aucune décision Day-0 gelée ni commit de
déviation ne fige le recreate-on-Err (le Err-swallow vient d'un hotfix Cas D `6ca9702`, pas
d'un design) ; la direction fail-fast est ratifiée par le kickoff C5 (« le self-heal n'est PAS
un backstop », vérifié au code) ; l'INTENT (recréer seulement l'absent légitime, crasher le
reste) reste réalisable en corrigeant le variant discriminant.

## 3. Approche corrigée (à coder — supersede la lettre du plan)

**Discriminer `NotFound` (recreate BRUYANT) vs toute autre erreur (fail-fast diagnostiquable),
et propager le fail-fast aux call-sites.**

- **Sites internes** (`boot_storage_namespace` :2487-2497 et `boot_feed_namespace`
  :2581-2587) — remplacer le `match { Ok(opt)=>opt, Err(e)=>{warn!; None} }` par :
  - `Ok(opt) => opt` (Ok(Some)=reuse ; Ok(None) défensif, unreachable en 0.98, route vers
    recreate-loud) ;
  - `Err(e) if e.to_string().contains("Replica not found") => None` — absent légitime →
    branche recreate existante ;
  - `Err(e) => return Err(anyhow!(...))` — fail-fast avec contexte diagnostiquable :
    app_name/feed_key + namespace_id hex + cause upstream + remède opérateur (« refusing to
    silently recreate; restore the iroh store or clear the M8 storage_namespaces row »).
- **Recreate BRUYANT** : enrichir le `warn!` des branches recreate (:2515-2530 / :2605-2615)
  avec `ns = %ns_hex` + libellé explicite (« previous replica absent from local store —
  recreating fresh namespace »).
- **Call-sites** : propager. Storage `:727-729` → `return Err(e.context(...))` (abort du
  boot) ; feed `:762-765` → idem. Chaîne vérifiée : `start()` Err → `main.rs:203-205`
  `.context("daemon start failed")?` → exit ≠ 0 → systemd `Restart=on-failure` +
  `RestartSec=5` (`deploy/nexus-shell-daemon.service:31-32`) = crash-loop diagnostiquable
  dans journalctl, comportement VOULU par la spec.
- **Commentaires rectifiés** : runtime.rs:2480-2486 + :2578-2580 (« Ok(None) OR an Err » →
  sémantique réelle 0.98) ; doc-comment wrapper `docs.rs:153-154` corrigé.
- **NE CHANGE PAS** : branches outer `None` first-boot (:2532-2541 / :2617-2626) ; caller
  project_doc ; mécanisme recreate lui-même (gap `import_ticket` = dette pré-existante
  tracée) ; wire ; pins iroh ; 0 dep.

## 4. Restitution des scans (fan-out 5 + adversarial 5)

| Scan | Verdict-hint | Findings clés | Adversarial |
|---|---|---|---|
| S1a upstream | PLAN-ADAPT | S1a-1..4 : NotFound=Err jamais Ok(None) ; S1a-5 : variant érasé → string-match ; S1a-6 : recreate = namespace ALÉATOIRE neuf (destructivité C5 confirmée) | 6/6 CONFIRMED |
| S1b testabilité | PLAN-ADAPT | 0 dep ; AUCUN test direct existant (seuls 2 indirects chemin heureux) ; fn privées atteignables via `mod tests` ; delta solide = +2 (NotFound→recreate-loud), fail-fast ×2 = best-effort | 5/5 CONFIRMED (S1b-3 « DESIGN-CONFLICT » requalifié PLAN-ADAPT en synthèse : prémisse fausse ≠ décision gelée) |
| S2 historique | EXECUTE + 1 adapt | Err-swallow introduit par hotfix Cas D `6ca9702`, pas un design ; direction fail-fast ratifiée C5 ; aucune décision gelée contre | S2a-3 REFUTED (frontière statiquement déterminable), reste CONFIRMED |
| S3 threat/dispo | PLAN-ADAPT | Scope littéral insuffisant (call-sites avalent) ; crash-loop systemd 5s = voulu ; threat model ne nomme pas la classe → carry G ; DoS corruption locale acceptable pré-launch | S3-1 CONFIRMED (le point structurant) |
| S4 wire | (scan dégénéré) | Le scan a rendu un finding vide ; son adversarial a fait le travail : 0-bump vérifié (0 match `FEED_FORMAT_VERSION\|DOMAIN_\|_ANNOUNCEMENT_VERSION` dans runtime.rs, control-flow pur), + risques zombie-tests et lien Phase B | compensé + re-vérifié main thread §5 |

## 5. Contre-vérification main thread (adversariale, de première main)

1. `api.rs:262-265` lu : `Ok(Some(Doc::new(...)))` hardcodé — CONFIRMÉ.
2. `store/fs.rs:323` lu : `Ok(None) => return Err(OpenError::NotFound)` — CONFIRMÉ ;
   `store.rs:26` : `#[error("Replica not found")]` — string matcher exact CONFIRMÉ.
3. Call-sites lus (`runtime.rs:727-729`, `:762-765`) : warn + dégradation — CONFIRMÉ.
4. `git show 6ca9702` lu : le body décrit le cas légitime arrivant en
   `Err("Replica not found")` « au lieu de Ok(None) » — CONFIRMÉ empiriquement.
5. `main.rs:203-205` + `deploy/nexus-shell-daemon.service:31-32` lus : propagation exit≠0 +
   `Restart=on-failure`/5s — CONFIRMÉ.

## 6. Plan de tests (delta cible +2 solides, +2 best-effort)

1. `boot_storage_namespace_recreates_loud_on_absent_replica` — M8 pointé vers 32 octets
   aléatoires (namespace jamais créé), node vivant → `Err(NotFound)` → recreate ; assert
   `Ok`, ns_id M8 ré-écrit ≠ octets injectés. Garde-fou : le self-heal légitime SURVIT à A2.
2. `boot_feed_namespace_recreates_loud_on_absent_replica` — idem avec `FEED_NAMESPACE_KEY`.
3. `boot_storage_namespace_fail_fast_on_docs_error` (best-effort) — namespace VALIDE persisté
   puis node/docs shutdown → erreur non-NotFound → assert `Err` contenant le marqueur
   diagnostiquable + ns_id M8 INCHANGÉ (aucune recréation destructrice). Si le harness
   shutdown s'avère env-sensible (message/timing RPC), documenter le gap plutôt que flaky.
4. `boot_feed_namespace_fail_fast_on_docs_error` (best-effort) — miroir feed.

Harness : `create_node()` in-process (PAS multi_daemon networked), `CoordinatorDb` in-memory,
`#[tokio::test(multi_thread)]`, `mod tests { use super::*; }` accède aux fn privées.
Précédents : `boot_storage_namespace_persistent_reopen` (:4127),
`boot_feed_namespace_persistent_reopen` (:4139).

## 7. Risques

- **Matcher fragile** : `.contains("Replica not found")` obligatoire (préfixe « open
  failed: » ajouté par `docs.rs:160`) ; à RE-VÉRIFIER au bump 1.0.1 (Phase B).
- **Tests fail-fast env-sensibles** : delta garanti +2, aspirationnel +4 ; la branche
  fail-fast est un garde trivial si non testable hermétiquement.
- **Sémantique d'échec de `start()` changée** : toute erreur docs non-NotFound aborte le
  boot (une corruption d'UNE app aborte tout — acté cohérent C4/C5 « personne sur le
  réseau » ; granularité per-app à revisiter si l'hébergement multi-app grandit).
- **Scope creep interdit** : ne pas toucher outer `None` first-boot ni project_doc.

## 8. Wire check (0-bump + test-acteur §6.12)

0-bump CONFIRMÉ : control-flow + logs + commentaires uniquement ; aucune sérialisation/JCS
touchée ; M8 `storage_namespaces` = SQLite local (pas wire) ; format `DocTicket` inchangé.
Test-acteur : `boot_*_namespace` = logique de boot interne au daemon, lue par AUCUN runtime
distinct → PAS une frontière docs-contrat, aucune étiquette requise.

## 9. Commit shape

`fix(daemon): Sprint 81 Phase A2 — fail-fast diagnostiquable sur erreur docs non-NotFound au
boot namespace (0-bump)` — body : discrimination NotFound/autre aux 2 sites + propagation
call-sites + prémisse fausse corrigée (commentaires) + inchangés + delta tests.

## 10. Carries (hors scope A2, tracés)

1. **THREAT_MODEL** : nommer la classe « perte silencieuse warn-only » + résidu DoS local
   (corruption FS ciblée → crash-loop) → replier dans **Phase G** (qui amende déjà
   THREAT_MODEL.md).
2. **Matcher « Replica not found » à re-vérifier au bump** (Phase B) et aux phases
   migration F/H : si 1.0.x change la sémantique (vrai Ok(None) ou autre Display),
   re-calibrer le discriminateur.
3. **`import_ticket` jamais appelé dans les branches recreate** → recreate orpheline les
   entries répliquées (pré-existant, toléré pré-launch C4/C5) → dette durabilité
   post-launch.
4. **Granularité per-app du fail-fast storage** (une app corrompue aborte tout le boot) →
   point d'attention si multi-app grandit.
