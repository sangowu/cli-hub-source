param(
    [string]$ProjectRoot = (Split-Path -Parent $PSScriptRoot)
)

$ErrorActionPreference = "Stop"

if ([System.Environment]::OSVersion.Platform -ne [System.PlatformID]::Win32NT) {
    throw "Windows bundle verification must run on Windows."
}

$packageJsonPath = Join-Path $ProjectRoot "package.json"
$package = Get-Content -LiteralPath $packageJsonPath -Raw | ConvertFrom-Json
$version = [string]$package.version
$releaseRoot = Join-Path $ProjectRoot "src-tauri\target\release"

$artifacts = @(
    [pscustomobject]@{
        Kind = "Application"
        Path = Join-Path $releaseRoot "cli-hub.exe"
        VersionSource = "File"
    },
    [pscustomobject]@{
        Kind = "NSIS"
        Path = Join-Path $releaseRoot "bundle\nsis\CLI Hub_${version}_x64-setup.exe"
        VersionSource = "File"
    },
    [pscustomobject]@{
        Kind = "MSI"
        Path = Join-Path $releaseRoot "bundle\msi\CLI Hub_${version}_x64_en-US.msi"
        VersionSource = "Msi"
    }
)

function Get-MsiProperty {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path,
        [Parameter(Mandatory = $true)]
        [string]$Property
    )

    $installer = New-Object -ComObject WindowsInstaller.Installer
    $database = $installer.GetType().InvokeMember(
        "OpenDatabase",
        "InvokeMethod",
        $null,
        $installer,
        @($Path, 0)
    )
    $query = "SELECT ``Value`` FROM ``Property`` WHERE ``Property`` = '$Property'"
    $view = $database.GetType().InvokeMember(
        "OpenView",
        "InvokeMethod",
        $null,
        $database,
        @($query)
    )
    $view.GetType().InvokeMember("Execute", "InvokeMethod", $null, $view, $null) | Out-Null
    $record = $view.GetType().InvokeMember("Fetch", "InvokeMethod", $null, $view, $null)

    if ($null -eq $record) {
        throw "MSI property '$Property' was not found in $Path."
    }

    return [string]$record.GetType().InvokeMember(
        "StringData",
        "GetProperty",
        $null,
        $record,
        1
    )
}

$results = foreach ($artifact in $artifacts) {
    if (-not (Test-Path -LiteralPath $artifact.Path -PathType Leaf)) {
        throw "Missing $($artifact.Kind) artifact: $($artifact.Path)"
    }

    $file = Get-Item -LiteralPath $artifact.Path
    if ($file.Length -lt 1MB) {
        throw "$($artifact.Kind) artifact is unexpectedly small: $($file.Length) bytes."
    }

    $artifactVersion = if ($artifact.VersionSource -eq "Msi") {
        Get-MsiProperty -Path $file.FullName -Property "ProductVersion"
    } else {
        [string]$file.VersionInfo.ProductVersion
    }

    if ($artifactVersion -ne $version) {
        throw "$($artifact.Kind) version '$artifactVersion' does not match package.json '$version'."
    }

    $signature = Get-AuthenticodeSignature -LiteralPath $file.FullName
    $hash = Get-FileHash -LiteralPath $file.FullName -Algorithm SHA256

    [pscustomobject]@{
        Kind = $artifact.Kind
        Version = $artifactVersion
        SizeMB = [Math]::Round($file.Length / 1MB, 2)
        Signature = [string]$signature.Status
        SHA256 = $hash.Hash
        Path = $file.FullName
    }
}

$results | Format-Table -AutoSize

if ($results.Signature -contains "NotSigned") {
    Write-Warning "One or more artifacts are unsigned. They are suitable for local MVP testing, not public release."
}

Write-Host "Windows bundle verification passed for version $version."
