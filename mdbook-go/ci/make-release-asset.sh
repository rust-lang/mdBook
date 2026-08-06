#!/usr/bin/env bash
# Build and package a single mdbook-go release asset for one OS/arch
# target, mirroring the Rust counterpart `ci/make-release-asset.sh`.
#
# Usage (CI):
#   GITHUB_REF=refs/tags/v0.1.0 \
#     ./ci/make-release-asset.sh ubuntu-22.04 linux/amd64
#
# Outputs:
#   dist/mdbook-go-<version>-<goos>-<goarch>.tar.gz   (linux + darwin)
#   dist/mdbook-go-<version>-<goos>-<goarch>.zip      (windows)
#
# The script depends on:
#   - build-cross.sh to produce the raw binary
#   - gh release upload (called from the GitHub Actions caller, not
#     here, so this script is also usable for local dry-run packaging)
#
# The macOS tar bug mentioned in the Rust version (BSD tar writing
# all-zero blocks for the first ~8MB) does not apply to GNU tar or
# to the BSD tar shipped with current macOS releases; we do not
# replicate the `sudo /usr/sbin/purge` workaround here.

set -ex

if [[ -z "${GITHUB_REF:-}" ]]; then
    echo "GITHUB_REF must be set (e.g. refs/tags/v0.1.0)" >&2
    exit 1
fi
TAG="${GITHUB_REF#refs/tags/}"

OS="$1"
TARGET="$2"
GOOS="${TARGET%/*}"
GOARCH="${TARGET#*/}"

# Strip the leading 'v' from the version for filename consistency with
# the Rust release script.
VERSION="${TAG#v}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
cd "$ROOT"

# Build the bare binary (Linux only — for the other OSes the GitHub
# Actions runner of that OS does the build).
"$SCRIPT_DIR/build-cross.sh" "$TARGET" >&2

EXT=""
case "$GOOS" in
    windows) EXT=".exe" ;;
esac

RAW="dist/mdbook-go-${VERSION}-${GOOS}-${GOARCH}${EXT}"

case "$OS" in
    ubuntu*|macos*)
        ASSET="dist/mdbook-go-${VERSION}-${GOOS}-${GOARCH}.tar.gz"
        # -C dist so the archive contains just `mdbook-go-...` rather
        # than `dist/mdbook-go-...`.
        tar -C dist -czf "$ASSET" "$(basename "$RAW")"
        ;;
    windows*)
        ASSET="dist/mdbook-go-${VERSION}-${GOOS}-${GOARCH}.zip"
        # 7z is what the Rust script uses; we fall back to `zip` only if
        # 7z is missing (the GitHub-hosted windows-latest image ships
        # 7z by default).
        if command -v 7z >/dev/null; then
            (cd dist && 7z a "$ASSET" "$(basename "$RAW")")
        else
            (cd dist && zip "$ASSET" "$(basename "$RAW")")
        fi
        ;;
    *)
        echo "OS should be ubuntu*, macos*, or windows* — was: $OS" >&2
        exit 1
        ;;
esac

echo "Built: $ASSET"
# Emit the asset name on stdout so the caller can `gh release upload`
# without re-parsing the path.
echo "MDBOOK_ASSET=$ASSET"
echo "MDBOOK_TAG=$TAG"