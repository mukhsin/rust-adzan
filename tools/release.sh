#!/usr/bin/env bash
# tools/release.sh — buat GitHub release + upload installer sebagai aset.
# Memakai kredensial git yang tersimpan (git credential), tanpa gh CLI.
# Pemakaian: bash tools/release.sh <versi> [file-catatan.md]
set -euo pipefail

VERSION="${1:?Pemakaian: tools/release.sh <versi> [file-catatan.md]}"
REPO="mukhsin/rust-adzan"
TAG="v$VERSION"
FILE="dist/Jadwal Sholat $VERSION x64-setup.exe"

[ -f "$FILE" ] || { echo "Installer tidak ditemukan: $FILE (jalankan 'just bundle' dulu)"; exit 1; }

TOKEN=$(printf "protocol=https\nhost=github.com\n\n" | git credential fill | sed -n 's/^password=//p')
[ -n "$TOKEN" ] || { echo "Kredensial GitHub tidak ditemukan — jalankan 'git push' sekali dulu."; exit 1; }

API="https://api.github.com/repos/$REPO"
AUTH="Authorization: Bearer $TOKEN"

# Tolak kalau release dengan tag yang sama sudah ada.
CODE=$(curl -s -o /dev/null -w "%{http_code}" -H "$AUTH" "$API/releases/tags/$TAG")
[ "$CODE" = "404" ] || { echo "Release $TAG sudah ada (HTTP $CODE) — naikkan versi dulu."; exit 1; }

# Susun body JSON; catatan dari file opsional di-escape (backslash, kutip,
# newline). JSON dibangun via printf ke file lalu dikirim `-d @file` — kalau
# diinterpolasi langsung ke arg curl, backslash escape-nya dimakan shell.
if [ "${2:-}" != "" ] && [ -f "$2" ]; then
  BODY=$(sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' -e 's/\t/\\t/g' "$2" | sed -e ':a' -e 'N' -e '$!ba' -e 's/\n/\\n/g')
else
  BODY="Rilis Jadwal Sholat $VERSION — aplikasi desktop jadwal sholat + adzan otomatis untuk Windows, 100% offline.\\n\\nUnduh **Jadwal Sholat $VERSION x64-setup.exe**, jalankan, selesai — WebView2 diunduh otomatis saat instalasi bila belum ada.\\n\\nButuh: Windows 10/11 64-bit."
fi

echo "Membuat release $TAG ..."
RESP=$(mktemp)
REQ=$(mktemp)
trap 'rm -f "$RESP" "$REQ"' EXIT
printf '{"tag_name":"%s","target_commitish":"main","name":"Jadwal Sholat %s","draft":false,"prerelease":false,"body":"%s"}' \
  "$TAG" "$TAG" "$BODY" > "$REQ"
HTTP=$(curl -s -o "$RESP" -w "%{http_code}" -X POST -H "$AUTH" -H "Accept: application/vnd.github+json" \
  "$API/releases" \
  -d @"$REQ")
[ "$HTTP" = "201" ] || { echo "Gagal membuat release (HTTP $HTTP):"; head -c 600 "$RESP"; echo; exit 1; }

UPLOAD_URL=$(sed -n 's/.*"upload_url": *"\([^"]*\)".*/\1/p' "$RESP" | sed 's/{.*//')
NAME_ENC=$(basename "$FILE" | sed 's/ /%20/g')

echo "Mengunggah $(basename "$FILE") ..."
UP=$(mktemp)
HTTP=$(curl -s -o "$UP" -w "%{http_code}" -X POST -H "$AUTH" -H "Content-Type: application/octet-stream" \
  --data-binary @"$FILE" "$UPLOAD_URL?name=$NAME_ENC")
[ "$HTTP" = "201" ] || { echo "Gagal upload aset (HTTP $HTTP):"; head -c 600 "$UP"; echo; exit 1; }

echo "Release selesai:"
sed -n 's/.*"html_url": *"\([^"]*\)".*/  halaman : \1/p' "$RESP" | head -1
sed -n 's/.*"browser_download_url": *"\([^"]*\)".*/  download: \1/p' "$UP"
