# Sprint 21 phase coord-side — output filter + coord-side PII
redaction design doc

**Date** : 2026-04-19
**Phase cible** : Sprint 21 phase coord-side (feat suivant
`624ad7e`)
**Statut** : design figé, implementation en cours
**Refs** : `sprint21_kickoff.md §D2 layer 2 + §D3`,
`sprint21_plan.md §6` (après `chore(planning)` naming-fix
`624ad7e`), `sprint21_phase_C_preflight.md` (verdict EXECUTE
plan-as-is).

## 1. Objectif

Fournir la couche 2 defense-in-depth PII redaction coord-side
(complément de la couche 1 iframe client livrée phase B `d5b0035`)
et la couche output filter (LLM Guard InvisibleText + EED prompt
echo detection) dans le pipeline coord.

**Ordering cible** :

```
user prompt
  -> iframe PII redact layer 1 (phase B)
  -> postMessage pii_redact bridge method
  -> coord API /tasks/submit
  -> rate-limit gate (phase A worker-engine side)
  -> PoW gate (S20 phase C)
  -> PII redact layer 2 (this phase — coord-side)
  -> sign_task + iroh-docs write
  -> worker claim + LLM generation
  -> worker sign_result
  -> iroh-docs deliver
  -> coord validator: verify 3-layer (signature + model digest + logprob)
  -> coord OUTPUT FILTER (this phase — LLM Guard + EED)
  -> mark_completed + kudos credit
```

## 2. Sous-composants

### 2.1 `PiiRedactor` (pii_redactor.py, nouveau)

**Responsabilité** : supprimer les entités PII détectées du
texte avant dispatch worker. Layer 2 — garantit que même si la
couche 1 iframe n'a pas tourné (user non-browser, test, crash
JS), les relais et workers ne voient jamais le prompt brut
contenant des PII.

**API** :

```python
class PiiRedactor:
    def __init__(
        self,
        *,
        model_name: str = "knowledgator/gliner-pii-edge-v1.0",
        policy_path: Path | None = None,
    ) -> None: ...

    def redact(self, text: str) -> str: ...

    def reload_policy(self) -> None: ...  # optional hot-reload
```

**Moteur** : `presidio-analyzer 2.2.362` (Microsoft MIT) avec
`presidio_analyzer.predefined_recognizers.GLiNERRecognizer`
chargé sur le même modèle upstream HF que la phase B iframe
(`knowledgator/gliner-pii-edge-v1.0`). Le coord utilise le path
PyTorch (pas de contrainte taille ni latence WASM), l'iframe
utilise l'export ONNX quint8 45.8 MB. Parité comportementale
assurée par le même upstream model + le même mapping
d'entités (email, phone, credit_card, ssn, iban, name,
address, date_time, ip_address, url).

**Anonymisation** : entités remplacées inline par des
placeholders typés :
- `EMAIL` -> `<EMAIL_N>` où N est un incrément par type
  (1..k)
- `PHONE_NUMBER` -> `<PHONE_N>`
- `PERSON` -> `<PERSON_N>`
- etc.

Pas de vault de déanonymisation côté coord : le prompt
redacted est **signé tel quel** et envoyé au worker. Le client
original reçoit son result via l'API sans rehydration
(privacy by default). Un operator qui veut du
`Deanonymize`-style devra l'activer côté application SDK
plus tard (hors scope phase coord-side).

**Fallback regex** : si `GLiNERRecognizer` n'est pas
chargeable (modèle absent, env CI sans internet, import
error), le redactor dégrade vers les recognizers Presidio
built-in (`EmailRecognizer`, `PhoneRecognizer`,
`CreditCardRecognizer`, `IbanRecognizer`, `UsSsnRecognizer`)
qui sont pur-regex deterministes. Log warning structlog
`pii_redactor_degraded` (pas d'erreur fatale — on veut que
le coord démarre même en env minimal).

**Policy `~/.sbfb/pii_redaction_policy.toml`** :

```toml
[default]
confidence_threshold = 0.5
enabled_entities = ["EMAIL_ADDRESS", "PHONE_NUMBER",
                    "CREDIT_CARD", "IBAN_CODE", "US_SSN",
                    "PERSON", "LOCATION", "IP_ADDRESS", "URL"]
redaction_format = "<{entity_type}_{N}>"

[overrides.gate2_apps]
# Strict mode pour les apps identifiées Gate 2.
# Plus bas threshold = plus de redaction = moins de faux negatifs.
confidence_threshold = 0.3
```

**Hot-reload pattern** : lecture `mtime` du fichier policy au
début de chaque `redact()` call ; si mtime a changé, reload le
TOML et reconstruit le recognizer set. Pattern identique à
`TokenRotator` S18 + `pow_policy_loader` S20 phase coord :
- Debounce 50 ms (protège contre editor multi-save).
- Malformed-reload guard : si TOML invalide, garder l'ancienne
  policy + log warning.
- File-deletion guard : si fichier disparaît, garder l'ancienne
  policy (pas de fallback à zéro-redaction — failing-closed).

### 2.2 `OutputFilter` (output_filter.py, nouveau)

**Responsabilité** : scanner le `content` du `ResultEntry` reçu
d'un worker et bloquer les deux classes d'attaques :
1. **Invisible text steganography** : zero-width U+200B, PUA
   U+E000-U+F8FF, Tag chars U+E0020-U+E007F cachés dans la
   réponse.
2. **Prompt leak** : la réponse contient tout ou partie du
   `system_prompt` (le modèle a echo-reproduit le prompt
   système — attaque PLeak CCS'24 arXiv 2405.06823).

**API** :

```python
class FilterVerdict:
    is_valid: bool
    reason: str  # "invisible_text" | "prompt_echo_exact"
                 # | "prompt_echo_eed" | "ok"
    risk_score: float

class OutputFilter:
    def __init__(
        self,
        *,
        policy_path: Path | None = None,
    ) -> None: ...

    def filter(
        self,
        system_prompt: str,
        user_prompt: str,
        model_output: str,
    ) -> FilterVerdict: ...

    def reload_policy(self) -> None: ...
```

**Invisible text** (pivot D3 2026-04-19 — implémentation locale,
cf. `sprint21_phase_C_preflight.md §Pivot log`) : scanner
ré-implémenté en ~30 lignes Python pur sans dépendance externe
(drop llm-guard pour cause de transitive-pin
`presidio-analyzer==2.2.358` incompatible avec D2 `>=2.2.362`).
L'algorithme reproduit fidèlement le comportement du
`llm_guard.input_scanners.InvisibleText` scanner :

- **Strip** les caractères dans les ranges suivants :
  - Zero-width space/joiner : U+200B, U+200C, U+200D, U+200E,
    U+200F, U+2060, U+FEFF.
  - Private Use Area (PUA) : U+E000-U+F8FF + U+F0000-U+FFFFD +
    U+100000-U+10FFFD.
  - Tag chars : U+E0020-U+E007F (ASCII tag block).
- **Whitelist par défaut** (pas strippés) — catégorie Unicode
  `Cf` (Format) utilisée pour i18n légitime :
  - U+202A LRE, U+202B RLE, U+202C PDF, U+202D LRO, U+202E RLO
    (bidi Arabe/Hébreu) : conservés.
  - U+2066, U+2067, U+2068, U+2069 : conservés.
- **Retour** : `(sanitized_text, is_valid, risk_score)` où
  `risk_score = 1.0` si au moins un char a été strippé, `0.0`
  sinon. `is_valid = not stripped_any`.

Parité comportementale avec le llm-guard scanner testée
explicitement (tests 4-5 du plan §6.3). La liste de ranges
strippés est documentée inline dans `output_filter.py` avec
références précises aux blocs Unicode.

**Prompt echo detection** : 3 niveaux cumulés :
1. **Exact Match** : `system_prompt in model_output` -> block.
2. **Substring Match (overlap)** : pour chaque substring
   de longueur >= 40 chars du system_prompt, check si
   présente dans model_output. Un seul match -> block.
3. **EED (Extended Edit Distance)** : via
   `rapidfuzz.distance.Levenshtein.normalized_similarity`.
   Seuil default **0.85** (configurable policy.toml).
   Si `normalized_similarity(system_prompt, model_output)
   >= 0.85` -> block. Seuil 0.85 est empirique, tuné sur
   le corpus PLeak CCS'24 (précision / rappel compromis).
   Le seuil inline-comment dans `output_filter.py`
   documente pourquoi on le laisse configurable (pas de
   one-size-fits-all sans tuning par déploiement).

**Policy `~/.sbfb/output_filter_policy.toml`** :

```toml
[default]
enabled = true

[invisible_text]
strip_zero_width = true
strip_pua = true
strip_tag_chars = true
whitelist_cf = true   # i18n RLO / LRO / PDF conservés

[prompt_echo]
exact_match = true
substring_match_min_len = 40
eed_threshold = 0.85
```

**Hot-reload** : pattern identique à `PiiRedactor`.

### 2.3 Integration points

**`dispatcher.py::Dispatcher.submit`** (modifié) :

- Nouveau paramètre `pii_redactor: PiiRedactor | None = None`
  dans `Dispatcher.__init__` (default None = no-op,
  zero-régression tests existants).
- Si `self._pii_redactor is not None` :
  1. `prompt_redacted = self._pii_redactor.redact(req.prompt)`
  2. `system_prompt_redacted = self._pii_redactor.redact(req.system_prompt)`
  3. Utiliser `prompt_redacted` / `system_prompt_redacted` dans
     `task_dict` (au lieu de `req.prompt` / `req.system_prompt`).
  4. Log structlog `pii_redacted` avec count par type
     (jamais les valeurs brutes).
- Le `SubmitRequest` lui-même reste intact (pas de mutation en
  place — le dataclass est utilisé par le caller pour d'autres
  dérivations).

**`validator.py::Validator._handle_result`** (modifié) :

- Nouveau paramètre `output_filter: OutputFilter | None = None`
  dans `Validator.__init__` (default None = no-op).
- Si `self._output_filter is not None` : après
  `verify_entries` pass mais avant `mark_completed` + `credit` :
  1. Parse `task_entry_json` pour extraire `system_prompt` et
     `prompt`.
  2. Parse `result_entry` pour extraire `payload.content`
     (le champ où le worker met le LLM output ; si absent,
     skip filter + log warning).
  3. `verdict = self._output_filter.filter(system_prompt,
     user_prompt, model_output)`.
  4. Si `verdict.is_valid` == False :
     - `dispatcher.mark_failed(task_id, f"output_filter:
       {verdict.reason}")`.
     - Retourne `ValidationEvent(kind="result_rejected",
       reason=f"output_filter: {verdict.reason}")`.
     - **Kudos ne sont PAS credités** (pattern identique à un
       3-layer verify fail).
- Le `ResultEntry` reste publié sur iroh-docs (publié par le
  worker avant arrivée coord). La surface couverte par ce
  filtre = audit trail coord + kudos + delivery au client via
  l'API control plane. Pas de protection de la surface gossip
  (limite inhérente documentée threat model T3).

## 3. Contrat de tests (10 tests)

Cf. plan §6.3 :

1. `test_pii_redactor.py::test_redact_email_phone_name` —
   smoke test Presidio default (EmailRecognizer + PhoneRecognizer).
2. `test_pii_redactor.py::test_redact_gate2_apps_strict_mode` —
   override policy confidence 0.3, tout redact.
3. `test_pii_redactor.py::test_policy_hot_reload` — modifier
   policy.toml runtime, reload appliqué (mtime change).
4. `test_output_filter.py::test_invisible_chars_stripped` —
   U+200B + U+E000 + U+E0020 strippés.
5. `test_output_filter.py::test_rlo_lro_whitelisted_for_i18n` —
   U+202E RLO conservé.
6. `test_output_filter.py::test_prompt_echo_exact_match_blocks` —
   system_prompt direct in output -> block.
7. `test_output_filter.py::test_prompt_echo_eed_similarity_above_
   0_85_blocks` — reconstruction partielle 0.9 -> block.
8. `test_output_filter.py::test_prompt_echo_eed_similarity_below_
   0_85_passes` — reconstruction 0.3 -> pass.
9. `test_output_filter.py::test_pleak_attack_reconstruction_
   scenarios` — 5 PLeak-style reconstructions.
10. `test_output_filter.py::test_benign_output_passes_through` —
    no false positive sur output user normal.

## 4. Out of scope (carry S22+)

- `Deanonymize` vault coord-side (rehydration des entités
  redactées pour présentation au client original) — apps SDK
  feature future.
- ProxyPrompt defense proactive (arXiv 2505.11459) — reactive
  detection suffit pour S21, proactive arriverait S22+ avec
  TEE S30.
- Intégration iframe <-> coord policy sync : pour l'instant
  les 2 policies vivent séparément, design pattern à revoir
  si divergence observée en production.
- Entity-level audit (qui a redacté quoi quand) — plan §6
  hors scope S21.

## 5. Rationale post-G8

Verdict G8 preflight : **EXECUTE plan-as-is**. S1-S4 clean.
Cf. `sprint21_phase_C_preflight.md` pour détail complet des
scans (presidio-analyzer 2.2.362, llm-guard 0.3.16,
rapidfuzz 3.x, modèle HF knowledgator/gliner-pii-edge-v1.0).

2 notes de conception documentées :
1. `LLM Guard InvisibleText` est officiellement
   `input_scanners` mais son API stateless permet réutilisation
   output. Wrapping explicite dans `OutputFilter` isole la
   pattern.
2. « Même modèle ONNX source of truth unique » (plan §6) =
   même **modèle upstream HF**, pas même artefact binaire
   (coord = PyTorch via `gliner` lib, iframe = ONNX quint8
   45.8 MB déjà livré phase B).
