# Review S81 Phase K — wrap-up bi-axe (binding attestation loaded-stage↔manifeste + câblage T1 + acceptance + clôture docs-contrat + planning)

## Verdict: PASS

> **Promu PASS-PENDING → PASS le 2026-07-11 après quorum PASS + Codex
> round 3.** Historique : review Workflow (6 dimensions adversariales)
> = FAIL sur 1 P1 + 5 P2 → tous traités. Codex passe 1 = FAIL, 2 P1
> (collision echo NEUVE + quorum) + P2/P3 → collision fixée root-cause.
> Codex passe 2 = P1 collision **CLOSED**, P2/P3 restants fermés (sweep
> docs + HUB/driver-signed REFERENCE/EXPLANATION + MIXED au vocabulaire +
> doc-comments loaded_stage) ; seul résiduel = `b3_p2_quorum` NOT-RUN
> (blocage matériel, Mac M2 éteint). Le quorum a ensuite été JOUÉ →
> **PASS, 1er de l'histoire du projet (C10)** ; review delta Workflow
> (5 agents adversariaux) + Codex round 3 = « gate bloquante levée,
> aucun P0/P1 restant » ; les 2 résiduels P2/P3 du round 3 corrigés
> post-verdict (cf. addendum réconciliation round 3). Suites §7.4
> dual-platform vertes sur l'état final.

## Codex reconciliation (3 passes GPT-5.6 Sol reasoning max)

- **Passe 1** (`sprint81_phase_k_codex_review.md` remplacé passe 2) : FAIL,
  P1 collision contrôle/données (un stage echo interceptait une frame
  ressemblant à une AttestationRequest → prompt echo légitime avalé, casse
  echo byte-identique) + P1 quorum NOT-RUN + 7 P2/4 P3. **Fix collision
  root-cause** : `ShardProtocol::accept` n'intercepte QUE si
  `loaded_stage().is_some()` ET au 1er frame ; un echo n'intercepte plus
  (echoe, driver fail-close) ; +1 test `shard_echo_never_swallows_...` (2
  variantes dont 40-octets) ; test transport-only ajusté (echo n'atteste
  plus → is_err). Tous P2/P3 passe 1 fermés (TOCTOU SI-12 THREAT v17 +
  route S82 ; sweep docs/sharding ; SPEC HUB/driver-signed ; /result 9
  champs ; baseline MIXED ; C9 à-reconfirmer ; compteurs +11/2095 ;
  scratch supprimé+gitignore ; blake3 retry Interrupted).
- **Passe 2** : **P1 collision CLOSED** (vérif code : gate loaded_stage +
  1er-frame, montage prod `ShardStageForwarder.loaded_stage()=Some`, 3
  liens réels attestés avant step, echo byte-identique, fail-close
  préservé). Reste : P1 quorum honnête-pending (hardware) + 3 P2 + 2 P3
  documentaires → **tous fermés depuis** (observe.curl.md « unknown id » ;
  REFERENCE signer=driver + driver↔stage ; EXPLANATION driver signe ;
  MIXED ajouté au vocabulaire contract ; doc-comment `loaded_stage` +
  commentaire test corrigés ; review « 9 champs »). Edge-case Codex
  (client ancien non-conforme sautant l'attestation) : hors chemin driver
  réel, noté résiduel théorique.
- **Suites après fixes** : nextest Win **2095/2095** 0-skip ; fmt/clippy 0 ;
  gates sharding-docs + frontier PASS ; compile core+daemon OK.

## Dispositions (fixes appliqués en Phase K)

- **S81-K-R-1 (P1) — RÉSOLU root-cause.** `ShardStageAttestation::decode`
  (`shard.rs`) valide désormais `model_digest_hex` = exactement 64 chars
  `[0-9a-f]` (rejet loud, aligne le décodeur sur son doc-contract SPEC §5.2) ;
  `digest_prefix_hex` passe en `digest_hex.get(..16).unwrap_or(digest_hex)`
  (jamais un slice par octet) ; test neuf
  `attestation_digest_is_strictly_validated_and_never_panics` (longueur / case /
  charset / multi-octet rejetés + hand-built multi-octet fail-close sans panic).
  nextest Win re-mesuré **2094/2094** (delta K = +10).
- **Gate T2 (acté) — traité.** `sprint81_t2_acceptance.json` : `status`
  top-level + axe transport + `b3_p2_quorum` = `NOT-RUN` HONNÊTE et
  auto-descriptif (rig Mac éteint ce jour) ; la finalisation (jouer le quorum,
  recomputer, ajuster les 3 claims pré-cochés si BLOCK) est la dernière étape
  AVANT `git add` — bloquée sur l'allumage du Mac par l'opérateur.
- **S81-K-R-2 (P2) — RÉSOLU.** Sweep S78 étendu à TOUT `docs/sharding/`
  (EXPLANATION/HOW_TO_WIRE/README/REFERENCE/WIRING_SPEC/llms.txt/3 exemples) :
  statuts LIVE-PROVEN, carries résiduels re-routés S82 ; le gate
  `check-sharding-docs.sh` lui-même requalifié (marqueur S82 résiduel + ancre
  `shard_session_response`, ex-`project_shard_session` symbole mort) — clean.
- **S81-K-R-3 (P2) — RÉSOLU.** SPEC §6 row `/result` corrigée aux 9 champs
  réels de `ShardSessionResultView` (`ttft_s`/`toks_per_s`/`run_proof` hex…).
- **S81-K-R-4 (P2) — RÉSOLU.** J1b-3 (cap `participants` decode) + D3-2
  (`piece` non normalisé) routés `sprint82_audit_plan.md` §3 Sharding.
- **S81-K-R-5 (P2) — RÉSOLU.** C9 requalifié « à RE-CONFIRMER PO »
  (roadmap v5), jamais au passé ; verification.md aligné.
- **S81-K-R-6 (P2) — RÉSOLU.** Palier `k_stage_attestation_binding`
  `PASS`→`ACTED` (invariant code discharged par tests hermétiques, pas une
  acceptance live).

---

**Findings d'origine (historique review Workflow, tous disposés ci-dessus) :**

1. **S81-K-R-1 (P1, CONFIRMÉ 2 dimensions + spot-check code)** — panic distant hors
   char-boundary dans le chemin fail-closed NEUF de K-1. `crates/nexus-core-rs/src/shard.rs:703`
   (`&digest_hex[..digest_hex.len().min(16)]`) slice par index OCTET une `String`
   attaquant-contrôlée : `ShardStageAttestation::decode` (`:665-681`) ne valide NI la
   longueur NI le charset de `model_digest_hex` (contredit son propre doc-contract « 64
   chars lowercase hex » de SPEC §5.2). Un membre admis byzantin qui atteste p.ex.
   `"…é…"` (char multi-octet chevauchant l'octet 16) garantit le mismatch digest
   (`:719`) → branche d'erreur `:723` → `digest_prefix_hex` **panique AVANT
   `sanitize_diagnostic`** (`shard_session.rs:1044`). Aggravant vérifié : `Cargo.toml:481`
   `[profile.release] panic = "abort"` → en build release (les 3 nœuds live du flip H),
   le panic dans la task spawnée (`http.rs:2379`) **abort le PROCESSUS DAEMON ENTIER**
   = crash-DoS distant déclenchable par une seule frame d'attestation malformée. Sous
   unwind (debug), session `Generating` bloquée à vie (`mark_failed` jamais atteint).
   **Viole l'invariant NON négociable « fail-closed jamais silencieux » : un crash n'est
   pas la défaillance proprement enregistrée sur la session.** Le gate de membership
   (résiduel SI-4/N0) le tient hors P0, mais c'est un défaut de correctness DANS le
   livrable K-1, cheap à corriger root-cause. Les 2 dimensions qui l'ont vu concluent
   « fix en Phase K, PAS carry » — signal opérationnel au-dessus du seuil P2. **Fix :**
   (racine) valider dans `decode` que `model_digest_hex` = exactement 64 chars `[0-9a-f]`
   (rejet loud, aligne le décodeur sur son doc-contract) ; (défense) `digest_hex.get(..16).unwrap_or(digest_hex)`
   dans `digest_prefix_hex` ; +1 test hex malformé → Err propre, jamais panic.

2. **Gate de finalisation T2 (acté, ordering DUR — bloque le commit, PAS un défaut de
   design)** — `.planning/active/sprint81_t2_acceptance.json:7`/`:11`/`:61` : `status`
   top-level et `axes.transport.status` = `NOT-RUN` (palier `b3_p2_quorum` en attente du
   Mac M2 éteint), l'artefact portant lui-même « committing a NOT-RUN here would violate
   C10 ». C'est l'état transitoire ANNONCÉ de K-3 (la review l'acte), mais le commit K
   exige : rig Mac allumé → jouer `b3_p2_quorum` en vocabulaire FERMÉ (`PASS` /
   `BLOCK{diagnosis}` — **jamais `RIG-ABSENT`**, machine présente juste éteinte, cf.
   `vocabulary_note:85`) → remplacer le palier par le JSON brut du harness → recomputer
   `axes.transport.status` + `status` top-level, AVANT `git add`. Corollaire : les 3
   claims canon déjà pré-cochés (`sprint81_verification.md:317-318` `[x] T1/T2`,
   `SPRINT_LOG.md:19` row 81, `CLAUDE.md:188-189`) doivent rester cohérents avec le
   résultat final (ajuster si `BLOCK`). Deux dimensions l'ont classé P1 ; je l'acte comme
   gate de finalisation obligatoire, pas comme défaut bloquant la review.

Une fois S81-K-R-1 corrigé (suites §7.4 re-jouées) et T2 finalisé, la review bascule à
PASS-PENDING pour le gate Codex. Les 5 P2 et 15 P3 restants sont NON bloquants (dette
honnêteté docs / planning / couverture), à documenter au commit body et router S82.

## Synthèse exécutive

Phase K livre les 9 sous-livrables K-1..K-9 du wrap-up bi-axe. Périmètre confronté :
diff tracked (22 fichiers) + 6 fichiers neufs, contre HEAD=`43623a5` (fin Phase J).
Six dimensions ont été jouées, chacune avec une passe adversariale de challenge —
**0 finding réfuté, 0 downgrade** sur les 6 dimensions ; les seuls arbitrages de forme
sont deux cas où deux dimensions ont splitté la sévérité du MÊME constat.

**Le cœur K-1 (binding attestation) est fonctionnellement CORRECT et l'invariant de
sécurité tenu** : chokepoint unique tracé exhaustivement (les 2 seules affectations
`st.link = Some(...)` en `shard_session.rs:1396`/`:1461` sont précédées d'un
`attest_stage_link` fail-closed ; l'echo digest-zéros ne sert AUCUNE session réelle),
`verify_stage_attestation` pur/hermétiquement testé sur ses 3 branches, interception
pré-forwarder préservant R-I-1 (mount 0-frame), self-claim jamais sur-claimé (byzantin =
résiduel SI-4/N0 déclaré partout), digest serve via `blake3_hash_file` STREAMING (Codex
P1 J honoré, jamais `std::fs::read`), séparation `REQUEST_KIND`/`REPLY_KIND` fermant le
bypass par réflexion echo. **MAIS** le chemin fail-closed lui-même panique sur un digest
malformé (S81-K-R-1) — un défaut de correctness dans le livrable, pas dans son design.

**Invariants de phase RE-VÉRIFIÉS et TENUS** : 0 bump wire (grep diff `crates/` sur
`*_FORMAT_VERSION`/`DOMAIN_*`/ALPN/`KNOWN_OP_TYPES` = 0 hit code ; `SHARD_ATTEST_PAYLOAD_V`
explicitement doc « NOT a wire *_FORMAT_VERSION », payload JSON dans frame opaque =
pattern J) ; 0 dep (`Cargo.toml`/`Cargo.lock` = 0 octet au diff) ; chemin echo drive
byte-inchangé côté driver (0 hunk sur la marche echo, 4 tests `EchoForwarder` verts) ;
delta tests +9 (2084→2093 Win / 2088→2097 Docker), jamais en baisse ; iroh strictement
seul ; upgrade ≠ Gate 1 (toutes occurrences « Gate 1 » du diff = « R-iroh-audit P0
INCHANGÉ ») ; français docs / anglais code.

La dette restante est majoritairement de l'**honnêteté documentaire** (sweep S78
incomplet dans `docs/sharding/`, 2 champs faux dans une row SPEC, C9 anticipé au passé)
et de la **traçabilité planning** (2 carries J non routés, quelques compteurs stales) —
aucune ne compromet un invariant, toutes fermables mécaniquement ou par une ligne de
routage S82.

## Dimension 1 — Code K-1 attestation (sécurité + correctness, ligne par ligne)

PASS sauf le panic. Chemins d'attestation tracés exhaustivement : aucun bypass de
chokepoint, aucune collision de décode pour le trafic réel (`ShardStepRequest` a des
champs inconnus de `ShardStageAttestationRequest` sous `deny_unknown_fields` bilatéral +
fp32 n'est pas JSON, test-locké bidirectionnel), TOCTOU réfuté (forwarder immuable pour
la vie du process → chute conn → re-dial → re-attestation), sanitisation correcte sinon.
Le SEUL défaut vivant est S81-K-R-1 (panic). Retenus additionnels : S81-K-R-8 (pas de
retry `Interrupted` sur `blake3_hash_file`), S81-K-R-9 (course load-vs-hash non
documentée dans le self-claim), S81-K-R-10 (`ShardBackendForwarder` hérite
`loaded_stage()=None`, 0 call-site de montage, footgun dormant), S81-K-R-11 (pas de
`conn.close` explicite sur lien à attestation rejetée, incohérent avec `:1192-1197`),
S81-K-R-12 (tentative décode JSON sur chaque frame jusqu'à 256 MiB, garde-taille nommée
gratuite).

## Dimension 2 — Code K-2 câblage T1 + tests reuse + CI

PASS. Strip-relay direct-only (`blobs.rs`/`docs.rs`) borné 100×50ms + échec bruyant +
garde `assert!` anti-ticket-vidé, pas de TOCTOU. Les 2 tests self-heal reuse assertent
l'ÉGALITÉ de l'id de namespace (bon discriminant du bras self-heal qui génère un id
aléatoire) — ferment le trou T1(3) hermétiquement. Gating harmonisé `== "1"` exhaustif
(grep repo : tous les setters posent `"1"`, aucun appelant cassé). Job GHA
`integration-nightly.yml` valide, calqué `supply-chain.yml`, couvre les 2 crates
`multi_daemon`, junit uploadé, red-run = signal pas gate. Delta +9 exact.
Retenus : S81-K-R-13 (profil ci kill 90s sans override `binary(multi_daemon)` vs budget
worst-case ~120s spawn séquentiel 2×30s + poll 60s ; bénin aujourd'hui, faux signal aux
réparations S82), S81-K-R-14 (`save-if` master-only absent vs posture anti-poisoning
`rust-ci.yml:94-97`), S81-K-R-15 (branche timeout `attest_stage_link:1054` sans test +
sous-bras `is_first != expect_first` de `shard.rs:735` jamais déclencheur).

## Dimension 3 — Docs sécurité/protocole (honnêteté vs code)

PASS sur le sweep THREAT_MODEL (0 ref S78 vivante non-requalifiée : 8 « ex-S78 → S82 » +
2 historiques v14 verbatim + 2 dans l'entrée v17 décrivant le sweep) et sur §16
attestation (self-claim jamais sur-claimé, mount 0-frame, drop mid-decode « compté pas
simulé »). LOOPBACK §3 (6 lignes) et SPEC §5.1/5.2 collent au code (routes, constantes
`MAX_RESULT_TEXT_BYTES` 64 KiB, `REDACT_HEX_RUN` 32, structs, guards, kinds). Drift-gate
`spec_consts_exist` étendu presence-only (R-K-5 respecté). **MAIS** l'honnêteté docs a 3
trous : S81-K-R-3 (SPEC §6 row `/result` décrit `ttft_ms` + `driver RunProofEntry`, alors
que la vue réelle expose `ttft_s` whole-seconds + `run_proof` = hex de la SIGNATURE ;
`ttft_ms` est justement routé S82), S81-K-R-2 (lot S78 stale, ci-dessous), S81-K-R-7
(qualificatif « session réelle » omis à `THREAT_MODEL:148` + `LOOPBACK:92`, et « byte-identical »
sur-large côté accept-loop).

## Dimension 4 — Artefacts planning

Suites Observed exactes (2093/2091+6 env-block/412/deny 4/4/operator 201+10), delta K +9
vérifié au diff, 27 SHAs résolus, escalade boot-SEED OVERDUE 3/3 BLOQUANTE présente au
sprint82_audit_plan §3, supply-chain complet (RUSTSEC-2026-0185, yanked=deny,
ed25519-dalek 3.0.0). §P73 (4 patterns) ancré au code. Retenus : S81-K-R-6 (palier
`k_stage_attestation_binding` étiqueté `PASS` alors que son évidence hermétique = la
définition exacte d'`ACTED{evidence}` du contrat du fichier ligne 6 ; aucun harness n'a
émis ce PASS ; seul palier PASS sans `artefact`/`run`), S81-K-R-4 (2 carries J routés
« Phase K / audit gate » par le body `43623a5:157-160` — J1b-3 cap `participants` decode
+ D3-2 `piece` non normalisé — ABSENTS du plan [grep 0 hit] et non livrés en K),
S81-K-R-16..R-20 (compteurs/énumérations : « 15 phases » → 16+Phase 0 ×4, « convergence_*
11 scénarios », « 30+ tests » → 26, `bf07960` absent de la stack, row 81 compteurs
symboliques).

## Dimension 5 — Scope cuts + invariants + wire

PASS sur tous les invariants (voir §Scope cuts). Le seul défaut code de la dimension est
le panic (S81-K-R-1, ici en accord avec dimension 1, arbitré P1). Confirme
indépendamment 0 bump wire, `Cargo.*` 0 octet, echo drive byte-inchangé, delta +9,
self-claim jamais sur-claimé. Retenus doc : S81-K-R-7 (SPEC §5.2 « byte-identical »
sur-large côté accept), S81-K-R-20 (SPRINT_LOG « Docker miroir +70 » stale vs +79/2097
déjà écrit dans CLAUDE.md).

## Dimension 6 — Research grounding (préflight PLAN-ADAPT vs livré)

PASS. Les 10 items K-1..K-10 mappés au diff ; l'adaptation mount→stage-link est
argumentée sur pièces et livre un invariant **équivalent-ou-plus-fort** (le stage-link est
le chokepoint que TOUT frame de données traverse, re-dials/fallbacks couverts ; le
mount-time n'aurait attesté qu'une connexion jetée avant le drive). R-K-3 (sur-design)
tenu, R-K-4 (sweep S78 THREAT_MODEL) PASS, aucune décision PO C1..C10/D1..D8 contredite.
Retenus : S81-K-R-2 (sweep S78 arrêté à la lettre — `docs/sharding/` frères stales),
S81-K-R-5 (C9 écrit au passé « re-confirmé au wrap-up » à `roadmap_v5:89` +
`sprint82_audit_plan:144` alors que `verification.md:325-326` le laisse `[ ]` non
tranché), S81-K-R-21 (détection misconfiguration différée du mount au 1er generate —
sécurité intacte, candidat early-detection S82).

## Findings retenus (post-adversarial)

### Bloquant

- **S81-K-R-1 (P1)** — `crates/nexus-core-rs/src/shard.rs:703` (+ `:665-681`, `:723`,
  `Cargo.toml:481`) — panic distant hors char-boundary sur `model_digest_hex` non
  validé, AVANT sanitisation, sous `panic="abort"` release = crash-DoS daemon entier ;
  viole « fail-closed jamais silencieux ». **Fix : valider 64-hex lowercase dans
  `decode` (rejet loud) + `get(..16).unwrap_or` défensif + 1 test non-panic.** À
  appliquer EN Phase K (cheap root-cause, pattern H/J).

### P2 (dette honnêteté / planning — NON bloquant, commit body + route S82)

- **S81-K-R-2** — `docs/sharding/WIRING_SPEC.md:51`/`:58` (« (S78) signs a RunProof » /
  « RUN-PROOF(S78) ») contredisent la section RUN-PROOF requalifiée « driver emission
  LIVE since S81 I/J » DU MÊME fichier (et `:51` est faux : c'est le DRIVER qui signe,
  per-worker → S82) ; les 6 frères non touchés sont stales sous l'index `llms.txt`
  fraîchement « LIVE-PROVEN » — dont `HOW_TO_WIRE.md:77` factuellement FAUX (« Sans store
  de session live (carry S78), la route répond {found:false} » alors que le registre est
  live depuis Phase I). `sprint81_verification.md:321` (« docs/sharding requalifiés »)
  sur-claime. **Fix : corriger les 2 lignes WIRING_SPEC + les headers de statut des
  frères, OU borner le claim verification.md et router le lot nommément dans
  `sprint82_audit_plan.md` §3.**
- **S81-K-R-3** — `docs/protocol/SHARD_PROTOCOL_SPEC.md:264` : row `/result` décrit
  `ttft_ms` + `driver RunProofEntry`, inexistants sur la route ; la vue réelle
  `ShardSessionResultView` expose `ttft_s` (whole seconds) + `run_proof` (hex de la
  signature RunProof), et `ttft_ms` est explicitement routé S82 (audit_plan:111).
  **Fix : corriger les 2 champs faux de la row.** (Le drift-gate est presence-only,
  aveugle aux noms de champs des rows §6 — c'est par là que le finding est passé.)
- **S81-K-R-4** — `.planning/active/sprint82_audit_plan.md` §3 Sharding : J1b-3 (cap
  `participants` decode, edge churn plan large) + D3-2 (`reply.piece` non normalisé)
  routés « Phase K / audit gate » par `43623a5:157-160`, absents du plan (grep 0 hit) et
  non livrés en K. Règle kickoff « chaque review route » violée. **Fix : ajouter les 2
  items à §3.**
- **S81-K-R-5** — `.planning/roadmap_v5_factory_complete_vision.md:89` +
  `.planning/active/sprint82_audit_plan.md:144` affirment C9 « re-confirmé au wrap-up »
  (passé) alors que `sprint81_verification.md:325-326` le laisse `[ ] DEMANDÉS … jamais
  tranchés ici` (C9 BLOQUANT, discipline DEMANDER). **Fix : reformuler « re-confirmation
  DEMANDÉE au wrap-up » (forme correcte `CLAUDE.md:239`) ou obtenir l'ack PO avant
  commit.**
- **S81-K-R-6** — `.planning/active/sprint81_t2_acceptance.json:73-75` : palier
  `k_stage_attestation_binding` étiqueté `PASS` avec évidence hermétique = définition
  exacte d'`ACTED{evidence}` (contrat ligne 6) ; aucun harness n'a émis ce PASS.
  **Fix : relabel `"status": "ACTED"` (évidence inchangée).**

### P3 (doc-honnêteté / couverture / robustesse — commit body / S82)

- **S81-K-R-7** — `SPEC §5.2` (« byte-identical to S77 Phase B ») + `THREAT_MODEL.md:148`
  + `LOOPBACK_ENDPOINTS_TRUST_TIERS.md:92` : l'interception d'attestation est dans
  l'accept-loop PARTAGÉ (toute session de l'ALPN) ; « byte-identical » ne vaut que côté
  DRIVER, et le qualificatif « d'une session réelle » manque aux 2 sommaires. Fix : une
  phrase d'honnêteté par site, 0 code (miroir de §P73(1) correct).
- **S81-K-R-8** — `crypto.rs:199-207` : `blake3_hash_file` ne retry pas `ErrorKind::Interrupted`.
  Fix : `continue` sur Interrupted, ou `std::io::copy(&mut file, &mut hasher)`.
- **S81-K-R-9** — `main.rs:317-337` : course load-vs-hash non documentée (digest = octets
  fichier au moment du hash, pas VRAM). Fix : 1 ligne commentaire + THREAT_MODEL §16.
- **S81-K-R-10** — `crates/nexus-worker-core/src/llm/shard.rs:641-649` :
  `ShardBackendForwarder` (backend réel) hérite `loaded_stage()=None`, 0 call-site de
  montage. Fix : doc-note « ne jamais monter en session réelle attestée » ou param digest.
- **S81-K-R-11** — `shard_session.rs:1386-1396`/`:1446-1460` : pas de `conn.close(...)`
  explicite sur la connexion à attestation rejetée (incohérent `:1192-1197`). Fix :
  `link.conn.close(0u32.into(), b"attestation-rejected")` dans les 2 branches d'erreur.
- **S81-K-R-12** — `shard.rs:332` : tentative décode JSON sur chaque frame jusqu'à
  256 MiB. Fix : constante nommée `SHARD_ATTEST_REQUEST_MAX_BYTES` + garde-taille avant
  `decode`.
- **S81-K-R-13** — `.github/workflows/integration-nightly.yml:62-65` + `.config/nextest.toml:47-67` :
  profil ci kill 90s sans override `binary(multi_daemon)` (budget worst-case ~120s).
  Fix : `[[profile.ci.overrides]]` 60s×3, ou noter la dette calibrage S82.
- **S81-K-R-14** — `.github/workflows/integration-nightly.yml:43-48` : `save-if`
  master-only absent (posture anti-poisoning `rust-ci.yml:94-97`). Fix : 1 ligne.
- **S81-K-R-15** — (i) `shard_session.rs:1054` branche timeout `attest_stage_link` sans
  test ; (ii) `shard.rs:735` sous-bras `is_first != expect_first` jamais déclencheur.
  Fix (ii) = 1 assertion pure ; (i) = note d'acceptation verification.md.
- **S81-K-R-16** — « 15 phases » faux à `verification.md:5`, `CLAUDE.md`, `SPRINT_LOG.md:19`,
  `sprint82_audit_plan.md:19`/`:47` (réel = 16 phases + Phase 0). Fix : « 16 phases
  (+ Phase 0) ».
- **S81-K-R-17** — `verification.md:94` « convergence_* (11 scénarios) » (3 fn
  `convergence_`, 12 tests dans le groupe `two-node-convergence`) + commentaire
  `.config/nextest.toml:20` « all eleven » stale. Fix : libellé exact.
- **S81-K-R-18** — `verification.md:120-122` : `shard_session::tests::*` = 26 tests, pas
  « 30+ ». Fix : écrire 26.
- **S81-K-R-19** — `verification.md` §2 + `sprint82_audit_plan.md:20` : `bf07960`
  (in-window) absent de la stack, « 4 chores acceptance » n'en nomme que 3. Fix : ajouter
  bf07960 + corriger l'énumération.
- **S81-K-R-20** — `SPRINT_LOG.md:19` row 81 : compteurs symboliques (« final au body K »,
  « Docker miroir +70 ») alors que 2093/+79/2097 sont connus et inlinés dans CLAUDE.md.
  Fix : inliner « 2014→2093 (+79) / Docker 2018→2097 ».
- **S81-K-R-21** — `shard_session.rs:1386` : détection misconfiguration différée du mount
  au 1er generate (conséquence assumée de l'adaptation stage-link, mount 0-frame ;
  sécurité intacte). Fix : tracer « early-detection au readiness barrier » comme candidat
  S82 dans `sprint82_audit_plan.md` §3.

## Scope cuts + invariants vérifiés

- **0 bump wire** : grep diff `crates/` sur `*_FORMAT_VERSION`/`ANNOUNCEMENT_VERSION`/
  `DOMAIN_*_V1`/ALPN/`KNOWN_OP_TYPES` = 0 hit code ; `shard_plan.rs` (structs signés) 0
  octet ; `SHARD_ATTEST_*` = payload JSON in-frame opaque, `deny_unknown_fields` +
  discriminant `kind` + cross-rejet bidirectionnel = pattern J. ✓
- **0 dep / iroh seul** : `Cargo.toml`/`Cargo.lock` 0 octet au diff ; seuls ALPN au diff
  = mentions doc. ✓
- **Chemin echo byte-inchangé (driver)** : `attest_stage_link` appelé UNIQUEMENT depuis
  `drive_decode_loop` (`:1386`/`:1446`), gaté `model_digest != 0` (`:1079`) ; `drive_pipeline`
  transport/echo = 0 attestation ; 4 tests `EchoForwarder` verts. ✓ (Nuance côté
  accept-loop = S81-K-R-7, doc-only.)
- **fail-closed** : les 3 branches de `verify_stage_attestation` + les 3 tests session +
  interception-avant-forwarder couvrent l'invariant « aucun step frame vers un exec non
  atteste ». ✓ (Défaut : le fail-closed panique au lieu d'enregistrer — S81-K-R-1.)
- **attestation = self-claim, jamais sur-claim byzantin** : cadrage MISCONFIGURATION /
  SI-4 résiduel uniforme code/spec/THREAT/aggregate `honest_labels`. ✓
- **total tests jamais en baisse** : +9 (2093 Win / 2097 Docker), 0 supprimé. ✓
- **upgrade ≠ Gate 1** : toutes occurrences « Gate 1 » = « R-iroh-audit P0 INCHANGÉ ». ✓
- **français docs / anglais code** : respecté. ✓

## Note quorum pending rig (T2 K-3) — la review l'acte

L'agrégat `sprint81_t2_acceptance.json` est HONNÊTE et auto-descriptif : 9 paliers
transport PASS/ACTED (équivalences b3-fetch-blob + PublicRegistryView ACTÉES avec
évidence nommée du flip H + probe Phase K), sharding PASS (J live + K binding
hermétique), et le SEUL `NOT-RUN` est `b3_p2_quorum` (Mac M2 éteint 2026-07-11 — un
« wait », JAMAIS `RIG-ABSENT`, conforme D4/`vocabulary_note:85`). C'est la 1ère quorum
PASS de l'historique projet visée (cible C10). La review ACTE que l'agrégat est finalisé
AVANT le commit K : rig allumé → `b3_p2_quorum` en vocabulaire fermé → JSON brut embarqué
→ statuts recomputés → claims canon ajustés si `BLOCK`. Tant que ce séquencement est tenu,
ce n'est pas un défaut de livrable ; c'est le gate de fermabilité par-sprint (T2 acceptance
machine-lisible, plus jamais un `DIFFERE-materiel` en prose).

## Prochaine étape

Verdict FAIL car 1 P1 CONFIRMÉ (S81-K-R-1, panic du chemin fail-closed neuf). Séquence de
sortie :
1. Corriger S81-K-R-1 en Phase K (validation 64-hex dans `decode` + slice défensif +
   test non-panic) ; re-jouer les blocs §7.4 dual-platform + docs-gates.
2. Finaliser T2 (jouer `b3_p2_quorum` rig Mac allumé → recomputer l'agrégat), ajuster les
   3 claims canon si `BLOCK`.
3. (Optionnel, cheap) balayer les P2 honnêteté docs S81-K-R-2/R-3/R-5/R-6 et router
   S81-K-R-4 dans `sprint82_audit_plan.md` — sinon les documenter au commit body.
4. Re-review → PASS-PENDING → gate Codex GPT-5.6 Sol reasoning max (le panic distant est
   précisément le genre de défaut que Codex bloquerait ; le corriger avant évite une
   boucle).

## Addendum finalisation quorum — 2026-07-11 (post-run rig, review delta Workflow)

Le SEUL RÉSIDUEL du verdict ci-dessus (`b3_p2_quorum` NOT-RUN, blocage
matériel) est **DÉCHARGÉ** : le palier a été joué le 2026-07-11 avec le rig
complet et est **PASS — 1er quorum de l'histoire du projet (C10)**. Les
sections de cette review qui décrivent l'agrégat T2 à l'état `NOT-RUN`
(« Note quorum pending rig », citations `:7/:11/:61 = NOT-RUN`) documentent
l'état TRANSITOIRE d'avant finalisation et sont remplacées par ce qui suit.

- **Run quorum** : séquence 3 runs honnête, consignée dans l'agrégat
  (`sprint81_t2_acceptance.json`, palier `b3_p2_quorum`) avec contrat de
  provenance explicite (raws committés : run 1 BLOCK
  `sprint81_t2_quorum_k_block.json` + run 3 PASS
  `scripts/acceptance/.b3_quorum_k.json` embarqué ; attributions per-worker
  = logs opérateur non committés, dits tels). Run 1 BLOCK consent-opt-in
  (valide A3/A4 : la tâche ATTEINT la réplique worker over WAN) → geste
  opérateur `ConsentLevel::All(4)` sur les 2 hôtes (opt-in S76) → run 2
  BLOCK stage=claim (boot-froid worker PC 3s avant submit ; signal produit
  routé `sprint82_audit_plan.md`, famille S75 re-drive-on-ingest ; la même
  tâche converge +2m08 une fois le worker stable) → run 3 **PASS 6s**
  end-to-end (budget 30s), 2 identités distinctes PC `a424d8e748` + Mac
  `81cfeab05c`, `result_text` "Paris." répliqué au VPS.
- **Agrégat T2 recomputé** : top-level `PASS` bi-axe ; axe transport `PASS`
  (baseline_098 `MIXED` = référence différentielle, qualifiée « had never
  PASSed pre-bump ») ; axe sharding `PASS` inchangé.
- **Review delta Workflow (5 agents adversariaux)** post-finalisation :
  P2/P3 d'honnêteté documentaire tous traités in-phase — provenance
  narrative T2 (runs 1/2/3 requalifiés « operator-inspected/corroborated,
  uncommitted »), task id run 2 ajouté, référence raw run 1 ajoutée,
  baseline « never PASSed » qualifiée pré-bump, résidu adjacence
  `EXPLANATION.md` reformulé HUB, LT-7 « Worker quorum E2E » CLOSED dans
  `CLAUDE.md`, doc-comments hérités corrigés (`shard.rs`
  `LoadedStageDescriptor` + `from_loaded_stage` bras `None`,
  `ShardBackendForwarder` note footgun `loaded_stage=None`, commentaire
  `retries=1` re-scopé `[profile.ci]` entier, notes « second boot =
  same-process » sur les 2 tests A2). Constat agents : P2 Codex #1/#2/#3
  CONFIRMÉS corrigés, `check-sharding-docs.sh` + `check-frontier-contracts.sh`
  exit 0.
- **Suites re-jouées post-fixes (dual-platform)** : Win `cargo fmt --check`
  + clippy + nextest **2095/2095 0-skip** + doctests + release daemon
  VERTS ; Docker canonique `sbfb-ci` fmt + clippy + nextest **2099/2099
  0-skip** + doctests VERTS ; bloc web complet VERT (lint + tsc + vitest +
  coverage + build + size-limit 6/6 + scan-en-strings). Un re-run final
  post-doc-comments est consigné avant commit.

Reste avant commit : gate Codex GPT-5.6 Sol round 3 sur l'état final →
réconciliation → promotion du header à `## Verdict: PASS`.

## Réconciliation Codex round 3 (2026-07-11, post-quorum)

Verdict brut round 3 (`sprint81_phase_k_codex_review.md`, output
`codex exec` non réécrit) : **« Gate bloquante levée : aucun P0/P1
restant »** — « PASS avec résiduels P2/P3, pas CLEAN ». Points 2/3/4/6/7/8
CORRIGÉ ; le run quorum est accepté sous le modèle de preuve déclaré
(« run matériel attesté par l'opérateur, logs non committés clairement
qualifiés dans l'agrégat »). Les 2 résiduels, corrigés post-verdict dans
la même fenêtre pré-commit (critère d'arrêt boucle : P2/P3 documentés) :

- **P2 propagation de provenance** (points 1/9) : le qualificatif
  « operator-corroborated / logs non committés » présent dans l'agrégat
  manquait sur les 4 surfaces de synthèse → ajouté à
  `sprint81_verification.md` (bloc Provenance), `CLAUDE.md` (segment T2),
  `SPRINT_LOG.md` row 81 (segment quorum), `sprint82_audit_plan.md`
  (évidence cold-boot : timings operator-corroborated, raw run 2 écrasé
  par run 3).
- **P3 commentaire de test all-zero** (point 5) : `shard.rs` test du
  binding — le commentaire affirmait que l'all-zeros couvre « echo left
  serving in a real session » (ancien modèle pré-gate d'interception) →
  reformulé : l'all-zeros ne provient que d'un appel direct
  `from_loaded_stage(None)` ; l'echo en session réelle renvoie la requête
  (chemin couvert par les tests fail-closed de décodage).

Gates re-jouées après ces 2 fixes : `cargo fmt --all --check` +
`cargo nextest run -p nexus-core-rs` (le seul crate au diff post-round-3).
Les suites complètes §7.4 dual-platform avaient déjà tourné vertes sur
l'état pré-fixes (Win fmt/clippy/nextest 2095 0-skip/doctests/release ;
Docker fmt/clippy/nextest 2099 0-skip/doctests ; web complet) — les 2
fixes sont 1 commentaire de test + 4 qualificatifs de prose planning.
