# Rust Adzan 🕌

Aplikasi desktop Windows untuk menampilkan jadwal sholat dan memutar
audio adzan otomatis saat masuk waktu sholat. Dibangun dengan
[Dioxus](https://dioxuslabs.com) 0.7 — 100% offline (jadwal dihitung
lokal, tidak butuh internet).

## Fitur

- **Jadwal sholat offline** — perhitungan astronomis via crate `salah`
  (port library Adhan). Default metode **Kemenag Indonesia**
  (Subuh 20°, Isya 18°); tersedia juga MWL, Egyptian, Karachi,
  Umm al-Qura, ISNA.
- **Lokasi** — pilih dari 47 kota Indonesia (beserta zona waktu
  WIB/WITA/WIT) atau input koordinat manual.
- **Audio adzan** — putar MP3 saat masuk waktu; adzan bawaan (umum +
  khusus Subuh) sudah ter-embed di aplikasi, dan bisa diganti dengan
  file sendiri lewat Pengaturan. Pemutaran bisa dihentikan kapan saja
  lewat tombol Stop.
- **Countdown** — hitung mundur real-time ke sholat berikutnya
  (setelah Isya mengarah ke Subuh esok hari).
- **System tray** — klik kiri ikon tray men-toggle widget; klik kanan membuka
  menu (tampilkan main window, widget, keluar). Aplikasi tetap berjalan
  di tray.
- **Alur window** — buka aplikasi → main window. Tutup main window →
  widget muncul sebagai gantinya. Tutup widget → semua tersimpan di tray.
- **Notifikasi** — toast Windows saat pengingat X menit sebelum adzan
  dan saat masuk waktu.
- **Widget desktop** — jendela melayang kecil yang selalu di atas,
  menampilkan lokasi, tanggal, dan countdown (dengan toggle tema +
  pintasan pengaturan di title bar).
- **Auto-start** — opsi menjalankan otomatis saat Windows menyala
  (registry HKCU, tanpa hak admin).
- **Settings tersimpan** — `%APPDATA%\rust-adzan\config.json`.

## Menjalankan

Butuh Rust (MSVC) + WebView2 (sudah bawaan Windows 11) + Node.js (untuk
regenerasi CSS Tailwind).

```bash
npm install              # sekali, untuk tool Tailwind
cargo run                # mode dev
# atau
dx serve                 # hot-reload via dioxus-cli
```

Styling pakai **Tailwind CSS v4**: kelas dipindai langsung dari source rsx
(`assets/input.css`), hasilnya `assets/tailwind.css` yang di-embed ke binary.
Jalankan `just css` setiap menambah kelas Tailwind baru yang belum pernah
dipakai.

### Perintah just

Development bisa dipermudah dengan [just](https://github.com/casey/just)
(`winget install Casey.Just`). Jalankan `just` untuk melihat semua perintah:

```bash
just dev          # hot-reload via dx serve
just run          # cargo run sekali
just test         # unit test
just fmt          # format kode
just clippy       # linter
just verify       # fmt-check + clippy + test
just release      # build rilis
just bundle       # installer NSIS ringan
just prune        # arsipkan installer ke dist/ + bersihkan target/
just release-all  # bump versi + build + GitHub release (lihat bawah)
```

### Audio adzan

Adzan bawaan (`assets/adzan.mp3` + `assets/adzan-subuh.mp3`) sudah
**ter-embed di binary** — aplikasi langsung siap pakai. Untuk memakai
file sendiri: pilih audio apa pun (MP3/WAV/OGG/FLAC/M4A/AAC) lewat menu
**Pengaturan** di aplikasi, tanpa perlu build ulang. Urutan prioritas:
file kustom → file di folder assets → adzan bawaan. Pemutaran tes bisa
dihentikan lewat tombol Stop.

### Ikon aplikasi (taskbar & tray)

Ikon diambil dari `assets/mosque.png`, dipakai apa adanya (transparan).
Setelah mengganti PNG, jalankan `just icon` untuk men-generate
`assets/icon.rgba`, `assets/icon-tray.rgba`, dan `assets/icon.ico`
(yang terakhir di-embed ke exe untuk ikon di Explorer + metadata file).

## Build rilis

```bash
cargo build --release   # binary portable di target/release/rust-adzan.exe
just bundle             # installer NSIS ringan di target/nsis/ (WebView2 diunduh saat instalasi)
just prune              # arsipkan installer ke dist/ lalu bersihkan target/
```

Binary release **portable** — adzan bawaan, ikon, dan aset lain sudah
ter-embed; satu-satunya dependensi adalah WebView2 (bawaan Windows 10/11).
Log aplikasi: `%APPDATA%\jadwal-sholat\app.log`.

### Rilis ke GitHub

```bash
just release-all 0.2.0
```

Satu perintah untuk rilis lengkap: naikkan versi di `Cargo.toml` +
`tools/installer.nsi`, commit + push (pesan `release: vX.Y.Z`), build
installer, arsip ke `dist/`, lalu buat release `vX.Y.Z` di GitHub dengan
installer sebagai aset unduhan — via `tools/release.sh`, memakai kredensial
git yang tersimpan (tanpa gh CLI). Syarat: working tree bersih, format
versi `x.y.z`, dan versi tersebut belum pernah dirilis. Untuk catatan
rilis kustom: `just release-all 0.2.0 notes=catatan.md`.

## Struktur

```
src/
├── main.rs        # entry: window frameless + ikon, mulai scheduler
├── prayer.rs      # perhitungan jadwal + next prayer (+ test, dikalibrasi Aladhan)
├── scheduler.rs   # engine trigger adzan/notifikasi (+ test deterministik)
├── audio.rs       # rodio playback (thread terpisah)
├── notify.rs      # toast notification Windows
├── autostart.rs   # registry Run key
├── cities.rs      # 47 kota Indonesia (lat/long/zona waktu)
├── settings.rs    # load/save config JSON (+ tema dark/light)
├── state.rs       # settings global (dibagi antar window & thread)
├── bin/
│   └── gen-icon.rs    # tool: mosque.png → RGBA ikon (just icon)
└── ui/
    ├── mod.rs           # app root, title bar custom, tray, widget window
    ├── icons.rs         # komponen ikon lucide (inline SVG)
    ├── main_view.rs     # jadwal + countdown (desain sample)
    ├── settings_view.rs # settings minimalist
    └── widget.rs        # widget window
assets/
├── input.css      # sumber Tailwind (dark variant class-based)
├── tailwind.css   # hasil build (di-embed, jangan edit manual)
├── mosque.png     # logo (sumber ikon taskbar & tray)
├── icon.rgba      # hasil gen-icon untuk window/taskbar
└── icon-tray.rgba # hasil gen-icon untuk tray
```

## Catatan teknis

- Jadwal diverifikasi terhadap [Aladhan API](https://aladhan.com)
  metode Kemenag: selisih ≤ 2 menit (perbedaan konvensi ihtiyat).
- Trigger adzan pakai window toleransi 2 detik + state "sudah diputar"
  agar tidak dobel/terlewat; reset otomatis saat ganti tanggal.
- Widget adalah window kedua (frameless, always-on-top, skip taskbar)
  yang dibuat tersembunyi saat startup lalu di-show/hide sesuai setting.
