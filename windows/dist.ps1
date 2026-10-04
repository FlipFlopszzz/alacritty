# Fork release flow: copy the optimized dist-profile binary to the official
# Alacritty install location (C:\Program Files\Alacritty). No MSI involved.
#
#   powershell -File windows\dist.ps1
#
# First run may show one UAC prompt: it creates the install dir and grants
# this user modify rights, so later runs need no elevation.

$ErrorActionPreference = 'Stop'
$repo = 'D:\dev\alacritty'
$installDir = 'C:\Program Files\Alacritty'
$dst = Join-Path $installDir 'alacritty.exe'
Set-Location $repo

cargo build --profile dist
if ($LASTEXITCODE -ne 0) { throw 'cargo build --profile dist failed' }

if (-not (Test-Path $installDir)) {
    Write-Output 'Install dir missing, elevating once to create it and grant write access...'
    Start-Process -FilePath 'powershell' -Verb RunAs -Wait -ArgumentList `
        '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $repo 'windows\grant-install-dir.ps1')
}

Copy-Item (Join-Path $repo 'target\dist\alacritty.exe') $dst -Force

# ConPTY pair vendored from the Microsoft.Windows.Console.ConPTY NuGet
# (see windows\conpty\VERSION); conpty.dll makes alacritty spawn OpenConsole.exe.
$conptyDir = Join-Path $repo 'windows\conpty'
Copy-Item (Join-Path $conptyDir 'conpty.dll') $installDir -Force
Copy-Item (Join-Path $conptyDir 'OpenConsole.exe') $installDir -Force

& $dst -V
Write-Output "installed: $dst (+ conpty.dll, OpenConsole.exe)"
