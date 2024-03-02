#!/bin/bash

function develop() {
  nix develop \
    --experimental-features 'nix-command flakes' \
    --ignore-environment
    "."
}

function build() {
  nix build  \
    --experimental-features 'nix-command flakes' \
    --show-trace \
    --verbose \
    --option eval-cache false \
    -L \
    "."
}

function test() {
  nix develop \
    --experimental-features 'nix-command flakes' \
    . -c bash -c "cargo test"
}

function run() {
  nix run  \
    --experimental-features 'nix-command flakes' \
    --show-trace \
    --verbose \
    --option eval-cache false \
    -L \
    "." -- $@
}
