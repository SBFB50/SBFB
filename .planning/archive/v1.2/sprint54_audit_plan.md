# Sprint 53 — Audit plan pour session fraiche S54

**Mode d'emploi** : ce document est destine a une session Claude
Code fraiche qui n'a pas ecrit le code de S53. Lire dans l'ordre :
1. Ce fichier (audit_plan)
2. `sprint53_kickoff.md` §4 (D1..D4 gelees — ne pas rebattre)
3. `sprint53_plan.md` (phases prevues vs livrees)
4. `sprint53_verification.md` (self-report fail-fast)

**Ne PAS lire avant d'avoir forme une opinion** :
- Les reviews de phases (sprint53_phase_*_review.md) — ce sont
  des self-reports, l'audit est independant
- Les preflights (sprint53_phase_*_preflight.md) — meme raison

**Timebox** : 2-3h max. **Delivrable** : `sprint52_audit_findings.md`
(malgre le nommage, c'est l'audit de S53, nomme pour le sprint
qui le joue : S54 Phase 0).

---

## Track A — P2P smoke test resultats

**Question** : les resultats P2P documentes dans les reviews et
verification.md sont-ils credibles et complets ?

**Methodes** :
- Verifier que Phase A review documente les resultats LAN (Win-Mac)
- Verifier que Phase B review documente les resultats WAN (dev-VPS)
- Croiser les niveaux atteints (1/2/3) avec verification.md §4 rows 15-21
- Verifier que les bugs trouves (gossip deadlock, bootstrap empty)
  sont traces dans les phases ulterieures (D, E, F, G)
- Grep `running.json` references dans le code pour verifier que
  le daemon ecrit bien ce fichier sur les 3 OS

**Signal** :
- P0 : niveau 1 non atteint (daemon ne demarre pas) et non documente
- P1 : resultats documentes contredisent le code (ex: claim "P2P works"
  mais gossip subscribe est commente)
- P2 : resultats incomplets (un OS manquant dans le rapport)
- P3 : details manquants (pas de latence, pas de RAM usage)

---

## Track B — Gossip pipeline correctness

**Question** : la chaine gossip (subscribe → bootstrap → outbox →
NeighborUp → browse pull) est-elle coherente end-to-end ?

**Methodes** :
```bash
# Verifier la chaine gossip dans runtime.rs
grep -n "join_topic\|gossip_sender\|GossipCmd\|outbox\|NeighborUp\|browse_request" \
  crates/nexus-shell-daemon/src/runtime.rs

# Verifier que le browse pull endpoint existe
grep -n "browse/pull\|RequestBrowse" \
  crates/nexus-shell-daemon/src/routes.rs \
  crates/nexus-shell-daemon/src/runtime.rs

# Verifier les tests gossip
cargo nextest run -p nexus-shell-daemon --locked -- gossip 2>&1 | tail -5

# Verifier le frontend Browse refresh
grep -n "browse/pull\|Rafraichir\|handleRefresh" web/src/pages/Browse.tsx
```

**Signal** :
- P0 : gossip subscribe bloque le daemon au demarrage
- P1 : browse_request broadcast sans rate-limit ET sans PoW
        (le gossip PoW mitigue mais verifier)
- P2 : outbox in-memory perd les entries au redemarrage (carry S54)
- P3 : code style, naming conventions

---

## Track C — Node identity persistence

**Question** : le node key persistent (Phase E) est-il correctement
implemente et securise ?

**Methodes** :
```bash
# Verifier le fichier node key
grep -n "node.key\|load_or_generate\|node_key" \
  crates/nexus-shell-daemon/src/runtime.rs

# Verifier les permissions fichier
grep -n "permissions\|0600\|readonly\|set_permissions" \
  crates/nexus-shell-daemon/src/runtime.rs

# Verifier que le key est Ed25519
grep -n "SecretKey\|Ed25519\|iroh.*key" \
  crates/nexus-shell-daemon/src/runtime.rs
```

**Signal** :
- P0 : key stockee en clair dans un dossier world-readable
- P1 : pas de verification de permissions au chargement
- P2 : permissions 0600 non appliquees (carry note S53)
- P3 : pas de rotation key mecanisme (LT)

---

## Track D — Route collision fix (Phase A)

**Question** : le fix de collision route daemon-served SPA est-il
correct et n'introduit-il pas de regression ?

**Methodes** :
```bash
# Verifier la structure des routes
grep -n "route\|Router\|get\|post\|api/daemon" \
  crates/nexus-shell-daemon/src/routes.rs | head -30

# Verifier les tests de routes
cargo nextest run -p nexus-shell-daemon --locked -- route 2>&1 | tail -10

# Verifier qu'aucune route ne collision avec le SPA fallback
grep -n "fallback\|catch_all\|not_found" \
  crates/nexus-shell-daemon/src/routes.rs
```

**Signal** :
- P0 : API endpoint inaccessible (masque par SPA fallback)
- P1 : regression sur un endpoint existant
- P2 : namespace non documente

---

## Track E — Edition 2024 / unsafe set_var scope cut

**Question** : le re-scoping du carry P2-REVIEW-B-1-S51 (unsafe
set_var → edition 2024 upgrade) est-il justifie et correctement
documente ?

**Methodes** :
```bash
# Verifier l'edition du workspace
grep "edition" Cargo.toml

# Compter les appels set_var/remove_var non-unsafe
grep -rnE "(std::env::|env::)(set_var|remove_var)" crates/ \
  --include="*.rs" | wc -l

# Verifier que le CLAUDE.md documente le re-scoping
grep -A 3 "set_var" CLAUDE.md

# Verifier le preflight Phase C documente la raison
cat .planning/active/sprint53_phase_C_preflight.md
```

**Signal** :
- P0 : n/a (pas de changement code)
- P1 : carry non documente dans CLAUDE.md
- P2 : documentation incomplete du re-scoping rationale
- P3 : compteur carry incorrect (doit etre 3/3 MANDATORY S55)

---

## Track F — Process meta

**Question** : le process sprint a-t-il ete respecte (commit
discipline, reviews, preflights, scope cuts) ?

**Methodes** :
```bash
# Verifier les commits atomiques
git log --oneline b85a3a1..HEAD

# Verifier le format des commit messages
git log --format="%s" b85a3a1..HEAD

# Verifier que chaque phase a un review PASS
ls .planning/active/sprint53_phase_*_review.md

# Verifier les preflights existants
ls .planning/active/sprint53_phase_*_preflight.md

# Verifier le design review G1
test -f .planning/active/sprint53_design_review.md && echo "G1 present"
```

**Signal** :
- P0 : commit non-atomique (multiple phases dans un commit)
- P1 : phase sans review, design review G1 absent
- P2 : preflight manquant pour une phase code (E, F n'ont pas
  de preflight — verifier si justifie car inserees post-plan)
- P3 : format commit body incomplet

---

## Track G — G1 Design Review Board presence

**Question** : le fichier sprint53_design_review.md existe-t-il
et couvre-t-il D1..D4 ?

**Methodes** :
```bash
cat .planning/active/sprint53_design_review.md | head -30
```

**Signal** :
- P1 bloquant si absent (§3 Track G1)
- P2 : review superficielle (< 1 page, pas de scoring D1..D4)

---

## Carries residuels post-S53 (rappel pour l'auditeur)

| Item | Compteur S54 | Source |
|---|---|---|
| P2-A-1 rand blocker upstream | 12+/3 | exemption |
| P2-AUDIT-2 iroh transitives | herite | pin 0.98 |
| P2-REVIEW-B-1-S51 edition 2024 upgrade | 3/3 MANDATORY | re-scoped S53 |
| P2-REVIEW-A-1-S52 nextest timeout | 2/3 | S52 review |
| P2-REVIEW-B-1-S52 Woodpecker E2E | 2/3 | S52 review |
| P2-REVIEW-B-2-S52 GHA 9/9 | 2/3 | S52 review |
| P2-AUDIT-1-S52 images CI pin | 2/3 | S52 audit |
| P2-S53-outbox non-persistant | 1/3 NEW | S53 Phase F review |
| P2-S53-browse_request rate-limit | 1/3 NEW | S53 Phase G review |
| P2-S53-gossip params struct | 1/3 NEW | S53 Phase D review |
| P2-S53-node_key perms 0600 | 1/3 NEW | S53 Phase E review |
| P2-S53-route collision doc | 1/3 NEW | S53 Phase A review |
| P2-S53-periodic republish | 1/3 NEW | S53 Phase F review |

**Attention S54 pair** : 4 items a 2/3 (nextest timeout, Woodpecker
E2E, GHA 9/9, CI image pinning) deviennent 3/3 MANDATORY S55 si
non adresses. P2-REVIEW-B-1-S51 est deja 3/3 MANDATORY.
