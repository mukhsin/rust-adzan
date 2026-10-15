use super::icons;
use crate::autostart;
use crate::cities::{CITIES, find_city};
use crate::settings::{CalcMethod, MadhabChoice, Settings};
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

#[component]
pub fn SettingsView() -> Element {
    // Draft lokal; setiap perubahan langsung disimpan + disebarkan.
    let mut draft = use_signal(|| (*state::settings()).clone());
    let mut status = use_signal(String::new);
    // Status pemutaran audio (di-refresh berkala agar ikon play/stop akurat).
    let mut playing = use_signal(crate::audio::is_playing);
    use_future(move || async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
            let now = crate::audio::is_playing();
            if *playing.read() != now {
                playing.set(now);
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

            div { class: "text-xs uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-medium px-1 mt-1", "Lokasi" }
            div { class: "bg-zinc-50 dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-xl p-4 flex flex-col gap-3",
                Field { label: "Kota",
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
                        option { value: "", "— Koordinat manual —" }
                        for city in CITIES {
                            option { value: city.name, "{city.name}" }
                        }
                    }
                }
                Field { label: "Latitude",
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
                Field { label: "Longitude",
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
                Field { label: "Zona Waktu",
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

            div { class: "text-xs uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-medium px-1", "Perhitungan" }
            div { class: "bg-zinc-50 dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-xl p-4 flex flex-col gap-3",
                Field { label: "Metode",
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
                        option { value: "kemenag", "Kemenag (Subuh 20°, Isya 18°)" }
                        option { value: "mwl", "Muslim World League" }
                        option { value: "egyptian", "Egyptian General Authority" }
                        option { value: "karachi", "Universitas Karachi" }
                        option { value: "umm_al_qura", "Umm al-Qura, Makkah" }
                        option { value: "north_america", "ISNA (Amerika Utara)" }
                    }
                }
                Field { label: "Mazhab (Ashar)",
                    select {
                        class: "{SELECT}",
                        value: match draft.read().madhab { MadhabChoice::Shafi => "shafi", MadhabChoice::Hanafi => "hanafi" },
                        onchange: move |e| {
                            let v = e.value();
                            let mut s = draft.read().clone();
                            s.madhab = if v == "hanafi" { MadhabChoice::Hanafi } else { MadhabChoice::Shafi };
                            apply(s);
                        },
                        option { value: "shafi", "Syafi'i / Maliki / Hambali" }
                        option { value: "hanafi", "Hanafi" }
                    }
                }
            }

            div { class: "text-xs uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-medium px-1", "Audio Adzan" }
            div { class: "bg-zinc-50 dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-xl p-4 flex flex-col gap-3",
                Field { label: "Mainkan adzan saat masuk waktu",
                    Switch {
                        checked: draft.read().audio_enabled,
                        on_change: move |on| {
                            let mut s = draft.read().clone();
                            s.audio_enabled = on;
                            apply(s);
                        },
                    }
                }
                Field { label: "Volume", hint: "{(draft.read().volume * 100.0) as i32}%",
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
                Field { label: "File adzan", hint: "Kosong = adzan bawaan aplikasi",
                    div { class: "flex items-center gap-1.5",
                        button {
                            class: "{BTN_GHOST}",
                            onclick: move |_| {
                                if let Some(p) = pick_audio() {
                                    let mut s = draft.read().clone();
                                    s.adzan_file = Some(p);
                                    apply(s);
                                }
                            },
                            "Pilih…"
                        }
                        button {
                            class: "{BTN_ICON}",
                            title: if *playing.read() {
                                "Hentikan pemutaran"
                            } else {
                                "Tes putar adzan"
                            },
                            onclick: move |_| {
                                if *playing.read() {
                                    crate::audio::stop_current();
                                    status.set("⏹ pemutaran dihentikan".into());
                                } else {
                                    let s = state::settings();
                                    match crate::audio::play_adzan(&s, false) {
                                        Ok(_) => status.set("▶ memutar adzan…".into()),
                                        Err(e) => status.set(format!("✕ {e}")),
                                    }
                                }
                            },
                            if *playing.read() {
                                icons::IconSquare { class: "w-3.5 h-3.5" }
                            } else {
                                icons::IconPlay { class: "w-3.5 h-3.5" }
                            }
                        }
                    }
                }
                Field { label: "File khusus Subuh", hint: "Opsional; kosong = adzan Subuh bawaan",
                    div { class: "flex items-center gap-1.5",
                        button {
                            class: "{BTN_GHOST}",
                            onclick: move |_| {
                                if let Some(p) = pick_audio() {
                                    let mut s = draft.read().clone();
                                    s.subuh_file = Some(p);
                                    apply(s);
                                }
                            },
                            "Pilih…"
                        }
                        button {
                            class: "{BTN_ICON}",
                            title: if *playing.read() {
                                "Hentikan pemutaran"
                            } else {
                                "Tes putar adzan Subuh"
                            },
                            onclick: move |_| {
                                if *playing.read() {
                                    crate::audio::stop_current();
                                    status.set("⏹ pemutaran dihentikan".into());
                                } else {
                                    let s = state::settings();
                                    match crate::audio::play_adzan(&s, true) {
                                        Ok(_) => status.set("▶ memutar adzan subuh…".into()),
                                        Err(e) => status.set(format!("✕ {e}")),
                                    }
                                }
                            },
                            if *playing.read() {
                                icons::IconSquare { class: "w-3.5 h-3.5" }
                            } else {
                                icons::IconPlay { class: "w-3.5 h-3.5" }
                            }
                        }
                    }
                }
            }

            div { class: "text-xs uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-medium px-1", "Notifikasi" }
            div { class: "bg-zinc-50 dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-xl p-4 flex flex-col gap-3",
                Field { label: "Tampilkan toast notifikasi",
                    Switch {
                        checked: draft.read().notif_enabled,
                        on_change: move |on| {
                            let mut s = draft.read().clone();
                            s.notif_enabled = on;
                            apply(s);
                        },
                    }
                }
                Field { label: "Pengingat sebelum adzan", hint: "Menit sebelum waktu masuk (0 = nonaktif)",
                    div { class: "flex items-center gap-1",
                        button {
                            class: "{BTN_ICON}",
                            title: "Kurangi",
                            onclick: move |_| {
                                let mut s = draft.read().clone();
                                s.pre_notif_minutes = s.pre_notif_minutes.saturating_sub(1);
                                apply(s);
                            },
                            icons::IconMinus { class: "w-3.5 h-3.5" }
                        }
                        span { class: "w-12 text-center text-sm tabular-nums text-zinc-700 dark:text-zinc-300",
                            "{draft.read().pre_notif_minutes} menit"
                        }
                        button {
                            class: "{BTN_ICON}",
                            title: "Tambah",
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

            div { class: "text-xs uppercase tracking-wider text-zinc-400 dark:text-zinc-500 font-medium px-1", "Lainnya" }
            div { class: "bg-zinc-50 dark:bg-zinc-900/50 border border-zinc-200 dark:border-zinc-800 rounded-xl p-4 flex flex-col gap-3",
                Field { label: "Jalankan saat Windows menyala",
                    Switch {
                        checked: draft.read().autostart,
                        on_change: move |on| {
                            match autostart::set_enabled(on) {
                                Ok(()) => {
                                    let mut s = draft.read().clone();
                                    s.autostart = on;
                                    apply(s);
                                }
                                Err(err) => status.set(format!("✕ {err}")),
                            }
                        },
                    }
                }
            }

            if !status.read().is_empty() {
                div { class: "text-xs text-zinc-400 dark:text-zinc-500 px-1 pb-2", "{status.read()}" }
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

fn pick_audio() -> Option<PathBuf> {
    rfd::FileDialog::new()
        .add_filter("Audio", &["mp3", "wav", "ogg", "flac", "m4a", "aac"])
        .set_title("Pilih file adzan")
        .pick_file()
}
