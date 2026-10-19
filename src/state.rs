use crate::settings::Settings;
use std::sync::{Arc, Mutex, OnceLock, RwLock};

/// Settings global yang dibagikan antara UI utama, widget, dan scheduler thread.
static SETTINGS: OnceLock<RwLock<Arc<Settings>>> = OnceLock::new();

/// Perintah UI lintas window (dari dom widget → dom utama).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MainCommand {
    /// Sembunyikan widget dan tampilkan window utama.
    ShowMain,
}

static MAIN_CMD_TX: OnceLock<tokio::sync::mpsc::UnboundedSender<MainCommand>> = OnceLock::new();
static MAIN_CMD_RX: OnceLock<Mutex<tokio::sync::mpsc::UnboundedReceiver<MainCommand>>> =
    OnceLock::new();

/// Kirim perintah ke window utama (aman dipanggil dari dom mana pun).
pub fn send_main_command(cmd: MainCommand) {
    let tx = MAIN_CMD_TX.get_or_init(|| {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let _ = MAIN_CMD_RX.set(Mutex::new(rx));
        tx
    });
    let _ = tx.send(cmd);
}

/// Ambil satu perintah yang tertunda (dipoll window utama tiap 100 ms).
pub fn pop_main_command() -> Option<MainCommand> {
    MAIN_CMD_RX.get()?.lock().unwrap().try_recv().ok()
}

/// Handle tao Window milik widget (dibuat tersembunyi saat startup, di-show/hide sesuai setting).
pub static WIDGET_WINDOW: Mutex<Option<Arc<dioxus::desktop::tao::window::Window>>> =
    Mutex::new(None);

thread_local! {
    /// Handle CheckMenuItem "Widget" di tray. muda item bersifat !Send (Rc internal),
    /// tetapi handler tray dan setup selalu berjalan di main thread, jadi thread_local aman.
    pub static WIDGET_MENU_ITEM: std::cell::RefCell<Option<dioxus::desktop::trayicon::menu::CheckMenuItem>>
        = const { std::cell::RefCell::new(None) };

    /// Handle item "Tampilkan Jendela" di tray — teksnya diganti saat bahasa UI berubah.
    pub static TRAY_SHOW_ITEM: std::cell::RefCell<Option<dioxus::desktop::trayicon::menu::MenuItem>>
        = const { std::cell::RefCell::new(None) };

    /// Handle item "Keluar" di tray — teksnya diganti saat bahasa UI berubah.
    pub static TRAY_QUIT_ITEM: std::cell::RefCell<Option<dioxus::desktop::trayicon::menu::MenuItem>>
        = const { std::cell::RefCell::new(None) };
}

pub fn settings() -> Arc<Settings> {
    let lock = SETTINGS.get_or_init(|| RwLock::new(Arc::new(crate::settings::load())));
    lock.read().expect("SETTINGS lock").clone()
}

/// Simpan settings baru: tulis global + simpan ke disk.
pub fn update_settings(new: Settings) {
    crate::settings::save(&new);
    let lock = SETTINGS.get_or_init(|| RwLock::new(Arc::new(crate::settings::load())));
    *lock.write().expect("SETTINGS lock") = Arc::new(new);
}
