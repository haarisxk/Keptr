$ErrorActionPreference = "Stop"
$env:Path = "$env:USERPROFILE\.cargo\bin;C:\Program Files\Git\cmd;C:\Program Files\nodejs;" + $env:Path
$env:TAURI_SIGNING_PRIVATE_KEY = "dW50cnVzdGVkIGNvbW1lbnQ6IHJzaWduIGVuY3J5cHRlZCBzZWNyZXQga2V5ClJXUlRZMEl5eUZicCtFUkptMlVzbUJabjFzZnI0Y2kvazltcVowSzh0SVg0K0RES1dTOEFBQkFBQUFBQUFBQUFBQUlBQUFBQXBOc2d4Q0hYdXUraXNKT3cvb2psNmpxTHhPbEZuanBQK2pVL29aMGtzekZpSVdSVGlnTlczQmNlSDViZDA5RW1QdWVKZnkzNm1aZFEzT0dSM3I0UHpvVTFxRHFIc05ad1dSWG1CalZVQUZMUUxTUGhsek8wL3FhSGRzaHQ3MUU1WGJPR2wzR1lvVjA9Cg=="
$env:TAURI_SIGNING_PRIVATE_KEY_PASSWORD = "keptr2026"
Write-Host "=== Environment Ready ==="
Write-Host "Node: $(node --version)"
Write-Host "Cargo: $(cargo --version)"
Write-Host "=== Starting Tauri Build ==="
npm run tauri build
