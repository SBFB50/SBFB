# Sprint 83 Phase D — Preflight G8

**Phase** : D — Extraction du vrai contrat Factory (`FactoryProcessProfile.v1`).
**Méthode** : Workflow ultracode `wf_cdc0b899-17f` — 7 agents Opus 4.8 1M (5 scans
factuels S1a/S1b/S2/S3/S4 → vérification adversariale → synthèse). 873 099 tokens,
143 tool-calls, 0 erreur.
**Verdict** : **PLAN-ADAPT** (aucun DESIGN-CONFLICT). Signaux de scan : S1a=PLAN-ADAPT,
S1b=EXECUTE, S2=EXECUTE, S3=PLAN-ADAPT, S4=EXECUTE ; adversarial=PLAN-ADAPT ;
synthèse=PLAN-ADAPT.

---

## 1. Rationale du verdict

Aucune décision Day-0 figée n'est en tension. Tout vérifié sur disque :

- census `DOMAIN_*_V1 = 25` GELÉ (compté live ; `check-frontier-contracts.sh:190`) ;
- `contracts.rs` registre `len == 7` (contracts.rs:143) ; `FactoryProcessProfile` = entrée
  4/7 (contracts.rs:105-108), stub `{}` à contracts.rs:58 ;
- `gate-check` reste un diagnostic, JAMAIS une autorité (AGENT_SYSTEM.md:47) ;
- `FactoryProcessProfile.v1` = profil **LOCAL** non-signé (0 `DOMAIN`, 0 `*_FORMAT_VERSION`) ;
  le remplir ne fait que rafraîchir `contracts.schema.json` en gardant `len==7` ;
- AM-3 = amendement in-sprint (ratifié Option A 2026-07-18), pas une Day-0 figée ; sa lettre
  « étiquette = schema drift-gate + `// FRONTIER:` » est réconciliée sans conflit par le
  précédent Phase B DANS le même crate (`accessibility_profile.rs`) ;
- `blake3 = "1.5"` déjà dépendance workspace résolue 1.8.5, utilisée par 5 crates
  (Cargo.toml:70) → **0 nouveau node lock** ; tri-licence CC0-1.0 / Apache-2.0 /
  Apache-2.0-WITH-LLVM-exception toutes allowlistées (deny.toml:104,105,113).

**Pourquoi PLAN-ADAPT et pas EXECUTE pur** : le texte du plan omet des adaptations
matérielles, chacune vérifiée comme un vrai piège sur disque (voir §2). Prior-art OSS concret
qui ancre le design : **in-toto Attestation Statement** `subject:[{name,digest:{blake3}}]`
pour le liage per-source, **OPA Gatekeeper `enforcementAction` (deny vs dryrun/warn)** +
**NIST OSCAL `implementation-status`** pour l'axe `documented|enforced`, **RFC 8785 JCS**
pour l'écueil de concaténation sans délimiteur.

## 2. Adaptations obligatoires (approche corrigée que le code suit)

1. **Cargo.toml** : ajouter `blake3 = { workspace = true }` en `[dependencies]`. Qualifier
   « 0 nouveau node lock » dans la section Pre-launch du body (désamorce un faux flag review
   « new runtime dep »).
2. **Struct** (contracts.rs:58) : garder `#[serde(deny_unknown_fields)]` +
   `#[schemars(title="FactoryProcessProfile.v1")]`, SANS aucune const `DOMAIN_*` ni
   `*_FORMAT_VERSION`. Champs :
   - `sources: BTreeMap<SourceId,{path, blake3 hex}>` — forme in-toto subject[] per-source,
     ordre de clés déterministe, pour que `PROFILE_STALE` nomme QUELLE source a dérivé ;
   - trois buckets SÉPARÉS (D.1) : `process_gates` G1-G10, `publication_gates` FG0-FG10
     **+ FG-CSP-authoring**, `finality_hooks` (lightcheck + auditor-gate) ; chaque règle porte
     `enforcement: Documented|Enforced` + un `mechanism` avec ancre machine-résolvable
     (chemin hook + token grep / symbole gates.rs / nom de script) pour les Enforced ;
   - items descriptifs (9 rôles, 10 modes, preflight S1a-S4, PASS-PENDING, Codex-brut,
     réconciliation, 3 blocs vérif, body 9 sections, carries, audit, wrap-up) en enums/consts
     nommés miroir de l'énumération (README §6.9, pas de littéral magique).
3. **documented|enforced depuis la vérité in-repo** PROCESS.md Guarantee Matrix « Preserved
   By » (:48-62) : **Enforced** = G1 (precommit-lightcheck) + G5 (git hooks+lightcheck) + les
   2 hooks de finalité + FG-CSP-authoring (`run_gate_csp_authoring` non-délégable) ;
   **Documented** = G2/G3/G4/G6/G7/G8/G9/G10 (prompt) ET les 9 rôles + 10 modes (axe
   descriptif — NE PAS inventer un faux Enforced).
4. **extract()/detect_stale() PURES sur bytes INJECTÉS** :
   `extract(sources:&[(SourceId,&[u8])]) -> FactoryProcessProfile` et
   `detect_stale(profile,&current) -> Result<(),ProfileStale{source_id}>`. AUCUN `std::fs` en
   code de PRODUCTION (charte lib.rs:8-10) — le **test** fait `fs::read` depuis
   `CARGO_MANIFEST_DIR/../..`. Avant blake3 : canonicaliser chaque source (strip `\r` + BOM
   UTF-8 de tête). Hacher CHAQUE source séparément (map per-source), JAMAIS une concaténation
   `name+content` (idiome repo `template_lock.rs`/`provenance.rs` sans délimiteur = écueil).
5. **PROFILE_STALE fail-closed** : `detect_stale` recalcule blake3 sur les bytes courants
   canonicalisés, compare à la map embarquée, renvoie `Err(ProfileStale{source_id})` au 1er
   mismatch — miroir du binding S81 K « loaded-stage ↔ manifeste signé ». Diagnostic renvoyé à
   l'appelant, JAMAIS câblé comme autorité de verdict process au runtime.
6. **Deux goldens** : (a) rafraîchir `contracts.schema.json` via
   `UPDATE_SNAPSHOTS=1 cargo test -p sbfb-workflow-core schema_snapshot_stub` (self-refresh,
   `len` reste 7) ; (b) NOUVEAU golden d'instance `factory_profile.golden.json` en mode
   **STRICT** copié sur `accessibility_profile.rs:823-831` (golden manquant/vide = HARD FAIL,
   seul `UPDATE_SNAPSHOTS` écrit) — PAS le stub self-write fail-open (contracts.rs:186-196),
   sinon supprimer le golden l'auto-guérit et PROFILE_STALE devient défaisable.
7. **Étiquette FRONTIER (AM-3)** EXACTEMENT comme le frère Phase B
   (`accessibility_profile.rs:341-347`) : commentaire de provenance en PROSE au-dessus de la
   struct, JAMAIS la forme colon+espace `// FRONTIER: FactoryProcessProfile` (vérifié live :
   elle matche le grep du gate → malformed/census). Ne pas toucher le tag anti-silent-removal
   `// FRONTIER: ShardPlan `.
8. **Docs-contrat** : ne toucher AUCUN fichier `docs/factory/` ni `llms.txt` (clôture
   GUIDE+llms.txt de ProfileV1 déférée à AJ1 par AM-3) — `check-factory-docs.sh` reste vert
   tel quel. L'étiquette per-phase = commentaire prose + les 2 goldens.
9. **Tests non-vacuous** : `extract_matches_golden` (golden STRICT) ;
   `profile_stale_on_source_digest_mismatch` (muter UN byte source ET/OU un digest embarqué →
   `Err(ProfileStale{source_id})` au bon id + `Ok` sur inchangé = rouge-avant/bloc-après) ;
   `documented_vs_enforced_marked` (pour CHAQUE règle Enforced, RÉSOUDRE son ancre mécanisme
   contre le fichier réel — grep token hook / symbole gates.rs / script — et prouver qu'une
   ancre inexistante passe rouge ; §P72 anti-score / gate-discriminant). Assertion déterminisme
   variante CRLF.
10. **Gate dual-platform** : `cargo nextest run -p sbfb-workflow-core --locked` exit 0 Win natif
    ET Docker sbfb-ci ; `check-frontier-contracts.sh` exit 0 (census 25) ;
    `check-factory-docs.sh` exit 0 ; T1 = 0 changement `web/`.

## 3. Faits réfutés par l'adversarial (à ne PAS sur-corriger)

- **README non dans le digest set = blind spot** → RÉFUTÉ (sur-affirmé) : chaque item extrait
  a sa source autoritaire DANS le digest set (G1-G10=PROCESS.md:48-62, 10 modes=AGENT_SYSTEM.md
  :82-96, FG=FACTORY_GATES.md:35-284). README redondant, pas source unique.
- **Faux PROFILE_STALE cross-OS CRLF/BOM par défaut** → probabilité sur-affirmée : `.gitattributes`
  force `eol=lf` sur toutes les extensions du digest set (`*.rs/*.md/*.sh/*.toml/*.json`). Risque
  résiduel = BOM isolé / chemin non couvert → la canonicalisation LF/BOM reste une
  défense-en-profondeur légitime (adaptation 4), mais ne pas la présenter comme le défaut.

## 4. Invariants à tenir (commande de vérif par invariant)

- census `DOMAIN_*_V1 == 25` : `bash scripts/check-frontier-contracts.sh` exit 0.
- registre `len==7` : `cargo nextest run -p sbfb-workflow-core every_contract_emits_nonempty_draft_2020_12_object` exit 0.
- 0 `DOMAIN` neuf : `grep -rn 'const DOMAIN_' crates/sbfb-workflow-core/src/` → vide.
- 0 `*_FORMAT_VERSION` neuf : `grep -rnE 'const [A-Z_]*_FORMAT_VERSION' crates/sbfb-workflow-core/src/` → vide.
- pas de tag FRONTIER gate-parsé ProfileV1 : `grep -rnE '// FRONTIER: FactoryProcessProfile' crates/sbfb-workflow-core/src/` → vide.
- blake3 0 node lock : `git diff Cargo.lock` ne change aucune version blake3.
- docs-contrat intact : `git diff --name-only | grep -E '^docs/factory/|llms.txt'` → vide + `check-factory-docs.sh` exit 0.
- golden STRICT (défaisabilité fermée) : nouveau test échoue sur golden manquant/vide (pattern accessibility_profile.rs:823-831).
- T1 : `git diff --name-only | grep -E '^web/'` → vide.
- gate dual-platform : nextest Win natif ET Docker sbfb-ci exit 0.

## 5. Open questions

Aucune (`open_questions: []`). Travail de remplissage cadré et ratifié.
