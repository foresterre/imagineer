import '.justfiles/dav1d.just'
import '.justfiles/dav1d_prerequisites.just'

set windows-shell := ["pwsh", "-NoLogo", "-Command"]

default:
    @just --choose

install-dav1d-prerequisites:
    just _install-dav1d-prerequisites-{{ os() }}

install-dav1d:
    just _install-dav1d-{{ os() }}

# format all workspace packages
fmt:
    cargo fmt --all

# run linter on all workspace packages
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# run tests in workspace
test:
    cargo test --all-features --all

# determine the current Minimum Supported Rust Version for sic
msrv:
    cargo install cargo-msrv
    cargo msrv find --output-format json --all-features --write-msrv

deny:
    cargo deny --all-features check

# general check to run prior to committing source code
pre-commit:
    just fmt
    just lint
    just test
    just deny

bump-version new_version:
    sed -i.bak -E \
        -e '/^\[workspace\.package\]$/,/^\[/ s/^version = "[^"]*"/version = "{{ new_version }}"/' \
        -e '/^\[workspace\.dependencies\]$/,/^\[/ s/^(sic_[a-z_]+ = [{] version = ")[^"]*"/\1{{ new_version }}"/' \
        Cargo.toml
    rm Cargo.toml.bak
    cargo update --workspace

update-changelog new_version:
    grep -q '^## Unreleased$' CHANGELOG.md || { echo "error: CHANGELOG.md has no '## Unreleased' header" >&2; exit 1; }
    sed -i.bak -e '/^## Unreleased$/{p;s/.*//;p;s/.*/## {{ new_version }} - {{ datetime("%Y-%m-%d") }}/;}' CHANGELOG.md
    rm CHANGELOG.md.bak

publish-workspace new_version:
    just update-changelog {{ new_version }}
    just bump-version {{ new_version }}
    git commit --only -m "{{ new_version }}" -- Cargo.toml Cargo.lock CHANGELOG.md
    cargo publish --workspace
    git tag v{{ new_version }}

release new_version:
    just publish-workspace {{ new_version }}
    git push origin main v{{ new_version }}
    gh release create v{{ new_version }} --title {{ new_version }} --generate-notes
