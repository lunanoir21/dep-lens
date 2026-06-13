#!/usr/bin/env bash
# Run dep-lens against every per-ecosystem fixture in examples/ and print a
# one-line summary for each. Exits non-zero if any scanner errors or returns
# zero packages.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CLI="$ROOT/packages/dep-lens/dist/cli.js"

if [[ ! -f "$CLI" ]]; then
  echo "Building Node CLI..."
  (cd "$ROOT/packages/dep-lens" && npm run build >/dev/null)
fi

CORE_BIN="$ROOT/packages/dep-lens-linux-x64/bin/dep-lens-core"
if [[ ! -x "$CORE_BIN" ]] || [[ "$ROOT/target/release/dep-lens-core" -nt "$CORE_BIN" ]]; then
  echo "Building Rust core..."
  (cd "$ROOT" && cargo build --release >/dev/null)
  cp "$ROOT/target/release/dep-lens-core" "$CORE_BIN"
fi

status=0

run() {
  local name="$1"
  shift
  printf '%-20s ' "$name"
  if ! out=$(env "$@" node "$CLI" --json --path "$ROOT/examples/$name" 2>&1); then
    echo "ERROR: $out"
    status=1
    return
  fi
  python3 -c '
import json, sys
report = json.loads(sys.argv[1])
s = report["summary"]
if s["total"] == 0:
    print("FAIL: 0 packages found")
    sys.exit(1)
print("total={} permissive={} weak={} strong={} unknown={}".format(
    s["total"], s["permissive"], s["weakCopyleft"], s["strongCopyleft"], s["unknown"]))
' "$out" || status=1
}

run npm-test-project
run cargo-test-project
run go-test-project DEP_LENS_GOPATH="$ROOT/examples/go-test-project/.cache/gopath"
run python-test-project
run ruby-test-project DEP_LENS_GEM_HOME="$ROOT/examples/ruby-test-project/vendor/bundle/ruby/3.3.0"
run php-test-project
run java-test-project DEP_LENS_M2="$ROOT/examples/java-test-project/.m2/repository"
run dart-test-project DEP_LENS_PUB_CACHE="$ROOT/examples/dart-test-project/.pub-cache"
run cpp-test-project

exit $status
