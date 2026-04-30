# Sprint 43 Phase C — review

HEAD: a55aa1c (preflight) / working-tree diff | Timebox: 20m

## Verdict : PASS (post-fix P1)

P1 corrige : test stale proxy_contributor_verify_bad_gateway
supprime. Delta ajuste +6 (1105→1111). 147/147 daemon tests PASS.

## Dimensions

| Dim | Status | Evidence |
|---|---|---|
| Security | ok | 0 secret, 0 unsafe, unwrap() en test-only uniquement |
| Scope-cuts | ok | 6/6 items grepped, 0 match dans diff |
| Tests-delta | ok (post-fix) | annonce +7 corrige +6 (1105→1111), test stale supprime, 147/147 PASS |
| Research | ok | 0 nouvelle dep, axum/serde/rusqlite deja traces §Research |
| G8 | ok | sprint43_phase_C_preflight.md present, verdict EXECUTE |
| Patterns | P3 | contributor routes sans /v1/ prefix — inconsistance avec /api/v1/files, /api/v1/consent etc. |

## Acknowledged by G8 preflight (not re-derived)

- S1 SOTA : routes API thin wrappers sur modules Rust deja portes (CanaryInputManager S40, ContributorRegistry S41). 0 domaine nouveau.
- S2 historiques : 0 commit conflict sur canary.py et contributor.py.
- S3 threat model : fast-path verified. Conversion proxy→direct, meme DB, meme logique.
- S4 wire format : 0 canonical.rs/schemas touche.

## Findings

### P1 — BLOQUANT : test stale non nettoyé apres suppression proxy

`crates/nexus-shell-daemon/src/http.rs:1700-1721`

```
async fn proxy_contributor_verify_bad_gateway_when_coord_unreachable()
    assert_eq!(resp.status(), StatusCode::BAD_GATEWAY);  // line 1720
```

Le test teste l'ancienne branche "coord unreachable → 502" de
`proxy_contributor_verify` (supprime). Le nouveau handler direct
`contributor_api::verify_contributor` accede au `coordinator_db`
Mutex et retourne 500 (mutex ok mais SQLite echoue sans vraie DB)
— jamais 502. Test en FAIL confirme par `cargo nextest run -p
nexus-shell-daemon` : `FAIL proxy_contributor_verify_bad_gateway_when_coord_unreachable`.

Fix : supprimer le test ou le remplacer par un test qui verifie
le comportement du handler direct (ex: verifie que hex valides
mais DB absente → 500 INTERNAL_SERVER_ERROR).

Le test `proxy_contributor_verify_rejects_non_hex_path_params`
(ligne 1677) passe car `validate_hex` dans le nouveau handler
retourne toujours BAD_REQUEST — pas de fix requis pour celui-ci.

### P2 — Rouge-ligne documente : #[allow(dead_code)] nouveaux

`crates/nexus-shell-daemon/src/http.rs:140,147`

Deux annotations `#[allow(dead_code)]` nouvelles sur `coord_http_client`
et `coord_base_url`. Rouge-ligne trigger (cf. §5 "diff introduit
un #[allow(dead_code)] nouveau"). Raison legitime : le seul
appelant (`proxy_contributor_verify`) est supprime dans ce meme
diff. Les champs restent dans la struct car runtime.rs les
initialise encore. Candidats a suppression en S44/S45 (Python
coordinator suppression).

Recommandation : ajouter un commentaire `// TODO(S45): supprimer
avec suppression proxy Python` sur les deux champs pour rendre
le chemin de nettoyage explicite et eviter que les #[allow]
restent indefiniment.

Verdict rouge-ligne : **documente, non-bloquant** (raison claire,
champs encore necessaires a la construction de DaemonHttpState,
pas masquage d'un code path actif).

### P3 — Inconsistance prefixe route contributor

`crates/nexus-shell-daemon/src/http.rs:246-255`

Routes contributor portees sous `/api/contributor/...` (sans `v1`),
alors que les routes de cette meme phase C et des phases precedentes
utilisent `/api/v1/...` (files, consent, deploy, apps, kudos, tasks).
Les routes canary sont aussi `/api/canary/...` (etabli S22/S30) —
le contributor suit le meme pattern que canary.

Pas bloquant (pre-launch, 0 client externe), mais inconsistance
visible. A documenter ou normaliser en S44 quand toutes les routes
sont portees.

## Recommendation

PASS — P1 corrige (test stale supprime), P2 documente (allow
dead_code coord fields, cleanup S45), P3 informationnel (prefix
route /api/contributor sans /v1/). Commit autorise.
