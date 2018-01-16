# This script takes care of testing your crate

set -ex

main() {
    cross build --target $TARGET
    cross build --manifest-path ./book-example/src/for_developers/mdbook-wordcount/Cargo.toml
    cross build --target $TARGET --release

    if [ ! -z $DISABLE_TESTS ]; then
        return
    fi

    cross test --target $TARGET
    cross test --target $TARGET --release
}

# we don't run the "test phase" when doing deploys
if [ -z $TRAVIS_TAG ]; then
    main
fi