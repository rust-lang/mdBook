# This script takes care of testing your crate

set -ex

main() {
    cargo build --target $TARGET --all --no-default-features
    cargo build --target $TARGET --all
    cargo build --target $TARGET --all --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cargo test --target $TARGET --no-default-features
    cargo test --target $TARGET
    cargo test --target $TARGET --release
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi