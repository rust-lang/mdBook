# `before_deploy` phase: here we package the build artifacts

set -ex

cargo build --target $TARGET --release

mkdir staging

cp target/$TARGET/release/mdbook staging

cd staging

tar czf ../${PROJECT_NAME}-${TRAVIS_TAG}-${TARGET}.tar.gz *
