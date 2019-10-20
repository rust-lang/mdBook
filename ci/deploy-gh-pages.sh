#!/usr/bin/env bash
# Updates gh-pages with latest docs.
set -ex

cargo run -- build book-example
cd book-example/book
touch .nojekyll
git init
git config --local user.email ""
git config --local user.name "GitHub Deployer"
git add .
git commit -m "Deploy to gh-pages"
remote="https://${GITHUB_ACTOR}:${GITHUB_TOKEN}@github.com/${GITHUB_REPOSITORY}.git"
git push "$remote" HEAD:gh-pages --force
