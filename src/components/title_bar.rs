//! Bar atas window utama ala komponen "Bar Atas" di design system pen:
//! brand + tab kiri, kontrol jendela (minimize / maximize / close) kanan.
//! Seluruh bar bisa dipakai drag window; tab & tombol tidak ikut drag.

use crate::components::ui::icons;
use crate::i18n::tr;
use crate::state;
use dioxus::prelude::*;

/// Tab aplikasi pada bar atas (design system pen).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Harian,
    Bulanan,
    Kalender,
    Pengaturan,
    Tentang,
}

#[component]
pub fn TitleBar(tab: Tab, on_select_tab: EventHandler<Tab>) -> Element {
    let lang = state::settings().language;
    let tabs = [
        (Tab::Harian, tr(lang, "tab_daily")),
        (Tab::Bulanan, tr(lang, "tab_monthly")),
        (Tab::Kalender, tr(lang, "tab_kalender")),
        (Tab::Pengaturan, tr(lang, "settings_title")),
        (Tab::Tentang, tr(lang, "tab_tentang")),
    ];

    // Status maximize reaktif: glyph tombol tengah berganti restore saat
    // dimaksimalkan. Poll ringan menangkap perubahan dari luar tombol ini
    // (mis. Win+Panah Atas) karena tao tidak mengirim event maximize.
    let mut maximized = use_signal(|| dioxus::desktop::window().is_maximized());
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(300)).await;
            maximized.set(dioxus::desktop::window().is_maximized());
        }
    });

    rsx! {
        div {
            class: "shrink-0 flex items-center gap-6 p-2 select-none cursor-default",
            onmousedown: move |_| { dioxus::desktop::window().drag(); },

            div { class: "flex items-center gap-2.5 px-4",
                icons::Mosque { class: "w-[22px] h-[22px] text-ink" }
                span { class: "text-base font-bold tracking-wide text-ink", "Jadwal Sholat" }
            }

            div { class: "flex items-center gap-4",
                for (t, label) in tabs {
                    {
                        let active = t == tab;
                        rsx! {
                            button {
                                class: if active {
                                    "flex flex-col gap-[5px] px-2 py-1.5 border-b-[2.5px] border-ink text-[13.5px] font-bold text-ink cursor-pointer"
                                } else {
                                    "flex flex-col gap-[5px] px-2 py-1.5 border-b-[2.5px] border-transparent text-[13.5px] font-semibold text-ink-2 hover:text-ink transition-colors cursor-pointer"
                                },
                                onmousedown: move |e| e.stop_propagation(),
                                onclick: move |_| on_select_tab.call(t),
                                "{label}"
                            }
                        }
                    }
                }
            }

            div { class: "flex-1" }

            div { class: "flex items-center gap-0.5",
                button {
                    class: "w-[42px] h-[42px] flex items-center justify-center rounded-lg text-ink-2 hover:bg-ink/5 transition-colors cursor-pointer",
                    aria_label: tr(lang, "win_minimize"),
                    onmousedown: move |e| e.stop_propagation(),
                    onclick: move |_| dioxus::desktop::window().set_minimized(true),
                    icons::IconMinus { class: "w-[15px] h-[15px]" }
                }
                button {
                    class: "w-[42px] h-[42px] flex items-center justify-center rounded-lg text-ink-2 hover:bg-ink/5 transition-colors cursor-pointer",
                    aria_label: if maximized() {
                        tr(lang, "win_restore")
                    } else {
                        tr(lang, "win_maximize")
                    },
                    onmousedown: move |e| e.stop_propagation(),
                    onclick: move |_| {
                        let win = dioxus::desktop::window();
                        let next = !win.is_maximized();
                        win.set_maximized(next);
                        maximized.set(next);
                    },
                    if maximized() {
                        icons::IconRestore { class: "w-3 h-3" }
                    } else {
                        icons::IconSquare { class: "w-3 h-3" }
                    }
                }
                button {
                    class: "w-[42px] h-[42px] flex items-center justify-center rounded-lg text-ink-2 hover:bg-red-500 hover:text-white transition-colors cursor-pointer",
                    aria_label: tr(lang, "close_hint"),
                    onmousedown: move |e| e.stop_propagation(),
                    onclick: move |_| dioxus::desktop::window().set_visible(false),
                    icons::IconClose { class: "w-[15px] h-[15px]" }
                }
            }
        }
    }
}
