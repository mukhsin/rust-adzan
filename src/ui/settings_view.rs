//! Window pengaturan — restyle dari design system "Jadwal Sholat
//! Impeccable.pen": kartu grup putih (radius 14 + shadow lembut) di atas
//! lembar hangat, baris setting setinggi 58px berpemisah hairline, kontrol
//! Dropdown / Stepper / Sakelar / Tersegmen senada widget.
//!
//! Satu grup = satu komponen (`*Group`), tersusun dua kolom di
//! `SettingsView`. Perubahan nilai lewat [`apply_settings`]: persist ke
//! disk + sebar ke dom lain + refresh draft.

use super::icons;
use crate::autostart;
use crate::cities::{CITIES, find_city};
use crate::i18n::tr;
use crate::settings::{CalcMethod, Language, MadhabChoice, Settings, Theme};
use crate::state;
use dioxus::prelude::*;
use std::path::PathBuf;

const ZONA_WAKTU: [(&str, &str); 3] = [
    ("Asia/Jakarta", "WIB — Jakarta (UTC+7)"),
    ("Asia/Makassar", "WITA — Makassar (UTC+8)"),
    ("Asia/Jayapura", "WIT — Jayapura (UTC+9)"),
];

const SELECT: &str = "max-w-[210px] bg-pass border border-pass-line rounded-lg px-3 py-2 text-[13px] font-semibold text-ink outline-none focus:border-ink-3 transition-colors";
const INPUT: &str = "w-24 bg-pass border border-pass-line rounded-lg px-3 py-2 text-[13px] font-semibold text-ink text-right outline-none focus:border-ink-3 transition-colors tabular-nums";
// Tombol sekunder ala komponen "Tombol Sekunder" (pass + garis pass-line).
const BTN: &str = "flex items-center gap-2 bg-pass border border-pass-line rounded-lg px-3 py-2 text-[13px] font-semibold text-ink hover:border-ink-3 transition-colors cursor-pointer";
const BTN_ICON: &str = "p-2 bg-pass border border-pass-line rounded-lg text-ink-2 hover:border-ink-3 transition-colors cursor-pointer";
const BTN_CONFIRM: &str = "px-3 py-2 rounded-lg text-[13px] font-semibold bg-ink text-on-ink hover:opacity-90 transition-opacity cursor-pointer";
const BTN_DANGER: &str = "px-3 py-2 rounded-lg text-[13px] font-semibold bg-red-500 text-white hover:bg-red-600 transition-colors cursor-pointer";

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
    // Status pemutaran per slot (di-poll berkala agar ikon play/stop akurat);
    // sinyalnya dipakai komponen AudioGroup.
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

    rsx! {
        div { class: "flex-1 px-4 flex flex-col gap-[18px] overflow-y-auto",

            // Grup pengaturan dua kolom ala frame "Pengaturan" di design
            // system: kiri LOKASI + AUDIO, kanan PERHITUNGAN + TAMPILAN +
            // NOTIFIKASI + LAINNYA. items-start = kartu tak dipaksa sama
            // tinggi, persis perilaku Baris Grup di pen.
            div { class: "flex items-start gap-[18px]",
                div { class: "flex-1 flex flex-col gap-[18px]",
                    LocationGroup { draft }
                    CalculationGroup { draft }
                    AppearanceGroup { draft }
                }
                div { class: "flex-1 flex flex-col gap-[18px]",
                    AudioGroup { draft, playing_umum, playing_subuh }
                    NotificationGroup { draft }
                    MiscGroup { draft,
                        on_autostart_error: move |err: String| dialog.set(Dialog::AutostartError(err)),
                    }
                }
            }

            // Reset dipindah ke sini dari footer lama; versi kini tampil di
            // layar Tentang.
            div { class: "flex items-center justify-center pt-1 pb-2",
                button {
                    class: "px-3 py-2 rounded-lg text-[13px] font-semibold text-red-500 border border-pass-line hover:border-red-400 transition-colors cursor-pointer",
                    onclick: move |_| dialog.set(Dialog::ResetConfirm),
                    "{tr(lang, \"reset_btn\")}"
                }
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

/// Simpan perubahan pengaturan dari form: persist ke disk, sebar ke semua
/// dom (widget, tray), lalu refresh draft agar tampilan mengikuti.
fn apply_settings(mut draft: Signal<Settings>, s: Settings) {
    crate::settings::save(&s);
    state::update_settings(s.clone());
    draft.set(s);
}

/// Grup LOKASI: kota, koordinat manual, zona waktu.
#[component]
fn LocationGroup(draft: Signal<Settings>) -> Element {
    let lang = state::settings().language;

    rsx! {
        Group { label: tr(lang, "sec_location"),
            Row { label: tr(lang, "f_city"),
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
                            apply_settings(draft, s);
                        }
                    },
                    option { value: "", "{tr(lang, \"city_manual\")}" }
                    for city in CITIES {
                        option { value: city.name, "{city.name}" }
                    }
                }
            }
            Row { label: tr(lang, "f_lat"),
                input {
                    class: "{INPUT}",
                    r#type: "text",
                    value: "{draft.read().lat:.4}",
                    onchange: move |e| {
                        if let Ok(lat) = e.value().trim().parse::<f64>() {
                            let mut s = draft.read().clone();
                            s.lat = lat;
                            s.city = None;
                            apply_settings(draft, s);
                        }
                    },
                }
            }
            Row { label: tr(lang, "f_lng"),
                input {
                    class: "{INPUT}",
                    r#type: "text",
                    value: "{draft.read().lng:.4}",
                    onchange: move |e| {
                        if let Ok(lng) = e.value().trim().parse::<f64>() {
                            let mut s = draft.read().clone();
                            s.lng = lng;
                            s.city = None;
                            apply_settings(draft, s);
                        }
                    },
                }
            }
            Row { label: tr(lang, "f_tz"),
                select {
                    class: "{SELECT}",
                    value: draft.read().timezone.clone(),
                    onchange: move |e| {
                        let tz = e.value();
                        if ZONA_WAKTU.iter().any(|(name, _)| *name == tz) {
                            let mut s = draft.read().clone();
                            s.set_timezone_name(&tz);
                            s.city = None;
                            apply_settings(draft, s);
                        }
                    },
                    for (name, label) in ZONA_WAKTU {
                        option { value: name, "{label}" }
                    }
                }
            }
        }
    }
}

/// Grup PERHITUNGAN: metode perhitungan + mazhab ashar.
#[component]
fn CalculationGroup(draft: Signal<Settings>) -> Element {
    let lang = state::settings().language;

    rsx! {
        Group { label: tr(lang, "sec_calc"),
            Row { label: tr(lang, "f_method"),
                select {
                    class: "{SELECT}",
                    value: method_name(draft.read().method),
                    onchange: move |e| {
                        let v = e.value();
                        let mut s = draft.read().clone();
                        if let Some(m) = method_from_name(&v) {
                            s.method = m;
                            apply_settings(draft, s);
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
            Row { label: tr(lang, "f_madhab"),
                select {
                    class: "{SELECT}",
                    value: match draft.read().madhab {
                        MadhabChoice::Shafi => "shafi",
                        MadhabChoice::Hanafi => "hanafi",
                    },
                    onchange: move |e| {
                        let v = e.value();
                        let mut s = draft.read().clone();
                        s.madhab = if v == "hanafi" { MadhabChoice::Hanafi } else { MadhabChoice::Shafi };
                        apply_settings(draft, s);
                    },
                    option { value: "shafi", "{tr(lang, \"madhab_shafi\")}" }
                    option { value: "hanafi", "{tr(lang, \"madhab_hanafi\")}" }
                }
            }
        }
    }
}

/// Grup AUDIO ADZAN: sakelar aktif, volume, file adzan umum & khusus Subuh
/// (pilih file + tes putar/stop per slot).
#[component]
fn AudioGroup(
    draft: Signal<Settings>,
    playing_umum: Signal<bool>,
    playing_subuh: Signal<bool>,
) -> Element {
    let lang = state::settings().language;

    rsx! {
        Group { label: tr(lang, "sec_audio"),
            Row { label: tr(lang, "f_audio_enabled"),
                Switch {
                    checked: draft.read().audio_enabled,
                    on_change: move |on| {
                        let mut s = draft.read().clone();
                        s.audio_enabled = on;
                        apply_settings(draft, s);
                    },
                }
            }
            Row { label: tr(lang, "f_volume"), hint: "{(draft.read().volume * 100.0) as i32}%",
                input {
                    class: "w-32 accent-ink cursor-pointer",
                    r#type: "range",
                    min: "0", max: "100",
                    value: "{(draft.read().volume * 100.0) as i32}",
                    oninput: move |e| {
                        if let Ok(v) = e.value().parse::<f64>() {
                            let mut s = draft.read().clone();
                            s.volume = (v / 100.0) as f32;
                            apply_settings(draft, s);
                        }
                    },
                }
            }
            Row { label: tr(lang, "f_adzan_file"), hint: deskripsi_sumber(&draft.read(), false),
                div { class: "flex items-center gap-2",
                    button {
                        class: "{BTN}",
                        onclick: move |_| {
                            if let Some(p) = pick_audio(lang) {
                                let mut s = draft.read().clone();
                                s.adzan_file = Some(p);
                                apply_settings(draft, s);
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
                            icons::IconSquare { class: "w-[13px] h-[13px]" }
                        } else {
                            icons::IconPlay { class: "w-[13px] h-[13px]" }
                        }
                    }
                }
            }
            Row { label: tr(lang, "f_subuh_file"), hint: deskripsi_sumber(&draft.read(), true),
                div { class: "flex items-center gap-2",
                    button {
                        class: "{BTN}",
                        onclick: move |_| {
                            if let Some(p) = pick_audio(lang) {
                                let mut s = draft.read().clone();
                                s.subuh_file = Some(p);
                                apply_settings(draft, s);
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
                            icons::IconSquare { class: "w-[13px] h-[13px]" }
                        } else {
                            icons::IconPlay { class: "w-[13px] h-[13px]" }
                        }
                    }
                }
            }
        }
    }
}

/// Grup NOTIFIKASI: sakelar toast + pengingat X menit sebelum adzan.
#[component]
fn NotificationGroup(draft: Signal<Settings>) -> Element {
    let lang = state::settings().language;

    rsx! {
        Group { label: tr(lang, "sec_notif"),
            Row { label: tr(lang, "f_notif_enabled"), hint: tr(lang, "hint_notif_toast"),
                Switch {
                    checked: draft.read().notif_enabled,
                    on_change: move |on| {
                        let mut s = draft.read().clone();
                        s.notif_enabled = on;
                        apply_settings(draft, s);
                    },
                }
            }
            Row { label: tr(lang, "f_pre_notif"), hint: tr(lang, "hint_pre_notif"),
                Stepper {
                    value: draft.read().pre_notif_minutes,
                    on_decrease: move |_| {
                        let mut s = draft.read().clone();
                        if s.pre_notif_minutes > 1 {
                            s.pre_notif_minutes -= 1;
                        }
                        apply_settings(draft, s);
                    },
                    on_increase: move |_| {
                        let mut s = draft.read().clone();
                        s.pre_notif_minutes = (s.pre_notif_minutes + 1).min(60);
                        apply_settings(draft, s);
                    },
                }
            }
        }
    }
}

/// Grup TAMPILAN: tema (ikut sistem / gelap / terang) + format jam 24/12.
#[component]
fn AppearanceGroup(draft: Signal<Settings>) -> Element {
    let lang = state::settings().language;

    rsx! {
        Group { label: tr(lang, "sec_appearance"),
            Row { label: tr(lang, "f_theme"),
                Segmented {
                    options: vec![
                        ("system".to_string(), tr(lang, "theme_system").to_string()),
                        ("dark".to_string(), tr(lang, "theme_dark").to_string()),
                        ("light".to_string(), tr(lang, "theme_light").to_string()),
                    ],
                    active: match draft.read().theme {
                        Theme::System => "system",
                        Theme::Dark => "dark",
                        Theme::Light => "light",
                    }
                    .to_string(),
                    on_select: move |v: String| {
                        let mut s = draft.read().clone();
                        s.theme = match v.as_str() {
                            "dark" => Theme::Dark,
                            "light" => Theme::Light,
                            _ => Theme::System,
                        };
                        apply_settings(draft, s);
                    },
                }
            }
            Row { label: tr(lang, "f_time_format"),
                Segmented {
                    options: vec![
                        ("24".to_string(), tr(lang, "fmt_24").to_string()),
                        ("12".to_string(), tr(lang, "fmt_12").to_string()),
                    ],
                    active: if draft.read().time_format_24 {
                        "24".to_string()
                    } else {
                        "12".to_string()
                    },
                    on_select: move |v: String| {
                        let mut s = draft.read().clone();
                        s.time_format_24 = v == "24";
                        apply_settings(draft, s);
                    },
                }
            }
        }
    }
}

/// Grup LAINNYA: bahasa UI + auto-start saat Windows menyala.
#[component]
fn MiscGroup(draft: Signal<Settings>, on_autostart_error: EventHandler<String>) -> Element {
    let lang = state::settings().language;

    rsx! {
        Group { label: tr(lang, "sec_misc"),
            Row { label: tr(lang, "f_language"),
                Segmented {
                    options: vec![
                        ("id".to_string(), "Indonesia".to_string()),
                        ("en".to_string(), "English".to_string()),
                    ],
                    active: match draft.read().language {
                        Language::Id => "id",
                        Language::En => "en",
                    }
                    .to_string(),
                    on_select: move |v: String| {
                        let mut s = draft.read().clone();
                        s.language = if v == "en" { Language::En } else { Language::Id };
                        apply_settings(draft, s);
                    },
                }
            }
            Row { label: tr(lang, "f_autostart"),
                Switch {
                    checked: draft.read().autostart,
                    on_change: move |on| match autostart::set_enabled(on) {
                        Ok(()) => {
                            let mut s = draft.read().clone();
                            s.autostart = on;
                            apply_settings(draft, s);
                        }
                        Err(err) => on_autostart_error.call(err),
                    },
                }
            }
        }
    }
}

/// Kartu grup setting ala komponen "Grup Pengaturan": pass, radius 14,
/// shadow lembut, label mono uppercase ink-3, padding [14, 26, 10, 26].
#[component]
fn Group(label: &'static str, children: Element) -> Element {
    rsx! {
        div { class: "bg-pass rounded-[14px] shadow-[0_8px_18px_#14161a1a] dark:shadow-[0_8px_18px_#00000080] px-[26px] pt-3.5 pb-2.5 flex flex-col",
            div { class: "font-mono text-[11px] font-semibold tracking-[0.14em] uppercase text-ink-3 pb-1",
                "{label}"
            }
            {children}
        }
    }
}

/// Baris setting ala komponen "Baris Pengaturan": kiri label + keterangan,
/// kanan kontrol; pemisah hairline bawah (baris terakhir tanpa garis).
/// Tinggi baris DIBUAT SERAGAM (58px) agar grup bersebelahan di kolom
/// kiri/kanan sejajar.
#[component]
fn Row(label: &'static str, hint: Option<String>, children: Element) -> Element {
    rsx! {
        div { class: "flex items-center justify-between gap-3 h-[58px] shrink-0 border-b border-hairline last:border-b-0",
            div { class: "flex flex-col gap-0.5 min-w-0",
                span { class: "text-[13.5px] font-semibold text-ink", "{label}" }
                if let Some(h) = hint {
                    span { class: "text-[11.5px] text-ink-3 truncate", "{h}" }
                }
            }
            div { class: "flex items-center gap-2 shrink-0",
                {children}
            }
        }
    }
}

/// Sakelar ala komponen "Sakelar" 36×20: isian ink saat nyala, kenop
/// warna on-ink; mati = isian redup dengan kenop pass.
#[component]
fn Switch(checked: bool, on_change: EventHandler<bool>) -> Element {
    rsx! {
        button {
            class: if checked {
                "w-9 h-5 rounded-full bg-ink transition-colors relative shrink-0 cursor-pointer"
            } else {
                "w-9 h-5 rounded-full bg-ink-3/40 transition-colors relative shrink-0 cursor-pointer"
            },
            onclick: move |_| on_change.call(!checked),
            span { class: if checked {
                "absolute top-[2px] left-[18px] w-4 h-4 rounded-full bg-on-ink shadow transition-all"
            } else {
                "absolute top-[2px] left-[2px] w-4 h-4 rounded-full bg-pass shadow transition-all"
            } }
        }
    }
}

/// Stepper ala komponen "Stepper": − nilai + dalam kotak bergaris pass-line.
#[component]
fn Stepper(value: u32, on_decrease: EventHandler<()>, on_increase: EventHandler<()>) -> Element {
    let lang = state::settings().language;
    rsx! {
        div { class: "flex items-center gap-2.5 border border-pass-line rounded-lg px-2.5 py-[5px]",
            button {
                class: "font-mono text-[13px] font-semibold text-ink-2 hover:text-ink transition-colors cursor-pointer",
                title: tr(lang, "dec"),
                onclick: move |_| on_decrease.call(()),
                "−"
            }
            span { class: "font-mono text-xs font-semibold text-ink tabular-nums text-center min-w-[64px]",
                "{value} {tr(lang, \"unit_minutes\")}"
            }
            button {
                class: "font-mono text-[13px] font-semibold text-ink-2 hover:text-ink transition-colors cursor-pointer",
                title: tr(lang, "inc"),
                onclick: move |_| on_increase.call(()),
                "+"
            }
        }
    }
}

/// Tersegmen ala komponen "Tersegmen": lajur board pill, segmen aktif ink
/// dengan teks on-ink. Dipakai untuk pilihan pendek (tema, format jam,
/// bahasa).
#[component]
fn Segmented(
    options: Vec<(String, String)>,
    active: String,
    on_select: EventHandler<String>,
) -> Element {
    rsx! {
        div { class: "flex items-center gap-0.5 bg-board rounded-full p-[3px]",
            for (value, label) in options {
                {
                    let is_active = value == active;
                    rsx! {
                        button {
                            class: if is_active {
                                "px-3.5 py-[5px] rounded-full bg-ink text-on-ink text-[12.5px] font-bold transition-colors cursor-pointer"
                            } else {
                                "px-3.5 py-[5px] rounded-full text-[12.5px] font-semibold text-ink-2 hover:text-ink transition-colors cursor-pointer"
                            },
                            onclick: move |_| on_select.call(value.clone()),
                            "{label}"
                        }
                    }
                }
            }
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
            class: "fixed inset-0 z-50 flex items-center justify-center bg-ink/60",
            onclick: move |_| on_cancel.call(()),
            div {
                class: "w-80 bg-pass border border-pass-line rounded-[14px] p-5 shadow-[0_8px_18px_#14161a1a] dark:shadow-[0_8px_18px_#00000080] flex flex-col gap-2",
                onclick: move |e| e.stop_propagation(),
                h2 { class: "text-sm font-semibold text-ink", "{title}" }
                p { class: "text-xs text-ink-2 leading-relaxed", "{message}" }
                div { class: "flex justify-end gap-2 mt-3",
                    if show_cancel {
                        button {
                            class: "{BTN}",
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
