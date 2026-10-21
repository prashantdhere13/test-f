# Build the ffplayout engine for Windows (MSVC).
# Requires: Rust MSVC toolchain + Visual Studio C++ ("Desktop development with C++") workload.
# Run from the repo root (D:\ffplayout-master\ffplayout-master) or any cwd.

$ErrorActionPreference = "Stop"

# 1. Make sure the MSVC toolchain is the default (needed for link.exe resolution)
write-host "==> Ensuring MSVC toolchain is default"
rustup default stable-x86_64-pc-windows-msvc
rustup target add x86_64-pc-windows-msvc

# 2. Build the engine release binary
Write-Host "==> Building ffplayout (release)..."
cargo build --release -p ffplayout

if ($LASTEXITCODE -ne 0) {
    Write-Host "BUILD FAILED"
    exit $LASTEXITCODE
}

$exe = Join-Path (Get-Location) "target\release\ffplayout.exe"
if (Test-Path $exe) {
    Write-Host ""
    Write-Host "SUCCESS: $exe"
    Write-Host "Run it with:  & `"$exe`" -i   (to initialize defaults)"
} else {
    Write-Host "Build finished but $exe was not found at expected path."
}
