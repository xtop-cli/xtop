# xtop installer for Windows
# Installs xtop by building from source (requires Rust/Cargo)
#
# Usage:
#   .\install.ps1                     # default build
#   .\install.ps1 -RuntimeWidgets     # also enable plugin-wasm + plugin-external
#                                     # (sandboxed .wasm widgets and helper
#                                     # processes in any language)

param(
    [switch]$RuntimeWidgets
)

$ErrorActionPreference = "Stop"

$AppName = "xtop"
$RepoUrl = "https://github.com/xtop-cli/xtop.git"
$InstallDir = "$env:USERPROFILE\.cargo\bin" # Standard Cargo bin location

Write-Host "Installing $AppName..." -ForegroundColor Green

# Check for Cargo
if (-not (Get-Command "cargo" -ErrorAction SilentlyContinue)) {
    Write-Host "Error: Cargo (Rust) is not installed." -ForegroundColor Red
    Write-Host "Please install Rust first from https://rustup.rs/"
    exit 1
}

# Create temp directory
$TempDir = Join-Path $env:TEMP "xtop_install"
if (Test-Path $TempDir) { Remove-Item -Recurse -Force $TempDir }
New-Item -ItemType Directory -Force -Path $TempDir | Out-Null

Write-Host "Working in $TempDir"

# Clone or Copy Source
if (Test-Path "Cargo.toml") {
    Write-Host "Detected local source, building from current directory..." -ForegroundColor Cyan
    Copy-Item -Recurse . "$TempDir\$AppName"
} elseif (Get-Command "git" -ErrorAction SilentlyContinue) {
    Write-Host "Cloning repository..." -ForegroundColor Cyan
    git clone --depth 1 "$RepoUrl" "$TempDir\$AppName"
    if ($LASTEXITCODE -ne 0) {
        Write-Host "Failed to clone repository." -ForegroundColor Red
        exit 1
    }
} else {
    Write-Host "Error: git is not installed and no local source found." -ForegroundColor Red
    exit 1
}

Set-Location "$TempDir\$AppName"

Write-Host "Building $AppName..." -ForegroundColor Cyan
$FeatureArgs = @()
if ($RuntimeWidgets) {
    $FeatureArgs = @("--features", "plugin-wasm,plugin-external")
}
cargo build --release @FeatureArgs

if ($LASTEXITCODE -ne 0) {
    Write-Host "Build failed." -ForegroundColor Red
    exit 1
}

# Ensure Install Dir exists (it should if cargo is installed)
if (-not (Test-Path $InstallDir)) {
    New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null
}

$BinaryPath = "target\release\$AppName.exe"
if (Test-Path $BinaryPath) {
    Write-Host "Installing binary to $InstallDir..." -ForegroundColor Cyan
    Copy-Item -Force $BinaryPath "$InstallDir\$AppName.exe"
    
    Write-Host "$AppName installed successfully!" -ForegroundColor Green
    Write-Host "Run it with: $AppName"
} else {
    Write-Host "Error: Compiled binary not found." -ForegroundColor Red
    exit 1
}

# Cleanup
Set-Location $env:USERPROFILE
Remove-Item -Recurse -Force $TempDir
