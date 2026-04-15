# Sprint 18 Phase E1 — nexus-phase-auditor review

**HEAD pre-commit** : `94cccb2`
**Draft commit title** : `feat(sprint18): Phase E1 — NVIDIA driver CVE check at launcher startup`
**Timebox** : ~18 min

---

## Verdict : PASS

0 findings P0/P1. 3 findings P2/P3 trackees comme carry-overs non-bloquants dans le commit body. Commit autorise.

(Note : l'evaluation multi-dimension a initialement propose CONCERN sur le pattern nouveau `_with(injectable)`, mais la recommandation finale de l'auditor est "Commit autorise" car aucun finding P0/P1, et les P2/P3 sont deferreds vers PATTERNS.md cote docs/rust/. Verdict aligne sur la conclusion operationnelle.)

---

## Dimensions

### Security

**semgrep scan** : non disponible (pas installe sur le poste). Fallback grep manuel effectue.

**unsafe/unwrap** :
- Aucun bloc `unsafe` dans `driver_check.rs`.
- Les `unwrap_or_default()` en production (`load_cache` ligne 333, `store_cache` ligne 348) sur `SystemTime::duration_since(UNIX_EPOCH)` — seul cas theorique d'echec : horloge systeme avant l'epoque Unix. Comportement degeneratif : `now = 0`, la cache expire toujours, on retente le fetch. Fail-open preserve. Acceptable.
- Les `unwrap()` nues aux lignes 476, 543, 547-548, 582, 589 sont exclusivement dans `#[cfg(test)]` — conforme au pattern PATTERNS.md §Sprint 5 "no `.unwrap()` in production code".

**TLS** : `reqwest` hoiste en workspace dep avec `default-features = false, features = ["json", "rustls-tls"]`. openssl explicitement exclu. `native-tls` feature absente. Conforme.

**User-Agent** : `concat!("sbfb-launcher/", env!("CARGO_PKG_VERSION"))` — bien forme, ne leak rien de la machine locale. Conforme.

**Timeout** : `FETCH_TIMEOUT = 10s` configure sur le `reqwest::Client`. Un TCP hang NVD ne peut pas bloquer indefiniment. Conforme.

**Leak info locale** : la query envoyee a NVD contient uniquement le CPE filter `cpe:2.3:o:nvidia:gpu_display_driver:*` — pas de version locale, pas d'IP, pas de node_id. Conforme.

**Fail-open** : `check_nvidia_drivers` retourne `DriverCheckReport` (jamais `Err`). Tous les chemins d'erreur resettent `fetch_failed = true` et retournent un rapport vide. `fetch_local_driver_version` retourne `Option<String>` — NVML absent → `None` → skip silencieux. Conforme.

**Path traversal cache** : `default_cache_path()` appelle `sbfb_home().map(|d| d.join(CACHE_LEAF))` ou `CACHE_LEAF = "nvd-cache.json"` (constante litterale). Aucun input utilisateur ne contribue au chemin. `store_cache` utilise `path.with_extension("json.tmp")` — toujours dans le meme repertoire parent. Conforme.

**Loopback / wire / zip** : non touches par ce diff. N/A.

---

### Patterns

Patterns documentes dans `docs/rust/PATTERNS.md` pertinents pour ce diff :

**Atomic write (Sprint 7.6 pattern)** : `store_cache` utilise `path.with_extension("json.tmp")` + `std::fs::rename`. Pattern respecte. Conforme.

**No `.unwrap()` en production** (§Sprint 5) : tous les `unwrap` sont dans `#[cfg(test)]`. Conforme.

**`#[serde(default)]` pour robustesse runtime** (CLAUDE.md policy) : utilise sur `NvdCpeMatch`, `NvdMetrics`, `NvdCve.metrics`, `NvdCve.configurations`, `NvdVulnWrapper`, `NvdResponse.vulnerabilities`. Justifie : NVD peut omettre des champs pour les CVE recents. Conforme.

**Signature fail-open vs `Result<T>`** : le plan specifiait `pub async fn check_nvidia_drivers() -> Result<DriverCheckReport>`. L'impl livre `-> DriverCheckReport` directement. Justification documentee (launcher ne doit jamais bloquer). La deviation est intentionnelle, documentee dans le module doc, et coherente avec la politique "warning-only" du plan. Pas de finding.

**Background `tokio::spawn` detache** : le plan disait "Startup async call". L'impl spawne un task detachee. Consequence : si le launcher s'arrete avant la fin du fetch (~10s timeout), le task est droppee sans warning. Ce n'est pas un probleme de correctness (le launcher abort est Ctrl+C conscient), mais le report peut ne jamais s'afficher si l'utilisateur quitte dans les 10 premieres secondes. **P3** — nit.

**Pattern drift detecte** : `check_nvidia_drivers_with` comme variante injectable pour les tests est un pattern nouveau (injectable API URL + tempdir). Ce pattern est bon et merite d'etre documente dans PATTERNS.md comme "injectable variant for hermetic testing". **P2** — tech debt tracker.

---

### Scope-cuts

Grep des fichiers du diff contre les 8 items scope-cut kickoff §6 :

| Scope cut | Fichiers grep | Resultat |
|---|---|---|
| `wasmtime` | `Cargo.toml`, `driver_check.rs`, `main.rs` | 0 match |
| `pyodide` | idem | 0 match |
| `pow_gossip` / `PoW` | idem | 0 match |
| `encryption.at.rest` | idem | 0 match |
| `tls.*pin` | idem | 0 match |
| `pkarr` | idem | 0 match |
| `ML-DSA` | idem | 0 match |
| `structured.output` | idem | 0 match |

Aucun scope leak detecte. Conforme.

---

### Tests-delta

Plan annonce : +5 tests unit.
Reel mesure : 464 passed (baseline 458 post-Phase D) → +6.

Tests livres vs plan :

| Test | Plan | Presente |
|---|---|---|
| `version_affected_by_cve_exact_criteria_match` | test 1 | oui |
| `nvd_fetch_stores_cache_and_reuses_within_ttl` | test 2 | oui |
| `cache_miss_when_ttl_expired` | test 3 | oui |
| `offline_fallback_returns_empty_report_not_err` | test 4 | oui |
| `filter_critical_cves_only_counts_critical` | test 5 | oui |
| `version_range_bounds_include_and_exclude` | **bonus, hors plan** | oui |

**Bonus test legitime** : `version_range_bounds_include_and_exclude` couvre l'edge case `version_start_including + version_end_excluding` que `cpe_match_covers` implemente (lignes 401-418). Ce cas NVD est courant (ranges "≥ X.Y, < Z.W") et n'est pas couvert par les tests exacts de CPE match. Le test bonus est une couverture nouvelle du chemin de matching, pas de la feature-creep. Delta reel +6 vs annonce +5 : legitime.

Test 2 note : utilise une URL sentinel non-accessible pour prouver que la cache evite le fetch. Pattern correct. **P3** — nit.

---

## LOC ratio 3.7x — evaluation scope creep vs decomposition legitime

Plan : ~180 LOC. Reel : 659 LOC. Ratio 3.7x.

Decomposition constatee :
- ~150 LOC : structs NVD schema (`NvdResponse`, `NvdVulnWrapper`, `NvdCve`, `NvdMetrics`, `CvssMetric`, `CvssData`, `NvdConfig`, `NvdNode`, `NvdCpeMatch`, `CacheEnvelope`) + leurs derives serde. La reponse NVD reelle est profondement imbriquee — ces structs sont necessaires pour un parse correct, pas optionnels.
- ~90 LOC : `parse_version`, `cmp_versions`, `cpe_match_covers`, `criteria_version` — le matching de version range NVD est plus complexe que le plan ne le supposait (4 bornes independantes, CPE exact vs range). Non-reducible.
- ~190 LOC : tests + fixtures `sample_cve`, `tmp_dir`. Tout est dans `#[cfg(test)]`, hors LOC production.
- ~230 LOC : code core + doc comments (plan ~180 LOC). L'ecart s'explique par les doc-comments explicatifs (design choices, why fail-open) qui constituent ~40-50 LOC.

**Verdict LOC** : decomposition entierement legitime. Le plan sous-estimait la profondeur du schema NVD. Aucune feature hors-plan ajoutee. Pas scope creep.

---

## Findings

- **P2** : `check_nvidia_drivers_with` (pattern injectable) est utile et nouveau. Ajouter une entree dans `docs/rust/PATTERNS.md` §Sprint 18 : "injectable variant pattern — surcharge `_with(api_url, cache_path, ttl, local_version)` pour tests hermetiques sans NVML ni reseau". — `crates/nexus-launcher/src/driver_check.rs:143`

- **P3** : `tokio::spawn` detache ligne `main.rs:190` — si l'utilisateur fait Ctrl+C en moins de 10s (FETCH_TIMEOUT), le warning CVE ne s'affiche jamais sans message d'explication. Nit : ajouter un log `[launcher] driver check: result not yet available (background task still running)` sur shutdown propre si le task est encore pending. Non-bloquant pour le commit.

- **P3** : Test `nvd_fetch_stores_cache_and_reuses_within_ttl` utilise une URL sentinel non-accessible. Risque negligeable, non-bloquant.

---

## Recommendation

**Commit autorise.** 0 finding P0/P1. Les 3 findings P2/P3 sont des ameliorations non-urgentes : tracker P2 dans PATTERNS.md au prochain sprint qui touche `docs/rust/`, et les deux P3 peuvent etre adressees dans le carry-over Phase E1 already documente (real-hardware fixture format / background task timeout).

Le delta +1 test bonus est legitime. Le ratio LOC 3.7x est justifie par la profondeur du schema NVD et les doc-comments. La signature `-> DriverCheckReport` (fail-open direct) est une amelioration par rapport au plan `-> Result<DriverCheckReport>`.
