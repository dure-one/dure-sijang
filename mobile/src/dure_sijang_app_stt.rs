#[cfg(not(target_os = "android"))]
use crate::install_stt::InstallStatus;
use crate::Config;
use crate::LogLevel;
use crate::Settings;
use eframe::egui::Rect;
use egui_material3::menu::{Corner, FocusState, Positioning};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Default)]
pub struct LogSettings {
    pub show_logs: bool,
    pub log_level: LogLevel,
}

#[derive(Debug, Clone)]
pub struct UadNgLists {
    pub apps: HashMap<String, AppEntry>,
}

// UAD-NG's uad_lists.json is a top-level JSON object keyed by package id
// (https://github.com/0x192/universal-android-debloater resources/assets/uad_lists.json),
// not an array of entries with an embedded id field.
impl<'de> Deserialize<'de> for UadNgLists {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let apps = HashMap::<String, AppEntry>::deserialize(deserializer)?;
        Ok(UadNgLists { apps })
    }
}

impl Serialize for UadNgLists {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.apps.serialize(serializer)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppEntry {
    pub list: String,
    pub description: String,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub dependencies: Vec<String>,
    #[serde(rename = "neededBy", default, deserialize_with = "null_to_empty_vec")]
    pub needed_by: Vec<String>,
    #[serde(default, deserialize_with = "null_to_empty_vec")]
    pub labels: Vec<String>,
    pub removal: String,
}

fn null_to_empty_vec<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<Vec<String>>::deserialize(deserializer).map(|opt| opt.unwrap_or_default())
}

#[cfg(test)]
mod uad_ng_lists_tests {
    use super::UadNgLists;

    // UAD-NG's uad_lists.json is a top-level JSON object keyed by package id,
    // e.g. https://github.com/0x192/universal-android-debloater resources/assets/uad_lists.json
    const SAMPLE_MAP_JSON: &str = r#"{
        "org.lineageos.jelly": {
            "list": "Oem",
            "description": "LineageOS Browser App.",
            "dependencies": [],
            "neededBy": [],
            "labels": [],
            "removal": "Recommended"
        }
    }"#;

    #[test]
    fn deserializes_upstream_map_format() {
        let lists: UadNgLists = serde_json::from_str(SAMPLE_MAP_JSON).unwrap();
        assert_eq!(lists.apps.len(), 1);
        let entry = lists.apps.get("org.lineageos.jelly").unwrap();
        assert_eq!(entry.list, "Oem");
        assert_eq!(entry.removal, "Recommended");
    }
}

#[doc(hidden)]
pub struct DureSijangApp {
    pub config: Option<Config>,
    pub shizuku_connected: bool,
    // top app bar state
    pub title_text: String,
    pub show_navigation: bool,
    pub show_actions: bool,
    pub is_scrolled: bool,
    pub custom_height: f32,
    pub use_custom_height: bool,
    //
    pub custom_selected: usize,
    // menu control
    pub items_button_rect: Option<Rect>,
    pub standard_menu_open: bool,
    // Knob options
    pub anchor_corner: Corner,
    pub menu_corner: Corner,
    pub default_focus: FocusState,
    pub positioning: Positioning,
    pub quick: bool,
    pub has_overflow: bool,
    pub stay_open_on_outside_click: bool,
    pub stay_open_on_focusout: bool,
    pub skip_restore_focus: bool,
    pub x_offset: f32,
    pub y_offset: f32,
    pub no_horizontal_flip: bool,
    pub no_vertical_flip: bool,
    pub typeahead_delay: f32,
    pub list_tab_index: i32,

    pub disabled: bool,

    // Settings
    pub settings: Settings,

    // Dialog states
    pub dlg_settings: crate::dlg_settings_stt::DlgSettings,

    // LEGACY: Deleted for mycart browser project
    // pub dlg_adb_install: crate::dlg_adb_install_stt::DlgAdbInstall,
    pub dlg_about: crate::dlg_about_stt::DlgAbout,
    pub dlg_update: crate::dlg_update_stt::DlgUpdate,
    // LEGACY: Deleted for mycart browser project
    // pub dlg_dashcounter_details: crate::dlg_dashcounter_details_stt::DlgDashCounterDetails,
    // pub dlg_mobile_list: crate::dlg_mobile_list_stt::DlgMobileList,

    // Installation status (desktop only)
    #[cfg(not(target_os = "android"))]
    pub install_status: InstallStatus,
    #[cfg(not(target_os = "android"))]
    pub install_dialog_open: bool,
    #[cfg(not(target_os = "android"))]
    pub install_message: String,

    // Update status (both desktop and Android)
    pub update_status: String,
    pub update_available: bool,
    pub update_checking: bool,

    // Background worker queues for fetching app data
    // LEGACY: Deleted for mycart browser project
    // pub google_play_queue: Option<std::sync::Arc<crate::calc_googleplay::GooglePlayQueue>>,
    // pub fdroid_queue: Option<std::sync::Arc<crate::calc_fdroid::FDroidQueue>>,
    // pub apkmirror_queue: Option<std::sync::Arc<crate::calc_apkmirror::ApkMirrorQueue>>,

    // First-run initialization flag
    pub first_update_done: bool,

    // Pinch-to-zoom state (Android)
    pub zoom_factor: f32,

    // WebView widgets (stored separately from BrowserState metadata)
    pub webview_widgets: std::collections::HashMap<egui::Id, egui_webview::EguiWebView>,

    // Browser state (MVVM)
    pub browser_state: crate::browser_stt::BrowserState,
}
