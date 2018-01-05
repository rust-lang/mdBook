#!/bin/bash
# Deploys the `book-example` to GitHub Pages

set -ex

# Only run this on the master branch for stable
if [ "$TRAVIS_PULL_REQUEST" != "false" ] || 
   [ "$TRAVIS_BRANCH" != "master" ] ||
   [ "$TRAVIS_RUST_VERSION" != "stable" ] ||
   [ "$TARGET" != "x86_64-unknown-linux-gnu" ]; then
   exit 0
fi

# Make sure we have the css dependencies
npm install -g stylus nib 

NC='\033[39m'
CYAN='\033[36m'
GREEN='\033[32m'

rev=$(git rev-parse --short HEAD)

echo -e "${CYAN}Running cargo doc${NC}"
cargo doc --features regenerate-css > /dev/null

echo -e "${CYAN}Running mdbook build${NC}"
cargo run -- build book-example/

echo -e "${CYAN}Copying book to target/doc${NC}"
cp -R book-example/book/* target/doc/

cd target/doc

echo -e "${CYAN}Initializing Git${NC}"
git init
git config user.name "Michael Bryan"
git config user.email "michaelfbryan@gmail.com"

git remote add upstream "https://$GH_TOKEN@github.com/rust-lang-nursery/mdBook.git"
git fetch upstream --quiet
git reset upstream/gh-pages --quiet

touch .

echo -e "${CYAN}Pushing changes to gh-pages${NC}"
git add -A . 
git commit -m "rebuild pages at ${rev}" --quiet
git push -q upstream HEAD:gh-pages --quiet

echo -e "${GREEN}Deployed docs to GitHub Pages${NC}"
