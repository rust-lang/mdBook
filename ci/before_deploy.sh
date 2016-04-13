# `before_deploy` phase: here we package the build artifacts

set -ex

mktempd() {
  echo $(mktemp -d 2>/dev/null || mktemp -d -t tmp)
}

mk_artifacts() {
  cargo build --target $TARGET --release
}

mk_tarball() {
  local td=$(mktempd)
  local out_dir=$(pwd)

  cp target/$TARGET/release/mdbook $td

  pushd $td

  tar czf $out_dir/${PROJECT_NAME}-${TRAVIS_TAG}-${TARGET}.tar.gz *

  popd $td
  rm -r $td
}

main() {
  mk_artifacts
  mk_tarball
}

main
