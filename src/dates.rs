//! Tanggal penting kalender Islam (diagnostic Umm al-Qura, selisih ±1 hari
//! dari rukyat lokal — lihat `hijri`).
//!
//! [`upcoming`] memindai 360 hari ke depan (satu tahun hijriah penuh —
//! tiap tanggal hijriah pasti terlewati tepat sekali) dan hasilnya di-cache
//! per tanggal masehi, aman untuk layar yang render tiap detik.

use crate::hijri::to_hijri;
use chrono::{Duration, NaiveDate};
use std::sync::Mutex;

/// Peristiwa penting: (bulan hijriah, tanggal, jenis).
const PENTING: [(u32, u32, EventKind); 6] = [
    (1, 1, EventKind::TahunBaru),
    (3, 12, EventKind::Maulid),
    (7, 27, EventKind::IsraMiraj),
    (9, 1, EventKind::AwalRamadan),
    (10, 1, EventKind::IdulFitri),
    (12, 10, EventKind::IdulAdha),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKind {
    TahunBaru,
    Maulid,
    IsraMiraj,
    AwalRamadan,
    IdulFitri,
    IdulAdha,
}

impl EventKind {
    /// Kunci terjemahan nama peristiwa (lihat `i18n::PHRASES`).
    pub fn key(self) -> &'static str {
        match self {
            EventKind::TahunBaru => "ev_tahun_baru",
            EventKind::Maulid => "ev_maulid",
            EventKind::IsraMiraj => "ev_isra_miraj",
            EventKind::AwalRamadan => "ev_awal_ramadan",
            EventKind::IdulFitri => "ev_idul_fitri",
            EventKind::IdulAdha => "ev_idul_adha",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Event {
    pub date: NaiveDate,
    pub kind: EventKind,
}

/// Status Ramadan hari ini.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RamadanStatus {
    /// Ramadan berjalan: hari ke-N.
    Berlangsung { day: u32 },
    /// Menunggu Ramadan: sisa hari & tahun hijriah Ramadan yang datang.
    Menunggu { days: i64, year: i32 },
}

/// Peristiwa penting berikutnya setelah `today` (inklusif — hari ini ikut
/// dihitung), urut tanggal, maksimal `count`. Cache per tanggal masehi.
pub fn upcoming(today: NaiveDate, count: usize) -> Vec<Event> {
    let mut cache = CACHE.lock().expect("cache tanggal penting");
    if cache.as_ref().is_none_or(|(hari, _)| *hari != today) {
        *cache = Some((today, pindai(today)));
    }
    let (_, events) = cache.as_ref().expect("cache terisi");
    events.iter().take(count).copied().collect()
}

/// Status Ramadan untuk `today`.
pub fn ramadan_status(today: NaiveDate) -> RamadanStatus {
    let (_, month, day) = to_hijri(today);
    if month == 9 {
        return RamadanStatus::Berlangsung { day };
    }
    let awal = upcoming(today, PENTING.len())
        .into_iter()
        .find(|e| e.kind == EventKind::AwalRamadan)
        .expect("1 Ramadhan selalu ada dalam pindahan 360 hari");
    RamadanStatus::Menunggu {
        days: (awal.date - today).num_days(),
        // Tahun Ramadan yang datang bisa sama atau +1 dari tahun hari ini —
        // ambil langsung dari tanggalnya.
        year: to_hijri(awal.date).0,
    }
}

/// Hari ke-N Ramadhan bila `today` jatuh saat puasa, `None` selain itu —
/// ringkasan [`ramadan_status`] untuk UI.
pub fn ramadan_day(today: NaiveDate) -> Option<u32> {
    match ramadan_status(today) {
        RamadanStatus::Berlangsung { day } => Some(day),
        RamadanStatus::Menunggu { .. } => None,
    }
}

/// Tanggal penting bila (bulan, tanggal) hijriah adalah hari peristiwa —
/// dipakai penanda layar Kalender.
pub fn by_hijri(month: u32, day: u32) -> Option<EventKind> {
    PENTING
        .iter()
        .find(|(m, d, _)| *m == month && *d == day)
        .map(|(_, _, kind)| *kind)
}

static CACHE: Mutex<Option<(NaiveDate, Vec<Event>)>> = Mutex::new(None);

/// Pindai `today + 0..360` hari, kumpulkan seluruh peristiwa yang jatuh
/// pada rentang itu, urutkan. Tanggal hijriah berulang tiap ±354 hari,
/// jadi tiap peristiwa muncul tepat sekali.
fn pindai(today: NaiveDate) -> Vec<Event> {
    let mut events = Vec::new();
    for i in 0..360 {
        let date = today + Duration::days(i);
        let (_, month, day) = to_hijri(date);
        if let Some(kind) = by_hijri(month, day) {
            events.push(Event { date, kind });
        }
    }
    events.sort_by_key(|e| e.date);
    events
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn upcoming_mengandung_ramadan_dan_fitri() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 21).unwrap();
        let events = upcoming(today, 6);
        assert!(events.len() >= 2, "{events:?}");
        // Urut naik & semuanya >= today.
        for w in events.windows(2) {
            assert!(w[0].date <= w[1].date);
            assert!(w[0].date >= today);
        }
        // Dalam setahun ke depan pasti ada 1 Ramadhan & 1 Syawal.
        assert!(events.iter().any(|e| e.kind == EventKind::AwalRamadan));
        assert!(events.iter().any(|e| e.kind == EventKind::IdulFitri));
    }

    #[test]
    fn ramadan_status_2026_menunggu() {
        // Okt 2026 = 10 Jumadil Awal 1448 H → Ramadan berikutnya tetap di
        // tahun 1448 H (bulan ke-9 tahun yang sama, ± Februari 2027).
        let today = NaiveDate::from_ymd_opt(2026, 10, 21).unwrap();
        match ramadan_status(today) {
            RamadanStatus::Menunggu { days, year } => {
                assert!(days > 30 && days <= 354, "days: {days}");
                assert_eq!(year, 1448);
            }
            other => panic!("seharusnya Menunggu, dapat: {other:?}"),
        }
    }

    #[test]
    fn by_hijri_cocok_tabel() {
        assert_eq!(by_hijri(9, 1), Some(EventKind::AwalRamadan));
        assert_eq!(by_hijri(10, 1), Some(EventKind::IdulFitri));
        assert_eq!(by_hijri(9, 2), None);
        assert_eq!(by_hijri(5, 10), None);
    }

    #[test]
    fn ramadan_day_mengikuti_bulan_hijri() {
        let dasar = NaiveDate::from_ymd_opt(2026, 10, 21).unwrap();
        // Okt 2026 bukan Ramadan.
        assert_eq!(ramadan_day(dasar), None);
        // Cari 1 Ramadhan berikutnya dari kalender itu sendiri (tanpa
        // hardcode tanggal masehi), lalu cek hitungan hari-N.
        let awal = (0..400)
            .map(|i| dasar + Duration::days(i))
            .find(|&t| to_hijri(t).1 == 9)
            .expect("Ramadhan pasti datang dalam setahun");
        assert_eq!(to_hijri(awal).2, 1);
        assert_eq!(ramadan_day(awal), Some(1));
        assert_eq!(ramadan_day(awal + Duration::days(14)), Some(15));
        assert_eq!(ramadan_day(awal - Duration::days(1)), None);
    }
}
