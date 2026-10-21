//! Arah kiblat — bearing awal (great-circle) dari lokasi ke Ka'bah,
//! dihitung murni dari koordinat tanpa data tambahan.

/// Koordinat Ka'bah, Makkah.
const KAABA_LAT: f64 = 21.422_487;
const KAABA_LNG: f64 = 39.826_206;

/// Bearing qiblat dari utara sejati, searah jarum jam, 0.0..360.0 derajat.
pub fn bearing(lat: f64, lng: f64) -> f64 {
    let phi1 = lat.to_radians();
    let phi2 = KAABA_LAT.to_radians();
    let delta_lng = (KAABA_LNG - lng).to_radians();
    let y = delta_lng.sin();
    let x = phi1.cos() * phi2.tan() - phi1.sin() * phi2.cos() * delta_lng.cos();
    let deg = y.atan2(x).to_degrees();
    (deg + 360.0) % 360.0
}

/// Kunci terjemahan arah mata angin 8 penjuru untuk sebuah bearing
/// (sektor 45°, utara = 0).
pub fn arah_key(deg: f64) -> &'static str {
    const KEYS: [&str; 8] = [
        "dir_n", "dir_ne", "dir_e", "dir_se", "dir_s", "dir_sw", "dir_w", "dir_nw",
    ];
    let sektor = ((deg / 45.0).round() as usize) % 8;
    KEYS[sektor]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bearing_jakarta_barat_laut() {
        // Referensi umum: qiblat Jakarta ≈ 295° (barat laut).
        let deg = bearing(-6.2, 106.816_666);
        assert!((294.0..=296.0).contains(&deg), "Jakarta: {deg}");
        assert_eq!(arah_key(deg), "dir_nw");
    }

    #[test]
    fn bearing_new_york_timur_laut() {
        // Referensi umum: qiblat New York ≈ 58° (timur laut, lingkaran
        // besar lewat utara Atlantik).
        let deg = bearing(40.7128, -74.006);
        assert!((57.0..=60.0).contains(&deg), "New York: {deg}");
        assert_eq!(arah_key(deg), "dir_ne");
    }

    #[test]
    fn bearing_normalisasi_dan_sektor() {
        // Cermin kasus "barat Makkah": titik di TIMUR Makkah, lintang sama.
        // Great-circle menekuk ke kutub, bearing ke arah barat ~280°, dan
        // atan2 menghasilkan nilai negatif yang wajib dinormalisasi +360.
        let deg = bearing(21.4225, 50.0);
        assert!((275.0..=285.0).contains(&deg), "timur Makkah: {deg}");
        assert_eq!(arah_key(deg), "dir_w");
        // Titik barat Makkah, lintang sama: bearing < 90° (bukan timur murni).
        let deg = bearing(21.4225, 30.0);
        assert!((75.0..=85.0).contains(&deg), "barat Makkah: {deg}");
        assert_eq!(arah_key(deg), "dir_e");
        // Tepat utara & sektor pembulatan 337.5° → utara.
        assert_eq!(arah_key(0.0), "dir_n");
        assert_eq!(arah_key(337.5), "dir_n");
        assert_eq!(arah_key(359.9), "dir_n");
    }
}
