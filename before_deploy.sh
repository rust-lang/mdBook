# `before_deploy` phase: here we package the build artifacts

set -ex

if [ "$TRAVIS_RUST_VERSION" = "stable" ]; then
  cargo build --release

  mkdir staging

  cp target/release/mdbook staging

  cd staging

  # release tarball will look like 'mdbook-v1.2.3-x86_64-unknown-linux-gnu.tar.gz'
  tar czf ../${PROJECT_NAME}-${TRAVIS_TAG}-${TARGET}.tar.gz *
fi
