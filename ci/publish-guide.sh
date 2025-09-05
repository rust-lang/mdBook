#!/usr/bin/env bash
# This publishes the user guide to GitHub Pages.
#
# If this is a pre-release, then it goes in a separate directory called "pre-release".
# Commits are amended to avoid keeping history which can balloon the repo size.
set -ex

cargo run --no-default-features -F search -- build guide

VERSION=$(cargo metadata --format-version 1 --no-deps | jq '.packages[] | select(.name == "mdbook") | .version')

if [[ "$VERSION" == *-* ]]; then
    PRERELEASE=true
else
    PRERELEASE=false
fi

git fetch origin gh-pages
git worktree add gh-pages gh-pages
git config user.name "Deploy from CI"
git config user.email ""
cd gh-pages
if [[ "$PRERELEASE" == "true" ]]
then
    rm -rf pre-release
    mv ../guide/book pre-release
    git add pre-release
    git commit --amend -m "Deploy $GITHUB_SHA pre-release to gh-pages"
else
    # Delete everything except pre-release and .git.
    find . -mindepth 1 -maxdepth 1 -not -name "pre-release" -not -name ".git" -exec rm -rf {} +
    # Copy the guide here.
    find ../guide/book/ -mindepth 1 -maxdepth 1 -exec mv {} . \;
    git add .
    git commit --amend -m "Deploy $GITHUB_SHA to gh-pages"
fi

git push --force origin +gh-pages
