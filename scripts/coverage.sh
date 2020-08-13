#!/bin/sh

set -x -e

export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests -Cpanic=abort"
export RUSTDOCFLAGS="-Cpanic=abort"

rm -rf ./target/debug/deps/*.gcda
rm -rf ./target/debug/coverage

cargo build && cargo test

grcov ./target/debug/ -s . -t html --llvm --branch --ignore-not-existing -o ./target/debug/coverage/
open ./target/debug/coverage/index.html
