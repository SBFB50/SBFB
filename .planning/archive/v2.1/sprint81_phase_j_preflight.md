# Preflight S81 Phase J — Benchmark live sharding 2-machines + T2 axe shard

## Verdict: DESIGN-CONFLICT

- **Phase** : J (Sprint 81, Cas B) — jouer `b3_shard` LIVE (RTX 5080 dev Win + Mac M2) via
  l'orchestrateur Phase I sur la stack POST-migration iroh 1.0.1 (FLIP H d'abord, même session),
  et solder le carry P1 sharding S77.
- **Date** : 2026-07-10.
- **Méthode** : Workflow fan-out 7 scans (S1a prior-art benchmarking WAN, S1b deps/advisories,
  S2 décisions historiques, S3 threat model, S4 wire/contrats, S5 inventaire harness/CLI,
  S6 rig/convergence) + vérification adversariale par-scan (refuted_ids écartés) + spot-checks
  indépendants du synthétiseur au HEAD `58cef6d`.

Le **but** de Phase J ne touche AUCUNE Day-0 (pipeline-parallel exclusif, ALPN `sbfb/shard/1`
inchangé, topologie STAR/HUB figée, 0 bump wire, groupe privé Ed25519, incentive non-monétaire).
Les axes deps (**S1b EXECUTE**, 0 dep, `cargo deny` vert, lock uniformément 1.0.1) et threat
(**S3 EXECUTE** au niveau surface : aucune nouvelle classe de menace, tête dial-only) sont propres.

MAIS un **conflit de conception load-bearing** empêche d'EXÉCUTER Phase J telle qu'écrite. Le plan §J
promet « run `b3_shard` LIVE → **si PASS → carry P1 sharding S77 CLOSED** » avec « **Delta tests 0 Rust**
(acceptance live) ». L'inventaire code (S5-G1, S6-1, S6-7, confirmés, 0 réfuté ; re-vérifiés main.rs) prouve
que **le SEUL chemin `serve` livrable câble `EchoForwarder` (transport-only)** : le vrai backend
d'inférence `ShardBackendForwarder` (fork llama.cpp, layer-block) est défini/exporté dans
`nexus-worker-core` mais possède **ZÉRO call-site de montage** sur `sbfb/shard/1` (toutes les autres
occurrences sont `#[cfg(test)]`). Un run J via l'orchestrateur émettrait donc un **PASS CREUX** (echo du
prompt : `result_text == prompt`, `tokens == 1`, `run_proof` = preuve signée du DRIVER, `toks_per_s == 1`
par plancher `.max(1)`) que le harness étiquette pourtant « **A ~20 GB model was sharded across the 5080 +
Mac M2 and generated a verified response** / per-shard signed, N0-N3 bound » (`b3_shard_pipeline.sh:382-385`,
re-vérifié). Ce PASS ne prouve QUE le transport QUIC cross-machine + le cycle de vie — **pas l'inférence
shardée**, dont le RIG-ABSENT S77 portait précisément sur l'inférence réelle. **Il ne clôt donc PAS
légitimement le carry P1 sharding S77.**

La résolution est un **arbitrage de périmètre mutuellement exclusif** que seul le PO/kickoff peut trancher :

- **Option A — J prouve le TRANSPORT cross-machine (echo) honnêtement scopé.** J documente que le run
  atteste transport + lifecycle sur la stack 1.0.1, PAS l'inférence ; le carry P1 sharding **reste OUVERT**
  (verdict fermé au vocabulaire scopé, pas un `PASS` global). Reste 0 Rust côté data-plane, mais **exige
  la correction du harness** (tell anti-faux-vert + libellés mensongers, S4-2/G5). → contredit la ligne
  livrable « si PASS → carry P1 CLOSED ».
- **Option B — J câble d'abord l'inférence réelle sur `sbfb/shard/1` puis run.** Monter
  `ShardBackendForwarder` dans un `serve` gaté + rôles tête/queue (tokenize / forward_tokens → hidden fp32-LE
  → detokenize) + boucle de décode autorégressive + framing correct — travail Rust NEUF substantiel (G2,
  confirmé : le framing actuel envoie `prompt.as_bytes()`, le forwarder réel exige des octets fp32-LE et
  n'appelle QUE `forward_hidden` ; la tokenisation n'existe QUE dans `examples/shard_node.rs` via pipe SSH,
  jamais sur l'ALPN ; le drive fige `tokens=1`, aucune boucle de décode). Plus build fork Metal/CUDA per-machine
  + poids GGUF ~20 Go sur les 2 machines. → contredit le scope-cut « J = 0 Rust acceptance » (c'est une phase
  feature, pas une acceptance mince).

Chaque option contredit une ligne FIGÉE du plan §J (carry-closure OU 0-Rust) ; le choix décide de ce que
S81 promet réellement (clôture du carry-phare sharding OU report). **C'est une décision produit PO →
DESIGN-CONFLICT**, à trancher AVANT toute exécution live (S6 le nomme identiquement).

## Rationale du verdict

Signaux des 7 scans : **S1a EXECUTE** (méthodologie du verdict, aucun conflit — au contraire elle
VALIDE le garde-fou anti-faux-BLOCK du plan), **S1b EXECUTE** (0 dep), **S2 EXECUTE-conditionnel**
(0 décision gelée violée ; 1 BLOCKER réfuté), **S3 EXECUTE** (surface) **avec 1 finding certif SI-9**,
**S4 EXECUTE** (0-bump) **avec 1 dette d'honnêteté**, **S5 DESIGN-CONFLICT** (echo-only → PASS creux),
**S6 DESIGN-CONFLICT** (echo-only + 2 mécanismes de preuve disjoints). Vérifications adversariales :
**2 claims réfutés** (S2-J-03 cosine, S6-5 skew wire — voir plus bas), tous les autres confirmés.

Le verdict global est **DESIGN-CONFLICT** (pas EXECUTE, pas PLAN-ADAPT) parce que les deux scans les plus
seniors sur le versant exécution (S5 inventaire harness, S6 rig) convergent sur un blocage STRUCTUREL qui
n'est PAS un simple re-scoping livrable-délégué-au-préflight (ce qu'était Phase I) : le chemin livré ne
peut produire qu'un faux-vert, et le remède franchit une frontière de périmètre figée que seul le PO peut
arbitrer. Il n'est PAS DESIGN-CONFLICT sur une Day-0 technique (aucune n'est heurtée) — le conflit porte
sur la **promesse de livrable de la phase** (clôture carry P1 vs 0-Rust), qui est une décision de scope PO.

**Ré-vérifications indépendantes (spot-checks au HEAD `58cef6d`, tous CONFIRMÉS) :**
- `main.rs:266` : `shard_protocol_factory(entry, Arc::new(EchoForwarder))` + `main.rs:278` imprime
  « serving sbfb/shard/1 (transport-only echo forwarder) ». `grep ShardBackendForwarder` repo-entier :
  défini/exporté `worker-core/src/llm/shard.rs:193,523,535` + doc `core-rs/src/shard.rs:237` ; **0 montage**
  hors `#[cfg(test)]` (`shard_session.rs:1121+`). → PASS creux garanti si joué tel quel.
- `b3_shard_pipeline.sh:367-375` : gate = `run_proof` non-vide ET `toks_per_s>=1` — les DEUX satisfaits par
  l'echo ; `:382` « per-shard signed, N0-N3 bound » et `:384-385` « verified response » = **libellés
  mensongers** sur le chemin echo.
- `sprint77_phase_f_spike.md:53` : le seul cross-backend mesuré du repo = CUDA→Metal `min_cosine`
  **0.99922882** (frontière `:45` = 0.99999888), verdict GO, `:57-60` = **base de calibration EXPLICITE de
  TOPLOC N0**. La valeur « 0.978 » n'apparaît NULLE PART sauf dans l'assertion non-sourcée du préflight I. →
  réfutation S2-J-03 confirmée.

## S1a — Prior-art benchmarking pipeline-parallel WAN — EXECUTE (valide le garde-fou du plan)

Adversarial : **7 confirmés / 0 réfuté** (ancres repo relues + chiffres externes Petals / leyten-shard
vérifiés WebFetch). La méthodologie du verdict est un SPEC cohérent, aligné sur les garde-fous
anti-faux-BLOCK du plan lui-même.

- **S1a-1 (FAIT CARDINAL)** — Le décodage single-stream d'un pipeline distribué est **latence-borné, pas
  bande-passante-borné** (~8–16 KB de hidden state/token, négligeable ; débit plafonné par Σ(compute + RTT
  par frontière), un round-trip par token). Le verdict J DOIT diagnostiquer le débit contre `1/(compute +
  RTT_frontier)`, jamais contre une cible bande passante ni un débit single-machine.
- **S1a-2 (SEUIL PASS HONNÊTE)** — Le débit décode single-stream vit dans une fenêtre **~1–3 tok/s**,
  indépendamment du GPU (Petals 3×A100 LAN = 1,71 steps/s → 1,23 @100 ms ; leyten/shard plancher = 1,87
  tok/s). PASS honnête = (a) tokens PRODUITS et CORRECTS bout-en-bout + (b) débit cohérent avec le plafond
  RTT-borné mesuré. Exiger un débit proche du single-machine (5080 seul = dizaines/centaines tok/s) ou les
  chiffres polis Petals/shard (quant + spéculatif) = **faux-BLOCK garanti**.
- **S1a-3 (HUB vs direct-s2s)** — Ratio traversées WAN = `2N/(N+1)` : tend vers 2× pour un long chaînage
  mais vaut **~1× pour UN seul shard distant**. Le bench 2-machines (1 frontière) coûte ~1 RTT dev↔Mac/token
  dans les DEUX topologies. Confirme le garde-fou du plan : juger le HUB contre l'enveloppe Petals direct-s2s
  §6 fixe une barre trop haute → faux-BLOCK.
- **S1a-4 (MÉTRIQUES)** — Trio minimal honnête à émettre = **{tok/s décode, TTFT compté SÉPARÉMENT, `rtt_frontier_ms`
  mesuré}** + preuve de correction des tokens.
- **S1a-5 (l'orchestrateur S'ASSIED sur le plancher)** — L'orchestrateur (HUB dialer single-stream) N'A PAS
  les astuces qui battent le plancher (async-pipelining/overlap, décodage spéculatif, quant blockwise, micro-batch).
  Il DOIT donc s'asseoir sur ~1–3 tok/s = comportement ATTENDU d'un HUB nu = **PASS**, pas une régression.
- **S1a-6 (PIÈGE Mac Metal)** — 1er run Metal = compilation shaders (plusieurs sec), cache persistant ensuite.
  Warm-up ≥1 run court JETÉ avant toute mesure ; cold vs warm reportés séparément (idem capture graphe CUDA 5080).
- **S1a-7 (PIÈGE calibration RTT)** — Débit RTT-borné → la calibration du chemin domine. iroh 1.0 direct
  (hole-punch) dev↔Mac 192.168.1.x = sous-ms LAN ; fallback relais VPS Helsinki = dizaines de ms (~10–50×).
  **Observation renforçante** : les 2 machines sont sur le MÊME LAN → la « frontière WAN » du plan est lâche,
  le bon chemin est sous-ms LAN. Le harness DOIT asserter un chemin DIRECT + loguer `rtt_frontier_ms` AVANT
  d'émettre PASS/BLOCK (= le prérequis DUR « calibration N0 »). Un BLOCK dû à un fallback relais non-calibré
  serait mal-diagnostiqué.
- Méthodo (S1a-8/9, INFO) : prompt fixe + décodage déterministe (greedy/temp 0, seed fixe) pour vérifier la
  correction ; ≥3–5 runs chauds ; médiane + dispersion ; jeter le 1er run. En pipeline single-stream, 1 seul
  nœud calcule à la fois (bulles) → débit mono-requête ~= single-machine est physiquement IMPOSSIBLE.

## S1b — Deps / advisories — EXECUTE

Adversarial : **S1b-3 confirmé (MAJOR)** ; reste INFO. 0 réfuté.

- **0 dépendance nouvelle** : `b3_shard_pipeline.sh` + `flip_convergence_check.sh` = pur bash+curl+ssh ; ils
  pilotent les routes daemon DÉJÀ livrées Phase I. Features `llm_llama_cpp_metal/cuda` + deps (llama-cpp-2,
  llguidance, cudarc) existent déjà dans `Cargo.toml`.
- `cargo deny check` sur HEAD = **VERT** (exit 0 : advisories/bans/licenses/sources ok). Aucun yank actif
  (`yanked="deny"` ne déclenche pas) ; le signal ed25519-dalek 3.0.0-rc.0 réellement carrié = WARNING
  duplicate/multiple-versions (`multiple-versions="warn"`), documenté **P2-AUDIT-2-RESIDUEL carry S82** ; 6
  RUSTSEC en ignore-list (hickory 0.24 ×4 + quick-xml ×2) routés S82. Rien ne bloque.
- **S1b-3 (MAJOR, prérequis toolchain)** — **AUCUN build arm64 Mac Metal du fork n'existe, AUCUNE lib native
  n'est committée** (fork VENDORÉ en-repo `vendor/llama-cpp-2` + `vendor/llama-cpp-sys-2/llama.cpp` = 913
  fichiers SOURCE, patch `patches/llama-cpp-shard.patch`, câblé `[patch.crates-io]` ; 0 binaire prébuild). Le
  CI ne build JAMAIS la feature (`Cargo.toml:498-499` « CI never builds that feature »). Un vrai PASS
  d'INFÉRENCE (Option B) exige un build FROM-SOURCE per-machine (5080 = cmake+CUDA+NASM `llm_llama_cpp_cuda` ;
  Mac = cmake+Xcode/Metal `llm_llama_cpp_metal`). La source atteint le Mac via bundle/scp (fork committé → le
  bundle le porte ENTIER, pas de fetch submodule). Le serve ECHO actuel n'exige NI Metal NI GGUF — ces
  prérequis ne mordent qu'en Option B.
- Précondition « jamais un mélange 0.98/1.0 » **OK au lock** : iroh uniformément 1.0.1 / iroh-blobs 0.103.0 /
  iroh-docs 0.101.0 / iroh-gossip 0.101.0 ; les 16 hits « 0.98 » de `Cargo.lock` sont des sous-chaînes de
  checksums. (Le FLIP H runtime des 3 nœuds reste PENDING operator — versant exécution, pas lock.)

## S2 — Décisions historiques (cohérence Day-0) — EXECUTE-conditionnel

Adversarial : **7 confirmés / 1 réfuté (S2-J-03)**.

- **S2-J-01 (PIÈGE)** — Le « §6 » cité = §6 « Enveloppe de performance » du **design addendum**
  (`sharding_design_addendum_sota_2026-05-30.md:146-166`), PAS le kickoff (0 mention Petals). Enveloppe
  direct-s2s (Petals 2.29→1.57 steps/s, « RTT>80 ms ou relais hot-path = NO-GO produit »). Juger J contre
  cette enveloppe = **faux-BLOCK garanti** ; juger contre une **baseline HUB**.
- **S2-J-02 (PIÈGE, ordonnancement DUR)** — Le bench tourne sur la stack POST-flip 1.0.1 ; le **FLIP H est
  PENDING operator** (commit `12e3954` « flip live PENDING operator »). Lancer avant le flip = flottes relais
  0.98/1.0 divergentes → partition possible → faux-BLOCK. **H (même session) AVANT J.**
- **S2-J-04 (FIGÉ)** — `RIG-ABSENT` légitime UNIQUEMENT si une machine est génuinement HS. L'excuse historique
  « orchestrateur absent » (racine S76/S77) est MORTE (Phase I `bb6c4f9` l'a livré). Rig nominal = MÊME
  matériel que l'axe transport → toute défaillance soft (convergence, N0 false-reject, backend lent) sort en
  `BLOCK{diagnosis}`, JAMAIS `RIG-ABSENT`.
- **S2-J-05 (FIGÉ, R12 amendé PO C1)** — Verdict = vocabulaire FERMÉ `PASS/BLOCK{diagnosis}/RIG-ABSENT` émis
  par le **HARNESS**, jamais de prose Claude ni `DIFFERE-*`. Délivrable = artefact JSON intégré au T2 bi-axe.
- **S2-J-06 (INVARIANT WIRE)** — Si le T2 exige le canal de retour des RunProofs par-worker, il DOIT rider un
  transport EXISTANT (feed raw-op additif / iroh-docs), **JAMAIS un nouvel ALPN de contrôle** (`shard.rs:4`
  control-plane sur docs/blobs/gossip figé). Ce transport initiateur→workers n'existe pas encore en code (R-J-6).
- **S2-J-07 (FIGÉ)** — Le RunProof exposé par `/result` = celui du DRIVER (tête), réduit à hex(signature),
  NON tiers-vérifiable (payload canonique + pubkey signataire absents, interdits SI-3/SI-4). Le harness gate
  sur `run_proof` NON-VIDE seulement. Ne PAS gater PASS sur une preuve per-worker tiers-vérifiable via `/result`.
- **S2-J-08 (PRÉREQUIS DUR PARTIELLEMENT DISCHARGÉ)** — Convergence WAN = prérequis DUR. Statut réel :
  E3 hot-subscribe convergence PROUVÉE LIVE PASS (commit `8872596`) MAIS **(a) sur la paire PC-dev↔VPS-ancre,
  PAS la paire rig shard PC-Win↔Mac-M2** ; (b) sur `presets::N0` PRE-EOL, PAS la stack post-flip 1.0.1 ;
  (c) carry S75 re-drive boot-SEED **OVERDUE 3/3 OUVERT** (escaladé à l'audit gate S81). Le préflight J DOIT
  ré-attester la convergence sur la paire/stack pertinente, sinon J hérite légitimement `RIG-ABSENT/BLOCK{convergence}`.
  **NUANCE croisée S6-2 (voir plus bas)** : la readiness-barrier shard DIAL en direct depuis le mount-config,
  donc la convergence WAN ne gate PAS le MOUNT shard — elle gate l'axe transport, pas l'axe shard.
- **RÉFUTÉ — S2-J-03 (ex-BLOCKER)** : la prémisse numérique « cosine MESURÉ 0.978 CUDA+Metal < seuil
  same-backend >0.999 → `BLOCK{n0-false-reject}` garanti » est **contredite par la mesure du repo**. Le seul
  cross-backend mesuré = 0.99922882 (frontière 0.99999888), verdict GO, base EXPLICITE de calibration TOPLOC
  N0 — donc AU-DESSUS de la ligne 0.999 citée : un split correct PASSERAIT. De plus **N0 TOPLOC n'est PAS
  seuillé sur le cosine** (gates exposant/mantisse/top-k, `lib.rs:216-217`, conçus pour la dérive cross-GPU).
  Le « 0.978 » n'est pas « mesuré ». Résidu **non-bloquant** : documenter/calibrer la fence au préflight J
  reste un item P2 prudent (déjà tracké R-I-2) — mais PAS un BLOCKER (**R-J-3 downgradé P2**).
- FIGÉ mineurs confirmés : STAR/HUB gelée, J = 0 Rust data-plane (S2-J-09) ; churn tranchée en I (S2-J-10) ;
  RTT multipath UNVERIFIED (S2-J-11) ; un PASS J ne franchit PAS Gate 1/3 ni ne lève R-iroh-audit P0 (S2-J-12).

## S3 — Threat model — EXECUTE (surface) + 1 finding certif SI-9

Adversarial : **S3-2 confirmé (MAJOR)** ; S3-4 MINOR ; reste INFO. 0 réfuté.

- **Surface INCHANGÉE** (S3-1) : la tête ne sert JAMAIS `sbfb/shard/1` (dial-only) ; `SHARD_ALPN` monté
  seulement par le worker `serve` dédié ; admission par-session signée `DOMAIN_SHARD_PLAN_V1` + `is_member`
  AVANT insert ; group/mount/generate duress-gatés. **0 nouvelle classe de menace, 0 nouveau `DOMAIN_*`,
  0 nouvel invite.** (Précision : ce sont **3** routes signantes duress-gatées, pas « 5 » — result/drop-shard
  sont lecture/coupe locales non-gatées.)
- **S3-2 (MAJOR, certif SI-9)** — Le run J tel que SCRIPTÉ **N'EXERCE PAS la voie SI-9** (deadline par-hop →
  re-route `fallback_node` → resume-from-cache) : le harness casse la boucle sur `result_text` PUIS appelle
  `drop-shard`, qui est la **coupe explicite comptée POST-drive** (`http.rs:2438-2443` « A mid-drive drop is
  handled by the SI-9 fallback path instead »), pas la voie withholding-timeout (couverte UNIQUEMENT par les
  tests hermétiques `shard_session.rs:1676/1925`). **K ne pourra certifier SI-9 §16 au-delà de hermetic +
  coupe-comptée-live que si J drope un shard PENDANT le décode** (injecter un withholding mid-drive pour que
  le hop deadline arme + fallback+replay tournent live), SINON le libellé de certif K DOIT être scopé
  « hermetic (fallback/resume + write-path) + live counted-cut » avec la voie withholding→fallback→resume
  carriée hermetic-only.
- **S3-4 (MINOR, fuite artefact)** — La voie failure de `GET /result` réinjecte un **préfixe 16-hex (64 bits)
  de la pubkey d'un membre du groupe PRIVÉ** (`sanitize_diagnostic` redige les 64-hex mais PRÉSERVE
  volontairement les troncatures 16-hex ; test `:1767-1772`), que le harness committe dans
  `.b3_shard_last_result.json` (`diagnosis`/`last_response`). Casse le whitelist SI-3/SI-4 que la route status
  applique strictement. Dommage réel faible (rig = 2 machines de l'opérateur) mais l'artefact committé est
  public (AGPL) et le motif généralise → **vérifier que l'artefact T2 committé ne porte aucun préfixe pubkey
  membre** (ou retirer `worker_hex` du champ failure HTTP, le garder en `info!` local).
- INFO : le token `x-sbfb-token` ne fuit PAS (parsé inline, jamais dans `last_response`, pas de `set -x` ;
  résidu : un `bash -x` le tracerait). Poids modèle transférés au Mac = modèle PUBLIC, activations déjà en
  clair (SI-1 résiduel ASSUME) → pas de surface de confidentialité neuve ; N0 détecte un GGUF/quant divergent
  (intégrité, aligner la quant cross-machine avant le run).

## S4 — Wire / contrats loopback — EXECUTE (0-bump) + 1 dette d'honnêteté

Adversarial : **S4-2 confirmé (MAJOR)** ; reste INFO. 0 réfuté.

- **0-BUMP CONFIRMÉ** (S4-1) : `SHARD_PLAN_FORMAT_VERSION` / `RUN_PROOF_FORMAT_VERSION` /
  `COMPUTE_GROUP_FORMAT_VERSION` restent à 1 ; `SHARD_ALPN=b"sbfb/shard/1"` + `DOMAIN_SHARD_PLAN_V1` inchangés.
  Tous les champs JSON lus par le harness (`found`/`rtt_frontier_ms` sur `/status` ;
  `result_text`/`ttft_s`/`toks_per_s`/`run_proof` sur `/result`) matchent 1:1 les DTOs, drift-gatés snapshot
  + whitelist SI-3/SI-4 verrouillée exact-keys.
- **Verdict DÉJÀ ÉMIS PAR LE HARNESS** (S4-3) : `rig_absent/block/pass` écrivent le champ `status` + exit
  3/1/0 + artefact JSON de forme fixe (encodeur dual python3/pure-bash). Phase J est un RUN, pas un câblage de
  verdict — MAIS le run tel-quel produit un faux-vert (cf. DESIGN-CONFLICT).
- **Gate RTT HUB-cohérente** (S4-4) : `rtt_frontier_ms` = RTT QUIC d'UN saut tête↔shard (unité baseline HUB),
  gate 80 ms triviale sur LAN ; RTT null correctement sautée (pas de faux-BLOCK).
- **S4-2 (MAJOR, dette d'honnêteté)** — Le harness proclame collecter/vérifier des **RunProofs PAR SHARD** +
  un **binding N0-N3** (`b3_shard_pipeline.sh:18-19,328,368,382`) alors que le wire n'expose qu'UNE preuve —
  celle du DRIVER (`shard_session.rs:441` + drive_pipeline signe UNE RunProof tête `participants=executed_by`).
  Le verdict `:367` teste correctement le run_proof unique non-vide, MAIS la trace PASS et la diagnose BLOCK
  affirment un fait NON vérifié. J doit SOIT (a) câbler le canal de retour per-worker (feed raw-op/docs,
  0-bump), SOIT **(b) corriger le libellé harness** en « driver-signed RunProof over the measured run » +
  re-router le binding per-shard N0-N3 en carry explicite.
- **S4-5 (MINEUR, artefact scratch)** — `.b3_shard_last_result.json` on-disk est PÉRIMÉ (narre le monde
  pré-Phase-I : « no production orchestrator creates one », « Phase J read-only STUB ») mais gitignored, écrasé
  au prochain run. La sortie par défaut `B3_ARTIFACT` pointe vers ce chemin gitignored → **le livrable T2
  committé (K) devra COPIER la sortie dans un fichier tracké distinct** ; réconcilier le schéma PLAT du harness
  avec l'enveloppe palier des artefacts axe-transport (ajouter `iroh_lock=1.0.1` + `date` + `vocabulary_note`).

## S5 — Inventaire harness/CLI pour jouer b3_shard LIVE — DESIGN-CONFLICT

Adversarial : **5 confirmés / 0 réfuté** (G1/G2/G5/G3/G4). Tout le contrôle-plane pour monter/piloter une
session shard 2-machines EXISTE et est câblé (CLI `shard-session identity|serve|group|mount|status|generate|
result|drop-shard`, 5 routes authed duress-gatées, orchestrateur 6 étapes, harness qui poll + émet le verdict
fermé). La SÉQUENCE opérateur (SEQ-1) est jouable telle quelle **pour une session TRANSPORT (echo)**. Mais :

- **G1-ECHO-ONLY (BLOCKER)** — Seul chemin `serve` livrable = `EchoForwarder` (transport-only). Le vrai
  `ShardBackendForwarder` a **0 call-site de montage**. Run J = PASS creux (echo) que le harness étiquette
  « ~20 GB sharded / verified response ». → **ne clôt PAS le carry P1** (racine du DESIGN-CONFLICT).
- **G2-FRAMING-DECODE (BLOCKER)** — Même G1 corrigé : incompatibilité de framing (drive envoie
  `prompt.as_bytes()`, le forwarder réel exige fp32-LE hidden-state, `forward_hidden` uniquement) + un seul
  passage `tokens=1`, aucune boucle de décode autorégressive. Tokenize/detokenize + rôles head/tail n'existent
  QUE dans `examples/shard_node.rs` (pipe SSH). **Une vraie génération cross-shard n'est PAS composable via
  serve+drive sans code neuf.**
- **G5-ANTIFALSEGREEN (MAJOR)** — Le gate verdict ne distingue PAS echo de vraie inférence (run_proof non-vide
  + toks_per_s>=1 satisfaits par l'echo). Ajouter un « tell » : exiger `result_text != prompt` OU `tokens>=2`
  OU `model_digest != 0` — sinon un run echo remonte un PASS creux au lieu d'un `BLOCK{echo-transport-only}`.
- **G3-NO-MOUNTCONFIG (MAJOR)** — Aucun générateur/template du mount-config JSON (`MountSessionRequest` =
  session_id + group signé VERBATIM + `workers[].addr` collés depuis stdout de `serve` + model spec forçant le
  split VRAM). `mount` lit un fichier JSON brut. Note `rig.local.env.example:32-36` PÉRIMÉE (« no production
  orchestrator creates one yet »). Erreur-prone → écrire un générateur.
- **G4-MAC-BINARY (MAJOR)** — Le Mac (`theophilevasseur@192.168.1.53` arm64) n'a pas de binaire au HEAD migré ;
  repo **≈15 commits devant origin** (`c899d54`) → source via bundle/scp + build in-place (respecter
  `Cargo.lock` iroh=1.0.1). Pour un run RÉEL : `--features llm_llama_cpp_metal` (build Metal) + GGUF ~20 Go.
- Nuances INFO : la readiness DIAL en direct (pas de gossip) → convergence WAN ne gate PAS le mount shard
  (NUANCE-DIRECTDIAL) ; le head daemon DOIT être en identity **Normal** (pas duress) sinon mount/generate
  échouent silencieusement (DURESS-GATE) ; l'admission du dial de la tête réussit (node_id == pow_keypair,
  HEAD-ADMISSION-OK) ; 2 schémas T2 divergents à réconcilier en K (T2-SCHEMA).

## S6 — Rig / convergence — DESIGN-CONFLICT

Adversarial : **5 confirmés / 1 réfuté (S6-5)**.

- **S6-1 (BLOCKER)** = G1 (echo-only) → PASS creux, ne clôt pas le carry P1. **Racine du DESIGN-CONFLICT.**
- **S6-2 (MAJOR)** — La convergence WAN (carry RE-DRIVE-ON-INGEST, boot-SEED, SeedAnnounced) est **HORS du
  chemin critique du data-plane shard** : le mount reçoit `workers[].addr` = EndpointAddr complet imprimé par
  `serve` ; `open_shard_connection` seed le MemoryLookup puis dial QUIC DIRECT — aucun docs/gossip/directory/
  pkarr. Le blocker S76 concerne la propagation de tâches coordinateur→worker via iroh-docs (un AUTRE plan).
  → convergence WAN gate l'axe TRANSPORT, pas l'axe shard.
- **S6-3 (MAJOR)** — Rig = **LAN, pas WAN** : Mac `192.168.1.53` (RFC1918) joignable SSH direct depuis le PC ;
  le VPS Helsinki ne participe PAS au bench 2-machines (ancre relay/pkarr seule). Dial QUIC direct ~1–5 ms,
  gate RTT 80 ms triviale. Ne PAS retuner `RTT_GATE_MS` ni lire un PASS LAN comme WAN.
- **S6-4 (MAJOR, calibration N0)** — Concrètement : booter le head + les 2 nœuds `serve` avec le **MÊME env
  zero-n0** (`SBFB_ZERO_N0=1` + `SBFB_ZERO_N0_PKARR_RELAYS` + `SBFB_CUSTOM_RELAYS`) au chokepoint unique
  (`node.rs:327,349`) — env non-uniforme = postures divergentes. QUIC addr-discovery DÉSACTIVÉ en Topologie B
  → échantillonner `rtt_frontier` APRÈS bascule directe (sinon fenêtre transitoire relay-routée Helsinki →
  faux `BLOCK{rtt>80ms}`).
- **S6-7 (MAJOR, 2 mécanismes de preuve DISJOINTS)** — (1) `examples/shard_node.rs` = split layer-block RÉEL
  bit-exact (cosine 0.99923) mais via pipe SSH stdin/stdout, **SANS iroh, SANS `sbfb/shard/1`** (indépendant
  de la migration/flip) ; (2) orchestrateur + harness = data-plane QUIC `sbfb/shard/1` mais **echo-only**.
  **Aucun des deux ne prouve aujourd'hui « inférence réelle SUR `sbfb/shard/1` cross-machine » bout-en-bout.**
  Phase J doit trancher lequel atteste le carry et documenter honnêtement ce que chacun NE prouve PAS.
- **RÉFUTÉ — S6-5 (ex-MAJOR)** : la prémisse « build depuis origin = mismatch wire 0.98/1.0 au handshake » est
  **FAUSSE** — `origin c899d54` EST déjà iroh 1.0.1 (c'est le commit Phase B « bump iroh =1.0.1 »). Pas de skew
  0.98/1.0. La conclusion actionnable « build Mac from HEAD via bundle/scp » **survit pour une AUTRE raison** :
  la sous-commande `shard-session serve` (et `shard_session.rs`) est ABSENTE à origin (arrive Phase I
  `bb6c4f9`) → origin n'a aucune commande `serve`. Guidance corrigée : bundle/scp HEAD + respecter
  `Cargo.lock` (iroh=1.0.1) pour la reproductibilité, PAS pour éviter un skew inexistant.
- MINEURs : flip H ne change RIEN au protocole de trame `sbfb/shard/1` (recompilé + handshake vert iroh 1.0 en
  E), commute seulement discovery/home-relay quasi-orthogonal sur LAN (S6-6) ; `rig.local.env` présent = celui
  de b3_live (pas shard) → créer un rig shard dédié (S6-8) ; un PASS J = preuve DRIVER seule, pas per-shard
  N0-N3 (S6-9).

## Prérequis GO/NO-GO du run live (checklist opérateur ordonnée)

**GATE 0 — DÉCISION PO (bloquante, AVANT tout) :** trancher **Option A** (J prouve transport+lifecycle
cross-machine echo, honnêtement scopé ; carry P1 sharding **reste OUVERT** ; 0 Rust data-plane mais harness
corrigé) **vs Option B** (câbler l'inférence réelle sur `sbfb/shard/1` — travail Rust NEUF substantiel + build
fork Metal/CUDA + GGUF ~20 Go — puis run pour clore le carry P1). Sans ce tranchage, J produit un faux-vert.

**Si Option A (transport-echo scopé) — prérequis minimum :**
1. **FLIP H d'abord, même session** : basculer les 3 nœuds sur iroh 1.0.1 ; vérifier `flip_convergence_check.sh`
   vert ; **jamais un mélange 0.98/1.0** (S2-J-02).
2. **Head daemon en identity Normal** (pas duress) sinon mount/generate échouent silencieusement (DURESS-GATE).
3. **Source au HEAD sur le Mac** : `git bundle`/scp de l'arbre (fork vendoré committé → porté entier) + build
   in-place `nexus-shell-daemon` (`Cargo.lock` iroh=1.0.1) ; **la sous-commande `serve` n'existe qu'au HEAD**
   (S6-5 corrigé).
4. **Générer le mount-config** (aucun générateur n'existe, G3) : session_id + group signé verbatim +
   `workers[].addr` depuis stdout de `serve` + model spec forçant le split VRAM. Créer un `rig.local.env` shard
   dédié (`MAC_SSH=theophilevasseur@192.168.1.53`, `PC_DAEMON` réel, `SHARD_SESSION_ID` de la session montée).
5. **Corriger le harness AVANT d'émettre un verdict** : ajouter le tell anti-faux-vert (`result_text != prompt`
   OU `tokens>=2` OU `model_digest != 0`, G5) + remplacer les libellés « verified response / per-shard signed,
   N0-N3 bound » par « driver-signed RunProof over the measured run, transport+lifecycle only » (S4-2).
6. **Calibration N0 / chemin direct** : env zero-n0 uniforme sur les 3 nœuds (S6-4) ; asserter un chemin DIRECT
   hole-punché dev↔Mac (LAN) + loguer `rtt_frontier_ms` APRÈS bascule directe (pas de relais Helsinki, S1a-7).
7. **Convergence** : la convergence WAN ne gate PAS le mount shard (dial direct, S6-2) — mais ré-attester la
   convergence transport si le T2 bi-axe l'exige (paire/stack pertinente, S2-J-08).
8. **Artefact** : copier la sortie harness dans un fichier T2 tracké distinct (le défaut est gitignored, S4-5) ;
   vérifier qu'aucun préfixe pubkey membre 16-hex ne fuit dans l'artefact committé (S3-4).

**Si Option B (inférence réelle) — prérequis ADDITIONNELS :** monter `ShardBackendForwarder` gaté dans `serve`
+ rôles tête/queue (tokenize/forward_tokens/hidden fp32-LE/detokenize) + boucle décode + framing (G1/G2) ;
build fork `--features llm_llama_cpp_metal` (Mac) / `_cuda` (5080) ; GGUF arch-llama ~20 Go sur les 2 machines,
quant ALIGNÉE cross-machine (N0 détecte un GGUF divergent, S3-5) ; warm-up ≥1 run jeté (compile shaders Metal,
S1a-6) ; décodage déterministe (seed fixe, greedy) pour vérifier la correction des tokens (S1a-8).

**Interprétation du verdict (les deux options) :** débit attendu **~1–3 tok/s** = plancher HUB single-stream
ATTENDU = PASS (S1a-2/5), JAMAIS comparé au single-machine ni à l'enveloppe Petals §6 (faux-BLOCK, S1a-2/3,
S2-J-01). `RIG-ABSENT` légitime UNIQUEMENT si une machine est génuinement HS (S2-J-04).

## Amendements au plan (§J) — proposés, subordonnés à la décision PO Gate 0

1. **Ligne livrable « si PASS → carry P1 sharding S77 CLOSED »** : à qualifier. Un PASS du chemin `serve` echo
   livré est **transport+lifecycle only** et ne clôt PAS le carry P1 (inférence). Selon Gate 0 : Option A → le
   carry reste OUVERT avec un verdict fermé scopé ; Option B seule le clôt.
2. **Ligne « Delta tests attendu : 0 Rust »** : vraie SEULEMENT en Option A (et encore : correction harness
   shell/checks). Option B = phase FEATURE (backend mount + head/tail + framing + décode) = delta Rust
   substantiel — plus une acceptance mince.
3. **Libellés harness** (S4-2/G5) : corriger indépendamment de A/B (« verified response » / « per-shard signed,
   N0-N3 bound » sont mensongers sur le chemin echo).
4. **Certif SI-9 pour K** (S3-2) : le run J scripté n'exerce PAS la voie withholding→fallback→resume ; soit J
   drope un shard MID-décode, soit le libellé de certif K est scopé « hermetic + live counted-cut ».
5. **R-J-3 (ex-S2-J-03) downgradé P2** : la fence N0 n'est PAS un BLOCKER (cosine réel 0.99923 > 0.999, N0
   non seuillé-cosine) ; simple item de calibration/documentation au préflight, pas un gate.
6. **T2 schéma** (S4-5/T2-SCHEMA) : envelopper l'artefact shard PLAT dans l'enveloppe palier
   (`iroh_lock=1.0.1` + `date` + `vocabulary_note`) pour coller à l'axe transport.

## Signal méta (à écrire — escalade de la trajectoire des préflights sharding)

Le compteur portait **2 PLAN-ADAPT consécutifs** (dont Phase I : re-scope +4..8→+8..14, routes/registre/SI-9
FORCÉS en I). Phase J n'est PAS un 3e PLAN-ADAPT mais une **escalade au-dessus** : DESIGN-CONFLICT. La cause
est la MÊME classe de dérive de deux préflights de suite — **le modèle du plan « S78 absorbé en I mince
(composer) + J mince (0-Rust run) » a systématiquement sous-estimé le travail d'inférence restant**. I a
révélé que la surface HTTP/registre/SI-9 n'était pas « composée » ; J révèle que **le backend d'inférence réel
n'a JAMAIS été câblé sur `sbfb/shard/1`** (seul l'echo l'est ; le split bit-exact vit sur un pipe SSH disjoint).
Conclusion méta pour l'audit gate S81 : la fermeture du **carry-phare sharding S77** dans S81 dépend d'une
décision PO de périmètre (A/B) qui n'était pas explicitée au kickoff — à porter au PO avant J, et à tracer au
`sprint82_audit_plan` si Option A (carry P1 sharding demeure OUVERT post-S81).

## Risques résiduels

- **R-J-1 (BLOCKER, Gate 0)** — Echo-only serve → PASS creux ; la clôture du carry P1 exige Option B
  (Rust neuf) ou l'acceptation d'un scope transport-only (Option A). Décision PO.
- **R-J-2 (BLOCKER, Option B)** — Framing + rôles head/tail + boucle décode absents de l'ALPN (existent
  seulement dans le pipe SSH) ; travail Rust substantiel, pas une composition.
- **R-J-3 (P2, ex-BLOCKER réfuté)** — Fence N0 : cosine réel 0.99923 > 0.999, N0 non seuillé-cosine →
  calibration/doc prudente au préflight, PAS un gate `BLOCK{n0-false-reject}`.
- **R-J-4 (MAJOR, honnêteté)** — Libellés harness mensongers (« verified response / per-shard signed ») +
  gate incapable de distinguer echo de vraie inférence → tell anti-faux-vert requis (S4-2/G5).
- **R-J-5 (MAJOR, certif K)** — SI-9 withholding→fallback→resume non exercé live par le run scripté →
  drop mid-décode OU libellé certif scopé (S3-2).
- **R-J-6 (INFO, transport manifeste/RunProofs)** — Le canal de retour per-worker (double-generate E2E)
  n'existe pas en code ; à rider 0-bump (feed raw-op/docs) SEULEMENT si le T2 l'exige, jamais un ALPN neuf.
- **R-J-7 (MINOR, artefacts)** — `B3_ARTIFACT` défaut gitignored → copier dans un fichier T2 tracké (K) ;
  vérifier l'absence de préfixe pubkey membre 16-hex (S3-4) ; réconcilier le schéma plat/palier.
- **R-J-8 (INFO, convergence)** — Convergence WAN ne gate PAS le mount shard (dial direct) mais gate l'axe
  transport ; carry S75 boot-SEED OVERDUE 3/3 escaladé à l'audit gate S81.

## Arbitrage PO (post-préflight, 2026-07-10)

**GATE 0 TRANCHÉ : Option B — inférence réelle.** Décision PO explicite en session
(AskUserQuestion, même jour que le préflight, FLIP H déjà DONE `bd5d680`). La Phase J
devient une phase FEATURE Rust (montage `ShardBackendForwarder` gaté + rôles head/tail +
boucle décode + framing fp32-LE) suivie du run live ; la ligne « Delta tests 0 Rust » du
plan §J est SUPERSEDED par cet arbitrage (amendement §J item 2 ci-dessus) ; la ligne
« si PASS → carry P1 CLOSED » redevient atteignable et reste le critère. Les corrections
harness R-J-4 (tell anti-faux-vert + libellés) restent dues AVANT tout verdict.

## Verdict final motivé

**DESIGN-CONFLICT.** Le but de Phase J (bench live 2-machines, verdict fermé émis par le harness, solder le
carry P1) ne heurte AUCUNE Day-0 technique, et les axes deps/threat-surface/wire sont propres (0 dep, 0 bump,
0 nouvelle classe de menace). Mais le chemin `serve` livré est **echo-only** : un run tel-quel émettrait un
**PASS creux** que le harness étiquette « verified response » — un faux-vert qui **ne clôt PAS le carry P1**.
Le remède franchit une frontière de périmètre FIGÉE du plan §J (soit « si PASS → carry CLOSED », soit
« 0 Rust acceptance ») selon deux options mutuellement exclusives (A transport-scopé / B inférence-réelle),
dont le choix décide de ce que S81 promet réellement — **une décision produit PO à trancher AVANT toute
exécution live** (convergence indépendante des 2 scans exécution S5+S6). Les findings méthodologiques (S1a)
et de cohérence (S2, S3, S4) constituent, une fois la décision prise, le SPEC complet du verdict et de la
checklist GO ci-dessus.
