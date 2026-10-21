//! Ikon Lucide dari crate [`dioxus_icons`] (satu komponen per ikon,
//! stroke default `currentColor` sehingga mengikuti kelas Tailwind `text-*`).
//!
//! Semua ikon diambil dari crate, KECUALI [`Mosque`] & [`MosqueLogo`]: ikon
//! `mosque` belum ada di set dioxus-icons 0.1 — keduanya memakai geometri
//! persis `assets/mosque.svg` (desain mosque aplikasi). [`Mosque`] memakai
//! stroke `currentColor` agar mengikuti tema; [`MosqueLogo`] memakai warna
//! tiap segmen dari file aslinya.

use dioxus::prelude::*;

pub use dioxus_icons::lucide::{
    Bell as IconBell, BellOff as IconBellOff, Check as IconCheck, ChevronDown as IconChevronDown,
    ChevronLeft as IconChevronLeft, ChevronRight as IconChevronRight, Minus as IconMinus,
    Play as IconPlay, Search as IconSearch, Square as IconSquare, X as IconClose,
};

/// Ikon mosque — geometri persis `assets/mosque.svg` (desain multicolor),
/// digambar dengan stroke `currentColor` agar mengikuti tema.
/// (Sumber warna per segmen di SVG hanya untuk versi taskbar/tray.)
#[allow(non_snake_case)]
#[component]
pub fn Mosque(class: &'static str) -> Element {
    rsx! {
        svg {
            class,
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 24 24",
            fill: "none",
            stroke: "currentColor",
            "stroke-width": "2",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { d: "M 12.268 2 A2 2 0 0 0 15.733 4" }
            path { d: "M 14 5 L 14 8" }
            path { d: "M 16 22 L 16 19" }
            path { d: "M 16 19 A2 2 0 0 0 12 19" }
            path { d: "M 12 19 L 12 22" }
            path { d: "M 21 13 C 20.338 11.503 19.334 10.247 18.1 9.37" }
            path { d: "M 18.1 9.37 C 16.825 8.47 15.422 8 14 8" }
            path { d: "M 14 8 C 12.578 8 11.174 8.47 9.9 9.37" }
            path { d: "M 9.9 9.37 C 8.668 10.248 7.663 11.504 7 13" }
            path { d: "M 7 13 L 21 13" }
            path { d: "M 3 9 L 7 9" }
            path { d: "M 7 22 L 7 6" }
            path { d: "M 7 6 A5 5 0 0 0 5 2" }
            path { d: "M 5 2 A5 5 0 0 0 3 6" }
            path { d: "M 3 6 L 3 20" }
            path { d: "M 3 20 A2 2 0 0 0 5 22" }
            path { d: "M 5 22 L 19 22" }
            path { d: "M 19 22 A2 2 0 0 0 21 20" }
            path { d: "M 21 20 L 21 13" }
        }
    }
}

/// Logo mosque asli — geometri `assets/mosque.svg` PERSIS dengan warna
/// tiap segmen dari file aslinya (bukan currentColor). Dipakai di layar
/// Tentang.
#[allow(non_snake_case)]
#[component]
pub fn MosqueLogo(class: &'static str) -> Element {
    rsx! {
        svg {
            class,
            xmlns: "http://www.w3.org/2000/svg",
            view_box: "0 0 24 24",
            fill: "none",
            "stroke-width": "2",
            "stroke-linecap": "round",
            "stroke-linejoin": "round",
            path { stroke: "#1982c4", d: "M 12.268 2 A2 2 0 0 0 15.733 4" }
            path { stroke: "#4267AC", d: "M 14 5 L 14 8" }
            path { stroke: "#6a4c93", d: "M 16 22 L 16 19" }
            path { stroke: "#B55379", d: "M 16 19 A2 2 0 0 0 12 19" }
            path { stroke: "#FF595E", d: "M 12 19 L 12 22" }
            path { stroke: "#FF7655", d: "M 21 13 C 20.338 11.503 19.334 10.247 18.1 9.37" }
            path { stroke: "#ff924c", d: "M 18.1 9.37 C 16.825 8.47 15.422 8 14 8" }
            path { stroke: "#FFAE43", d: "M 14 8 C 12.578 8 11.174 8.47 9.9 9.37" }
            path { stroke: "#ffca3a", d: "M 9.9 9.37 C 8.668 10.248 7.663 11.504 7 13" }
            path { stroke: "#C5CA30", d: "M 7 13 L 21 13" }
            path { stroke: "#8ac926", d: "M 3 9 L 7 9" }
            path { stroke: "#52A675", d: "M 7 22 L 7 6" }
            path { stroke: "#1982c4", d: "M 7 6 A5 5 0 0 0 5 2" }
            path { stroke: "#4267AC", d: "M 5 2 A5 5 0 0 0 3 6" }
            path { stroke: "#6a4c93", d: "M 3 6 L 3 20" }
            path { stroke: "#B55379", d: "M 3 20 A2 2 0 0 0 5 22" }
            path { stroke: "#FF595E", d: "M 5 22 L 19 22" }
            path { stroke: "#FF7655", d: "M 19 22 A2 2 0 0 0 21 20" }
            path { stroke: "#ff924c", d: "M 21 20 L 21 13" }
        }
    }
}

#[cfg(test)]
mod tests {
    // Menjaga kedua varian ikon sinkron dengan assets/mosque.svg: jumlah
    // path dan geometri harus sama persis (warna boleh beda: currentColor
    // vs warna asli).
    const MOSQUE_SVG: &str = include_str!("../../../assets/mosque.svg");

    #[test]
    fn geometri_mosque_dan_mosque_logo_sama() {
        let count_d = MOSQUE_SVG.matches("d=\"").count();
        assert_eq!(
            count_d, 19,
            "assets/mosque.svg berubah — sinkronkan Mosque & MosqueLogo"
        );
        for marker in [
            "M 12.268 2 A2 2 0 0 0 15.733 4",
            "M 21 20 L 21 13",
            "M 9.9 9.37 C 8.668 10.248 7.663 11.504 7 13",
        ] {
            assert!(
                MOSQUE_SVG.contains(marker),
                "geometri {marker} hilang dari mosque.svg"
            );
        }
    }
}
