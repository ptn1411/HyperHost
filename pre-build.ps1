# pre-build.ps1 — Run before cargo tauri build to compile CLI and prepare binaries
$ErrorActionPreference = "Stop"

Write-Host "🔧 Building dh..." -ForegroundColor Cyan

# Build CLI binary (release, no GUI features)
Push-Location "$PSScriptRoot\src-tauri"
cargo build --release --no-default-features --bin dh
Pop-Location

# Copy CLI to binaries with Tauri sidecar naming convention
$source = "$PSScriptRoot\src-tauri\target\release\dh.exe"
$dest = "$PSScriptRoot\src-tauri\binaries\dh-x86_64-pc-windows-msvc.exe"

Copy-Item $source -Destination $dest -Force
Write-Host "✅ dh copied to binaries/" -ForegroundColor Green

# Verify all sidecars exist
$sidecars = @(
    "nginx-x86_64-pc-windows-msvc.exe",
    "mkcert-x86_64-pc-windows-msvc.exe",
    "dh-x86_64-pc-windows-msvc.exe"
)

Write-Host "`n📦 Sidecar binaries:" -ForegroundColor Cyan
foreach ($s in $sidecars) {
    $path = "$PSScriptRoot\src-tauri\binaries\$s"
    if (Test-Path $path) {
        $size = [math]::Round((Get-Item $path).Length / 1MB, 1)
        Write-Host "  ✓ $s (${size}MB)" -ForegroundColor Green
    } else {
        Write-Host "  ✗ $s MISSING" -ForegroundColor Red
    }
}
