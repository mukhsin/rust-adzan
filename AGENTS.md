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
- Data Al-Qur'an ter-embed (`assets/quran.json`): regen via
  `bun tools/quran-build.mjs` (mushaf + terjemahan Kemenag; output
  di-commit). Layer: `src/quran.rs` (zero-copy, OnceLock). Jangan edit
  JSON manual.
- Font Arab Amiri (`assets/fonts/amiri-regular.woff2`, subset Arab, OFL):
  daftarkan di `tools/embed-fonts.sh` lalu regen `fonts.css`; dipakai
  kelas `font-arab`.
- Ekspor gambar (`src/export.rs`): SVG dirakit manual → resvg (font
  sistem). Koordinat elemen `<line>` WAJIB named args `format!` —
  argumen posisi pernah ketuker dan menghasilkan garis diagonal.
- Testing: `JADWAL_FAKE_DATE=YYYY-MM-DD` memalsukan tanggal (bisa
  digabung `JADWAL_FAKE_TIME`) — uji mode Ramadan & label Sholat Jumat
  (Dzuhur Jumat = "Sholat Jumat" via `prayer::nama_hari_ini`).
