#!/usr/bin/env sh

set -eux

rustup update
rustup toolchain install nightly
rustup component add miri --toolchain nightly

cargo fmt --check
cargo clippy --all-targets --no-default-features
cargo clippy --all-targets --all-features

MIRIFLAGS=-Zmiri-backtrace=full cargo +nightly miri test --examples --no-default-features
RUST_BACKTRACE=1 cargo test --examples
RUST_BACKTRACE=1 cargo test --examples -r --all-features
