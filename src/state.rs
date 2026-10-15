use crate::settings::Settings;
use std::sync::{Arc, Mutex, OnceLock, RwLock};

/// Settings global yang dibagikan antara UI utama, widget, dan scheduler thread.
static SETTINGS: OnceLock<RwLock<Arc<Settings>>> = OnceLock::new();

/// Handle tao Window milik widget (dibuat tersembunyi saat startup, di-show/hide sesuai setting).
pub static WIDGET_WINDOW: Mutex<Option<Arc<dioxus::desktop::tao::window::Window>>> =
    Mutex::new(None);

thread_local! {
    /// Handle CheckMenuItem "Widget" di tray. muda item bersifat !Send (Rc internal),
    /// tetapi handler tray dan setup selalu berjalan di main thread, jadi thread_local aman.
    pub static WIDGET_MENU_ITEM: std::cell::RefCell<Option<dioxus::desktop::trayicon::menu::CheckMenuItem>>
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
