$ErrorActionPreference = "Stop"
cd "C:\Users\Haaris\Documents\Keptr"

$gh_exe = "C:\Program Files\GitHub CLI\gh.exe"
$git_exe = "C:\Program Files\Git\cmd\git.exe"
$token = & $gh_exe auth token

Write-Host "Setting up remote origin..."
try {
    & $git_exe remote remove origin 2>$null
} catch {}

& $git_exe remote add origin "https://oauth2:$($token)@github.com/haarisxk/Keptr.git"

Write-Host "Pushing to GitHub..."
& $git_exe push -u origin master --force

Write-Host "GitHub Setup Complete!"
