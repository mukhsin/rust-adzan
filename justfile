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

# build installer MSI via dx bundle
bundle:
    dx bundle

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
