#!/bin/bash

set -e

mkdir -p ci-tmp

ensure_cargo_bin() {
    if ! command -v "$1" &> /dev/null; then
        if [ -z "$2" ]; then
            cargo install $2
        else
            cargo install $1
        fi
    fi
}

cmp_output() {
    if ! diff -u $1 $2; then
        echo "License check failed"
        exit 1
    fi
}

ensure_cargo_bin cargo-license
ensure_cargo_bin cargo-deny

cargo license --color never > ci-tmp/LICENSE-dependencies-latest

cmp_output LICENSE-dependencies ci-tmp/LICENSE-dependencies-latest

cargo deny check
