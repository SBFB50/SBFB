# Sprint 43 Phase B — review

HEAD: working tree (post-preflight `3f6b384`) | Timebox: ~18m

## Verdict : PASS

Rigor signal : 1 P2, 1 P3 documentes. Tous verifiés code actuel.

## Dimensions

| Dim | Status | Evidence |
|---|---|---|
| Security | P2 | content-disposition/content-type header injection via unsanitized original_name — files.rs:160 |
| Patterns | ok | atomic write consent.rs:137-139 conforme shell/PATTERNS.md §atomic. 0 unwrap() hors #[cfg(test)]. |
| Scope-cuts | ok | 6/6 cuts grepped — 0 match. kudos refs = threat labels uniquement. |
| Tests-delta | ok | annonce +14 (1091→1105), reel 1105 nextest workspace PASS. consent 8 + files 6 = 14. |
| Research | ok | sha2 = workspace dep pre-existant (Cargo.toml:70), hex idem. 0 nouvelle dep externe. |
| G8 | ok | sprint43_phase_B_preflight.md present, verdict EXECUTE. |

## Acknowledged by G8 preflight (not re-derived)
- S1 SOTA : CAS pattern standard, consent JSON trivial I/O — CLEAN
- S2 historiques : 0 commit conflict sur files.py / consent.py — CLEAN
- S3 threat model : 0 nouveau composant securite — fast-path CLEAN
- S4 wire format : 0 canonical.rs/schemas touche — fast-path CLEAN

## Findings

- **P2** : `stream_file` (files.rs:160) insere `manifest.original_name` directement dans le header `content-disposition: inline; filename="<name>"` sans sanitisation. Un `original_name` contenant `"` ou `\r\n` peut casser le header ou injecter des headers supplementaires. Meme risque sur `manifest.content_type` passe a `content-type` (files.rs:157). Vecteur : upload d'un fichier avec `x-original-name: evil\r\nX-Injected: header`. Fix : stripper les chars `"`, `\r`, `\n` du `original_name` avant insertion ; idem sur `content_type` (whitelist MIME ou strip CRLF). Severite P2 (daemon loopback auth-required reduit l'exposition mais le manifest persiste sur disque).

- **P3** : Plan §B.4 exige "tests integration HTTP pour chaque endpoint" — les 14 tests livres sont tous des unit tests (serde/validation), aucun HTTP oneshot roundtrip pour les 7 nouvelles routes. Precedent S42 Phase B accepte le meme pattern (deploy.rs = unit tests, http.rs coverage = "pas de nouvelle logique"). Alignement avec S42 = acceptable. Recommandation : HTTP oneshot tests pour consent + files en Phase D wrap-up ou S44.

## Recommendation

Commit autorise sous reserve de fix P2 (sanitisation header). Le fix est ~5 LOC dans `stream_file` — ne justifie pas un pivot. Corriger avant `git commit` : strip `\r\n"` sur `original_name` et `content_type` dans `stream_file`.
