use crate::adb::{get_users, PackageFingerprint, UserInfo};
use crate::material_symbol_icons::{ICON_DELETE, ICON_INFO, ICON_TOGGLE_OFF, ICON_TOGGLE_ON};
use crate::models::PackageInfoCache;
use crate::shared_store_stt::get_shared_store;
use crate::dure_sijang_app::{UadNgLists, DureSijangApp};
use eframe::egui;
use egui_i18n::tr;
use egui_material3::{icon_button_standard, theme::get_global_color, DataTableCell};
use std::collections::HashMap;

/// Helper function to convert enabled code to string
fn enabled_to_string(enabled: i32) -> &'static str {
    match enabled {
        0 => "DEFAULT",
        1 => "ENABLED",
        2 => "DISABLED",
        3 => "DISABLED_USER",
        _ => "UNKNOWN",
    }
}
