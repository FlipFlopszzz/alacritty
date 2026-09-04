# One-time elevated setup (invoked by windows\dist.ps1): create the install
# directory and grant the current user modify rights.
$installDir = 'C:\Program Files\Alacritty'
New-Item -ItemType Directory -Path $installDir -Force | Out-Null
icacls $installDir /grant "$($env:USERNAME):(OI)(CI)M" | Out-Null
Write-Output "bootstrapped: $installDir"
