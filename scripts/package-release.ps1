[CmdletBinding()]
param(
    [Parameter(Mandatory = $true)]
    [ValidatePattern('^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$')]
    [string]$VersionLabel,

    [string]$BuildRoot = 'src-tauri\target\release',

    [string]$OutputDirectory = 'release-artifacts'
)

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version Latest

$repositoryRoot = [System.IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))

function Get-RepositoryPath {
    param([Parameter(Mandatory = $true)][string]$Path)

    if ([System.IO.Path]::IsPathRooted($Path)) {
        return [System.IO.Path]::GetFullPath($Path)
    }
    return [System.IO.Path]::GetFullPath((Join-Path $repositoryRoot $Path))
}

function Get-SingleArtifact {
    param(
        [Parameter(Mandatory = $true)][string]$Directory,
        [Parameter(Mandatory = $true)][string]$Filter,
        [Parameter(Mandatory = $true)][string]$Description
    )

    if (-not (Test-Path -LiteralPath $Directory -PathType Container)) {
        throw "$Description directory does not exist: $Directory"
    }
    $matches = @(Get-ChildItem -LiteralPath $Directory -File -Filter $Filter)
    if ($matches.Count -ne 1) {
        throw "Expected exactly one $Description in $Directory, found $($matches.Count)."
    }
    return $matches[0]
}

$resolvedBuildRoot = Get-RepositoryPath -Path $BuildRoot
$resolvedOutput = Get-RepositoryPath -Path $OutputDirectory

if (-not (Test-Path -LiteralPath $resolvedBuildRoot -PathType Container)) {
    throw "Release build directory does not exist: $resolvedBuildRoot"
}

if (Test-Path -LiteralPath $resolvedOutput) {
    $existing = @(Get-ChildItem -LiteralPath $resolvedOutput -Force)
    if ($existing.Count -gt 0) {
        throw "Output directory must be empty: $resolvedOutput"
    }
} else {
    New-Item -ItemType Directory -Path $resolvedOutput | Out-Null
}

$releaseExe = Join-Path $resolvedBuildRoot 'envcompass.exe'
if (-not (Test-Path -LiteralPath $releaseExe -PathType Leaf)) {
    throw "Release executable does not exist: $releaseExe"
}

$nsis = Get-SingleArtifact `
    -Directory (Join-Path $resolvedBuildRoot 'bundle\nsis') `
    -Filter '*_x64-setup.exe' `
    -Description 'x64 NSIS installer'
$msi = Get-SingleArtifact `
    -Directory (Join-Path $resolvedBuildRoot 'bundle\msi') `
    -Filter '*_x64_*.msi' `
    -Description 'x64 MSI installer'

$setupName = "EnvCompass-$VersionLabel-windows-x64-setup.exe"
$portableName = "EnvCompass-$VersionLabel-windows-x64-portable.zip"
$msiName = "EnvCompass-$VersionLabel-windows-x64.msi"

$setupPath = Join-Path $resolvedOutput $setupName
$portablePath = Join-Path $resolvedOutput $portableName
$msiPath = Join-Path $resolvedOutput $msiName

Copy-Item -LiteralPath $nsis.FullName -Destination $setupPath
Copy-Item -LiteralPath $msi.FullName -Destination $msiPath

$stage = Join-Path $resolvedOutput ".portable-stage-$PID"
New-Item -ItemType Directory -Path $stage | Out-Null
try {
    Copy-Item -LiteralPath $releaseExe -Destination (Join-Path $stage 'EnvCompass.exe')
    Copy-Item -LiteralPath (Join-Path $repositoryRoot 'packaging\QUICKSTART.txt') -Destination $stage
    Copy-Item -LiteralPath (Join-Path $repositoryRoot 'LICENSE') -Destination $stage
    Compress-Archive -LiteralPath @(
        (Join-Path $stage 'EnvCompass.exe'),
        (Join-Path $stage 'QUICKSTART.txt'),
        (Join-Path $stage 'LICENSE')
    ) -DestinationPath $portablePath -CompressionLevel Optimal
} finally {
    if (Test-Path -LiteralPath $stage) {
        Remove-Item -LiteralPath $stage -Recurse -Force
    }
}

Add-Type -AssemblyName System.IO.Compression.FileSystem
$archive = [System.IO.Compression.ZipFile]::OpenRead($portablePath)
try {
    $entryNames = @($archive.Entries | ForEach-Object { $_.FullName } | Sort-Object)
} finally {
    $archive.Dispose()
}
$expectedEntries = @('EnvCompass.exe', 'LICENSE', 'QUICKSTART.txt') | Sort-Object
if (($entryNames -join "`n") -ne ($expectedEntries -join "`n")) {
    throw "Portable ZIP layout is invalid: $($entryNames -join ', ')"
}

$artifacts = @($setupPath, $portablePath, $msiPath)
$checksumPath = Join-Path $resolvedOutput 'SHA256SUMS.txt'
$checksumLines = foreach ($artifact in $artifacts) {
    $hash = (Get-FileHash -LiteralPath $artifact -Algorithm SHA256).Hash.ToLowerInvariant()
    "$hash  $([System.IO.Path]::GetFileName($artifact))"
}
[System.IO.File]::WriteAllLines($checksumPath, $checksumLines, [System.Text.UTF8Encoding]::new($false))

$published = @($artifacts + $checksumPath) | ForEach-Object {
    $item = Get-Item -LiteralPath $_
    [pscustomobject]@{
        Name = $item.Name
        Bytes = $item.Length
        SHA256 = if ($item.Extension -ne '.txt') {
            (Get-FileHash -LiteralPath $item.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        } else {
            '-'
        }
    }
}

$published | Format-Table -AutoSize
