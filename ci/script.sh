# This script takes care of testing your crate

set -ex

main() {
    cross build --target $TARGET --all --no-default-features
    cross build --target $TARGET --all
    cross build --target $TARGET --all --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET --no-default-features
    cross test --target $TARGET
    cross test --target $TARGET --release
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi