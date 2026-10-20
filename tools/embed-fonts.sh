#!/usr/bin/env bash
# Regenerate assets/fonts.css — base64-embed assets/fonts/*.woff2 supaya font
# ikut ter-embed di binary (CSS di-include_str! dan webview offline).
# Jalankan hanya saat file font berganti: bash tools/embed-fonts.sh
set -euo pipefail
cd "$(dirname "$0")/.."
OS=$(base64 -w0 assets/fonts/open-sans-var.woff2)
MM=$(base64 -w0 assets/fonts/martian-mono-var.woff2)
RC=$(base64 -w0 assets/fonts/roboto-condensed-var.woff2)
cat > assets/fonts.css <<EOF
/* Dibuat otomatis: base64 dari assets/fonts/*.woff2 (lihat tools/embed-fonts.sh).
   Font: Open Sans, Martian Mono, Roboto Condensed — SIL Open Font License 1.1. */
@font-face {
  font-family: "Open Sans";
  src: url(data:font/woff2;base64,$OS) format("woff2");
  font-weight: 300 800;
  font-style: normal;
  font-display: block;
}
@font-face {
  font-family: "Martian Mono";
  src: url(data:font/woff2;base64,$MM) format("woff2");
  font-weight: 100 800;
  font-style: normal;
  font-display: block;
}
@font-face {
  font-family: "Roboto Condensed";
  src: url(data:font/woff2;base64,$RC) format("woff2");
  font-weight: 100 900;
  font-style: normal;
  font-display: block;
}
EOF
echo "OK: assets/fonts.css diperbarui"
