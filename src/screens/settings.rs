//! Layar Pengaturan — enam grup komponen tersusun dua kolom ala frame
//! "Pengaturan" design system pen. Satu grup = satu komponen di file ini;
//! perubahan nilai lewat [`apply_settings`] (persist + sebar + refresh
//! draft).

use crate::autostart;
use crate::cities::{CITIES, find_city};
use crate::components::ui::button::{BTN, BTN_ICON};
use crate::components::ui::drop_search::DropSearch;
use crate::components::ui::form::INPUT;
use crate::components::ui::group::{Group, Row};
use crate::components::ui::icons;
use crate::components::ui::modal::Modal;
use crate::components::ui::segmented::Segmented;
use crate::components::ui::stepper::{OffsetStepper, Stepper};
use crate::components::ui::switch::Switch;
use crate::components::ui::tooltip::{PosisiTooltip, Tooltip};
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
        div { class: "flex-1 p-4 flex flex-col gap-[18px] overflow-y-auto",

            // Grup pengaturan dua kolom ala frame "Pengaturan" di design
            // system: kiri LOKASI + AUDIO, kanan PERHITUNGAN + TAMPILAN +
            // NOTIFIKASI + LAINNYA. items-start = kartu tak dipaksa sama
            // tinggi, persis perilaku Baris Grup di pen.
            div { class: "flex items-start gap-[18px]",
                div { class: "flex-1 flex flex-col gap-[18px]",
                    LocationGroup { draft }
                    AudioGroup { draft, playing_umum, playing_subuh }
                    NotificationGroup { draft }
                    // AppearanceGroup { draft }
                }
                div { class: "flex-1 flex flex-col gap-[18px]",
                    CalculationGroup { draft }
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
            Row { label: tr(lang, "f_city"), tooltip: Some(tr(lang, "hint_kota_tz").to_string()),
                DropSearch {
                    value: draft
                        .read()
                        .city
                        .clone()
                        .unwrap_or_else(|| tr(lang, "city_manual").to_string()),
                    active: draft.read().city.clone().unwrap_or_default(),
                    options: CITIES
                        .iter()
                        .map(|c| (c.name.to_string(), c.name.to_string()))
                        .collect(),
                    searchable: true,
                    on_select: move |nama: String| {
                        if let Some(city) = find_city(&nama) {
                            let mut s = draft.read().clone();
                            s.city = Some(city.name.to_string());
                            s.lat = city.lat;
                            s.lng = city.lng;
                            s.timezone = city.timezone.name().to_string();
                            apply_settings(draft, s);
                        }
                    },
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
                DropSearch {
                    value: draft.read().timezone.clone(),
                    active: draft.read().timezone.clone(),
                    options: ZONA_WAKTU
                        .iter()
                        .map(|(name, label)| (name.to_string(), label.to_string()))
                        .collect(),
                    searchable: false,
                    on_select: move |tz: String| {
                        let mut s = draft.read().clone();
                        s.set_timezone_name(&tz);
                        s.city = None;
                        apply_settings(draft, s);
                    },
                }
            }
        }
    }
}

/// Grup PERHITUNGAN: metode perhitungan, mazhab ashar, dan koreksi waktu
/// per sholat (±30 menit — selaraskan jadwal dengan masjid setempat).
#[component]
fn CalculationGroup(draft: Signal<Settings>) -> Element {
    let lang = state::settings().language;

    // Jadwal TANPA koreksi — sumber hint "waktu bawaan" di baris yang
    // offset-nya ≠ 0. Dihitung sekali per render grup.
    let mut dasar = (*state::settings()).clone();
    dasar.prayer_offsets.clear();
    let dasar_times =
        crate::prayer::day_times(&dasar, crate::prayer::now_in_tz(&dasar).date_naive());

    // Baris koreksi: (kunci nama, sholat).
    const BARIS_KOREKSI: [(&str, crate::prayer::PrayerKind); 5] = [
        ("subuh", crate::prayer::PrayerKind::Subuh),
        ("dzuhur", crate::prayer::PrayerKind::Dzuhur),
        ("ashar", crate::prayer::PrayerKind::Ashar),
        ("maghrib", crate::prayer::PrayerKind::Maghrib),
        ("isya", crate::prayer::PrayerKind::Isya),
    ];

    rsx! {
        Group { label: tr(lang, "sec_calc"),
            Row { label: tr(lang, "f_method"),
                DropSearch {
                    value: tr(lang, method_label(draft.read().method)).to_string(),
                    active: method_name(draft.read().method).to_string(),
                    options: METODE
                        .iter()
                        .map(|(kunci, kunci_i18n)| {
                            ((*kunci).to_string(), tr(lang, kunci_i18n).to_string())
                        })
                        .collect(),
                    searchable: false,
                    on_select: move |kunci: String| {
                        if let Some(m) = method_from_name(&kunci) {
                            let mut s = draft.read().clone();
                            s.method = m;
                            apply_settings(draft, s);
                        }
                    },
                }
            }
            Row { label: tr(lang, "f_madhab"),
                DropSearch {
                    value: if draft.read().madhab == MadhabChoice::Hanafi {
                        tr(lang, "madhab_hanafi").to_string()
                    } else {
                        tr(lang, "madhab_shafi").to_string()
                    },
                    active: if draft.read().madhab == MadhabChoice::Hanafi {
                        "hanafi".to_string()
                    } else {
                        "shafi".to_string()
                    },
                    options: vec![
                        ("shafi".to_string(), tr(lang, "madhab_shafi").to_string()),
                        (
                            "hanafi".to_string(),
                            tr(lang, "madhab_hanafi").to_string(),
                        ),
                    ],
                    searchable: false,
                    on_select: move |kunci: String| {
                        let mut s = draft.read().clone();
                        s.madhab = if kunci == "hanafi" {
                            MadhabChoice::Hanafi
                        } else {
                            MadhabChoice::Shafi
                        };
                        apply_settings(draft, s);
                    },
                }
            }
            for (key, kind) in BARIS_KOREKSI {
                {
                    let offset = draft.read().offset_of(kind);
                    // Hint hanya saat dikoreksi: tampilkan jadwal bawaan agar
                    // user tahu titik nolnya.
                    let hint = if offset != 0 {
                        dasar_times.map(|t| {
                            tr(lang, "offset_hint").replace(
                                "{waktu}",
                                &crate::prayer::format_time(t.waktu(kind), draft.read().time_format_24),
                            )
                        })
                    } else {
                        None
                    };
                    rsx! {
                        Row { label: tr(lang, key), hint: hint,
                            OffsetStepper {
                                value: offset,
                                on_decrease: move |_| {
                                    let mut s = draft.read().clone();
                                    let next = (s.offset_of(kind) - 1).max(-30);
                                    // 0 disimpan sebagai tanpa-entri agar config tetap ramping.
                                    if next == 0 { s.prayer_offsets.remove(&kind); }
                                    else { s.prayer_offsets.insert(kind, next); }
                                    apply_settings(draft, s);
                                },
                                on_increase: move |_| {
                                    let mut s = draft.read().clone();
                                    let next = (s.offset_of(kind) + 1).min(30);
                                    if next == 0 { s.prayer_offsets.remove(&kind); }
                                    else { s.prayer_offsets.insert(kind, next); }
                                    apply_settings(draft, s);
                                },
                            }
                        }
                    }
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
                    Tooltip { label: tr(lang, "pick_audio_title").to_string(), posisi: PosisiTooltip::Atas,
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
                    }
                    Tooltip { label: if *playing_umum.read() { tr(lang, "stop_play").to_string() } else { tr(lang, "test_play").to_string() }, posisi: PosisiTooltip::Atas,
                        button {
                            class: "{BTN_ICON}",
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
            }
            Row { label: tr(lang, "f_subuh_file"), hint: deskripsi_sumber(&draft.read(), true),
                div { class: "flex items-center gap-2",
                    Tooltip { label: tr(lang, "pick_audio_title").to_string(), posisi: PosisiTooltip::Atas,
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
                    }
                    Tooltip { label: if *playing_subuh.read() { tr(lang, "stop_play").to_string() } else { tr(lang, "test_play_subuh").to_string() }, posisi: PosisiTooltip::Atas,
                        button {
                            class: "{BTN_ICON}",
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
            Row { label: tr(lang, "f_imsak_notif"), hint: tr(lang, "hint_imsak_notif"),
                Switch {
                    checked: draft.read().imsak_notif,
                    on_change: move |on| {
                        let mut s = draft.read().clone();
                        s.imsak_notif = on;
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
            Row { label: tr(lang, "f_update_check"), hint: tr(lang, "hint_update_check"),
                Switch {
                    checked: draft.read().update_check,
                    on_change: move |on| {
                        let mut s = draft.read().clone();
                        s.update_check = on;
                        apply_settings(draft, s);
                    },
                }
            }
            Row { label: tr(lang, "update_check_row"),
                Tooltip { label: tr(lang, "hint_update_check").to_string(), posisi: PosisiTooltip::Kiri,
                    button {
                        class: "{BTN}",
                        onclick: move |_| crate::update::check_now_manual(),
                        "{tr(lang, \"update_check_btn\")}"
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

/// (kunci nilai, kunci terjemahan) metode perhitungan — sumber dropdown
/// custom di grup Perhitungan.
const METODE: [(&str, &str); 6] = [
    ("kemenag", "m_kemenag"),
    ("mwl", "m_mwl"),
    ("egyptian", "m_egyptian"),
    ("karachi", "m_karachi"),
    ("umm_al_qura", "m_umm_al_qura"),
    ("north_america", "m_isna"),
];

/// Kunci terjemahan label untuk satu metode.
fn method_label(m: CalcMethod) -> &'static str {
    let nama = method_name(m);
    METODE
        .iter()
        .find(|(kunci, _)| *kunci == nama)
        .map(|(_, kunci_i18n)| *kunci_i18n)
        .expect("metode selalu ada di tabel METODE")
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
