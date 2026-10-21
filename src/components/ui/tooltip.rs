//! Tooltip ala komponen "Tooltip" design system pen: bubble gelap (`$ink`,
//! radius 6, padding 6×10, teks 11px `$on-ink`) dengan panah yang MENEMPEL
//! ke bubble (tanpa celah), muncul DI ATAS target saat hover menggantikan
//! tooltip `title` native yang lambat muncul dan tak bisa di-gaya.
//!
//! Muncul/hilang dianimasikan (fade + scale 150ms); posisi bisa dipilih:
//! atas (default), bawah, kiri, kanan — `origin-*` mengikuti arah agar
//! scale tumbuh dari sisi anchor.
//!
//! Wrapper selalu ter-render (hanya opacity yang berubah) supaya transisi
//! jalan dua arah. Bubble `pointer-events-none` supaya tidak mengganggu
//! hover target. Hanya untuk window utama — window widget berukuran tetap,
//! bubble akan terpotong tepi window.

use dioxus::prelude::*;

/// Posisi bubble relatif terhadap target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PosisiTooltip {
    /// Di atas target, panah menunjuk ke bawah (default).
    #[default]
    Atas,
    /// Di bawah target, panah menunjuk ke atas.
    Bawah,
    /// Di kanan target, panah menunjuk ke kiri.
    /// (Belum ada pemakai — opsi API komponen untuk kebutuhan mendatang.)
    #[allow(dead_code)]
    Kanan,
    /// Di kiri target, panah menunjuk ke kanan.
    Kiri,
}

/// Kelas wrapper bubble per posisi: arah susunan, jarak dari anchor,
/// dan transform-origin animasi.
fn kelas_posisi(posisi: PosisiTooltip) -> &'static str {
    match posisi {
        PosisiTooltip::Atas => {
            "flex-col bottom-[calc(100%_+_6px)] left-1/2 -translate-x-1/2 origin-bottom"
        }
        PosisiTooltip::Bawah => {
            "flex-col top-[calc(100%_+_6px)] left-1/2 -translate-x-1/2 origin-top"
        }
        PosisiTooltip::Kanan => {
            "flex-row left-[calc(100%_+_6px)] top-1/2 -translate-y-1/2 origin-left"
        }
        PosisiTooltip::Kiri => {
            "flex-row right-[calc(100%_+_6px)] top-1/2 -translate-y-1/2 origin-right"
        }
    }
}

/// SVG panah penutup sesuai posisi — selalu menunjuk ke arah target.
fn panah(posisi: PosisiTooltip) -> Element {
    let (vb, d, w, h) = match posisi {
        PosisiTooltip::Atas => ("0 0 10 5", "M0 0 L10 0 L5 5 Z", "10", "5"),
        PosisiTooltip::Bawah => ("0 0 10 5", "M0 5 L10 5 L5 0 Z", "10", "5"),
        PosisiTooltip::Kanan => ("0 0 5 10", "M5 0 L5 10 L0 5 Z", "5", "10"),
        PosisiTooltip::Kiri => ("0 0 5 10", "M0 0 L0 10 L5 5 Z", "5", "10"),
    };
    rsx! {
        svg {
            class: "text-ink shrink-0",
            view_box: vb,
            fill: "currentColor",
            width: w,
            height: h,
            path { d }
        }
    }
}

#[component]
pub fn Tooltip(label: String, posisi: PosisiTooltip, children: Element) -> Element {
    let mut show = use_signal(|| false);
    let animasi = if *show.read() {
        "opacity-100 scale-100"
    } else {
        "opacity-0 scale-95 pointer-events-none"
    };
    rsx! {
        div { class: "relative inline-flex",
            div { class: "pointer-events-none absolute z-50 flex items-center transition-all duration-150 ease-out {kelas_posisi(posisi)} {animasi}",
                // Panah di sisi yang menghadap target; bubble & panah menempel.
                // Bawah: panah di atas bubble. Selain itu (Atas/Kanan/Kiri):
                // bubble dulu, panah setelahnya.
                if posisi == PosisiTooltip::Bawah {
                    {panah(posisi)}
                }
                div { class: "bg-ink text-on-ink rounded-md px-2.5 py-1.5 text-[11px] leading-snug whitespace-nowrap",
                    "{label}"
                }
                if posisi != PosisiTooltip::Bawah {
                    {panah(posisi)}
                }
            }
            div {
                class: "inline-flex",
                onmouseenter: move |_| show.set(true),
                onmouseleave: move |_| show.set(false),
                {children}
            }
        }
    }
}
