#!/usr/bin/env bash
# Updates all compatible Cargo dependencies.
#
# I wasn't able to get Renovate to update compatible dependencies in a way
# that I like, so this script takes care of it. This uses `cargo upgrade` to
# ensure that `Cargo.toml` also gets updated. This also makes sure that all
# transitive dependencies are updated.

set -ex

git fetch origin update-dependencies
if git checkout update-dependencies
then
    git reset --hard origin/master
else
    git checkout -b update-dependencies
fi

cat > commit-message << 'EOF'
Update cargo dependencies

```
EOF
cargo upgrade >> commit-message
echo '```' >> commit-message
if git diff --quiet
then
    echo "No changes detected, exiting."
    exit 0
fi
# Also update any transitive dependencies.
cargo update

git config user.name "github-actions[bot]"
git config user.email "github-actions[bot]@users.noreply.github.com"

git add Cargo.toml Cargo.lock
git commit -F commit-message

git push --force origin update-dependencies

gh pr create --fill \
    --head update-dependencies \
    --base master
