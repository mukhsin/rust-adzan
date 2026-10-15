# Assets

## Audio adzan

- `adzan.mp3` — adzan bawaan (Dzuhur, Ashar, Maghrib, Isya)
- `adzan-subuh.mp3` — adzan bawaan khusus Subuh

Kedua file ini **di-embed ke binary** (`include_bytes!` di `src/audio.rs`)
sehingga selalu tersedia walau exe dipindah. Setelah mengganti file,
build ulang aplikasi.

User tetap bisa memakai file sendiri lewat **Pengaturan → Audio Adzan**
(Pilih…) tanpa perlu build ulang — file kustom diprioritaskan di atas
bawaan.

## Ikon

- `mosque.png` / `mosque.svg` — logo (sumber ikon taskbar & tray);
  setelah mengganti PNG jalankan `just icon`
- `icon.rgba` / `icon-tray.rgba` — hasil konversi (di-embed)

## CSS

- `input.css` — sumber Tailwind; regenerate `tailwind.css` via `just css`
