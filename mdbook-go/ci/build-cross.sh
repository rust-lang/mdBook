#!/usr/bin/env bash
# Cross-platform build for mdbook-go.
#
# Usage:
#   ./ci/build-cross.sh                       # build every supported target
#   ./ci/build-cross.sh linux/amd64           # build a single target
#   ./ci/build-cross.sh linux/arm64 darwin/arm64
#
# Output:
#   dist/mdbook-go-<version>-<goos>-<goarch>[.exe]
#
# The script is the Go analogue of `ci/make-release-asset.sh` in the
# Rust source tree. Both `make-release-asset.sh` (in this directory, see
# M6.7) and GitHub Actions' `.github/workflows/mdbook-go-ci.yml`
# cross-build job call this script.
#
# No external dependencies beyond the Go toolchain — no docker, no
# zig, no osxcross. GOOS/GOARCH is enough because mdbook-go's CGo
# surface is empty (goldmark, fsnotify, gorilla/websocket, etc. are all
# pure Go on the platforms we support).

set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

# Read the version from the source-of-truth: cmd/mdbook/main.go prints
# "mdbook-go 0.1.0 ..." in its `version` switch case. We grep that line
# rather than `git describe` so the script works in CI where the
# working tree may be a shallow checkout with no tags.
VERSION="$(grep -oE 'mdbook-go [0-9]+\.[0-9]+\.[0-9]+' cmd/mdbook/main.go | head -1 | awk '{print $2}')"
if [[ -z "$VERSION" ]]; then
    echo "could not extract version from cmd/mdbook/main.go" >&2
    exit 1
fi

TARGETS=(
    linux/amd64
    linux/arm64
    darwin/amd64
    darwin/arm64
    windows/amd64
    windows/arm64
)

if [[ $# -gt 0 ]]; then
    TARGETS=("$@")
fi

mkdir -p dist

for target in "${TARGETS[@]}"; do
    goos="${target%/*}"
    goarch="${target#*/}"
    ext=""
    if [[ "$goos" == "windows" ]]; then
        ext=".exe"
    fi
    asset="dist/mdbook-go-${VERSION}-${goos}-${goarch}${ext}"
    echo "==> $target -> $asset" >&2
    # -trimpath strips the build-machine's GOPATH from the binary so the
    # same source produces a bit-identical artifact on any host. -s -w
    # strips the symbol table / DWARF, taking the binary from ~14 MiB
    # down to ~10 MiB.
    GOOS="$goos" GOARCH="$goarch" \
        go build -trimpath -ldflags="-s -w" \
        -o "$asset" ./cmd/mdbook
done

echo
echo "Artifacts in dist/:" >&2
ls -lh dist/ >&2