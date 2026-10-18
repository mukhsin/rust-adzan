#!/usr/bin/env bash
# tools/update-notes.sh — perbarui body (release note) GitHub release yang sudah ada.
# Memakai kredensial git yang tersimpan (git credential), tanpa gh CLI.
# Pemakaian: bash tools/update-notes.sh <versi> <file-catatan.md>
set -euo pipefail

VERSION="${1:?Pemakaian: tools/update-notes.sh <versi> <file-catatan.md>}"
NOTES="${2:?Pemakaian: tools/update-notes.sh <versi> <file-catatan.md>}"
REPO="mukhsin/rust-adzan"
TAG="v$VERSION"

[ -f "$NOTES" ] || { echo "File catatan tidak ditemukan: $NOTES"; exit 1; }

TOKEN=$(printf "protocol=https\nhost=github.com\n\n" | git credential fill | sed -n 's/^password=//p')
[ -n "$TOKEN" ] || { echo "Kredensial GitHub tidak ditemukan — jalankan 'git push' sekali dulu."; exit 1; }

API="https://api.github.com/repos/$REPO"
AUTH="Authorization: Bearer $TOKEN"

# PATCH hanya menerima ID numerik (by tag hanya untuk GET) — ambil ID dulu.
REL_ID=$(curl -s -H "$AUTH" "$API/releases/tags/$TAG" | sed -n 's/.*"id": *\([0-9]*\).*/\1/p' | head -1)
[ -n "$REL_ID" ] || { echo "Release $TAG tidak ditemukan."; exit 1; }

# Body JSON di-escape (backslash, kutip, tab, newline) lalu dibangun via printf
# ke file dan dikirim `-d @file` — interpolasi langsung ke arg curl membuat
# shell memakan backslash escape sehingga JSON rusak.
BODY=$(sed -e 's/\\/\\\\/g' -e 's/"/\\"/g' -e 's/\t/\\t/g' "$NOTES" | sed -e ':a' -e 'N' -e '$!ba' -e 's/\n/\\n/g')
REQ=$(mktemp)
RESP=$(mktemp)
trap 'rm -f "$REQ" "$RESP"' EXIT
printf '{"body":"%s"}' "$BODY" > "$REQ"

echo "Memperbarui release note $TAG ..."
HTTP=$(curl -s -o "$RESP" -w "%{http_code}" -X PATCH -H "$AUTH" -H "Accept: application/vnd.github+json" \
  "$API/releases/$REL_ID" \
  -d @"$REQ")
[ "$HTTP" = "200" ] || { echo "Gagal memperbarui release (HTTP $HTTP):"; head -c 600 "$RESP"; echo; exit 1; }

echo "Release note diperbarui:"
sed -n 's/.*"html_url": *"\([^"]*\)".*/  halaman: \1/p' "$RESP" | head -1
