#!/bin/bash

set -o errexit -o nounset

rev=$(git rev-parse --short HEAD)

cd book-example/book

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
