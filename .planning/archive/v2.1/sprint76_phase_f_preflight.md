# Sprint 76 — Phase F Preflight (G8)

## Verdict: PLAN-ADAPT

Phase F (doc-only, D5 « Quantization 4-bit documentée ») est **faisable et
respecte EXACTEMENT toutes les décisions Day-0 figées** — aucun DESIGN-CONFLICT,
0 wire, 0 dep, 0 migration. Le verdict est **PLAN-ADAPT** (et non EXECUTE) pour
**un seul point concret** : le **Livrable 3 (lien depuis le panneau D1 vers
QUANTIZATION.md)** ne peut PAS être un href relatif — la SPA ne sert aucun
markdown du repo (preuve ci-dessous). Le plan reste valide ; seule l'approche
concrète du lien doit être précisée (lien externe forge OU texte d'orientation
non-cliquable). Aucune Day-0 n'est touchée.

## Contexte (phase F doc-only D5)

Le runtime quantifié 4-bit **EXISTE DÉJÀ** : `LlamaCppBackend.ensure_model`
charge n'importe quel GGUF pré-quantifié via `load_from_file` + `with_n_gpu_layers`
(offload GPU) — le 4-bit est **baked dans le `.gguf`**, il n'existe aucun paramètre
runtime de quantification. Phase F livre donc **de la DOCUMENTATION** :

- **Livrable 1** : `docs/operators/QUANTIZATION.md` (reco format GGUF par taille de
  carte + table d'empreintes VRAM + pré-condition quorum « même GGUF » + cible
  single-GPU honnête ≤14B + gros modèles = sharding cross-machine S77).
- **Livrable 2** : design-note caps VRAM (décrit `vram_budget_remaining_bytes` +
  cap admission `max_vram_mb`, réutilisés **tels quels**, 0 code).
- **Livrable 3** : lien depuis le panneau D1 « offrir ma puissance »
  (`web/src/components/GpuConsentDialog.tsx`) vers QUANTIZATION.md.

Les 5 tests F.3 sont des `test -f` / `grep` purs (stdlib Rust), dont le verrou
anti-scope-creep n°5 `llama_cpp_unchanged_doc_only`.

## S1a OSS / factualité quant (chiffres VRAM, formats GGUF, llama-cpp-2)

Verdict scan : INFO. **Tous les chiffres factuels du plan §9 sont corrects ou
justes en ordre de grandeur** (vérifié WebSearch HuggingFace/bartowski 2026 +
Context7 `/utilityai/llama-cpp-rs` + lecture source). Aucun n'est faux au point de
casser la doc.

- **Formats GGUF** : Q4_K_M = défaut défendable confirmé (« recommended for most
  users / production, best quality-to-size balance ») ; IQ4_XS plus serré que
  Q4_K_M à perplexité quasi-identique (i-quant) ; Q4_K_S plus petit ; Q2_K très
  compressé/dégradé. Tous cohérents avec le plan.
- **Table VRAM 70B EXACTE** (chiffres bartowski Llama-3.1-70B verbatim) :
  Q4_K_M=42.5 GB / IQ4_XS=37.9 GB / Q2_K=26.4 GB (`sprint76_plan.md:567,581`). La
  conclusion « ne tient sur AUCUNE carte 16GB » est **JUSTE** (la plus petite quant
  Q2_K=26.4 GB dépasse déjà 16 GB).
- **14B Q4_K_M ~8.5-9 GB** (bartowski Qwen2.5-14B), tient 1×16GB = OK (cible
  honnête single-GPU). Note optionnelle non-bloquante : ~8.5 GB est la taille
  FICHIER ; +1-2 GB de KV-cache au runtime (déjà géré par les caps existants).
- **7B/8B Q4_K_M ~4.6 GB** = midpoint correct (8B réel = 4.92 GB, 7B ~4.37 GB ;
  les deux ≪ 16 GB). **32B Q4_K_M ~22 GB** plausible (~21 GB mesuré, carte 24GB
  hors-cible). Ordres de grandeur justes.
- **Offload CPU 70B 2-5 tok/s** = plausible (CPU-only 2.5-6 tok/s selon bande
  passante RAM ; hybride 1-3 tok/s).
- **llama-cpp-2** : exemple canonique Context7 = `LlamaModelParams::default()
  .with_n_gpu_layers(...)` + `load_from_file` — **IDENTIQUE à `llama_cpp.rs:149-159`**.
  Aucun paramètre runtime de quant ; on charge un `*-Q4_K_M.gguf` tel quel. D5
  « DOC-ONLY, runtime quant déjà présent, 4-bit baked » = techniquement EXACT.
- **API multi-GPU S77 réelle et SÉPARÉE** : `with_devices` + `with_tensor_split` +
  `split_mode` constituent la vraie API tensor-split, correctement exclue vers S77.
- **Pré-condition quorum « même GGUF »** techniquement fondée : quants/builds
  différents → logits/tokens divergents → exact-match impossible. Cohérent D3
  (RuntimeTuple cohorte homogène, Phase C) + PATTERNS §P60.2.

## S1b deps (0 dep attendu)

Verdict scan : INFO. **0 nouvelle dep, ni Cargo.lock ni package.json touchés.**

- `llama-cpp-2 = "0.1.143"` est **DÉJÀ** dans `[workspace.dependencies]`
  (`Cargo.toml:362`), optionnelle derrière la feature `llm_llama_cpp`
  (`nexus-worker-core/Cargo.toml:27,142`). Phase F doc-only n'introduit ni
  n'upgrade aucune dep.
- Créer `docs/operators/QUANTIZATION.md` (dossier inexistant → création pure,
  0 impact build) + lien front (texte/href, 0 import npm) + 5 tests grep/test-f
  (stdlib `std::fs`/`std::path`) → **ni Cargo.lock ni package(-lock).json touchés**.
- Les 5 tests vivent dans `nexus-worker-core` (propriétaire de `llama_cpp.rs` +
  des caps). Pas de module doc-presence générique réutilisable, MAIS le pattern
  d'accès fichier checké-in est déjà établi 2× dans le crate :
  `env!("CARGO_MANIFEST_DIR")` + join (`engine/runtime.rs:2283`,
  `nexus-core-rs/.../task_response.rs:282`). Pour atteindre la racine workspace
  `docs/operators/QUANTIZATION.md` depuis `crates/nexus-worker-core`, ajouter
  `.parent().parent()` (crates/ puis racine) avant `.join("docs/operators/...")`.
  Emplacement : `#[cfg(test)] mod` inline OU nouveau `tests/quantization_doc.rs`
  (le crate n'a aucun fichier `tests/` aujourd'hui — neutre, 0 dep dev).

## S2 décisions historiques D5 (figées respectées ?)

Verdict scan : EXECUTE. **Les 6 ancres Day-0 sont TOUTES honorées par plan §9 +
kickoff §4 — 0 contradiction, 0 scope creep runtime.**

1. **doc-only** — plan F.1 (`sprint76_plan.md:552`) « Livrer de la DOCUMENTATION,
   pas un nouveau runtime de quantification ».
2. **tensor-split mono-machine REJETÉ FERMEMENT** — plan F.1 (`:573-575`) + kickoff
   §4 (`:636-644` « REJETÉ FERMEMENT … NON ré-évaluable »).
3. **mono-machine 2-GPU ENTERRÉ** — kickoff Résolution PO (`:955` « personne n'a
   2 GPU »).
4. **AWQ/GPTQ/EXL2/bitsandbytes rejetés** — kickoff §4 (`:619-631`, gap factuel
   in-process Rust).
5. **Q2_K-défaut rejeté** — kickoff §4 (`:632-635`).
6. **70B = sharding cross-machine S77** — plan F.4 grep S77 (`:604`) + kickoff
   (`:604-605`).

- **Frontière D3/D5 nette** : `model_digest = blake3(NOM)` est une affaire **D3
  déjà tranchée en Phase C** (`1cc28e7`) comme doc-note S77 avec test de verrou
  `model_digest_is_name_hash_doc_note_s77()` (`engine/runtime.rs:1860`). Phase F
  **n'y touche PAS** ; elle énonce seulement en prose la pré-condition quorum
  « même GGUF » — cohérent avec le name-hash maintenu. Aucun chevauchement.
- **Aucun commit antérieur ne contredit** le plan (seul `c85397b` S20-D existe,
  hors-sujet tensor-split).
- **P3 hygiène doc** : 3 pointeurs fichier:ligne du kickoff/plan ont dérivé —
  voir Notes pour le code (citer les lignes RÉELLES vérifiées dans la doc).

## S3 threat model (obligation Phase F vs Phase G)

Verdict scan : INFO (PASS). **Phase F n'a AUCUNE obligation threat-model
bloquante.**

- La surface menace concernée (mismatch GGUF/quant, quorum cross-machine, cohorte
  homogène) est **déjà entièrement couverte par THREAT_MODEL §15.2** (`:895-926`,
  ajoutée S76 Phase D) : row « Worker menteur » (`:916`, rés L) + row « Divergence
  cross-GPU lue comme un bug » (`:917`, rés M). La cohorte homogène y est qualifiée
  de routage **ADVISORY, JAMAIS frontière de confiance** (`:903-905`).
- Le plan §10 (`:635-636,649`) confie la **consolidation THREAT_MODEL à Phase G**
  (wrap-up), pas à Phase F.
- **Q2 — le mismatch de quant est une CONDITION DE CORRECTION, PAS un vecteur
  d'attaque** : un worker au GGUF/quant différent produit un `result_text`
  divergent rejeté comme outlier par l'exact-match (`validate_quorum_pre_guardrail`,
  INCHANGÉ). Sans même GGUF, le quorum **ne se forme pas** (disponibilité dégradée,
  pas compromission). Un attaquant qui mismatch **s'auto-exclut** de la majorité.
  La doc DOIT énoncer le « même GGUF » comme **exactitude/joignabilité du quorum**,
  avec renvoi 1-ligne vers §15.2 (cohorte advisory, défense = exact-match), et
  **JAMAIS comme barrière anti-Sybil** (sur-promesse contredite par §15.2).
- **Q3 — documenter empreintes VRAM/tailles modèles ne crée AUCUNE fuite** :
  données publiques d'ingénierie GGUF (llama.cpp / model cards HF) ; les caps sont
  une logique d'admission worker-side défensive (`consent.rs:417-432`) ; le budget
  VRAM live (`gpu/mod.rs:147`) n'est PAS exporté sur le wire par Phase F.

## S4 wire format (0 bump)

Verdict scan : INFO. **0 bump wire (confirmé improbable).**

- Aucune constante `*_VERSION`/`FORMAT_VERSION`/`ANNOUNCEMENT_VERSION`/`DOMAIN_*`
  n'est mutée (toutes =1 sauf `INVITE_FORMAT_VERSION=2` ; `canonical.rs:74-239`
  DOMAIN_*_V1 inchangés ; `task.rs:61` TASK_FORMAT_VERSION=1). Phase F = doc + lien
  front → 0 constante touchée.
- **Hook lightcheck Check 4 (wire-marker) = strictement WARN, jamais BLOCK** :
  il matche les **NOMS de fichiers stagés** contre `canonical\.rs|schemas/|_VERSION`
  (`phase-precommit-lightcheck.sh:208`), pas le contenu, puis `WARNINGS++` + exit 0
  (`:474-486`, exit 2 ssi `ERRORS>0`). `docs/operators/QUANTIZATION.md` et
  `GpuConsentDialog.tsx` ne matchent PAS → **même pas de WARN**. Les mots
  FORMAT/VERSION/GGUF/Q4_K_M **dans le texte** de la doc ne sont lus par aucun check
  (il n'existe AUCUN grep de contenu FORMAT_VERSION dans les hooks ; les « WARN
  bénins » S76-B/C/D venaient de PATHS de fichiers).
- Livrables 2/3 ne touchent aucun canonical/serde struct : caps réutilisés tels
  quels (accesseurs/gates runtime), lien front = JSX statique → `ConsentSnapshot`/
  `caps` wire reste **byte-identique**.

## S5 code-refs + livrable 3 lien front (cible exacte)

Verdict scan : **PLAN-ADAPT** (le point qui détermine le verdict global).

**Scans 1-3 (références Rust) — CONFIRMÉS, vérifiés en source :**

- **SCAN 1 (invariant n°5)** : `crates/nexus-worker-core/src/llm/llama_cpp.rs:143-164`
  (`ensure_model`). Seule construction de `LlamaModelParams` = l.149-150 :
  `LlamaModelParams::default().with_n_gpu_layers(...)`. **Grep whole-crate
  `with_split_mode|with_devices|split_mode|with_tensor_split|with_main_gpu` = 0 hit**
  (vérifié indépendamment ce préflight). Le test `llama_cpp_unchanged_doc_only`
  sera **VERT à l'écriture** et ROUGE si le tensor-split S77 était ajouté.
- **SCAN 2** : `crates/nexus-worker-core/src/gpu/mod.rs:147` —
  `pub fn vram_budget_remaining_bytes(&self, max_vram_fraction: f32) -> u64`
  (corps l.147-151 : `allowed = total*fraction`, `saturating_sub(used)`).
  Réutilisable tel quel pour la design-note Livrable 2.
- **SCAN 3** : gate cap VRAM = `crates/nexus-worker-core/src/consent.rs:422-425`
  (`if task.estimated_vram_mb > max_v → Reject(CapVram)`). **CRUCIAL** :
  `task.estimated_vram_mb` (`nexus-core-rs/src/task.rs:258-261`, `#[serde(default)]`,
  « Filled by the app submitting ») est l'**estimé DÉCLARÉ par l'app**, PAS la
  taille GGUF réelle → justifie pile la design-note « câblage VRAM-live = S77 ».

**SCAN 4 — Livrable 3 (POINT CRITIQUE, P1) :** Le panneau D1 « offrir ma
puissance » = `web/src/components/GpuConsentDialog.tsx` (marqueurs « Sprint 76
Phase A (D1) » l.114-117 ; slider « VRAM max (GB) » l.328-337 ; section « Limites
strictes » l.310-348). Point d'insertion naturel = juste sous le CapSlider VRAM
(après l.337). **PROBLÈME** : `docs/operators/QUANTIZATION.md` est un fichier REPO,
pas un asset servi au runtime :

- (a) `docs/operators/` **n'existe pas** (Glob = aucun fichier ; à créer).
- (b) le shell-daemon n'a **AUCUNE route de service de markdown** (`.md`).
- (c) `web/public/` ne contient **aucun doc** (favicon.svg, icons.svg, models/,
  sbfb-bridge.js seulement — Glob vérifié) → un href relatif `/docs/operators/...`
  renverrait **404**.

→ Un href RELATIF est **infaisable**. Le seul mécanisme viable = **lien EXTERNE**
calqué sur le pattern `repo_url` existant : `<a href={url} target="_blank"
rel="noopener noreferrer">` (`Browse.tsx:418-426`, idem `VerificationDetail.tsx`,
`BrowsedProject.tsx`). **P2** : le front n'a AUCUNE constante URL-source canonique
(grep `github.com/|codeberg|VITE_|repository` dans `web/src` = 0 réf applicative).
Remotes git : `origin https://github.com/SBFB50/SBFB.git` + `codeberg
https://codeberg.org/SBFB/SBFB.git`. CLAUDE.md : « pas encore de déploiement
live ». → micro-décision PO à trancher (voir plan_adapt / Notes pour le code).

## Findings (table sévérité/titre/evidence)

| Sév | Titre | Evidence |
|---|---|---|
| INFO | Table VRAM 70B EXACTE (42.5/37.9/26.4 GB), « ne tient sur aucune 16GB » JUSTE | `sprint76_plan.md:567,581` vs bartowski Llama-3.1-70B |
| INFO | 14B Q4_K_M ~8.5-9 GB cible single-GPU 1×16GB validée | `sprint76_plan.md:566,581` vs bartowski Qwen2.5-14B |
| INFO | 7B/8B ~4.6 GB midpoint OK ; 32B ~22 GB plausible (carte 24GB hors-cible) | `sprint76_plan.md:581` vs bartowski 8B=4.92GB |
| INFO | llama-cpp-2 : load_from_file + with_n_gpu_layers suffisent, quant baked | `llama_cpp.rs:149-159` vs Context7 /utilityai/llama-cpp-rs |
| INFO | Offload CPU 70B 2-5 tok/s = ordre de grandeur plausible | `sprint76_plan.md:569-570` |
| INFO | `llama-cpp-2 = "0.1.143"` déjà dans workspace, 0 dep nouvelle | `Cargo.toml:362` ; `nexus-worker-core/Cargo.toml:27,142` |
| INFO | Ni Cargo.lock ni package.json touchés (doc + lien + tests stdlib) | Glob docs/operators=∅ ; `GpuConsentDialog.tsx` (0 import) |
| INFO | Pattern test fichier checké-in déjà établi (CARGO_MANIFEST_DIR ×2) | `engine/runtime.rs:2283` ; `task_response.rs:282` |
| INFO | 6 ancres Day-0 D5 toutes honorées par plan §9 + kickoff §4 | `sprint76_plan.md:552,573-575,604` ; kickoff:619-644,955 |
| INFO | Invariant code : llama_cpp.rs câble UNIQUEMENT with_n_gpu_layers, grep=0 | `llama_cpp.rs:149-150` (grep split_mode/devices=0 ce préflight) |
| INFO | Frontière D3/D5 nette : model_digest=blake3(NOM) tranché Phase C 1cc28e7 | `engine/runtime.rs:1128,1860` ; kickoff:679-684,951-952 |
| INFO | Pré-condition quorum « même GGUF » = exactitude (déjà couvert §15.2) | THREAT_MODEL §15.2 `:903-905,916-917` |
| INFO | 0 bump wire ; toutes constantes =1 sauf INVITE=2 | `canonical.rs:74-239` ; `task.rs:61` |
| INFO | Check 4 wire-marker = WARN-only sur PATHS ; F ne matche même pas | `phase-precommit-lightcheck.sh:208,474-486` |
| INFO | Caps VRAM réutilisables tels quels ; estimated_vram_mb = estimé déclaré | `gpu/mod.rs:147-151` ; `consent.rs:422-425` ; `task.rs:258-261` |
| **P1** | **Livrable 3 : la SPA ne sert PAS QUANTIZATION.md → href relatif=404, doit être lien externe forge** | `GpuConsentDialog.tsx:114-117,328-337` ; shell-daemon 0 route .md ; `web/public/` sans docs ; pattern externe `Browse.tsx:418-426` |
| **P2** | **Aucune constante URL-source canonique dans le front → choix forge = micro-décision PO** | grep `web/src` github/VITE/repository=0 ; remotes github.com/SBFB50/SBFB + codeberg.org/SBFB/SBFB |
| P3 | 3 pointeurs fichier:ligne du kickoff/plan dérivés — citer les lignes réelles dans la doc | runtime.rs:1082→1128 ; config.rs racine pas llm/ ; cap VRAM=consent.rs:422-425 |

## Notes pour le code (instructions concrètes)

1. **Livrable 1** : créer `docs/operators/QUANTIZATION.md` (`mkdir docs/operators`).
   Inclure : table d'empreintes (Q4_K_M / IQ4_XS / Q2_K) avec 7B/8B ~4.6 GB,
   14B Q4_K_M ~8.5 GB (cible honnête single-GPU 1×16GB), 32B ~22 GB (carte 24GB
   hors-cible), 70B Q2_K=26.4/IQ4_XS=37.9/Q4_K_M=42.5 GB. Énoncer explicitement
   « 70B = sharding cross-machine 2+ machines × 1 GPU = S77 ; ne tient sur aucune
   carte 16GB ; mono-machine 2-GPU n'est pas une cible » (test n°3). Offload CPU
   2-5 tok/s = palliatif. Pré-condition quorum « même GGUF » (test n°4).
   Optionnel honnêteté : noter que ~8.5 GB = taille FICHIER, +1-2 GB KV-cache au
   runtime (déjà borné par les caps existants).
2. **Pré-condition quorum** : la formuler comme **EXACTITUDE / joignabilité du
   quorum** (« sans même GGUF, l'exact-match ne se forme pas »), avec renvoi 1-ligne
   vers THREAT_MODEL §15.2 (cohorte = ADVISORY, défense réelle = exact-match).
   **NE PAS** la présenter comme une barrière anti-Sybil (sur-promesse contredite
   par §15.2).
3. **Livrable 2 (design-note caps VRAM)** : citer les lignes RÉELLES vérifiées —
   `vram_budget_remaining_bytes` = `crates/nexus-worker-core/src/gpu/mod.rs:147`
   (corps 147-151) ; gate admission = `crates/nexus-worker-core/src/consent.rs:422-425`
   (`RejectReason::CapVram`). Honnêteté : le cap lit `task.estimated_vram_mb`
   (`nexus-core-rs/src/task.rs:258-261`, `#[serde(default)]`, estimé DÉCLARÉ par
   l'app), **PAS la taille GGUF réelle** → câblage VRAM-live = S77 (design-note,
   hors scope bloquant). NE modifier AUCUNE struct (`ConsentCaps` intacte).
4. **Livrable 3 — APPROCHE CORRIGÉE (PLAN-ADAPT)** : insérer le lien dans
   `web/src/components/GpuConsentDialog.tsx`, section « Limites strictes », juste
   sous le CapSlider VRAM (après l.337). Texte FR : « ta carte 16 Go → modèles
   ≤14B Q4_K_M ». Le lien DOIT être **EXTERNE** (`<a target="_blank"
   rel="noopener noreferrer">`, pattern `Browse.tsx:418-426`), JAMAIS un href
   relatif (404 garanti : pas de route .md, pas de doc dans `web/public/`).
   **Choix PO à trancher (P2)** :
   - Option A (recommandée) : `<a>` externe vers la forge publique, p.ex.
     `https://codeberg.org/SBFB/SBFB/src/branch/master/docs/operators/QUANTIZATION.md`
     (ou github.com/SBFB50/SBFB). Cohérent avec le pattern repo_url existant.
   - Option B (si PO refuse de hardcoder une URL vu « pas encore de go-live ») :
     **texte d'orientation non-cliquable** « ta carte 16 Go → modèles ≤14B Q4_K_M
     (voir docs/operators/QUANTIZATION.md) ». Satisfait l'intention produit sans
     dépendance à une URL externe instable.
   - Demander l'arbitrage PO avant de coder le lien (forge canonique + cliquable
     ou non). En attendant, Option B est le défaut le plus sûr (0 URL fragile).
5. **Invariant n°5 (test verrou)** : NE PAS toucher `llama_cpp.rs` ni `config.rs`.
   Le test `llama_cpp_unchanged_doc_only` grep `with_split_mode|with_devices` sur
   `crates/nexus-worker-core/src/llm/llama_cpp.rs` et assert l'ABSENCE — déjà vert
   (grep=0 vérifié ce préflight).
6. **Emplacement des 5 tests Rust** : `#[cfg(test)] mod` inline dans `llama_cpp.rs`
   OU nouveau `crates/nexus-worker-core/tests/quantization_doc.rs`. Pour atteindre
   `docs/operators/QUANTIZATION.md` à la racine workspace depuis le crate :
   `Path::new(env!("CARGO_MANIFEST_DIR")).parent().parent().join("docs/operators/QUANTIZATION.md")`
   (pattern existant `engine/runtime.rs:2283` / `task_response.rs:282`). Stdlib
   uniquement, 0 dep dev.
7. **P3 — pointeurs périmés à NE PAS recopier** depuis kickoff/plan dans la doc :
   `runtime.rs:1082` (réel `engine/runtime.rs:1128`) ; `config.rs:331-356` cité
   comme `llm/config.rs` (réel = `crates/nexus-worker-core/src/config.rs`, racine
   du crate, pas le sous-module llm/) ; `consent.rs:417-432` (le cap VRAM précis
   est `422-425`). Citer les lignes réelles vérifiées.
8. **Commit/threat-model** : Phase F ne touche PAS THREAT_MODEL (consolidation =
   Phase G, plan §10). Body 9 sections (plan F.5). Titre `docs(operators): Sprint
   76 Phase F — ...` AVEC « Phase F » (sinon hooks Codex non armés). Section body
   EXACTE `## Scope cuts` (pas `## Scope cuts respectes`).

## Conclusion

**PLAN-ADAPT.** Phase F est faisable et **respecte EXACTEMENT les 5 décisions D5
figées** (doc-only ; tensor-split rejeté fermement ; mono-machine 2-GPU enterré ;
AWQ/GPTQ/EXL2/bitsandbytes rejetés ; Q2_K-défaut rejeté ; 70B=sharding cross-machine
S77) — **aucun DESIGN-CONFLICT**. Tous les chiffres factuels du plan sont corrects
(S1a), 0 dep (S1b), 0 bump wire (S4), 0 obligation threat-model (couvert §15.2,
consolidation Phase G — S3). L'invariant code tient déjà (S2/S5 : `llama_cpp.rs:150`
câble UNIQUEMENT `with_n_gpu_layers`, grep split_mode/devices = 0). Les caps VRAM
sont réutilisables tels quels (`gpu/mod.rs:147`, `consent.rs:422-425`) et la
design-note S77 « câblage VRAM-live » est honnête (`estimated_vram_mb` = estimé
déclaré, `task.rs:258-261`).

**L'unique adaptation (S5 P1, evidence concrète)** : le Livrable 3 ne peut pas être
un href relatif — la SPA ne sert aucun markdown (pas de route .md, `web/public/`
sans docs). Mécanisme corrigé = **lien externe forge** (`<a target=_blank>`,
pattern `Browse.tsx:418-426`) OU **texte d'orientation non-cliquable** vers
`docs/operators/QUANTIZATION.md` — micro-décision PO (P2) sur la forge canonique
(github.com/SBFB50/SBFB ou codeberg.org/SBFB/SBFB) vs texte non-cliquable.
**Aucune Day-0 n'est touchée.** Le code peut démarrer après arbitrage du mécanisme
du lien (défaut sûr = Option B texte non-cliquable si pas de réponse PO).
