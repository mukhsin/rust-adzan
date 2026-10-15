pub mod icons;
pub mod main_view;
pub mod settings_view;
pub mod widget;

use crate::settings::Theme;
use crate::state;
use dioxus::desktop::trayicon::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use dioxus::desktop::trayicon::{init_tray_icon, use_tray_icon};
use dioxus::desktop::use_muda_event_handler;
use dioxus::desktop::use_tray_icon_event_handler;
use dioxus::desktop::{
    Config, WindowBuilder,
    tao::dpi::{LogicalPosition, LogicalSize},
    tao::platform::windows::WindowBuilderExtWindows,
};
use dioxus::prelude::*;
use state::WIDGET_MENU_ITEM;

/// CSS hasil kompilasi Tailwind (assets/tailwind.css, dibangun via tools/build-css.sh).
pub static TAILWIND_CSS: &str = include_str!("../../assets/tailwind.css");
/// Data piksel RGBA (256×256) untuk ikon window/taskbar.
pub static WINDOW_ICON_RGBA: &[u8] = include_bytes!("../../assets/icon.rgba");
/// Data piksel RGBA (32×32) untuk tray icon.
pub static TRAY_ICON_RGBA: &[u8] = include_bytes!("../../assets/icon-tray.rgba");

/// Tampilan yang sedang aktif di window utama.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum View {
    Main,
    Settings,
}

pub fn app() -> Element {
    let mut view = use_signal(|| View::Main);
    let mut theme = use_signal(|| state::settings().theme);

    // Sinkronkan tema dari dom lain (mis. toggle di widget window) tiap detik.
    let mut sync_tick = use_signal(|| 0u64);
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            sync_tick += 1;
        }
    });
    let _ = sync_tick();
    if *theme.read() != state::settings().theme {
        theme.set(state::settings().theme);
    }

    // Perintah lintas window: tombol jam di widget menampilkan main window.
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
            while let Some(cmd) = state::pop_main_command() {
                match cmd {
                    state::MainCommand::ShowMain => {
                        let win = dioxus::desktop::window();
                        win.set_minimized(false);
                        win.set_visible(true);
                        win.set_focus();
                    }
                }
            }
        }
    });

    let mut root_class = String::from(
        "h-screen w-screen flex flex-col overflow-hidden font-sans \
         bg-white dark:bg-zinc-950 text-zinc-900 dark:text-zinc-50 \
         transition-colors duration-300",
    );
    if theme.read().is_dark() {
        root_class.push_str(" dark");
    }

    let title: &'static str = if *view.read() == View::Main {
        "Jadwal Sholat"
    } else {
        "Pengaturan"
    };

    rsx! {
        style { {TAILWIND_CSS} }
        TraySetup {}
        div { class: "{root_class}",
            TitleBar {
                title,
                show_back: *view.read() == View::Settings,
                on_back: move |_| view.set(View::Main),
                show_settings: *view.read() == View::Main,
                on_settings: move |_| view.set(View::Settings),
                theme,
            }
            if *view.read() == View::Main {
                main_view::MainView {}
            } else {
                settings_view::SettingsView {}
            }
        }
    }
}

/// Title bar custom (window frameless): logo + judul, toggle tema, tombol settings,
/// minimize, maximize, close. Seluruh bar bisa dipakai drag window.
#[component]
pub fn TitleBar(
    title: &'static str,
    show_back: bool,
    on_back: EventHandler<()>,
    show_settings: bool,
    on_settings: EventHandler<()>,
    theme: Signal<Theme>,
) -> Element {
    let btn = "p-1.5 rounded-md hover:bg-zinc-200 dark:hover:bg-zinc-800 text-zinc-500 dark:text-zinc-400 transition-colors";
    let dark = theme.read().is_dark();

    rsx! {
        div {
            class: "h-10 shrink-0 flex items-center justify-between px-3 select-none cursor-default bg-zinc-50 dark:bg-zinc-900 border-b border-zinc-200 dark:border-zinc-800 transition-colors duration-300",
            onmousedown: move |_| { dioxus::desktop::window().drag(); },

            div { class: "flex items-center gap-2",
                if show_back {
                    button {
                        class: "{btn}",
                        title: "Kembali",
                        onmousedown: move |e| e.stop_propagation(),
                        onclick: move |_| on_back.call(()),
                        icons::IconBack { class: "w-3.5 h-3.5" }
                    }
                } else {
                    icons::Mosque { class: "w-4 h-4 text-zinc-600 dark:text-zinc-300" }
                }
                span { class: "text-xs font-medium text-zinc-600 dark:text-zinc-300", "{title}" }
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
                if show_settings {
                    button {
                        class: "{btn}",
                        title: "Pengaturan",
                        onmousedown: move |e| e.stop_propagation(),
                        onclick: move |_| on_settings.call(()),
                        icons::IconSettings { class: "w-3.5 h-3.5" }
                    }
                }
                div { class: "w-px h-4 bg-zinc-300 dark:bg-zinc-700 mx-1" }
                button {
                    class: "{btn}",
                    title: "Minimize",
                    onmousedown: move |e| e.stop_propagation(),
                    onclick: move |_| dioxus::desktop::window().set_minimized(true),
                    icons::IconMinus { class: "w-3.5 h-3.5" }
                }
                button {
                    class: "{btn}",
                    title: "Maximize",
                    onmousedown: move |e| e.stop_propagation(),
                    onclick: move |_| dioxus::desktop::window().toggle_maximized(),
                    icons::IconSquare { class: "w-3 h-3" }
                }
                button {
                    class: "p-1.5 rounded-md hover:bg-red-500 hover:text-white dark:hover:bg-red-500 dark:hover:text-white text-zinc-500 dark:text-zinc-400 transition-colors",
                    title: "Tutup (lanjut di widget)",
                    onmousedown: move |e| e.stop_propagation(),
                    onclick: move |_| show_widget_only(),
                    icons::IconClose { class: "w-3.5 h-3.5" }
                }
            }
        }
    }
}

/// Inisialisasi system tray + handler menu tray (sekali saja).
#[component]
fn TraySetup() -> Element {
    // Inisialisasi tray (sekali) — jangan memanggil hook lain di dalam use_hook.
    use_hook(|| {
        let menu = Menu::new();
        let show = MenuItem::with_id("show", "Tampilkan Jendela", true, None);
        let widget = CheckMenuItem::with_id("widget", "Widget Desktop", true, false, None);
        let quit = MenuItem::with_id("quit", "Keluar", true, None);
        menu.append_items(&[
            &show,
            &PredefinedMenuItem::separator(),
            &widget,
            &PredefinedMenuItem::separator(),
            &quit,
        ])
        .expect("isi menu tray");

        let icon = dioxus::desktop::trayicon::Icon::from_rgba(TRAY_ICON_RGBA.to_vec(), 32, 32)
            .expect("tray icon");
        let _tray = init_tray_icon(menu, Some(icon));
        WIDGET_MENU_ITEM.with(|cell| *cell.borrow_mut() = Some(widget));
    });

    // Hook di level atas komponen, bukan di dalam hook lain.
    let tray = use_tray_icon();

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
            toggle_widget();
        }
    });

    // PENTING: event menu tray di dioxus-desktop 0.7 datang sebagai `MudaMenuEvent`,
    // bukan `TrayMenuEvent` — handler `muda::MenuEvent` (menubar) dipasang lebih
    // dulu dan muda menyimpannya di OnceCell, sehingga pemasangan handler kedua
    // dioxus untuk `tray_icon::menu::MenuEvent` gagal diam-diam. Karena itu
    // `use_tray_menu_event_handler` tidak pernah terpanggil.
    use_muda_event_handler(move |event| match event.id().0.as_str() {
        "show" => show_main_only(),
        "widget" => toggle_widget(),
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

/// Buat widget window (lazy — dipanggil saat pertama kali widget dibutuhkan).
/// Setelah webview selesai load, dioxus otomatis memvisibilitaskan window ini
/// (flag app-level `is_visible_before_start`), pas dengan saat ia diminta tampil.
fn spawn_widget_creation() {
    spawn(async move {
        let main_win = dioxus::desktop::window();
        let mut builder = WindowBuilder::new()
            .with_decorations(false)
            .with_resizable(false)
            .with_always_on_top(true)
            .with_skip_taskbar(true)
            .with_visible(false)
            .with_inner_size(LogicalSize::new(300.0, 280.0));

        // Posisikan di pojok kanan-bawah layar.
        if let Some(monitor) = main_win.current_monitor() {
            let scale = monitor.scale_factor();
            let size = monitor.size();
            let w = (300.0 * scale) as i32;
            let h = (280.0 * scale) as i32;
            let x = size.width as i32 - w - (12.0 * scale) as i32;
            let y = size.height as i32 - h - (80.0 * scale) as i32;
            builder =
                builder.with_position(LogicalPosition::new(x as f64 / scale, y as f64 / scale));
        }

        let cfg = Config::new()
            .with_window(builder)
            .with_exits_when_last_window_closes(false);

        let ctx = main_win
            .new_window(VirtualDom::new(widget::WidgetApp), cfg)
            .await;
        *state::WIDGET_WINDOW.lock().unwrap() = Some(ctx.window.clone());
        ctx.window.set_visible(true);
    });
}

/// Tampilkan widget (window-nya dibuat lazy saat pertama kali).
pub fn show_widget() {
    WIDGET_MENU_ITEM.with(|cell| {
        if let Some(item) = cell.borrow().as_ref() {
            item.set_checked(true);
        }
    });
    if let Some(w) = state::WIDGET_WINDOW.lock().unwrap().as_ref() {
        w.set_visible(true);
        w.set_focus();
    } else {
        spawn_widget_creation();
    }
}

/// Sembunyikan widget (jika ada) + sinkron centang tray.
pub fn hide_widget() {
    WIDGET_MENU_ITEM.with(|cell| {
        if let Some(item) = cell.borrow().as_ref() {
            item.set_checked(false);
        }
    });
    if let Some(w) = state::WIDGET_WINDOW.lock().unwrap().as_ref() {
        w.set_visible(false);
    }
}

/// Ganti window yang tampil: main → widget. Invarian: hanya satu window
/// yang terlihat pada satu waktu.
pub fn show_widget_only() {
    dioxus::desktop::window().set_visible(false);
    show_widget();
}

/// Ganti window yang tampil: widget → main.
pub fn show_main_only() {
    hide_widget();
    let win = dioxus::desktop::window();
    win.set_minimized(false);
    win.set_visible(true);
    win.set_focus();
}

/// Toggle widget dari tray/klik kiri ikon.
fn toggle_widget() {
    let visible = state::WIDGET_WINDOW
        .lock()
        .unwrap()
        .as_ref()
        .is_some_and(|w| w.is_visible());
    if visible {
        hide_widget();
    } else {
        show_widget_only();
    }
}
