# Sprint 77 Phase G — Review G4 (N0 TOPLOC fingerprint)

> Orchestration **Workflow ultracode** (`wf_ae0f4885-38d`, 6 agents : 5 dimensions
> paralleles + synthese adversariale). Cas B, Phase G.

## Verdict : PASS

(Codex CLEAN, reconcilie — detail en fin de fichier.)

0 finding P0/P1 dans les 5 dimensions apres scrutin adversarial. Les 5 invariants
load-bearing sont confirmes par lecture directe + re-verification independante de la
synthese :

| Invariant | Statut | Evidence |
|---|---|---|
| `scope_respected` | ✅ | rerun/redundancy/validator intouches ; `compare()` expose mais call-sites tous `#[cfg(test)]` |
| `research_grounding_ok` | ✅ | top-k=128, exp/mant 8/7 bits, seuils 38/10/8 named-consts arXiv 2501.16007v2, recalibration Phase K |
| `no_float_ok` | ✅ | `to_bytes` ne serialise que u32 BE + u16 BE ; seul float→int = `bf16_bits` (reinterpret cross-platform) |
| `honesty_caveat_ok` | ✅ | caveat auto-attestation present task.rs/shard_plan.rs/verification.rs/toploc.rs |
| `zero_bump_wire_ok` | ✅ | `*expected==reported_lp` INCHANGE, slots `[u8;32]` reutilises, 0 nouveau `DOMAIN_*`, 0 bump version |

Verification adversariale (synthese) : la branche even-median validee manuellement
(n=4 [1,2,3,8]→median_x2=5 accept ; n=2 [7,9]→16 reject) — correcte, le « non-exercee »
etait un trou de COUVERTURE, pas un bug. DoS cap `n>TOPLOC_TOP_K` precede textuellement
`Vec::with_capacity(n)`. Aucun overflow (mant≤127, count≤128 → sum≤~16K u64).

## Dimensions
- **D1 correctness** : 0 P0/P1. bf16_bits troncature correcte+deterministe ; masks
  0x7F80>>7 / 0x007F corrects ; from_topk cap no-op sur chemin worker (F1 deja k-clampe)
  + sort-by-index canonique ; compare() arithmetique entiere exactement equivalente ;
  roundtrip + cap DoS corrects.
- **D2 tests** : 3 gaps P2 de couverture (corriges, cf. ci-dessous).
- **D3 scope+research** : scope Phase G respecte (0 leak H/I), grounding TOPLOC fidele,
  deviation sketch-vs-poly justifiee (verdict PLAN-ADAPT).
- **D4 securite+wire** : no-float OK, DoS cap OK, 0-bump OK, caveat OK, THREAT_MODEL coherent.
- **D5 livrables+patterns** : 8 livrables presents, named-consts OK, patterns miroir ;
  5 P2 doc-honnetete (corriges).

## P2 corriges EN-PHASE (in-phase, avant commit)

**Doc-honnetete (3 refs perimees + 1 claim en avance + PATTERNS) :**
- `shard_plan.rs:459` (`RunProof::new` doc « reserved-zero ») → « defaults to zero ;
  a worker overwrites it with its commitment ».
- `shard_plan.rs:696` (assertion test « reserved-zero until Phase G » — calendrier FAUX)
  → « RunProof::new defaults the N0 slot to zero (not provided) ».
- `canonical.rs:280` (« reserved N0 TOPLOC slot ») → « N0 TOPLOC slot (a BLAKE3
  commitment) ».
- THREAT_MODEL §16 N0 + §15.2 row I : le claim « chaque worker fingerprinte ET PUBLIE
  dans RunProof » etait EN AVANCE sur le code (grep `RunProof {` worker-core = ZERO).
  Reformule : G livre la PRIMITIVE + le helper worker qui CALCULE le commitment ;
  l'ecriture/emission signee ride le data-plane (H/I/J).
- PATTERNS : amende §P60.3 (« currently [0u8;32] = S77 » → update pointant la primitive
  livree) + ajoute **§P64** (commitment binde ≠ tolerant ; sketch-vs-poly ; no-float
  pre-image ; comparaison entiere ; auto-attestation).

**Couverture tests (3 branches non exercees) :**
- `compare_even_length_median` : branche midpoint pair (`errs[n/2-1]+errs[n/2]`).
- `compare_same_index_exponent_mismatch` : branche `Some(_)`+exposant-divergent (distincte
  du bras index-absent).
- `compare_threshold_boundaries` : frontieres strictes `<` (exp 38, mean 10, median 8) —
  un off-by-one `<`/`<=` part rouge.

## P3 retenus (documentes, non bloquants)
- bf16_bits nomme « truncation » (nuance « cast » corrigee) — ecart ≤1 ULP vs bf16 tensor
  absorbe par les seuils (recalibration Phase K, tracee).
- indices dupliques : non-determinisme latent SANS chemin reel (F1 produit indices uniques
  = positions ; doc-comment declare doublons non-attendus).
- from_topk determinisme au bord du cap depend d'un appelant magnitude-ordonne (in-vivo
  `top_k_by_magnitude` deterministe) — documente comme contrat.
- bf16_bits(NaN) non-canonicalise : risque ~nul (NaN sink en fin de top-k, n_embd≫128).

## Suites (Phase G)
- Windows : clippy `-D warnings` workspace ✅, nextest workspace **1913→1916** (+13 TOPLOC
  +3 boundary, -0), doctests ✅, release ✅.
- Docker canonique Linux (crates touchees) : toploc compile + tests + doctests ✅.
- Le seul fail Docker initial (`start_writes_running_json_and_responds_to_health`,
  nexus-shell-daemon) = ENV (auth_token conteneur) ; PASSE sur Windows (crate non touchee).

## Codex reconciliation

Codex GPT 5.5 (`sprint77_phase_g_codex_review.md`, output brut `codex exec -o`,
format livrable canonique) : **8 livrables, 8 CONFIRME, 0 GAP, 0 PARTIEL**. Codex a
verifie chaque livrable avec evidence fichier:ligne + les 3 invariants transverses :
no-float pre-image (to_bytes serialise seulement u32+u16), 0-bump wire (slots
`[u8;32]` reutilises, `DOMAIN_RUN_PROOF_V1` inchange, 0 nouveau `DOMAIN_*`/
`*_FORMAT_VERSION` dans le diff indexe), scope (`compare()` trouve uniquement dans
docs+tests `toploc.rs` ; `rerun`/`redundancy`/`validator` absents du diff ; 0 emission
prod `RunProof`). Re-execute `cargo test -p nexus-core-rs toploc` (17) +
`cargo test -p nexus-worker-core toploc_commitment_is_deterministic_and_swap_sensitive`
= verts. **0 GAP → aucune boucle de fix. review promu PASS.**

> Note process : 1er run Codex en format « PASS/CLEAN » (mon prompt) refuse par le
> hook lightcheck (Check 7 exige le vocabulaire CONFIRME/PARTIEL/GAP du template
> `.claude/templates/codex_phase_review.txt`) ; re-run au format canonique, output
> brut non reecrit. Meme verdict (8/8 CONFIRME).
