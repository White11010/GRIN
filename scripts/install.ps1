# Install GRIN from GitHub Releases (Windows x86_64).
# Usage:
#   irm https://raw.githubusercontent.com/White11010/GRIN/main/scripts/install.ps1 | iex
#   irm ... -OutFile install.ps1; .\install.ps1 -Version v0.1.0

#Requires -Version 5.1

[CmdletBinding()]
param(
    [string]$Version = $env:GRIN_INSTALL_VERSION,
    [string]$BinDir = $env:GRIN_INSTALL_DIR,
    [string]$Repo = $env:GRIN_INSTALL_REPO,
    [switch]$Help
)

$ErrorActionPreference = 'Stop'

$DefaultRepo = 'White11010/GRIN'
if (-not $Repo) { $Repo = $DefaultRepo }
if (-not $BinDir) {
    $BinDir = Join-Path $env:USERPROFILE '.local\bin'
}

function Show-Usage {
    @'
Install GRIN from GitHub Releases.

Usage:
  install.ps1 [-Version <tag>] [-BinDir <path>] [-Repo owner/name]

Options:
  -Version   Release tag (e.g. v0.1.0). Default: latest GitHub release.
  -BinDir    Directory to install grin.exe into. Default: %USERPROFILE%\.local\bin
  -Repo      GitHub repository as owner/name. Default: White11010/GRIN
  -Help      Show this help.

Environment:
  GRIN_INSTALL_REPO     Override default repository (owner/name).
  GRIN_INSTALL_DIR      Default install directory if -BinDir is not passed.
  GRIN_INSTALL_VERSION  Release tag when -Version is not passed.

Examples:
  irm .../install.ps1 | iex
  $env:GRIN_INSTALL_VERSION = 'v0.1.0'; irm .../install.ps1 | iex
  irm .../install.ps1 -OutFile install.ps1; .\install.ps1 -Version v0.1.0
'@ | Write-Host
}

if ($Help) {
    Show-Usage
    exit 0
}

if ($env:OS -notmatch 'Windows') {
    Write-Error 'install.ps1: unsupported operating system (expected Windows).'
}

if (-not [Environment]::Is64BitOperatingSystem) {
    Write-Error 'install.ps1: 32-bit Windows is not supported; use cargo install grin.'
}

$arch = [Environment]::GetEnvironmentVariable('PROCESSOR_ARCHITECTURE')
$archWoW = [Environment]::GetEnvironmentVariable('PROCESSOR_ARCHITEW6432')
if ($arch -eq 'ARM64' -or $archWoW -eq 'ARM64') {
    Write-Error ( @(
        'install.ps1: no prebuilt Windows ARM64 archive in this release channel yet.',
        'install.ps1: install with: cargo install grin'
    ) -join "`n" )
}

[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

# User-Agent avoids 403 from GitHub on some PowerShell defaults.
$GitHubRequestHeaders = @{
    'User-Agent' = 'GRIN-install-script'
}

function Get-WebErrorDetail {
    param([System.Management.Automation.ErrorRecord]$ErrorRecord)
    $detail = $ErrorRecord.Exception.Message
    if ($ErrorRecord.Exception.Response -and $ErrorRecord.Exception.Response.StatusCode) {
        $code = [int]$ErrorRecord.Exception.Response.StatusCode
        $detail = "HTTP $code - $detail"
    }
    return $detail
}

function Get-LatestReleaseTag {
    param([string]$Repo)

    # Resolve latest via github.com redirect (no api.github.com — avoids API 403/rate limits).
    $latestUrl = "https://github.com/$Repo/releases/latest"
    try {
        $response = Invoke-WebRequest `
            -Uri $latestUrl `
            -UseBasicParsing `
            -MaximumRedirection 5 `
            -Headers $GitHubRequestHeaders
    }
    catch {
        $detail = Get-WebErrorDetail -ErrorRecord $_
        Write-Error ( @(
            "install.ps1: failed to resolve latest release ($detail).",
            'install.ps1: set a tag manually, e.g. $env:GRIN_INSTALL_VERSION = ''v0.1.2'', then re-run.'
        ) -join "`n" )
    }

    $finalUri = $response.BaseResponse.ResponseUri.AbsoluteUri
    if ($finalUri -match '/releases/tag/(v[^/?#]+)') {
        return $Matches[1]
    }

    Write-Error 'install.ps1: could not parse latest release tag from GitHub redirect.'
}

$target = 'x86_64-pc-windows-msvc'

if (-not $Version) {
    $Version = Get-LatestReleaseTag -Repo $Repo
}

if ($Version -notmatch '^v') {
    Write-Error "install.ps1: expected tag like v0.1.0, got: $Version"
}

$asset = "grin-$Version-$target.zip"
$url = "https://github.com/$Repo/releases/download/$Version/$asset"

$tmp = Join-Path ([IO.Path]::GetTempPath()) ("grin-install-" + [Guid]::NewGuid().ToString('n'))
New-Item -ItemType Directory -Path $tmp -Force | Out-Null

try {
    $zipPath = Join-Path $tmp $asset
    Write-Host "install.ps1: installing GRIN $Version ($target) from $url"
    try {
        Invoke-WebRequest -Uri $url -OutFile $zipPath -UseBasicParsing -Headers $GitHubRequestHeaders
    }
    catch {
        $detail = Get-WebErrorDetail -ErrorRecord $_
        Write-Error "install.ps1: download failed ($detail). Check the tag and your network, or install from source."
    }

    $extractDir = Join-Path $tmp 'extract'
    Expand-Archive -Path $zipPath -DestinationPath $extractDir -Force

    $binary = Get-ChildItem -Path $extractDir -Filter 'grin.exe' -Recurse -File |
        Select-Object -First 1
    if (-not $binary) {
        Write-Error "install.ps1: expected binary 'grin.exe' inside archive."
    }

    if (-not (Test-Path -LiteralPath $BinDir)) {
        New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
    }
    if (-not (Test-Path -LiteralPath $BinDir -PathType Container)) {
        Write-Error "install.ps1: install directory is not a directory: $BinDir"
    }

    $dest = Join-Path $BinDir 'grin.exe'
    Copy-Item -LiteralPath $binary.FullName -Destination $dest -Force

    $binDirFull = [IO.Path]::GetFullPath($BinDir)
    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $pathParts = @()
    if ($userPath) {
        $pathParts = $userPath -split ';' | Where-Object { $_ -ne '' }
    }

    $alreadyOnPath = $false
    foreach ($part in $pathParts) {
        try {
            if ([IO.Path]::GetFullPath($part).TrimEnd('\') -ieq $binDirFull.TrimEnd('\')) {
                $alreadyOnPath = $true
                break
            }
        }
        catch {
            continue
        }
    }

    if (-not $alreadyOnPath) {
        $newPath = if ($userPath) { "$userPath;$binDirFull" } else { $binDirFull }
        [Environment]::SetEnvironmentVariable('Path', $newPath, 'User')
        $env:Path = "$env:Path;$binDirFull"
        Write-Host "install.ps1: added $binDirFull to your user PATH."
    }

    Write-Host "install.ps1: installed to $dest"
    Write-Host 'install.ps1: open a new terminal (or restart this one), then run: grin help'
}
finally {
    if (Test-Path -LiteralPath $tmp) {
        Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
    }
}
