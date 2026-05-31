# Sprint 71 Phase B Preflight

Date: 2026-05-30
HEAD: `2f9238d`
Verdict: **PLAN-ADAPT**

> Resume executif : le plan §6/D2 suppose que "forcer greedy+seed pour
> les taches verifiables" est principalement un cablage d'un nouveau
> champ de tache. La lecture de code prouve un constat plus fort et plus
> simple cote llama_cpp, ET un gap reel non identifie cote Ollama :
> 1. **llama_cpp est DEJA deterministe** (greedy argmax inconditionnel,
>    `llama_cpp.rs:327`) quel que soit `temperature` — il n'a besoin
>    d'AUCUNE modification de sampler. (S1a APPROACH-ALIGNED.)
> 2. **Ollama ne passe NI temperature NI seed** au daemon
>    (`ollama.rs:171-197` : aucun `.options()`) — il herite du defaut
>    Modelfile (~0.8) et est donc **non-deterministe aujourd'hui**.
>    C'est le vrai site B-2 a corriger (S1a LIB-EXISTS : `GenerationOptions`
>    de ollama-rs 0.2.6 expose `.temperature()` + `.seed()`).
> 3. Le champ de tache "verifiable" doit etre **signe** (dans les
>    canonical bytes), PAS dispatch-only comme `redundancy_factor`,
>    sinon le mode d'execution n'est pas authentifie (S4).
>
> Aucun finding S1b/S2/S3/S4 bloquant (pas de DESIGN-CONFLICT). Le seul
> finding bloquant pour le plan d'origine est S1a (LIB-EXISTS sur le
> chemin Ollama + APPROACH-ALIGNED redondant sur llama_cpp) → PLAN-ADAPT.

## Evidence Rules
- Claim policy : chaque affirmation cite un chemin:ligne, une sortie de
  commande, une URL datee, ou une hypothese explicite.
- Local sources read :
  - `prompts/agent/preflight.md` (procedure portable, integrale)
  - `.planning/active/sprint71_plan.md §6 (Phase B)`, `§10`, `§12`, `§13`
  - `.planning/active/sprint71_kickoff.md §5 (D2, D8)`, `§5 acknowledged (D2/D8 adjust)`
  - `.planning/active/sprint71_phase_a_preflight.md` (continuite tip/scope)
  - `crates/nexus-coordinator-rs/src/validator.rs` (quorum, integral)
  - `crates/nexus-coordinator-rs/src/db.rs:370-418` (set/insert/get_task_results)
  - `crates/nexus-coordinator-rs/src/redundancy.rs` (module mort, integral)
  - `crates/nexus-core-rs/src/task.rs` (Task, `task_canonical_bytes`, integral)
  - `crates/nexus-worker-core/src/engine/runtime.rs:899-918,1043-1085` (verify + submit)
  - `crates/nexus-worker-core/src/llm/mod.rs:170-254` (GenerateParams)
  - `crates/nexus-worker-core/src/llm/ollama.rs:147-217` (OllamaBackend::generate)
  - `crates/nexus-worker-core/src/llm/llama_cpp.rs:280-398` (sampler chain)
  - `crates/nexus-worker-core/src/llm/factory.rs` (backend selection)
  - `crates/nexus-worker-core/src/build_executor.rs` (execute_build, integral)
  - `crates/sbfb-factory/src/process.rs:7-60` (PROVIDERS vs LlmBackend)
  - `docs/security/THREAT_MODEL.md` (S3, sections quorum/collusion)
  - ollama-rs 0.2.6 source vendore (`~/.cargo/.../ollama-rs-0.2.6/src/generation/options.rs`)
- Commands run (extraits pertinents inline dans chaque scan).

## Scope
- Plan source : `.planning/active/sprint71_plan.md §6 (Phase B, lignes 194-256)`.
- Target files (plan §6 B.2) :
  - `crates/nexus-worker-core/src/engine/runtime.rs` — site de soumission (l.1043-1054), forcer greedy+seed pour taches verifiables.
  - `crates/nexus-core-rs/src/task.rs` — champ/flag `verifiable` (decision S4 ci-dessous).
  - `crates/nexus-coordinator-rs/src/validator.rs` — inchange logiquement, doc/assert.
  - `crates/nexus-coordinator-rs/src/redundancy.rs` — retrait (mort) ou DEPRECATED (D8).
  - `crates/nexus-worker-core/src/build_executor.rs` — `execute_build` (l.126) cabler/retirer (D8).
  - `crates/sbfb-factory/src/process.rs:24` — distinction provider/backend documentee (D8).
  - `crates/nexus-worker-core/src/llm/mod.rs` — **AJOUT au scope plan** : champ `seed` sur `GenerateParams` (cf. PLAN-ADAPT).
  - `crates/nexus-worker-core/src/llm/ollama.rs` — **AJOUT au scope plan** : cabler `.options(temperature, seed)` (cf. PLAN-ADAPT).
  - `docs/rust/PATTERNS.md` — decision provider/backend + greedy quorum + deps off-sprint.
- Deps/APIs/specs touches : ollama-rs 0.2.6 (`GenerationOptions`), llama-cpp-2 0.1.146 (sampler). Deps off-sprint G13 : portable-pty 0.9.0, async-stream 0.3.6, futures 0.3.32.
- Security/protocol surfaces : canonical bytes `Task` (decision verifiable signed-vs-dispatch), `TASK_FORMAT_VERSION` (reste 1), quorum result-spoof.
- Tests expected (plan §6 B.3) : `verifiable_task_uses_greedy_seed`, `two_honest_workers_same_hash`, `quorum_accepts_deterministic_redundancy`, `quorum_rejects_nondeterministic_divergence`, + trace G13 CVE.

---

## S1a OSS Prior Art

- **Domain** : determinisme d'inference LLM pour quorum hash-exact (greedy
  decoding + seed). Familles de reference : llama.cpp (moteur), Ollama
  (wrapper), BOINC/reproducible-builds (quorum sur sortie deterministe —
  deja cite par S55 commit `0cb576d`).

- **Sources (consultees 2026-05-30)** :
  - Ollama API `/api/generate` options — context7 `/ollama/ollama`
    (doc `api.md`) : exemple "Generate with Reproducible Outputs (Seed)"
    `options.seed`, et "Chat Request" `options.seed + options.temperature:0`.
    Confirme : `seed` + `temperature` sont des options runtime per-requete.
  - GitHub `ollama/ollama#5321` "Generated outputs inconsistent despite
    seed and temperature" — non-determinisme residuel rapporte meme avec
    `seed=123, temperature=0, num_ctx` fixe ; le rapport note un ecart
    1ere-vs-Nieme execution (warm-up/init), pas un non-determinisme
    fondamental. https://github.com/ollama/ollama/issues/5321
  - Web consensus (substack Kleine "Seed vs Temperature", Castillo
    "Controlling randomness in LLMs", tspi.at ollamaparams) : temperature=0
    => greedy (argmax) => deterministe ; **mais temperature=0 seule n'est
    pas pleinement deterministe sans seed fixe a cause du float GPU
    non-deterministe** — recommandation universelle : poser temperature=0
    ET seed fixe pour reproductibilite same-machine.
  - llama.cpp `ggml-org/llama.cpp#3005` "Enable Fully Greedy Decoding" +
    PR #9897 : un sampler greedy/argmax terminal est deterministe par
    definition ; **la temperature divise les logits mais ne change pas
    l'argmax** (scaling positif preserve l'ordre des logits). Maintainer :
    "Setting temp = 0 will no longer be equivalent to greedy decoding" —
    d'ou l'importance d'un selecteur greedy EXPLICITE plutot que de
    s'appuyer sur temp=0. https://github.com/ggml-org/llama.cpp/discussions/3005
  - ollama-rs 0.2.6 (dep pinnee, source vendore) :
    `src/generation/options.rs:80` `pub fn temperature(self, f32)`,
    `:86` `pub fn seed(self, i32)`, `:110` `top_k`, `:116` `top_p` ;
    `src/generation/completion/request.rs:74` `pub fn options(GenerationOptions)`.
    => l'API seed+temperature deterministe **existe deja** dans la version
    pinnee, elle n'est simplement pas utilisee par le code worker.

- **Constats de code verifies (le coeur du finding)** :
  1. **llama_cpp backend est DEJA deterministe** :
     `crates/nexus-worker-core/src/llm/llama_cpp.rs:325-327` construit
     `LlamaSampler::chain_simple([LlamaSampler::temp(temp), LlamaSampler::greedy()])`.
     `greedy()` est un selecteur argmax TERMINAL et inconditionnel. La
     temperature `temp(temp)` (defaut 0.7, `:325`) divise les logits mais
     ne deplace pas l'argmax (preuve OSS llama.cpp #3005). Le chemin
     watermark (`:373-377`) chaine aussi `temp(temp), greedy()`. **Donc le
     backend llama_cpp produit deja une sortie deterministe same-machine
     pour un prompt+modele+contexte donnes, sans aucun changement.**
     Classification : **APPROACH-ALIGNED** (le plan veut greedy ; le code
     l'a deja sur ce backend).
  2. **Ollama backend NE passe NI temperature NI seed** :
     `crates/nexus-worker-core/src/llm/ollama.rs:171-180` (`req_build`)
     attache seulement `system` + `format` (schema). Aucun `.options()`.
     => Ollama applique le defaut Modelfile (`temperature ~0.8`,
     `top_k=40`, `top_p=0.9`, seed aleatoire). **Le backend Ollama est
     non-deterministe aujourd'hui** — deux workers honnetes Ollama
     produiront presque toujours des `result_text` differents → quorum
     hash-exact echoue. C'est le **vrai gap B-2**, non identifie dans le
     plan (qui parlait du site `runtime.rs` generique).
     Classification : **LIB-EXISTS** — `GenerationOptions` de ollama-rs
     0.2.6 couvre exactement le besoin (`.temperature(0.0).seed(fixed)`).
  3. **`GenerateParams` n'a pas de champ `seed`** (`llm/mod.rs:172-203`).
     `temperature: Option<f32>` existe. Pour le determinisme :
     - llama_cpp : `temperature=Some(0.0)` suffit (greedy deja terminal) ;
       le seed est **inerte** au niveau argmax (greedy ne tire pas).
     - Ollama : il faut poser `.temperature(0.0)` ET `.seed(fixed)` sur
       `GenerationOptions`, car Ollama (llama.cpp interne) peut garder un
       residu non-deterministe sans seed (issue #5321). Un champ `seed`
       sur `GenerateParams` est donc **necessaire** pour le chemin Ollama.

- **Finding** : **LIB-EXISTS (Ollama path) + APPROACH-ALIGNED (llama_cpp path)**.
- **Impact** : **adaptation requise (PLAN-ADAPT)**. Le plan suggere
  "forcer greedy+seed au site de soumission wor->backend" comme si les
  deux backends avaient besoin du meme traitement. La realite :
  - llama_cpp : poser `temperature=Some(0.0)` (cohrence/lisibilite ;
    fonctionnellement deja greedy) — pas de changement de sampler.
  - Ollama : ajouter un champ `seed` a `GenerateParams`, le cabler dans
    `OllamaBackend::generate` via `GenerationOptions::default()
    .temperature(0.0).seed(SEED)`, et poser `top_k`/`top_p` si on veut
    durcir (optionnel same-machine). **C'est le delta reel vs le plan.**

> Reponse explicite a la question PO du brief : **non, poser uniquement
> temperature=0 ne suffit PAS** dans le cas general, parce que le backend
> Ollama (chemin par defaut `BackendKind::Ollama`, `factory.rs:42`) ne
> passe meme pas la temperature aujourd'hui et reste sujet au residu
> float-GPU non-deterministe sans seed (preuve issue #5321 + consensus
> web). Il faut **temperature=0 ET seed fixe** plumbe jusqu'a Ollama.
> Pour llama_cpp seul, temperature=0 (ou meme le greedy deja en place)
> suffit et le seed est inerte. Comme le worker peut tourner sur l'un OU
> l'autre backend (`BackendKind`, defaut Ollama), l'implementation
> robuste pose les DEUX (temperature=0 + seed) — le seed etant ignore par
> le chemin greedy llama_cpp, pose par le chemin Ollama.

---

## S1b Dependencies, CVEs, Release Notes

- **Scanned** (versions extraites de `Cargo.lock` via grep) :
  - `portable-pty 0.9.0` (G13)
  - `async-stream 0.3.6` (G13)
  - `futures 0.3.32` (G13)
  - `ollama-rs 0.2.6` (touchee par PLAN-ADAPT)
  - `llama-cpp-2 0.1.146` (touchee — lecture seule, pas de bump)
- **Commande** :
  `grep -A2 -E '^name = "(portable-pty|async-stream|futures|ollama-rs|llama-cpp-2)"' Cargo.lock`
  → portable-pty 0.9.0 / async-stream 0.3.6 / futures 0.3.32 / ollama-rs 0.2.6 / llama-cpp-2 0.1.146.
- **Evaluation advisories (RustSec / GHSA, etat 2026-05-30)** :
  - **portable-pty 0.9.0** : crate wezterm (PTY cross-platform). Aucun
    RustSec advisory connu sur la lignee 0.9.x. Surface : spawn de
    process terminal cote Factory Operator (local, loopback, gate Phase C).
    N'est PAS sur un chemin crypto/wire/network expose au reseau P2P.
    Finding : **non-bloquant** (pas de CVE, surface locale gardee).
  - **async-stream 0.3.6** : macro de stream async (tokio-rs). Pur
    glue async, pas de surface reseau/crypto. Aucun advisory.
    Finding : **non-bloquant**.
  - **futures 0.3.32** : runtime async fondamental, ecosysteme tokio.
    Version stable, aucun advisory critique ouvert sur 0.3.x. (Le seul
    advisory historique notable de l'ecosysteme — RUSTSEC-2020-0059
    `futures-util` Mutex — est resolu depuis bien avant 0.3.32.)
    Finding : **non-bloquant**.
  - **ollama-rs 0.2.6** : client HTTP local Ollama. Pas de CVE connue.
    L'API `GenerationOptions` utilisee par PLAN-ADAPT est stable dans
    0.2.x. Pas de bump requis. Finding : **non-bloquant**.
  - **llama-cpp-2 0.1.146** : binding llama.cpp. Lecture seule
    (aucun changement de sampler requis). Pas de bump. Finding : **clean**.
- **Note** : un `cargo audit`/`cargo deny` live (reseau) confirmerait
  l'absence d'advisory a date ; en environnement preflight, l'evaluation
  ci-dessus s'appuie sur l'historique RustSec connu + la nature des
  crates. Aucune des 3 deps G13 n'est sur un chemin crypto/wire/network
  expose — meme une advisory mineure ne serait pas bloquante.
- **Finding S1b** : **clean (non-bloquant)**. Aucune CVE critique/high
  sur crypto/wire/network/sandbox/signing. Aucun breaking major sur une
  API que la phase utilise. Trace G13 satisfaite (3 deps scannees, versions
  citees). (Fail-fast row #16 satisfaite.)

---

## S2 Historical Decisions

- **Commandes** :
  - `git log --all --oneline -- <validator.rs|redundancy.rs|build_executor.rs|task.rs>`
  - `git show 34c77ce/0cb576d/dc163ea --no-patch --format=%B`
  - reverse-commit : `git log --all --oneline 0cb576d..HEAD -- redundancy.rs`
  - `rg "RedundancyDispatcher::new|register_task|collect_result|redundancy::"`

- **Decisions traversees** :

  1. **Exclusion de `redundancy_factor` des canonical bytes — S23
     `34c77ce`** (P1 audit gate, plan §13 R3). Rationale : `redundancy_factor`
     est une **politique de dispatch coordinateur, pas l'identite crypto
     de la tache** ; deux coordinateurs signant la meme tache logique avec
     des facteurs differents doivent produire la meme signature. Implemente
     via `task_canonical_bytes()` qui retire le champ avant JCS
     (`task.rs:39-52`). **Statut : valide, toujours en vigueur.** Pertinent
     directement pour la decision S4 du champ `verifiable` (voir S4).
     Reversion ? Non — confirme present au HEAD (`task.rs:39-52` + tests
     `task_canonical_excludes_redundancy_factor`). **Non-bloquant** (sert
     de PRECEDENT, pas de conflit).

  2. **Quorum SHA256 DB-backed — S55 `0cb576d`** (origine de
     `validate_quorum`). Body explicite : *"DB-persistent (survives
     restarts) **vs in-memory RedundancyDispatcher existant**"*. Conçu
     pour les **build tasks** (`BUILD_DEFAULT_REDUNDANCY=3`, sortie binaire
     deterministe a la reproducible-builds.org), comparaison de hash. Le
     nom de colonne `sha256` vient de la (hash de binaire de build). Le
     call-site inference (`validator.rs:68`) y passe `result_text` brut —
     extension semantique S71 : la "colonne sha256" stocke en fait le
     `result_text` (egalite brute), pas un hash. **Confirme** par lecture :
     `db.rs:385-418` (`insert_task_result(sha256)`, `get_task_results`
     selectionne la colonne `sha256` litterale) + `validator.rs:68,91,115`.
     Aucun endroit ne hash reellement le `result_text` cote quorum
     inference. **Statut : valide ; le quorum n'a PAS besoin de changer
     (le plan §6 le confirme).** Non-bloquant.

  3. **`RedundancyDispatcher` (Rust) — port S40 `0b9df49` de redundancy.py
     S23 `dc163ea`**, supersede par S55 `0cb576d`. **Reverse-commit check** :
     `git log --all --oneline 0cb576d..HEAD -- redundancy.rs` → **0 commit**
     (aucune evolution depuis S55, sauf l'upgrade edition 2024 `1d010b0`
     mecanique). `rg "RedundancyDispatcher::new|register_task|collect_result|
     redundancy::"` hors `redundancy.rs` → **0 appelant vivant** (les hits
     `register_task_doc` dans `runtime.rs`/`dispatch_loop.rs` sont une
     methode d'engine SANS rapport, `runtime.rs:548`). `pub mod redundancy`
     est exporte (`lib.rs:32`) mais jamais importe. **Conclusion : module
     mort total, reversion CONFIRMEE (S55 l'a remplace par le quorum DB).**
     → D8 retrait justifie. **Non-bloquant** (confirmed reversion).
     Decision D8/R7 : aucun appelant S75 nommable n'existe dans le code
     ou la roadmap pour CE struct precis (S75 = GPU partage cross-machine,
     qui reutiliserait le quorum DB-backed, pas l'in-memory mort). →
     **retrait pur** recommande (pas DEPRECATED), avec note PATTERNS.

  4. **`execute_build` — S55 `2a17c0b` (build executor) + S56 `cff6d06`
     (dette pair P2)**. `execute_build` (`build_executor.rs:126`) wrappe
     `execute_build_with_timeout` (`:130`). `rg execute_build` → seul
     `execute_build` (wrapper) appelle `execute_build_with_timeout` ;
     **aucun appelant externe ni de l'un ni de l'autre**. Les tests
     internes (`build_timeout_expires`, etc.) testent `sha256_file` et
     `wait_child_with_timeout`, **pas** `execute_build`/`_with_timeout`.
     LT-7 Tier 2 (build pipeline) etait le consommateur prevu ; le worker
     reel ne cable jamais ce chemin. **Reversion ambigue** : pas
     explicitement annule, mais jamais branche. Decision D8/R7 : c'est
     du code LT-7 (self-hosted build, Tier 1+2 DONE S55, Tier 3 infra
     S60) — un appelant futur est **nommable** (LT-7 worker quorum E2E,
     carry post-tag, lie a S75). → **DEPRECATED + ROADMAP_COMMITMENTS**
     plutot que retrait, OU cablage dans le chemin build worker si le
     dispatcher route deja `task_type="build"`. Decision finale a la
     lecture du chemin de routage `task_type` en Phase B (le plan le
     prevoit). **Non-bloquant** (ambiguous reversion → SCOPE-CUT-CONSISTENT
     interne au choix D8).

  5. **logprobs/watermark = V2 (inerte aujourd'hui)** — confirme
     `kickoff §5 D2` + `plan §12 #13` + code `runtime.rs:1081`
     (`logprobs_hash: [0u8; 32]`) et `model_digest = blake3(model_name)`
     (`runtime.rs:1072`, non discriminant). Le deferral est une **decision
     PO actee** (PO-11), pas un gap a combler S71. **Non-bloquant**
     (documented future gap).

- **Finding S2** : **clean (non-bloquant)**. Toutes les decisions
  traversees sont soit des precedents valides (exclusion canonical S23,
  quorum DB S55), soit des reversions confirmees (RedundancyDispatcher),
  soit des deferrals PO actes (logprobs V2). Aucune decision rejetee
  re-introduite sans reversion avec rationale toujours valide → **pas de
  DESIGN-CONFLICT S2**.

---

## S3 Local Patterns And Threat Model

- **Threats/contracts checked** : result-spoofing compute (S23 "Gate 3
  C-ResultSpoof foundation", `dc163ea`), collusion worker, rejet des
  outliers (`validator.rs:113-156`), Sybil (THREAT_MODEL §residual).
  S3 **FULL** (la phase touche un composant securite : le mode
  d'execution authentifie d'une tache + la propriete de quorum).

- **Asset** : l'integrite du resultat compute (le quorum est la
  mitigation centrale contre un worker qui ment sur sa sortie / facture
  des kudos sans calculer).

- **Actors / vectors** : worker malveillant (sortie forgee), workers en
  collusion (Sybil multi-keypair), worker honnete non-deterministe (faux
  outlier).

- **Le forcage greedy/seed ouvre-t-il une surface d'attaque ?**
  Analyse :
  - **Determinisme = pre-requis du quorum hash-exact, pas une faiblesse.**
    Avant B-2, le quorum etait inutilisable pour l'inference (deux honnetes
    divergent → tout rejete). Greedy+seed rend les honnetes convergents.
    Un worker **malveillant** qui voudrait passer le quorum doit produire
    le MEME `result_text` que les honnetes — c.-a-d. **calculer reellement**
    la sortie greedy correcte. Le determinisme n'AIDE PAS l'attaquant : il
    doit reproduire l'argmax exact, ce qui requiert le bon modele + le bon
    prompt + le bon contexte (= le travail honnete). **Surface non
    aggravee.**
  - **Collusion** : deux workers malveillants colludant pouvaient deja
    converger sur une fausse sortie commune AVANT B-2 (ils choisissent le
    meme faux texte). Le determinisme ne change pas ce vecteur — il est
    deja le **residual M (Sybil multi-keypair)** documente
    (THREAT_MODEL §654). Mitigations existantes inchangees : attribution
    Ed25519 (`ResultEntry` signe, `validator.rs:29`), majorite stricte
    (`best_count > majority_threshold`, `validator.rs:125`), detection
    outliers (`validator.rs:128-138`). **Pas de regression.**
  - **Le seed fixe est-il un secret ?** Non — c'est un parametre PUBLIC
    de reproductibilite (constant ou derive du task_id). Le connaitre
    n'aide pas a forger : il faut quand meme l'inference reelle. (A
    documenter PATTERNS : le seed determinisme N'EST PAS le
    `watermark_seed` PRF, qui lui reste un secret par-tache —
    `task.rs:179-187`. Deux notions de "seed" distinctes a ne pas
    confondre.)

- **Le rejet des outliers est-il preserve ?** **Oui.** `validate_quorum`
  (`validator.rs:113-156`) reste inchange logiquement : il compte les
  `r.sha256` (= `result_text`) identiques, exige une majorite stricte
  (`best_count > redundancy_factor/2`), et logge/rejette les divergents.
  Avec les sorties deterministes, les honnetes tombent dans le meme
  bucket → la majorite emerge ; un worker non-deterministe (ou
  malveillant) tombe en outlier et est detecte
  (`quorum_single_outlier_detected` test existant, `validator.rs:408`).
  Le test plan B.4 `quorum_rejects_nondeterministic_divergence` verrouille
  explicitement cette propriete. **Pas de regression sur un threat
  couvert.**

- **Limite documentee (D2 ⚠️ / R1)** : le determinisme greedy est garanti
  **same-machine / same-backend / same-model-quant**. Cross-GPU, le float
  non-deterministe peut casser le bit-exact (preuve OSS : consensus web +
  issue #5321). Le test B-3 tourne same-machine (dev) ; la preuve
  cross-GPU reelle est **scope-cut S75** (plan §12 #11). A documenter
  PATTERNS sans pretendre le cross-GPU.

- **HARDENING_ROADMAP** : pas de pre-requirement S71 manquant identifie
  pour ce scope (la phase DURCIT le quorum, elle ne regresse rien).

- **Finding S3** : **clean (non-bloquant)**. Le forcage greedy/seed
  n'ouvre pas de nouvelle surface d'attaque (l'attaquant doit toujours
  faire le travail honnete pour passer le quorum) ; le rejet des outliers
  et la detection de divergence sont preserves. Limite cross-GPU
  documentee (carry S75). **Pas de DESIGN-CONFLICT S3.**

---

## S4 Protocol And Wire Invariants

- **Wire/security files checked** : `crates/nexus-core-rs/src/task.rs`
  (`Task`, `task_canonical_bytes`, `TASK_FORMAT_VERSION`),
  `crates/nexus-worker-core/src/llm/mod.rs` (`GenerateParams`, hors-wire).

### Decision CRITIQUE : le champ `verifiable` est SIGNED, pas dispatch-only

- **Question** : ou placer le mode deterministe ? Trois options examinees :
  - (A) Champ `verifiable: bool` **signe** (dans les canonical bytes).
  - (B) Champ `verifiable: bool` **dispatch-only** (exclu des canonical
    bytes, comme `redundancy_factor`).
  - (C) Pas de champ : deriver le mode de `redundancy_factor > 1`.

- **Tranche : (A) — champ SIGNE, inclus dans les canonical bytes.**

- **Justification (evidence-backed)** :
  1. **Le mode deterministe change la SEMANTIQUE d'execution worker**
     (greedy vs sampling). Pour que **tous les workers d'un quorum
     s'accordent sur le meme mode**, le mode doit faire partie de
     l'**identite cryptographique** de la tache que chaque worker verifie
     (`runtime.rs:899` `verify_signature()` AVANT execution). Si le mode
     etait dispatch-only (exclu des canonical bytes), un coordinateur (ou
     un MITM applicatif) pourrait servir `verifiable=true` a un worker et
     `verifiable=false` a un autre **sans casser la signature** → les deux
     workers executent en modes differents → divergence legitime →
     faux rejet de quorum, OU pire, ouverture a une manipulation du mode.
     → Le mode DOIT etre signe. **C'est l'inverse exact de
     `redundancy_factor`** : `redundancy_factor` est exclu (S23 `34c77ce`)
     PRECISEMENT parce qu'il NE change PAS ce que le worker calcule (un
     worker fait le meme travail que la tache soit repliquee 1x ou 3x) —
     c'est une politique cote coordinateur. `verifiable` change ce que le
     worker calcule → identite, pas politique.
  2. **Le worker ne lit deja PAS `redundancy_factor`** (grep
     `runtime.rs` : 0 hit `redundancy_factor`, le worker lit
     `is_open_source` `runtime.rs:914` mais pas le facteur). Donc l'option
     (C) "deriver de redundancy_factor>1" est **infaisable** cote worker :
     le worker n'a pas (et ne doit pas avoir, champ exclu non authentifie)
     ce signal. Le pattern correct est `is_open_source` (`task.rs:142`,
     **signe**, lu par le worker `runtime.rs:914`) — `verifiable` suit
     EXACTEMENT ce pattern.
  3. **Pattern a suivre** : `is_open_source` (`task.rs:126-143`) — `bool`,
     `#[serde(default)]`, signe (participe aux canonical bytes car NON
     retire par `task_canonical_bytes`), lu par le worker pour decider
     son comportement (consent L2). `verifiable` = meme forme.

- **Forme concrete** (PLAN-ADAPT n'altere pas cette decision wire) :
  ```rust
  /// True iff this task requires deterministic (greedy, fixed-seed)
  /// inference so independent workers reach an identical result_text
  /// for hash-exact quorum. Set by the coordinator at craft time.
  /// Part of the signed canonical bytes (unlike redundancy_factor):
  /// the execution MODE is task identity, not dispatch policy, so all
  /// workers in a quorum must agree on it under one signature.
  /// `#[serde(default)]` = runtime tolerance (omitted => false =
  /// best-effort sampling, the pre-S71 behavior).
  #[serde(default)]
  pub verifiable: bool,
  ```
  - **NE PAS** ajouter `obj.remove("verifiable")` dans
    `task_canonical_bytes` (`task.rs:42-44`) — le champ DOIT rester dans
    les bytes signes. (Contraste explicite avec la ligne
    `obj.remove("redundancy_factor")`.)

- **Impact canonical bytes / signature** :
  - Ajouter un champ signe **change les canonical bytes** de toute tache
    qui le serialise. Conforme pre-launch (cf. ci-dessous) : on **redefinit
    la v1 courante** (aucun noeud tiers en prod). Les anciens tests
    "legacy decode" qui figent une forme v1 sans `verifiable` devront etre
    mis a jour (pattern §Pre-launch policy : redefinir v1, supprimer les
    zombies). Le test `task_canonical_bytes_contain_the_four_consent_fields`
    (`task.rs:711`) et `task_wire_default_factor_1` (`task.rs:767`) sont a
    revoir : ils asserent une forme JSON precise.
  - **`#[serde(default)]`** : justifie comme **runtime tolerance** (un
    client minimal qui omet le champ obtient `false` = sampling
    best-effort, pas une erreur 422), PAS comme compat historique.
    Rationale a ecrire dans la doc du champ (meme phrasing que
    `is_open_source` / `redundancy_factor`).

- **`TASK_FORMAT_VERSION`** : **reste 1** (`task.rs:61`). Aucune CVE
  n'exige un bump ; pre-launch policy autorise l'edition libre du
  canonical sans bump (CLAUDE.md §Pre-launch). **Conforme.**

- **Champ `seed` sur `GenerateParams`** (`llm/mod.rs`) : **HORS-WIRE**.
  `GenerateParams` est un type interne worker (params d'appel backend),
  pas un type de protocole signe/gossipe. Ajouter `seed: Option<u32>`
  (ou la valeur de seed deterministe constante) **n'a AUCUN impact sur
  les canonical bytes ni la signature**. (A ne pas confondre avec un
  champ wire.) Le seed deterministe peut etre une constante worker-wide
  (p.ex. `0`) ou derive du `task_id` — choix d'implementation Phase B,
  pas une decision wire.

- **Day 0 status** : **preserved.** D2 (greedy seed-fixe, PO-11) respecte ;
  logprobs/watermark restent V2 (`#13`). Le quorum (`validator.rs`)
  inchange logiquement. `redundancy_factor` reste dispatch-only (S23
  preserve). Pas de re-debat des Day 0 figees.

- **Finding S4** : **clean (non-bloquant)**. La decision "verifiable =
  champ signe" est conforme au pattern `is_open_source` et a la logique
  S23 (mode=identite vs policy=dispatch). `TASK_FORMAT_VERSION` reste 1
  (pre-launch). `#[serde(default)]` = runtime tolerance documentee. Aucun
  decodeur multi-version tolerant introduit. **Pas de DESIGN-CONFLICT S4.**

---

## Plan Adaptation

> Requis car S1a = **LIB-EXISTS (Ollama) + APPROACH-ALIGNED (llama_cpp)**.

- **Original plan** (§6 D2 / B.2) : "forcer l'inference greedy
  (`temperature=0`, seed fixe) pour les taches verifiables [...] au chemin
  de soumission worker→backend (`runtime.rs:1043-1054`)" — formule comme
  si un traitement uniforme au site de soumission suffisait, et comme si
  l'ajout d'un champ de tache + le reglage de temperature etait l'essentiel.

- **Evidence requiring adaptation** :
  - `llm/llama_cpp.rs:327` — le backend llama_cpp chaine deja
    `greedy()` TERMINAL → deja deterministe quel que soit `temp` (preuve
    OSS llama.cpp #3005 : argmax invariant sous scaling temperature).
  - `llm/ollama.rs:171-197` — le backend Ollama ne passe NI temperature
    NI seed → non-deterministe aujourd'hui (defaut Modelfile ~0.8).
  - ollama-rs 0.2.6 `generation/options.rs:80,86` — `GenerationOptions`
    expose `.temperature()` + `.seed()` ; `completion/request.rs:74`
    `.options()`. L'outil existe, il n'est pas branche.
  - issue ollama/ollama#5321 + consensus web — temperature=0 seule
    insuffisante sans seed (residu float-GPU).

- **Corrected approach** (concret) :
  1. **`GenerateParams`** (`llm/mod.rs:172-203`) : ajouter
     `seed: Option<u32>` (+ builder `.with_seed()`) + un drapeau de
     determinisme (ou reutiliser `temperature=Some(0.0)` comme signal).
     Hors-wire — aucun impact canonical.
  2. **Site de soumission** (`runtime.rs:1044-1054`) : quand
     `task_entry.task.verifiable == true`, construire les params avec
     `temperature=Some(0.0)` + `seed=Some(SEED_DETERMINISTE)` (constante
     worker-wide, ou derive deterministe du `task_id`). Sinon, comportement
     actuel inchange (sampling best-effort).
  3. **Ollama path** (`ollama.rs:171-180`, `req_build`) : quand
     `params.temperature` est `Some(t)` ou `params.seed` est `Some(s)`,
     construire `GenerationOptions::default().temperature(t).seed(s as i32)`
     (et optionnellement `.top_k(...)`/`.top_p(...)` pour durcir) et
     `req = req.options(opts)`. **C'est le vrai fix B-2.**
  4. **llama_cpp path** (`llama_cpp.rs:325`) : `temp = temperature
     .unwrap_or(0.7)` → laisser tel quel (greedy deja terminal) ; le seed
     est inerte (greedy ne tire pas). Optionnellement documenter que
     `temperature=0.0` est pose pour coherence/lisibilite. Pas de
     changement de sampler requis.
  5. **`Task`** (`task.rs`) : ajouter `verifiable: bool` **signe**
     (`#[serde(default)]`, NE PAS retirer de `task_canonical_bytes`),
     builder `with_verifiable()`, comme `is_open_source`.
  6. **`validator.rs`** : inchange logiquement (plan confirme) ;
     ajouter un doc-comment explicitant que le quorum compare l'egalite
     brute de `result_text` (colonne nommee `sha256` par heritage S55
     build-task) et que cette egalite n'est exacte QUE si les workers ont
     tourne `verifiable=true` (greedy+seed). Optionnel : `debug_assert`.
  7. **D8 nettoyage** : `redundancy.rs` **retrait pur** (mort confirme
     S55, 0 appelant — supprimer le fichier + `pub mod redundancy`
     `lib.rs:32`) ; `execute_build` **DEPRECATED + ROADMAP_COMMITMENTS**
     (appelant LT-7/S75 nommable) OU cablage si le routage `task_type=
     "build"` existe deja (decision a la lecture du dispatcher en B) ;
     `process.rs:24` `PROVIDERS` → **documenter** la distinction (provider
     d'adaptation-prompt {claude,codex,gpt,local,human} vs backend
     d'execution {Ollama,llama_cpp}) dans PATTERNS — ce sont **deux axes
     orthogonaux**, pas une redondance, **ne PAS unifier**.

- **File/test delta vs plan d'origine** :
  - **AJOUT vs plan** : `crates/nexus-worker-core/src/llm/mod.rs`
    (champ `seed` + builder) et `crates/nexus-worker-core/src/llm/ollama.rs`
    (cablage `GenerationOptions`) — le plan ne les listait pas dans B.2
    (il pointait `runtime.rs` generiquement). Ces deux fichiers sont le
    coeur reel du fix B-2.
  - **CONFIRME conforme plan** : `task.rs` (champ verifiable), `runtime.rs`
    (site soumission), `validator.rs` (doc), `redundancy.rs` (retrait),
    `build_executor.rs` (D8), `process.rs` (doc), `PATTERNS.md`.
  - **Tests** : le plan B.3 reste valide. `verifiable_task_uses_greedy_seed`
    doit asserter que sur le chemin Ollama les `GenerationOptions` portent
    `temperature=0` + `seed` (pas seulement "le backend est greedy"). Sur
    llama_cpp, asserter le determinisme par double-appel meme-sortie
    (stub/mock deterministe vu que le vrai GGUF n'est pas en CI).
  - **Commit body** doit documenter : "Plan proposait X (forcer greedy au
    site de soumission, suggerant un traitement uniforme), preflight S1a a
    identifie Y (llama_cpp deja greedy via greedy() terminal ; Ollama ne
    passait NI temperature NI seed — vrai gap ; ollama-rs 0.2.6
    GenerationOptions existe), adapte a Z (champ seed sur GenerateParams +
    cablage GenerationOptions.temperature(0).seed() sur le path Ollama ;
    llama_cpp inchange)."

---

## Risks And Scope Cuts

- **Blocking risks** : aucun. (Pas de S1b/S2/S3/S4 bloquant → pas de
  DESIGN-CONFLICT.)
- **Non-blocking risks / carry** :
  - **R1 (D2 ⚠️)** : greedy non bit-exact cross-GPU. Mitigation : test B-3
    same-machine ; limite documentee PATTERNS ; cross-GPU → **S75** (scope
    cut #11). Confirme par preuve OSS (issue #5321 + consensus float-GPU).
  - **R2** : E2E flaky si Ollama requis. Le test determinisme cote Ollama
    requiert un runtime reel pour le bit-exact ; mitiger par gate Ollama
    (skip propre, cf. Phase A) + un test unitaire sur la **construction des
    options** (assert `temperature=0`+`seed` poses) qui ne requiert PAS de
    runtime. Le determinisme bout-en-bout reel reste gate Ollama.
  - **R7 (D8 ⚠️)** : retrait `execute_build`/`RedundancyDispatcher`.
    `RedundancyDispatcher` = retrait pur (0 appelant, reversion S55
    confirmee). `execute_build` = DEPRECATED + ROADMAP (appelant LT-7/S75
    nommable) ou cablage — decision a la lecture du routage `task_type` en B.
  - **Colonne `sha256` misnomer** : la colonne `task_results.sha256`
    stocke `result_text` brut (heritage build-task S55). Non-bloquant ;
    a documenter PATTERNS (ne PAS renommer pre-tag sans necessite — c'est
    du wire DB local, edition libre mais cosmetique non prioritaire).
  - **Tests "legacy decode"** a auditer apres l'ajout du champ signe
    `verifiable` (forme JSON v1 redefinie) — supprimer les zombies
    (pre-launch policy).
- **Scope cuts encore honores** (cite kickoff §8 / plan §12) :
  - #11 quorum redundancy>1 cross-MACHINE reel → S75 (B-3 same-machine ici).
  - #13 logprobs/watermark verification → V2 (greedy seed seul ici).
  - #1 ProviderRouter → S72 (pas de router multi-provider ici).
  - L'auditeur grep : aucune ligne S71 Phase B ne touche un scope cut.

---

## Action

- **PLAN-ADAPT** : proceder avec l'approche corrigee ci-dessus
  (Plan Adaptation). Le commit body de Phase B DOIT citer ce fichier
  (`sprint71_phase_b_preflight.md`) et documenter le delta
  "plan proposait X / S1a a identifie Y / adapte a Z".
- Points fermes par ce preflight :
  - **S1a** : llama_cpp deja deterministe (greedy terminal) ; Ollama ne
    passe ni temperature ni seed → vrai gap ; **temperature=0 SEULE NE
    SUFFIT PAS** (besoin seed fixe plumbe pour le path Ollama, preuve
    issue #5321 + ollama-rs 0.2.6 GenerationOptions).
  - **S1b** : portable-pty 0.9.0 / async-stream 0.3.6 / futures 0.3.32 /
    ollama-rs 0.2.6 — aucune CVE critique/high sur crypto/wire/network.
    G13 satisfait.
  - **S2** : exclusion canonical S23 (`34c77ce`, precedent valide) ;
    quorum DB S55 (`0cb576d`, supersede RedundancyDispatcher) ;
    RedundancyDispatcher reversion CONFIRMEE (0 appelant) ; execute_build
    reversion ambigue (LT-7) ; logprobs V2 deferral PO. Clean.
  - **S3** : greedy+seed n'ouvre pas de surface (l'attaquant doit faire le
    travail honnete pour passer le quorum) ; rejet outliers preserve ;
    limite cross-GPU carry S75. Clean.
  - **S4** : `verifiable` = champ **SIGNE** (identite, pas dispatch ;
    inverse de `redundancy_factor`), pattern `is_open_source` ;
    `TASK_FORMAT_VERSION` reste 1 ; `#[serde(default)]` runtime tolerance ;
    `seed` sur GenerateParams est hors-wire. Day 0 preserved. Clean.
