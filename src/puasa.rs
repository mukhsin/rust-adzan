//! Puasa sunnah — jenis puasa yang diikutkan user dan deteksi puasa yang
//! jatuh pada suatu tanggal. Tanggal hijriah memakai konversi Umm al-Qura
//! (`hijri::to_hijri`), jadi selisih ±1 hari dari rukyat lokal berlaku di
//! sini juga.
//!
//! Saat Ramadhan seluruh puasa sunnah sengaja dikosongkan: puasa wajib
//! sudah terwakili pil Ramadhan, dan qada lebih utama daripada sunnah.
//! Dua hari raya (1 Syawal, 10 Zulhijjah) juga kosong — puasa haram di
//! hari itu, sekalipun jatuh Senin/Kamis.
//!
//! Pil UI & toast pengingat memakai daftar kunci i18n hasil
//! [`daftar_hari_ini`] — urutannya kanonik (Senin → Kamis → Ayyamul Bidh →
//! Syawal → hari penting), tidak mengikuti urutan Vec settings.

use crate::hijri::to_hijri;
use crate::i18n::tr;
use crate::settings::Language;
use chrono::{Datelike, NaiveDate};

/// Jenis puasa sunnah yang bisa diikutkan (settings → pil & pengingat).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PuasaSunnahKind {
    /// Tiap hari Senin.
    Senin,
    /// Tiap hari Kamis.
    Kamis,
    /// Ayyamul Bidh: tanggal 13–15 tiap bulan hijriah.
    AyyamulBidh,
    /// Enam hari di bulan Syawal (2–7 Syawal, selepas Idul Fitri).
    Syawal,
    /// Puasa satu hari pada tanggal penting: Tarwiyah (8 Zulhijjah),
    /// Arafah (9 Zulhijjah), Tasu'a (9 Muharam), Asyura (10 Muharam).
    HariPenting,
}

/// Seluruh jenis, urut kanonik — sumber default settings & urutan pil.
pub const SEMUA: [PuasaSunnahKind; 5] = [
    PuasaSunnahKind::Senin,
    PuasaSunnahKind::Kamis,
    PuasaSunnahKind::AyyamulBidh,
    PuasaSunnahKind::Syawal,
    PuasaSunnahKind::HariPenting,
];

/// Puasa sunnah yang jatuh pada `today` (kunci i18n namanya), dari jenis
/// yang diaktifkan. Kosong saat Ramadhan — lihat doc modul.
pub fn daftar_hari_ini(today: NaiveDate, aktif: &[PuasaSunnahKind]) -> Vec<&'static str> {
    let mut daftar = Vec::new();
    let (bulan_hijri, tanggal_hijri) = {
        let (_, bulan, tanggal) = to_hijri(today);
        (bulan, tanggal)
    };
    if bulan_hijri == 9 {
        return daftar; // Ramadhan: puasa wajib, bukan urusan sunnah.
    }
    if (bulan_hijri, tanggal_hijri) == (10, 1) || (bulan_hijri, tanggal_hijri) == (12, 10) {
        return daftar; // Hari raya (Idul Fitri / Idul Adha): puasa haram.
    }

    let mut tambah = |kunci: &'static str| daftar.push(kunci);
    if aktif.contains(&PuasaSunnahKind::Senin) && today.weekday() == chrono::Weekday::Mon {
        tambah("puasa_senin");
    }
    if aktif.contains(&PuasaSunnahKind::Kamis) && today.weekday() == chrono::Weekday::Thu {
        tambah("puasa_kamis");
    }
    if aktif.contains(&PuasaSunnahKind::AyyamulBidh) && (13..=15).contains(&tanggal_hijri) {
        tambah("puasa_ayyamul_bidh");
    }
    if aktif.contains(&PuasaSunnahKind::Syawal)
        && bulan_hijri == 10
        && (2..=7).contains(&tanggal_hijri)
    {
        tambah("puasa_syawal");
    }
    if aktif.contains(&PuasaSunnahKind::HariPenting) {
        let kunci = match (bulan_hijri, tanggal_hijri) {
            (12, 8) => Some("puasa_tarwiyah"),
            (12, 9) => Some("puasa_arafah"),
            (1, 9) => Some("puasa_tasua"),
            (1, 10) => Some("puasa_asyura"),
            _ => None,
        };
        if let Some(kunci) = kunci {
            tambah(kunci);
        }
    }
    daftar
}

/// Gabungan nama puasa hari ini untuk pil UI ("Puasa Senin · Puasa
/// Ayyamul Bidh"); `None` bila hari ini tidak ada puasa sunnah yang
/// diikutkan. Pil menambahkan prefiks 🌙 sendiri (pola pil Ramadhan).
pub fn label_gabung(lang: Language, today: NaiveDate, aktif: &[PuasaSunnahKind]) -> Option<String> {
    let daftar = daftar_hari_ini(today, aktif);
    if daftar.is_empty() {
        return None;
    }
    Some(
        daftar
            .iter()
            .map(|kunci| tr(lang, kunci))
            .collect::<Vec<_>>()
            .join(" · "),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn semua() -> Vec<PuasaSunnahKind> {
        SEMUA.to_vec()
    }

    /// Tanggal masehi pertama >= `dasar` yang jatuh pada (bulan, tanggal)
    /// hijriah — agar test bebas hardcode tanggal masehi.
    fn cari_hijri(dasar: NaiveDate, bulan: u32, tanggal: u32) -> NaiveDate {
        (0..400)
            .map(|i| dasar + Duration::days(i))
            .find(|&t| {
                let (_, m, d) = to_hijri(t);
                m == bulan && d == tanggal
            })
            .expect("tanggal hijriah pasti datang dalam setahun")
    }

    #[test]
    fn senin_kamis_ikut_hari_masehi() {
        // 2026-10-26 = Senin (15 Jumadil Awal → Ayyamul Bidh juga),
        // 2026-10-22 = Kamis (11 Jumadil Awal), 2026-10-21 = Rabu.
        let senin = NaiveDate::from_ymd_opt(2026, 10, 26).unwrap();
        let daftar = daftar_hari_ini(senin, &semua());
        assert!(daftar.contains(&"puasa_senin"), "{daftar:?}");
        assert!(daftar.contains(&"puasa_ayyamul_bidh"), "{daftar:?}");
        assert_eq!(daftar.len(), 2);

        let kamis = NaiveDate::from_ymd_opt(2026, 10, 22).unwrap();
        assert_eq!(daftar_hari_ini(kamis, &semua()), vec!["puasa_kamis"]);

        let rabu = NaiveDate::from_ymd_opt(2026, 10, 21).unwrap();
        assert!(daftar_hari_ini(rabu, &semua()).is_empty());
    }

    #[test]
    fn ayyamul_bidh_tanggal_13_sampai_15() {
        let dasar = NaiveDate::from_ymd_opt(2026, 10, 21).unwrap(); // 10 Jumadil Awal
        let ke_12 = dasar + Duration::days(2);
        assert!(daftar_hari_ini(ke_12, &semua()).is_empty());
        for offset in 3..=5i64 {
            let hari = dasar + Duration::days(offset);
            assert!(
                daftar_hari_ini(hari, &semua()).contains(&"puasa_ayyamul_bidh"),
                "tanggal {} harus Ayyamul Bidh",
                to_hijri(hari).2
            );
        }
        let ke_16 = dasar + Duration::days(6);
        assert!(!daftar_hari_ini(ke_16, &semua()).contains(&"puasa_ayyamul_bidh"));
    }

    #[test]
    fn syawal_enam_hari_selepas_idul_fitri() {
        let dasar = NaiveDate::from_ymd_opt(2026, 10, 21).unwrap();
        let idul_fitri = cari_hijri(dasar, 10, 1);
        // 1 Syawal = Idul Fitri — puasa haram, walau kebetulan Senin.
        assert!(daftar_hari_ini(idul_fitri, &semua()).is_empty());
        let idul_adha = cari_hijri(dasar, 12, 10);
        assert!(daftar_hari_ini(idul_adha, &semua()).is_empty());
        for offset in 1..=6i64 {
            let hari = idul_fitri + Duration::days(offset);
            assert!(
                daftar_hari_ini(hari, &semua()).contains(&"puasa_syawal"),
                "{} Syawal harus puasa Syawal",
                to_hijri(hari).2
            );
        }
        // Hari ke-8 ke atas bukan lagi.
        let delapan = cari_hijri(dasar, 10, 8);
        assert!(
            !daftar_hari_ini(delapan, &semua()).contains(&"puasa_syawal"),
            "8 Syawal di luar rentang puasa Syawal"
        );
    }

    #[test]
    fn hari_penting_tarwiyah_arafah_tasua_asyura() {
        let dasar = NaiveDate::from_ymd_opt(2026, 10, 21).unwrap();
        // Pakai contains: tanggal penting bisa kebetulan jatuh Senin/Kamis
        // atau 13–15 hijriah, jadi puasa lain sah ikut masuk daftar.
        for (bulan, tanggal, kunci) in [
            (12u32, 8u32, "puasa_tarwiyah"),
            (12, 9, "puasa_arafah"),
            (1, 9, "puasa_tasua"),
            (1, 10, "puasa_asyura"),
        ] {
            let hari = cari_hijri(dasar, bulan, tanggal);
            assert!(
                daftar_hari_ini(hari, &semua()).contains(&kunci),
                "{} harus {}",
                to_hijri(hari).2,
                kunci
            );
        }
    }

    #[test]
    fn kosong_saat_ramadhan() {
        let dasar = NaiveDate::from_ymd_opt(2026, 10, 21).unwrap();
        let awal_ramadhan = cari_hijri(dasar, 9, 1);
        assert_eq!(to_hijri(awal_ramadhan).1, 9);
        // Hari mana pun di Ramadhan (termasuk tanggal 13–15), sunnah kosong.
        for offset in 0..10 {
            assert!(
                daftar_hari_ini(awal_ramadhan + Duration::days(offset), &semua()).is_empty(),
                "Ramadhan harus bebas puasa sunnah"
            );
        }
    }

    #[test]
    fn filter_jenis_yang_diaktifkan() {
        let senin = NaiveDate::from_ymd_opt(2026, 10, 26).unwrap(); // Senin + Ayyamul Bidh
        let hanya_senin = daftar_hari_ini(senin, &[PuasaSunnahKind::Senin]);
        assert_eq!(hanya_senin, vec!["puasa_senin"]);
        assert!(daftar_hari_ini(senin, &[]).is_empty());
    }

    #[test]
    fn label_gabung_terurut_kanonik_dan_none() {
        let lang = Language::Id;
        let senin = NaiveDate::from_ymd_opt(2026, 10, 26).unwrap();
        assert_eq!(
            label_gabung(lang, senin, &semua()).expect("Senin + Ayyamul Bidh"),
            "Puasa Senin · Puasa Ayyamul Bidh"
        );
        let rabu = NaiveDate::from_ymd_opt(2026, 10, 21).unwrap();
        assert_eq!(label_gabung(lang, rabu, &semua()), None);
    }
}
