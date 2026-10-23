//! Data Al-Qur'an offline — teks Arab mushaf Kemenag + terjemahan resmi
//! Kementerian Agama RI, ter-embed di binary (`assets/quran.json`, dibangun
//! oleh `tools/quran-build.mjs`; hasilnya di-commit). Parse JSON sekali
//! secara lazy (`OnceLock`) dengan zero-copy borrow; setelah itu semua
//! layar hanya meminjam referensi.

use serde::Deserialize;
use std::sync::OnceLock;

const DATA_JSON: &str = include_str!("../assets/quran.json");

/// Satu surah lengkap dengan seluruh ayatnya.
#[derive(Deserialize, Debug)]
pub struct Surah<'a> {
    pub nomor: u8,
    /// Nama Latin gaya Indonesia (Kemenag), mis. "Al-Fatihah".
    pub nama: &'a str,
    /// Nama Arab (tulisan mushaf).
    pub arab: &'a str,
    /// `true` = Makkiyah (diturunkan di Mekah), `false` = Madaniyah.
    pub mekah: bool,
    /// Pasangan (teks Arab, terjemahan); nomor ayat = indeks + 1.
    pub ayat: Vec<(&'a str, &'a str)>,
}

#[derive(Deserialize)]
struct DataQuran<'a> {
    lisensi: &'a str,
    #[serde(borrow)]
    surah: Vec<Surah<'a>>,
}

fn data() -> &'static DataQuran<'static> {
    static DATA: OnceLock<DataQuran<'static>> = OnceLock::new();
    DATA.get_or_init(|| {
        serde_json::from_str(DATA_JSON).expect("assets/quran.json harus JSON valid")
    })
}

/// Seluruh 114 surah, terurut nomor mushaf.
pub fn surah_list() -> &'static [Surah<'static>] {
    &data().surah
}

/// Satu surah berdasar nomor mushaf (1–114).
pub fn baca(nomor: u8) -> Option<&'static Surah<'static>> {
    let nomor = nomor as usize;
    if (1..=114).contains(&nomor) {
        Some(&surah_list()[nomor - 1])
    } else {
        None
    }
}

/// Total ayat seluruh Al-Qur'an (6236).
#[allow(dead_code)]
pub fn total_ayat() -> usize {
    surah_list().iter().map(|s| s.ayat.len()).sum()
}

/// Atribusi sumber & lisensi data (dipakai bila Tentang menampilkan
/// atribusi lengkap; untuk kini Tentang memakai ringkasannya).
#[allow(dead_code)]
pub fn lisensi() -> &'static str {
    data().lisensi
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn struktur_lengkap() {
        assert_eq!(surah_list().len(), 114);
        assert_eq!(total_ayat(), 6236);
        assert_eq!(surah_list()[0].nama, "Al-Fatihah");
        assert!(surah_list()[0].mekah);
        assert_eq!(surah_list()[113].nama, "An-Nas");
        // Al-Baqarah surah terpanjang.
        assert_eq!(baca(2).unwrap().ayat.len(), 286);
        // Ayat Kursi (Al-Baqarah 255) berisi teks & terjemahan.
        let (arab, terjemahan) = baca(2).unwrap().ayat[254];
        assert!(!arab.is_empty() && !terjemahan.is_empty());
    }

    #[test]
    fn at_taubah_tanpa_basmalah() {
        // At-Taubah (9) satu-satunya surah tanpa basmalah pembuka.
        let (arab, _) = baca(9).unwrap().ayat[0];
        assert!(
            !arab.starts_with("بِسْمِ"),
            "At-Taubah terbuka basmalah: {arab}"
        );
    }

    #[test]
    fn semua_teks_terisi() {
        for s in surah_list() {
            assert!(
                !s.nama.is_empty() && !s.arab.is_empty(),
                "surah {}",
                s.nomor
            );
            assert_eq!(s.ayat.is_empty(), false, "surah {} tanpa ayat", s.nomor);
            for (i, (arab, terjemahan)) in s.ayat.iter().enumerate() {
                assert!(
                    !arab.is_empty(),
                    "surah {} ayat {} arab kosong",
                    s.nomor,
                    i + 1
                );
                assert!(
                    !terjemahan.is_empty(),
                    "surah {} ayat {} terjemahan kosong",
                    s.nomor,
                    i + 1
                );
            }
        }
    }

    #[test]
    fn baca_di_luar_jangkauan() {
        assert!(baca(0).is_none());
        assert!(baca(115).is_none());
        assert!(baca(1).is_some());
        assert!(baca(114).is_some());
    }

    #[test]
    fn lisensi_tercantum() {
        assert!(lisensi().contains("Kementerian Agama"));
    }
}
