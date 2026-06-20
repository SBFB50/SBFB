# Sprint 76 — Phase F Review

## Verdict: PASS

(Promu de PASS-PENDING -> PASS apres reconciliation Codex CLEAN — cf.
section « Codex reconciliation » en fin de fichier.)

Phase F (« Quantization 4-bit documentee », D5) est doc-only, suites
vertes, scope D5 integralement tenu, doc honnete et sourcee. Verdict
**PASS-PENDING** : aucun P0/P1, mais le gate Codex (`codex exec`) n'a
pas encore tourne — il reste bloquant avant le commit. Apres un Codex
CLEAN/CONFIRME, la phase passe PASS. 2 findings P2 (honnetete doc) a
documenter dans le body, aucun a corriger pour debloquer le commit.

## Perimetre reviewe (4 fichiers, doc-only)

Diff vs HEAD `24bda54` = 5 entrees working-tree (4 fichiers livrables
+ 1 preflight untracked) :

- **NOUVEAU** `docs/operators/QUANTIZATION.md` (livrable 1+2 : reco GGUF,
  table empreintes VRAM, cible <=14B single-GPU, 70B=sharding S77,
  pre-condition quorum meme-GGUF, design-note caps integree §5, CVE §8).
  8778 octets.
- **M** `web/src/components/GpuConsentDialog.tsx` (+10 lignes) : un
  `<p data-testid="consent-quantization-hint">` texte non-cliquable
  (livrable 3, Option B).
- **M** `web/src/components/__tests__/GpuConsentDialog.test.tsx`
  (+11 lignes) : +1 `it()`.
- **NOUVEAU** `crates/nexus-worker-core/tests/quantization_doc.rs` :
  5 `#[test]`.
- **untracked** `.planning/active/sprint76_phase_f_preflight.md`
  (a stager au commit, precedent Phase E).

`git diff HEAD --stat` = 21 insertions (front uniquement) ; les 3
nouveaux fichiers sont untracked. Aucun `.rs` de `src/` touche, aucune
struct wire/canonical modifiee.

## D1 — Diff complet ligne par ligne

**Verdict: PASS.** Diff propre et entierement intentionnel. Aucun
fichier parasite, aucun `console.log`/TODO/FIXME, aucun `<a href>`
relatif (Option B confirmee, 0 risque 404 SPA). Le JSX ajoute est
valide : `<p>` ferme, `className="text-xs text-white/40"` aligne sur le
style attenue existant, insere exactement entre le CapSlider VRAM
(`consent-cap-vram`, GpuConsentDialog.tsx:336) et le CapSlider Heures
(`consent-cap-hours`, :356). Resolution de chemin des tests Rust
correcte : `CARGO_MANIFEST_DIR` (crates/nexus-worker-core) `.parent()`
`.parent()` -> racine workspace -> `docs/operators/QUANTIZATION.md`
(quantization_doc.rs:20-26) ; lecture `src/llm/llama_cpp.rs` relative
au crate (l.34-37, correct, le fichier appartient au crate).

- INFO — Hint JSX bien forme et conforme au style muet existant
  (GpuConsentDialog.tsx:338-347). Texte FR accentue correct
  (jusqu'a / tres gros / relevent / a venir / Detail).
- INFO — Les 5 assertions Rust matchent le contenu reel (grep) : doc
  8778o > seuil 1024 ; tokens presents Q4_K_M(15) / IQ4_XS(5) /
  Q2_K(6) / 14B(6) / 70B(7) / S77(6) / sharding(3) / quorum(5) /
  GGUF(16) / exact-match(2). `llama_cpp.rs:150` cable bien
  `with_n_gpu_layers` (unique), 0 `with_split_mode`/`with_devices`.
- P3 — Nit §3 ligne 55 : « 32B Q4_K_M ~22 Go (carte 24 Go hors-cible) »
  melange deux idees (deborde de 16 Go / 24 Go hors-scope). Lisible
  mais legerement ambigu. Editorial, optionnel.

## D2 — Scope cuts semantiques + invariant anti-scope-creep

**Verdict: PASS.** Phase VERITABLEMENT doc-only, verifie au niveau du
CODE. Invariant cardinal vert : grep
`with_split_mode|with_devices|with_main_gpu|split_mode|with_tensor_split|with_n_gpu_layers`
sur `llama_cpp.rs` = 1 seul hit (l.150 `with_n_gpu_layers`).
`llama_cpp.rs` ABSENT du diff. Le test 5 `llama_cpp_unchanged_doc_only`
(quantization_doc.rs:97-109) est un garde bidirectionnel : PRESENCE
`with_n_gpu_layers` + ABSENCE des 2 forbidden — il virerait au rouge si
le tensor-split S77 etait cable.

- INFO — Diff = 5 entrees, 0 code runtime de quantification. Le test
  `tests/` lit des fichiers en texte (`std::fs::read_to_string`), aucun
  import backend, independant de la feature `llm_llama_cpp`.
- INFO — Les 5 D5 figees respectees PAR la doc : tensor-split rejete
  fermement (QUANTIZATION.md:82-84 hors-cible) ; mono-machine 2-GPU
  enterre (:79-80) ; 70B=S77 sharding (:56/:73/:85-87) ; Q2_K pas
  defaut (:36/:40 « dernier recours ») ; AWQ/GPTQ/EXL2/bitsandbytes 0
  mention (GGUF-only). 0 bump wire / 0 migration / 0 dep.
- INFO — Aucun struct/canonical/wire touche. Design-note §5
  references exactes (cf. D3).

## D3 — Research grounding + honnetete de la doc

**Verdict: PASS.** Doc solidement sourcee et honnete sur les 5 points
critiques. Tous les chiffres falsifiables matchent le kickoff §4 D5 a
l'unite : 70B bartowski (Q4_K_M 42.52 / Q4_K_S 40.35 / IQ4_XS 37.90 /
Q3_K_M 34.27 / Q2_K 26.38), perplexite arXiv 2601.14277v1 (F16 7.32 /
Q4_K_M 7.56 / Q4_K_S 7.62 / Q4_0 7.74), 14B ~8.5, 8B ~4.9, 32B ~22,
cible single-GPU <=14B. Pre-condition quorum formulee comme
EXACTITUDE/joignabilite (cohorte ADVISORY, renvoi §15.2), zero
sur-promesse anti-Sybil. Honnetete VRAM-live explicite (le cap lit
`task.estimated_vram_mb` DECLARE, pas la taille GGUF reelle ; cablage
VRAM-live = S77). 2 claims CVE NON-APPLICABLE correctes. Refs croisees
verifiees au fichier:ligne reel ; pointeurs perimes signales par le
preflight (runtime.rs:1082, consent.rs 417-432) NON recopies.

- **P2 — Ligne 35 : sur-attribution a l'arXiv d'un « meilleur tradeoff
  4-bit » que ses propres chiffres contredisent.** La cellule Q4_K_S
  (QUANTIZATION.md:35) dit « l'arXiv 2601.14277 le donne comme meilleur
  tradeoff 4-bit (ppl 7.62 vs 7.56) ». (1) Les ppl citees montrent que
  Q4_K_S (7.62) est PIRE que Q4_K_M (7.56) sur l'axe qualite —
  auto-contradictoire, confirme par la table §6 (:128). (2) Le
  fact-sheet kickoff ne contient PAS l'assertion « Q4_K_S = meilleur
  tradeoff », seulement les ppl brutes. Conclusion attribuee au papier
  non presente dans la source. Non bloquant (decision operateur non
  faussee). Fix : retirer « l'arXiv le donne comme meilleur tradeoff »,
  garder « plus petit que Q4_K_M (ppl 7.62 vs 7.56, ecart minime,
  cf. §6) ».
- P3 — Colonnes IQ4_XS/Q2_K pour 7B/8B/14B/32B (table §3, :50-54) sont
  des estimes coherents (ratios ~85% / ~65% du Q4_K_M alignes sur le
  70B mesure) mais non explicitement etiquetes comme extrapoles. Note
  « valeurs ~ extrapolees du ratio mesure sur le 70B » leverait
  l'ambiguite. Editorial, optionnel.

## D4 — Verification semantique des tests

**Verdict: PASS.** Les 5 tests Rust + le test Vitest sont
semantiquement justes ET complets vs plan §F.3. Chaque token greppe est
reellement present dans la doc (50 occurrences cumulees, 0 forbidden).
Feature-independence confirmee : aucun `[[test]] required-features` dans
`Cargo.toml`, le test integration lit les fichiers en texte -> tourne
dans la suite par defaut nextest workspace meme sans `llm_llama_cpp`
(opt-in Cargo.toml:27) — d'ou le +5 dans 1799->1804.

- INFO — Les 5 tests assertent reellement leur contenu (grep confirme).
  `quantization_doc_present` (:41) file exists + len>1024 ;
  `_has_footprint_table` (:55) ; `_states_70b_is_s77` (:68) ;
  `_states_quorum_precondition` (:81) ; `llama_cpp_unchanged_doc_only`
  (:97).
- INFO — `llama_cpp_unchanged_doc_only` = vrai verrou anti-regression
  bidirectionnel (PRESENCE + ABSENCE). Garde anti-scope-creep demande
  par les D5 figees.
- INFO — Test Vitest non-vacuous : `getByTestId("consent-quantization-hint")`
  + 3 `toHaveTextContent` (≤14B, Q4_K_M, docs/operators/QUANTIZATION.md),
  GpuConsentDialog.test.tsx:144-153. Echouerait si le hint etait retire
  ou deviait.
- INFO — Aucun test manquant vs plan §F.3 (5/5, noms identiques).

## D5 — Fidelite PLAN-ADAPT + readiness commit

**Verdict: PASS.** Phase F implemente fidelement le verdict PLAN-ADAPT
et les 5 D5 figees. Option B (texte non-cliquable) correctement code :
`<p>` plain text sous le CapSlider VRAM, sans `<a href>`, sans href
relatif -> 0 risque 404 SPA. FR accentue correct + `scan-en-strings`
non declenche (aucun mot EN_WORDS ; Q4_K_M/70B/path = tokens
techniques). Invariant cardinal tenu. Chiffres factuels + design-note
file:line exacts. Delta tests EXACT : Rust +5 (1799->1804), Vitest +1
(397->398). `http.rs:8528` fmt = faux positif toolchain pre-existant
(http.rs ABSENT du diff Phase F).

- INFO — Option B correcte (GpuConsentDialog.tsx:338-347).
- INFO — Texte FR correct + scan-en-strings non declenche.
- INFO — 5 assertions Rust matchent + invariant llama_cpp tenu.
- INFO — Chiffres factuels + design-note exacts : `vram_budget_remaining_bytes`
  gpu/mod.rs:147 ; CapVram gate consent.rs:422-425 ; `estimated_vram_mb`
  `#[serde(default)]` task.rs:258-261. arXiv 2601.14277v1 ;
  CVE-2026-34159 + CVE-2026-2069 non-applicables (QUANTIZATION.md:165-173).
  Pin llama-cpp-2 0.1.143 documente (:184).
- P3 — `http.rs` fmt drift = env-note correcte (a documenter dans le
  body, pas un fix Phase F).
- P3 — Stager les 3 artefacts planning au commit (precedent Phase E
  `768e235` incluait preflight+review+codex_review). `sprint76_phase_f_preflight.md`
  est untracked.

## Critic de completude (adversarial)

**Verdict: PASS.** Re-verification independante de chaque claim
load-bearing confirmee. Diff = 5 fichiers, 0 parasite, 0 `.rs` de
`src/`. Invariant cardinal verrouille cote source ET test. 5 file:line
design-note exacts. Claim §7 (divergence rejetee outlier par
`validate_quorum_pre_guardrail`) grounded validator.rs:210-219/295-298
+ test `quorum_single_outlier_detected` (:665). Claim §8 CVE exacte :
SBFB cable `llguidance` (Cargo.toml:27, llama_cpp.rs build_matcher
:478) pour le sampling contraint ; GBNF natif touche uniquement par le
chemin Ollama (process separe). Pin llama-cpp-2 0.1.143 matche.

**Le critic a trouve 1 finding neuf que les 5 dimensions ont manque :**

- **P2 — Ligne 34 : claim IQ4_XS « a perplexite quasi identique » a
  Q4_K_M entierement non-sourcee.** La seule table de perplexite citee
  (§6 arXiv 2601.14277, QUANTIZATION.md:124-129) ne contient AUCUNE
  ligne IQ4_XS (verifie : IQ4_XS jamais dans une table ppl). Pendant
  exact, sur l'axe qualite IQ4_XS, du P2 que D3 a trouve sur Q4_K_S
  (ligne 35), mais non releve par D3. La claim de TAILLE de la meme
  ligne (« Gagne ~10% ») est correcte (70B 42.52->37.90 = -10.9%). Seul
  le segment perplexite est non-etaye. Non bloquant (claim communement
  vraie, ordre de grandeur juste). Fix : reformuler sans claim ppl
  chiffree non-sourcee, ou ajouter une ligne IQ4_XS a la table §6 si
  source.

- P3 — §5 ne dit pas que le cap VRAM est INERTE quand
  `estimated_vram_mb=0` (default serde). task.rs:253-254 dit « Zero
  means unknown — cap stays inert » ; le gate consent.rs:422-425 ne
  rejette que si `> max_v`. « le contributeur garde la main via le
  cap » est partiellement optimiste (app declarant 0 contourne le cap
  VRAM ; heures/watts restent ; VRAM-live = S77). Editorial, optionnel.

## Suites §7.4 (statut verifie main thread — NON re-execute)

- Rust nextest workspace = **1804 passed / 0 skipped** (1799->1804, +5
  = les 5 tests `quantization_doc`).
- clippy `--all-targets -D warnings` = clean ; doctests ok ; release
  `nexus-shell-daemon` ok.
- Frontend = **398 Vitest passed** (397->398, +1) ; lint + tsc +
  coverage (87.2 / 79.01 / 85.92 / 88.52) + build + size
  (128.76 kB < 130) + scan-en-strings = verts.
- `cargo fmt --all --check` : SEUL `http.rs:8528` flagge = **faux
  positif** derive toolchain local rustfmt 1.9.0/rustc 1.95 vs
  canonique 1.94 ; `http.rs` NON touche Phase F (committe clean sous
  1.94 en Phase E). Les fichiers Phase F sont fmt-clean sous 1.95. fmt
  canonique = Docker `rust:1.94` differe pre-push (pattern S74/S75).

Verification independante de ce reviewer (cheap, cible, sans relancer
les suites lourdes) : grep tokens doc (10/10 presents), grep invariant
llama_cpp (1 hit l.150, 0 forbidden), `wc -c` doc (8778), file:line
design-note (gpu/mod.rs:147, consent.rs:422-425, task.rs:258-261 OK),
5 `#[test]` presents, JSX hint + Vitest assertions OK, lignes 34/35 +
table §6 confirmees pour les 2 P2.

## Findings (severite / titre / evidence / fix)

| Sev | Titre | Evidence | Fix |
|---|---|---|---|
| P2 | Ligne 34 IQ4_XS « ppl quasi identique » non-sourcee | QUANTIZATION.md:34 vs table §6 :124-129 (0 ligne IQ4_XS) | Reformuler sans claim ppl chiffree, ou ajouter ligne IQ4_XS a §6 si source |
| P2 | Ligne 35 Q4_K_S « meilleur tradeoff 4-bit » sur-attribue a l'arXiv + auto-contradictoire | QUANTIZATION.md:35 vs §6 :128 (7.62 > 7.56) | Retirer « l'arXiv le donne comme meilleur tradeoff » ; garder « ppl 7.62 vs 7.56, ecart minime cf. §6 » |
| P3 | §5 ne dit pas cap VRAM inerte si estimated_vram_mb=0 | QUANTIZATION.md:109-115 vs task.rs:253-254 + consent.rs:422-425 | Demi-phrase « si l'app declare 0 = inconnu, cap inerte ; heures/watts restent ; VRAM-live = S77 » |
| P3 | Colonnes IQ4_XS/Q2_K <70B non etiquetees extrapolees | QUANTIZATION.md:50-54 | Note « valeurs ~ extrapolees du ratio mesure sur le 70B » |
| P3 | Nit §3 cellule 32B ambigue (16 Go / 24 Go) | QUANTIZATION.md:55 | Scinder en deux temps |
| P3 | Stager les 3 artefacts planning au commit (precedent Phase E) | git status untracked sprint76_phase_f_preflight.md ; `768e235 --name-only` | `git add` preflight + review + codex_review avant commit |
| P3 | http.rs fmt drift = env-note body, pas un fix | git diff HEAD (http.rs absent) | Noter sous ## Verification §7.4 (faux positif toolchain 1.95 vs 1.94, Docker 1.94 differe pre-push) |

## Notes pour le commit body (P2)

A documenter (non bloquant, pas de fix requis pour debloquer le commit) :

1. **P2 doc honnetete ligne 34** — IQ4_XS « perplexite quasi identique »
   est une connaissance generale non-sourcee (table §6 = F16/Q4_K_M/
   Q4_K_S/Q4_0 seulement). Claim communement vraie ; reformulation
   future ou ajout d'une source ppl IQ4_XS.
2. **P2 doc honnetete ligne 35** — Q4_K_S « meilleur tradeoff 4-bit »
   attribue a l'arXiv : conclusion non presente dans la source et ses
   ppl la minent (7.62 > 7.56). Reformuler en futur lot doc.
3. **env-note fmt** — `http.rs:8528` flagge par fmt local = faux positif
   toolchain (rustfmt 1.9.0/rustc 1.95 vs canonique 1.94) ; http.rs non
   touche Phase F ; fmt canonique Docker `rust:1.94` differe pre-push.
4. **Docker dual-platform differe recovery pre-push** (pattern S74/S75) ;
   les fichiers Phase F sont platform-agnostiques (doc + web + test
   texte).

Rappel headers body (calque Phase E `768e235`) : titre EXACT
`docs(operators): Sprint 76 Phase F — quantization 4-bit documentee (D5)`
AVEC « Phase F » (sinon hooks lightcheck/Codex non armes) ; section
EXACTE `## Scope cuts` (regex `agentctl.py` strict `^## Scope cuts\s*$`,
PAS « respectes ») listant les rejets D5 (tensor-split / AWQ-GPTQ-EXL2 /
Q2_K-defaut / mono-2GPU / 70B->S77).

## Conclusion

**PASS-PENDING.** Phase F est une livraison doc-only propre : invariant
anti-scope-creep verrouille cote source ET test, 5 D5 figees respectees,
doc sourcee a l'unite vs kickoff §4 D5, design-note file:line exactes,
tests semantiquement justes et complets (5 Rust + 1 Vitest), suites
vertes (1804 Rust / 398 Vitest, verifie main thread). Aucun P0/P1.
2 P2 sont des nuances d'honnetete editoriale dans la doc (lignes 34-35,
claims de perplexite non-sourcees), a documenter dans le body sans
bloquer le commit. Le verdict reste **PASS-PENDING et non PASS** car le
gate Codex (`codex exec`) n'a pas encore tourne — il est bloquant avant
le commit. Apres Codex CLEAN/CONFIRME, ecrire le commit body (calque
Phase E) et stager preflight + review + codex_review avec les 4 fichiers
livrables. Debloque Arc 3.5 -> Phase G wrap-up (6/6).

## Codex reconciliation

**Codex GPT 5.5 (`codex exec`, modele externe cross-model) = 5/5
CONFIRME, 0 GAP, 0 PARTIEL.** Artefact brut :
`.planning/active/sprint76_phase_f_codex_review.md` (output `codex exec
-o` non reecrit). Codex a verifie sur le working tree reel (`git diff` +
lecture) :

- Livrable 1 (`QUANTIZATION.md`) CONFIRME — reco par carte, table
  empreintes 3 formats, cible <=14B, 70B=S77, pre-condition quorum
  formulee comme exactitude (« pas de formulation meme-GGUF=anti-Sybil
  trouvee »). Chiffres coherents (14B 8.5<=16 ; 70B 42.52>16 ; Q2_K
  26.38>16).
- Livrable 2 (design-note caps) CONFIRME — `gpu/mod.rs:147`,
  `consent.rs:422`, `task.rs:258` cites avec les bonnes lignes ; honnete
  sur `estimated_vram_mb` declare + VRAM-live=S77.
- Livrable 3 (pointeur front) CONFIRME — `GpuConsentDialog.tsx:338`,
  texte non-cliquable, `rg "<a|href="` = 0 href relatif (pas de 404).
- Livrable 4 (tests) CONFIRME — 5 Rust + 1 Vitest, assertions utiles ;
  `llama_cpp_unchanged_doc_only` assert ABSENCE split_mode/devices +
  PRESENCE with_n_gpu_layers.
- Invariant CONFIRME — `git diff llama_cpp.rs` vide, `rg
  with_split_mode|with_devices` = 0.

**Traitement des 2 P2 review (honnetete editoriale)** : corriges
EN-PHASE *avant* Codex (donc Codex a audite la doc deja corrigee, d'ou
0 GAP) :
- `QUANTIZATION.md:34` (IQ4_XS) — « perplexite quasi identique »
  non-sourcee -> reformule en propriete generale des i-quants, note
  explicite « non tabule dans le §6 ».
- `QUANTIZATION.md:35` (Q4_K_S) — attribution « meilleur tradeoff »
  retiree -> garde les chiffres (7.62 vs 7.56, ecart dans le bruit).
- Bonus P3 corriges : cap VRAM inerte si `estimated_vram_mb=0` (note
  §5) ; colonnes IQ4_XS/Q2_K <70B etiquetees « extrapolees » (§3).
Apres ces edits doc, les 5 tests Rust `quantization_doc` re-confirmes
verts (tokens grep tous presents).

**Sequence respectee** : review PASS-PENDING -> Codex CONFIRME 5/5 ->
promote PASS. Aucune boucle de correction Codex necessaire (0 GAP). Pret
au commit.
