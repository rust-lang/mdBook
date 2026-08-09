#!/usr/bin/env bash
# Diff harness for mdbook-go: runs the Rust mdbook binary and the Go port on
# the same fixture, then compares the two output trees byte for byte.
#
# Usage:
#   ./harness/diff.sh [fixture ...]
#
# With no arguments every fixture under fixtures/ is checked. Since M2 the
# comparison is strict: any difference is a failure. Differences that are
# knowingly deferred to a later milestone belong in KNOWN_DIFFS.md, and the
# fixture that triggers them should be listed in SKIP below.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPO_ROOT="$(cd "$ROOT/.." && pwd)"
RUST_BIN="${MDBOOK_RUST_BIN:-$REPO_ROOT/target/debug/mdbook}"
GO_BIN="$ROOT/bin/mdbook-go"

# Fixtures expected to differ, as "name # reason".
# - external-plugin: M3 外部插件链路已冻结（见 internal/plugin/cmd.go 顶部注释与
#   doc/plan/progress.md 的 M3 段落），代码保留、不跑端到端 diff。
# - ts-markdown-basic_markdown: goldmark 把跨两行的 HTML 起始标签视为 HTML 块，
#   pulldown-cmark 视为段内 inline HTML。这是已知解析器差异，登记在
#   MIGRATION.md 的 "Markdown / HTML differences" 一节。
SKIP=(
  "external-plugin # M3 external-plugin frozen, see doc/plan/progress.md"
  "ts-markdown-basic_markdown # goldmark vs pulldown-cmark: HTML block boundary, see MIGRATION.md"
)

if [[ $# -gt 0 ]]; then
  FIXTURES=("$@")
else
  FIXTURES=()
  for dir in "$ROOT"/tests/*/; do
    FIXTURES+=("$(basename "$dir")")
  done
fi

if [[ ! -x "$RUST_BIN" ]]; then
  echo "building Rust mdbook (debug)" >&2
  (cd "$REPO_ROOT" && cargo build --bin mdbook) >&2
fi

echo "=== building Go binary ===" >&2
(cd "$ROOT" && go build -o "$GO_BIN" ./cmd/doclens)

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

failed=0
for fixture in "${FIXTURES[@]}"; do
  fixture_dir="$ROOT/tests/$fixture"
  if [[ ! -d "$fixture_dir" ]]; then
    echo "unknown fixture: $fixture" >&2
    exit 2
  fi

  skip=0
  for entry in "${SKIP[@]:-}"; do
    [[ "${entry%% *}" == "$fixture" ]] && skip=1
  done
  if [[ $skip -eq 1 ]]; then
    echo "SKIP $fixture (see KNOWN_DIFFS.md)" >&2
    continue
  fi

  rust_out="$TMP/$fixture/rust"
  go_out="$TMP/$fixture/go"
  mkdir -p "$rust_out" "$go_out"

  "$RUST_BIN" build "$fixture_dir" --dest-dir "$rust_out" >/dev/null 2>&1
  "$GO_BIN" build --dir "$fixture_dir" --dest-dir "$go_out" >/dev/null

  if diff -r "$rust_out" "$go_out" >"$TMP/$fixture.diff" 2>&1; then
    count="$(find "$go_out" -type f | wc -l | tr -d ' ')"
    echo "OK   $fixture ($count files identical)" >&2
  else
    echo "DIFF $fixture" >&2
    head -50 "$TMP/$fixture.diff" >&2
    failed=1
  fi
done

exit $failed
