<#
.SYNOPSIS
    Bump HyperHost version across all project files.

.DESCRIPTION
    Updates version in: tauri.conf.json, Cargo.toml, package.json, App.tsx
    Then optionally creates a git tag and pushes.

.EXAMPLE
    .\bump-version.ps1 0.2.0
    .\bump-version.ps1 0.2.0 -Push
#>
param(
    [Parameter(Mandatory=$true, Position=0)]
    [ValidatePattern('^\d+\.\d+\.\d+$')]
    [string]$Version,

    [switch]$Push
)

$ErrorActionPreference = "Stop"
$Root = Split-Path $PSScriptRoot -Parent

Write-Host "`n🚀 Bumping HyperHost to v$Version`n" -ForegroundColor Cyan

# --- 1. tauri.conf.json ---
$tauriConf = Join-Path $Root "src-tauri\tauri.conf.json"
$json = Get-Content $tauriConf -Raw | ConvertFrom-Json
$oldVersion = $json.version
$json.version = $Version
$json | ConvertTo-Json -Depth 10 | Set-Content $tauriConf -Encoding UTF8
Write-Host "  ✅ tauri.conf.json    $oldVersion → $Version" -ForegroundColor Green

# --- 2. Cargo.toml ---
$cargoToml = Join-Path $Root "src-tauri\Cargo.toml"
$content = Get-Content $cargoToml -Raw
$content = $content -replace '(?m)^version\s*=\s*"[^"]*"', "version = `"$Version`""
Set-Content $cargoToml $content -NoNewline -Encoding UTF8
Write-Host "  ✅ Cargo.toml         → $Version" -ForegroundColor Green

# --- 3. package.json ---
$pkgJson = Join-Path $Root "package.json"
$pkg = Get-Content $pkgJson -Raw | ConvertFrom-Json
$pkg.version = $Version
$pkg | ConvertTo-Json -Depth 10 | Set-Content $pkgJson -Encoding UTF8
Write-Host "  ✅ package.json       → $Version" -ForegroundColor Green

# --- 4. App.tsx (hardcoded version display) ---
$appTsx = Join-Path $Root "src\App.tsx"
if (Test-Path $appTsx) {
    $content = Get-Content $appTsx -Raw
    $content = $content -replace 'v\d+\.\d+\.\d+</span>', "v$Version</span>"
    Set-Content $appTsx $content -NoNewline -Encoding UTF8
    Write-Host "  ✅ App.tsx            → v$Version" -ForegroundColor Green
}

Write-Host "`n📦 All files updated to v$Version" -ForegroundColor Yellow

# --- 5. Git tag + push (optional) ---
if ($Push) {
    Write-Host "`n🔖 Creating git tag v$Version..." -ForegroundColor Cyan
    Set-Location $Root
    git add -A
    git commit -m "release: v$Version"
    git tag "v$Version"
    git push origin main --tags
    Write-Host "  ✅ Pushed v$Version to origin" -ForegroundColor Green
}

Write-Host "`n✨ Done! Next steps:" -ForegroundColor Cyan
if (-not $Push) {
    Write-Host "  1. git add -A && git commit -m 'release: v$Version'" -ForegroundColor White
    Write-Host "  2. git tag v$Version" -ForegroundColor White
    Write-Host "  3. git push origin main --tags" -ForegroundColor White
    Write-Host "`n  Or run: .\bump-version.ps1 $Version -Push" -ForegroundColor DarkGray
}
Write-Host "  → CI will build & upload latest.json with version $Version" -ForegroundColor White
Write-Host ""
