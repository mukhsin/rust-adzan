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
- **Audio adzan** — putar MP3 saat masuk waktu; file khusus Subuh
  opsional; fallback bunyi beep jika file belum diatur.
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
just dev      # hot-reload via dx serve
just run      # cargo run sekali
just test     # unit test
just fmt      # format kode
just clippy   # linter
just verify   # fmt-check + clippy + test
just release  # build rilis
just bundle   # installer MSI
```

### Audio adzan

Letakkan file `assets/adzan.mp3` (dan opsional `assets/adzan_subuh.mp3`),
atau pilih file audio apa pun (MP3/WAV/OGG/FLAC/M4A/AAC) lewat menu
**Pengaturan** di aplikasi. Tanpa file, aplikasi tetap memberi tanda
dengan bunyi beep.

### Ikon aplikasi (taskbar & tray)

Ikon diambil dari `assets/mosque.png`, dipakai apa adanya (transparan).
Setelah mengganti PNG, jalankan `just icon` untuk men-generate
`assets/icon.rgba` + `assets/icon-tray.rgba` yang di-embed ke binary.

## Build rilis

```bash
cargo build --release   # binary di target/release/rust-adzan.exe
dx bundle               # installer MSI (opsional)
```

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
