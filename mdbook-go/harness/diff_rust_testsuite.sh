#!/usr/bin/env bash
# Diff harness that reuses the Rust mdBook testsuite as fixtures.
#
# Walks tests/testsuite/ looking for directories that contain a book.toml and
# a src/SUMMARY.md (i.e. real build fixtures, not error-condition tests) and
# runs both Rust and Go against each, then byte-diff's the output.
#
# Usage:
#   ./harness/diff_rust_testsuite.sh [testsuite-dir ...]
#   ./harness/diff_rust_testsuite.sh             # all testsuite/*/ dirs
#
# Outputs a table: PASS / DIFF / SKIP (and reason). Does NOT modify SKIP in
# diff.sh — the goal is to discover which testsuite fixtures Go can already
# pass. Add entries to SKIP only after a manual decision.
set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPO_ROOT="$(cd "$ROOT/.." && pwd)"
TESTSUITE_DIR_DEFAULT="$REPO_ROOT/tests/testsuite"
RUST_BIN="${MDBOOK_RUST_BIN:-$REPO_ROOT/target/debug/mdbook}"
GO_BIN="$ROOT/bin/mdbook-go"

if [[ ! -x "$GO_BIN" ]]; then
  echo "building Go binary first" >&2
  (cd "$ROOT" && go build -o "$GO_BIN" ./cmd/doclens)
fi

if [[ $# -gt 0 ]]; then
  SEARCH_DIRS=("$@")
else
  SEARCH_DIRS=("$TESTSUITE_DIR_DEFAULT")
fi

# Discover candidate fixture dirs: every <search_dir>/<category>/<name>/book.toml.
CANDIDATES=()
for search in "${SEARCH_DIRS[@]}"; do
  while IFS= read -r -d '' toml; do
    dir="$(dirname "$toml")"
    CANDIDATES+=("$dir")
  done < <(find "$search" -mindepth 3 -name book.toml -print0 2>/dev/null)
done

# Helper: extract the category dir (e.g. renderer/missing_renderer -> renderer).
category_dir() {
  local d="$1"
  local rel="${d#$REPO_ROOT/tests/testsuite/}"
  echo "${rel%%/*}"
}

# Filter to ones that look like a buildable book: must have src/SUMMARY.md
# AND no [preprocessor.*].command (we've frozen external plugin protocol).
BUILDABLE=()
SKIP_REASON=()
declare -A SKIP_MAP
for dir in "${CANDIDATES[@]}"; do
  name="$(basename "$(dirname "$dir")")/$(basename "$dir")"

  if [[ ! -f "$dir/src/SUMMARY.md" ]]; then
    SKIP_MAP["$dir"]="no src/SUMMARY.md"
    continue
  fi

  # Skip fixtures that depend on a custom preprocessor command (we've frozen
  # M3 external plugin protocol and haven't validated it against Rust).
  if grep -qE '^\s*command\s*=' "$dir/book.toml" 2>/dev/null; then
    if grep -qE '^\s*\[preprocessor\.' "$dir/book.toml" 2>/dev/null; then
      SKIP_MAP["$dir"]="uses external preprocessor command (M3 frozen)"
      continue
    fi
  fi

  # Skip fixtures whose Rust-side test asserts the build must fail (uses
  # BookTest::expect_failure() or has "ERROR " lines in expect_stderr). These
  # exercise error paths, not output equivalence.
  cat_path="$(category_dir "$dir")"
  test_rs="$(find "$REPO_ROOT/tests/testsuite/$cat_path" -maxdepth 1 -name "*.rs" 2>/dev/null | head -1)"
  if [[ -n "$test_rs" ]] && grep -qE 'expect_failure\(\)|expect_stderr\(str!\[\[r?#"\s*$' "$test_rs" 2>/dev/null; then
    # Narrower check: look for the test function name in this file
    fn_name="$(basename "$dir")"
    if grep -qE "fn ${fn_name}\(\)" "$test_rs" 2>/dev/null; then
      if awk "/fn ${fn_name}\(\)/,/^}/" "$test_rs" | grep -q 'expect_failure'; then
        SKIP_MAP["$dir"]="fixture tests an error path (expect_failure in Rust testsuite)"
        continue
      fi
    fi
  fi

  BUILDABLE+=("$dir")
done

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

results_file="$TMP/results.tsv"
: > "$results_file"

pass=0
diff=0
skip=0

for dir in "${BUILDABLE[@]}"; do
  rel="${dir#$REPO_ROOT/}"
  short="$(echo "$rel" | sed 's|tests/testsuite/||')"

  rust_out="$TMP/$short/rust"
  go_out="$TMP/$short/go"
  mkdir -p "$rust_out" "$go_out"

  # Run both; capture stderr to detect runtime failures.
  rust_log="$TMP/$short.rust.log"
  go_log="$TMP/$short.go.log"

  if ! "$RUST_BIN" build "$dir" --dest-dir "$rust_out" >"$rust_log" 2>&1; then
    # Rust failure is expected for fixtures that test error paths — those are
    # already pre-filtered into SKIP_MAP above. Anything left here is a real
    # regression.
    err="$(grep -E '^ERROR|^WARN.*not found' "$rust_log" | head -1)"
    echo -e "BUILD_FAIL\t$short\tRust build failed: ${err:-$(head -1 "$rust_log")}" >> "$results_file"
    skip=$((skip+1))
    continue
  fi

  if ! "$GO_BIN" build --dir "$dir" --dest-dir "$go_out" >"$go_log" 2>&1; then
    err="$(grep -E '^ERROR|^WARN' "$go_log" | head -1)"
    echo -e "BUILD_FAIL\t$short\tGo build failed: ${err:-$(head -1 "$go_log")}" >> "$results_file"
    skip=$((skip+1))
    continue
  fi

  if diff -r "$rust_out" "$go_out" >"$TMP/$short.diff" 2>&1; then
    count="$(find "$go_out" -type f | wc -l | tr -d ' ')"
    echo -e "PASS\t$short\t$count files identical" >> "$results_file"
    pass=$((pass+1))
  else
    diff_size="$(wc -l < "$TMP/$short.diff" | tr -d ' ')"
    echo -e "DIFF\t$short\t$diff_size diff lines (see $TMP/$short.diff)" >> "$results_file"
    diff=$((diff+1))
  fi
done

# Skipped candidates
for dir in "${CANDIDATES[@]}"; do
  rel="${dir#$REPO_ROOT/}"
  short="$(echo "$rel" | sed 's|tests/testsuite/||')"
  reason="${SKIP_MAP[$dir]:-}"
  if [[ -n "$reason" ]]; then
    echo -e "SKIP\t$short\t$reason" >> "$results_file"
    skip=$((skip+1))
  fi
done

# Print table, PASS first then DIFF then SKIP then BUILD_FAIL
printf "%-11s %-55s %s\n" "STATUS" "FIXTURE" "DETAIL"
printf "%-11s %-55s %s\n" "----------" "-------------------------------------------------------" "------"
sort -t$'\t' -k1,1 -k2,2 "$results_file" \
  | awk -F'\t' 'BEGIN{p=0;d=0;s=0;f=0}
    /^[A-Z]/ {
      st=$1; fx=$2; dt=$3
      if (st=="PASS") p++
      else if (st=="DIFF") d++
      else if (st=="SKIP") s++
      else if (st=="BUILD_FAIL") f++
      printf "%-11s %-55s %s\n", st, fx, dt
    }
    END {
      printf "\n%d PASS, %d DIFF, %d SKIP/BUILD_FAIL (of %d candidates)\n",
        p, d, s+f, p+d+s+f
    }'