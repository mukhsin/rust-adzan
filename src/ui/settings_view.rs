use super::icons;
use crate::autostart;
use crate::cities::{CITIES, find_city};
use crate::i18n::tr;
use crate::settings::{CalcMethod, Language, MadhabChoice, Settings};
use crate::state;
use dioxus::prelude::*;
use std::path::PathBuf;

const ZONA_WAKTU: [(&str, &str); 3] = [
    ("Asia/Jakarta", "WIB — Jakarta (UTC+7)"),
    ("Asia/Makassar", "WITA — Makassar (UTC+8)"),
    ("Asia/Jayapura", "WIT — Jayapura (UTC+9)"),
];

const SELECT: &str = "flex-1 max-w-[210px] bg-zinc-100 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg px-2 py-1.5 text-sm text-zinc-900 dark:text-zinc-100 outline-none focus:ring-2 ring-zinc-500/50 transition-colors";
const INPUT: &str = "w-24 bg-zinc-100 dark:bg-zinc-800 border border-zinc-200 dark:border-zinc-700 rounded-lg px-2 py-1.5 text-sm text-zinc-900 dark:text-zinc-100 text-right outline-none focus:ring-2 ring-zinc-500/50 transition-colors";
const BTN_GHOST: &str = "px-3 py-1.5 rounded-lg text-xs font-medium border border-zinc-200 dark:border-zinc-700 text-zinc-600 dark:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors";
const BTN_ICON: &str = "p-1.5 rounded-lg border border-zinc-200 dark:border-zinc-700 text-zinc-600 dark:text-zinc-300 hover:bg-zinc-100 dark:hover:bg-zinc-800 transition-colors";
const BTN_CONFIRM: &str = "px-3 py-1.5 rounded-lg text-xs font-medium bg-zinc-900 text-zinc-50 dark:bg-zinc-100 dark:text-zinc-900 hover:opacity-90 transition-opacity";
const BTN_DANGER: &str = "px-3 py-1.5 rounded-lg text-xs font-medium bg-red-500 text-white hover:bg-red-600 transition-colors";

/// Jenis dialog modal yang sedang tampil.
#[derive(Debug, Clone, PartialEq)]
enum Dialog {
    None,
    ResetConfirm,
    AutostartError(String),
}

#[component]
pub fn SettingsView() -> Element {
    // Draft lokal; setiap perubahan langsung disimpan + disebarkan.
    let mut draft = use_signal(|| (*state::settings()).clone());
    let mut dialog = use_signal(|| Dialog::None);
    let lang = state::settings().language;
    let versi = tr(lang, "version_label").replace("{versi}", env!("CARGO_PKG_VERSION"));
    // Status pemutaran per slot (di-poll berkala agar ikon play/stop akurat).
    let mut playing_umum = use_signal(|| crate::audio::is_playing(crate::audio::Slot::Umum));
    let mut playing_subuh = use_signal(|| crate::audio::is_playing(crate::audio::Slot::Subuh));
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            let umum = crate::audio::is_playing(crate::audio::Slot::Umum);
            if *playing_umum.read() != umum {
                playing_umum.set(umum);
            }
            let subuh = crate::audio::is_playing(crate::audio::Slot::Subuh);
            if *playing_subuh.read() != subuh {
                playing_subuh.set(subuh);
            }
        }
    });

    let mut apply = move |s: Settings| {
        crate::settings::save(&s);
        state::update_settings(s.clone());
        draft.set(s);
    };

    rsx! {
        div { class: "flex-1 p-5 flex flex-col gap-4 overflow-y-auto",

            div { class: "text-xs uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-medium px-1 mt-1", "{tr(lang, \"sec_location\")}" }
            div { class: "bg-zinc-50 dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-xl p-4 flex flex-col gap-3",
                Field { label: tr(lang, "f_city"),
                    select {
                        class: "{SELECT}",
                        value: draft.read().city.clone().unwrap_or_default(),
                        onchange: move |e| {
                            let name = e.value();
                            if let Some(city) = find_city(&name) {
                                let mut s = draft.read().clone();
                                s.city = Some(city.name.to_string());
                                s.lat = city.lat;
                                s.lng = city.lng;
                                s.timezone = city.timezone.name().to_string();
                                apply(s);
                            }
                        },
                        option { value: "", "{tr(lang, \"city_manual\")}" }
                        for city in CITIES {
                            option { value: city.name, "{city.name}" }
                        }
                    }
                }
                Field { label: tr(lang, "f_lat"),
                    input {
                        class: "{INPUT}",
                        r#type: "text",
                        value: "{draft.read().lat:.4}",
                        onchange: move |e| {
                            if let Ok(lat) = e.value().trim().parse::<f64>() {
                                let mut s = draft.read().clone();
                                s.lat = lat;
                                s.city = None;
                                apply(s);
                            }
                        },
                    }
                }
                Field { label: tr(lang, "f_lng"),
                    input {
                        class: "{INPUT}",
                        r#type: "text",
                        value: "{draft.read().lng:.4}",
                        onchange: move |e| {
                            if let Ok(lng) = e.value().trim().parse::<f64>() {
                                let mut s = draft.read().clone();
                                s.lng = lng;
                                s.city = None;
                                apply(s);
                            }
                        },
                    }
                }
                Field { label: tr(lang, "f_tz"),
                    select {
                        class: "{SELECT}",
                        value: draft.read().timezone.clone(),
                        onchange: move |e| {
                            let tz = e.value();
                            if ZONA_WAKTU.iter().any(|(name, _)| *name == tz) {
                                let mut s = draft.read().clone();
                                s.set_timezone_name(&tz);
                                s.city = None;
                                apply(s);
                            }
                        },
                        for (name, label) in ZONA_WAKTU {
                            option { value: name, "{label}" }
                        }
                    }
                }
            }

            div { class: "text-xs uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-medium px-1", "{tr(lang, \"sec_calc\")}" }
            div { class: "bg-zinc-50 dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-xl p-4 flex flex-col gap-3",
                Field { label: tr(lang, "f_method"),
                    select {
                        class: "{SELECT}",
                        value: method_name(draft.read().method),
                        onchange: move |e| {
                            let v = e.value();
                            let mut s = draft.read().clone();
                            if let Some(m) = method_from_name(&v) {
                                s.method = m;
                                apply(s);
                            }
                        },
                        option { value: "kemenag", "{tr(lang, \"m_kemenag\")}" }
                        option { value: "mwl", "{tr(lang, \"m_mwl\")}" }
                        option { value: "egyptian", "{tr(lang, \"m_egyptian\")}" }
                        option { value: "karachi", "{tr(lang, \"m_karachi\")}" }
                        option { value: "umm_al_qura", "{tr(lang, \"m_umm_al_qura\")}" }
                        option { value: "north_america", "{tr(lang, \"m_isna\")}" }
                    }
                }
                Field { label: tr(lang, "f_madhab"),
                    select {
                        class: "{SELECT}",
                        value: match draft.read().madhab { MadhabChoice::Shafi => "shafi", MadhabChoice::Hanafi => "hanafi" },
                        onchange: move |e| {
                            let v = e.value();
                            let mut s = draft.read().clone();
                            s.madhab = if v == "hanafi" { MadhabChoice::Hanafi } else { MadhabChoice::Shafi };
                            apply(s);
                        },
                        option { value: "shafi", "{tr(lang, \"madhab_shafi\")}" }
                        option { value: "hanafi", "{tr(lang, \"madhab_hanafi\")}" }
                    }
                }
            }

            div { class: "text-xs uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-medium px-1", "{tr(lang, \"sec_audio\")}" }
            div { class: "bg-zinc-50 dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-xl p-4 flex flex-col gap-3",
                Field { label: tr(lang, "f_audio_enabled"),
                    Switch {
                        checked: draft.read().audio_enabled,
                        on_change: move |on| {
                            let mut s = draft.read().clone();
                            s.audio_enabled = on;
                            apply(s);
                        },
                    }
                }
                Field { label: tr(lang, "f_volume"), hint: "{(draft.read().volume * 100.0) as i32}%",
                    input {
                        class: "w-32 accent-zinc-900 dark:accent-zinc-100",
                        r#type: "range",
                        min: "0", max: "100",
                        value: "{(draft.read().volume * 100.0) as i32}",
                        oninput: move |e| {
                            if let Ok(v) = e.value().parse::<f64>() {
                                let mut s = draft.read().clone();
                                s.volume = (v / 100.0) as f32;
                                apply(s);
                            }
                        },
                    }
                }
                Field { label: tr(lang, "f_adzan_file"), hint: deskripsi_sumber(&draft.read(), false),
                    div { class: "flex items-center gap-1.5",
                        button {
                            class: "{BTN_GHOST}",
                            onclick: move |_| {
                                if let Some(p) = pick_audio(lang) {
                                    let mut s = draft.read().clone();
                                    s.adzan_file = Some(p);
                                    apply(s);
                                }
                            },
                            "{tr(lang, \"pick\")}"
                        }
                        button {
                            class: "{BTN_ICON}",
                            title: if *playing_umum.read() {
                                tr(lang, "stop_play")
                            } else {
                                tr(lang, "test_play")
                            },
                            onclick: move |_| {
                                if *playing_umum.read() {
                                    crate::audio::stop_slot(crate::audio::Slot::Umum);
                                } else {
                                    let s = state::settings();
                                    let _ = crate::audio::play_adzan(&s, false);
                                }
                            },
                            if *playing_umum.read() {
                                icons::IconSquare { class: "w-3.5 h-3.5" }
                            } else {
                                icons::IconPlay { class: "w-3.5 h-3.5" }
                            }
                        }
                    }
                }
                Field { label: tr(lang, "f_subuh_file"), hint: deskripsi_sumber(&draft.read(), true),
                    div { class: "flex items-center gap-1.5",
                        button {
                            class: "{BTN_GHOST}",
                            onclick: move |_| {
                                if let Some(p) = pick_audio(lang) {
                                    let mut s = draft.read().clone();
                                    s.subuh_file = Some(p);
                                    apply(s);
                                }
                            },
                            "{tr(lang, \"pick\")}"
                        }
                        button {
                            class: "{BTN_ICON}",
                            title: if *playing_subuh.read() {
                                tr(lang, "stop_play")
                            } else {
                                tr(lang, "test_play_subuh")
                            },
                            onclick: move |_| {
                                if *playing_subuh.read() {
                                    crate::audio::stop_slot(crate::audio::Slot::Subuh);
                                } else {
                                    let s = state::settings();
                                    let _ = crate::audio::play_adzan(&s, true);
                                }
                            },
                            if *playing_subuh.read() {
                                icons::IconSquare { class: "w-3.5 h-3.5" }
                            } else {
                                icons::IconPlay { class: "w-3.5 h-3.5" }
                            }
                        }
                    }
                }
            }

            div { class: "text-xs uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-medium px-1", "{tr(lang, \"sec_notif\")}" }
            div { class: "bg-zinc-50 dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-xl p-4 flex flex-col gap-3",
                Field { label: tr(lang, "f_notif_enabled"),
                    Switch {
                        checked: draft.read().notif_enabled,
                        on_change: move |on| {
                            let mut s = draft.read().clone();
                            s.notif_enabled = on;
                            apply(s);
                        },
                    }
                }
                Field { label: tr(lang, "f_pre_notif"), hint: tr(lang, "hint_pre_notif"),
                    div { class: "flex items-center gap-2",
                        button {
                            class: "{BTN_ICON}",
                            title: tr(lang, "dec"),
                            onclick: move |_| {
                                let mut s = draft.read().clone();
                                if s.pre_notif_minutes > 1 {
                                    s.pre_notif_minutes -= 1;
                                }
                                apply(s);
                            },
                            icons::IconMinus { class: "w-3.5 h-3.5" }
                        }
                        span { class: "text-center text-sm tabular-nums text-zinc-700 dark:text-zinc-300",
                            "{draft.read().pre_notif_minutes}"
                        }
                        button {
                            class: "{BTN_ICON}",
                            title: tr(lang, "inc"),
                            onclick: move |_| {
                                let mut s = draft.read().clone();
                                s.pre_notif_minutes = (s.pre_notif_minutes + 1).min(60);
                                apply(s);
                            },
                            icons::IconPlus { class: "w-3.5 h-3.5" }
                        }
                    }
                }
            }

            div { class: "text-xs uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-medium px-1", "{tr(lang, \"sec_misc\")}" }
            div { class: "bg-zinc-50 dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-xl p-4 flex flex-col gap-3",
                Field { label: tr(lang, "f_language"),
                    select {
                        class: "{SELECT}",
                        value: match draft.read().language { Language::Id => "id", Language::En => "en" },
                        onchange: move |e| {
                            let mut s = draft.read().clone();
                            s.language = if e.value() == "en" { Language::En } else { Language::Id };
                            apply(s);
                        },
                        option { value: "en", "English" }
                        option { value: "id", "Bahasa Indonesia" }
                    }
                }
                Field { label: tr(lang, "f_autostart"),
                    Switch {
                        checked: draft.read().autostart,
                        on_change: move |on| {
                            match autostart::set_enabled(on) {
                                Ok(()) => {
                                    let mut s = draft.read().clone();
                                    s.autostart = on;
                                    apply(s);
                                }
                                Err(err) => {
                                    dialog.set(Dialog::AutostartError(err));
                                }
                            }
                        },
                    }
                }
            }

            div { class: "flex flex-col items-center gap-2 pt-1 pb-2",
                button {
                    class: "px-3 py-1.5 rounded-lg text-xs font-medium border border-red-300 dark:border-red-900/60 text-red-500 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-950/40 transition-colors",
                    onclick: move |_| dialog.set(Dialog::ResetConfirm),
                    "{tr(lang, \"reset_btn\")}"
                }
                span { class: "text-sm text-zinc-400 dark:text-zinc-600 select-none", "{versi}" }
            }

            {match dialog.read().clone() {
                Dialog::None => rsx! {},
                Dialog::ResetConfirm => rsx! {
                    Modal {
                        title: tr(lang, "reset_btn"),
                        message: tr(lang, "modal_reset_msg"),
                        confirm_text: tr(lang, "confirm_reset"),
                        danger: true,
                        show_cancel: true,
                        on_confirm: move |_| {
                            let _ = autostart::set_enabled(false);
                            let s = Settings::default();
                            state::update_settings(s.clone());
                            draft.set(s);
                            dialog.set(Dialog::None);
                        },
                        on_cancel: move |_| dialog.set(Dialog::None),
                    }
                },
                Dialog::AutostartError(err) => rsx! {
                    Modal {
                        title: tr(lang, "autostart_title"),
                        message: err.clone(),
                        confirm_text: "OK",
                        danger: false,
                        show_cancel: false,
                        on_confirm: move |_| dialog.set(Dialog::None),
                        on_cancel: move |_| dialog.set(Dialog::None),
                    }
                },
            }}
        }
    }
}

/// Modal dialog bertema aplikasi (pengganti dialog OS yang jelek di tema gelap).
/// Klik backdrop atau tombol batal = tutup tanpa aksi.
#[component]
fn Modal(
    title: String,
    message: String,
    confirm_text: String,
    danger: bool,
    show_cancel: bool,
    on_confirm: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let lang = state::settings().language;
    rsx! {
        div {
            class: "fixed inset-0 z-50 flex items-center justify-center bg-black/60",
            onclick: move |_| on_cancel.call(()),
            div {
                class: "w-80 bg-white dark:bg-zinc-950 border border-zinc-200 dark:border-zinc-800 rounded-2xl p-5 shadow-2xl flex flex-col gap-2",
                onclick: move |e| e.stop_propagation(),
                h2 { class: "text-sm font-semibold text-zinc-900 dark:text-zinc-50", "{title}" }
                p { class: "text-xs text-zinc-500 dark:text-zinc-400 leading-relaxed", "{message}" }
                div { class: "flex justify-end gap-2 mt-3",
                    if show_cancel {
                        button {
                            class: "{BTN_GHOST}",
                            onclick: move |_| on_cancel.call(()),
                            "{tr(lang, \"cancel\")}"
                        }
                    }
                    button {
                        class: if danger { "{BTN_DANGER}" } else { "{BTN_CONFIRM}" },
                        onclick: move |_| on_confirm.call(()),
                        "{confirm_text}"
                    }
                }
            }
        }
    }
}

/// Baris setting: label (+ hint) di kiri, kontrol di kanan.
#[component]
fn Field(label: &'static str, hint: Option<String>, children: Element) -> Element {
    rsx! {
        div { class: "flex items-center justify-between gap-3",
            div { class: "flex flex-col",
                label { class: "text-sm text-zinc-700 dark:text-zinc-300", "{label}" }
                if let Some(h) = hint {
                    span { class: "text-xs text-zinc-400 dark:text-zinc-500", "{h}" }
                }
            }
            {children}
        }
    }
}

/// Toggle switch minimalis (aksen zinc senada tema).
#[component]
fn Switch(checked: bool, on_change: EventHandler<bool>) -> Element {
    rsx! {
        button {
            class: if checked {
                "w-9 h-5 rounded-full bg-zinc-900 dark:bg-zinc-100 transition-colors relative shrink-0"
            } else {
                "w-9 h-5 rounded-full bg-zinc-300 dark:bg-zinc-700 transition-colors relative shrink-0"
            },
            onclick: move |_| on_change.call(!checked),
            span { class: if checked {
                "absolute top-0.5 left-[18px] w-4 h-4 rounded-full bg-white dark:bg-zinc-900 shadow transition-all"
            } else {
                "absolute top-0.5 left-0.5 w-4 h-4 rounded-full bg-white dark:bg-zinc-400 shadow transition-all"
            } }
        }
    }
}

fn method_name(m: CalcMethod) -> &'static str {
    match m {
        CalcMethod::Kemenag => "kemenag",
        CalcMethod::MuslimWorldLeague => "mwl",
        CalcMethod::Egyptian => "egyptian",
        CalcMethod::Karachi => "karachi",
        CalcMethod::UmmAlQura => "umm_al_qura",
        CalcMethod::NorthAmerica => "north_america",
    }
}

fn method_from_name(s: &str) -> Option<CalcMethod> {
    match s {
        "kemenag" => Some(CalcMethod::Kemenag),
        "mwl" => Some(CalcMethod::MuslimWorldLeague),
        "egyptian" => Some(CalcMethod::Egyptian),
        "karachi" => Some(CalcMethod::Karachi),
        "umm_al_qura" => Some(CalcMethod::UmmAlQura),
        "north_america" => Some(CalcMethod::NorthAmerica),
        _ => None,
    }
}

fn pick_audio(lang: Language) -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Audio", &["mp3", "wav", "ogg", "flac", "m4a", "aac"])
        .set_title(tr(lang, "pick_audio_title"))
        .pick_file()
}

/// Deskripsi sumber audio aktif saat ini (kustom / file / bawaan ter-embed)
/// untuk ditampilkan di settings. `subuh` memilih baris mana yang dideskripsi.
fn deskripsi_sumber(s: &Settings, subuh: bool) -> String {
    let lang = s.language;
    let resolved = if subuh {
        crate::audio::resolve_subuh(s)
    } else {
        crate::audio::resolve_adzan(s)
    };
    match resolved {
        Some(p) => {
            // Cukup nama file — full path terlalu panjang untuk card yang sempit.
            let nama = p
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| p.display().to_string());
            let kustom = if subuh {
                s.subuh_file.as_ref() == Some(&p)
            } else {
                s.adzan_file.as_ref() == Some(&p)
            };
            if kustom {
                tr(lang, "src_custom").replace("{nama}", &nama)
            } else {
                tr(lang, "src_file").replace("{nama}", &nama)
            }
        }
        None => tr(lang, "src_embedded").to_string(),
    }
}
