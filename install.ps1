# install.ps1
# Segfault's Secure SHell Utils installer for Windows

$ErrorActionPreference = "Stop"

$RepoName = "github.com/segfaultuwu/ssshu"
$BinaryName = "ssshu.exe"

Write-Host ""
Write-Host "== ssshu installer ==" -ForegroundColor Cyan
Write-Host ""

# -----------------------------
# Check cargo
# -----------------------------

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "Rust is not installed." -ForegroundColor Red
    Write-Host ""
    Write-Host "Install rustup first:"
    Write-Host "https://win.rustup.rs/"
    exit 1
}

# -----------------------------
# Check git
# -----------------------------

if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Write-Host "Git is not installed." -ForegroundColor Red
    exit 1
}

# -----------------------------
# Clone/update repo
# -----------------------------

$InstallDir = "$env:USERPROFILE\.ssshu"

if (Test-Path $InstallDir) {
    Write-Host "Updating existing repository..." -ForegroundColor Yellow

    Set-Location $InstallDir
    git pull
}
else {
    Write-Host "Cloning repository..." -ForegroundColor Yellow

    git clone https://github.com/segfaultuwu/ssshu.git $InstallDir

    Set-Location $InstallDir
}

# -----------------------------
# Build release
# -----------------------------

Write-Host ""
Write-Host "Building ssshu..." -ForegroundColor Cyan

cargo build --release

# -----------------------------
# Install binary
# -----------------------------

$TargetDir = "$env:USERPROFILE\.local\bin"

if (-not (Test-Path $TargetDir)) {
    New-Item -ItemType Directory -Path $TargetDir | Out-Null
}

Copy-Item `
    ".\target\release\$BinaryName" `
    "$TargetDir\$BinaryName" `
    -Force

# -----------------------------
# Add PATH
# -----------------------------

$CurrentUserPath = [Environment]::GetEnvironmentVariable(
    "Path",
    "User"
)

if ($CurrentUserPath -notlike "*$TargetDir*") {

    Write-Host ""
    Write-Host "Adding $TargetDir to PATH..." -ForegroundColor Yellow

    [Environment]::SetEnvironmentVariable(
        "Path",
        "$CurrentUserPath;$TargetDir",
        "User"
    )
}

# -----------------------------
# Check OpenSSH
# -----------------------------

if (-not (Get-Command ssh -ErrorAction SilentlyContinue)) {

    Write-Host ""
    Write-Host "OpenSSH Client not found." -ForegroundColor Yellow
    Write-Host "Install it with:"
    Write-Host ""
    Write-Host "Add-WindowsCapability -Online -Name OpenSSH.Client~~~~0.0.1.0"
}

# -----------------------------
# Done
# -----------------------------

Write-Host ""
Write-Host "ssshu installed successfully!" -ForegroundColor Green
Write-Host ""

Write-Host "Restart your terminal and run:"
Write-Host ""
Write-Host "ssshu --help"
Write-Host ""