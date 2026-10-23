//! Render jadwal bulanan menjadi gambar (PNG/JPG) siap cetak.
//!
//! SVG dirakit manual — tanpa dependency HTML — lalu dirasterisasi via
//! `resvg` memakai font sistem (isi tabel latin/angka saja). Latar putih
//! agar hemat tinta saat dicetak, apa pun tema aplikasi.

use crate::i18n::{self, tr};
use crate::prayer::{self, DayTimes};
use crate::settings::{CalcMethod, Language, Settings};
use chrono::{Datelike, Days, NaiveDate};

const MARGIN: f32 = 40.0;
const LEBAR_HARI: f32 = 104.0;
/// Cukup untuk kata "Tanggal" 13px (~50px) + padding — jangan sampai
/// nempel kolom Imsak di sebelahnya.
const LEBAR_TANGGAL: f32 = 66.0;
const LEBAR_JAM: f32 = 100.0;
const TINGGI_BARIS: f32 = 34.0;
const Y_TABEL: f32 = 150.0;
const FOOTER: f32 = 64.0;
/// Jarak teks dari tepi kiri kolom (semua kolom rata kiri).
const PADDING_KOLOM: f32 = 8.0;

const FONT: &str = "Segoe UI";
const WARNA_TEKS: &str = "#111827";
const WARNA_REDUP: &str = "#6b7280";
const WARNA_GARIS: &str = "#e5e7eb";
const WARNA_KEPALA: &str = "#1f2937";
const WARNA_HARI_INI: &str = "#fef3c7";

/// Hasil rasterisasi: PNG siap simpan + dimensi pikselnya.
/// (`lebar`/`tinggi` dibaca unit test untuk verifikasi skala.)
#[allow(dead_code)]
pub struct HasilRender {
    pub lebar: u32,
    pub tinggi: u32,
    pub png: Vec<u8>,
}

/// SVG jadwal bulanan. Tanpa sorotan hari ini — ini arsip sebulan penuh,
/// hari peng-export-an tidak relevan.
pub fn svg_bulanan(settings: &Settings, tahun: i32, bulan: u32) -> String {
    svg_bulanan_dengan(settings, tahun, bulan)
}

fn svg_bulanan_dengan(settings: &Settings, tahun: i32, bulan: u32) -> String {
    let lang = settings.language;
    let awal = NaiveDate::from_ymd_opt(tahun, bulan, 1).expect("bulan valid");
    let bulan_depan = if bulan == 12 {
        NaiveDate::from_ymd_opt(tahun + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(tahun, bulan + 1, 1)
    }
    .expect("bulan valid");
    let jumlah_hari = (bulan_depan - awal).num_days() as u32;

    let lebar = MARGIN * 2.0 + LEBAR_HARI + LEBAR_TANGGAL + 7.0 * LEBAR_JAM;
    let tinggi = Y_TABEL + (jumlah_hari as f32 + 1.0) * TINGGI_BARIS + FOOTER;
    let peride = format!("{} {}", i18n::month_name(lang, bulan as usize - 1), tahun);

    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{lebar}\" height=\"{tinggi}\" \
         viewBox=\"0 0 {lebar} {tinggi}\">\
         <rect width=\"{lebar}\" height=\"{tinggi}\" fill=\"#ffffff\"/>\
         <text x=\"{x}\" y=\"64\" font-family=\"{FONT}\" font-size=\"26\" font-weight=\"700\" \
         fill=\"{WARNA_TEKS}\">{judul}</text>\
         <text x=\"{x}\" y=\"96\" font-family=\"{FONT}\" font-size=\"18\" \
         fill=\"{WARNA_TEKS}\">{peride}</text>\
         <text x=\"{x}\" y=\"124\" font-family=\"{FONT}\" font-size=\"13\" \
         fill=\"{WARNA_REDUP}\">{info}</text>",
        x = MARGIN,
        judul = escape_xml(crate::settings::APP_NAME),
        info = escape_xml(&info_line(settings)),
    );

    // Baris kepala tabel — semua kolom rata kiri (ala imsakiyah cetak).
    let kepala = [
        "hari", "tgl", "imsak", "subuh", "terbit", "dzuhur", "ashar", "maghrib", "isya",
    ];
    svg += &format!(
        "<rect x=\"{MARGIN}\" y=\"{Y_TABEL}\" width=\"{w}\" height=\"{TINGGI_BARIS}\" \
         fill=\"{WARNA_KEPALA}\"/>",
        w = lebar - MARGIN * 2.0
    );
    for (i, kunci) in kepala.iter().enumerate() {
        svg += &teks_rata(
            tepi_kolom(i),
            Y_TABEL + 22.0,
            tr(lang, kunci),
            13.0,
            "600",
            "#ffffff",
            "start",
        );
    }

    for hari in 1..=jumlah_hari {
        let tanggal = awal + Days::new(hari as u64 - 1);
        let y = Y_TABEL + hari as f32 * TINGGI_BARIS;
        let jadwal = prayer::day_times(settings, tanggal);

        // Garis bawah baris.
        svg += &format!(
            "<line x1=\"{MARGIN}\" y1=\"{y1}\" x2=\"{x2}\" y2=\"{y2}\" stroke=\"{WARNA_GARIS}\" \
             stroke-width=\"1\"/>",
            y1 = y + TINGGI_BARIS,
            x2 = lebar - MARGIN,
            y2 = y + TINGGI_BARIS
        );

        // Hari & tanggal pada kolom terpisah.
        svg += &teks_rata(
            tepi_kolom(0),
            y + 22.0,
            i18n::day_name(lang, tanggal.weekday().num_days_from_monday() as usize),
            13.0,
            "400",
            WARNA_TEKS,
            "start",
        );
        svg += &teks_rata(
            tepi_kolom(1),
            y + 22.0,
            &hari.to_string(),
            13.0,
            "600",
            WARNA_TEKS,
            "start",
        );

        for i in 0..7 {
            let waktu = jadwal.map(|j| waktu_kolom(&j, i));
            let isi = waktu.as_deref().unwrap_or("-").to_string();
            svg += &teks_rata(
                tepi_kolom(i + 2),
                y + 22.0,
                &isi,
                14.0,
                "400",
                WARNA_TEKS,
                "start",
            );
        }
    }

    svg += &format!(
        "<text x=\"{}\" y=\"{}\" font-family=\"{FONT}\" font-size=\"12\" text-anchor=\"middle\" \
         fill=\"{WARNA_REDUP}\">{} {}</text></svg>",
        lebar / 2.0,
        tinggi - 24.0,
        tr(lang, "dibuat_dengan"),
        escape_xml(crate::settings::APP_NAME),
    );
    svg
}

/// Jam kolom ke-i (0=Imsak … 6=Isya) dalam format 24 jam.
fn waktu_kolom(j: &DayTimes, i: usize) -> String {
    let t = match i {
        0 => j.imsak,
        1 => j.subuh,
        2 => j.terbit,
        3 => j.dzuhur,
        4 => j.ashar,
        5 => j.maghrib,
        _ => j.isya,
    };
    prayer::format_time(t, true)
}

/// Tepi kiri kolom ke-i (0=Hari, 1=Tgl, 2..=jam) — anchor rata kiri.
fn tepi_kolom(i: usize) -> f32 {
    let dasar = match i {
        0 => MARGIN,
        1 => MARGIN + LEBAR_HARI,
        _ => MARGIN + LEBAR_HARI + LEBAR_TANGGAL + (i as f32 - 2.0) * LEBAR_JAM,
    };
    dasar + PADDING_KOLOM
}

/// Elemen `<text>` rata tengah horizontal.
fn teks(x: f32, y: f32, isi: &str, ukuran: f32, tebal: &str, warna: &str) -> String {
    teks_rata(x, y, isi, ukuran, tebal, warna, "middle")
}

/// Elemen `<text>` dengan anchor bebas (start/middle/end).
fn teks_rata(
    x: f32,
    y: f32,
    isi: &str,
    ukuran: f32,
    tebal: &str,
    warna: &str,
    rata: &str,
) -> String {
    format!(
        "<text x=\"{x}\" y=\"{y}\" font-family=\"{FONT}\" font-size=\"{ukuran}\" \
         font-weight=\"{tebal}\" text-anchor=\"{rata}\" fill=\"{warna}\">{}</text>",
        escape_xml(isi)
    )
}

/// Baris info di bawah judul: lokasi · metode — ringkas, tanpa derajat
/// sudut & zona waktu (tidak berguna untuk arsip cetak).
fn info_line(settings: &Settings) -> String {
    let lokasi = settings
        .city
        .clone()
        .unwrap_or_else(|| format!("{:.4}, {:.4}", settings.lat, settings.lng));
    format!("{} · {}", lokasi, nama_metode_ringkas(settings.method))
}

/// Nama metode ringkas untuk ekspor (label dropdown punya derajat sudut).
fn nama_metode_ringkas(m: CalcMethod) -> &'static str {
    match m {
        CalcMethod::Kemenag => "Kemenag",
        CalcMethod::MuslimWorldLeague => "Muslim World League",
        CalcMethod::Egyptian => "Egyptian",
        CalcMethod::Karachi => "Karachi",
        CalcMethod::UmmAlQura => "Umm al-Qura",
        CalcMethod::NorthAmerica => "ISNA",
    }
}

/// Rasterisasi SVG menjadi PNG pada `skala` (mis. 2.0 = resolusi 2×).
fn render_png(svg: &str, skala: f32) -> Result<HasilRender, String> {
    let mut fontdb = resvg::usvg::fontdb::Database::new();
    fontdb.load_system_fonts();
    let opt = resvg::usvg::Options {
        font_family: FONT.to_string(),
        fontdb: std::sync::Arc::new(fontdb),
        ..Default::default()
    };

    let tree =
        resvg::usvg::Tree::from_str(svg, &opt).map_err(|e| format!("SVG tidak valid: {e}"))?;
    let size = tree
        .size()
        .to_int_size()
        .scale_by(skala)
        .ok_or("ukuran gambar tidak valid")?;
    let mut pixmap = resvg::tiny_skia::Pixmap::new(size.width(), size.height())
        .ok_or("gagal membuat kanvas gambar")?;
    pixmap.fill(resvg::tiny_skia::Color::from_rgba8(255, 255, 255, 255));
    resvg::render(
        &tree,
        resvg::tiny_skia::Transform::from_scale(skala, skala),
        &mut pixmap.as_mut(),
    );
    let png = pixmap.encode_png().map_err(|e| e.to_string())?;
    Ok(HasilRender {
        lebar: size.width(),
        tinggi: size.height(),
        png,
    })
}

/// Konversi PNG → JPG (JPEG tanpa alpha, kualitas 1–100).
fn png_ke_jpg(png: &[u8], kualitas: u8) -> Result<Vec<u8>, String> {
    use image::codecs::jpeg::JpegEncoder;
    let img = image::load_from_memory(png).map_err(|e| e.to_string())?;
    let mut keluar = std::io::Cursor::new(Vec::new());
    img.to_rgb8()
        .write_with_encoder(JpegEncoder::new_with_quality(&mut keluar, kualitas))
        .map_err(|e| e.to_string())?;
    Ok(keluar.into_inner())
}

/// Render + encode sekaligus sesuai ekstensi file tujuan (.png/.jpg/.jpeg).
/// Format lain batal dengan `Err`.
pub fn render_bulanan(
    settings: &Settings,
    tahun: i32,
    bulan: u32,
    tujuan: &std::path::Path,
) -> Result<Vec<u8>, String> {
    let svg = svg_bulanan(settings, tahun, bulan);
    encode_svg(&svg, tujuan)
}

/// Encode SVG → PNG/JPG sesuai ekstensi file tujuan.
fn encode_svg(svg: &str, tujuan: &std::path::Path) -> Result<Vec<u8>, String> {
    let hasil = render_png(svg, 2.0)?;
    match tujuan
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_ascii_lowercase())
        .as_deref()
    {
        Some("png") | None => Ok(hasil.png),
        Some("jpg") | Some("jpeg") => png_ke_jpg(&hasil.png, 90),
        Some(lain) => Err(format!("format .{lain} tidak didukung")),
    }
}

/// Render kalender satu bulan (lihat [`svg_kalender`]) sesuai ekstensi.
pub fn render_kalender(
    settings: &Settings,
    tahun: i32,
    bulan: u32,
    tujuan: &std::path::Path,
) -> Result<Vec<u8>, String> {
    let svg = svg_kalender(settings, tahun, bulan);
    encode_svg(&svg, tujuan)
}

// ===== Kalender Masehi–Hijriah–Jawa =====

const SEL_LEBAR: f32 = 118.0;
const SEL_TINGGI: f32 = 124.0;
const Y_GRID: f32 = 176.0;

const WARNA_AHAD: &str = "#b3492f";
const WARNA_JUMAT: &str = "#3f7d54";
const WARNA_DIM: &str = "#6e675a";
const WARNA_HARI_INI_TEKS: &str = "#92400e";

/// SVG kalender satu bulan ala layar Kalender: kolom Ahad-first, tiap sel
/// memuat angka hijriah (Arab-Indic, kanan-atas), tanggal masehi besar di
/// tengah, dan baris weton (tgl Jawa · pasaran · neptu). Hari ini disorot
/// amber; Ahad merah, Jum'at hijau; hari puasa sunnah tint amber; tabel
/// dikelilingi border luar selain garis antar sel.
pub fn svg_kalender(settings: &Settings, tahun: i32, bulan: u32) -> String {
    let lang = settings.language;
    let today = prayer::now_in_tz(settings).date_naive();
    let first = NaiveDate::from_ymd_opt(tahun, bulan, 1).expect("bulan valid");
    let (next_y, next_m) = if bulan == 12 {
        (tahun + 1, 1)
    } else {
        (tahun, bulan + 1)
    };
    let last = NaiveDate::from_ymd_opt(next_y, next_m, 1).expect("bulan valid") - Days::new(1);
    let last_day = last.day() as usize;

    let lead = first.weekday().num_days_from_sunday() as usize;
    let weeks = (lead + last_day).div_ceil(7);
    let grid_first = first - Days::new(lead as u64);

    let lebar = MARGIN * 2.0 + 7.0 * SEL_LEBAR;
    let tinggi = Y_GRID + weeks as f32 * SEL_TINGGI + FOOTER;

    // Rentang hijriah grid — sama seperti layar Kalender.
    let (h1y, m1, d1) = crate::hijri::to_hijri(grid_first);
    let (h2, m2, d2) = crate::hijri::to_hijri(last);
    let _ = h1y;
    let rentang = format!(
        "{} {} – {} {} {} H",
        d1,
        i18n::hijri_month_name(lang, (m1 - 1) as usize),
        d2,
        i18n::hijri_month_name(lang, (m2 - 1) as usize),
        h2
    );
    let kota = settings
        .city
        .clone()
        .unwrap_or_else(|| format!("{:.4}, {:.4}", settings.lat, settings.lng));
    let periode = format!("{} {}", i18n::month_name(lang, bulan as usize - 1), tahun);

    let mut svg = format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{lebar}\" height=\"{tinggi}\" \
         viewBox=\"0 0 {lebar} {tinggi}\">\
         <rect width=\"{lebar}\" height=\"{tinggi}\" fill=\"#ffffff\"/>\
         <text x=\"{x}\" y=\"64\" font-family=\"{FONT}\" font-size=\"26\" font-weight=\"700\" \
         fill=\"{WARNA_TEKS}\">{judul}</text>\
         <text x=\"{x}\" y=\"96\" font-family=\"{FONT}\" font-size=\"18\" \
         fill=\"{WARNA_TEKS}\">{periode}</text>\
         <text x=\"{x}\" y=\"124\" font-family=\"{FONT}\" font-size=\"13\" \
         fill=\"{WARNA_REDUP}\">{rentang} · {kota}</text>",
        x = MARGIN,
        judul = escape_xml(crate::settings::APP_NAME),
        rentang = escape_xml(&rentang),
        kota = escape_xml(&kota),
    );

    // Kepala kolom Ahad..Sabtu (Ahad merah, Jum'at hijau).
    for i in 0..7usize {
        let chrono_idx = if i == 0 { 6 } else { i - 1 };
        let latin = match i {
            0 => match lang {
                Language::Id => "Ahad",
                Language::En => "Sunday",
            },
            5 => i18n::day_name(lang, 4),
            _ => i18n::day_name(lang, chrono_idx),
        };
        let warna = match i {
            0 => WARNA_AHAD,
            5 => WARNA_JUMAT,
            _ => WARNA_DIM,
        };
        svg += &teks(
            MARGIN + i as f32 * SEL_LEBAR + SEL_LEBAR / 2.0,
            Y_GRID - 14.0,
            latin,
            11.0,
            "600",
            warna,
        );
    }

    for idx in 0..weeks * 7 {
        let col = idx % 7;
        let row = idx / 7;
        let date = grid_first + Days::new(idx as u64);
        let x0 = MARGIN + col as f32 * SEL_LEBAR;
        let y0 = Y_GRID + row as f32 * SEL_TINGGI;
        let in_month = date.month() == bulan && date.year() == tahun;
        let is_today = date == today;
        let weekday = date.weekday().num_days_from_sunday(); // 0 = Ahad
        let (_, h_m, h_d) = crate::hijri::to_hijri(date);

        if is_today {
            svg += &format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"6\" \
                 fill=\"{WARNA_HARI_INI}\"/>",
                x0 + 2.0,
                y0 + 2.0,
                SEL_LEBAR - 4.0,
                SEL_TINGGI - 4.0
            );
        } else if h_m == 9 {
            // Puasa wajib bulan Ramadhan: tint ungu muda — beda warna dari
            // tint amber puasa sunnah (layar: bg-ramadan/10).
            svg += &format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"6\" \
                 fill=\"#ddd6fe\"/>",
                x0 + 2.0,
                y0 + 2.0,
                SEL_LEBAR - 4.0,
                SEL_TINGGI - 4.0
            );
        } else if !crate::puasa::daftar_hari_ini(date, &settings.puasa_sunnah).is_empty() {
            // Hari puasa sunnah: tint amber lembut (layar: bg-amber/10) —
            // sel today tetap solid amber, prioritas.
            svg += &format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"6\" \
                 fill=\"{WARNA_HARI_INI}\" fill-opacity=\"0.45\"/>",
                x0 + 2.0,
                y0 + 2.0,
                SEL_LEBAR - 4.0,
                SEL_TINGGI - 4.0
            );
        }
        // Pemisah sel: garis kanan (kecuali kolom terakhir) & bawah
        // (kecuali baris terakhir). Named args — argumen posisi pernah
        // ketuker dan menghasilkan garis diagonal corat-coret.
        if col < 6 {
            svg += &format!(
                "<line x1=\"{gx}\" y1=\"{y0}\" x2=\"{gx}\" y2=\"{gy}\" \
                 stroke=\"{WARNA_GARIS}\" stroke-width=\"1\"/>",
                gx = x0 + SEL_LEBAR,
                gy = y0 + SEL_TINGGI
            );
        }
        if row + 1 < weeks {
            svg += &format!(
                "<line x1=\"{x0}\" y1=\"{gy}\" x2=\"{gx}\" y2=\"{gy}\" \
                 stroke=\"{WARNA_GARIS}\" stroke-width=\"1\"/>",
                gx = x0 + SEL_LEBAR,
                gy = y0 + SEL_TINGGI
            );
        }

        let warna_masehi = if is_today {
            WARNA_HARI_INI_TEKS
        } else if !in_month {
            WARNA_DIM
        } else {
            match weekday {
                0 => WARNA_AHAD,
                5 => WARNA_JUMAT,
                _ => WARNA_TEKS,
            }
        };

        // Angka hijriah kanan-atas (Arab-Indic).
        svg += &teks_rata(
            x0 + SEL_LEBAR - 10.0,
            y0 + 22.0,
            &crate::screens::calendar::arabic_numerals(h_d),
            12.0,
            "400",
            WARNA_DIM,
            "end",
        );
        // Tanggal masehi besar di tengah.
        svg += &teks_rata(
            x0 + SEL_LEBAR / 2.0,
            y0 + 72.0,
            &date.day().to_string(),
            26.0,
            "700",
            warna_masehi,
            "middle",
        );
        // Baris weton: tgl Jawa · pasaran · neptu.
        let (weton_tgl, pasaran, neptu) = crate::screens::calendar::info_weton(date);
        let warna_weton = if is_today {
            WARNA_HARI_INI_TEKS
        } else if !in_month {
            WARNA_DIM
        } else {
            match weekday {
                0 => WARNA_AHAD,
                5 => WARNA_JUMAT,
                _ => WARNA_DIM,
            }
        };
        svg += &teks_rata(
            x0 + 10.0,
            y0 + SEL_TINGGI - 10.0,
            &weton_tgl.to_string(),
            9.5,
            "600",
            warna_weton,
            "start",
        );
        svg += &teks_rata(
            x0 + SEL_LEBAR / 2.0,
            y0 + SEL_TINGGI - 10.0,
            &pasaran.to_uppercase(),
            9.5,
            "600",
            warna_weton,
            "middle",
        );
        svg += &teks_rata(
            x0 + SEL_LEBAR - 10.0,
            y0 + SEL_TINGGI - 10.0,
            &neptu.to_string(),
            9.5,
            "600",
            warna_weton,
            "end",
        );
    }

    // Border luar tabel kalender — garis antar sel hanya internal;
    // perimeter perlu satu rect penutup agar tabel terbungkus penuh.
    svg += &format!(
        "<rect x=\"{MARGIN}\" y=\"{Y_GRID}\" width=\"{}\" height=\"{}\" \
         fill=\"none\" stroke=\"{WARNA_GARIS}\" stroke-width=\"1\"/>",
        7.0 * SEL_LEBAR,
        weeks as f32 * SEL_TINGGI
    );

    // Keterangan: hari ini, Ahad, Jum'at, puasa sunnah, Ramadhan.
    let mut keterangan_x = MARGIN;
    let legenda = [
        (WARNA_HARI_INI, tr(lang, "today").to_uppercase()),
        (WARNA_AHAD, "AHAD".to_string()),
        (WARNA_JUMAT, i18n::day_name(lang, 4).to_uppercase()),
        (
            // Amber-200 — lebih pekat dari tint sel agar swatch terbaca.
            "#fde68a",
            tr(lang, "sec_puasa").to_uppercase(),
        ),
        (
            // Violet-300 — swatch tint Ramadhan.
            "#c4b5fd",
            i18n::hijri_month_name(lang, 8).to_uppercase(),
        ),
    ];
    for (warna, label) in legenda {
        svg += &format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"4\" fill=\"{warna}\"/>",
            keterangan_x + 4.0,
            tinggi - 27.0
        );
        svg += &teks_rata(
            keterangan_x + 14.0,
            tinggi - 24.0,
            &label,
            10.0,
            "600",
            WARNA_DIM,
            "start",
        );
        keterangan_x += 24.0 + label.len() as f32 * 7.0;
    }
    svg += &format!(
        "<text x=\"{}\" y=\"{}\" font-family=\"{FONT}\" font-size=\"12\" text-anchor=\"middle\" \
         fill=\"{WARNA_REDUP}\">{} {}</text></svg>",
        lebar / 2.0,
        tinggi - 24.0,
        tr(lang, "dibuat_dengan"),
        escape_xml(crate::settings::APP_NAME),
    );
    svg
}

/// Jenis gambar yang bisa diunduh dari layar Bulanan / Kalender.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JenisUnduhan {
    /// Jadwal sholat satu bulan (tabel imsakiyah).
    Bulanan,
    /// Kalender Masehi–Hijriah–Jawa satu bulan.
    Kalender,
}

/// Alur unduh gambar, dipanggil dari onclick (main thread): dialog simpan,
/// lalu render di thread latar dan toast hasilnya.
pub fn unduh_gambar(settings: &Settings, jenis: JenisUnduhan, tahun: i32, bulan: u32) {
    let lang = settings.language;
    let awalan = match jenis {
        JenisUnduhan::Bulanan => "jadwal-sholat",
        JenisUnduhan::Kalender => "kalender-sholat",
    };
    let Some(tujuan) = rfd::FileDialog::new()
        .set_file_name(format!("{awalan}-{bulan:02}-{tahun}.png").as_str())
        .add_filter("Gambar", &["png", "jpg", "jpeg"])
        .set_title(tr(lang, "unduh_title"))
        .save_file()
    else {
        return; // user batal
    };

    let settings = settings.clone();
    std::thread::Builder::new()
        .name("unduh-gambar".into())
        .spawn(move || {
            let hasil = match jenis {
                JenisUnduhan::Bulanan => render_bulanan(&settings, tahun, bulan, &tujuan),
                JenisUnduhan::Kalender => render_kalender(&settings, tahun, bulan, &tujuan),
            }
            .and_then(|data| std::fs::write(&tujuan, data).map_err(|e| e.to_string()));
            let nama = tujuan
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            match hasil {
                Ok(()) => {
                    crate::logger::info(&format!("gambar tersimpan: {}", tujuan.display()));
                    crate::notify::show(
                        tr(lang, "unduh_berhasil_title"),
                        &tr(lang, "unduh_berhasil_body").replace("{nama}", &nama),
                    );
                }
                Err(e) => {
                    crate::logger::error(&format!("unduh gambar gagal: {e}"));
                    crate::notify::show(
                        tr(lang, "unduh_gagal_title"),
                        &tr(lang, "unduh_gagal_body").replace("{galat}", &e),
                    );
                }
            }
        })
        .expect("gagal spawn unduh-gambar");
}

/// Escape teks untuk konten elemen/atribut XML.
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn svg_memuat_header_dan_semua_hari() {
        let s = Settings::default();
        let svg = svg_bulanan(&s, 2026, 10);

        assert!(svg.contains("Jadwal Sholat"), "judul: {svg}");
        assert!(svg.contains("Oktober 2026"), "periode: {svg}");
        assert!(svg.contains("Jakarta"), "lokasi: {svg}");
        // Info ringkas: metode tanpa derajat, tanpa zona waktu.
        assert!(svg.contains("· Kemenag<"), "metode ringkas: {svg}");
        assert!(!svg.contains("20°"), "derajat ikut terekspor: {svg}");
        assert!(!svg.contains("Asia/Jakarta"), "zona ikut terekspor: {svg}");

        for kunci in [
            "hari", "tgl", "imsak", "subuh", "terbit", "dzuhur", "ashar", "maghrib", "isya",
        ] {
            assert!(
                svg.contains(tr(Language::Id, kunci)),
                "kolom {kunci}: {svg}"
            );
        }

        // Oktober = 31 hari; Februari 2027 = 28 hari (sel tanggal: "31").
        assert!(svg.contains(">31<"), "hari 31 hilang");
        assert!(svg.contains(">Sabtu<"), "nama hari hilang");
        let pendek = svg_bulanan(&s, 2027, 2);
        assert!(pendek.contains(">28<"));
        assert!(!pendek.contains(">29<"));
    }

    #[test]
    fn bulanan_tanpa_sorotan_hari_ini() {
        // Arsip sebulan penuh: penanda hari peng-export-an tidak relevan.
        let s = Settings::default();
        let svg = svg_bulanan(&s, 2026, 10);
        assert!(!svg.contains(WARNA_HARI_INI));
    }

    #[test]
    fn escape_xml_karakter_khusus() {
        assert_eq!(escape_xml("a<b>&\"c\""), "a&lt;b&gt;&amp;&quot;c&quot;");
    }

    #[test]
    fn rasterisasi_png_lalu_jpg() {
        let s = Settings::default();
        let svg = svg_bulanan(&s, 2026, 10);
        let hasil = render_png(&svg, 2.0).expect("render berhasil");

        // PNG magic + lebar 2× (dasar 950 → 1900).
        assert_eq!(&hasil.png[..4], &[0x89, b'P', b'N', b'G']);
        assert_eq!(hasil.lebar, 1900);

        let jpg = png_ke_jpg(&hasil.png, 85).expect("jpg berhasil");
        assert_eq!(&jpg[..2], &[0xFF, 0xD8]);
    }

    #[test]
    fn render_bulanan_ikut_ekstensi() {
        let s = Settings::default();
        let png =
            render_bulanan(&s, 2026, 10, std::path::Path::new("jadwal.png")).expect("png berhasil");
        assert_eq!(&png[..4], &[0x89, b'P', b'N', b'G']);

        let jpg =
            render_bulanan(&s, 2026, 10, std::path::Path::new("jadwal.jpg")).expect("jpg berhasil");
        assert_eq!(&jpg[..2], &[0xFF, 0xD8]);

        assert!(render_bulanan(&s, 2026, 10, std::path::Path::new("jadwal.gif")).is_err());
    }

    #[test]
    fn svg_kalender_memuat_grid_dan_hijriah() {
        let s = Settings::default();
        let svg = svg_kalender(&s, 2026, 10);

        assert!(svg.contains("Oktober 2026"), "periode: {svg}");
        assert!(svg.contains("Jakarta"), "kota: {svg}");
        // Kolom Ahad-first + legend.
        assert!(svg.contains(">Ahad<"), "kolom ahad: {svg}");
        assert!(svg.contains(">Sabtu<"), "kolom sabtu: {svg}");
        assert!(svg.contains(&tr(Language::Id, "today").to_uppercase()));
        // Angka Arab-Indic (tanggal hijriah) muncul di sel.
        assert!(
            svg.contains('\u{0660}') || svg.contains('\u{0661}'),
            "angka hijriah hilang"
        );
        // Baris weton: nama pasaran ter-uppercase.
        assert!(svg.contains("PAHING") || svg.contains("KLIWON") || svg.contains("WAGE"));
        // Sel tanpa today juga valid (bulan bebas highlight).
        let svg2 = svg_kalender(&s, 2027, 2);
        assert!(svg2.contains("Februari 2027"));
    }

    /// Ekstrak nilai atribut numerik dari satu elemen `<line …>`.
    fn atribut_line(segmen: &str, kunci: &str) -> f32 {
        let pola = format!("{kunci}=\"");
        let awal = segmen.find(&pola).expect("atribut line ada") + pola.len();
        let akhir = segmen[awal..].find('"').expect("atribut tertutup");
        segmen[awal..awal + akhir].parse().expect("angka koordinat")
    }

    #[test]
    fn garis_kalender_semua_lurus() {
        // Regresi: argumen format! pernah ketuker sehingga garis batas sel
        // jadi diagonal corat-coret. Semua <line> harus vertikal ATAU
        // horizontal murni.
        let s = Settings::default();
        let svg = svg_kalender(&s, 2026, 10);
        assert!(svg.contains("<line"), "tidak ada garis sama sekali");

        for segmen in svg.split("<line ").skip(1) {
            let isi = &segmen[..segmen.find("/>").expect("elemen line self-closing")];
            let (x1, y1, x2, y2) = (
                atribut_line(isi, "x1"),
                atribut_line(isi, "y1"),
                atribut_line(isi, "x2"),
                atribut_line(isi, "y2"),
            );
            assert!(
                x1 == x2 || y1 == y2,
                "garis diagonal: ({x1},{y1}) → ({x2},{y2})"
            );
        }
    }

    #[test]
    fn render_kalender_png_jpg() {
        let s = Settings::default();
        let png = render_kalender(&s, 2026, 10, std::path::Path::new("kalender.png"))
            .expect("png berhasil");
        assert_eq!(&png[..4], &[0x89, b'P', b'N', b'G']);

        let jpg = render_kalender(&s, 2026, 10, std::path::Path::new("kalender.jpg"))
            .expect("jpg berhasil");
        assert_eq!(&jpg[..2], &[0xFF, 0xD8]);
    }
}
