# Review S81 Phase I — Orchestrateur de session sharding in-vivo (ex-S78)

## Verdict: PASS

**Le P1 bloquant est FERME** par la boucle de correction post-FAIL (delta verifie
+ passe adversariale CONCORDANTS) et AUCUN nouveau P0/P1 n'apparait — verdict
PASS-PENDING (Codex + verdicts finaux des blocs dual-platform consignes au
commit). Le deadline par-hop SI-9 couvre desormais TOUT le hop (`open_bi` +
`write_frame` + `read_frame` + `send.finish`) sous un unique
`tokio::time::timeout(deadline, ...)` (`shard_session.rs:729-750`), la docstring
`drive_hop` (`:721-727`) est rendue vraie sur le chemin write (rationale
D1-1/D2-1 in-code), et un test de regression REEL — pas un faux-vert —
`hop_deadline_bounds_the_write_path` (`:1664-1739`) epingle le withholding
byzantin cote WRITE : frame 64 MiB > fenetre QUIC stream contre un
`BlackholeProtocol` custom qui admet le bi-stream et ne draine JAMAIS son recv en
tenant les deux streams ouverts -> le drive echoue CLEAN sur « SI-9 »,
`worker_drop_count==1`, `result_text` None (aucun result creux). Les 3 P2
(D2-2 re-dial non-borne, D3-1 garde `Generating` sans test, D3-2 invariant
readiness-zero-frame trivialement asserte) sont EGALEMENT fermes en boucle.
Detail complet ci-dessous en « Boucle de correction (post-FAIL) », qui SUPERSEDE
le diagnostic FAIL de la Synthese executive et des dimensions conserve en
historique.

Reste de la surface : posture solide, aucun autre P0/P1/P2. Les 9 P3 restent
documentables au commit body — dont D1-2 (docstring route drop-shard) et D2-3
(doc-note champ `failure`) desormais CORRIGES doc en boucle. Whitelist SI-3/SI-4
intacte, 0 bump wire, 0 dep, heberger != publier tenu. Tout ce qui suit (Synthese
executive + Dimensions 1-5 + Conformite preflight) est le snapshot review-time du
FAIL, conserve tel quel comme historique ; l'etat autoritaire post-boucle est la
section « Boucle de correction ».

## Synthese executive

Phase I livre l'orchestrateur de session in-vivo en pure composition des
primitives S77 : module neuf `crates/nexus-shell-daemon/src/shard_session.rs`
(~1100 lignes), registre en-memoire gate `DOMAIN_SHARD_PLAN_V1` + `is_member`, 5
routes loopback (`group`/`mount`/`generate`/`result`/`drop-shard`), sous-commande
CLI operateur, projection privacy `member_count`/`rtt_frontier_ms`-only, snapshots
schema drift-gated + whitelist-testes, Zod front tolerant version-skew. Le cycle
de vie 6 etapes (placement / manifeste / READINESS / dispatch / mesure+RunProof /
teardown) est fidele au PLAN-ADAPT. Les invariants de phase sont tenus : 0 bump
wire (`sbfb/shard/1`, `SHARD_PLAN_FORMAT_VERSION=1`, `RUN_PROOF_FORMAT_VERSION=1`
inchanges), 0 dep nouvelle (Cargo.toml/package.json intacts), whitelist SI-3/SI-4
double-testee, duress-gate sur group/mount/generate AVANT tout signing/dial.

**Le seul defaut bloquant** est le demi-armement de SI-9 (write_frame hors
deadline). Il n'est pas cosmetique : (1) il rend une garantie in-code FAUSSE sur
le chemin de liveness ; (2) il laisse le livrable central du PLAN-ADAPT (« armer
SI-9 dans le composant qui le possede », preflight `:26-29`/`:103-104`)
INCOMPLET ; (3) il est exploitable par le membre byzantin exact que SI-9 cible
(withholding, Sev M, carry OUVERT). Le preflight previent explicitement (`:28-29`)
que « livrer le driver sans l'armer re-certifierait un livrable PROVISIONAL en
laissant intacte sa faille de liveness Sev-M connue, dans le composant meme qui la
possede » — c'est precisement l'etat du code.

Ponderation adversariale des 5 dimensions : sur 22 findings bruts, 1 P1
(dual-reporte, dedup), 3 P2 CONFIRMES, 9 P3 CONFIRMES, 8 INFO (dont 4 downgrades
depuis P3/P2 et 2 deviations ADAPTE-JUSTIFIE). Zero finding refute (les 2
« downgrades » de D4 tombent en P3/INFO, jamais refutes).

## Boucle de correction (post-FAIL)

Etat AUTORITAIRE apres la boucle de correction appliquee sur 3 fichiers
(`shard_session.rs`, `http.rs`, `schemas/shard.rs` + snapshot regenere). Verifie
en lecture seule (Read/Grep/git) sur l'arbre de travail vs HEAD `12e3954` ;
concorde avec la passe adversariale independante. Aucun `cargo`/`npm` relance ici
(lock contention) — les verdicts de suites ci-dessous sont ceux consignes par le
main-thread.

**Fix P1 (D1-1/D2-1 — write-stall byzantin) — FERME.** `drive_hop`
(`shard_session.rs:728-751`) enveloppe le hop COMPLET dans un unique
`tokio::time::timeout(deadline, async { open_bi + write_frame + read_frame +
send.finish })` ; le map_err de sortie porte le litteral « SI-9 withholding guard
— covers open/write/read ». Les DEUX sites d'appel heritent du deadline :
primaire `:895` et fallback `:939`, tous deux avec `hop_deadline`. La docstring
`:721-727` est reecrite pour dire « The WHOLE hop … runs under ONE deadline
(SI-9) » avec le rationale D1-1/D2-1 (le write est le point ou le workload Phase-J
bloque reellement). Faux-vert ecarte par re-verification des deps epinglees :
noq-proto-1.0.1 `stream_receive_window`=1.25 MB, iroh-1.0.1
`QuicTransportConfigBuilder::new` NE l'override PAS, nexus-core-rs n'ajoute aucun
transport config custom -> la frame 64 MiB (`MAX_SHARD_FRAME_BYTES`=256 MiB donc
non rejetee pre-write) backpressure vraiment `write_all` contre un
`BlackholeProtocol` non-lecteur, et le deadline hop est le SEUL garant de
liveness. Pre-fix : le test HANGUE ; post-fix : SI-9 tire au deadline 1 s.
`drive_hop` est le SEUL chemin d'ecriture shard-plane du daemon et il est
integralement sous le timeout.

**Fix P2 D2-2 (re-dial primaire non-borne) — FERME.** Le re-dial d'etage (bras
`None`, conns vide apres `std::mem::take`) est desormais enveloppe dans
`tokio::time::timeout(readiness_deadline, open_shard_connection(...))`
(`shard_session.rs:884-890`), coherent avec la barriere de readiness qui borne le
meme dial avec son propre deadline. Le happy first-drive reutilise la connexion
de la barriere et n'est pas affecte.

**3 tests neufs (ferment D3-1, D3-2 + le write-withholding) :**
- `hop_deadline_bounds_the_write_path` (`:1664-1739`) — `BlackholeProtocol`
  custom (admet le bi-stream, ne lit jamais, tient les 2 streams ouverts across
  `conn.closed()` — pas de STOP_SENDING) + frame 64 MiB ; asserte `err` contient
  « SI-9 », `worker_drop_count==1`, `result_text` None. Regression reelle du
  chemin WRITE (le T1 `StallingForwarder` d'origine ne couvrait que le read).
- `generate_rejects_concurrent_drive` (`:1742`) — status force `Generating` puis
  `generate_session` -> `Err("already generating")` renvoye AVANT tout reseau
  (l'adresse dummy n'est jamais dialee ; check-and-set atomique sous le Mutex du
  registre). Ferme D3-1.
- `successful_mount_emits_zero_frames_before_generate` (`:1785`) — 2
  `CountingForwarder`, somme==0 apres un mount REUSSI (readiness = handshake +
  echantillon RTT, `accept_bi` du worker bloque donc `forward()` ne tourne
  jamais), puis exactement 1+1 apres `generate`. Preuve happy-path qui remplace
  l'assertion trivialement-vraie sur un mount qui ECHOUAIT. Ferme D3-2.

**2 docstrings P3 — CORRIGES doc :**
- **D1-2** — docstring de la route `drop-shard` (`http.rs:2424-2429`) rendue
  honnete : elle dit desormais que post-drive « only the counter moves — the next
  drive re-dials regardless » et qu'un drop mid-drive est gere par le fallback
  SI-9, en phase exacte avec `drop_tail_shard`.
- **D2-3** — doc-note sur le champ `failure` (`schemas/shard.rs:149-155`) :
  divulgue honnetement que le texte peut porter des prefixes de cle worker
  TRONQUES a 8 octets (convention de log du repo, non-inversible vers la cle 256
  bits ; la whitelist de proprietes porte sur les identites completes, absentes).
  Snapshot `shard_session_result_view.schema.json` regenere en consequence
  (`failure.description` == la doc-note verbatim, `failure` dans `required`).

**Suites depuis la boucle.** nextest cible `shard_session or schemas::shard` =
24/24 PASS (dont les 3 tests neufs). Blocs complets Rust Win nextest workspace +
Docker `sbfb-ci` : RE-LANCES cote main-thread (verdicts finaux consignes au
commit). Bloc web : ACQUIS — la boucle n'a touche AUCUN fichier `web/` (verifie :
seuls `shard_session.rs`, `http.rs`, `schemas/shard.rs` + snapshot dans le delta).

**Delta tests FIGE (recompte git diff + fichiers).** 17 Rust net-new vs HEAD
`12e3954` = 15 module `shard_session` (fichier neuf : 12 pre-boucle + 3 boucle) +
1 `http.rs` (`shard_session_routes_noop_in_duress`) + 1 `schemas/shard.rs`
(`shard_session_result_view_schema_is_whitelisted`) ; 0 suppression. Les tests
`shard_session_response_pins_empty_envelope`, `shard_session_projection_hides_member_identities`
(http.rs) et `shard_session_view_schema_is_whitelisted` (schemas/shard.rs)
PRE-EXISTENT a HEAD -> hors compte net-new. Vitest : 1 nouveau `it()` + 1 modifie
(inchange par la boucle). Le commit body FIGE 17 Rust net-new / 1 nouveau Vitest.

**Residus INFO non-bloquants (ne rouvrent RIEN) :** (a) deux commentaires
encore cadres « bounds read only » — le module doc (`shard_session.rs:40-43`,
step Dispatch) et la constante `HOP_DEADLINE_DEFAULT_MS` (`:122-128`) — non
alignes avec l'elargissement du deadline (la docstring AUTORITAIRE `drive_hop`
`:721-727` EST corrigee et le test phare epingle le write path : doc-honnetete
seule, a folder dans la passe docs-contract Phase K) ; (b) sur le chemin fallback
d'echec, `fb_conn` est droppe implicitement (`:939-943`) plutot que ferme
explicitement comme le primaire (`:907`) — pre-existant, benin (quinn/iroh ferme
au drop, noeuds de test arretes de toute facon), NON introduit par la boucle,
note pour symetrie. Aucun de ces residus n'est un P0/P1/P2.

## Dimension 1 — Correctness (diff ligne par ligne)

- **D1-1 (P1 — CONFIRMED, = D2-1)** — `drive_hop` n'arme AUCUN deadline sur
  `write_frame`. Re-verifie moi-meme : `drive_hop` (`shard_session.rs:722-737`)
  enveloppe `open_bi` (`:723`) et `read_frame` (`:730`) dans
  `tokio::time::timeout(deadline, ...)` mais `write_frame` (`:727`) est un `await`
  NU. `write_frame` -> `send.write_all(payload)` (`nexus-core-rs/src/shard.rs:139`)
  applique le backpressure de flow-control QUIC stream-level ; `MAX_SHARD_FRAME_BYTES`
  = 256 MiB (`shard.rs:85`) >> fenetre QUIC. Un membre admis byzantin (SI-9
  withholding) qui accepte le bi-stream mais ne draine jamais son recv sature la
  fenetre et BLOQUE `write_all(payload)` sur une grande frame, AVANT que le
  deadline `read_frame` (`:730`) n'entre en jeu (il ne s'arme qu'apres retour de
  `write_frame`). La docstring `:721` « every await bounded by `deadline` (SI-9) »
  est donc FAUSSE sur le chemin write. Aucun timeout externe (drive_pipeline await
  nu, spawn fire-and-forget cote HTTP). Le T1 `StallingForwarder` ne couvre PAS ce
  cas : il LIT la frame puis dort dans `forward()`, exercant seulement le timeout
  read. **Fix trivial 0-wire dialer-side** : envelopper le hop complet (open_bi +
  write_frame + read_frame) dans un unique `tokio::time::timeout(deadline, ...)`.
  Evidence : `shard_session.rs:727` (write_frame hors timeout) + `shard.rs:139`
  (write_all backpressure).

- **D1-2 (P3 — CONFIRMED)** — `drop_tail_shard` : « cut » fantome apres un drive.
  `generate_session` fait `std::mem::take(&mut record.conns)` (`:774`) puis
  teardown ferme tout (`:806-808`), donc post-drive `record.conns` est VIDE.
  `drop_tail_shard` : `record.conns.remove(&tail)` (`:449`) renvoie None post-drive
  (ne ferme AUCUNE connexion) mais `worker_drop_count += 1` inconditionnel (`:452`).
  A decharge : la docstring lib (`:435-438`) hedge honnetement « if still held » ;
  l'over-affirmation est cote docstring de la ROUTE operateur (« close the tail
  shard's connection and count the drop », qualifie de « post-run explicit cut »).
  Aucun dommage fonctionnel (le harness b3 ne lit que l'increment du compteur).
  Doc-honnetete a acter au body. Evidence : `shard_session.rs:449`/`:452`.

- **D1-3 (INFO — DOWNGRADED de P3)** — mount d'un `session_id` duplique : sonde
  readiness complete gaspillee avant le rejet `insert_gated` « already mounted ».
  Aucune fuite (record droppe -> quinn ferme les Connection), chemin d'erreur
  operateur, ZERO impact correctness/securite — pur gaspillage de dials + churn
  worker. Un pre-check `status_data(session_id).is_some()` l'eviterait. Micro-optim,
  pas un finding a documenter. Evidence : `shard_session.rs:674-713`.

- **D1-4 (INFO — CONFIRMED)** — `generate` renvoie 202 `accepted:true` meme quand
  la tache spawnee fera `Err("already generating")` (avalee par `let _ =`). La
  concurrence est CORRECTEMENT gatee sous lock (`:767-769`), jamais de double-drive ;
  reponse optimiste acceptable pour un outil operateur. Fire-and-forget : un drive
  interrompu par shutdown laisse status=Generating mais le registre est in-memory
  (efface au restart) -> pas d'etat bloque persistant. Evidence : `http.rs`
  (shard_session_generate) + `shard_session.rs:767-769`.

POINTS D1 VERIFIES CORRECTS (adversarial, non-findings) : alignement des cles
head=pow_keypair=initiator (workers admettent le dial de la tete) ; ordre des args
`RunProof::new` (pas de swap des deux `[u8;32]`) ; caps participants jamais
depassees (post-success sign infaillible) ; aucun `std::sync::Mutex` tenu
across-await ; `worker_drop_count` pas double-compte (2 vues du meme evenement) ;
replay-cache `insert` avant `get` dans la meme iteration (expect infaillible) ;
reuse de connexion correcte (used vs conns, re-dial sur re-drive, teardown sans
fuite ni double-close). LIMITE DE DESIGN documentee : la readiness barrier prouve
la readiness TRANSPORT, pas BACKEND — ce qui RENFORCE la P1, car le seul backstop
contre un backend lent (le deadline hop) ne couvre pas write_frame.

## Dimension 2 — Securite deep (THREAT_MODEL §16 SI-1..SI-11, §5.9)

- **D2-1 (P1 — CONFIRMED, = D1-1)** — meme defaut, vu cote securite : le garde
  SI-9 est a moitie arme (write path non-borne), laissant OUVERT le livrable
  SI-9 1(d) sur le workload Phase-J (grosses frames). Re-verifie identiquement.
  P1 tenu. Evidence : `shard_session.rs:727` + `shard.rs:139`.

- **D2-2 (P2 — CONFIRMED)** — re-dial primaire du drive non borne par le deadline.
  Sur une 2e generation (conns vide par `std::mem::take` `:774`), le re-dial de
  l'etage tombe dans le bras `None` et appelle
  `open_shard_connection(endpoint, lookup, addr).await` (`:866`) SANS timeout,
  incoherent avec la barriere de readiness qui, elle, enveloppe le meme dial dans
  `tokio::time::timeout(deadline, ...)` (re-verifie `:591`). Consequence : sur une
  2e generation vers un etage injoignable, la liveness ne depend plus du
  `readiness_deadline` de la session mais du timeout connect interne d'iroh. P2
  (pas P1) car `endpoint.connect` s'auto-termine sur un pair vraiment mort. Fix :
  envelopper `:866` dans `timeout(readiness_deadline, ...)`. Evidence :
  `shard_session.rs:866` vs `:591`.

- **D2-3 (P3 — CONFIRMED)** — le champ `failure` de la projection result peut
  embarquer un prefixe pubkey worker tronque a 8 octets. `mark_failed` stocke des
  diagnostics contenant `hex::encode(&worker_pubkey[..8])` / `&fallback[..8]`
  (`:888`, `:895`, `:911`, `:920`) et `result_data` expose `r.failure.clone()`
  (`:430`). Le test whitelist `shard_session_result_view_schema_is_whitelisted`
  (`schemas/shard.rs`) n'asserte que l'ensemble des NOMS de proprietes, jamais les
  VALEURS -> un prefixe 8 octets DANS la valeur passe. Argumentaire non-bloquant
  CONFIRME : route en `authed_routes` (loopback bearer+Host+Origin) ; l'appelant EST
  l'initiateur du groupe, deja detenteur de TOUTES les pubkeys completes ;
  troncature 64 bits non-inversible vers la cle 256 bits ; coherent avec les 409
  mount tronques et la convention repo (`hex::encode(&pk[..8])` en logs).
  Recommandation : doc-note sur le champ OU redaction. Evidence :
  `shard_session.rs:430`.

- **D2-4 (INFO — CONFIRMED)** — `gate_session` accepte un plan a 0 assignation
  (`is_pipeline_contiguous` renvoie true pour un vec vide, `shard_plan.rs:215`).
  NON atteignable via HTTP : le manifeste est batit server-side par `place_and_sign`
  -> `plan_placement` (2+ etages contigus, sinon `EndpointFederation` rejete) ;
  l'operateur ne fournit jamais le manifeste. Atteignable seulement par
  `ShardSessionRecord` hand-built (tests). Defense-in-depth possible :
  `gate_session` pourrait rejeter `assignments.is_empty()`. Evidence :
  `shard_plan.rs:215`.

VERIFICATIONS SECURITE PASSANTES (a consigner) : (a) les 5 routes sont dans
`authed_routes` (bearer+Host+Origin), pas dans public/token ; (b) gate complet
AVANT insert ET AVANT tout dial (verify_signature manifeste + groupe + binding
group_id/initiator + contiguite + is_member par worker/fallback), re-run avant
probe reseau ; (c) SSRF borne par le handshake iroh cryptographique (endpoint-id
== addr.id) + loopback durci ; (d) 0 bump wire (les 2 `.schema.json` neufs =
snapshots de projection HTTP, pas des types wire signes) ; (e) registre Debug
manuel = count-only (pas de dump d'identites en logs) ; toute exposition
d'identite uniformement tronquee a 8 octets ; (f) run_proof expose = hex de la
signature 64 octets seule, participants/pubkeys jamais serialises. CONTEXTE
process (non-finding) : la MAJ THREAT_MODEL §16/§5.9 est planifiee Phase K
(docs-contract) ; l'absence de MAJ en I est intentionnelle — MAIS le P1
write-stall doit etre corrige AVANT que K ne certifie SI-9 comme « arme ».

## Dimension 3 — Branch coverage semantique + delta tests

- **D3-1 (P2 — CONFIRMED)** — garde de generation concurrente
  (`status==Generating`) load-bearing mais ZERO test. Re-verifie : garde prise SOUS
  `registry.lock()` (`:763`) avec set Generating (`:770`) = check-and-set atomique,
  SEUL rempart contre deux drives concurrents (le handler HTTP ne teste que 404, pas
  le statut). Aucun des tests ne met une session en Generating puis rappelle
  generate. Scenario regression (garde deplacee/supprimee -> TOCTOU double-drive, 2
  RunProof signes, outcome ecrase) non rattrape. Test trivial manquant
  (hand_built, status=Generating, insert, generate -> attendre « already
  generating »). Evidence : `shard_session.rs:767-769`.

- **D3-2 (P2 — CONFIRMED)** — l'invariant phare « readiness barrier n'emet AUCUNE
  frame de dispatch » (cause racine RIG-ABSENT S77) n'est que trivialement asserte.
  Le SEUL test qui utilise `CountingForwarder` asserte compteur==0 sur un mount qui
  ECHOUE (la sonde readiness n'emet jamais de frame ET generate n'est jamais
  appelee), donc le compteur vaut 0 quelle que soit la position du ghost. Le
  happy-path utilise `EchoForwarder` (pas de compteur). Aucun test n'isole « un
  mount REUSSI seul emet 0 frame » (CountingForwarder==0 apres mount, >0 apres
  generate). Si mount regressait pour emettre une probe-frame ou fuir une frame de
  dispatch avant l'ACK complet, aucun test ne le detecterait. Evidence :
  assertion CountingForwarder==0 dans `mount_readiness_blocks_on_unreachable_shard`
  vs intention nommee du CountingForwarder.

- **D3-3 (P3 — CONFIRMED)** — delta Vitest reel = 1 nouveau `it()` (« tolerates an
  OLDER daemon that predates rtt_frontier_ms ») + 1 MODIFIE (« parses a found
  session with its aggregate member_count », present a HEAD). Le brief disait « 2
  nouveaux » -> sur-compte de 1. Cote Rust : 14 nouveaux (12 `shard_session.rs` + 1
  `http.rs` `shard_session_routes_noop_in_duress` + 1 `schemas/shard.rs`
  `shard_session_result_view_schema_is_whitelisted`), 0 suppression = PLAFOND exact
  de la fourchette +8..14. Le commit body doit FIGER 14 Rust / 1 nouveau Vitest
  (+1 renforce), pas la fourchette. Evidence : `daemon.test.ts` (1 seul +it()).

- **D3-4 (P3 — CONFIRMED)** — sequence « generate deux fois » (contrat re-dial
  documente `:254-256`) non testee de bout en bout. La BRANCHE re-dial
  (`conns.remove()==None` -> open_shard_connection frais) EST couverte par les
  hand-built ; mais la SEQUENCE mount->generate->generate sur une meme session
  montee (conns peuple par readiness puis vide par le 1er generate) n'est jamais
  jouee. Non-bloquant (run LIVE = Phase J). Evidence : `shard_session.rs:254-256`.

- **D3-5 (P3 — CONFIRMED)** — resume-from-cache mono-etage seulement.
  `hop_deadline_reroutes_to_fallback_and_resumes` utilise un plan a UNE assignation,
  donc le replay cache ne re-joue que le prompt. Le cas load-bearing d'un etage
  mid-pipeline (activation cachee = SORTIE de l'etage N-1) n'est pas couvert.
  L'assertion `result_text=="resume-me"` ne distingue pas « replay du bon frame de
  frontiere » d'un simple echo du prompt. Evidence : plan mono-etage + code replay
  `:855`/`:912`.

- **D3-6 (P3 — CONFIRMED)** — mappings de statut HTTP des 5 routes non testes
  (logique module couverte, wrappers minces non). Non testes : mount 409 CONFLICT,
  generate 400 body!=path, generate 404 non-monte, drop-shard 200 {found,dropped},
  group 400 hex invalide. Risque faible (logique sous-jacente couverte au module) ;
  a tracer comme frontieres loopback Phase K. Evidence : `http.rs` (routes
  non testees hors duress-200).

- **D3-7 (INFO — CONFIRMED)** — R-I-5 (famille iroh-networked) documentee (le
  module acte l'exemption d'env Docker-on-Windows) + branches quasi-inatteignables
  acceptables (probe « handshake OK mais aucun RTT sample », parse_pubkey_hex « 32
  octets valides mais pas un point de courbe »). Couverture semantique
  gate/erreur/happy solide par ailleurs. Evidence : commentaire env dans
  `shard_session.rs`.

Note d'evidence D3 : les numeros de ligne du reviewer sont ~7 lignes trop hauts
vs l'arbre de travail (version pre-edition-finale) mais le CONTENU decrit est
exact a chaque emplacement (ex : garde re-verifiee `:767-769`, dial readiness
borne `:591`) — aucun finding refute par cette imprecision. Aucun gap de
couverture P0/P1 : les chemins load-bearing SONT couverts (rejets
tampered/non-member/binding/duplicate, fail-clean, fallback-resume, whitelist
privacy SI-3/SI-4).

## Dimension 4 — Conformite au preflight PLAN-ADAPT + scope cuts + research grounding

- **D4-1 (P3 — DOWNGRADED de P2)** — `run_proof` expose par `/result` =
  `hex(o.run_proof.signature)` seul (`:427`) -> non verifiable par un tiers (il
  manque le payload canonique ET la pubkey signataire, cette derniere interdite par
  SI-3/SI-4). MAIS : comportement intentionnel + documente (schema
  `result_view.schema.json:3` « the full signed entry stays in the node-local
  registry » ; le harness b3 ne gate que sur non-vide, anti-faux-vert OK), et le
  livrable (f) « emission RunProof » EST livre au niveau registre. Aucune
  defaillance -> P3 (surcotee P2), a acter au body : la projection HTTP seule n'est
  pas un ancrage de verification externe, le verificateur Phase J depend encore du
  canal control-plane differe. Evidence : `shard_session.rs:427` +
  `result_view.schema.json:3`.

- **D4-2 (INFO — DOWNGRADED de P3)** — R-I-1 readiness : la propriete « aucun frame
  de dispatch avant ACK » est asseuree par construction (`probe_shard_readiness`
  `:584-610` ne contient QUE open_shard_connection + boucle conn_rtt, aucun
  write_frame) mais pas forcee sur un shard SAIN par un test. Le finding admet
  lui-meme « scenario de defaillance : aucun ». Pure note de robustesse de test
  sans defaillance reproductible -> INFO (recoupe D3-2 cote couverture).

- **D4-3 (INFO — CONFIRMED, ADAPTE-JUSTIFIE)** — consequence preflight #2 « monter
  SHARD_ALPN dans le noeud daemon » : le code monte UNIQUEMENT SEED_ALPN au boot ;
  la tete est un pur DIALER, SHARD_ALPN est monte worker-side via la sous-commande
  CLI `serve`. Deviation JUSTIFIEE : `shard_protocol_factory` EXIGE un
  ComputeGroupEntry signe per-session + forwarder, inexistants au boot -> monter le
  handler au boot est INFAISABLE. Correlat verifie : node identity == pow_keypair
  (meme secret_bytes) -> le groupe mint admet le dial de la tete. La formulation
  litterale du preflight etait infaisable ; le code la corrige correctement, 0
  impact Day-0. Evidence : `runtime.rs` (SEED_ALPN seul) + `shard.rs:344-348`.

- **D4-4 (INFO — CONFIRMED, ADAPTE-JUSTIFIE)** — routes `/group` + `/mount` hors
  liste preflight #3 (qui ne listait que generate/result/drop-shard). Necessite
  prouvee : le registre n'est peuple QUE par `mount_session` (qui exige un
  ComputeGroupEntry signe, mint par `/group`) ; sans `/mount` aucun chemin ne leve
  le stub `live_shard_session` ni ne donne au harness une session LIVE a driver ; la
  CLI operateur route group/mount sur ces memes routes. Completion naturelle imposee
  par les livrables #1/#2/#4, pas du scope creep.

BILAN 7 LIVRABLES (voir §Conformite preflight ci-dessous) : #1-#7 LIVRES, 0
MANQUANT, 2 deviations toutes ADAPTE-JUSTIFIE. La seule reserve de conformance qui
touche un livrable EXIGE est la P1 (livrable 1(d) « deadline par-hop SI-9 »
INCOMPLET sur write_frame) — c'est le motif du FAIL.

## Dimension 5 — Livrables + Patterns + Docs-contract + Commit-readiness

- **D5-1 (P3 — CONFIRMED)** — l'etiquette result-view n'encode pas la nullabilite
  wire : les champs `Option<T>` sont `#[schemars(required)]` -> le schema les met
  dans `required` mais emet `type:"integer"/"string"` sans union `null`, alors que
  le wire serialise `null` avant tout drive. Un consommateur machine qui VALIDE
  contre le snapshot rejetterait `{"ttft_s": null}`. Pattern EXACT deja ratifie
  S77 Phase L (Codex-revu) -> pas une regression ; les vrais consommateurs (b3 =
  scrape bash, front = Zod nullable independant) ne cassent pas. A expliciter dans
  l'index docs-contract Phase K. Evidence : `schemas/shard.rs:131-152` +
  `shard_session_result_view.schema.json:9-38`.

- **D5-2 (P3 — CONFIRMED)** — routes `generate` + `drop-shard` : aucune etiquette
  generee ni commentaire-frontiere par-route (seule `result` en a). `generate`
  renvoie `{accepted}` et `drop-shard` `{found,dropped}` en `serde_json::json!`
  inline non type. Trace via le bloc d'enregistrement de route (« consumes
  generate/result/drop-shard verbatim »). Le DoD (d) de la cloture docs-contract
  Phase K doit INDEXER les 3 frontieres loopback (pas seulement result) ; le commit
  body doit NOMMER les 3, sinon K herite une trace partielle. Evidence :
  `http.rs` (generate/drop-shard inline) vs `schemas/shard.rs:120-125`.

- **D5-3 (INFO — CONFIRMED)** — delta tests : cible module 15/15 = 12
  (`shard_session::tests`) + 3 (`http::tests::shard_session_*`) ; net NEW = 14 =
  plafond de la fourchette +8..14. Garde anti-faux-vert : annonce == git-count au
  commit body.

VERIFICATIONS D5 POSITIVES : FRONTIER gate SAFE (le commentaire-frontiere en
backtick ne matche pas le grep du gate ; PROMISE_RE anti STALE-PHASE-K clean) ;
error style `Result<_,String>` coherent avec le reste du daemon ; 3 named consts de
deadline (0 magic number) ; 2 snapshots neufs + 2 regeneres, tous drift-gated +
whitelist-testes ; Zod front `.nullable().optional()` avec justification
version-skew ecrite (2 formes Vitest couvertes) ; §J du plan amende fidele au code
(HUB dialer, resume-from-cache vs coupe explicite comptee, RunProof du DRIVER) ;
duress-gate sur group/mount/generate mirror `seed_request_peer` ; les 16 fichiers
appartiennent tous a la phase, le preflight untracked = artefact Cas B attendu.

## Suites §7.4

Etat honnete au moment de la review (verdicts finaux consignes au commit) :

- **nextest cible module `shard_session`** : 15/15 PASS (12 `shard_session::tests`
  + 3 `http::tests::shard_session_*`), apres 2 fixes (impl Debug manuelle du
  registre + fix root-cause connexions consommees jamais reutilisees).
- **Vitest `daemon.test.ts`** : 51/51 PASS.
- **`check-frontier-contracts.sh`** : clean.
- **`cargo check` x2** : verts.
- **3 blocs complets** (Rust Win nextest workspace, web lint/tsc/vitest/build/size,
  Docker `sbfb-ci`) : EN COURS cote main-thread au moment de la review.

RESERVE FAIL : la correction de la P1 (write_frame sous deadline) DOIT etre
accompagnee d'un test couvrant le withholding cote WRITE (variante
`StallingForwarder` qui accepte le bi-stream mais ne draine jamais son recv,
ABSENTE aujourd'hui), puis les 3 blocs complets re-executes AVANT re-review. Le run
workspace anterieur (905/906, seul fail = churn AVANT fix) n'est plus
representatif une fois le fix P1 + son test ajoutes.

## Conformite preflight PLAN-ADAPT (7 livrables + deviations)

- **#1 module orchestrateur 6 etapes** — LIVRE : (a) placement `place_and_sign` /
  (b) manifeste `sign` / (c) READINESS `probe_shard_readiness` 0-wire transport /
  (d) dispatch `drive_hop` **avec deadline par-hop — INCOMPLET sur write_frame
  (P1)** + fallback resume-from-cache / (e) `RunMetrics` via `Instant` / (f)
  `RunProofEntry::sign` 1re emission PROD / (g) teardown `conn.close`. **Le
  livrable (d) « deadline par-hop SI-9 » est le point de FAIL.**
- **#2 registre gate** — LIVRE : insert gate `DOMAIN_SHARD_PLAN_V1` + `is_member`
  AVANT insert, `live_shard_session` lit le registre, projection
  `member_count`/`rtt_frontier_ms`-only (SI-3/SI-4 double-testee).
- **#3 surface HTTP** — LIVRE : generate/result/drop-shard (+ group/mount
  ADAPTE-JUSTIFIE) dans `authed_routes`.
- **#4 CLI operateur** — LIVRE : sous-commande identity/serve/group/mount/status/
  generate/result/drop-shard.
- **#5 T1 in-process** — LIVRE : EchoForwarder + cas d'erreur (readiness-block,
  hop-timeout-no-fallback fail-clean, reroute-resume, drop-shard). **Gap : le
  withholding cote WRITE n'est pas exerce (a ajouter avec le fix P1) ; l'invariant
  readiness-zero-frame n'est que trivialement asserte (D3-2 P2).**
- **#6 delta tests** — LIVRE : 14 Rust neufs (plafond +8..14) + 1 Vitest neuf.
  Garde annonce == git-count a figer au commit (D3-3/D5-3).
- **#7 enrichissements doc** — LIVRE : topologie HUB statee, cycle 6 etapes nomme,
  churn resume-vs-coupe documente, 3 frontieres loopback tracees Phase K.

Deviations : #2 « monter SHARD_ALPN au boot » = ADAPTE-JUSTIFIE (tete = dialer,
ALPN worker-side via CLI serve, infaisable au boot) ; routes /group + /mount hors
liste #3 = ADAPTE-JUSTIFIE (necessaires pour peupler le registre). Scope
cuts/Day-0 intacts : 0 dep, 0 bump wire, pipeline-parallel exclusif, Parallax,
ALPN unique, groupe prive Ed25519, heberger != publier.

## P2/P3 a documenter au commit body (P1 + 3 P2 fermes en boucle)

P2 : AUCUN ouvert. **D2-2, D3-1, D3-2 sont FERMES en boucle post-FAIL** (re-dial
borne `:884-890` ; tests `generate_rejects_concurrent_drive` et
`successful_mount_emits_zero_frames_before_generate`). Cf. section « Boucle de
correction (post-FAIL) ».

P3 :
- **D1-2** — CORRIGE doc : docstring de la route `drop-shard` (`http.rs:2424-2429`)
  rendue honnete (post-run seul le compteur bouge, le prochain drive re-dial,
  mid-drive gere par le fallback SI-9). A acter au body comme doc-honnetete.
- **D2-3** — CORRIGE doc : doc-note sur le champ `failure` (`schemas/shard.rs:149-155`)
  divulguant les prefixes worker 8 octets tronques + snapshot result-view
  regenere. A acter au body comme doc-honnetete.
- **D3-3 / D5-3** — chiffres FIGES : **17 Rust net-new** (15 module `shard_session`
  = 12+3 boucle + 1 `http.rs` + 1 `schemas/shard.rs`) / 1 nouveau Vitest (+1
  renforce), pas la fourchette.
- **D3-4** — sequence double-generate non testee E2E.
- **D3-5** — resume-from-cache mono-etage seulement (mid-pipeline non exerce).
- **D3-6** — 5 mappings de statut HTTP non testes -> frontieres loopback Phase K.
- **D4-1** — `run_proof` = hex(signature) seul, non verifiable par un tiers (le
  registre garde l'entry complete ; ancrage externe = canal control-plane differe
  Phase J).
- **D5-1** — result-view : `Option<T>` + `#[schemars(required)]` sans union `null`
  (null-avant-drive) ; tradeoff ratifie S77 Phase L, a expliciter dans
  llms.txt/REFERENCE Phase K.
- **D5-2** — nommer les 3 frontieres loopback (generate/result/drop-shard) dans le
  body + la cloture docs-contract Phase K, pas seulement result.

INFO (contexte) : D1-3 (mount duplique gaspille des sondes), D1-4 (generate 202
optimiste), D2-4 (gate_session accepte 0 assignation, inatteignable via HTTP),
D3-7 (R-I-5 exemption env documentee), D4-2 (R-I-1 sans defaillance), D4-3/D4-4
(deviations ADAPTE-JUSTIFIE).

## Codex reconciliation

Codex GPT 5.5 (codex exec, artefact brut sprint81_phase_i_codex_review.md,
output -o non reecrit) round 1 : **PARTIEL, 0 P0/P1**. Le P1 interne SI-9 est
confirme corrige par Codex (« le timeout couvre bien open_bi + write_frame +
read_frame »). Codex a re-execute les suites en verification independante
(15 shard_session + 3 http + 6 schemas + 51 Vitest + frontier gate clean) et
valide les invariants (0 bump wire, 0 dep, privacy, duress, delta annonce).
Dispositions sur ses findings :
- **P2 (readiness deadline non holistique)** — FIXE : `probe_shard_readiness`
  place desormais handshake + boucle RTT sous UN timeout partage (miroir du fix
  drive_hop), le couple ne peut plus durer ~2x deadline. Message d'erreur
  unifie « did not answer (handshake + RTT) within ».
- **P3 (teardown fallback implicite)** — FIXE : close explicite
  `fallback-hop-failed` via inspect_err sur l'echec du hop fallback (le drop
  fermait deja, le code d'application est plus diagnostique).
- **P3 (prose stub perimee)** — FIXE : le commentaire Zod front (« Phase J has
  no live session store ») et le message session-mount du harness b3_shard
  (« live_shard_session is a Phase J STUB ») decrivent desormais le registre
  live S81 + le flux operateur de montage.
- **P3 (untracked hors scope)** — ACTE : `.planning/research/
  psyche_nous_analysis_2026-07.md` est un artefact de recherche PO demande
  en parallele de la phase ; il n'est PAS stage dans le commit de phase
  (commit chore(research) separe).
- **PARTIEL livrable 6 (« session shard 2-machines reelle » ambigu au plan
  §Phase I)** — ACTE sans edit : le plan est un artefact fige du kickoff ; le
  cadrage « Phase I = OUTIL, run LIVE = Phase J » est explicite dans le
  preflight (livrable 4), l'amendement §Phase J et le commit body.
Suites re-jouees post-dispositions : fmt/clippy clean, nextest cible
shard_session + schemas verts, bash -n harness OK. Aucun P0/P1 — critere
d'arret de boucle atteint (CLEAN ou P2/P3 documentes), review promue PASS.
