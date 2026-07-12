Verdict global : **Cible 1 = PARTIEL** ; **Cible 2 = GAP**. Le worktree est resté propre et `HEAD=e7ff73c`.

## Cible 1 — `e7ff73c` : PARTIEL

| Contrôle | Verdict | Preuve / conclusion |
|---|---|---|
| 1. Cohérence commande | **OK** | La commande canonique emploie partout `-m gpt-5.6-sol -c model_reasoning_effort=max` : [README.md:1064](/C:/Users/FlowUP/Documents/Code/nexus/docs/claude/README.md:1064), tableau [README.md:1074](/C:/Users/FlowUP/Documents/Code/nexus/docs/claude/README.md:1074), bootstrap [README.md:2352](/C:/Users/FlowUP/Documents/Code/nexus/docs/claude/README.md:2352), [AGENT_SYSTEM.md:66](/C:/Users/FlowUP/Documents/Code/nexus/docs/agent/AGENT_SYSTEM.md:66), [PROVIDER_CONFIG.md:10](/C:/Users/FlowUP/Documents/Code/nexus/docs/agent/PROVIDER_CONFIG.md:10). `xhigh` n’apparaît qu’en option inférieure descriptive, jamais comme valeur prescrite. |
| 2. Complétude canon courant | **OK** | Le grep demandé ne laisse qu’un `GPT 5.5` dans le changelog historique Sprint 65, [README.md:2994](/C:/Users/FlowUP/Documents/Code/nexus/docs/claude/README.md:2994). Aucun canon forward-looking ne prescrit encore 5.5. Les occurrences dans `SPRINT_LOG` et `.planning/**` enregistrent bien des exécutions passées. |
| 3a. Slug exact | **OK** | Le catalogue Codex courant donne explicitement `codex -m gpt-5.6-sol` et les efforts Low→Ultra, dont Max. [Documentation officielle des modèles Codex](https://developers.openai.com/codex/models). |
| 3b. Alias invalides | **OK** | Rejoués indépendamment avec CLI `0.144.1` : `solar`, `sol-pro` et `codex-sol` retournent chacun exit 1 + HTTP 400 « model … not supported when using Codex with a ChatGPT account ». |
| 3c. CLI `>=0.144.1` | **PARTIEL** | Cette machine établit seulement : `0.142.5` échoue et `0.144.1` réussit. Cela prouve que l’upgrade vers `0.144.1` a résolu le cas local, mais pas que `0.144.1` est mathématiquement la version minimale — aucune preuve n’exclut une version intermédiaire. |
| 3d. Scope `-m` / `-c` | **OK** | Le CLI décrit `-m` comme override du modèle « for this run » et `-c` comme override prioritaire « for that invocation » ; ils ne réécrivent pas `config.toml`. [Référence officielle CLI](https://developers.openai.com/codex/cli/reference). Le `config.toml` local est resté daté du 05/07 après les probes. |
| Disponibilité / formulation Pro | **PARTIEL** | L’accès de **ce compte** à Sol est prouvé. En revanche, « tier flagship inclus dans l’offre Pro » généralise trop : la page preview officielle indique encore qu’un abonnement payant seul ne garantit pas l’accès. Les sources OpenAI sont actuellement discordantes avec le catalogue Codex plus récent ; la formulation sûre est « accès vérifié sur ce compte ». [Éligibilité preview officielle](https://help.openai.com/en/articles/20001325-a-preview-of-gpt-56-sol-terra-and-luna). |
| Date « sorti le 09/07 » | **PARTIEL** | L’annonce publique OpenAI est datée du 26 juin 2026 ; le 09/07 peut être une date de rollout local, mais n’est pas établie comme date de sortie générale. [Annonce OpenAI](https://openai.com/index/previewing-gpt-5-6-sol/). |
| 4. Forme CLI | **OK** | `codex-cli 0.144.1 --help` confirme `exec`, `-m/--model`, `-c/--config`, `--dangerously-bypass-approvals-and-sandbox`, stdin et `-o/--output-last-message`. La commande documentée est syntaxiquement valide. |
| 5. Hook/templates | **OK** | Le hook ne nomme aucun modèle ; il exige seulement l’artefact brut, [phase-precommit-lightcheck.sh:289](/C:/Users/FlowUP/Documents/Code/nexus/.claude/hooks/phase-precommit-lightcheck.sh:289). Les trois templates Codex ne contiennent ni slug ni effort, par exemple [codex_phase_review.txt:1](/C:/Users/FlowUP/Documents/Code/nexus/.claude/templates/codex_phase_review.txt:1). |
| 6. Code/scripts | **OK** | `rg -i "gpt-5|GPT 5" crates scripts web` retourne zéro occurrence. |

## Cible 2 — `bb6c4f9` : GAP

### 1. SI-9 / liveness — OK

`drive_hop` place bien `open_bi`, `write_frame` et `read_frame` dans **un seul** `tokio::time::timeout`, [shard_session.rs:735](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:735). Le scénario byzantin qui accepte le stream sans lire est exercé avec une frame de 64 MiB, [shard_session.rs:1657](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:1657).

La readiness handshake+RTT partage également un timeout unique, [shard_session.rs:591](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:591). Le re-dial ultérieur est borné, [shard_session.rs:884](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:884), tout comme la re-readiness du fallback, [shard_session.rs:936](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:936).

### 2. Zéro bump wire — OK

Les blobs Git sont identiques entre `12e3954` et `bb6c4f9` :

- `shard_plan.rs` : `0f74dbb…` aux deux révisions.
- `compute_group.rs` : `512e62e…` aux deux révisions.
- `shard.rs` : `c2a871b…` aux deux révisions.

Les constantes restent `1` dans [shard_plan.rs:77](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/shard_plan.rs:77) et l’ALPN reste `sbfb/shard/1`, [node.rs:82](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/node.rs:82).

Les nouveaux DTO/schémas sont explicitement des projections loopback HTTP, non des payloads signés, [schemas/shard.rs:108](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/schemas/shard.rs:108).

Réserve hors wire : les champs `Option` sont requis dans les snapshots mais le JSON Schema ne permet pas explicitement `null`, alors que les réponses pending sérialisent `null`. Le snapshot n’est donc pas fidèle à toutes les réponses runtime.

### 3. Privacy SI-3/SI-4 — GAP

Les chemins normaux sont corrects :

- status/result ne projettent aucun membre : [shard_session.rs:405](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:405), [http.rs:2154](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:2154), [http.rs:2381](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:2381) ;
- le `Debug` du registre imprime seulement le compte, [shard_session.rs:319](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:319) ;
- les identités explicitement formatées sont tronquées à `[..8]`, par exemple [shard_session.rs:360](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:360).

Mais le diagnostic transport n’est pas assaini : `open_bi failed: {e}` est conservé tel quel, [shard_session.rs:737](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:737), puis loggé et stocké, [shard_session.rs:840](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:840), et enfin projeté dans `failure`, [shard_session.rs:417](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:417).

Or `open_bi` retourne directement un `ConnectionError`, [noq connection.rs:981](/C:/Users/FlowUP/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/noq-1.0.1/src/connection.rs:981), et une fermeture QUIC distante affiche la raison fournie par le pair, [noq-proto frame.rs:961](/C:/Users/FlowUP/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/noq-proto-1.0.1/src/frame.rs:961). Un worker byzantin peut donc placer une pubkey complète dans sa raison de fermeture, qui fuit ensuite dans le log et `/result`.

### 4. Gate d’insertion — OK

`insert_gated` appelle obligatoirement `gate_session` avant l’accès à la map privée, [shard_session.rs:331](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:331).

Le gate vérifie :

- signature manifest `DOMAIN_SHARD_PLAN_V1` ;
- signature compute-group ;
- binding `group_id` ;
- binding `initiator` ;
- pipeline contigu ;
- membership et adresse de chaque worker et fallback.

Les signatures utilisent effectivement leurs domaines canoniques : [shard_plan.rs:359](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/shard_plan.rs:359), [compute_group.rs:221](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/compute_group.rs:221). Aucun autre chemin de production ne peuple le registre ; l’unique appel est après readiness dans `mount_session`, [shard_session.rs:699](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:699).

### 5. Duress — OK

Les trois handlers court-circuitent avant signature ou réseau :

- group : [http.rs:2214](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:2214) ;
- mount : [http.rs:2263](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:2263) ;
- generate : [http.rs:2320](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:2320).

Le test dédié passe également.

### 6. Zéro dépendance — OK

Le diff ne contient aucun `Cargo.toml`, `Cargo.lock` ou `package.json`.

### 7. Concurrence — PARTIEL

Le cœur demandé tient : le check-and-set `Generating` est atomique sous le mutex, puis la garde est relâchée avant le premier `await`, [shard_session.rs:781](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:781). Aucun lock n’est tenu à travers un `await`. Le test dédié passe, [shard_session.rs:1754](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:1754).

Deux défauts restent :

1. Au début d’un nouveau drive, seul `status` est modifié ; `outcome` et `failure` précédents ne sont pas effacés, [shard_session.rs:788](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:788). Un échec ne supprime pas l’ancien succès, [shard_session.rs:467](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:467), et un succès ne supprime pas l’ancienne erreur, [shard_session.rs:831](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:831). `/result` peut donc retourner immédiatement l’ancien texte et l’ancien RunProof pendant une nouvelle génération — faux vert direct pour la boucle de polling du harness.

2. Le handler HTTP fait seulement un précheck d’existence, spawn, puis répond 202 ; l’erreur `already generating` du task est ignorée, [http.rs:2346](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/http.rs:2346). Deux requêtes simultanées peuvent donc recevoir toutes deux `accepted:true`, même si une seule drive réellement.

### 8. Teardown / connexions — OK

Les connexions consommées sont retirées du pool, rangées dans `used`, puis toutes fermées ; les connexions non consommées le sont également, [shard_session.rs:801](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:801).

Le primaire en échec est fermé à [shard_session.rs:910](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:910), le fallback en échec à [shard_session.rs:946](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:946). Aucun handle consommé ne retourne dans `record.conns`.

### Invariant transversal RunProof — GAP

Sur un reroutage réussi, le fallback traite réellement la frame, mais `RunProof.participants` est construit depuis les assignments primaires originaux, [shard_session.rs:985](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-shell-daemon/src/shard_session.rs:985). Le primaire tombé est donc signé comme participant et le fallback réel est omis, alors que le contrat définit `participants` comme les travailleurs effectivement impliqués, [shard_plan.rs:456](/C:/Users/FlowUP/Documents/Code/nexus/crates/nexus-core-rs/src/shard_plan.rs:456). C’est une attestation signée factuellement fausse.

### Vérifications exécutées

- `cargo test -p nexus-shell-daemon shard_session --locked -- --nocapture` : **18 pass, 0 fail**.
- `cargo test -p nexus-core-rs schemas::shard --locked -- --nocapture` : **6 pass, 0 fail**.
- Worktree final : propre.

## GAPs — Cible 1

- **P0 :** aucun.
- **P1 :** aucun.
- **P2 :** « inclus dans l’offre Pro » n’est pas une règle générale suffisamment établie ; seul l’accès de ce compte est prouvé.
- **P3 :** `>=0.144.1` présenté comme minimum exact alors que seules `0.142.5` et `0.144.1` ont été discriminées ; date « sortie 09/07 » non étayée comme date publique.

## GAPs — Cible 2

- **P0 :** aucun.
- **P1 :** RunProof faux après fallback : participants primaires signés au lieu du fallback réellement exécutant.
- **P1 :** état/resultat stale entre deux générations, permettant un ancien résultat/RunProof de satisfaire immédiatement un nouveau polling.
- **P1 :** raison QUIC distante non assainie, capable de faire fuiter une identité complète dans `failure` et le tracing.
- **P2 :** deux POST `generate` concurrents peuvent tous deux répondre `accepted:true`, l’erreur du task perdant étant ignorée.
- **P2 :** le churn du harness n’est pas réellement testé : `drop-shard` est appelé après obtention du résultat, sans nouvelle génération ni assertion de failover, [b3_shard_pipeline.sh:338](/C:/Users/FlowUP/Documents/Code/nexus/scripts/acceptance/b3_shard_pipeline.sh:338).
- **P2 :** les JSON Schemas requis n’acceptent pas explicitement les `null` réellement émis par les champs `Option` pending.
- **P3 :** prose résiduelle périmée dans le harness (« aucun orchestrateur ») et dans le commentaire du status view (« two fields »).
