# Sprint 4 verification prompt

**À coller dans une session Claude fraîche ouverte dans
`C:\Users\FlowUP\Documents\Code\nexus`**. Ce document est
volontairement self-contained : aucune lecture d'historique de
conversation n'est nécessaire, aucun contexte implicite n'est
supposé. L'objectif est de confirmer que Sprint 4 est intact sur
la branche actuelle, sans rien modifier.

---

## Mission

Tu es un vérificateur. Ton rôle est d'exécuter les commandes du
tableau fail-fast ci-dessous, capturer la sortie, comparer au
critère, et produire un rapport final. Tu ne corriges **rien** —
si une ligne échoue, tu la marques `FAIL` avec la sortie brute et
tu continues la vérification jusqu'au bout. Tu ne commits pas,
tu ne pushes pas, tu ne touches pas au code.

Tu n'as pas le droit de deviner. Si une commande n'est pas
installée sur la machine (cargo, uv, pytest, ruff), tu le signales
et tu marques la ligne `SKIP: tool missing`.

## État attendu à l'entrée

- Working tree clean (`git status --short` ne doit rien afficher
  d'inattendu). Les fichiers `.planning/audit_sprint2/` sont
  déjà gitignorés, donc leur présence est normale et attendue.
- HEAD sur `master`, 9 commits au-dessus de `f68d997` (Sprint 3
  verification checklist). Le commit de tête doit contenir
  "Sprint 4" dans son message.
- Les crates Rust `nexus-core-rs`, `nexus-core-py`,
  `nexus-worker-core`, `nexus-worker` existent dans
  `crates/`.
- Les packages Python `nexus-coordinator`, `nexus-sdk`,
  `nexus-app-gov` existent dans `packages/`.
- L'exemple `hello-world-app` existe dans `examples/`.
- Le document `.planning/sprint4_verification.md` existe et liste
  17 lignes de critères (c'est le bilan que tu re-exécutes ici).

## Prérequis d'environnement

- `cargo` (rustup 1.29+, cargo 1.94+) accessible via
  `~/.cargo/bin/cargo` — sur Windows bash, utiliser
  `export PATH="$HOME/.cargo/bin:$PATH"`.
- `uv` accessible via
  `/c/Users/FlowUP/AppData/Local/Microsoft/WinGet/Links/uv` sur
  Windows — utiliser
  `export PATH="/c/Users/FlowUP/AppData/Local/Microsoft/WinGet/Links:$PATH"`.
- Python 3.13 via le venv workspace `.venv/` (uv gère
  automatiquement).
- Le workspace doit être synchronisé avant les tests Python :
  `uv sync --package nexus-coordinator --extra test` une seule
  fois en début de run (installe nexus-core-py wheel via maturin +
  nexus-coordinator + nexus-sdk + nexus-app-gov + hello-world).

## Commande d'échauffement (à exécuter avant la checklist)

```bash
export PATH="$HOME/.cargo/bin:/c/Users/FlowUP/AppData/Local/Microsoft/WinGet/Links:$PATH"
cargo --version
uv --version
git status --short
git log --oneline -12
```

Attendu :
- `cargo 1.94` ou supérieur
- `uv 0.11` ou supérieur
- `git status --short` vide (sauf `.planning/audit_sprint2/`
  éventuellement, qui est gitignoré)
- `git log --oneline -12` montre les 9 commits Sprint 4 au-dessus
  de `f68d997`, commit de tête `9f71c70 docs(sprint4):
  verification checklist + W9.1 doc cleanup` (ou successeur
  si de nouveaux commits ont été ajoutés — dans ce cas, vérifie
  que les 9 commits Sprint 4 sont bien dans l'historique)

Si l'un de ces échoue, **arrête** et signale l'état au user avant
de continuer — la vérification n'a pas de sens sur un
environnement cassé.

---

## Tableau fail-fast (17 lignes)

Chaque ligne = 1 commande à exécuter, 1 critère à vérifier. Tu
exécutes dans l'ordre, tu captures stdout+stderr, tu compares au
critère. Aucune commande ne dépend des suivantes sauf mention
explicite.

| # | Check | Commande | Critère |
|---|---|---|---|
| 1 | canonical.rs JCS + domain prefix | `cargo test -p nexus-core-rs --lib canonical::tests` | `test result: ok. 4 passed` |
| 2 | GossipClient owned (no lifetime) | `grep -n "GossipClient<'" crates/nexus-core-rs/src/gossip.rs` | exit code 1 (grep found nothing) |
| 3 | DocsClient owned (no lifetime) | `grep -n "DocsClient<'" crates/nexus-core-rs/src/docs.rs` | exit code 1 (grep found nothing) |
| 4 | sign_claim + mint_invite exposed | `uv run python -c "import nexus_core; assert hasattr(nexus_core, 'sign_claim'); assert hasattr(nexus_core, 'mint_invite'); assert hasattr(nexus_core, 'decode_invite'); print('OK')"` | prints `OK` |
| 5 | Coordinator boot tests | `uv run --package nexus-coordinator pytest packages/nexus-coordinator/tests/test_coordinator_boot.py -q` | `3 passed` |
| 6 | Dispatcher tests | `uv run --package nexus-coordinator pytest packages/nexus-coordinator/tests/test_dispatcher.py -q` | `3 passed` |
| 7 | Full-loop test (dispatcher→validator→kudos) | `uv run --package nexus-coordinator pytest packages/nexus-coordinator/tests/test_full_loop.py -q` | `1 passed` |
| 8 | Kudos chain integrity | `uv run --package nexus-coordinator pytest packages/nexus-coordinator/tests/test_kudos_hash_chain.py -q` | `5 passed` |
| 9 | Invite v2 Python roundtrip | `uv run --package nexus-coordinator pytest packages/nexus-coordinator/tests/test_invite.py -q` | `6 passed` |
| 10 | Invite v1 hard-refused (Rust) | `cargo test -p nexus-worker-core --lib invite::tests::decode_refuses_v1` | `1 passed` |
| 11 | hello-world-app LOC < 100 | `wc -l examples/hello-world-app/src/hello_world_app/*.py` | total ≤ 100 lines (expected 45) |
| 12 | Gov manifest via /app endpoint | `uv run --package nexus-coordinator pytest packages/nexus-coordinator/tests/test_apps.py -q` | `2 passed` |
| 13 | No residual TODO(W9.1) | `grep -c 'TODO(W9.1)' crates/nexus-worker-core/src/engine/runtime.rs` | `0` |
| 14 | W9.1 claim→execute→result integration | `cargo test -p nexus-worker-core --lib engine::runtime::tests::engine_claims_and_executes_tasks_on_registered_doc` | `1 passed` |
| 15 | Format (Rust + Python) | `cargo fmt --all --check && uv run ruff format --check packages/ examples/` | both exit 0 |
| 16 | Rust workspace tests | `cargo test -p nexus-core-rs --lib && cargo test -p nexus-worker-core --lib && cargo test -p nexus-worker --test e2e` | core-rs ≥ 62, worker-core ≥ 94, worker e2e ≥ 10 |
| 17 | Python coordinator full suite | `uv run --package nexus-coordinator pytest packages/nexus-coordinator/tests/ -q` | `27 passed, 1 skipped` (1 skip = Windows POSIX perms) |

### Lignes spéciales

- **Ligne 2 et 3** : `grep` retourne exit code 1 quand rien ne
  matche. Si tu utilises un outil qui affiche "No matches found",
  c'est le succès attendu.
- **Ligne 13** : même principe — 0 matches = succès.
- **Ligne 11** : le critère `≤ 100` est inclusif. Si la ligne
  affiche 45 (attendu), c'est green. Si > 100, c'est fail.
- **Ligne 4** : l'import `nexus_core` échoue si le wheel n'est
  pas installé. Si c'est le cas, lance `uv sync --package
  nexus-coordinator --extra test` **une seule fois**, puis
  retente la ligne. Si l'échec persiste, marque FAIL.

### Tests SDK + gov (si budget)

Les deux sous-crates testent via un changement de répertoire à
cause d'une collision de noms `tests/` — pas dans le tableau
principal mais recommandé pour un audit complet :

```bash
cd packages/nexus-sdk && uv run --project ../../ pytest tests/ -q
# attendu: 6 passed
cd ../nexus-app-gov && uv run --project ../../ pytest tests/ -q
# attendu: 3 passed
cd ../..  # back to repo root
```

Ce sont des bonus — les lignes du tableau principal sont
prioritaires.

---

## Mode d'emploi

1. Lance l'échauffement ci-dessus. Vérifie `git status` clean et
   `git log -12` conforme.
2. Exécute les 17 lignes du tableau dans l'ordre. Pour chaque
   ligne, capture la sortie et marque `PASS` / `FAIL` / `SKIP` (si
   outil manquant).
3. **Ne corrige rien.** Même si une ligne échoue de façon
   évidente (ex: un fichier manquant), tu ne modifies aucun code.
   Ton rôle est exclusivement de rapporter.
4. À la fin, imprime un tableau récapitulatif de la forme :

   ```
   Sprint 4 verification — 2026-MM-DD
   HEAD: <short hash> <subject>

   | # | Check                          | Status |
   |---|--------------------------------|--------|
   | 1 | canonical.rs JCS + domain      | PASS   |
   | 2 | GossipClient owned             | PASS   |
   | ...                                         |
   | 17| Python coordinator full suite  | PASS   |

   Totaux: X PASS / Y FAIL / Z SKIP
   ```

5. Si une ligne est FAIL, joins la sortie brute (max 20 lignes
   par échec) en annexe sous le tableau, avec le format :

   ```
   ### FAIL row N — <check name>
   Command: <commande>
   Expected: <critère>
   Observed:
   ```
   <stdout/stderr brut>
   ```
   ```

6. Termine par un résumé d'une phrase : "Sprint 4 est intact"
   (tout PASS ou SKIP tool-missing) ou "Sprint 4 a régressé sur
   N ligne(s)".

---

## Ce que tu ne dois PAS faire

- Ne pas écrire de code.
- Ne pas faire de commit, ni de push, ni de rebase, ni de reset.
- Ne pas "fixer au passage" un warning ou une dette. Si tu en
  vois, mentionne-les dans le résumé mais laisse-les tels quels.
- Ne pas relancer une ligne en FAIL avec un flag différent pour
  "essayer de la faire passer". Si c'est rouge, c'est rouge.
- Ne pas réimporter les tests Python sans `uv run` — tous les
  tests Python doivent passer par `uv run` pour que le venv
  workspace soit actif.
- Ne pas lancer `uv sync` avec un extra différent de `--extra test`
  sans justification — la checklist repose sur ce set de deps.
- Ne pas faire de bench, pas de perf check, pas de profiling. La
  vérification est fonctionnelle uniquement.

## En cas de doute

Si une ligne est ambiguë (ex: le test pytest remonte
`5 passed, 1 warning`, le critère dit `5 passed`), considère-la
`PASS` — le critère est un plancher, pas un plafond strict. Les
warnings ne sont pas des échecs.

Si tu ne sais pas si un ajout/suppression de test est voulu
(ex: `94 passed` observé vs `≥ 94` attendu), vérifie
`git log --oneline -12 -- crates/nexus-worker-core/src/` et
lis les messages de commit pour tracer l'évolution. Les lignes
"≥ X" du tableau tolèrent les ajouts de tests ultérieurs.

## Référence pour les attendus exacts

Le fichier `.planning/sprint4_verification.md` contient le même
tableau avec les résultats observés lors de la clôture du
Sprint 4. Lis-le si tu veux voir ce que "green" ressemble côté
sortie réelle. Ne le modifie pas.

## Référence pour le scope

Le plan Sprint 4 complet est dans `.planning/sprint4_plan.md`
(9 sections, 8 décisions actées). Le kickoff original est dans
`.planning/sprint4_kickoff.md` (contexte mission + règles
opérationnelles). Les deux sont lisibles sans cargo/uv.

---

**Premier message attendu de la session de vérification** :

> Je vais vérifier Sprint 4 sur la HEAD actuelle. Je lance
> d'abord l'échauffement (git status, git log, cargo version, uv
> version), puis j'exécute les 17 lignes du tableau fail-fast
> dans l'ordre. Je ne toucherai à aucun fichier.
