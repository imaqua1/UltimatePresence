# Release build: runs tests, builds the app, and collects the installer + portable exe in dist/.
# Usage: powershell -ExecutionPolicy Bypass -File build.ps1
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot
$env:PATH = "$HOME\.cargo\bin;$env:PATH"

$version = (Get-Content src-tauri\tauri.conf.json -Raw | ConvertFrom-Json).version

if (-not (Test-Path node_modules)) { npm install --no-fund --no-audit; if (-not $?) { exit 1 } }

cargo test --manifest-path src-tauri\Cargo.toml --quiet
if ($LASTEXITCODE -ne 0) { throw "tests failed" }

# Discord images bundled with the app (each app's Discord app tab → "Open icons folder").
node tools/build-plugin.mjs
if ($LASTEXITCODE -ne 0) { throw "plugin build failed" }
node tools/make-state-icons.mjs
if ($LASTEXITCODE -ne 0) { throw "icon generation failed" }
node tools/make-blender-icons.mjs
if ($LASTEXITCODE -ne 0) { throw "icon generation failed" }

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
Copy-Item "src-tauri\target\release\ultimate-presence.exe" "dist\UltimatePresence-$version-portable.exe" -Force

Get-ChildItem dist | Select-Object Name, @{ n = "Size (MB)"; e = { [math]::Round($_.Length / 1MB, 2) } } | Format-Table
