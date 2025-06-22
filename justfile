# determine the current Minimum Supported Rust Version for sic
msrv:
    cargo install cargo-msrv
    cargo msrv find --output-format json --all-features --write-msrv

# format all workspace packages
fmt:
    cargo fmt --all

# run linter on all workspace packages
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# run tests in workspace
test:
    cargo test --all-features --all

deny:
    cargo deny --all-features check

# general check to run prior to committing source code
pre-commit:
    just fmt
    just lint
    just test
    just deny

# package a release for the current platform
pack-release:
    cargo run -p pack-release

publish-workspace new_version:
    cargo install cargo-publish-workspace
    cargo publish-workspace --new-version {{ new_version }}

# publish the workspace with a new workspace version, and package the result for the current platform
publish new_version:
    just publish-workspace {{ new_version }}
    just pack-release
