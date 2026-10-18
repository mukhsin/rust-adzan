use super::icons;
use super::schedule::{BULAN, HARI, Header, PrayerList};
use crate::prayer;
use crate::settings::Theme;
use crate::state;
use chrono::Datelike;
use dioxus::prelude::*;

/// Root komponen widget window — dom terpisah dari window utama,
/// state dibaca dari global `state::settings()` setiap tick.
/// Dua ukuran: kecil (hanya header) dan besar (header + daftar jadwal).
/// Drag memakai mekanisme native (`window.drag()`, smooth seperti main
/// window); saat drag selesai posisi di-snap ke margin oleh
/// `crate::ui::clamp_widget_position`.
#[allow(non_snake_case)]
pub fn WidgetApp() -> Element {
    let mut tick = use_signal(|| 0u64);
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            tick += 1;
        }
    });
    let mut theme = use_signal(|| state::settings().theme);
    let mut expanded = use_signal(|| false);
    // Handle window per-closure (Rc, murah untuk di-clone).
    let win_drag = dioxus::desktop::window();
    let win_expand = win_drag.clone();

    let _ = tick();
    // Sinkronkan tema dari dom lain (mis. toggle di main window).
    if *theme.read() != state::settings().theme {
        theme.set(state::settings().theme);
    }

    let settings = state::settings();
    let now = prayer::now_in_tz(&settings);
    let times = prayer::day_times(&settings, now.date_naive());
    let lokasi = match &settings.city {
        Some(kota) => format!("{kota}, Indonesia"),
        None => format!("({:.4}, {:.4})", settings.lat, settings.lng),
    };
    let tanggal = format!(
        "{}, {} {} {}",
        HARI[now.weekday().num_days_from_monday() as usize],
        now.day(),
        BULAN[(now.month() - 1) as usize],
        now.year()
    );

    let mut root_class = String::from(
        "h-screen w-screen flex flex-col overflow-hidden select-none font-sans \
         bg-white dark:bg-zinc-950 text-zinc-900 dark:text-zinc-50 \
         border border-zinc-200 dark:border-zinc-800 transition-colors duration-300",
    );
    if theme.read().is_dark() {
        root_class.push_str(" dark");
    }

    let btn = "p-1.5 rounded-md hover:bg-zinc-200 dark:hover:bg-zinc-800 text-zinc-500 dark:text-zinc-400 transition-colors";
    let is_expanded = *expanded.read();
    let dark = theme.read().is_dark();
    let content_class = if is_expanded {
        "flex-1 p-5 overflow-y-auto flex flex-col gap-6"
    } else {
        "flex-1 p-5 overflow-hidden"
    };

    rsx! {
        style { {crate::ui::TAILWIND_CSS} }
        div {
            class: "{root_class}",
            // Title bar: drag area, tombol tema, pengaturan & perbesar/perkecil.
            div {
                class: "h-10 shrink-0 flex items-center justify-between px-3 select-none cursor-default bg-zinc-50 dark:bg-zinc-900 border-b border-zinc-200 dark:border-zinc-800 transition-colors duration-300",
                onmousedown: move |_| {
                    // Drag native — modal loop OS, kembali saat mouse dilepas.
                    win_drag.drag();
                    // drag() di Windows async: clamp di sini jalan saat
                    // mousedown, bukan saat drag selesai. Karena itu snap
                    // ditangguhkan ke watcher yang memantau tombol mouse.
                    crate::ui::watch_widget_drag_release();
                },

                div { class: "flex items-center gap-2",
                    icons::Mosque { class: "w-4 h-4 text-zinc-600 dark:text-zinc-300" }
                    span { class: "text-xs font-medium text-zinc-600 dark:text-zinc-300", "Jadwal Sholat" }
                }

                div { class: "flex items-center gap-1",
                    button {
                        class: "{btn}",
                        title: "Ganti tema",
                        onmousedown: move |e| e.stop_propagation(),
                        onclick: move |_| {
                            let next = if dark { Theme::Light } else { Theme::Dark };
                            let mut s = (*state::settings()).clone();
                            s.theme = next;
                            state::update_settings(s);
                            theme.set(next);
                        },
                        if dark {
                            icons::IconSun { class: "w-3.5 h-3.5" }
                        } else {
                            icons::IconMoon { class: "w-3.5 h-3.5" }
                        }
                    }
                    button {
                        class: "{btn}",
                        title: "Buka Pengaturan",
                        onmousedown: move |e| e.stop_propagation(),
                        onclick: move |_| {
                            // Widget tetap tampil — main window boleh bersamaan.
                            state::send_main_command(state::MainCommand::ShowMain);
                        },
                        icons::IconSettings { class: "w-3.5 h-3.5" }
                    }
                    button {
                        class: "{btn}",
                        title: if is_expanded { "Perkecil widget" } else { "Tampilkan jadwal lengkap" },
                        onmousedown: move |e| e.stop_propagation(),
                        onclick: move |_| {
                            let next = !*expanded.read();
                            expanded.set(next);
                            crate::ui::apply_widget_size(&win_expand, next);
                        },
                        if is_expanded {
                            icons::IconMinimize2 { class: "w-3.5 h-3.5" }
                        } else {
                            icons::IconMaximize2 { class: "w-3.5 h-3.5" }
                        }
                    }
                }
            }

            {match times {
                Some(times) => rsx! {
                    div { class: "{content_class}",
                        Header { lokasi: lokasi.clone(), tanggal: tanggal.clone(), times: times, now: now.time() }
                        if is_expanded {
                            PrayerList { times: times, now: now.time() }
                        }
                    }
                },
                None => rsx! {
                    div { class: "flex-1 flex items-center justify-center text-xs text-zinc-500",
                        "Gagal menghitung jadwal."
                    }
                },
            }}
        }
    }
}
