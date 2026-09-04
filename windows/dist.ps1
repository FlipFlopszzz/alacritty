# Fork release builder: optimized dist binary + MSI installer.
#
#   powershell -File windows\dist.ps1
#
# Produces:
#   target\dist\alacritty.exe        optimized binary
#   Alacritty-dist-installer.msi     installer (repo root)
#
# Install: run the MSI manually — it upgrades in place over
# C:\Program Files\Alacritty and refreshes the Start menu entry.
#
# Requires WiX (one-time setup):
#   dotnet tool install --global wix --version 4.0.5
#   wix extension add -g WixToolset.UI.wixext/4.0.5
#   wix extension add -g WixToolset.Util.wixext/4.0.5

$ErrorActionPreference = 'Stop'
$repo = 'D:\dev\alacritty'
$wix = Join-Path $env:USERPROFILE '.dotnet\tools\wix.exe'
Set-Location $repo

cargo build --profile dist
if ($LASTEXITCODE -ne 0) { throw 'cargo build --profile dist failed' }

& $wix build -arch 'x64' `
    -ext WixToolset.UI.wixext -ext WixToolset.Util.wixext `
    -out 'Alacritty-dist-installer.msi' `
    'alacritty\windows\wix\alacritty.wxs'
if ($LASTEXITCODE -ne 0) { throw 'wix build failed' }

Write-Output "MSI: $repo\Alacritty-dist-installer.msi"
