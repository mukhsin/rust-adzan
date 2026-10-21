# Memory — Jadwal Sholat (rust-adzan)

## Aturan kerja (WAJIB)

- **JANGAN commit otomatis.** Setelah menyelesaikan perubahan kode, stop dan
  biarkan user review dulu. Commit HANYA saat user secara eksplisit meminta
  ("commit", "commit ya", dll).
- Validasi sebelum melapor selesai: `just verify` (fmt-check + clippy + test),
  dan untuk perubahan UI/runtime: rebuild release + jalankan sebentar.

## Konteks proyek

- App name tampil ke user: **"Jadwal Sholat"**; nama folder repo & package
  cargo tetap `rust-adzan`.
- Config + log: `%APPDATA%\jadwal-sholat\` (config.json, app.log).
- Installer: NSIS custom `tools/installer.nsi` via `just bundle` (bukan
  `dx bundle` — dx menurunkan dir instalasi & nama artefak dari nama binary).
- Saat build di bawah dx CLI, `build.rs` otomatis skip winresource
  (deteksi `OUT_DIR` mengandung `desktop-*`) untuk hindari konflik resource.
- Audio adzan default ter-embed di binary; urutan: kustom → folder assets →
  embed. Slot pemutaran terpisah: `Slot::Umum` / `Slot::Subuh`.
- Countdown lintas tengah malam (Subuh esok) wajib pakai
  `prayer::time_until` (koreksi +86400); scheduler trigger 2 detik memakai
  selisih mentah hari-yang-sama (jangan tertukar).
- `assets/tailwind.css` hasil generate (`just css` dari `assets/input.css`,
  ter-embed via `include_str!` di `layouts`) dan TIDAK di-track git. Resep
  `css` adalah dependency resep-resep compile di justfile; skrip skip regen
  bila hasil masih segar. Ganti class di rsx → regen otomatis saat `just`
  berikutnya; regen butuh bunx (bun).
