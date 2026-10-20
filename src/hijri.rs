//! Konversi tanggal Gregorian → Hijri (kalender Umm al-Qura).
//!
//! Memakai `icu_calendar` (ICU4X): panjang bulan resmi KACST untuk
//! 1300–1600 H (1882–2174 M), fallback tabular di luar rentang itu.
//! Tetap bisa selisih ±1 hari dari rukyat lokal — sifat bawaan semua
//! kalender hijri prediktif.

use crate::i18n::hijri_month_name;
use crate::settings::Language;
use chrono::{Datelike, NaiveDate};
use icu_calendar::Date;
use icu_calendar::cal::Hijri;
use icu_calendar::cal::hijri::UmmAlQura;

/// (tahun, bulan 1-12, hari) Hijri untuk tanggal Gregorian.
pub fn to_hijri(date: NaiveDate) -> (i32, u32, u32) {
    let iso = Date::try_new_iso(date.year(), date.month() as u8, date.day() as u8)
        .expect("tanggal dari chrono selalu valid");
    let hijri = iso.to_calendar(Hijri(UmmAlQura::default()));
    (
        hijri.era_year().year,
        hijri.month().ordinal as u32,
        hijri.day_of_month().0 as u32,
    )
}

/// Tanggal hijri terformat, mis. "10 Jumadil Awal 1448 H" (Id) /
/// "10 Jumada al-Ula 1448 AH" (En).
pub fn format(lang: Language, date: NaiveDate) -> String {
    let (year, month, day) = to_hijri(date);
    let era = match lang {
        Language::Id => "H",
        Language::En => "AH",
    };
    format!(
        "{} {} {} {}",
        day,
        hijri_month_name(lang, (month - 1) as usize),
        year,
        era
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn konversi_tanggal_diketahui() {
        use chrono::NaiveDate;
        // 1 Muharam 1446 (KACST/Umm al-Qura; Indonesia memulai 8 Juli).
        assert_eq!(
            to_hijri(NaiveDate::from_ymd_opt(2024, 7, 7).unwrap()),
            (1446, 1, 1)
        );
        // Hari ini saat fitur dibuat.
        assert_eq!(
            to_hijri(NaiveDate::from_ymd_opt(2026, 10, 21).unwrap()),
            (1448, 5, 10)
        );
    }

    #[test]
    fn format_dua_bahasa() {
        let d = chrono::NaiveDate::from_ymd_opt(2026, 10, 21).unwrap();
        assert_eq!(format(Language::Id, d), "10 Jumadil Awal 1448 H");
        assert_eq!(format(Language::En, d), "10 Jumada al-Ula 1448 AH");
    }
}
