#!/usr/bin/env bash
# Regenerate assets/tailwind.css dari assets/input.css.
# Butuh Node.js (bunx). Jalankan dari root proyek: bash tools/build-css.sh
#
# File hasil TIDAK di-track git (ter-embed ke binary via include_str!) —
# fresh clone tinggal compile lewat justfile; resep `css` jadi dependency
# resep-resep yang compile. Regenskip: bila hasil masih lebih baru dari
# semua sumber (src/**/*.rs + assets/*.css), regen dilewati supaya `just
# test`/`just check` tidak memanggil bunx tiap kali. Paksa regen: hapus
# assets/tailwind.css atau sentuh salah satu sumber.
set -euo pipefail
cd "$(dirname "$0")/.."

if [ -f assets/tailwind.css ] && [ -z "$(find src assets -maxdepth 2 \( -name '*.rs' -o -name '*.css' \) -newer assets/tailwind.css -print -quit 2>/dev/null)" ]; then
  echo "OK: assets/tailwind.css masih segar, regen dilewati"
  exit 0
fi

bunx -y @tailwindcss/cli -i assets/input.css -o assets/tailwind.css --minify
echo "OK: assets/tailwind.css diperbarui"
