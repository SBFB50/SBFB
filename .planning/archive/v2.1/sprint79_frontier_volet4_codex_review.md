### Point 1 : Correction drift hash
- Statut : CONFIRME
- Fichier(s) : scripts/check-frontier-contracts.sh:163, scripts/check-frontier-contracts.sh:171
- Evidence :
```sh
163:   manifest_hashes="$(find "$KNOW_DIR" -name MANIFEST.json -exec grep -oE '[0-9a-f]{16}' {} + 2>/dev/null | sort -u || true)"
171:     for h in $(grep -oE '[0-9a-f]{16}' "$pf" | sort -u || true); do
172:       if ! printf '%s\n' "$manifest_hashes" | grep -qxF "$h"; then
173:         echo "PROMPT-PROVENANCE: $pf cites blake3 16-hex '$h' absent from every $KNOW_DIR/*/MANIFEST.json"
175:         fail=1
```
Test réel : état propre `exit 0`; drift temporaire `8faa36021466192a -> 0000000000000000` donne `exit 1` avec `PROMPT-PROVENANCE`. Les 5 hashes actuels de `app-authoring.md` sont tous présents dans l’union des `MANIFEST.json`.

### Point 2 : `set -euo pipefail`
- Statut : CONFIRME
- Fichier(s) : scripts/check-frontier-contracts.sh:53, scripts/check-frontier-contracts.sh:163, scripts/check-frontier-contracts.sh:167, scripts/check-frontier-contracts.sh:172
- Evidence :
```sh
 53: set -euo pipefail
163:   manifest_hashes="$(find "$KNOW_DIR" -name MANIFEST.json -exec grep -oE '[0-9a-f]{16}' {} + 2>/dev/null | sort -u || true)"
167:     grep -qF "$KNOW_DIR" "$pf" || continue
172:       if ! printf '%s\n' "$manifest_hashes" | grep -qxF "$h"; then
```
Le `|| true` est après le pipeline complet, donc il avale bien le non-match sous `pipefail`. Le `grep -qF ... || continue` est dans une OR-list, et le `grep -qxF` est dans un `if !`, donc pas de sortie prématurée.

### Point 3 : BusyBox-safe
- Statut : CONFIRME
- Fichier(s) : scripts/check-frontier-contracts.sh:8, scripts/check-frontier-contracts.sh:163, scripts/check-frontier-contracts.sh:167, scripts/check-frontier-contracts.sh:172, scripts/check-frontier-contracts.sh:181
- Evidence :
```sh
  8: # Four deterministic checks (mirrors scripts/check-sharding-docs.sh
 10: # image: no grep -P, no --include, no \b, no \s, no mapfile/readarray):
163:   manifest_hashes="$(find "$KNOW_DIR" -name MANIFEST.json -exec grep -oE '[0-9a-f]{16}' {} + 2>/dev/null | sort -u || true)"
167:     grep -qF "$KNOW_DIR" "$pf" || continue
172:       if ! printf '%s\n' "$manifest_hashes" | grep -qxF "$h"; then
```
Vérifié dans `docker run --rm bash:5` : `grep -oE`, `grep -qxF`, `grep -qF`, `find -exec ... {} +`, `printf`, `sort -u` passent. Aucune construction interdite introduite dans le volet 4.

### Point 4 : Scope / faux positif 16-hex
- Statut : PARTIEL
- Fichier(s) : scripts/check-frontier-contracts.sh:167, scripts/check-frontier-contracts.sh:171, scripts/check-frontier-contracts.sh:172
- Evidence :
```sh
167:     grep -qF "$KNOW_DIR" "$pf" || continue
168:     # 4a — every inline 16-hex digest must be a known pack digest.
171:     for h in $(grep -oE '[0-9a-f]{16}' "$pf" | sort -u || true); do
172:       if ! printf '%s\n' "$manifest_hashes" | grep -qxF "$h"; then
173:         echo "PROMPT-PROVENANCE: $pf cites blake3 16-hex '$h' absent from every $KNOW_DIR/*/MANIFEST.json"
```
Oui : tout token lowercase 16-hex dans une fiche qui référence `docs/factory/knowledge/` sera traité comme digest de pack. Donc un préfixe SHA Git ou autre identifiant 16-hex absent des manifestes serait flagué. C’est partiellement documenté par la convention “every inline 16-hex digest”, mais le cas “16-hex non-pack interdit dans ces fiches” n’est pas explicitement nommé.

### Point 5 : ShellCheck SC2046
- Statut : CONFIRME
- Fichier(s) : scripts/check-frontier-contracts.sh:169, scripts/check-frontier-contracts.sh:170, scripts/check-frontier-contracts.sh:179, scripts/check-frontier-contracts.sh:180
- Evidence :
```sh
169:     # Intentional word-split over the unique hash list.
170:     # shellcheck disable=SC2046
171:     for h in $(grep -oE '[0-9a-f]{16}' "$pf" | sort -u || true); do
179:     # Intentional word-split over the unique path list.
180:     # shellcheck disable=SC2046
181:     for kp in $(grep -oE "$KNOW_DIR/[A-Za-z0-9_./-]+\.(json|md|ts)" "$pf" | sort -u || true); do
```
Les deux disables ciblent bien la ligne suivante avec substitution non quotée. Le split est borné par des patterns sans espace pour hashes et chemins. `shellcheck` n’est pas installé localement, mais `bash -n` passe.

## Resume final
- Total points : 5
- Confirmes : 4 / Gaps : 0 / Partiels : 1