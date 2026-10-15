use super::icons;
use super::main_view::{BULAN, HARI, Header};
use crate::prayer::{self};
use crate::state;
use chrono::Datelike;
use dioxus::prelude::*;

/// Root komponen widget window — dom terpisah dari window utama,
/// state dibaca dari global `state::settings()` setiap tick.
/// Title bar & konten sama seperti main window, tanpa daftar jadwal.
#[allow(non_snake_case)]
pub fn WidgetApp() -> Element {
    let mut tick = use_signal(|| 0u64);
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            tick += 1;
        }
    });

    let _ = tick();
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
        "h-screen w-screen flex flex-col overflow-hidden font-sans \
         bg-white dark:bg-zinc-950 text-zinc-900 dark:text-zinc-50 \
         border border-zinc-200 dark:border-zinc-800 transition-colors duration-300",
    );
    if settings.theme.is_dark() {
        root_class.push_str(" dark");
    }

    rsx! {
        style { {crate::ui::TAILWIND_CSS} }
        div { class: "{root_class}",
            // Title bar: sama seperti main window; satu-satunya tombol = ✕ (sembunyikan widget).
            div {
                class: "h-10 shrink-0 flex items-center justify-between px-3 select-none cursor-default bg-zinc-50 dark:bg-zinc-900 border-b border-zinc-200 dark:border-zinc-800 transition-colors duration-300",
                onmousedown: move |_| { dioxus::desktop::window().drag(); },

                div { class: "flex items-center gap-2",
                    icons::Mosque { class: "w-4 h-4 text-zinc-600 dark:text-zinc-300" }
                    span { class: "text-xs font-medium text-zinc-600 dark:text-zinc-300", "Jadwal Sholat" }
                }

                button {
                    class: "p-1.5 rounded-md hover:bg-red-500 hover:text-white dark:hover:bg-red-500 dark:hover:text-white text-zinc-500 dark:text-zinc-400 transition-colors",
                    title: "Sembunyikan widget",
                    onmousedown: move |e| e.stop_propagation(),
                    onclick: move |_| {
                        crate::ui::set_widget_visible(false);
                        crate::state::WIDGET_MENU_ITEM.with(|cell| {
                            if let Some(item) = cell.borrow().as_ref() {
                                item.set_checked(false);
                            }
                        });
                        let mut s = (*state::settings()).clone();
                        s.widget_enabled = false;
                        state::update_settings(s);
                    },
                    icons::IconClose { class: "w-3.5 h-3.5" }
                }
            }

            {match times {
                Some(times) => rsx! {
                    div { class: "flex-1 p-5 overflow-hidden",
                        Header { lokasi: lokasi.clone(), tanggal: tanggal.clone(), times: times, now: now.time() }
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
