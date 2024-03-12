#!/bin/bash

# Dependencies:
# - bash
# - nix

function develop() {
  # Start Development Environment.
  nix develop \
    --experimental-features 'nix-command flakes' \
    --show-trace \
    --verbose \
    --ignore-environment \
    "."
}

