#!/bin/bash

# Exit on error or variable unset
set -o errexit -o nounset

rev=$(git rev-parse --short HEAD)

# Run cargo doc
cargo doc

# Run mdbook to generate the book
target/debug/mdbook build book-example/

# Copy files from rendered book to doc root
cp book-example/book/* target/doc/

cd target/doc


git init
git config user.name "Mathieu David"
git config user.email "mathieudavid@mathieudavid.org"

git remote add upstream "https://$GH_TOKEN@github.com/azerupi/mdBook.git"
git fetch upstream
git reset upstream/gh-pages

touch .

git add -A .
git commit -m "rebuild pages at ${rev}"
git push -q upstream HEAD:gh-pages
