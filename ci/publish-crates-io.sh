#!/bin/bash

set -e

function publish {
    crate_name="$1"
    manifest="$2"

    # Get the version from Cargo.toml
    version=`sed -n -E 's/^version = "(.*)"/\1/p' $manifest`

    # Check crates.io if it is already published
    set +e
    output=`curl --fail --silent --head --location https://static.crates.io/crates/$crate_name/$version/download`
    res="$?"
    set -e
    case $res in
        0)
            echo "${crate_name}@${version} appears to already be published"
            return
            ;;
        22) ;;
        *)
            echo "Failed to check ${crate_name}@${version} res: $res"
            echo "$output"
            exit 1
            ;;
    esac

    cargo publish --manifest-path $manifest --no-verify
}

publish mdbook-core crates/mdbook-core/Cargo.toml
publish mdbook-markdown crates/mdbook-markdown/Cargo.toml
publish mdbook-renderer crates/mdbook-renderer/Cargo.toml
publish mdbook-html crates/mdbook-html/Cargo.toml
publish mdbook-preprocessor crates/mdbook-preprocessor/Cargo.toml
publish mdbook-summary crates/mdbook-summary/Cargo.toml
publish mdbook-driver crates/mdbook-driver/Cargo.toml
publish mdbook Cargo.toml
