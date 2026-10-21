//! Pemeriksaan pembaruan via dua sumber:
//! - **Share kantor** (`\\svrbtch-app-u01\SHARE APPLICATION\Jadwal
//!   Sholat\vX.Y.Z\`): untuk jaringan/VPN kantor tempat GitHub tak
//!   terjangkau; dipindai dengan `std::fs` biasa (UNC = folder biasa).
//! - **GitHub Releases**: sumber publik.
//!
//! Utamakan share kantor; bila gagal, fallback ke GitHub. Thread latar (`start`)
//! mengecek saat startup lalu tiap 1 jam; bila versinya lebih baru dari
//! binary, toast tampil SEKALI per versi (memo di `update-check.json` di
//! folder config) — klik toast membuka tempat sesuai sumber: folder share
//! di Explorer, atau halaman rilis di browser. [`check_now_manual`] dipakai
//! tombol "Cek sekarang" di pengaturan: hasilnya selalu di-toast, tanpa
//! memo.

use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicBool, Ordering};

/// Repo GitHub tempat rilis dipublikasikan (lihat `just release-all`).
const REPO: &str = "mukhsin/rust-adzan";

/// Root shared folder kantor — struktur: `<root>\vX.Y.Z\Jadwal Sholat
/// X.Y.Z x64-setup.exe` (konvensi sama dengan tools/check-update-share.ps1).
const SHARE_ROOT: &str = r"\\svrbtch-app-u01\SHARE APPLICATION\Jadwal Sholat";

/// Jeda antar-pemeriksaan otomatis.
const INTERVAL: std::time::Duration = std::time::Duration::from_secs(3600);

/// Pemicu cek langsung — disetel [`picu_cek_otomatis`] saat sakelar
/// diatur ke Otomatis, di-consume tick loop berikutnya.
static PICU_SEKARANG: AtomicBool = AtomicBool::new(false);

const USER_AGENT: &str = concat!(
    "jadwal-sholat/",
    env!("CARGO_PKG_VERSION"),
    " (github.com/mukhsin/rust-adzan)"
);

/// Sumber rilis terbaru — menentukan target klik toast.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sumber {
    /// Share kantor: klik membuka folder rilis di Explorer.
    Share,
    /// GitHub: klik membuka halaman rilis di browser.
    GitHub,
}

/// Rilis yang relevan.
#[derive(Debug, Clone, PartialEq)]
pub struct Release {
    /// Tag rilis, mis. "v1.1.0" (bisa juga "1.1.0" — dinormalisasi saat
    /// dibandingkan).
    pub tag: String,
    /// Target klik: URL halaman rilis (GitHub) atau path folder (share).
    pub url: String,
}

/// Memo toast agar satu versi tidak mengganggu berulang kali.
#[derive(Debug, Default, Serialize, Deserialize)]
struct UpdateState {
    notified_tag: Option<String>,
}

fn state_path() -> std::path::PathBuf {
    crate::settings::config_dir().join("update-check.json")
}

fn load_state() -> UpdateState {
    std::fs::read_to_string(state_path())
        .ok()
        .and_then(|text| serde_json::from_str(&text).ok())
        .unwrap_or_default()
}

fn save_state(state: &UpdateState) {
    match serde_json::to_string(state) {
        Ok(text) => {
            if let Err(e) = std::fs::write(state_path(), text) {
                crate::logger::error(&format!("gagal simpan state update: {e}"));
            }
        }
        Err(e) => crate::logger::error(&format!("gagal serialisasi state update: {e}")),
    }
}

/// Ambil rilis terbaru (bukan draft/prerelease) dari GitHub.
/// `None` = gagal (offline, rate-limit, respons aneh) — dicatat di log saja.
pub fn latest_release() -> Option<Release> {
    let url = format!("https://api.github.com/repos/{REPO}/releases/latest");
    let mut response = ureq::get(&url)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .call()
        .inspect_err(|e| crate::logger::info(&format!("cek pembaruan gagal: {e}")))
        .ok()?;
    let body = response.body_mut().read_to_string().ok()?;
    let json: serde_json::Value = serde_json::from_str(&body).ok()?;
    let tag = json["tag_name"].as_str()?.to_string();
    let url = json["html_url"].as_str()?.to_string();
    Some(Release { tag, url })
}

/// Pindai folder share `root`: cari subfolder `vX.Y.Z` dengan versi
/// tertinggi yang installer-nya benar-benar ada. `None` = share tak
/// terjangkau / belum ada rilis valid.
fn latest_from_share_dir(root: &std::path::Path) -> Option<Release> {
    let bacaan = std::fs::read_dir(root)
        .inspect_err(|e| crate::logger::info(&format!("share kantor tak terjangkau: {e}")))
        .ok()?;

    // Urut menurun, lalu ambil folder PERTAMA yang installer-nya lengkap:
    // folder terbaru yang upload-nya belum selesai dilewati, bukan
    // menyembunyikan rilis valid di bawahnya.
    let mut kandidat: Vec<((u64, u64, u64), String)> = bacaan
        .flatten()
        .filter_map(|entri| {
            let nama = entri.file_name().to_string_lossy().to_string();
            parse_semver(&nama).map(|ver| (ver, nama))
        })
        .collect();
    kandidat.sort_by_key(|(ver, _)| std::cmp::Reverse(*ver));

    for (_, nama) in kandidat {
        let versi = nama.trim_start_matches('v');
        let installer = root
            .join(&nama)
            .join(format!("Jadwal Sholat {versi} x64-setup.exe"));
        if installer.exists() {
            return Some(Release {
                tag: nama.clone(),
                url: root.join(&nama).to_string_lossy().to_string(),
            });
        }
        crate::logger::info(&format!(
            "share: installer {nama} tidak ditemukan, lanjut ke versi di bawahnya"
        ));
    }
    None
}

/// Rilis terbaru dari share kantor.
fn latest_from_share() -> Option<Release> {
    latest_from_share_dir(std::path::Path::new(SHARE_ROOT))
}

/// Rilis terbaru: **utamakan share kantor**; bila share gagal (tak
/// terjangkau / belum ada rilis valid), fallback ke GitHub. Kedua sumber
/// tidak pernah bisa diakses bersamaan — VPN kantor memblokir GitHub,
/// tanpa VPN share tak terjangkau — jadi tidak ada pembandingan antar
/// sumber, dan saat share berhasil GitHub tidak disentuh sama sekali.
fn latest_overall() -> Option<(Release, Sumber)> {
    if let Some(release) = latest_from_share() {
        return Some((release, Sumber::Share));
    }
    latest_release().map(|r| (r, Sumber::GitHub))
}

/// Parse "v1.2.3" / "1.2.3" menjadi tuple versi; `None` bila tak terbaca.
pub fn parse_semver(tag: &str) -> Option<(u64, u64, u64)> {
    let digits = tag.trim().trim_start_matches(['v', 'V']);
    let mut it = digits.split('.');
    let major = it.next()?.parse().ok()?;
    let minor = it.next()?.parse().ok()?;
    let patch = it
        .next()?
        .split(|c: char| !c.is_ascii_digit())
        .next()?
        .parse()
        .ok()?;
    Some((major, minor, patch))
}

/// `true` bila `tag` lebih baru dari versi binary saat ini.
pub fn lebih_baru(tag: &str) -> bool {
    match (parse_semver(tag), parse_semver(env!("CARGO_PKG_VERSION"))) {
        (Some(a), Some(b)) => a > b,
        // Tag tak terbaca: jangan ganggu user dengan toast palsu.
        _ => false,
    }
}

/// Toast pembaruan; klik membuka target sesuai sumber (folder share di
/// Explorer, atau halaman rilis GitHub di browser).
fn toast_rilis(release: &Release, sumber: Sumber) {
    let lang = crate::state::settings().language;
    let body = crate::i18n::tr(lang, "update_body").replace("{versi}", &release.tag);
    let target = match sumber {
        Sumber::GitHub => crate::notify::TargetBuka::Browser(release.url.clone()),
        Sumber::Share => crate::notify::TargetBuka::Folder(release.url.clone()),
    };
    crate::notify::show_clickable(crate::i18n::tr(lang, "update_title"), &body, target);
    crate::logger::info(&format!(
        "versi baru tersedia: {} (via {sumber:?})",
        release.tag
    ));
}

/// Minta thread auto-check memeriksa SEKARANG — dipanggil saat sakelar
/// diatur ke Otomatis, agar user tidak menunggu siklus.
pub fn picu_cek_otomatis() {
    PICU_SEKARANG.store(true, Ordering::Relaxed);
}

/// Loop pemeriksaan otomatis — panggil sekali dari `main`. Tick tiap detik
/// (murah): cek hanya berjalan saat mode Otomatis DAN jadwal (1 jam) lewat,
/// atau saat dipicu [`picu_cek_otomatis`].
pub fn start() {
    std::thread::Builder::new()
        .name("update-check".into())
        .spawn(|| {
            let mut berikut = std::time::Instant::now(); // cek pertama: langsung
            loop {
                let enabled = crate::state::settings().update_check;
                let sekarang = std::time::Instant::now();
                let dipicu = PICU_SEKARANG.swap(false, Ordering::Relaxed);
                if enabled && (sekarang >= berikut || dipicu) {
                    if dipicu {
                        // Pemicu sakelar (Otomatis baru dinyalakan): hasil
                        // selalu di-toast — termasuk "sudah terbaru" — sebagai
                        // umpan balik bahwa mode otomatis bekerja.
                        let hasil = cek_sekarang();
                        toast_hasil_cek(&hasil);
                    } else if let Some((release, sumber)) =
                        latest_overall().filter(|(r, _)| lebih_baru(&r.tag))
                    {
                        // Cek terjadwal: hanya bersuara bila ada versi baru
                        // yang belum pernah diberitahukan.
                        let mut state = load_state();
                        if state.notified_tag.as_deref() != Some(release.tag.as_str()) {
                            toast_rilis(&release, sumber);
                            state.notified_tag = Some(release.tag.clone());
                            save_state(&state);
                        }
                    }
                    berikut = sekarang + INTERVAL;
                }
                if PICU_SEKARANG.swap(false, Ordering::Relaxed) {
                    berikut = std::time::Instant::now();
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        })
        .expect("gagal spawn thread update-check");
}

/// Hasil cek manual — toast-nya ditangani [`toast_hasil_cek`].
#[derive(Debug, Clone)]
pub enum HasilCek {
    /// Ada versi lebih baru (dari sumber mana pun).
    Tersedia(Release, Sumber),
    /// Versi terpasang sudah yang tertinggi.
    Terbaru,
    /// Kedua sumber tak terjangkau.
    Gagal,
}

/// Jalankan cek dua sumber (blocking: HTTP + UNC — panggil dari thread
/// sendiri, bukan UI).
pub fn cek_sekarang() -> HasilCek {
    match latest_overall() {
        Some((release, sumber)) if lebih_baru(&release.tag) => HasilCek::Tersedia(release, sumber),
        Some(_) => HasilCek::Terbaru,
        None => HasilCek::Gagal,
    }
}

/// Toast hasil cek manual sesuai jenisnya.
pub fn toast_hasil_cek(hasil: &HasilCek) {
    let lang = crate::state::settings().language;
    match hasil {
        HasilCek::Tersedia(release, sumber) => toast_rilis(release, *sumber),
        HasilCek::Terbaru => {
            crate::notify::show(
                crate::i18n::tr(lang, "update_latest_title"),
                &crate::i18n::tr(lang, "update_latest_body")
                    .replace("{versi}", env!("CARGO_PKG_VERSION")),
            );
        }
        HasilCek::Gagal => {
            crate::notify::show(
                crate::i18n::tr(lang, "update_failed_title"),
                crate::i18n::tr(lang, "update_failed_body"),
            );
        }
    }
}

/// Status cek manual — dipoll UI (150ms) untuk animasi loading tombol.
static MANUAL_BERJALAN: AtomicBool = AtomicBool::new(false);

/// `true` selama cek manual berjalan (dipakai spinner tombol pengaturan).
pub fn cek_manual_berjalan() -> bool {
    MANUAL_BERJALAN.load(Ordering::Relaxed)
}

/// Cek manual dari UI: toast hasil. Re-entry diabaikan (swap guard),
/// statusnya dipoll lewat [`cek_manual_berjalan`].
pub fn check_now_manual() {
    if MANUAL_BERJALAN.swap(true, Ordering::Relaxed) {
        return;
    }
    std::thread::Builder::new()
        .name("update-check-manual".into())
        .spawn(move || {
            let hasil = cek_sekarang();
            toast_hasil_cek(&hasil);
            MANUAL_BERJALAN.store(false, Ordering::Relaxed);
        })
        .expect("gagal spawn cek manual");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn semver_dengan_dan_tanpa_v() {
        assert_eq!(parse_semver("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_semver("1.2.3"), Some((1, 2, 3)));
        assert_eq!(parse_semver("V0.1.0"), Some((0, 1, 0)));
        // Sufiks build (umum di tag GitHub) tetap terbaca bagian numeriknya.
        assert_eq!(parse_semver("2.0.0-rc.1"), Some((2, 0, 0)));
        assert_eq!(parse_semver("abc"), None);
        assert_eq!(parse_semver("1.2"), None);
    }

    #[test]
    fn bandingkan_dengan_versi_binary() {
        let (a, b, c) = parse_semver(env!("CARGO_PKG_VERSION")).unwrap();
        // Dirinya sendiri & versi lama tidak dianggap baru.
        assert!(!lebih_baru(&format!("v{a}.{b}.{c}")));
        assert!(!lebih_baru(&format!("v{a}.{b}.{}", c.saturating_sub(1))));
        // Naikkan patch/minor/major harus dianggap lebih baru.
        assert!(lebih_baru(&format!("v{a}.{b}.{}", c + 1)));
        assert!(lebih_baru(&format!("v{a}.{}.0", b + 1)));
        assert!(lebih_baru(&format!("v{}.0.0", a + 1)));
        // Tag aneh: bukan lebih baru (jangan toast palsu).
        assert!(!lebih_baru("tidak-valid"));
    }

    #[test]
    fn share_scan_ambil_tertinggi_validasi_installer() {
        let root = std::env::temp_dir().join(format!("jadwal-share-uji-{}", std::process::id()));
        let buat = |nama: &str, installer: Option<&str>| {
            let d = root.join(nama);
            std::fs::create_dir_all(&d).expect("buat folder uji");
            if let Some(f) = installer {
                std::fs::write(d.join(f), b"x").expect("tulis installer uji");
            }
        };
        buat("v1.1.0", Some("Jadwal Sholat 1.1.0 x64-setup.exe"));
        buat("v1.2.0", Some("Jadwal Sholat 1.2.0 x64-setup.exe"));
        buat("v9.9.9", None); // versi tertinggi tapi installer hilang
        buat("catatan", None); // bukan folder versi

        let hasil = latest_from_share_dir(&root).expect("v1.2.0 harus terpilih");
        assert_eq!(hasil.tag, "v1.2.0");
        assert!(hasil.url.contains("v1.2.0"), "url: {}", hasil.url);

        // Semua folder versi tanpa installer → bukan rilis.
        std::fs::remove_dir_all(root.join("v1.1.0")).unwrap();
        std::fs::remove_dir_all(root.join("v1.2.0")).unwrap();
        assert_eq!(latest_from_share_dir(&root), None);

        std::fs::remove_dir_all(&root).ok();
    }
}
