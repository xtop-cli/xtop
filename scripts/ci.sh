#!/usr/bin/env bash
# Local CI for the xtop kernel repo (single package: library + binary).
#
# Run it locally from the repo root (the existing GitHub workflow calls the
# same script, so local and remote gates stay identical):
#
#   ./scripts/ci.sh            # run every stage
#   ./scripts/ci.sh fmt        # run one stage
#   ./scripts/ci.sh audit      # run only the structural audit
#
# Stages: fmt | clippy | check | test | no-default | audit
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [[ ! -f Cargo.toml ]]; then
    echo "[ci] no Cargo.toml at repo root; nothing to run."
    exit 0
fi

stages=(fmt clippy check test no-default audit)
requested=("$@")
if [[ ${#requested[@]} -eq 0 ]]; then
    requested=("${stages[@]}")
fi

fmt() {
    echo "==> fmt (format check)"
    cargo fmt --all -- --check
}

clippy() {
    echo "==> clippy (all targets, warnings denied)"
    cargo clippy --all-targets -- -D warnings
}

check() {
    echo "==> check"
    cargo check
}

test() {
    echo "==> test"
    cargo test
}

no_default() {
    echo "==> check core only (no default features: samurai plugin, mcp"
    echo "    extension, blocks pack and effects are all excluded)"
    cargo check --no-default-features
}

audit() {
    echo "==> structural audit (full kernel)"
    ./scripts/audit.sh
}

for stage in "${requested[@]}"; do
    case "$stage" in
        fmt) fmt ;;
        clippy) clippy ;;
        check) check ;;
        test) test ;;
        no-default) no_default ;;
        audit) audit ;;
        *)
            echo "[ci] unknown stage: $stage (expected one of: ${stages[*]})" >&2
            exit 1
            ;;
    esac
done

echo "[ci] all stages passed."
