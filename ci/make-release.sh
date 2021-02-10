#!/usr/bin/env bash
# Builds the release and creates an archive and optionally deploys to GitHub.
set -ex

if [[ -z "$GITHUB_REF" ]]
then
  echo "GITHUB_REF must be set"
  exit 1
fi
# Strip mdbook-refs/tags/ from the start of the ref.
TAG=${GITHUB_REF#*/tags/}

host=$(rustc -Vv | grep ^host: | sed -e "s/host: //g")
export CARGO_PROFILE_RELEASE_LTO=true
cargo build --bin mdbook --release
cd target/release
case $1 in
  ubuntu*)
    asset="mdbook-$TAG-$host.tar.gz"
    tar czf ../../$asset mdbook
    ;;
  macos*)
    asset="mdbook-$TAG-$host.tar.gz"
    # There is a bug with BSD tar on macOS where the first 8MB of the file are
    # sometimes all NUL bytes. See https://github.com/actions/cache/issues/403
    # and https://github.com/rust-lang/cargo/issues/8603 for some more
    # information. An alternative solution here is to install GNU tar, but
    # flushing the disk cache seems to work, too.
    sudo /usr/sbin/purge
    tar czf ../../$asset mdbook
    ;;
  windows*)
    asset="mdbook-$TAG-$host.zip"
    7z a ../../$asset mdbook.exe
    ;;
  *)
    echo "OS should be first parameter, was: $1"
    ;;
esac
cd ../..

if [[ -z "$GITHUB_TOKEN" ]]
then
  echo "$GITHUB_TOKEN not set, skipping deploy."
else
  hub release edit -m "" --attach $asset $TAG
fi
