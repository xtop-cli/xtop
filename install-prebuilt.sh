#!/usr/bin/env bash
# xtop - prebuilt installer
# Downloads a precompiled binary from GitHub Releases, verifies it and
# installs it. No Rust toolchain and no compilation required.
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/xtop-cli/xtop/main/install-prebuilt.sh | bash
#   bash install-prebuilt.sh --version 0.1.0
#   bash install-prebuilt.sh --prefix /usr/local --yes
#
# The installer verifies the release SHA256SUMS and, when cosign is available,
# the keyless signature bundle. Pass --require-signature to make cosign
# mandatory.

set -euo pipefail

REPO="xtop-cli/xtop"
PROJECT="xtop"
PROJECT_DESC="cross-platform TUI system monitor (prebuilt binary)"
REPO_URL="https://github.com/${REPO}"
REPO_API="https://api.github.com/repos/${REPO}"
REPO_RAW="https://raw.githubusercontent.com/${REPO}/main"
SIGNING_WORKFLOW="${REPO_URL}/.github/workflows/release.yml"
SIGNING_OIDC_ISSUER="https://token.actions.githubusercontent.com"

PREFIX="${PREFIX:-${HOME}/.local}"
BIN_DIR="${BIN_DIR:-${PREFIX}/bin}"
BIN_DIR_OVERRIDE=0
VERSION=""
FLAG_YES=0
FLAG_VERBOSE=1
FLAG_NO_CHECKSUM=0
FLAG_REQUIRE_SIGNATURE=0

TEMP_DIR=""
OS_NAME=""
ARCH_NAME=""
TARGET_TRIPLE=""

log() { [ "${FLAG_VERBOSE}" -eq 1 ] && printf '[%s] %s\n' "${PROJECT}" "$*"; }
ok() { printf '[%s] %s\n' "${PROJECT}" "$*"; }
warn() { printf '[%s] warning: %s\n' "${PROJECT}" "$*" >&2; }
die() { printf '[%s] error: %s\n' "${PROJECT}" "$*" >&2; exit 1; }

cleanup() {
    local code=$?
    if [ -n "${TEMP_DIR}" ] && [ -d "${TEMP_DIR}" ]; then
        rm -rf "${TEMP_DIR}"
    fi
    exit "${code}"
}
trap cleanup EXIT INT TERM HUP

usage() {
    cat <<EOF
${PROJECT} - ${PROJECT_DESC}

Usage:
  curl -fsSL ${REPO_RAW}/install-prebuilt.sh | bash
  bash install-prebuilt.sh [options]

Options:
  -h, --help              Show this help and exit
  -p, --prefix <dir>      Install prefix (default: \${HOME}/.local)
  -b, --bin-dir <dir>     Binary directory (default: \${PREFIX}/bin)
  -y, --yes               Non-interactive (accepted for scripting)
  -q, --quiet             Minimal output
      --version <ver>     Install a specific version (e.g. 0.1.0); default: latest
      --no-checksum       Disable SHA-256 and signature verification (dangerous)
      --require-signature Abort unless cosign verifies the release signature

Environment variables:
  PREFIX                  Same as --prefix
  BIN_DIR                 Same as --bin-dir

Verification:
  The installer always downloads the release SHA256SUMS and refuses to
  install when the archive is not listed or the hash does not match. When
  cosign is installed and the release publishes SHA256SUMS.bundle, the
  keyless signature is verified too. Without cosign the install continues
  after the SHA-256 check unless --require-signature is given.

Examples:
  # Latest release
  curl -fsSL ${REPO_RAW}/install-prebuilt.sh | bash

  # Specific version
  curl -fsSL ${REPO_RAW}/install-prebuilt.sh | bash -s -- --version 0.1.0

  # System-wide, non-interactive
  bash install-prebuilt.sh --prefix /usr/local --yes
EOF
    exit 0
}

parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            -h|--help) usage ;;
            -y|--yes) FLAG_YES=1 ;;
            -q|--quiet) FLAG_VERBOSE=0 ;;
            --no-checksum) FLAG_NO_CHECKSUM=1 ;;
            --require-signature) FLAG_REQUIRE_SIGNATURE=1 ;;
            --version)
                shift
                [ $# -gt 0 ] || die "--version needs a value"
                VERSION="$1"
                ;;
            -p|--prefix)
                shift
                [ $# -gt 0 ] || die "--prefix needs a value"
                PREFIX="$1"
                [ "${BIN_DIR_OVERRIDE}" -eq 1 ] || BIN_DIR="${PREFIX}/bin"
                ;;
            -b|--bin-dir)
                shift
                [ $# -gt 0 ] || die "--bin-dir needs a value"
                BIN_DIR="$1"
                BIN_DIR_OVERRIDE=1
                ;;
            --) shift; break ;;
            -*) die "unknown option: $1 (use --help)" ;;
            *) break ;;
        esac
        shift
    done
}

detect_platform() {
    case "$(uname -s)" in
        Linux) OS_NAME="linux" ;;
        Darwin) OS_NAME="macos" ;;
        CYGWIN*|MINGW*|MSYS*)
            die "On Windows use PowerShell: irm ${REPO_RAW}/install-prebuilt.ps1 | iex"
            ;;
        *) die "unsupported operating system: $(uname -s)" ;;
    esac

    case "$(uname -m)" in
        x86_64|amd64) ARCH_NAME="x86_64" ;;
        aarch64|arm64) ARCH_NAME="aarch64" ;;
        *) die "unsupported architecture: $(uname -m)" ;;
    esac

    case "${OS_NAME}-${ARCH_NAME}" in
        linux-x86_64)
            if is_musl; then
                TARGET_TRIPLE="x86_64-unknown-linux-musl"
            else
                TARGET_TRIPLE="x86_64-unknown-linux-gnu"
            fi
            ;;
        linux-aarch64)
            is_musl && die "no prebuilt musl binary for aarch64 Linux; use install.sh (builds from source)"
            TARGET_TRIPLE="aarch64-unknown-linux-gnu"
            ;;
        macos-x86_64) TARGET_TRIPLE="x86_64-apple-darwin" ;;
        macos-aarch64) TARGET_TRIPLE="aarch64-apple-darwin" ;;
        *) die "no prebuilt binary for ${OS_NAME}-${ARCH_NAME}" ;;
    esac

    log "target: ${TARGET_TRIPLE}"
}

is_musl() {
    if [ -e /lib/ld-musl-x86_64.so.1 ] || [ -e /lib/ld-musl-aarch64.so.1 ]; then
        return 0
    fi
    if command -v ldd >/dev/null 2>&1; then
        ldd --version 2>&1 | grep -qi musl
        return $?
    fi
    return 1
}

validate_version() {
    printf '%s\n' "$1" | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+([-.][0-9A-Za-z.-]+)?$'
}

resolve_version() {
    local url json tag
    if [ -n "${VERSION}" ]; then
        validate_version "${VERSION}" \
            || die "invalid --version '${VERSION}': expected a dotted version such as 0.1.0"
        url="${REPO_API}/releases/tags/v${VERSION}"
    else
        url="${REPO_API}/releases/latest"
    fi

    log "resolving release..."
    json="$(download_stdout "${url}")" || die "cannot reach ${url}"

    tag="$(printf '%s' "${json}" \
        | grep -o '"tag_name"[[:space:]]*:[[:space:]]*"[^"]*"' \
        | head -n 1 \
        | sed 's/.*"tag_name"[[:space:]]*:[[:space:]]*"//; s/"$//')" || tag=""
    [ -n "${tag}" ] || die "release metadata does not include a tag_name"

    tag="${tag#v}"
    validate_version "${tag}" || die "release tag '${tag}' is not a valid version"

    if [ -n "${VERSION}" ] && [ "${VERSION}" != "${tag}" ]; then
        die "requested v${VERSION} does not match release tag v${tag}"
    fi

    VERSION="${tag}"
    log "release: v${VERSION}"
}

download_stdout() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL --retry 3 --max-time 30 "$1"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO- --tries=3 --timeout=30 "$1"
    else
        die "curl or wget is required"
    fi
}

download() {
    if command -v curl >/dev/null 2>&1; then
        curl -fsSL --retry 3 --max-time 300 "$1" -o "$2"
    elif command -v wget >/dev/null 2>&1; then
        wget -qO "$2" --tries=3 --timeout=300 "$1"
    else
        die "curl or wget is required"
    fi
}

sha256_file() {
    local out
    if command -v sha256sum >/dev/null 2>&1; then
        out="$(sha256sum "$1")" || return 1
        printf '%s\n' "${out%% *}"
    elif command -v shasum >/dev/null 2>&1; then
        out="$(shasum -a 256 "$1")" || return 1
        printf '%s\n' "${out%% *}"
    elif command -v openssl >/dev/null 2>&1; then
        out="$(openssl dgst -sha256 "$1")" || return 1
        printf '%s\n' "${out##* }"
    else
        return 1
    fi
}

verify_checksum() {
    local archive="$1" sums="$2" expected actual
    expected="$(awk -v f="$(basename "${archive}")" '$2 == f { print $1; exit }' "${sums}")"
    [ -n "${expected}" ] || die "SHA256SUMS does not list $(basename "${archive}")"

    actual="$(sha256_file "${archive}")" || die "no SHA-256 tool found (sha256sum, shasum or openssl)"
    [ "${expected}" = "${actual}" ] || die "checksum mismatch for $(basename "${archive}")"
    ok "checksum verified"
}

verify_signature() {
    local sums="$1"
    local bundle_url="${REPO_URL}/releases/download/v${VERSION}/SHA256SUMS.bundle"
    local bundle="${TEMP_DIR}/SHA256SUMS.bundle"

    if ! download "${bundle_url}" "${bundle}" 2>/dev/null || [ ! -s "${bundle}" ]; then
        if [ "${FLAG_REQUIRE_SIGNATURE}" -eq 1 ]; then
            die "--require-signature was given but SHA256SUMS.bundle is missing for v${VERSION}"
        fi
        warn "signature bundle unavailable; SHA-256 only"
        return 0
    fi

    if ! command -v cosign >/dev/null 2>&1; then
        if [ "${FLAG_REQUIRE_SIGNATURE}" -eq 1 ]; then
            die "--require-signature was given but cosign is not installed"
        fi
        warn "cosign is not installed; signature NOT verified (SHA-256 already checked)"
        return 0
    fi

    cosign verify-blob \
        --bundle "${bundle}" \
        --certificate-identity "${SIGNING_WORKFLOW}@refs/tags/v${VERSION}" \
        --certificate-oidc-issuer "${SIGNING_OIDC_ISSUER}" \
        "${sums}" >/dev/null 2>&1 \
        || die "cosign signature verification failed"
    ok "signature verified"
}

install_binary() {
    local archive="$1"
    local stage="${TEMP_DIR}/stage"
    local bin dest tmp

    mkdir -p "${stage}"
    tar -xzf "${archive}" -C "${stage}" || die "cannot extract $(basename "${archive}")"

    bin="$(find "${stage}" -type f -name "${PROJECT}" | head -n 1)"
    [ -n "${bin}" ] || die "archive does not contain ${PROJECT}"

    mkdir -p "${BIN_DIR}"
    dest="${BIN_DIR}/${PROJECT}"
    tmp="${BIN_DIR}/.${PROJECT}.tmp.$$"

    cp "${bin}" "${tmp}" || die "cannot write to ${BIN_DIR}"
    chmod 755 "${tmp}"
    mv -f "${tmp}" "${dest}" || die "cannot install ${dest}"

    ok "installed: ${dest}"
}

main() {
    local asset asset_url sums_url

    parse_args "$@"
    detect_platform
    resolve_version

    TEMP_DIR="$(mktemp -d "${TMPDIR:-/tmp}/${PROJECT}.XXXXXX")"
    asset="${PROJECT}-${VERSION}-${TARGET_TRIPLE}.tar.gz"
    asset_url="${REPO_URL}/releases/download/v${VERSION}/${asset}"
    sums_url="${REPO_URL}/releases/download/v${VERSION}/SHA256SUMS"

    log "downloading ${asset}..."
    download "${asset_url}" "${TEMP_DIR}/${asset}" || die "download failed: ${asset_url}"

    if [ "${FLAG_NO_CHECKSUM}" -eq 1 ]; then
        warn "checksum and signature verification disabled (--no-checksum)"
    else
        download "${sums_url}" "${TEMP_DIR}/SHA256SUMS" \
            || die "cannot download SHA256SUMS for v${VERSION}"
        verify_checksum "${TEMP_DIR}/${asset}" "${TEMP_DIR}/SHA256SUMS"
        verify_signature "${TEMP_DIR}/SHA256SUMS"
    fi

    install_binary "${TEMP_DIR}/${asset}"

    case ":${PATH}:" in
        *":${BIN_DIR}:"*) ;;
        *)
            warn "${BIN_DIR} is not on PATH"
            printf '  add it to your shell profile:\n    export PATH="%s:$PATH"\n' "${BIN_DIR}" >&2
            ;;
    esac

    ok "installation complete. Run '${PROJECT}' to start."
}

main "$@"
