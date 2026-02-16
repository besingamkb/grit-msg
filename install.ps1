$ErrorActionPreference = "Stop"

$Repo = if ($env:REPO) { $env:REPO } else { "besingamkb/grit-msg" }
$BinaryName = "grit-msg"
$Version = if ($env:VERSION) { $env:VERSION } else { "latest" }
$BinDir = if ($env:BIN_DIR) { $env:BIN_DIR } else { Join-Path $HOME ".local\bin" }

if ([Environment]::Is64BitOperatingSystem -eq $false) {
    throw "Unsupported Windows architecture: 32-bit"
}

$Target = "x86_64-pc-windows-msvc"

if ($Version -eq "latest") {
    $Latest = Invoke-RestMethod -Uri "https://api.github.com/repos/$Repo/releases/latest"
    if (-not $Latest.tag_name) {
        throw "Failed to resolve latest release version for $Repo."
    }
    $Version = $Latest.tag_name
}

$BaseUrl = "https://github.com/$Repo/releases/download/$Version"
$Archive = "$BinaryName-$Version-$Target.zip"
$Checksum = "$Archive.sha256"

$TempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("grit-msg-install-" + [System.Guid]::NewGuid())
New-Item -ItemType Directory -Path $TempDir | Out-Null

try {
    Write-Host "Downloading $Archive..."
    Invoke-WebRequest -Uri "$BaseUrl/$Archive" -OutFile (Join-Path $TempDir $Archive)
    Invoke-WebRequest -Uri "$BaseUrl/$Checksum" -OutFile (Join-Path $TempDir $Checksum)

    Write-Host "Verifying checksum..."
    $Expected = (Get-Content (Join-Path $TempDir $Checksum) -Raw).Trim().Split(" ")[0].ToLower()
    $Actual = (Get-FileHash (Join-Path $TempDir $Archive) -Algorithm SHA256).Hash.ToLower()
    if ($Expected -ne $Actual) {
        throw "Checksum mismatch."
    }

    New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
    Expand-Archive -Path (Join-Path $TempDir $Archive) -DestinationPath $TempDir -Force
    Copy-Item (Join-Path $TempDir "$BinaryName.exe") (Join-Path $BinDir "$BinaryName.exe") -Force

    Write-Host "Installed $BinaryName to $(Join-Path $BinDir "$BinaryName.exe")"
    $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
    if (-not $userPath) { $userPath = "" }
    if (-not ($userPath.Split(';') -contains $BinDir)) {
        Write-Host ""
        Write-Host "Your user PATH does not include $BinDir."
        Write-Host "Add it for future terminals with:"
        Write-Host "  setx PATH `"$($userPath.TrimEnd(';'));$BinDir`""
        Write-Host ""
        Write-Host "Then restart PowerShell."
    }
    Write-Host "Run: $BinaryName --help"
} finally {
    Remove-Item -Path $TempDir -Recurse -Force -ErrorAction SilentlyContinue
}
