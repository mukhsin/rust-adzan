# tools/check-update-share.ps1 — cek installer terbaru di shared folder kantor.
#
# Struktur yang diharapkan:
#   \\svrbtch-app-u01\SHARE APPLICATION\Jadwal Sholat\vX.Y.Z\Jadwal Sholat X.Y.Z x64-setup.exe
#
# Pemakaian (dari root repo):
#   powershell -NoProfile -ExecutionPolicy Bypass -File tools/check-update-share.ps1
#   powershell ... -CurrentVersion 1.2.0     (paksa versi pembanding)
#
# Versi pembanding default dibaca dari Cargo.toml.
#
# Exit code:
#   0 = ada versi lebih baru di share
#   1 = versi terpasang sudah terbaru
#   2 = share tidak terjangkau (VPN mati / folder tidak ada)
#   3 = share terjangkau tapi tidak ada rilis valid (folder kosong / installer hilang)

param(
    [string]$ShareRoot = "\\svrbtch-app-u01\SHARE APPLICATION\Jadwal Sholat",
    [string]$CurrentVersion = ""
)

$ErrorActionPreference = "Stop"

# Versi pembanding: argumen > Cargo.toml > 0.0.0 (selalu anggap ada yang baru).
if (-not $CurrentVersion) {
    $cargo = Join-Path $PSScriptRoot "..\Cargo.toml"
    if (Test-Path -LiteralPath $cargo) {
        $baris = Select-String -LiteralPath $cargo -Pattern '^version = "(.+)"' | Select-Object -First 1
        if ($baris) { $CurrentVersion = $baris.Matches[0].Groups[1].Value }
    }
}
if (-not $CurrentVersion) { $CurrentVersion = "0.0.0" }

if (-not (Test-Path -LiteralPath $ShareRoot)) {
    Write-Host "TIDAK TERJANGKAU: $ShareRoot"
    Write-Host "(VPN kantor belum terhubung, atau folder belum dibuat.)"
    exit 2
}

# Folder rilis: nama persis vX.Y.Z.
$folders = Get-ChildItem -LiteralPath $ShareRoot -Directory |
    Where-Object { $_.Name -match '^v\d+\.\d+\.\d+$' }

if (-not $folders) {
    Write-Host "KOSONG: belum ada folder vX.Y.Z di $ShareRoot"
    exit 3
}

$latest = $folders |
    Sort-Object { [version]$_.Name.Substring(1) } -Descending |
    Select-Object -First 1
$ver = $latest.Name.Substring(1)
$installer = Join-Path $latest.FullName "Jadwal Sholat $ver x64-setup.exe"

Write-Host "Rilis terbaru di share : v$ver"
if (Test-Path -LiteralPath $installer) {
    $sizeMb = [math]::Round((Get-Item -LiteralPath $installer).Length / 1MB, 1)
    Write-Host "Installer              : $latest\$($installer | Split-Path -Leaf) ($sizeMb MB)"
} else {
    Write-Host "Installer              : TIDAK ADA"
    Write-Host "  harusnya            : $installer"
    exit 3
}

Write-Host "Versi terpasang        : $CurrentVersion"
if ([version]$ver -gt [version]$CurrentVersion) {
    Write-Host "HASIL                  : PEMBARUAN TERSEDIA -> v$ver"
    Write-Host "LATEST=$ver"
    exit 0
}
Write-Host "HASIL                  : sudah terbaru"
exit 1
