//! Ikon system tray + menu (Tampilkan / Widget Desktop / Keluar).

use crate::layouts::window;
use crate::state;
use dioxus::desktop::trayicon::menu::{CheckMenuItem, MenuItem, PredefinedMenuItem};
use dioxus::desktop::trayicon::{TrayIconBuilder, use_tray_icon};
use dioxus::desktop::use_muda_event_handler;
use dioxus::desktop::use_tray_icon_event_handler;
use dioxus::prelude::*;

/// Inisialisasi system tray + handler menu tray (sekali saja).
#[component]
pub fn TraySetup() -> Element {
    // Inisialisasi tray (sekali) — jangan memanggil hook lain di dalam use_hook.
    // Dibangun lewat TrayIconBuilder langsung (bukan init_tray_icon) supaya
    // bisa memasang TOOLTIP nama aplikasi saat ikon tray di-hover.
    use_hook(|| {
        let lang = state::settings().language;
        let menu = dioxus::desktop::trayicon::menu::Menu::new();
        let show = MenuItem::with_id("show", crate::i18n::tr(lang, "menu_show"), true, None);
        let widget = CheckMenuItem::with_id(
            "widget",
            crate::i18n::tr(lang, "menu_widget"),
            true,
            false,
            None,
        );
        let quit = MenuItem::with_id("quit", crate::i18n::tr(lang, "menu_quit"), true, None);
        menu.append_items(&[
            &show,
            &PredefinedMenuItem::separator(),
            &widget,
            &PredefinedMenuItem::separator(),
            &quit,
        ])
        .expect("isi menu tray");

        let icon = dioxus::desktop::trayicon::Icon::from_rgba(
            crate::layouts::TRAY_ICON_RGBA.to_vec(),
            32,
            32,
        )
        .expect("tray icon");
        let tray = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(false)
            .with_tooltip(crate::settings::APP_NAME)
            .with_icon(icon)
            .build()
            .expect("tray icon builder failed");
        provide_context(tray);
        state::WIDGET_MENU_ITEM.with(|cell| *cell.borrow_mut() = Some(widget));
        state::TRAY_SHOW_ITEM.with(|cell| *cell.borrow_mut() = Some(show));
        state::TRAY_QUIT_ITEM.with(|cell| *cell.borrow_mut() = Some(quit));
        TRAY_LANG.with(|cell| cell.set(Some(lang)));
    });

    // Hook di level atas komponen, bukan di dalam hook lain.
    let tray = use_tray_icon();

    // Menu tray ikut ganti bahasa tanpa restart (render = main thread).
    sync_tray_language(state::settings().language);

    // Klik kiri ikon tray: toggle widget desktop.
    // Event `Click` di-emit dua kali (press & release) — reaksi hanya saat
    // release supaya toggle tidak langsung dibatalkan oleh event kedua.
    use_tray_icon_event_handler(|event| {
        if let dioxus::desktop::trayicon::TrayIconEvent::Click {
            button: dioxus::desktop::trayicon::MouseButton::Left,
            button_state: dioxus::desktop::trayicon::MouseButtonState::Up,
            ..
        } = event
        {
            window::toggle_widget();
        }
    });

    // PENTING: event menu tray di dioxus-desktop 0.7 datang sebagai `MudaMenuEvent`,
    // bukan `TrayMenuEvent` — handler `muda::MenuEvent` (menubar) dipasang lebih
    // dulu dan muda menyimpannya di OnceCell, sehingga pemasangan handler kedua
    // dioxus untuk `tray_icon::menu::MenuEvent` gagal diam-diam. Karena itu
    // `use_tray_menu_event_handler` tidak pernah terpanggil.
    use_muda_event_handler(move |event| match event.id().0.as_str() {
        "show" => window::show_main(),
        "widget" => window::toggle_widget(),
        "quit" => {
            if let Some(tray) = &tray {
                let _ = tray.set_visible(false);
            }
            std::process::exit(0);
        }
        _ => {}
    });

    rsx! {}
}

thread_local! {
    /// Bahasa yang sedang tampil di menu tray — untuk deteksi perubahan.
    static TRAY_LANG: std::cell::Cell<Option<crate::settings::Language>>
        = const { std::cell::Cell::new(None) };
}

/// Ganti teks item menu tray bila bahasa settings berubah.
/// Dipanggil dari render TraySetup (main thread — item muda tidak Send).
fn sync_tray_language(lang: crate::settings::Language) {
    let changed = TRAY_LANG.with(|cell| {
        if cell.get() != Some(lang) {
            cell.set(Some(lang));
            true
        } else {
            false
        }
    });
    if !changed {
        return;
    }
    use crate::i18n::tr;
    state::WIDGET_MENU_ITEM.with(|cell| {
        if let Some(item) = cell.borrow().as_ref() {
            item.set_text(tr(lang, "menu_widget"));
        }
    });
    state::TRAY_SHOW_ITEM.with(|cell| {
        if let Some(item) = cell.borrow().as_ref() {
            item.set_text(tr(lang, "menu_show"));
        }
    });
    state::TRAY_QUIT_ITEM.with(|cell| {
        if let Some(item) = cell.borrow().as_ref() {
            item.set_text(tr(lang, "menu_quit"));
        }
    });
}
