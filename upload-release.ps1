$ErrorActionPreference = "Stop"
cd "C:\Users\Haaris\Documents\Keptr"

$gh_exe = "C:\Program Files\GitHub CLI\gh.exe"
$token = & $gh_exe auth token

# 1. Ensure `gh release` targets the right repo
$repo = "haarisxk/Keptr"

# 2. Upload the setup files
Write-Host "Creating GitHub Release v0.3.0..."
try {
    & $gh_exe release create "v0.3.0" --repo $repo --title "Keptr v0.3.0 - Security First Vault" --notes "Initial private release of Keptr. Features Zero-Knowledge SQLite encrypted storage, advanced auto-type functionalities, Cloud file attachment syncing with persistent JWT tokens, and anti-forensics measures. 

**This software is proprietary and closed-source.**"
} catch {}

Write-Host "Uploading binaries..."
$msi = "src-tauri\target\release\bundle\msi\Keptr_0.3.0_x64_en-US.msi"
$nsis = "src-tauri\target\release\bundle\nsis\Keptr_0.3.0_x64-setup.exe"
$portable = "src-tauri\target\release\keptr.exe"

if (Test-Path $msi) { & $gh_exe release upload "v0.3.0" $msi --repo $repo --clobber }
if (Test-Path $nsis) { & $gh_exe release upload "v0.3.0" $nsis --repo $repo --clobber }
if (Test-Path $portable) { & $gh_exe release upload "v0.3.0" $portable --repo $repo --clobber }

Write-Host "Binaries uploaded successfully."
