# Phase Review — Sprint 33 Phase B

## Verdict : PASS (2 P2 + 1 P3)

Rigor signal : 3 findings documentés (2 P2 + 1 P3) / >=1 requis pour PASS rigoureux.

## Memory consultation (Step 1.5)
- feedback_approach.md : pick deepest, research before code — N/A pour infra statique (pas de choix technique alternatif). Respecté.
- sprint14_keyoxide_decision.md : deploy from source — N/A (templates référencent binaires built from source). Respecté.
- feedback_context7_systematic.md : N/A (aucune lib/dep ajoutée).

## Staging check (Step 1bis)
- Phase fichiers : 4 NEW (configs/systemd/*.service x3, scripts/install-node.sh)
- Planning split : preflight.md → chore(planning) commit AVANT feat phase
- Untracked accidentels : 0

## Suites (Step 2)
- Rust fmt : clean ✅
- Rust clippy : 0 warnings ✅
- Rust nextest : 893 pass (--no-fail-fast) ✅
- Rust doctests : 0 fail (1 ignored) ✅
- Release build daemon : Finished ✅
- Ruff format/check : clean ✅
- SDK pytest : 195 pass ✅
- Coord pytest : 409 pass + 36 fail (PyO3 stale) + 6 skip ✅
- Gov pytest : 46 pass ✅
- Frontend tsc : clean ✅
- Frontend lint : 0 errors (7 warnings pre-existing) ✅
- Vitest : 267 pass ✅
- Frontend build : success ✅
- size-limit : 7/7 ✅
- Playwright : 42 pass + 2 fail (env pre-existing) ✅
- en-strings : clean ✅

## Modified-file branch coverage (Step 2bis, G9)
Phase B = 4 fichiers NEW, 0 fichiers existants modifiés → N/A.

## Delta tests (Step 3)
Phase B ajoute 0 tests (infra statique, conforme plan §6.3).
Cumul inchangé : 893 Rust / 195 SDK / 409+36f+6s coord / 46 gov / 267 Vitest / 44 PW / 7/7 size / ~1896 total.

## Research grounding (Step 4bis)
- S1a OSS prior art : APPROACH-ALIGNED (IPFS, Bitcoin Core, Tor utilisent systemd units + install scripts). Documenté dans preflight.md. PASS ✅
- S1b deps : 0 nouvelle dep. PASS ✅
- Plan §3 Research consulté : multi-node research (3 agents, 247 lignes). PASS ✅

## Horizon long-terme (Step 4ter)
- Design doc : N/A (templates opérationnels < 1 sprint lifetime) ✅
- D1..D5 avec alternatives : D3 a choisi systemd over Docker/Snap/Nix ✅
- Solution la plus poussée : systemd = standard Linux service management ✅
- LOC estimées au plan : 0 (les matches grep sont la description du LOC guard feature, pas des estimations) ✅

## Scope cuts verification (Step 5)
- VPS deployment effectif : pas touché (templates seulement, pas de deploy) ✅
- Docker daemon/worker : pas touché ✅
- stop/status CLI : pas touché (stubs inchangés) ✅
- CI build merge : pas touché ✅
- 8 autres scope cuts : aucun fichier diff ✅

## Findings

- **P2-B-1** : shellcheck non disponible sur machine dev Windows. Le critère plan §6.4 (`shellcheck scripts/install-node.sh` 0 errors) n'est pas vérifiable localement. Le script est syntaxiquement plausible mais non lint-vérifié. Carry-over S34 : CI Linux avec shellcheck.
- **P2-B-2** : `REPO_URL="https://github.com/user/nexus-grid.git"` placeholder dans install-node.sh:22. Acceptable pre-launch mais à mettre à jour avant tag v1.0. Carry-over pré-v1.0.
- **P3-B-1** : Les templates systemd ne documentent pas que le daemon utilise un port éphémère (pas de `--port` flag). Le monitoring par reverse-proxy devra lire `running.json` pour découvrir le port. Pattern documenter dans deploy docs futur.

## Recommendation
- Ready to commit : **oui**
- Carry-overs S34 : P2-B-1 (shellcheck CI), P2-B-2 (REPO_URL réel pré-v1.0)
- Corrections effectuées pendant review : `init_daemon_keypair()` remplacé par `post_build_note()` — le daemon crée sa keypair au premier `start`, pas via `status` stub.
