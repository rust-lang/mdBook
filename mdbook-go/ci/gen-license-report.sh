#!/usr/bin/env bash
# Generate a dependency-license report for mdbook-go.
#
# Tool: github.com/google/go-licenses
# Output:
#   dist/licenses.csv               — one row per direct + transitive dep
#   dist/licenses/<module>/LICENSE* — copy of each license text
#
# Usage:
#   ./ci/gen-license-report.sh
#
# Both outputs are intended to be uploaded as release assets so that
# downstream packagers can satisfy the "conveyed with source" requirement
# of MIT / Apache-2.0 / BSD-style licenses.

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Install go-licenses if not already on PATH.
if ! command -v go-licenses >/dev/null; then
    GOBIN="$(go env GOBIN)"
    if [[ -z "$GOBIN" ]]; then
        GOBIN="$(go env GOPATH)/bin"
    fi
    echo "==> installing go-licenses to $GOBIN" >&2
    go install github.com/google/go-licenses@latest
fi

mkdir -p dist

echo "==> generating dist/licenses.csv" >&2
# `csv` covers our own packages + every transitive dep. Most of our own
# packages show up with license "MPL-2.0" once mdbook-go/LICENSE is in
# place; the external deps show their respective SPDX.
go-licenses csv ./... > dist/licenses.csv || {
    # go-licenses exits 1 when any package's license can't be determined
    # automatically. The CSV is still written and is the primary
    # deliverable, so we accept that and continue.
    echo "warning: go-licenses csv reported unresolved licenses; see CSV" >&2
}

echo "==> copying external license texts into dist/licenses/" >&2
# `--save_path` must live OUTSIDE the source tree: when it sits inside
# the project, go-licenses recurses and writes LICENSE files into its
# own output directory (which the next run then scans again, ad
# infinitum). Use a /tmp staging directory and copy the result back.
# `--force` is required: go-licenses refuses to write into a directory
# that already exists, and mktemp's directory counts as "existing".
TMP_SAVE="$(mktemp -d)"
trap 'rm -rf "$TMP_SAVE"' EXIT
go-licenses save --force ./... --save_path "$TMP_SAVE" 2>/dev/null || true
rm -rf dist/licenses
# Only keep the top-level external module directories; our own
# mdbook-go/* entries are noise and would inflate the asset bundle.
mkdir -p dist/licenses
for src in "$TMP_SAVE"/*; do
    base="$(basename "$src")"
    # Skip the project's own packages; we only ship license texts for
    # third-party deps so the release asset doesn't duplicate the
    # repo-root MPL-2.0 text.
    case "$base" in
        mdbook-go*) continue ;;
    esac
    mkdir -p "dist/licenses/$base"
    # Copy LICENSE* and NOTICE* but skip any actual source files
    # (some go-licenses versions copy a sample of the upstream source
    # into the save directory).
    for f in "$src"/*; do
        fname="$(basename "$f")"
        case "$fname" in
            LICENSE*|NOTICE*|COPYING*) cp "$f" "dist/licenses/$base/" ;;
        esac
    done
done

echo "==> done" >&2
echo "    CSV:   dist/licenses.csv ($(wc -l < dist/licenses.csv | tr -d ' ') lines)" >&2
echo "    Texts: $(find dist/licenses -type f 2>/dev/null | wc -l | tr -d ' ') files" >&2