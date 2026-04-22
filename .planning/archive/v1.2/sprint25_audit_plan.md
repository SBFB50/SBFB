# Sprint 25 — Audit Plan pour Sprint 26

**Redige** : 2026-04-22
**Sprint audite** : Sprint 25 (fondations securitaires pre-tool-calling :
key rotation + C3 handoffs + D5 capabilities + P2 batch DNS concurrent)
**Commits** : de `a6985b1` (chore open S25) a `55e42fd` (Phase D) inclus.

---

## 1. Checklist d'audit (Phase 0 Sprint 26)

L'auditeur independant (nouvel agent, session fraiche) doit :

1. Lire ce document + `sprint25_kickoff.md` D1-D5 + `sprint25_plan.md`
2. Verifier chaque commit Phase A-D contre le plan (scope, fichiers,
   tests annonces vs reels)
3. Scanner pour regressions, dead code, security smells
4. Emettre `sprint25_audit_findings.md` avec verdict PASS / CONDITIONAL
   PASS / FAIL + items P0-P3

---

## 2. Dimensions a auditer

### Track A — Key rotation ceremony (Phase B)

- [ ] `KeyRotationAnnouncement` struct immutable, pas de `pub` sur
  champs internes (si applicable)
- [ ] `sign()` utilise `DOMAIN_KEY_ROTATION_V1` domain separation
  (pas de signature raw sans domain)
- [ ] `verify()` valide signature avec `old_public_key` (pas la nouvelle)
- [ ] Canonical bytes via JCS deterministic (pattern canary S21 Phase E)
- [ ] `RevocationCache::apply_announcement` verifie signature avant
  insertion (pas d'insertion sans verification)
- [ ] `is_revoked` vs `is_in_transition` : semantique correcte
  (transition = les 2 cles valides, revoked = ancienne rejetee)
- [ ] `transition_days = 0` edge case : revocation immediate
- [ ] `CuratorListEntry::verify_signature` check `RevocationCache`
  AVANT accept signature (pas apres)
- [ ] Gossip subscribe topic `nexus-grid/key-rotation/v1` distinct
  des topics existants (canary, pow)
- [ ] `KEY_ROTATION_FORMAT_VERSION = 1` pre-launch compliance
- [ ] Pas de tolerant decoder multi-version

### Track B — StageGuardrailMap (Phase C)

- [ ] `StageGuardrailMap` accepte uniquement les 5 stages valides
  (on_claim_broadcast, on_task_dispatched, on_result_received,
  on_validator_post_task, on_quarantine_enqueue)
- [ ] Backward compat : `input_chain` wrape dans
  `{"on_task_dispatched": input_chain}` sans perte
- [ ] `OutputSafetyGuardrail` migre de validator.py inline vers
  `stage_guards["on_result_received"]` (pas de double execution)
- [ ] Stage absent = passthrough (pas d'erreur, pas de log spam)
- [ ] Chain error resilience : exception guardrail → log + continue
  (pattern HookRunner fire-and-forget S24 Phase C)
- [ ] Tripwire propagation : `InputTripwire` dans stage chain stop
  l'execution correctement
- [ ] Tests ordering : chain execute les guardrails dans l'ordre insere

### Track C — D5 capabilities gate-off-by-default (Phase D)

- [ ] 6 capabilities definies dans `capabilities.toml` : `tool_calling`,
  `rag_retrieval`, `mcp_server_expose`, `external_api_access`,
  `code_execution`, `file_system_access`
- [ ] Toutes OFF par defaut (gate-off-by-default, pas gate-on)
- [ ] `integrity_hash` SHA-256 recalcule a chaque mutation (enable/disable)
- [ ] Tamper detect : hash mismatch → fallback all-OFF (pas d'erreur
  silencieuse qui laisse des capabilities actives)
- [ ] `admin_check.py` : Unix `os.geteuid() == 0` check present
- [ ] `admin_check.py` : Windows `IsUserAnAdmin()` + MIL High check
  present (double verification, pas IsUserAnAdmin seul)
- [ ] `@require_capability` decorator : HTTP 403 quand disabled (pas
  de bypass via query param ou header)
- [ ] CLI `nexus-admin` : `enable`/`disable` appellent `require_admin()`
  AVANT mutation (pas apres)
- [ ] CLI `audit-trail` : retourne historique chronologique des mutations
- [ ] Semgrep rule `.semgrep/capability_gate.yml` : pattern-match
  endpoints `/tool/`, `/rag/`, `/mcp/` sans decorator
- [ ] `CAPABILITY_TOGGLES.md` status updated design-only → implemented

### Track D — DNS concurrent + quarantine alerting (Phase A)

- [ ] P2-E-1 : chaque `DnsEndpoint` utilise son propre `tls_name`
  (pas `endpoints[0].tls_name` global)
- [ ] P2-E-2 : `tokio::select!` concurrent DoH+DoT, premiere reponse
  gagne, pas de biais systematique
- [ ] P2-E-2 : both-fail retourne erreur combinee (pas seulement
  la derniere erreur)
- [ ] P2-D-2 : quarantine enqueue emet `structlog.warning` avec
  `worker_id`, `reason`, `task_id` (tous les 3 presents)
- [ ] HARDENING_ROADMAP `last_validated` = 2026-04-22

### Track E — Process / meta

- [ ] G8 preflight systematique 4/4 phases A-D (documents presents
  dans active/)
- [ ] Phase reviews A + C presentes (B et D reviews : verifier si
  absentes ou incluses dans commit body)
- [ ] Commit bodies contiennent delta tests cumule + scope cuts
- [ ] Pas de dead code introduit (unused imports, unreachable branches)
- [ ] Pre-launch protocol respecte (VERSION = 1 partout, 0 tolerant
  decoder multi-version, KEY_ROTATION_FORMAT_VERSION = 1 nouveau)
- [ ] SPDX / ruff F401 clean
- [ ] Scope cuts §4 honores (14 items, 0 intrusion)

---

## 3. Items connus (carry-over Sprint 25)

| ID | Severite | Description | Source |
|---|---|---|---|
| P2-D-1 | P2 | Redundancy full wire-up + persistence : `RedundancyDispatcher` existe (S23 Phase D) mais n'est instancie nulle part en production (dispatcher.py recoit `None` par defaut). Le wire-up `collect_result` dans le result path + persistence SQLite restent a faire. Libelle precedent "in-memory → SQLite" masquait l'etat reel | S23 carry, reclassifie S25 post-sprint |
| P2-E-1-iroh | P2 | iroh neighborhood enrichment | S23 carry |
| T-NN+2 | P3 | iframe Rust-wasm (PATTERNS §P34, triggers inactive) | S22 carry |

---

## 4. Zones a risque (recommandation d'attention supplementaire)

1. **Key rotation revocation cache concurrency** : le `RevocationCache`
   est wrape dans `Arc<RwLock>` dans le daemon. Verifier qu'il n'y a
   pas de deadlock potentiel entre le gossip subscribe thread et le
   curator verify path.
2. **Capability store file permissions** : `capabilities.toml` devrait
   etre en permissions restrictives (admin-writable only). Verifier
   que le `CapabilitiesStore` ne cree pas le fichier avec des
   permissions trop ouvertes (`mkdir(parents=True, exist_ok=True)`
   ligne 212 cree le dir sans restriction de permissions).
3. **StageGuardrailMap validation** : verifier qu'un stage name
   invalide (typo dans la cle) est detecte (warning ou reject) et
   ne silently ignore le chain. (Phase C review P2-C-1 confirme :
   pas de validation, carry S26+.)
4. **Admin check bypass via import mock** : verifier que
   `require_admin()` ne peut pas etre bypass par un import mock dans
   le contexte de production (uniquement en test).
5. **RevocationCache silent overwrite** (identifie post-sprint) :
   `apply_verified()` (`key_rotation.rs:248`) fait un `insert` sans
   log ni warning si une entree existe deja pour `old_public_key`.
   Le test `cache_overwrites_on_second_rotation` documente le
   comportement mais ne le signale pas comme risque. Pre-v1.0 c'est
   acceptable (0 node externe), mais post-v1.0 un attaquant avec
   acces a l'ancienne cle (pendant la transition) pourrait ecraser
   une rotation legitime avec une fausse pointant vers sa propre cle.
   Mitigation future : log warning sur overwrite + reject si
   transition_start de la nouvelle est anterieur a l'existante.
6. **CapabilitiesStore._write() hash ordering** (identifie post-sprint) :
   le hash est calcule sur `tomli_w.dumps(data)` SANS integrity_hash,
   puis le champ est insere et `tomli_w.dumps(data)` est appele une
   seconde fois (`capability_store.py:206-213`). Le contrat de
   determinisme de tomli_w n'est pas garanti par la lib — si un
   upgrade change l'ordre de serialisation, le hash load/store
   divergerait. Verifier que les tests tamper couvrent un round-trip
   write→load→verify_hash.
7. **Admin check Windows MIL null pointer** (identifie post-sprint) :
   `admin_check.py:62-64` — `GetSidSubAuthorityCount` et
   `GetSidSubAuthority` retournent des pointeurs natifs sans
   verification de NULL. Sur un SID malformed, c'est un segfault
   potentiel. Les 2 tests Windows sont en skip (non-Windows CI).
   Verifier sur une machine Windows reelle ou ajouter un guard
   NULL check.

---

## 5. G8 pivot retrospective

0 DESIGN-CONFLICT S25. 4/4 phases EXECUTE (all 4 preflights present
dans active/). Cinquieme sprint consecutif (S21-S25) sans
DESIGN-CONFLICT — G1 Design Review Board pre-gel suffisant depuis
S21 pour eliminer les conflits en amont.

S1a OSS prior art systematique : 4 scans S1a documentes. Aucun
APPROACH-NAIVE detecte — plans alignes SOTA.
