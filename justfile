# justfile — helper development rust-adzan
# Jalankan `just` (tanpa argumen) untuk melihat daftar perintah.

# daftar semua perintah yang tersedia
default:
    @just --list

# jalankan aplikasi mode dev dengan hot-reload (butuh dioxus-cli)
dev:
    dx serve

# compile + jalankan aplikasi sekali tanpa hot-reload
run:
    cargo run

# cek cepat apakah kode compile (tanpa menghasilkan binary)
check:
    cargo check

# build versi debug
build:
    cargo build

# build versi release (opt-level "s" + LTO)
release:
    cargo build --release

# regenerate assets/tailwind.css dari assets/input.css (butuh node/npm)
css:
    bash tools/build-css.sh

# regenerate ikon RGBA (taskbar & tray) dari assets/mosque.png
icon:
    cargo run --bin gen-icon

# build installer NSIS ringan (WebView2 diunduh saat instalasi)
bundle:
    cargo build --release
    mkdir -p target/nsis
    MSYS_NO_PATHCONV=1 "$USERPROFILE/.dx/tools/nsis-3.11/Bin/makensis.exe" \
      /DEXE_SRC="$(cygpath -w "$(pwd)/target/release/rust-adzan.exe")" \
      /DBOOTSTRAP_SRC="$(cygpath -w "$USERPROFILE/.dx/tools/MicrosoftEdgeWebview2Setup.exe")" \
      /DICON_SRC="$(cygpath -w "$(pwd)/assets/icon.ico")" \
      /DOUTDIR="$(cygpath -w "$(pwd)/target/nsis")" \
      tools/installer.nsi

# jalankan semua unit test (prayer, scheduler, dll)
test:
    cargo test

# jalankan satu test berdasarkan nama, contoh: just test-one next_prayer
test-one name:
    cargo test {{name}}

# format semua kode
fmt:
    cargo fmt

# cek format tanpa mengubah file (untuk CI/verify)
fmt-check:
    cargo fmt --check

# linter clippy
clippy:
    cargo clippy --all-targets

# fmt-check + clippy + test sekaligus (simulasi CI sebelum commit/push)
verify: fmt-check clippy test
    @echo "Semua cek lolos, aman untuk commit."

# buka dokumentasi API project di browser
doc:
    cargo doc --no-deps --open

# bersihkan hasil build
clean:
    cargo clean

# arsipkan installer ke dist/ lalu bersihkan seluruh target/ (hemat disk)
prune:
    mkdir -p dist
    if ls target/nsis/*.exe >/dev/null 2>&1; then \
        cp -v target/nsis/*.exe dist/; \
        echo "Installer diarsipkan ke dist/"; \
    else \
        echo "Tidak ada installer di target/nsis, lewati arsip."; \
    fi
    cargo clean
