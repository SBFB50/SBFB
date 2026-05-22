#!/usr/bin/env bash
# count-tests.sh — Parse nextest + Vitest output and print structured counters.
# Usage: ./scripts/count-tests.sh
# Requires: cargo nextest, npm (in web/)
set -euo pipefail

echo "=== Rust nextest ==="
NEXTEST_OUT=$(cargo nextest run --workspace --locked 2>&1) || true
RUST_COUNT=$(echo "$NEXTEST_OUT" | grep -oP '\d+ tests? run' | grep -oP '^\d+' || echo "0")
RUST_PASS=$(echo "$NEXTEST_OUT" | grep -oP '\d+ passed' | grep -oP '^\d+' || echo "0")
echo "  Total: $RUST_COUNT"
echo "  Passed: $RUST_PASS"

echo ""
echo "=== Rust doctests ==="
DOCTEST_OUT=$(cargo test --workspace --locked --doc 2>&1) || true
DOC_PASS=$(echo "$DOCTEST_OUT" | grep -oP 'test result: ok\. \d+ passed' | grep -oP '\d+ passed' | grep -oP '^\d+' || echo "0")
echo "  Passed: $DOC_PASS"

echo ""
echo "=== Vitest ==="
VITEST_OUT=$(cd web && npm run test:unit -- --reporter=verbose 2>&1) || true
VIT_TOTAL=$(echo "$VITEST_OUT" | grep -oP 'Tests\s+\d+' | grep -oP '\d+' | tail -1 || echo "0")
echo "  Total: $VIT_TOTAL"

echo ""
echo "=== Summary ==="
echo "  Rust nextest: $RUST_PASS"
echo "  Rust doctests: $DOC_PASS"
echo "  Vitest: $VIT_TOTAL"
TOTAL=$((RUST_PASS + DOC_PASS + VIT_TOTAL))
echo "  Combined: ~$TOTAL"
