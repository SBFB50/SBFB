# Sprint 82 Phase H — Review

Date : 2026-07-14. Review Workflow ultracode : 11 agents opus-4-8[1m]
(6 dimensions — diff intégral, sémantique des tests, fidélité PATTERNS,
scope/PLAN-ADAPT, sécurité+wire, livrables+process — + vérification
adversariale par dimension ; pipeline sans barrière ; 963k tokens,
125 tool calls). Arbre stable pendant la review (aucune édition
concurrente) ; les suites §7.4 avaient déjà rendu leur verdict et sont
passées en contexte figé aux agents.

## Verdict: PASS

Review Claude : 0 P0, 1 P1 (corrigé in-phase), 1 P2 (= le même défaut,
upgradé par la dimension process, même fix), 4 P3 (2 corrigés
in-phase, 2 dispositions consignées). Codex GPT-5.6 Sol round 1 :
4 CONFIRMÉ / 5 PARTIEL / 0 GAP — 6 écarts factuels/présent-vrai TOUS
corrigés in-phase et re-prouvés machine + 1 faux positif contextuel
documenté (cf. § Codex reconciliation). Verdict promu PASS après
réconciliation.

## Findings et dispositions

- **DC-H-ZIP-1 (P1, CONFIRMED) / H-PROC-1 (P2, UPGRADE — même défaut)**
  `docs/shell/PATTERNS.md` P21 : la prose AJOUTÉE par la phase disait
  « crate `zip`, 8.5 au lock » alors que `Cargo.lock` résout
  **zip 8.6.0** (8.5 = la contrainte déclarée `Cargo.toml:204`). Une
  nouvelle inexactitude factuelle dans une phase dont le livrable EST
  la fidélité — la classe exacte que la phase corrige.
  **CORRIGÉ in-phase** : « 8.6.0 au lock, contrainte declaree 8.5 —
  2.6 a l'origine S12 » + balayage du preflight au même geste (leçon
  Phase G : l'erreur d'un scan se propage dans tout artefact qui le
  recopie — le « 2.6→8.5 » du preflight §Passe de fidélité corrigé en
  « 2.6→8.6.0 » avec trace du fix). Re-vérifié machine :
  `grep -A2 '^name = "zip"' Cargo.lock` → 8.6.0.
- **H-PROC-3 (P3, CONFIRMED)** Preflight : tally « 3 UPGRADE-P2 »
  incohérent avec la ligne zip (4e mouvement P3→P2). **CORRIGÉ
  in-phase** : « 4 UPGRADE-P2 : frost, sybil-tail, perf-map seam,
  zip » + ligne zip clarifiée (le tally 14 P2 + 7 P3 est inchangé et
  re-réconcilié : 10 CONFIRMED-P2 + 4 UPGRADE-P2 / 7 P3).
- **H-D1-1 (P3, CONFIRMED)** Tripwire : l'assertion d'égalité EXACTE
  du listing couple la détection du suffixe à la propreté des temp
  (faux-positif possible si un futur upstream laisse un artefact
  annexe). **DISPOSITION : strictesse CONSERVÉE comme invariant
  volontaire** (« exactement un sibling ») — documentée dans le
  rustdoc du tripwire (ajout in-phase), qui explicite aussi le coût
  accepté. Couvre en même temps **H-D2-1 (P3, CONFIRMED)** : le
  chevauchement partiel avec `backup.exists()` du test frère est réel
  et VOULU — la valeur ajoutée (épingler le NOM produit + refuser tout
  sibling inattendu) est désormais dite dans le rustdoc.
- **H-D4-1 (P3, CONFIRMED)** L'extraction `forge_legacy_store` +
  consts `FX_*` n'était pas prescrite mot-pour-mot par le preflight
  (qui n'autorisait que « +1 test tripwire »). Refacto défendable au
  service du tripwire (0 duplication du forge), valeurs
  byte-préservées vérifiées par la dimension 1. **DISPOSITION :
  tracée dans le commit body (section Scope cuts / G8).**
- **H-PROC-2 (P3, CONFIRMED)** Le critère machine du plan cite
  `runtime.rs:2580`, le preflight :2751, la fn est aujourd'hui
  à :2754 — le numéro re-pourrit à chaque édition. **DISPOSITION : le
  commit body ancre par SYMBOLE `docs_migration_backup_path`, jamais
  par numéro de ligne** (cohérent avec le passage de la phase aux
  ancres-symbole dans PATTERNS).

## Dimensions (assessments)

1. **Diff intégral** : diff propre, 100 % dans le périmètre, refactor
   Rust strictement conservatif (`OsString::push` sans séparateur →
   chemin byte-identique ; guard et 2 tests daemon inchangés en
   comportement) ; const hors canonical/schemas/_VERSION → census 25 +
   Check 4 intacts ; re-export lib.rs trié ; aucun hunk parasite.
2. **Sémantique des tests** : le tripwire prouve le critère machine
   (migration RÉELLE via `DocsStore::persistent`, assertion du set
   exact, échec bruyant sur rename upstream) ; refacto FX_*
   byte-préservée ; delta +1 honnête (3→4 tests dans le binaire) ;
   pas de trou : le daemon dérive du MÊME const (pas d'une copie).
3. **Fidélité PATTERNS** : chaque claim nouveau re-vérifié au disque —
   tous fidèles sauf zip (P1 ci-dessus, corrigé) ; 0 promesse future
   dans la prose ajoutée ; annotations datées vers le passé immuable.
4. **Scope/PLAN-ADAPT** : 6 corrections + 6 étapes toutes
   matérialisées ; 21/21 fixes fidélité présents dans le diff ; defer
   T20 tenu (0 câblage TLS) ; note Track C exacte, C-4/C-5 légitimement
   subsumés (ledger E :116 les route Phase H).
5. **Sécurité + wire** : PASS, 0 finding — 0 changement de posture
   (annotation T20 purement navigation, « stays OPEN » explicite),
   0 bump wire, const = nom de fichier local hors wire, commentaires
   in-code ajoutés passent PROMISE_RE branche par branche.
6. **Livrables + process** : artefacts présents et cohérents ;
   testabilité §217-218 satisfaite (pointeur T20 résout vers node.rs ;
   critère tripwire satisfait via const partagée, correction :2580
   consignée) ; frontier_closure N/A justifié (aucune frontière neuve :
   const interne, pas de DTO loopback) ; langue conforme (PATTERNS
   anglais, planning français).

## Suites §7.4 (jouées avant review, re-runs post-fix)

- fmt Win 0 (re-run post-fix) + Docker 0 ; clippy workspace
  all-targets vert ; doctests verts ; release daemon vert.
- nextest Win **2100/2100 0-skip** (2099+1) — run idle ; familles
  `dispatch_loop::tests::boot_*` flaky UNIQUEMENT sous charge croisée
  (3 runs : échecs différents à chaque fois, PASS solo 6-12s et PASS
  au run idle complet) — classe env, pas une régression (le delta de
  phase produit un chemin byte-identique et ne touche pas dispatch_loop).
- nextest Docker sbfb-ci `--no-fail-fast` : **2104 total (2103+1),
  2098 verts + 6 fails = la classe env-instable documentée**
  (`multi_daemon` ×4 + `cross_daemon_blob` + `blob_serve_coep`,
  timeouts 30s réseau hôte Docker-on-Windows — CLAUDE.md : jamais
  compté régression, verts Win natif + Woodpecker).
- Binaire `store_migration` re-run post-fix rustdoc : 4/4 PASS.
- Web : lint + tsc verts ; Vitest **412/412** (re-run idle après 3
  flaky sous charge, classe `vitest_env_variance`) ; coverage vert ;
  build + size 6/6 verts ; scan-en-strings clean.
- Gates docs : check-frontier-contracts « clean, 25 frozen » +
  check-sharding-docs + check-factory-docs + check-spdx 352 — verts.

## Codex reconciliation

Rapport brut : `sprint82_phase_h_codex_review.md` (output `codex exec
-m gpt-5.6-sol -c model_reasoning_effort=max -o`, non réécrit).
Round 1 : **4 CONFIRMÉ (L1 const, L2 daemon, L3 tripwire, L4 T20) /
5 PARTIEL (L5-L9) / 0 GAP**. Chaque écart re-vérifié sur pièces par le
main thread AVANT correction (jamais sur parole) :

- **L6-1 P36 quorum (CONFIRMÉ réel, corrigé)** : `validator.rs`
  bras majorité → `Accepted` avec outliers loggés ; bras sans
  majorité → `Rejected` terminal (S74 B.2). Ma prose conflatait les
  deux bras (« logged as outliers and the task rejected ») — reprise
  du doc-comment ambigu du code. Réécrit bras par bras. L'intro §P36
  « hash of canonical result bytes » (pré-existante, non éditée au
  round 1) re-cadrée ère-Python vs `result_text` exact. Et
  `#[serde(default)]` précisé `#[serde(default =
  "default_redundancy_factor")]` (task.rs).
- **L6-2 plain-fetch (CONFIRMÉ réel, corrigé)** : « No plain-fetch
  exception remains » sur-affirmait — `FileUploadBlock.tsx:53-56`
  poste un multipart en `fetch` brut vers `127.0.0.1:18765` avec cast
  non-Zod (vérifié sur pièce). P1 shell reformulé : exception AppsTab
  fermée, exception FileUploadBlock DITE, bootstrap auth.ts noté.
- **L6-3 safeParse (CONFIRMÉ réel, corrigé)** : `daemon.ts` renvoie
  les non-2xx en `{kind:"error"}` AVANT tout parse — « on every
  response » corrigé en « on every SUCCESS payload » avec le
  court-circuit explicite.
- **L6-4 kudos ledger (CONFIRMÉ réel, corrigé)** :
  `kudos_ledger::credit` ne prend aucune clé (chaîne BLAKE3 non
  signée, vérifié sur pièce) — le rôle 3 de `pow_keypair` (§P43)
  re-scopé au task signing seul, la couverture Python-era du ledger
  dite au passé.
- **L5/L9 présent-vrai (CONFIRMÉ, corrigés)** : « handled in S82
  Phase I » → « routed to S82 Phase I » (la Phase I n'a pas encore
  été jouée) ; note Track C « TOUS corrigés dans le commit Phase H »
  reformulée (les corrections et la note voyagent dans le même commit
  atomique) + trace du round Codex ajoutée à la note.
- **L9 zip en français (FAUX POSITIF contextuel, documenté, 0 édit)** :
  la section P21 de `docs/shell/PATTERNS.md` est historiquement
  rédigée en FRANÇAIS (tout le paragraphe environnant) — l'invariant
  « prose PATTERNS en anglais » du prompt Codex est le style
  MAJORITAIRE du fichier, pas une règle par-ligne ; la cohérence
  locale de section prime. Aucune édition.
- **Note hors décompte (review PASS-PENDING)** : attendu — l'audit
  Codex a couru AVANT cette réconciliation ; le présent fichier est
  promu PASS par elle.

Critère d'arrêt (memory codex-loop-stop) : 0 P0/P1 Codex ; écarts
P2/P3 doc-only tous corrigés in-phase + re-prouvés machine
(`validator.rs:291-334`, `FileUploadBlock.tsx:53-68`,
`daemon.ts:258-295`, `kudos_ledger.rs:76`, `task.rs:279`,
`Cargo.lock` zip 8.6.0) → boucle stoppée au round 1, pas de re-run
Codex requis. Suites : fixes doc-only (0 source Rust/web touchée par
la réconciliation) — gates docs re-joués verts au commit.
