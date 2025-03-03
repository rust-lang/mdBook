#!/usr/bin/env bash
# Test selected downstream crates

set -e

root=$(pwd)
echo $root
folder="../mdbook-downstream"
projects=(
    "https://github.com/tommilligan/mdbook-admonish"
    "https://github.com/lambdalisue/rs-mdbook-alerts"
)


mkdir -p $folder

for project in ${projects[*]}
do
    echo "project: $project"
    name=$(basename $project)
    cd "$root/$folder"
    echo $name
    if [ ! -d $name ]; then
        git clone $project
    fi

    cd $name
    git pull
    cargo add mdbook --path $root
    cargo test
    git checkout .

done

