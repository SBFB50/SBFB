**Verdict Codex**
Je ne donnerais pas un PASS strict avant commit. 3 livrables sont confirmés, 2 sont partiels. Le point bloquant est dans `main.rs` : un échec `scan-secrets` peut appeler `process::exit(1)` avant l’écriture audit log, donc “log après chaque subcommand” n’est pas vrai sur ce chemin.

**Contexte Vérifié**
- Plan Phase A : `.planning/active/sprint69_plan.md:64-81`.
- Preflight G8 : `.planning/active/sprint69_phase_A_preflight.md:1-3` = `EXECUTE plan-as-is`.
- Review actuelle : `.planning/active/sprint69_phase_A_review.md:5-8` = `PASS-PENDING`, non committable avant Codex/reconciliation.
- Working tree lu via `git -c safe.directory=... status --short` : `preview.rs`, `main.rs`, `THREAT_MODEL.md` modifiés ; `audit_log.rs`, `count-tests.sh`, preflight/review untracked.

**1. audit_log.rs**
Statut : **CONFIRME**.

Evidence :
- `AuditEntry` contient `timestamp`, `command`, `args`, `result` : `crates/sbfb-factory/src/audit_log.rs:8-14`.
- `audit_log_path()` utilise `directories::BaseDirs` et vise `~/.sbfb/factory-audit.log` : `crates/sbfb-factory/src/audit_log.rs:16-19`.
- `log_entry()` délègue à `log_entry_to(&audit_log_path(), entry)` : `crates/sbfb-factory/src/audit_log.rs:22-23`.
- `log_entry_to(path, entry)` crée le parent, ouvre en append, sérialise JSON, écrit JSONL : `crates/sbfb-factory/src/audit_log.rs:26-36`.
- `audit_log_writes_jsonl` utilise `tempfile` + `log_entry_to()` : `crates/sbfb-factory/src/audit_log.rs:53-64`.
- `audit_log_appends` appelle `log_entry_to()` deux fois et vérifie deux lignes : `crates/sbfb-factory/src/audit_log.rs:67-84`.

Note : le plan live mentionne encore `gates_results` dans `AuditEntry` à `.planning/active/sprint69_plan.md:77`, mais ta mission le marque différé Phase B. Non bloquant si le commit body documente explicitement ce delta.

**2. main.rs**
Statut : **PARTIEL**.

Evidence conforme :
- `mod audit_log;` ajouté : `crates/sbfb-factory/src/main.rs:6`.
- `main()` capture `(cmd_name, cmd_args, result)` : `crates/sbfb-factory/src/main.rs:79-117`.
- Timestamp RFC3339 via `time::OffsetDateTime::now_utc().format(Rfc3339)` : `crates/sbfb-factory/src/main.rs:124-127`.
- `AuditEntry` construit puis `let _ = audit_log::log_entry(&entry);` : `crates/sbfb-factory/src/main.rs:124-132`.
- Le résultat de commande reste traité après logging : `crates/sbfb-factory/src/main.rs:134-137`.

Gaps :
- **Bloquant avant PASS strict** : `run_scan_secrets()` peut quitter le process avant retour à `main()` si `!result.passed` : `crates/sbfb-factory/src/main.rs:147-154`. Dans ce cas, `audit_log::log_entry()` à `main.rs:132` n’est jamais appelé. Impact : un échec `scan-secrets` n’est pas audité, alors que c’est probablement un des chemins les plus importants à tracer.
- Gap mineur : `Create` ne logge pas `--output` si fourni. Les args loggés sont seulement `--template` et `--name` : `crates/sbfb-factory/src/main.rs:81-90`. Impact : audit incomplet/reproductibilité partielle, non bloquant seul.

**3. preview.rs**
Statut : **CONFIRME**.

Evidence :
- `MAX_PREVIEW_ENTRIES = 10` : `crates/nexus-shell-daemon-core/src/preview.rs:17-19`.
- `load()` tient un write guard avant check + insert : `crates/nexus-shell-daemon-core/src/preview.rs:54-61`.
- Check correct : `guard.len() >= MAX_PREVIEW_ENTRIES && !guard.contains_key(&hash_hex)` : `crates/nexus-shell-daemon-core/src/preview.rs:55`.
- Variant `PreviewError::TooManyEntries` : `crates/nexus-shell-daemon-core/src/preview.rs:93-100`.
- Test rejet 11e entrée : `crates/nexus-shell-daemon-core/src/preview.rs:158-166`.
- Test reload même hash quand plein : `crates/nexus-shell-daemon-core/src/preview.rs:169-178`.
- Test acceptation après eviction : `crates/nexus-shell-daemon-core/src/preview.rs:181-191`.

S3 : race condition non vue dans `PreviewStore::load()` : le write guard est acquis à `preview.rs:54` et conservé jusqu’après l’insert à `preview.rs:61`.

**4. THREAT_MODEL.md**
Statut : **CONFIRME**.

Evidence :
- §13 insérée : `docs/security/THREAT_MODEL.md:658`.
- Surface preview décrite : `docs/security/THREAT_MODEL.md:660-664`.
- `T-PREVIEW-EXHAUSTION` présent : `docs/security/THREAT_MODEL.md:666-669`.
- Vecteurs volume d’entries + taille max : `docs/security/THREAT_MODEL.md:671-677`.
- Mitigations `MAX_PREVIEW_BYTES`, `MAX_PREVIEW_ENTRIES`, TTL, loopback, bearer : `docs/security/THREAT_MODEL.md:679-690`.
- Ancienne revue renommée §14 : `docs/security/THREAT_MODEL.md:701`.
- Historique v6 ajouté : `docs/security/THREAT_MODEL.md:716-730`.

**5. scripts/count-tests.sh**
Statut : **PARTIEL**.

Evidence conforme :
- Fichier présent avec shebang bash : `scripts/count-tests.sh:1`.
- Parse nextest : `scripts/count-tests.sh:7-12`.
- Parse doctests : `scripts/count-tests.sh:15-18`.
- Parse Vitest : `scripts/count-tests.sh:21-24`.
- Summary combiné : `scripts/count-tests.sh:27-31`.

Gaps :
- Le script avale les échecs avec `|| true` sur nextest, doctests et Vitest : `scripts/count-tests.sh:8`, `scripts/count-tests.sh:16`, `scripts/count-tests.sh:22`. Impact : utilisable comme compteur, pas comme gate fiable.
- Le total combiné exclut les doctests : `scripts/count-tests.sh:28-31`. Impact : partiel par rapport à “Rust nextest, doctests, Vitest, total combine”. Non bloquant si le commit body copie les résultats réels vérifiés manuellement, mais à ne pas présenter comme preuve de gate.

**Sécurité S1-S4**
- S1 path traversal audit log : **OK**. `audit_log_path()` ne prend aucune entrée utilisateur et construit le chemin via `BaseDirs` : `audit_log.rs:16-19`. Caveat : fallback relatif `factory-audit.log` à `audit_log.rs:19`, pas traversal, mais pas strictement `~/.sbfb`.
- S2 secret/credential : **OK code Phase A**. Scan statique sur fichiers Phase A : pas de clé/token hardcodé. Les hits sont noms de module/doc (`ScanSecrets`, bearer dans threat model), pas secrets.
- S3 MAX_PREVIEW_ENTRIES : **OK**. Write guard tenu pendant check+insert : `preview.rs:54-61`.
- S4 fire-and-forget : **PARTIEL/OK design**, car `let _ = audit_log::log_entry(&entry)` masque seulement l’échec d’écriture audit log : `main.rs:132`; le résultat métier est encore traité ensuite : `main.rs:134-137`. Mais le gap `scan-secrets` ci-dessus est distinct et bloquant pour “log après chaque subcommand”.

**Commandes**
- `cargo nextest run -p nexus-shell-daemon-core -E "test(preview)" --locked` : non confirmé dans ce sandbox, échec avant tests sur `target/debug/.cargo-lock` avec `Accès refusé`.
- Relance avec `CARGO_TARGET_DIR=C:\tmp\codex-nexus-target` pour `sbfb-factory audit` : aussi bloquée par `Accès refusé` sur le target temporaire.
- Checks statiques équivalents confirmés : `MAX_PREVIEW_ENTRIES` apparaît 6 fois, `Preview ephemere` présent, `scripts/count-tests.sh` présent.

**Résumé Final**
Total livrables : 5
Confirmés : 3
Partiels : 2
Gaps : 0

Bloquant avant commit PASS strict : **oui**, `main.rs` doit garantir l’audit log aussi sur le chemin `scan-secrets` échoué, ou ce scope cut doit être explicitement accepté. À mon avis, comme le livrable dit “après chaque subcommand”, il faut corriger avant promotion `PASS`.
