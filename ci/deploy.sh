#!/bin/bash

# Exit on error or variable unset
set -o errexit -o nounset

NC='\033[39m'
CYAN='\033[36m'
GREEN='\033[32m'

rev=$(git rev-parse --short HEAD)

echo -e "${CYAN}Running cargo doc${NC}"
# Run cargo doc
cargo doc

echo -e "${CYAN}Running mdbook build${NC}"
# Run mdbook to generate the book
target/"$TARGET"/debug/mdbook build book-example/

echo -e "${CYAN}Copying book to target/doc${NC}"
# Copy files from rendered book to doc root
cp -R book-example/book/* target/doc/

cd target/doc

echo -e "${CYAN}Initializing Git${NC}"
git init
git config user.name "Mathieu David"
git config user.email "mathieudavid@mathieudavid.org"

git remote add upstream "https://$GH_TOKEN@github.com/azerupi/mdBook.git"
git fetch upstream
git reset upstream/gh-pages

touch .

echo -e "${CYAN}Pushing changes to gh-pages${NC}"
git add -A .
git commit -m "rebuild pages at ${rev}"
git push -q upstream HEAD:gh-pages

echo -e "${GREEN}Deployement done${NC}"
