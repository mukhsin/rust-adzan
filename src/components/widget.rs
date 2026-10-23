//! Root dom widget window — menampilkan kartu tiket penuh di atas window
//! transparan. Dom terpisah dari window utama; state dibaca dari global
//! `state::settings()` setiap tick.

use crate::components::ticket::{Ticket, tanggal_labels};
use crate::i18n::tr;
use crate::layouts::window;
use crate::prayer;
use crate::state;
use chrono::Datelike as _;
use dioxus::prelude::*;

/// Root komponen widget window.
/// Drag memakai mekanisme native (`window.drag()`, smooth seperti main
/// window); saat drag selesai posisi di-snap ke margin oleh
/// `crate::layouts::window::clamp_widget_position`.
#[allow(non_snake_case)]
pub fn WidgetApp() -> Element {
    // Window widget dibuat tersembunyi; tampilkan setelah dom siap. Efek ini
    // berjalan pada poll setelah event initialize — yaitu SETELAH dioxus
    // menyetel ulang visibilitas window (set_visible(is_visible_before_start)
    // = false karena window utama mulai tersembunyi), jadi dipanggilannya
    // tidak tertimpa dan widget tampil dengan konten siap tanpa kedipan.
    use_effect(move || {
        dioxus::desktop::window().set_visible(true);
    });

    let mut tick = use_signal(|| 0u64);
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            tick += 1;
        }
    });
    // Handle window per-closure (Rc, murah untuk di-clone).
    let win_drag = dioxus::desktop::window();

    let _ = tick();
    // Tema & settings dibaca langsung tiap render — tick 1 detik menjamin
    // perubahan dari dom lain (toggle tema di pengaturan) tampil maksimal
    // 1 detik kemudian.
    let settings = state::settings();
    let lang = settings.language;
    let dark = settings.theme.is_dark();
    let now = prayer::now_in_tz(&settings);
    let (hari, tanggal, tanggal_hijri) = tanggal_labels(lang, &now);

    let mut root_class =
        String::from("h-screen w-screen overflow-hidden select-none font-sans text-ink");
    if dark {
        root_class.push_str(" dark");
    }

    rsx! {
        style { {crate::layouts::TAILWIND_CSS} }
        div {
            class: "{root_class}",
            // Drag native — modal loop OS, kembali saat mouse dilepas.
            onmousedown: move |_| {
                win_drag.drag();
                // drag() di Windows async: clamp di sini jalan saat mousedown,
                // bukan saat drag selesai. Karena itu snap ditangguhkan ke
                // watcher yang memantau tombol mouse.
                window::watch_widget_drag_release();
            },
            // Klik ganda kartu = buka pengaturan (tak ada tombol di desain).
            ondoubleclick: move |_| state::send_main_command(state::MainCommand::ShowMain),

            match prayer::day_times(&settings, now.date_naive()) {
                Some(times) => rsx! {
                    Ticket {
                        times: times,
                        now: now.time(),
                        jumat: now.weekday() == chrono::Weekday::Fri,
                        tz: now.format("%Z").to_string(),
                        hari: hari,
                        tanggal: tanggal,
                        tanggal_hijri: tanggal_hijri,
                        ramadan_day: crate::dates::ramadan_day(now.date_naive()),
                    }
                },
                None => rsx! {
                    div { class: "w-full h-full flex items-center justify-center bg-pass rounded-[14px] widget-holes",
                        span { class: "text-sm text-ink-3", "{tr(lang, \"calc_failed\")}" }
                    }
                },
            }
        }
    }
}
