# Sprint 79 Phase A — Synthese de review

Promotion du knowledge pack anime.js (9 couches) de `examples/daisyui-animejs-showcase/knowledge/` vers `docs/factory/knowledge/animejs/`, avec MANIFEST self-hashe + test hermetique de drift. Revue adversariale sur le diff STAGE comme source de verite. PLAN-ADAPT honore (la fausse affirmation kickoff « hashe gratuitement par provenance/FG8 » est refutee et n'est PAS utilisee par le code).

## Verifications independantes effectuees
- `git diff --cached --name-status` : 9x R100 (byte-identical) + 3 ajouts (preflight, test, MANIFEST.json) + `.gitattributes`. Rien d'extraneous.
- `cargo nextest run -p sbfb-factory animejs_manifest` : **1 passed**.
- Invariant wire/dep : `git diff --cached --name-only | grep -iE 'canonical|Cargo|_VERSION|DOMAIN_|schemas'` -> **NONE** ; `crates/sbfb-factory/Cargo.toml:14,22` confirme blake3 + serde_json deja deps (0 new dep).
- CR-bytes des blobs stages (`git show :path | od ... | grep -c '^0d'`) : **0 CR** sur primitives.json/README.md/DOCS.md/docs.json -> corpus LF-clean, hashes portables.
- Consommateur de l'ancien chemin : un seul, `examples/daisyui-animejs-showcase/knowledge/DOCTRINE_NEXT_SESSION_prompt.md` (NON stage, hors scope Phase A).

## Dimension 1 — Deliverables + diff correctness — `clean`
Diff complet et correct. 9 couches git-mv R100, MANIFEST.json miroir du jeu de cles daisyui (omet `default_theme`, ajoute `freshness`), hashes couvrant les 9 fichiers promus, counts coherents avec README. Test non-vacuux. P3 : `hash_convention` est une cle additive absente du miroir daisyui (utile, non-defaut) ; `method:"5-layer"` vs 9 entrees de hash (coherent : 1 couche logique = paire .json+.md).

## Dimension 2 — Test semantics + hermeticite — `clean`
`crates/sbfb-factory/tests/animejs_manifest.rs` est un detecteur de drift deterministe et hermetique (std::fs + serde_json + blake3 ; BTreeMap normalise l'ordre read_dir). Assert key-set (l.72) + assert digest map (l.80). Vacuite fermee sur tous les vecteurs (empty/missing/stale cassent l.72 ; MANIFEST manquant panique l.29). N'appelle PAS provenance/FG8 (noms en doc-comments seulement, l.2/10/12). P3 : `.gitattributes` exclu du set hashe (par-design, le CR-guard l.59-63 couvre le risque CRLF) ; faux-positif `grep -c $'\r'` (artefact Git Bash) ecarte par inspection `od` -> 0 CR.

## Dimension 3 — PLAN-ADAPT grounding + honnetete — `clean`
Faux-verdict re-verifie VRAI : `compute_output_hash` (provenance.rs:49) private, seul appelant `Provenance::generate` (l.29) cable a `template_engine.rs:264` sur un workspace d'APP ; FG8 (gates.rs:208) signe l'artifact_hash de l'app, ne parcourt jamais `docs/`. La cle plan T2 « provenance blake3 == MANIFEST » (plan:142-143) est incoherente (64-hex agregat vs 16-hex par-fichier) et a ete correctement reformulee par le preflight. 0 wording residuel trompeur dans le diff stage. P3 : doc stale `DOCTRINE_NEXT_SESSION_prompt.md:8` + README.md:4-5 disant encore « vitrine daisyui » — preserves byte-identique par R100, hors scope -> Phase E.

## Dimension 4 — Scope cuts + invariants pre-launch — `clean`
0 BUMP WIRE / 0 NEW DEP confirmes. ASSET HORS WORKSPACE D'APP : FG6 (`run_gate_fg6_secrets`, gates.rs:127-165) ne scanne qu'un `workspace` d'app passe -> 0 impact `docs/`. KNOWLEDGE NON-AUTHORITATIVE : aucune route daemon, aucun verdict PASS, aucun scoring ajoutes. Day-0 D1 (`docs/factory/knowledge/`) honore ; sous-dossier daisyui laisse en examples/ pour Phase E. P3 : divergence de conventions de troncature blake3 ([..8] gates / 16-hex MANIFEST / 64-hex provenance) tracee, back-fix daisyui (couverture partielle 3/4) defere Phase E ; obligation commit-body (documenter la deviation PLAN-ADAPT).

## Findings

| Severite | Description | Evidence | Disposition |
|---|---|---|---|
| P3 | `hash_convention` cle additive absente du MANIFEST daisyui miroir (documente la troncature 16-hex, utile) | `docs/factory/knowledge/animejs/MANIFEST.json:34` | Accepte (additif, conforme preflight) |
| P3 | `method:"5-layer"`/`layers` (5) vs `hashes` (9 fichiers physiques) — coherent (paire .json+.md) | `MANIFEST.json:3,13-19,35-45` | Accepte (lecture possible mais sans impact) |
| P3 | `.gitattributes` exclu du set hashe (dotfile) ; integrite repose sur eol=lf + CR-guard | `animejs_manifest.rs:50,59-63` ; `MANIFEST.json:34` | Accepte (par-design, frontiere connue) |
| P3 | Faux-positif `grep -c $'\r'` (artefact Git Bash) ; `od` montre 0 CR working-tree + blob stage | `od -An -tx1 \| grep -c '^0d'` => 0 ; nextest PASS | Aucune action (artefact d'outil) |
| P3 | Doc stale : `DOCTRINE_NEXT_SESSION_prompt.md:8` + README.md:4-5 referent encore « vitrine daisyui » | `DOCTRINE_NEXT_SESSION_prompt.md:8` ; `docs/factory/knowledge/animejs/README.md:4-5` | Hors scope (R100 byte-identical) -> Phase E |
| P3 | Divergence conventions blake3 ([..8]/16-hex/64-hex) + daisyui sous-hashe (3/4) | `gates.rs:154-155` ; `provenance.rs:79` ; `MANIFEST.json:34` | Tracee, back-fix Phase E |
| P3 (process) | Le commit body DOIT documenter la deviation PLAN-ADAPT (claim provenance/FG8 faux) + mecanisme corrige | `sprint79_phase_a_preflight.md:66-79,118-130` | Obligation a verifier au commit |

## Verdict: PASS

Aucun P0/P1. Diff correct et complet, test hermetique vert et non-vacuux, tous les invariants tenus (0 wire, 0 dep, asset hors workspace, knowledge non-authoritative, Day-0 D1). Les 7 findings P3 sont acceptes (additifs / par-design) ou differes Phase E (back-fix conventions blake3 daisyui ; docs stale README/DOCTRINE preserves byte-identical R100). L'obligation process (commit body documente la deviation PLAN-ADAPT + cite le preflight) est honoree.

## Codex reconciliation

Codex (GPT 5.5, `codex exec -o`) lance apres review PASS-PENDING ; output brut dans
`sprint79_phase_a_codex_review.md` (non reecrit). Verdict Codex : **5/5 CONFIRME, 0 GAP,
0 PARTIEL**. Codex a re-execute le test (`cargo test -p sbfb-factory --test animejs_manifest`
-> 1 passed), confirme les 9 renames R100 byte-identiques, le MANIFEST (hashes couvrant les 9
+ recompute vert), le `.gitattributes` (`check-attr` text=set / eol=lf / whitespace=unset), le
test non-vacuous SANS appel provenance/FG8, et le preflight PLAN-ADAPT. Cross-checks Codex :
0 dep nouvelle (`Cargo.toml`/`Cargo.lock` hors diff), 0 bump wire (`canonical`/`schemas`/
`DOMAIN_`/`*_VERSION` non touches), aucun cablage docs->provenance/FG8. Aucun GAP P0/P1/P2 ->
aucune correction requise ; suites non-relancees (0 changement post-Codex). Sequence respectee :
review PASS-PENDING -> Codex CLEAN -> promote PASS -> commit.