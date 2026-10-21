<#
.SYNOPSIS
    Copy the ffplayout backend + frontend from this Windows machine to an Ubuntu server.

.DESCRIPTION
    Two separate transfers, because demo_data/ is 14 GB of media and HLS output:

      1. CODE   - server.py, db_admin.py, frontend/, assets/, package.json ...
                  ~30 MB. No demo_data, no node_modules, no target/, no .venv/.
      2. DATA   - optional. Just db.json + playlists/ (~2 MB) so the existing
                  channels, users and playlists carry over to Ubuntu.
                  Media (storage/, 14 GB) is NOT copied - repoint the channel
                  storage path at your Linux media share instead.

    The tar|ssh pipe is run inside cmd.exe on purpose: PowerShell 5.1 pipelines
    coerce binary to strings and would corrupt the archive.

.PARAMETER Server
    Target as user@host, e.g. root@192.168.1.50

.PARAMETER RemoteDir
    Where the code lands. Default /root/ffplayout

.PARAMETER IncludeData
    Also copy demo_data/db.json, the small state json files and playlists/

.PARAMETER IncludeMedia
    Also copy demo_data/storage/  - SLOW, ~14 GB.

.EXAMPLE
    .\copy-to-ubuntu.ps1 -Server root@192.168.1.50
    .\copy-to-ubuntu.ps1 -Server root@192.168.1.50 -IncludeData
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)][string]$Server,
    [string]$RemoteDir = '/root/ffplayout',
    [switch]$IncludeData,
    [switch]$IncludeMedia
)

$ErrorActionPreference = 'Stop'
$Src = 'D:\ffplayout-master\ffplayout-master'
$DeployPack = 'D:\ffplayout-ubuntu'

if (-not (Test-Path $Src)) { throw "source not found: $Src" }
if (-not (Get-Command ssh -EA SilentlyContinue)) { throw 'ssh not found' }
if (-not (Get-Command tar  -EA SilentlyContinue)) { throw 'tar not found' }

function Send-Archive {
    <#  Runs `tar -cf - <flags> .` piped into ssh, inside cmd.exe so the
        stream stays binary-clean.  #>
    param([string[]]$TarFlags, [string]$Label)

    $flags = ($TarFlags -join ' ')
    $remote = "mkdir -p '$RemoteDir' && tar -xf - -C '$RemoteDir'"
    # Single-quoted PS strings: no interpolation, no quoting surprises.
    $line = 'tar -cf - {0} . | ssh {1} "{2}"' -f $flags, $Server, $remote

    Write-Host "      tar flags: $flags" -ForegroundColor DarkGray
    & cmd.exe /c $line
    if ($LASTEXITCODE -ne 0) { throw "$Label transfer failed (exit $LASTEXITCODE)" }
}

Write-Host "`n=== ffplayout -> $Server$RemoteDir ===" -ForegroundColor Cyan
Write-Host "source: $Src`n"

# ---------------------------------------------------------------- 1. SSH check
Write-Host "[1/4] testing ssh ..." -ForegroundColor Yellow
& ssh -o BatchMode=yes -o ConnectTimeout=10 $Server 'echo CONNECTED'
if ($LASTEXITCODE -ne 0) {
    throw "ssh failed. If you have no key yet:  ssh-keygen; ssh-copy-id $Server"
}

# ------------------------------------------------------------------- 2. code
Write-Host "`n[2/4] copying code ..." -ForegroundColor Yellow
$codeFlags = @(
    '--exclude', 'node_modules', '--exclude', 'target',
    '--exclude', '.venv',      '--exclude', '__pycache__',
    '--exclude', '.vite',      '--exclude', '.git',
    '--exclude', 'dist',       '--exclude', 'demo_data',
    '--exclude', '*.log',      '--exclude', '*.db',
    '--exclude', '*.db-shm',   '--exclude', '*.db-wal'
)
Push-Location $Src
try { Send-Archive -TarFlags $codeFlags -Label 'code' }
finally { Pop-Location }
Write-Host "      code copied" -ForegroundColor Green

# ------------------------------------------------------------------- 3. data
if ($IncludeData -or $IncludeMedia) {
    Write-Host "`n[3/4] copying data ..." -ForegroundColor Yellow
    $dataFlags = @('--exclude', './demo_data/*')
    if ($IncludeData) {
        $dataFlags += @(
            '--include', './demo_data/db.json',
            '--include', './demo_data/asset_availability.json',
            '--include', './demo_data/asset_segments.json',
            '--include', './demo_data/media_tc.json',
            '--include', './demo_data/playlists'
        )
    }
    if ($IncludeMedia) {
        $dataFlags += @('--include', './demo_data/storage')
    }
    # remote target must include the demo_data prefix
    Push-Location $Src
    try {
        $remote = "mkdir -p '$RemoteDir' && tar -xf - -C '$RemoteDir'"
        $line = 'tar -cf - {0} . | ssh {1} "{2}"' -f ($dataFlags -join ' '), $Server, $remote
        Write-Host "      tar flags: $($dataFlags -join ' ')" -ForegroundColor DarkGray
        & cmd.exe /c $line
        if ($LASTEXITCODE -ne 0) { throw "data transfer failed (exit $LASTEXITCODE)" }
    }
    finally { Pop-Location }
    Write-Host "      data copied to $RemoteDir/demo_data" -ForegroundColor Green
}
else {
    Write-Host "`n[3/4] skipping data  (add -IncludeData to migrate db.json + playlists)" -ForegroundColor DarkGray
}

# ------------------------------------------- 4. deploy pack + remote verify
Write-Host "`n[4/4] copying installer + verifying ..." -ForegroundColor Yellow
& ssh $Server "mkdir -p '$RemoteDir/deploy'"
if ($LASTEXITCODE -ne 0) { throw "could not create $RemoteDir/deploy on the server" }

foreach ($f in (Get-ChildItem $DeployPack -File)) {
    & scp -q $f.FullName "${Server}:$RemoteDir/deploy/"
    if ($LASTEXITCODE -eq 0) { Write-Host "      $($f.Name)" -ForegroundColor DarkGray }
}

# Literal here-string: PowerShell must not touch the bash $( ) / [ ] syntax.
$verify = @'
D="__D__"
echo "--- remote inventory ---"
ls -la "$D" | head -20
echo
echo "server.py   : $(wc -l < "$D/server.py") lines"
echo "db_admin.py : $(wc -l < "$D/db_admin.py") lines"
echo "frontend    : $(ls "$D/frontend" 2>/dev/null | tr '\n' ' ')"
echo "assets      : $(ls "$D/assets" 2>/dev/null | tr '\n' ' ')"
echo "pack        : $(ls "$D/deploy" 2>/dev/null | tr '\n' ' ')"
for d in node_modules target .venv demo_data dist; do
  if [ -e "$D/$d" ]; then echo "LEAK  $d present"; else echo "ok    $d absent"; fi
done
echo "total       : $(du -sh "$D" | cut -f1)"
'@.Replace('__D__', $RemoteDir)
& ssh $Server $verify

Write-Host @"

=== copy done ===

Next, on the server:

    ssh $Server
    cd $RemoteDir/deploy
    chmod +x deploy-ubuntu.sh
    sudo ./deploy-ubuntu.sh

That installs ffmpeg, the venv, builds the frontend, writes the systemd unit
and nginx site, creates the database and starts the service.
"@ -ForegroundColor Cyan