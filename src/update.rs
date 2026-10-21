//! Pemeriksaan pembaruan via GitHub Releases.
//!
//! Thread latar (`start`) mengecek rilis terbaru saat startup lalu tiap 12
//! jam; bila versinya lebih baru dari binary, toast tampil SEKALI per versi
//! (memo di `update-check.json` di folder config) dan klik toast membuka
//! halaman rilis di browser. [`check_now_manual`] dipakai tombol "Cek
//! sekarang" di pengaturan: hasilnya selalu di-toast, tanpa memo.

use serde::{Deserialize, Serialize};

/// Repo GitHub tempat rilis dipublikasikan (lihat `just release-all`).
const REPO: &str = "mukhsin/rust-adzan";

/// Jeda antar-pemeriksaan otomatis.
const INTERVAL: std::time::Duration = std::time::Duration::from_secs(12 * 3600);

const USER_AGENT: &str = concat!(
    "jadwal-sholat/",
    env!("CARGO_PKG_VERSION"),
    " (github.com/mukhsin/rust-adzan)"
);

/// Rilis yang relevan dari API GitHub.
#[derive(Debug, Clone, PartialEq)]
pub struct Release {
    /// Tag rilis, mis. "v1.1.0" (bisa juga "1.1.0" — dinormalisasi saat
    /// dibandingkan).
    pub tag: String,
    /// URL halaman rilis untuk dibuka di browser.
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

/// Toast pembaruan; klik toast membuka `url` di browser bawaan.
fn toast_rilis(release: &Release) {
    let lang = crate::state::settings().language;
    let body = crate::i18n::tr(lang, "update_body").replace("{versi}", &release.tag);
    crate::notify::show_clickable(
        crate::i18n::tr(lang, "update_title"),
        &body,
        release.url.clone(),
    );
    crate::logger::info(&format!("versi baru tersedia: {}", release.tag));
}

/// Loop pemeriksaan otomatis — panggil sekali dari `main`.
pub fn start() {
    std::thread::Builder::new()
        .name("update-check".into())
        .spawn(|| {
            loop {
                let enabled = crate::state::settings().update_check;
                if enabled {
                    if let Some(release) = latest_release() {
                        if lebih_baru(&release.tag) {
                            let mut state = load_state();
                            if state.notified_tag.as_deref() != Some(release.tag.as_str()) {
                                toast_rilis(&release);
                                state.notified_tag = Some(release.tag.clone());
                                save_state(&state);
                            }
                        }
                    }
                }
                std::thread::sleep(INTERVAL);
            }
        })
        .expect("gagal spawn thread update-check");
}

/// Cek atas permintaan user (tombol pengaturan): jalankan di thread sendiri
/// agar UI tidak nunggu HTTP, dan toast hasil apa pun adanya — "sudah
/// terbaru" maupun "tersedia" — tanpa memo sekali-per-versi.
pub fn check_now_manual() {
    std::thread::Builder::new()
        .name("update-check-manual".into())
        .spawn(|| {
            let lang = crate::state::settings().language;
            match latest_release() {
                Some(release) if lebih_baru(&release.tag) => toast_rilis(&release),
                Some(_) => {
                    crate::notify::show(
                        crate::i18n::tr(lang, "update_latest_title"),
                        &crate::i18n::tr(lang, "update_latest_body")
                            .replace("{versi}", env!("CARGO_PKG_VERSION")),
                    );
                }
                None => {
                    crate::notify::show(
                        crate::i18n::tr(lang, "update_failed_title"),
                        crate::i18n::tr(lang, "update_failed_body"),
                    );
                }
            }
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
}
