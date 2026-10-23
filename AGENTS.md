# Memory — Jadwal Sholat (rust-adzan)

## Aturan kerja (WAJIB)

- **JANGAN commit otomatis.** Setelah menyelesaikan perubahan kode, stop dan
  biarkan user review dulu. Commit HANYA saat user secara eksplisit meminta
  ("commit", "commit ya", dll).
- Validasi sebelum melapor selesai: `just fmt` + `just verify`
  (fmt-check + clippy + test). JANGAN build release otomatis — lama, dan
  user yang akan build/jalankan sendiri setelah review kode.

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
- Tanda waqf (U+06D6–06DC: ۙ ۚ ۛ dst.) dipisah saat render lewat
  `quran::segmen_teks` supaya bisa distyle/disembunyikan (setting
  "Tanda waqf") — jangan render string ayat mentah. Wajib inline-block
  ber-font-size sendiri: memutus shaping run agar mark-attachment Amiri
  (tanda menggantung tinggi kalau nempel huruf) tidak aktif. Tintanya
  menjorok balik ke kata sebelumnya → tanda NEMPEL huruf (tanpa spasi
  di teks, mis. akhir ayat 2) butuh ms-[10px]; tanda ber-spasi cukup
  ms-[4px]. Mode Mushaf selalu tanpa terjemahan (setting terjemahan
  hanya berlaku di mode Per ayat; pindah ke Mushaf mematikan switch
  terjemahan sekalian). Amiri subset terbukti punya glyph semua tanda
  ini (dicek fontkit).
- Font Arab Amiri (`assets/fonts/amiri-regular.woff2`, subset Arab, OFL):
  daftarkan di `tools/embed-fonts.sh` lalu regen `fonts.css`; dipakai
  kelas `font-arab`.
- Ekspor gambar (`src/export.rs`): SVG dirakit manual → resvg (font
  sistem). Koordinat elemen `<line>` WAJIB named args `format!` —
  argumen posisi pernah ketuker dan menghasilkan garis diagonal.
- Testing: `JADWAL_FAKE_DATE=YYYY-MM-DD` memalsukan tanggal (bisa
  digabung `JADWAL_FAKE_TIME`) — uji mode Ramadan & label Sholat Jumat
  (Dzuhur Jumat = "Sholat Jumat" via `prayer::nama_hari_ini`).
- Puasa sunnah (`src/puasa.rs`): Senin, Kamis, Ayyamul Bidh (13–15 H),
  Syawal (2–7 Syawal), hari penting (Tarwiyah, Arafah, Tasu'a, Asyura).
  Sengaja KOSONG saat Ramadan (pil Ramadan sudah mewakili). Toast
  pengingat dijendela imsak dan MENGGANTIKAN toast imsak generik di
  hari puasa. Uji: `JADWAL_FAKE_DATE` di Senin/Kamis (mis. hari puasa
  ganda: Senin yang jatuh 13–15 hijriah). Urutan pil kanonik dari
  `puasa::SEMUA`, tidak mengikuti urutan Vec di config.
- Tanggal hijriah EFEKTIF berganti saat MAGHRIB, bukan tengah malam —
  helper `prayer::tanggal_efektif_hijriah` dipakai untuk TAMPILAN
  (label hijriah tiket, badge Ramadan, pil puasa, kartu Tanggal
  Penting). JANGAN dipakai untuk trigger adzan/imsak maupun toast puasa
  pagi: semuanya tetap tanggal masehi (puasa Senin = siang Senin
  masehi). Toast persiapan puasa besok = Maghrib + 30 menit, deteksinya
  pakai tanggal masehi BESOK (setting `puasa_sunnah_notif_malam`).
- Jebakan versi: setelah bump/ubah versi, `cargo build --release` bisa
  menganggap exe masih fresh (pernah: installer berlabel 1.4.0 berisi exe
  1.4.1). Sebelum `just bundle`/`dist`, cek
  `(Get-Item 'target\release\rust-adzan.exe').VersionInfo.ProductVersion`
  sama dengan Cargo.toml; kalau basi, hapus manual
  `target/release/rust-adzan.exe` lalu build ulang.
