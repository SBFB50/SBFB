# Retro-review — Bloc off-sprint (Sprint 71 Phase D / G5)

## Verdict : RECONCILED (CONDITIONAL → fermeture in-sprint)

Retro-review **independante** des ~14 commits techniques du bloc
off-sprint `201b24d..d5ddb95` (+5574/-682, 33 fichiers) qui ont ete
livres **hors session agent** (par le PO, FlowUP), donc sans le cycle
preflight → review → Codex → audit → body 9 sections. Cette review
applique a posteriori les dimensions §4.5 / §7.4 que le bloc n'a jamais
recues, et **trace ou chaque finding est ferme** (Phase A/B/C/D du S71,
ou differe S72-S74).

Elle ne re-decouvre pas la dette : elle s'appuie sur l'audit-absorb
deja produit (`sprint70_audit_findings.md`, Phase 0) qui a cartographie
B-1..G13, et la verifie dimension par dimension. Le bloc **compile et
tournait vert** a HEAD (`d5ddb95`) — la dette est de **process et de
couverture**, pas de regression fonctionnelle silencieuse (sauf B-1/B-2,
qui sont des bugs de fond fermes en A/B).

Date : 2026-05-30. HEAD : `a0337c6`. Auditeur : session S71 Phase D
(n'a pas ecrit le code off-sprint → independance preservee).

---

## Perimetre audite

**14 commits techniques** (les 14 commits editoriaux restants — 13
`docs(community)` CHATONS + 1 `docs(readme)` — sont hors scope technique,
audit §7) :

| SHA | Titre | Surface introduite |
|-----|-------|--------------------|
| `a8a273f` | endpoint /api/sprint-history ultra deep + page SprintHistory | `sprint_history.rs` (1047 l.), endpoint, front |
| `5f2cc9a` | diff endpoint + inline code viewer + all sprints nav | `commit_diff_data`, `parse_unified_diff` |
| `e73c9fb` | format sprint_history + fix operator proxy port | format, proxy |
| `e26d9f2` | wire chat to Claude CLI subprocess + SSE streaming | `llm_bridge.rs`, SSE `handle_chat_stream` |
| `886eed0` | use claude.cmd on Windows for subprocess spawn | spawn Windows |
| `35ec331` | pass prompt via stdin instead of CLI argument | stdin plumbing |
| `eb06c35` | debug panel + bypassPermissions mode | `--permission-mode bypassPermissions` |
| `df69cfd` | show thinking in chat + collapsible | front AgentChat |
| `c3f4813` | embedded Claude Code terminal + live dashboard | `terminal.rs` PTY |
| `864b005` | persist terminal sessions as asciicast + session list | `.cast` writers, `list_sessions` |
| `0aa06db` | resume Claude sessions in terminal + open recordings | resume plumbing |
| `d5ddb95` | prompt paste buttons above terminal | front |
| `69019ed` | upgrade model refs 4.6 → 4.8 + fast mode | agents/docs |
| `a4d245d` | fix Codex reconciliation section in Phase G review | planning |

---

## Les 11 dimensions

### D1 — Commit hygiene / atomicite
Les commits sont **thematiquement atomiques** (un sujet par commit) mais
**ne portent pas** le body 9 sections, le titre `Sprint N Phase X`, ni
les gates. C'est la nature meme du bloc off-sprint. Pas de commit
fourre-tout : le decoupage par feature reste lisible et reviewable
a posteriori. **Finding G5/P1** (process) → ferme Phase D (cette review
+ retro-Codex). Pas de re-ecriture d'historique (pre-launch, mais
l'historique off-sprint est conserve tel quel — re-ecrire = perte de
tracabilite).

### D2 — Suites / couverture de tests
A la livraison : **0 test** sur `terminal.rs`, `sprint_history.rs`
(0/1047 lignes), `operator_server` unit, `process.rs`, spawn LLM/PTY
(audit G6/P1). Le parsing git+markdown de `sprint_history.rs` est
fragile et non couvert. **Finding G6/P1** → ferme Phase D : +13 tests
(`terminal::tests` 2, `process::tests` 3, `sprint_history::tests` 3,
endpoints HTTP 5), 125/125 sbfb-factory. La securite Factory (spawn,
auth, SSE) a deja recu +14 tests en Phase C.

### D3 — Branch coverage des surfaces modifiees
Les branches critiques non couvertes a la livraison :
- `parse_unified_diff` (add/del/ctx) → testee Phase D (fixture
  hermetique `parse_unified_diff_classifies_line_kinds`).
- `list_sessions` filtre extension `.cast` → testee Phase D.
- `resolve_kind` aliases → testee Phase D.
- `handle_chat_stream` gate SSE, auth middleware → testees Phase C.
Reste non couvert (assume) : le PTY live `handle_terminal_ws`
(spawn `claude` reel, non hermetique — exclu par design, cf. preflight
Phase D S1a). **Couvert / ferme A-D.**

### D4 — Securite (deep)
Surface la plus risquee du bloc :
- `--permission-mode bypassPermissions` (`llm_bridge.rs:80`) + SSE
  court-circuitant SENSITIVE_ACTIONS → **G2/P0**, ferme Phase C (gate SSE).
- CORS `Any` + zero auth sur serveur qui ecrit/spawn → **G7/P1**, ferme
  Phase C (token X-SBFB-Token + Host guard + CORS restreint, module
  `auth.rs`).
- Modele hardcode `"sonnet"` → **G9/P1**, ferme Phase C (`claude-opus-4-8[1m]`).
- Spawn sans timeout + `claude` resolu sans diagnostic → **G12/P1**,
  ferme Phase C (timeout + pre-spawn check).
**Nouveau finding du retro-Codex G5 (P1, ferme Phase D)** : le retro-Codex
GPT-5.5 a *live-probe* une **git option injection / ecriture de fichier
arbitraire** via les parametres `sha`/`rev` : `handle_commit_diff`
(`operator_server.rs`) ne rejetait que `len<4`/`..`/`/`, et `handle_audit`
n'avait aucun guard — un `--output=<path>` atteignait `git log`/`git diff`
qui ecrivaient un fichier (9142 octets observes sur `/api/audit`). C'est la
classe G20/P3 de l'audit-absorb, mais SOUS-EVALUEE : c'est un P1 (primitive
d'ecriture arbitraire), pas un P3 cosmetique. **Ferme Phase D** : guard
`is_safe_git_rev` (rejet leading-`-` + whitespace/control) sur les deux
endpoints + `--end-of-options` sur tous les appels git (defense en
profondeur) + 2 tests (option injection -> 400, zero fichier ecrit). Le P2
voisin (terminal `{name}.cast` non sanitise) est aussi ferme (guard
traversal + test). Exploitabilite reelle FAIBLE (endpoint gate token+Host+
CORS Phase C, loopback only) mais la primitive est reelle et le guard
existait justement pour ca.

Limite residuelle documentee : le PTY WS `/api/terminal/ws` est protege
par l'auth de connexion mais pas par le gate de contenu (pas de "dernier
message" a inspecter) → **carry T1 plein post-S71** (contrat §4 + review
Phase C l.141-146). **Apres le fix Phase D : pas de faille ouverte non
tracee.**

### D5 — Scope cuts / Day-0
Le bloc off-sprint a **anticipe** des surfaces (terminal embarque, chat
SSE, sprint-history viewer) qui appartiennent a l'arc Factory roadmap v5.
Aucune ne contredit une decision Day-0 figee. La couverture Phase D ne
touche **aucun** scope cut S72-S76 (ProviderRouter, routage reseau,
FTS5, fork, GPU, sharding). Le SSE chat cable au routage reseau reste
**non cable** (scope cut #2 → S72). **Conforme.**

### D6 — Research grounding
Dimension la plus faible du bloc : **aucun preflight G8** n'a precede
ces surfaces (pas de scan S1a OSS / S1b CVE / S2 decisions / S3 threat /
S4 wire). Consequence concrete : **G13/P2** — `portable-pty`,
`async-stream`, `futures` ajoutees sans scan CVE. → passe au preflight
Phase B (G13) et re-trace au preflight Phase D S1b (portable-pty 0.9.0 :
zero advisory RustSec au 2026-05-30, surface spawn non sollicitee par
les tests). **Ferme B + D (trace preflight).**

### D7 — Wire format / pre-launch protocol
Verifie : `terminal.rs`, `sprint_history.rs`, `process.rs` ne touchent
**aucun** `*_VERSION`, canonical JCS, domaine de signature, schema
protocole. Le `.cast` asciicast est un log on-disk local ; les reponses
JSON (`SprintHistoryResult`, `CommitDiffResult`) sont des contrats UI
locaux, pas des wire formats propages. **Aucune violation pre-launch.**
(Le double mismatch contrat JSON UI G8/G4 est cote *front* Viewer, hors
socle compute → S72/S74.)

### D8 — Patterns / tech debt
Dette structurelle absorbee :
- Double notion « provider » (`process.rs:24` string vs runtime
  `LlmBackend`) → **D8**, clarifiee Phase B (doc PATTERNS, non unifiee).
- `RedundancyDispatcher` mort, `execute_build` jamais appele → **D8**,
  Phase B.
- `sprint_history.rs` = git+markdown parsing fragile sans tests → couvert
  Phase D (parsers hermetiques).
Nouveau (Phase D) : aucun pattern introduit, code de test inline suivant
le pattern etabli du crate (`#[cfg(test)] mod tests`).

### D9 — Deliverables vs intent
Les features livrees (terminal, chat SSE, sprint-history, diff viewer)
**fonctionnent a l'usage** cote serveur (endpoints repondent, asciicast
persiste). Les mismatches d'usage sont cote front Viewer (G4/G8 →
S72/S74). Le bloc a livre de la **valeur produit reelle** (l'Operator
Factory est utilisable), au prix de la dette process/securite/test
maintenant reconciliee. **Intent atteint, dette payee A-D.**

### D10 — Conformite process (le finding central)
**G5/P1** : ~14 commits sans cycle (preflight/review/Codex/audit/body).
C'est la deviation que cette Phase D **reconcilie** : retro-review (ce
fichier, 11 dimensions) + retro-Codex (`sprint71_offsprint_codex_review.md`,
exec brut) + l'audit-absorb retroactif Phase 0. Le process portable
(AGENT_SYSTEM, gates, hooks) livre en S70 existe desormais pour eviter
la recidive ; le superviseur S71 + hooks lightcheck sont le backstop.
**Ferme Phase D + Phase 0.**

### D11 — Carry-overs / disposition
| Finding | Sev | Disposition |
|---------|-----|-------------|
| B-1 cle dispatch | P0 | CLOSED S71 A |
| B-2 quorum stochastique | P0 | CLOSED S71 B |
| B-3 zero E2E cross-process | P1 | CLOSED S71 A |
| G1 WIP terminal stash | P0 | CLOSED S71 A (stash drop, .cast garde) |
| G2 SSE non-gate | P0 | CLOSED S71 C |
| G7 CORS/auth | P1 | CLOSED S71 C |
| G9 modele sonnet | P1 | CLOSED S71 C |
| G12 spawn timeout/diag | P1 | CLOSED S71 C |
| G5 bloc non reconcilie | P1 | CLOSED S71 D (ce fichier) |
| G6 surfaces non testees | P1 | CLOSED S71 D (+16 tests) |
| git rev option injection (retro-Codex) | P1 | CLOSED S71 D (guard + --end-of-options) |
| terminal {name}.cast traversal (retro-Codex) | P2 | CLOSED S71 D (guard) |
| G13 deps off-sprint CVE | P2 | CLOSED B+D (trace preflight) |
| G4 Viewer casse | P1 | DEFER S74 (front, hors socle) |
| G8 mismatch contrat UI | P0-usage | DEFER S72 (front) |
| G10 socle readonly orphelin | P1 | DEFER S72-S74 |
| G16 pas d'E2E publish daemon reel | P1 | DEFER S72+ (hors socle compute) |
| G3/G17 boucle produit | P1-P2 | DEFER S74 |
| G14/G15/G18-23 | P2-P3 | DEFER S72+ / dette |

---

## Conclusion

Le bloc off-sprint a livre de la **valeur produit reelle** (Operator
Factory utilisable) mais a contracte une dette de **process** (G5),
**couverture** (G6), **securite Factory** (G2/G7/G9/G12) et **compute de
fond** (B-1/B-2/B-3). S71 phases A-D **ferment l'integralite des P0/P1 du
socle compute + securite + reconciliation** ; les findings **produit /
pipeline hors socle** — G4/P1 (Viewer), G8/P0-usage (contrat UI), G10/P1
(readonly orphelin), G3 (boucle produit) **et G16/P1 (E2E publish daemon
reel)** — restent legitimement differes S72-S74 conformement a
l'audit-absorb (ils ne bloquent pas l'assainissement du socle). La
couverture Phase D + cette retro-review + le retro-Codex satisfont
**G5/G6** et closent la deviation audit-absorb.

**Verdict : RECONCILED.** Aucun P0/P1 du **socle
compute+securite+reconciliation** ouvert apres S71 A-D. Les P1 hors socle
(G4/G10/G16) et le P0-usage UI (G8) sont traces DEFER S72-S74 pour
`sprint72_audit_findings.md` — declassement explicite : hors socle, pas
oublies.
