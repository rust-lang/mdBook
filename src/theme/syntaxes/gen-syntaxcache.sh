#!/bin/env bash
set -Eeuxo pipefail

bat cache --build --source=".." --target=".."
rm -f ../themes.bin
rm -f ../metadata.yaml