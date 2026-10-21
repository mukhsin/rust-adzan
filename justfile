# justfile — helper development rust-adzan
# Jalankan `just` (tanpa argumen) untuk melihat daftar perintah.

# daftar semua perintah yang tersedia
default:
    @just --list

# jalankan aplikasi mode dev dengan hot-reload (butuh dioxus-cli)
dev: css
    dx serve

# compile + jalankan aplikasi sekali tanpa hot-reload
run: css
    cargo run

# cek cepat apakah kode compile (tanpa menghasilkan binary)
check: css
    cargo check

# build versi debug
build: css
    cargo build

# build versi release (opt-level "s" + LTO)
release: css
    cargo build --release

# regenerate assets/tailwind.css bila basah/hilang (hasil generate TIDAK
# di-track git; regen butuh bunx). Skrip sendiri yang memutuskan skip.
css:
    bash tools/build-css.sh

# regenerate ikon RGBA (taskbar & tray) dari assets/mosque.png
icon:
    cargo run --bin gen-icon

# build installer NSIS ringan (WebView2 diunduh saat instalasi)
bundle: css
    cargo build --release
    mkdir -p target/nsis
    MSYS_NO_PATHCONV=1 "$USERPROFILE/.dx/tools/nsis-3.11/Bin/makensis.exe" \
      /DEXE_SRC="$(cygpath -w "$(pwd)/target/release/rust-adzan.exe")" \
      /DBOOTSTRAP_SRC="$(cygpath -w "$USERPROFILE/.dx/tools/MicrosoftEdgeWebview2Setup.exe")" \
      /DICON_SRC="$(cygpath -w "$(pwd)/assets/icon.ico")" \
      /DOUTDIR="$(cygpath -w "$(pwd)/target/nsis")" \
      tools/installer.nsi

# jalankan semua unit test (prayer, scheduler, dll)
test: css
    cargo test

# jalankan satu test berdasarkan nama, contoh: just test-one next_prayer
test-one name: css
    cargo test {{name}}

# format semua kode
fmt:
    cargo fmt

# cek format tanpa mengubah file (untuk CI/verify)
fmt-check:
    cargo fmt --check

# linter clippy
clippy: css
    cargo clippy --all-targets

# fmt-check + clippy + test sekaligus (simulasi CI sebelum commit/push)
verify: fmt-check clippy test
    @echo "Semua cek lolos, aman untuk commit."

# buka dokumentasi API project di browser
doc: css
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

# rilis lengkap: bump versi, commit+push, installer, GitHub release, arsip ke dist/
# contoh: just release-all 0.2.0   (opsional: notes=path file catatan rilis)
release-all version notes="":
    @echo '{{version}}' | grep -Eq '^[0-9]+\.[0-9]+\.[0-9]+$' || (echo "Versi harus format x.y.z (contoh: 0.2.0)"; exit 1)
    @test -z "$(git status --porcelain)" || (echo "Working tree tidak bersih — commit dulu perubahanmu."; exit 1)
    @sed -i 's/^version = ".*"/version = "{{version}}"/' Cargo.toml
    @sed -i 's/!define VERSION ".*"/!define VERSION "{{version}}"/' tools/installer.nsi
    @cargo build --release
    @git add Cargo.toml Cargo.lock tools/installer.nsi
    @git diff --cached --quiet || git commit -m "release: v{{version}}"
    @git push
    @just bundle
    @mkdir -p dist
    @cp -v "target/nsis/Jadwal Sholat {{version}} x64-setup.exe" dist/
    @bash tools/release.sh {{version}} {{notes}}
