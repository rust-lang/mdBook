# `script` phase: you usually build, test and generate docs in this phase

set -ex

case "$TRAVIS_OS_NAME" in
  linux)
    host=x86_64-unknown-linux-gnu
    ;;
  osx)
    host=x86_64-apple-darwin
    ;;
esac

# NOTE Workaround for rust-lang/rust#31907 - disable doc tests when crossing
if [ "$host" != "$TARGET" ]; then
  if [ "$TRAVIS_OS_NAME" = "osx" ]; then
    brew install gnu-sed --default-names
  fi

  find src -name '*.rs' -type f | xargs sed -i -e 's:\(//.\s*```\):\1 ignore,:g'
fi

cargo build --target $TARGET --verbose
cargo test --target $TARGET --verbose
