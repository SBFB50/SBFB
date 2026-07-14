# Sprint 82 Phase G — Preflight (G8)

Date : 2026-07-14. Phase G schématise les 3 request-bodies loopback
shard-session (`ShardGroupMintRequest`, `ShardGenerateRequest`,
`MountSessionRequest`), fige la métrique des familles `DOMAIN_*_V1`
non-schématisées (fin du flottement 21/22/23), acte D8
(accept-and-close incrémental du registre FRONTIER) et qualifie
S80-G-1 doc-lint (3 reports — décision fermante, jamais un 4e report).
Preflight ultracode = Workflow 12 agents (6 scans S1a/S1b/S2/S3/S4/S5
+ 6 vérifications adversariales par scan, pipeline sans barrière,
opus-4-8[1m]). Toutes les ancres ci-dessous re-vérifiées au disque le
2026-07-14 par DEUX passes indépendantes (scan + vérif adversariale ;
arbre propre, tip `21674f5`). Gates AVANT : `check-frontier-contracts.sh`
exit 0 « [1 tagged] » + `check-sharding-docs.sh` exit 0 (exécutés réels).

## Verdict: PLAN-ADAPT

Le plan est exécutable, aucune décision Day-0/PO n'est contredite
(aucun DESIGN-CONFLICT : le repli (b) est déjà dans l'enveloppe du
plan, et la question [G] du kickoff déléguait explicitement l'arbitrage
(a)/(b) au preflight). Mais **six faits du plan sont incomplets ou
inexacts** et imposent une exécution corrigée :

1. **« Option (a) recommandée » n'est PAS applicable uniformément aux
   3 request-bodies — le traitement est HÉTÉROGÈNE par construction.**
   `ShardGroupMintRequest` (http.rs:2195 — `group_id:String`,
   `members:Vec<String>`, `revision:Option<u64>`) et
   `ShardGenerateRequest` (http.rs:2308 — `session_id:Option<String>`,
   `prompt:String`, `max_tokens:Option<u32>`) sont 100 % primitifs :
   option (a) propre. `MountSessionRequest` (shard_session.rs:215)
   porte DEUX bloqueurs STRUCTURELS (pas un « churn de dep », le seul
   déclencheur de repli prévu au plan) : (i) `group: ComputeGroupEntry`
   (shard_session.rs:222) est l'enveloppe SIGNÉE que la doctrine exclut
   délibérément de la schématisation (« signed envelopes intentionally
   NOT schematised », schemas/shard.rs:24-32 + SHARD_PROTOCOL_SPEC.md:102-106 ;
   compute_group.rs:160 ne dérive PAS JsonSchema, signature `[u8;64]`
   via BigArray :174-175) — un derive récursif CONTREDIRAIT ce claim
   documenté ; (ii) `workers[].addr: iroh::EndpointAddr`
   (shard_session.rs:103/:179) n'a AUCUNE impl JsonSchema dans le repo
   (grep `impl JsonSchema for` = 0 hit) → `#[derive(JsonSchema)]` NE
   COMPILE PAS sans `#[schemars(schema_with)]` custom dont la forme
   JSON est possédée par iroh (drift au pin =1.0.1). Le move-to-core ne
   résout PAS cet obstacle : core re-exporte `iroh::EndpointAddr` sans
   JsonSchema (doc_sync.rs:79). → **Exécution : option (a) pour les 2
   bodies http.rs, repli (b) motivé pour MountSessionRequest** —
   exactement le repli déjà prévu au plan, cause corrigée (structurelle,
   pas dep).

2. **L'option (a) exige une décision de layering absente du plan — la
   voie est le DÉPLACEMENT vers core-rs (a2), précédent exact S77
   Phase L.** Les 3 structs vivent dans le daemon, qui n'a PAS schemars
   (Cargo.toml : 0 hit ; pin workspace `schemars = 1.2 features=["derive"]`
   Cargo.toml:360, déjà au lockfile avec schemars_derive 1.2.1 — S1b :
   AUCUN churn de lockfile). Core est dépendance du daemon (jamais
   l'inverse) : le mécanisme snapshot (`schema_snapshots()`
   schemas/shard.rs:281-309 + test `shard_schema_snapshot_matches_struct`
   shard.rs:525-555, refresh `UPDATE_SNAPSHOTS=1`) ne peut PAS
   référencer un type daemon. Le rationale S77-L est verbatim au disque
   (« Defined in nexus-core-rs (not the daemon) so its schema_for! can
   live next to the other shard schemas … the type lives where the
   schema is generated, and the daemon consumes it », shard.rs:68-72 +
   http.rs:2133-2139, commit 744f84a) — les DTO réponses ont été
   DÉPLACÉS pour cette raison précise. a1 (schemars en dep directe du
   daemon + infra snapshot re-créée daemon-side) est possible (idiome
   zeroize, daemon/Cargo.toml:70-75) mais duplique l'infra et ajoute
   une arête. → **Exécution : a2 — déplacer `ShardGroupMintRequest` +
   `ShardGenerateRequest` vers `nexus-core-rs/src/schemas/shard.rs`
   (elles passent de privées à `pub`), le daemon les importe ; +2
   entrées `schema_snapshots()` + 2 `*.schema.json` committés (même
   commit, sinon drift-test rouge) + tests whitelist miroir
   (:450/:492).** 0 changement Cargo.toml.

3. **Un tag `// FRONTIER:` littéral sur les 3 structs FERAIT ÉCHOUER le
   gate — la lettre de D8 est mécaniquement insatisfiable, son esprit
   est satisfait par le pattern précédent.** check-frontier-contracts.sh:150-157
   exige `const $domain` ET `const $version` déclarées sous crates/
   pour tout tag ; les 3 bodies (loopback, non signés) n'ont ni
   `DOMAIN_*_V1` ni `*_FORMAT_VERSION`. `FRONTIER-NO-SCHEMA` n'est
   consulté QUE dans la boucle des types déjà tagués (:160-163). Le
   précédent CANONIQUE de la classe exacte existe : `ShardSessionResultView`
   (shard.rs:121-126, S81 K) — « Not a signed wire type, so it carries
   no `// FRONTIER:` domain/version tag (that opt-in registry is for
   `DOMAIN_*_V1` families); its machine contract is the generated
   schema snapshot ». Le registre réel = 1 SEUL tag (`shard_plan.rs:188`,
   garde anti-suppression gate:172-175), 0 FRONTIER-NO-SCHEMA. →
   **Exécution : les 3 request-bodies suivent le pattern
   ShardSessionResultView (snapshot drift-gaté + commentaire prose
   « Loopback-API frontier » pour les 2 schématisées ; prose motivée
   équivalente pour MountSessionRequest), AUCUN tag registre.** L'esprit
   de D8 (chaque frontière NEUVE porte un contrat machine + décision
   consciente accept-and-close) est tenu plus fortement par le
   drift-gate nextest que par un tag que le gate rejetterait.

4. **Le critère machine du plan est imprécis — les 2 gates shell ne
   « couvrent » rien automatiquement.** Ils sont DÉJÀ exit 0 et le
   resteraient sans aucun livrable : check-frontier-contracts (2) est
   opt-in (« UNannotated wire types are NOT violations », :41-44) ;
   check-sharding-docs n'a AUJOURD'HUI zéro référence aux 3 structs
   (REQUIRED_ANCHORS :208-210). Le VRAI drift-gate des schémas est le
   test Rust `shard_schema_snapshot_matches_struct` (shard.rs:525 — PAS
   :317, ancre corrigée par la vérif adversariale) exécuté par
   `cargo test --workspace --locked` en CI (.woodpecker/ci-linux.yml:38)
   et nextest en local. → **Exécution : matérialiser la couverture —
   ajouter aux REQUIRED_ANCHORS de check-sharding-docs.sh les 3 structs
   (rendues résolvables par les tables Request-body SPEC §6) ; le
   critère de fermabilité liste le test nextest À CÔTÉ des 2 gates
   shell.**

5. **Métrique DOMAIN_*_V1 : le flottement 21/22/23 est une ambiguïté de
   PÉRIMÈTRE élucidée — le livrable est le grep committé, pas un nombre
   neuf.** Grep déterministe au tip : **25** const `DOMAIN_*_V1`
   distinctes all-crates (23 canonical.rs + `DOMAIN_KEYSTORE_V1`
   keystore.rs:102 + `DOMAIN_TRACE_EVENT_V1` trace-core/lib.rs:29,
   toutes `pub const` non-test, aucune _V2, 0 doublon cross-crate) ; 3
   familles schématisées (COMPUTE_GROUP, SHARD_PLAN [ShardPlan +
   ShardedSessionManifest], RUN_PROOF) → **22 non-schématisées**. Le
   « 22 of the 25 » est DÉJÀ en prose PATTERNS.md:3941 mais (i) SANS
   grep reproductible, (ii) avec un renvoi PÉRIMÉ « routed to the S80
   audit-plan » (:3942-3943). 23 = compte canonical.rs seul
   (sprint81_phase_b_preflight) ; 21 = jamais dérivable (WI-10 S79).
   `TASK_RESPONSE_V1` (tag de TaskResponse, schématisé) n'est PAS une
   const `DOMAIN_*_V1` — hors métrique par construction ; DOMAIN_TASK_V1
   (wire Task) reste non-schématisée. → **Exécution : committer le grep
   BusyBox-safe (le gate impose no `--include`/no `-P`, :8-9) :
   `find crates -name '*.rs' ! -path '*/llama.cpp/*' ! -path '*/target/*'
   -exec grep -hoE 'DOMAIN_[A-Z0-9_]+_V[0-9]+' {} + | sort -u` = 25,
   univers explicite, dans PATTERNS §P70 + re-dater le renvoi ; +
   tripwire de comptage dans check-frontier-contracts.sh (compte figé
   25 → toute famille NEUVE force la décision D8 consciente).**

6. **« Trancher S80-G-1 » = CONSIGNER une clôture déjà actée, résidu
   substantiel 0.** L'accept-and-close est acté verbatim au kickoff S81
   (archive v2.1, sprint81_kickoff.md:95-99 : « DOC-LINT-SEMANTIC
   (S80-G-1) → ACCEPT-AND-CLOSE acté : le doc-lint reste existence-only,
   la vérification sémantique des claims = revue LLM adversariale par
   sprint … exit condition remplie, l'item sort des carries ») et n'a
   PAS été re-porté dans sprint81_audit_plan.md (grep = 0). → **Exécution :
   formalisation au ledger (qualifiée redondante, réf S81), JAMAIS un
   4e report.** Conforme design_review:132-134.

## Faits additionnels vérifiés (à livrer dans la phase)

- **SPEC §3 stale (découverte vérif S3)** : la table « Generated JSON
  Schemas » (SHARD_PROTOCOL_SPEC.md:91-100) liste 8 rows pour 11
  snapshots au disque — il MANQUE `ShardSessionResultView` +
  `ShardSessionResultResponse` (ajoutés **S81 Phase I**, `bb6c4f9` —
  la mention « S81 K » de la première rédaction était une erreur du
  scan S3, détectée par la review dimension 7 [P2] : le struct
  shard.rs:119/:130 et git log -S le datent Phase I ; note du
  2026-07-14). Backfiller ces 2 rows EN PLUS des 2 nouvelles requests.
- **Note PATH-authoritative SPEC §6 (S3 CONFIRMED)** : l'invariant
  runtime « PATH is authoritative, disagreeing body id rejected (400) »
  (http.rs:2304-2306 doc + :2340-2350 enforcement) n'est PAS documenté
  dans SPEC §6 (:267-289) et JSON Schema ne peut pas l'exprimer — sans
  note prose, le schéma `session_id` optionnel serait mal lu. Ajouter
  la note §6.
- **PROMISE_RE clean (S3)** : aucun `///` des 3 structs ni de
  ShardWorkerSpec/ShardModelSpec ne matche PROMISE_RE (« Sprint 81
  Phase J; clamped… » = narration passée, re-testé NO MATCH). Les
  descriptions copiées dans les snapshots seront propres.
- **0 bump wire (S4 CONFIRMED)** : les 3 bodies sont Deserialize-only,
  jamais sérialisés vers un chemin gossip/blob (usages : axum Json
  http.rs:2216/:2265/:2328, config CLI cli.rs:263, tests). JsonSchema
  est additif, inerte à serde (SPEC:88-89 le documente déjà).
- **LOOPBACK doc N-A confirmé (S3)** : §3 = inventaire tiers à jour
  (6 lignes shard-session, front-matter S81-K ; group/mount T0→T1
  cible, generate T0→T0, :89-94). Aucune ligne request-body à toucher.
  NB : « périmètre représentatif verrouillé (D7, front-matter) » du
  kickoff n'est PAS encore sur disque — livrable Phase T, pas G.
- **Claims vivants hors chemin (S3)** : K-R-7 « session réelle » /
  « byte-identical » + binding loaded-stage↔manifeste (THREAT_MODEL
  §16 :1536-1571, :145-149) — Phase G ne touche pas le chemin
  drive/attestation.
- **Câblage gates (S5)** : frontier = ci-linux.yml:92 / ci.yml:136 /
  verify.sh:98-99 ; sharding = ci-linux.yml:87 / ci.yml:132 /
  verify.sh:95-96. Woodpecker = BusyBox bash:5 sans cargo → les
  modifications de gate restent BusyBox-safe.
- **Aucun commit DEVIATION/rejected/scope-cut ne contraint la zone**
  au-delà de la doctrine S77-L (S2, git log = 0 match).

## Plan d'exécution adapté (résumé opérationnel)

1. Move a2 : `ShardGroupMintRequest` + `ShardGenerateRequest` →
   `nexus-core-rs/src/schemas/shard.rs` (pub, `Debug + Deserialize +
   JsonSchema`, doc-comments préservés SANS promesse future, prose
   « Loopback-API frontier » miroir :121-126 ; PAS de
   `#[schemars(required)]` — les `Option` de request sont réellement
   optionnels, contrairement aux réponses always-serialized) ; imports
   daemon ; +2 fns `*_schema()` ; +2 entrées `schema_snapshots()` ;
   `UPDATE_SNAPSHOTS=1` → 2 snapshots committés ; +2 tests whitelist
   miroir.
2. MountSessionRequest : reste au daemon (repli b motivé — enveloppe
   signée + EndpointAddr upstream), prose frontière documentée sur la
   struct, table Request-body complète SPEC §6.
3. SPEC : §6 tables Request-body des 3 POST + note PATH-authoritative ;
   §3 backfill 2 rows S81-K + 2 rows nouvelles.
4. check-sharding-docs.sh : REQUIRED_ANCHORS +3 structs (BusyBox-safe).
5. check-frontier-contracts.sh : tripwire comptage DOMAIN_*_V1 figé à
   25 (message → décision D8 consciente pour toute famille neuve).
6. PATTERNS §P70 : grep committé + univers 25/22 explicite + renvoi
   re-daté + consignation S80-G-1 (clôture S81 formalisée) + D8 acté.
7. Rouge-avant-vert : drift-test sans snapshots → rouge ; tripwire à
   26 → rouge ; anchors absentes → rouge. Suites §7.4 complètes.

Frontière NEUVE (3 request-bodies) → indexée Phase T (invariant
kickoff #17). T1 = N-A (aucun `web/src/api` touché). T2 = N-A.
0 bump wire, 0 dep runtime, 0 changement de comportement observable
(move + derive additif).

## Détail des scans (verdicts adversariaux)

- **S1a prior-art snapshot** : 11 faits, 14/14 verdicts CONFIRMED
  (mécanisme jumeau shard/task_response, doctrine S77-L, coût par
  struct). Missed relevé : métrique 25 (reprise en S4), complétude
  3 requêtes confirmée (mount = même struct, pas de 4e frontière).
- **S1b deps/churn** : option (a) = 0 churn lockfile (schemars 1.2.1 +
  derive déjà présents, deny.toml warn-only multiple-versions,
  précédent direct-dep S20-D `c85397b`) ; 1 PARTIAL (chaîne cargo tree
  non re-exécutée, prouvée par ancres indirectes). Le critère de
  bascule « churn de dep » n'est PAS déclenché — le vrai motif de
  repli pour mount est TYPE-GRAPHE.
- **S2 décisions historiques** : 744f84a S77-L rationale ; S80-G-1
  verbatim ; flottement élucidé ; D8 kickoff:270-273 verbatim ;
  1 PARTIAL = ancre :317 corrigée en :525 (drift-test).
- **S3 threat model** : 0 claim de sécurité cassé ; JsonSchema additif
  documenté SPEC:88-89 ; enforcement PATH-authoritative présent ;
  missed → backfill SPEC §3 + note §6 (intégrés supra).
- **S4 wire + métrique** : 9/9 CONFIRMED + 5/5 plan_corrections
  CONFIRMED ; métrique 25/22 re-exécutée byte-exacte par la vérif.
- **S5 gates** : gates AVANT exit 0 réels ; boucle FRONTIER opt-in ;
  REQUIRED_ANCHORS sans les 3 structs ; missed → EndpointAddr survit
  au move (intégré au bloqueur mount) + coût tests whitelist (intégré
  au plan).
