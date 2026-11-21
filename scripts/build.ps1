# Release build: runs tests, builds the signed app, and puts the installer in dist/
# Usage: npm run release
$ErrorActionPreference = "Stop"
Set-Location (Join-Path $PSScriptRoot "..")
$env:PATH = "$HOME\.cargo\bin;$env:PATH"

$version = (Get-Content src-tauri\tauri.conf.json -Raw | ConvertFrom-Json).version

if (-not (Test-Path node_modules)) { npm install --no-fund --no-audit; if (-not $?) { exit 1 } }

# The plugin is bundled with the app, so it has to exist before anything compiles.
node scripts/build-plugin.mjs
if ($LASTEXITCODE -ne 0) { throw "plugin build failed" }

cargo test --manifest-path src-tauri\Cargo.toml --quiet
if ($LASTEXITCODE -ne 0) { throw "tests failed" }

# Update packages are signed with the key in %USERPROFILE%\.tauri (CI uses a repo secret).
$key = Join-Path $HOME ".tauri\ultimatepresence.key"
if (-not $env:TAURI_SIGNING_PRIVATE_KEY -and (Test-Path $key)) {
  $env:TAURI_SIGNING_PRIVATE_KEY = Get-Content $key -Raw
  $env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = ""
}

npx tauri build
if ($LASTEXITCODE -ne 0) { throw "build failed" }

New-Item -ItemType Directory -Force dist | Out-Null
Copy-Item "src-tauri\target\release\bundle\nsis\UltimatePresence_${version}_x64-setup.exe" dist\ -Force

Get-ChildItem dist | Select-Object Name, @{ n = "Size (MB)"; e = { [math]::Round($_.Length / 1MB, 2) } } | Format-Table
