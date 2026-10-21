//! Droplist (combobox) ala komponen "Droplist" & "Droplist — Cari" design
//! system pen: trigger dropdown tertutup (nilai + chevron), panel terbuka
//! di bawahnya (lebar 240, radius 10, shadow) berisi daftar item — item
//! aktif disorot amber dengan ikon centang. Mode `searchable` menambah
//! kotak cari (ikon + input) di atas daftar.
//!
//! Panel di-overlay di atas backdrop full-screen: klik di luar / Esc
//! menutup. Filter dikerjakan di client.

use super::icons;
use crate::i18n::tr;
use crate::state;
use dioxus::prelude::*;

/// Kelas trigger dropdown tertutup — turunan `{SELECT}` tapi berbentuk
/// tombol: nilai + chevron, selebar kolom kontrol.
const TRIGGER: &str = "flex items-center justify-between gap-2 w-[210px] bg-pass border border-pass-line rounded-lg px-3 py-2 text-[13px] font-semibold text-ink text-left outline-none hover:border-ink-3 focus:border-ink-3 transition-colors cursor-pointer";

/// Kelas panel terbuka — `bg-pass rounded-[10px]` + shadow kartu, padding
/// 6, gap 2 (spec pen: width 240, padding 6, gap 2).
const PANEL: &str = "absolute right-0 top-[calc(100%_+_6px)] z-50 w-[240px] flex flex-col gap-0.5 bg-pass rounded-[10px] border border-pass-line p-1.5 shadow-[0_8px_18px_#14161a1a] dark:shadow-[0_8px_18px_#00000080]";

#[component]
pub fn DropSearch(
    /// Teks yang tampil di trigger (label nilai aktif).
    value: String,
    /// Kunci nilai aktif — item yang disorot amber di daftar.
    active: String,
    /// Semua pilihan sebagai (kunci, label); difilter client-side.
    options: Vec<(String, String)>,
    /// Tampilkan kotak cari di atas daftar (untuk daftar panjang).
    searchable: bool,
    on_select: EventHandler<String>,
) -> Element {
    let lang = state::settings().language;
    let mut open = use_signal(|| false);
    let mut kueri = use_signal(|| String::new());

    let cari = kueri().to_lowercase();
    let daftar: Vec<(String, String)> = if !searchable || cari.is_empty() {
        options.clone()
    } else {
        options
            .iter()
            .filter(|(_, label)| label.to_lowercase().contains(&cari))
            .cloned()
            .collect()
    };

    rsx! {
        div { class: "relative min-w-0",
            button {
                class: "{TRIGGER}",
                onclick: move |_| {
                    kueri.set(String::new());
                    open.toggle();
                },
                span { class: "truncate", "{value}" }
                icons::IconChevronDown { class: "w-[13px] h-[13px] text-ink-2 shrink-0" }
            }
            if open() {
                // Backdrop transparan: klik di mana pun di luar panel menutup.
                div { class: "fixed inset-0 z-40", onmousedown: move |_| open.set(false) }
                div { class: "{PANEL}",
                    if searchable {
                        div { class: "flex items-center gap-2 border border-pass-line rounded-lg px-3 py-2",
                            icons::IconSearch { class: "w-[13px] h-[13px] text-ink-3 shrink-0" }
                            input {
                                class: "w-full bg-transparent text-[13px] font-semibold text-ink placeholder:text-ink-3 placeholder:font-normal outline-none",
                                r#type: "text",
                                value: "{kueri()}",
                                placeholder: tr(lang, "city_search_hint"),
                                autofocus: true,
                                oninput: move |e| kueri.set(e.value()),
                                onkeydown: move |e| {
                                    if e.key() == Key::Escape {
                                        open.set(false);
                                    }
                                },
                            }
                        }
                    }
                    div { class: "flex flex-col gap-0.5 max-h-[260px] overflow-y-auto",
                        if daftar.is_empty() {
                            span { class: "px-2.5 py-2 text-[13px] text-ink-3",
                                "{tr(lang, \"no_result\")}"
                            }
                        }
                        for (kunci, label) in daftar {
                            {
                                let aktif = kunci == active;
                                let item_class = if aktif {
                                    "flex items-center justify-between gap-2 rounded-md bg-amber px-2.5 py-2 text-[13px] font-semibold text-amber-ink cursor-pointer text-left"
                                } else {
                                    "flex items-center rounded-md px-2.5 py-2 text-[13px] text-ink hover:bg-ink/5 cursor-pointer text-left"
                                };
                                rsx! {
                                    button {
                                        key: "{kunci}",
                                        class: "{item_class}",
                                        onclick: move |_| {
                                            open.set(false);
                                            on_select.call(kunci.clone());
                                        },
                                        span { class: "truncate", "{label}" }
                                        if aktif {
                                            icons::IconCheck { class: "w-[13px] h-[13px] text-amber-ink shrink-0" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
