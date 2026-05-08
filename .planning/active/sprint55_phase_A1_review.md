# Sprint 55 Phase A.1 — review

HEAD: staged diff (pre-commit) | Timebox: 12m

## Verdict : PASS

## Dimensions

| Dim | Status | Evidence |
|---|---|---|
| Security | ok | 0 unsafe, 0 secrets, test-only files, grep confirmed |
| Patterns | ok | sleep→drop+await (broadcast), sleep→yield+drop+shutdown (mpsc), sleep→pause+advance (GCRA negative proof) — tous legit |
| Scope-cuts | ok | 15/15 grep verifie, 0 match dans le diff |
| Tests-delta | ok | +0/+0 annonce = reel (refactoring 6 tests existants) |
| Research | ok | 0 dep ajoutee, tokio APIs standard, trace preflight S1b confirme |
| G8 | ok | `sprint55_phase_A1_preflight.md` present, verdict EXECUTE |

## Acknowledged by G8 preflight (not re-derived)

- S1a SOTA : test refactoring interne, poll+deadline pattern standard tokio — APPROACH-ALIGNED
- S1b deps : 0 lib ajoutee — clean
- S2 historiques : 4 fichiers, 1 commit S22 non-applicable — clean
- S3 threat model : fast-path, 0 composant securite touche — clean
- S4 wire format : fast-path, 0 fichier wire format touche — clean

## Findings

- **P3** (nit) : `dispatch_loop_writes_to_doc` — `yield_now()` donne
  une seule opportunite de scheduling. Pattern correct car le message
  est deja dans le buffer mpsc (capacity 64) et `select!` drainera
  `recv()` avant `shutdown`. Race theorique si le runtime tokio
  ordonnance autrement, mais en pratique deterministe. Surveiller si
  flaky en CI rapide.

- **P3** (nit) : `rate_limit_gate_rejects_saturated_tuple` et
  `rate_limit_gate_defer_preserves_task` — `tokio::time::pause()`
  freezes le driver tokio globalement, incluant les timers internes
  iroh. Pour une preuve NEGATIVE (asserter l'absence de claim), le
  risque est conservateur : si iroh se bloque, le test passe quand
  meme. `tokio::time::resume()` avant `handle.await` est bien present
  (ligne 1713 et 1780). Pattern acceptable pour ce cas d'usage.

## Remaining sleeps non-touches (hors scope A.1)

Les 3 sleeps restants dans runtime.rs (lignes 1499, 1638, 1836) sont
dans des tests POSITIFS (poll d'evenement reel, pas synchronisation
fixe) : `engine_claims_and_executes_tasks_on_registered_doc`,
`rate_limit_gate_admits_fresh_tuple`, `rate_limit_gate_reloads_live_policy`.
Ces tests attendent un evenement asynchrone reel — `sleep+loop` est
le pattern correct la (pas de substitute deterministe sans refactor
significatif). Hors scope Phase A.1, coherent avec le plan §4.1.2.

## Verification scope-cuts (15 items)

Grep diff : LT-7 cross-platform, toolchain bundle, auto-update, build
log streaming, podman, outbox persistant, browse_request rate-limit,
test E2E multi-noeuds, windows cfg(unix), forbid-deny-doc, lightcheck,
rustfmt drift, flaky browse, Protocol Explorer/Ideas Hub, Kudos-v2 —
0 match dans les 3 fichiers modifies (test-only).

## Recommendation

Commit autorise. 2 P3 informationels, 0 bloquant.
