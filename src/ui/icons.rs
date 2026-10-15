//! Ikon Lucide dari crate [`dioxus_icons`] (satu komponen per ikon,
//! stroke default `currentColor` sehingga mengikuti kelas Tailwind `text-*`).
//!
//! Semua ikon diambil dari crate, KECUALI [`Mosque`]: ikon `mosque` belum ada
//! di set dioxus-icons 0.1 — komponennya memakai geometri persis
//! `assets/mosque.svg` (desain mosque aplikasi) dengan stroke `currentColor`.

use dioxus::prelude::*;

pub use dioxus_icons::lucide::{
    ArrowLeft as IconBack, Bell as IconBell, Calendar as IconCalendar, MapPin as IconMapPin,
    Minus as IconMinus, Moon as IconMoon, Settings as IconSettings, Square as IconSquare,
    Sun as IconSun, X as IconClose,
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
