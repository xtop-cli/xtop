#!/bin/bash

# xtop installer
# Installs xtop by building from source with automatic dependency detection and installation

set -euo pipefail

APP_NAME="xtop"
REPO_URL="https://github.com/xtop-cli/xtop.git"
INSTALL_DIR="/usr/local/bin"
VERSION="0.1.0"
BUILD_FEATURES="" # set by --with-runtime-widgets

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }

# Detect package manager and distribution
detect_distro() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        DISTRO="$ID"
        DISTRO_LIKE="${ID_LIKE:-$ID}"
    elif [ -f /etc/arch-release ]; then
        DISTRO="arch"
        DISTRO_LIKE="arch"
    elif [ -f /etc/debian_version ]; then
        DISTRO="debian"
        DISTRO_LIKE="debian"
    elif [ -f /etc/redhat-release ]; then
        DISTRO="rhel"
        DISTRO_LIKE="rhel fedora"
    else
        DISTRO="unknown"
        DISTRO_LIKE="unknown"
    fi
    
    # Detect package manager
    if command -v pacman &> /dev/null; then
        PKG_MANAGER="pacman"
        PKG_INSTALL="sudo pacman -S --noconfirm --needed"
        PKG_UPDATE="sudo pacman -Sy"
    elif command -v apt-get &> /dev/null; then
        PKG_MANAGER="apt"
        PKG_INSTALL="sudo apt-get install -y"
        PKG_UPDATE="sudo apt-get update"
    elif command -v dnf &> /dev/null; then
        PKG_MANAGER="dnf"
        PKG_INSTALL="sudo dnf install -y"
        PKG_UPDATE="sudo dnf check-update || true"
    elif command -v yum &> /dev/null; then
        PKG_MANAGER="yum"
        PKG_INSTALL="sudo yum install -y"
        PKG_UPDATE="sudo yum check-update || true"
    elif command -v zypper &> /dev/null; then
        PKG_MANAGER="zypper"
        PKG_INSTALL="sudo zypper install -y"
        PKG_UPDATE="sudo zypper refresh"
    elif command -v apk &> /dev/null; then
        PKG_MANAGER="apk"
        PKG_INSTALL="sudo apk add"
        PKG_UPDATE="sudo apk update"
    else
        PKG_MANAGER="unknown"
    fi
    
    log_info "Detected distribution: $DISTRO (package manager: $PKG_MANAGER)"
}

# Get package names for different distros
get_build_deps() {
    case "$PKG_MANAGER" in
        pacman)
            echo "base-devel git"
            ;;
        apt)
            echo "build-essential git"
            ;;
        dnf|yum)
            echo "gcc gcc-c++ make git"
            ;;
        zypper)
            echo "gcc gcc-c++ make git"
            ;;
        apk)
            echo "build-base git"
            ;;
        *)
            echo ""
            ;;
    esac
}

# Check if running as root
check_root_for_deps() {
    if [ "$EUID" -ne 0 ] && ! sudo -n true 2>/dev/null; then
        log_warn "Installing dependencies may require sudo password"
    fi
}

# Install build dependencies
install_build_deps() {
    local deps
    deps=$(get_build_deps)
    
    if [ -z "$deps" ]; then
        log_warn "Unknown package manager. Please install build dependencies manually."
        log_warn "Required: git and a Rust toolchain (rustup installs it automatically)"
        return 1
    fi
    
    log_info "Installing build dependencies: $deps"
    check_root_for_deps
    
    # Update package database
    $PKG_UPDATE 2>/dev/null || true
    
    # Install packages
    # shellcheck disable=SC2086
    $PKG_INSTALL $deps
    
    log_success "Build dependencies installed"
}

# Check if git is installed
check_git() {
    if command -v git &> /dev/null; then
        log_success "git is installed"
        return 0
    else
        log_warn "git is not installed"
        return 1
    fi
}

# Check if Rust/Cargo is installed
check_rust() {
    if command -v cargo &> /dev/null; then
        local version
        version=$(cargo --version)
        log_success "Cargo is installed: $version"
        return 0
    else
        log_warn "Cargo (Rust) is not installed"
        return 1
    fi
}

# Install Rust via rustup
install_rust() {
    log_info "Installing Rust via rustup..."
    
    if command -v rustup &> /dev/null; then
        log_info "rustup found, updating..."
        rustup update stable
    else
        # Install rustup (handle both direct execution and curl|bash)
        # Download the script first, then execute it
        local rustup_script
        rustup_script=$(mktemp)
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs -o "$rustup_script"
        chmod +x "$rustup_script"
        
        # Run with -y for non-interactive mode
        sh "$rustup_script" -y --default-toolchain stable
        rm -f "$rustup_script"
        
        # Add cargo to PATH for current session (avoid 'source' which can cause issues with curl|bash)
        export PATH="$HOME/.cargo/bin:$PATH"
    fi
    
    # Verify installation
    if command -v cargo &> /dev/null; then
        log_success "Rust installed successfully"
        return 0
    else
        log_error "Failed to install Rust"
        return 1
    fi
}

# Check all dependencies
check_all_deps() {
    log_info "Checking dependencies..."
    local missing=0
    
    detect_distro
    
    if ! check_git; then
        ((missing++))
    fi
    
    if ! check_rust; then
        ((missing++))
    fi
    
    # A C toolchain is not required: xtop and its dependencies are pure Rust.
    # git is checked above; cargo is the only remaining hard requirement.
    if [ $missing -gt 0 ]; then
        log_warn "$missing dependencies missing"
        return 1
    else
        log_success "All dependencies satisfied"
        return 0
    fi
}

# Install all required dependencies
install_all_deps() {
    log_info "Installing all dependencies..."
    detect_distro
    
    # Install build dependencies first
    install_build_deps || log_warn "Could not install some build deps, continuing..."
    
    # Install Rust if needed
    if ! check_rust; then
        install_rust
    fi
    
    log_success "All dependencies installed"
}

# Build and install xtop
do_install() {
    local source_dir=""
    local cleanup_temp=false
    
    echo -e "${BOLD}${GREEN}"
    echo "╔══════════════════════════════════════╗"
    echo "║         Installing $APP_NAME            ║"
    echo "╚══════════════════════════════════════╝"
    echo -e "${NC}"
    
    detect_distro
    
    # Check and install dependencies if missing
    if ! check_all_deps; then
        log_info "Installing missing dependencies..."
        install_all_deps
    fi
    
    # Verify cargo is available
    if ! command -v cargo &> /dev/null; then
        log_error "Cargo is still not available after installation attempts"
        log_error "Please install Rust manually: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
        exit 1
    fi
    
    # Determine source directory
    if [ -f "Cargo.toml" ] && grep -q "name.*=.*\"$APP_NAME\"" Cargo.toml 2>/dev/null; then
        # Running from project directory
        log_info "Building from current directory..."
        source_dir="$(pwd)"
    else
        # Clone from remote
        log_info "Cloning repository..."
        TEMP_DIR=$(mktemp -d)
        cleanup_temp=true
        
        if ! git clone --depth 1 "$REPO_URL" "$TEMP_DIR/$APP_NAME"; then
            log_error "Failed to clone repository from $REPO_URL"
            rm -rf "$TEMP_DIR"
            exit 1
        fi
        
        source_dir="$TEMP_DIR/$APP_NAME"
    fi
    
    # Build the project
    log_info "Building $APP_NAME (this may take a moment)..."
    cd "$source_dir"
    
    if ! cargo build --release $BUILD_FEATURES; then
        log_error "Build failed"
        [ "$cleanup_temp" = true ] && rm -rf "$TEMP_DIR"
        exit 1
    fi
    
    log_success "Build completed"
    
    # Install binary
    log_info "Installing binary to $INSTALL_DIR..."
    
    if [ ! -f "target/release/$APP_NAME" ]; then
        log_error "Binary not found at target/release/$APP_NAME"
        [ "$cleanup_temp" = true ] && rm -rf "$TEMP_DIR"
        exit 1
    fi
    
    if [ -w "$INSTALL_DIR" ]; then
        cp "target/release/$APP_NAME" "$INSTALL_DIR/$APP_NAME"
        chmod +x "$INSTALL_DIR/$APP_NAME"
    else
        log_info "Requesting sudo to install to $INSTALL_DIR"
        sudo cp "target/release/$APP_NAME" "$INSTALL_DIR/$APP_NAME"
        sudo chmod +x "$INSTALL_DIR/$APP_NAME"
    fi
    
    # Cleanup
    [ "$cleanup_temp" = true ] && rm -rf "$TEMP_DIR"
    
    echo ""
    log_success "$APP_NAME installed successfully!"
    echo -e "Run it with: ${BOLD}$APP_NAME${NC}"
    
    # Verify binary exists (don't run it as it may be interactive)
    if [ -x "$INSTALL_DIR/$APP_NAME" ]; then
        log_success "Binary installed at $INSTALL_DIR/$APP_NAME"
    fi
}

# Uninstall xtop
do_uninstall() {
    log_info "Uninstalling $APP_NAME..."
    
    if [ -f "$INSTALL_DIR/$APP_NAME" ]; then
        if [ -w "$INSTALL_DIR" ]; then
            rm -f "$INSTALL_DIR/$APP_NAME"
        else
            sudo rm -f "$INSTALL_DIR/$APP_NAME"
        fi
        log_success "$APP_NAME uninstalled successfully"
    else
        log_warn "$APP_NAME is not installed in $INSTALL_DIR"
    fi
}

# Show help
show_help() {
    echo -e "${BOLD}$APP_NAME Installer v$VERSION${NC}"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --help, -h        Show this help message"
    echo "  --check-deps      Check if all dependencies are installed"
    echo "  --install-deps    Install all required dependencies"
    echo "  --with-runtime-widgets  Build with the runtime widget hosts"
    echo "                    (plugin-wasm + plugin-external: sandboxed .wasm"
    echo "                    widgets and helper processes in any language)"
    echo "  --uninstall       Uninstall $APP_NAME"
    echo "  --version, -v     Show installer version"
    echo ""
    echo "Without options, the installer will:"
    echo "  1. Detect your distribution and package manager"
    echo "  2. Check and install required dependencies"
    echo "  3. Clone the repository (if not in project directory)"
    echo "  4. Build and install $APP_NAME to $INSTALL_DIR"
    echo ""
    echo "Supported distributions:"
    echo "  - Arch Linux (and derivatives like Manjaro, EndeavourOS)"
    echo "  - Debian/Ubuntu (and derivatives)"
    echo "  - Fedora/RHEL/CentOS"
    echo "  - openSUSE"
    echo "  - Alpine Linux"
}

# Parse arguments
main() {
    case "${1:-}" in
        --help|-h)
            show_help
            ;;
        --version|-v)
            echo "$APP_NAME installer v$VERSION"
            ;;
        --check-deps)
            check_all_deps
            ;;
        --install-deps)
            install_all_deps
            ;;
        --with-runtime-widgets)
            BUILD_FEATURES="--features plugin-wasm,plugin-external"
            do_install
            ;;
        --uninstall)
            do_uninstall
            ;;
        "")
            do_install
            ;;
        *)
            log_error "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
}

main "$@"
exit 0
