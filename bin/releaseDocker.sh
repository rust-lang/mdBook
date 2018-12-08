#!/usr/bin/env bash

# USAGE: ./bin/releaseDocker.sh <ACCOUNT_NAME> <VERSION>
#
# Example usage: ./bin/releaseDocker.sh dockeruser 0.2.1

set -e

SCRIPT_DIRECTORY="$(dirname "$0")"
PROJECT_DIRECTORY="$(dirname "${SCRIPT_DIRECTORY}")"

cd "${PROJECT_DIRECTORY}"

docker build --no-cache --build-arg VERSION="${2}" -t mdbook .
docker tag mdbook "${1}/mdbook:${2}"

echo "To run image:"
echo docker run -it mdbook sh

echo "To push (publish) image"
echo docker push "${1}/mdbook:${2}"

