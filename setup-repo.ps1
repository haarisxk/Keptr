$ErrorActionPreference = "Stop"
cd "C:\Users\Haaris\Documents\Keptr"

# 1. Get the authenticated token
$gh_exe = "C:\Program Files\GitHub CLI\gh.exe"
$git_exe = "C:\Program Files\Git\cmd\git.exe"
$token = & $gh_exe auth token
if ([string]::IsNullOrWhiteSpace($token)) {
    throw "GitHub token not found. Please authenticate first."
}

# 2. Check if the Keptr remote repository already exists, if not create it
Write-Host "Checking if Keptr repository exists on GitHub..."
try {
    Invoke-RestMethod -Uri "https://api.github.com/repos/haarisxk/Keptr" -Method Get -Headers @{Authorization="Bearer $token"} -ErrorAction Stop
    Write-Host "Repository already exists."
} catch {
    Write-Host "Creating private Keptr repository..."
    $body = @{name="Keptr"; private=$true} | ConvertTo-Json
    Invoke-RestMethod -Uri "https://api.github.com/user/repos" -Method Post -Headers @{Authorization="Bearer $token"; "Content-Type"="application/json"} -Body $body
}

# 3. Setup local Git
Write-Host "Initializing local Git..."
if (-not (Test-Path ".git")) {
    & $git_exe init
}

& $git_exe config user.name "Haaris"
& $git_exe config user.email "haaris@keptr.local"
& $git_exe add .
& $git_exe commit -m "Initial commit of Keptr Vault with secure storage, sync, and professional structure"

# 4. Push to origin with the token embedded for seamless authentication
Write-Host "Pushing to GitHub..."
& $git_exe remote remove origin -ErrorAction SilentlyContinue
& $git_exe remote add origin "https://oauth2:$($token)@github.com/haarisxk/Keptr.git"
& $git_exe push -u origin master --force

Write-Host "GitHub Setup Complete!"
