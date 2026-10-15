use chrono_tz::Tz;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct City {
    pub name: &'static str,
    pub lat: f64,
    pub lng: f64,
    pub timezone: Tz,
}

/// Daftar kota besar Indonesia beserta koordinat dan zona waktunya.
/// Koordinat pusat kota — cukup akurat untuk jadwal sholat (selisih < 1 menit).
pub const CITIES: &[City] = &[
    City {
        name: "Aceh Barat",
        lat: 4.4486,
        lng: 96.1669,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Balikpapan",
        lat: -1.2379,
        lng: 116.8529,
        timezone: chrono_tz::Asia::Makassar,
    },
    City {
        name: "Bandar Lampung",
        lat: -5.4292,
        lng: 105.2611,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Bandung",
        lat: -6.9175,
        lng: 107.6191,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Banjarmasin",
        lat: -3.3186,
        lng: 114.5944,
        timezone: chrono_tz::Asia::Makassar,
    },
    City {
        name: "Banda Aceh",
        lat: 5.5483,
        lng: 95.3238,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Batam",
        lat: 1.1301,
        lng: 104.0532,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Bekasi",
        lat: -6.2383,
        lng: 106.9756,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Bengkulu",
        lat: -3.8004,
        lng: 102.2655,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Bima",
        lat: -8.4600,
        lng: 118.7300,
        timezone: chrono_tz::Asia::Makassar,
    },
    City {
        name: "Bogor",
        lat: -6.5950,
        lng: 106.8166,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Cirebon",
        lat: -6.7320,
        lng: 108.5523,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Denpasar",
        lat: -8.6705,
        lng: 115.2126,
        timezone: chrono_tz::Asia::Makassar,
    },
    City {
        name: "Depok",
        lat: -6.4025,
        lng: 106.7942,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Gorontalo",
        lat: 0.5435,
        lng: 123.0568,
        timezone: chrono_tz::Asia::Makassar,
    },
    City {
        name: "Jayapura",
        lat: -2.5916,
        lng: 140.6690,
        timezone: chrono_tz::Asia::Jayapura,
    },
    City {
        name: "Jambi",
        lat: -1.6101,
        lng: 103.6131,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Jakarta",
        lat: -6.2088,
        lng: 106.8456,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Kendari",
        lat: -3.9985,
        lng: 122.5131,
        timezone: chrono_tz::Asia::Makassar,
    },
    City {
        name: "Kupang",
        lat: -10.1772,
        lng: 123.6070,
        timezone: chrono_tz::Asia::Makassar,
    },
    City {
        name: "Madiun",
        lat: -7.6298,
        lng: 111.5236,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Magelang",
        lat: -7.4797,
        lng: 110.2177,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Malang",
        lat: -7.9666,
        lng: 112.6326,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Makassar",
        lat: -5.1477,
        lng: 119.4327,
        timezone: chrono_tz::Asia::Makassar,
    },
    City {
        name: "Manado",
        lat: 1.4748,
        lng: 124.8421,
        timezone: chrono_tz::Asia::Makassar,
    },
    City {
        name: "Mataram",
        lat: -8.5833,
        lng: 116.1167,
        timezone: chrono_tz::Asia::Makassar,
    },
    City {
        name: "Medan",
        lat: 3.5952,
        lng: 98.6722,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Padang",
        lat: -0.9471,
        lng: 100.4172,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Palangkaraya",
        lat: -2.2088,
        lng: 113.9213,
        timezone: chrono_tz::Asia::Makassar,
    },
    City {
        name: "Palembang",
        lat: -2.9761,
        lng: 104.7754,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Palu",
        lat: -0.8917,
        lng: 119.8707,
        timezone: chrono_tz::Asia::Makassar,
    },
    City {
        name: "Pangkalpinang",
        lat: -2.1291,
        lng: 106.1133,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Pekanbaru",
        lat: 0.5071,
        lng: 101.4478,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Pontianak",
        lat: -0.0263,
        lng: 109.3425,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Purwokerto",
        lat: -7.4255,
        lng: 109.2342,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Samarinda",
        lat: -0.5017,
        lng: 117.1536,
        timezone: chrono_tz::Asia::Makassar,
    },
    City {
        name: "Semarang",
        lat: -6.9667,
        lng: 110.4167,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Serang",
        lat: -6.1203,
        lng: 106.1506,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Sidoarjo",
        lat: -7.4478,
        lng: 112.7183,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Solo",
        lat: -7.5755,
        lng: 110.8243,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Surabaya",
        lat: -7.2575,
        lng: 112.7521,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Sukabumi",
        lat: -6.9266,
        lng: 106.9308,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Tasikmalaya",
        lat: -7.3274,
        lng: 108.2207,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Tegal",
        lat: -6.8797,
        lng: 109.1256,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Ternate",
        lat: 0.7906,
        lng: 127.3842,
        timezone: chrono_tz::Asia::Makassar,
    },
    City {
        name: "Tangerang",
        lat: -6.1783,
        lng: 106.6319,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Tangerang Selatan",
        lat: -6.2930,
        lng: 106.7186,
        timezone: chrono_tz::Asia::Jakarta,
    },
    City {
        name: "Yogyakarta",
        lat: -7.7956,
        lng: 110.3695,
        timezone: chrono_tz::Asia::Jakarta,
    },
];

pub fn find_city(name: &str) -> Option<&'static City> {
    CITIES.iter().find(|c| c.name.eq_ignore_ascii_case(name))
}
