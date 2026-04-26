# Sprint 27 — Audit plan (pour S28 Phase 0)

**Date** : 2026-04-25
**Tip sortie S27** : sera le commit Phase E (post-migration)
**Auditeur** : session fraiche S28 Phase 0 (pas la meme session)

---

## 1. Dimensions d'audit

### Track A — P2 batch S26 (Phase A `f8b8e2d`)

1. **STAGE-1 validate_stage_guard_map** : verifier que
   `Dispatcher.__init__` appelle `validate_stage_guard_map(stage_guards)`
   et que le test `test_dispatcher_rejects_invalid_stage_guard_key`
   couvre au moins 2 cas (cle invalide + cle valide passthrough).
2. **EVENT-1 emit_capability_event** : verifier que le except dans
   `_emit_capability_event` log `logger.debug(exc_info=True)` et que
   le comportement reste non-bloquant (raise → caught → logged).
3. **DESC-1 TaskHandlerDescriptor** : verifier que le champ
   `description: str` est renseigne via `fn.__doc__` dans le decorateur
   `@task_handler`. Verifier que le manifest endpoint
   `GET /app/<name>/manifest` expose la description.
4. **ROTATE-1 JsonFileWriter** : verifier que la rotation declenche
   a `max_bytes` (10 MiB defaut), que les fichiers `.1`→`.5` sont
   decales correctement, et que le max 5 fichiers est enforce (pas
   de croissance illimitee).
5. **RENAME-1 TracingWriter** : grep exhaustif `EtwWriter` dans
   le workspace — 0 residuel attendu. Verifier que la documentation
   PATTERNS.md et les imports sont coherents.

### Track B — Watermark SynthID (Phase B `7bb656b`)

1. **PRF-1 determinism** : verifier que `_prf_score(token_id, context)`
   retourne exactement le meme score pour les memes inputs
   (test reproductibilite). Verifier que le secret est un parametre,
   pas un global.
2. **ZTEST-1 detection** : verifier que le z-test binomial a un
   false-positive rate acceptable (non-watermarked → z_score <
   threshold avec p > 0.95 sur 1000 runs aleatoires).
3. **INJECT-1 logit bias** : verifier que le delta `+2.0` est
   ajoute UNIQUEMENT aux tokens green (PRF score > 0.5). Verifier
   que le bias ne s'applique PAS quand `watermark.enabled = false`.
4. **CONFIG-1 watermark.toml** : verifier que le config sample
   `configs/watermark.toml.sample` est parsable et que les defaults
   (enabled=false, delta=2.0, window=4) sont documentes.
5. **RISK-1 llguidance conflit** : verifier si un test d'integration
   watermark + grammar llguidance existe. Si absent, documenter le
   gap comme P2 carry S29.

### Track C — Couche 3 multi-forge (Phase C `d52ce89`)

1. **PARSER-1 git-log format** : verifier que le format string
   `--format=%H|%aI|%GK|%G?|%GS` est correct pour git >= 2.34.
   Verifier que le parser handle le cas "git not found" ou "pas un
   repo git" avec erreur explicite.
2. **PARSER-2 SigType enum** : verifier que GPG et SSH sont les
   seuls variants, pas de catch-all `Other`. Verifier que les
   signatures X.509 sont ignorees (pas parsees, pas crash).
3. **CACHE-1 SQLite WAL** : verifier que TrustCache ouvre la base
   en WAL mode (pattern quarantine_queue S21). Verifier que le
   schema PRIMARY KEY (repo_url, fingerprint) previent les doublons.
4. **TRUST-1 score formula** : verifier que le score cross-forge
   est multiplicatif (forge_count × tenure × delegation_depth) et
   que le decay -1/hop est bien minimum 1 (pas 0 ou negatif).
5. **SEED-1 trust_web_seeds.toml** : verifier que le fichier
   bootstrap contient le placeholder FlowUP avec un fingerprint
   Ed25519 valide (pas de dummy `000...`). Les ONG sont commentees
   (pas de faux claims de partenariat).
6. **DELEG-1 DelegationCert** : verifier que `trust_level`,
   `valid_until`, `scope` sont ajoutes sans bump
   `DELEGATION_CERT_VERSION` (pre-launch policy). Verifier
   canonical JCS deterministe (meme input → meme bytes).

### Track D — Gate 3 docs (Phase D `814e485` + `4913f7f` + `6eee5ca`)

1. **ROADMAP-1 coherence** : verifier que HARDENING_ROADMAP §3 S27
   mentionne SynthID (pas Kirchenbauer residuel). Verifier
   `last_validated: 2026-04-25`.
2. **THREATS-1 §4.4** : verifier que COMPUTE_THREATS §4.4 mentionne
   "SynthID-inspired PRF z-test" et "BIRA rejection note". Verifier
   la reference arXiv:2509.23019.
3. **GATE3-1 checklist** : verifier que la checklist Gate 3 liste
   au minimum 14 items livres S22-S27 et que les items restants
   (audit externe S29, Tor S28+) sont explicites.
4. **PATTERNS-1 P37+P38** : verifier que P37 (watermark) et P38
   (trust-web) existent dans PATTERNS.md avec les bons chemins de
   fichiers (post `4913f7f`/`6eee5ca` corrections).
5. **SELFDIST-1** : verifier que SELF_DISTRIBUTION.md couvre les 8
   sections spec (principe, format bundle, canaux, bootstrap problem,
   lien S14, endpoint daemon, update P2P, implem target). Verifier
   coherence des chemins (post corrections).

### Meta-track — G8 traceability

1. Verifier que les 5 phases A-E ont chacune un
   `sprint27_phase_{X}_preflight.md` dans `.planning/archive/v1.2/`.
2. Verifier que les 4 phases A-D ont chacune un
   `sprint27_phase_{X}_review.md`.
3. Verifier la coherence verdict G8 × commit (5 EXECUTE → 5 commits
   phase livres, 0 DESIGN-CONFLICT → 0 pivot_proposal).
4. Phase D docs-only : verifier que les path corrections (`4913f7f`
   + `6eee5ca`) sont coherentes avec les corrections annoncees dans
   les commit bodies.

### Meta-track — Sprint pair S28 phase dette

S28 est un sprint pair → phase dette obligatoire (§6.2.1 Regle 1).
Candidats a inclure :
- Platform writers (journald, oslog) — scope cut S27 §7.9
- ONNX CI fixture (P2-B-1 S22 carry) — scope cut S27 §7.10
- EtwWriter → TracingWriter : verifier que le rename S27 Phase A
  est complet (Track A RENAME-1)

---

## 2. Calibration rigor G4

L'audit DOIT trouver au minimum 1 P2+ pour verdict PASS. Sinon
verdict CONCERN et re-audit dimension supplementaire.

---

## 3. Pre-launch protocol check

Verifier :
- `DELEGATION_CERT_VERSION = 1` (pas bumpe malgre extension S27)
- Aucun tolerant decoder multi-version introduit
- Aucun test "legacy decode" zombie introduit
- `#[serde(default)]` ajoutes avec rationale runtime tolerance
