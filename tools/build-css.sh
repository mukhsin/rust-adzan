#!/usr/bin/env bash
# Regenerate assets/tailwind.css dari assets/input.css.
# Butuh Node.js (bunx). Jalankan dari root proyek: bash tools/build-css.sh
set -euo pipefail
cd "$(dirname "$0")/.."
bunx -y @tailwindcss/cli -i assets/input.css -o assets/tailwind.css --minify
echo "OK: assets/tailwind.css diperbarui"
