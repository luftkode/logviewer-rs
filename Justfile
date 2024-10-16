import 'just/default_cmd.just'
mod ci 'just/ci.just'

PROJECT_NAME := "logviewer-rs"

alias r := run
alias l := lint
alias c := check
alias ca := check-all
alias f := fmt
alias t := test

@_default:
    just --list --no-aliases

init: install-devtools
    echo "Run 'install-extra-devtools' for some adittional productivity tools that fit into the existent workflow"

[doc("Checks both native and wasm"), no-exit-message]
check-all: check check-wasm

[doc("Quickly check if it compiles without compiling (native target)"), no-exit-message]
check *ARGS:
    cargo {{check}} {{ ARGS }}

[doc("Quickly check if the WASM target compiles without compiling"), no-exit-message]
check-wasm: (check "--target wasm32-unknown-unknown")

# Get trunk: https://trunkrs.dev/guide/introduction.html
[doc("serve as a local webserver with hot reloading and logging enabled (requires trunk)")]
serve *ARGS:
    RUST_LOG=debug trunk serve {{ARGS}}

# Run as a native app with logging enabled
[no-exit-message]
run *ARGS:
    RUST_LOG=debug cargo {{run}} {{ARGS}}

# Run tests
[no-exit-message]
test *ARGS="--workspace":
    cargo {{test}} {{ARGS}}

# Lint
[no-exit-message]
lint: clippy-native clippy-wasm && fmt
    typos

[doc("Clippy linting targeting native"), no-exit-message]
clippy-native: (clippy "--workspace --tests -- -D warnings")

[doc("Clippy linting targeting WASM"), no-exit-message]
clippy-wasm:
    CLIPPY_CONF_DIR="`pwd`/lint/wasm/clippy.toml" \
    just clippy "--workspace --tests --target wasm32-unknown-unknown -- -D warnings"

[private, no-exit-message]
clippy *ARGS:
    cargo {{clippy}} {{ARGS}}

[no-exit-message]
fmt *ARGS:
    cargo fmt --all -- {{ARGS}}

# Build the documentation (use `--open` to open in the browser)
[no-exit-message]
doc *ARGS:
    cargo {{doc}} {{ ARGS }}

# Update the dependencies
[group("Dependencies"), no-exit-message]
update:
    cargo update

# Audit Cargo.lock files for crates containing security vulnerabilities
[group("Dependencies"), no-exit-message]
audit *ARGS:
    cargo audit {{ ARGS }}

# Trunk is used to serve the app with a webserver, cargo-dist is used to generate and update workflows for distributing installers for various platforms
[doc("Install the required tools for performing all dev tasks for the project")]
install-devtools:
    cargo install trunk --locked
    cargo install cargo-dist --locked
    cargo install typos-cli
    cargo install cargo-audit

# Install nice-to-have devtools
install-extra-devtools:
    cargo install cargo-nextest --locked
    cargo install cargo-limit --locked
    cargo install bacon --locked

apt-install-hdf5-header:
    sudo apt install libhdf5-dev
