#![doc(hidden)]

use std::cell::Cell;
#[cfg(not(target_os = "android"))]
use std::collections::HashMap;
use std::process;
use std::sync::{Arc, Mutex};
use sys_locale::get_locale;

use eframe::egui;
use egui_i18n::tr;
use egui_material3::menu::{Corner, FocusState, Positioning};
use egui_material3::{dialog, menu, menu_item, MaterialButton};
use egui_material3::{get_global_theme, ContrastLevel, ThemeMode};

use crate::LogLevel;

pub use crate::dure_sijang_app_stt::*;
use crate::{Config, Settings};
use crate::browser_stt::SidebarTab;

use crate::install;
#[cfg(not(target_os = "android"))]
use crate::install_stt::InstallStatus;

use eframe::egui::Context;
use egui_material3::theme::{
    apply_theme, load_fonts, load_theme_from_json_str, load_themes, set_contrast_level,
    set_theme_mode, setup_local_fonts, setup_local_fonts_from_bytes, setup_local_theme,
    update_window_background,
};
use std::sync::OnceLock;

// Embedded theme data
const THEME_GREEN: &str = include_str!("../resources/material-theme-green.json");
const THEME_LIGHTBLUE: &str = include_str!("../resources/material-theme-lightblue.json");
const THEME_LIGHTPINK: &str = include_str!("../resources/material-theme-lightpink.json");
const THEME_YELLOW: &str = include_str!("../resources/material-theme-yellow.json");

/// Initialize common app components (database, i18n).
/// Call this early in main() before creating the app.
pub fn init_common() {
    // Set up database path before initializing anything that uses the database
    if let Ok(config) = Config::new() {
        let db_path = config.db_dir.join("dure-sijang.db");
        crate::db::set_db_path(db_path.to_string_lossy().to_string());
    }

    // Initialize i18n
    crate::init_i18n();
}

/// Initialize egui context with fonts, themes, and image loaders.
/// Call this in the eframe app creation callback.
pub fn init_egui(ctx: &Context) {
    setup_local_theme(Some("resources/material-theme.json"));
    // material icon fonts https://github.com/google/material-design-icons
    setup_local_fonts_from_bytes(
        "MaterialSymbolsOutlined",
        include_bytes!("../resources/MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].ttf"),
    );
    setup_local_fonts_from_bytes(
        "NotoSansKr",
        include_bytes!("../resources/noto-sans-kr.ttf"),
    );
    egui_extras::install_image_loaders(ctx);
    load_fonts(ctx);
    load_themes();
    update_window_background(ctx);

    // Restore saved custom font if configured
    if let Ok(config) = Config::new() {
        if let Ok(settings) = config.load_settings() {
            if !settings.font_path.is_empty() {
                setup_local_fonts(Some(&settings.font_path));
                load_fonts(ctx);
            }
        }
    }
}

static LOG_BUFFER: OnceLock<Arc<Mutex<String>>> = OnceLock::new();
static LOG_SETTINGS: OnceLock<Arc<Mutex<LogSettings>>> = OnceLock::new();

// Get or initialize the log buffer
fn get_log_buffer() -> &'static Arc<Mutex<String>> {
    LOG_BUFFER.get_or_init(|| Arc::new(Mutex::new(String::new())))
}

// Get or initialize log settings
fn get_log_settings() -> &'static Arc<Mutex<LogSettings>> {
    LOG_SETTINGS.get_or_init(|| {
        Arc::new(Mutex::new(LogSettings {
            show_logs: false,
            log_level: LogLevel::Info,
        }))
    })
}

// Update log settings
pub fn update_log_settings(settings: LogSettings) {
    if let Ok(mut log_settings) = get_log_settings().lock() {
        *log_settings = settings;
    }
}

// Function to append to log buffer
pub fn append_log(level: &str, message: String) {
    // Check if this log level should be captured
    // Logic: Show messages at the selected level and all higher priority levels
    // Priority order: ERROR > WARN > INFO > DEBUG > TRACE
    let should_log = if let Ok(settings) = get_log_settings().lock() {
        let message_level = match level {
            "ERROR" => LogLevel::Error,
            "WARN" => LogLevel::Warn,
            "INFO" => LogLevel::Info,
            "DEBUG" => LogLevel::Debug,
            "TRACE" => LogLevel::Trace,
            _ => return, // Skip unknown levels
        };

        // Check if message level is at or above the selected log level
        let level_priority = |lvl: LogLevel| -> i32 {
            match lvl {
                LogLevel::Error => 0,
                LogLevel::Warn => 1,
                LogLevel::Info => 2,
                LogLevel::Debug => 3,
                LogLevel::Trace => 4,
            }
        };

        level_priority(message_level) <= level_priority(settings.log_level)
    } else {
        false
    };

    if !should_log {
        return;
    }

    if let Ok(mut buffer) = get_log_buffer().lock() {
        buffer.push_str(&message);
        buffer.push('\n');

        // Keep only last 10000 characters to prevent memory issues
        if buffer.len() > 10000 {
            *buffer = buffer.chars().skip(buffer.len() - 10000).collect();
        }
    }
}

pub trait View {
    fn ui(&mut self, ui: &mut egui::Ui);
}

impl Default for DureSijangApp {
    fn default() -> Self {
        //

        // Log basic system info at app start
        log::info!("=== System Information ===");
        log::info!("OS: {}", std::env::consts::OS);
        log::info!("Architecture: {}", std::env::consts::ARCH);
        log::info!("Family: {}", std::env::consts::FAMILY);

        let config = Config::new().ok();
        if let Some(ref cfg) = config {
            let db_path = cfg.db_dir.join("uad.db");
            crate::db::set_db_path(db_path.to_string_lossy().to_string());
        }
        let settings = if let Some(ref cfg) = config {
            cfg.load_settings().unwrap_or_default()
        } else {
            Settings::default()
        };
        let cache_dir = if let Some(ref cfg) = config {
            cfg.cache_dir.clone()
        } else {
            std::path::PathBuf::from("./cache")
        };
        let tmp_dir = if let Some(ref cfg) = config {
            cfg.tmp_dir.clone()
        } else {
            std::path::PathBuf::from("./tmp")
        };

        let mut app = Self {
            config: config,
            shizuku_connected: false,

            title_text: "UAD-Shizuku".to_string(),
            show_navigation: false,
            show_actions: true,
            is_scrolled: false,
            custom_height: 64.0,
            use_custom_height: false,

            custom_selected: 0,

            items_button_rect: None,
            standard_menu_open: false,

            anchor_corner: Corner::BottomLeft,
            menu_corner: Corner::TopLeft,
            default_focus: FocusState::None,
            positioning: Positioning::Absolute,
            quick: false,
            has_overflow: false,
            stay_open_on_outside_click: false,
            stay_open_on_focusout: false,
            skip_restore_focus: false,
            x_offset: 0.0,
            y_offset: 0.0,
            no_horizontal_flip: false,
            no_vertical_flip: false,
            typeahead_delay: 200.0,
            list_tab_index: -1,

            disabled: false,

            settings: settings.clone(),

            // Dialog states
            dlg_settings: Default::default(),

            dlg_about: crate::dlg_about_stt::DlgAbout::default(),
            dlg_update: crate::dlg_update_stt::DlgUpdate::default(),

            // Installation status (desktop only)
            #[cfg(not(target_os = "android"))]
            install_status: install::check_install(),
            #[cfg(not(target_os = "android"))]
            install_dialog_open: false,
            #[cfg(not(target_os = "android"))]
            install_message: String::new(),

            // Update status (both desktop and Android)
            update_status: String::new(),
            update_available: false,
            update_checking: false,

            // First-run initialization flag
            first_update_done: false,

            // Pinch-to-zoom state
            zoom_factor: 1.0,

            // WebView widgets (desktop-only)
            #[cfg(not(target_os = "android"))]
            webview_widgets: HashMap::new(),

            // Android WebView manager
            #[cfg(target_os = "android")]
            android_webview_manager: crate::android_webview::AndroidWebViewManager::new(),

            // Browser state (MVVM)
            browser_state: crate::browser_stt::BrowserState::load_from_db(),
        };

        // Apply persisted theme preferences
        app.apply_saved_theme_preferences();

        // Apply persisted language preferences
        app.apply_saved_language();

        // Initialize log settings from loaded settings
        update_log_settings(LogSettings {
            show_logs: app.settings.show_logs,
            log_level: Self::string_to_log_level(&app.settings.log_level),
        });

        app
    }
}

impl DureSijangApp {
    /// Create a new DureSijangApp with eframe CreationContext
    ///
    /// This captures the window handle for webview creation.
    pub fn new(cc: &eframe::CreationContext) -> Self {
        let mut app = Self::default();

        // Capture window handle for webview parent (NEW - Task 5)

        app
    }

    fn apply_saved_theme_preferences(&self) {
        // Set theme mode and contrast level using utility functions
        set_theme_mode(self.settings.theme_mode.parse().unwrap_or(ThemeMode::Auto));
        set_contrast_level(
            self.settings
                .contrast_level
                .parse()
                .unwrap_or(ContrastLevel::Normal),
        );

        // Apply saved theme if not default
        if !self.settings.theme_name.is_empty() && self.settings.theme_name != "default" {
            self.apply_theme_by_name(&self.settings.theme_name);
        }
    }

    /// Detect system language using sys_locale and map to supported languages
    fn detect_system_language() -> String {
        match get_locale().as_deref() {
            Some("ko_KR") | Some("ko-KR") | Some("ko") => "ko-KR".to_string(),
            Some("en_US") | Some("en-US") | Some("en_GB") | Some("en-GB") | Some("en") | _ => {
                "en-US".to_string()
            }
        }
    }

    fn apply_saved_language(&self) {
        let language_to_apply =
            if self.settings.language == "Auto" || self.settings.language.is_empty() {
                Self::detect_system_language()
            } else {
                self.settings.language.clone()
            };
        egui_i18n::set_language(&language_to_apply);
    }

    fn apply_saved_text_style(&self, ctx: &egui::Context) {
        if !self.settings.override_text_style.is_empty() {
            let text_style = Self::string_to_text_style(&self.settings.override_text_style);
            ctx.style_mut(|s| {
                s.override_text_style = text_style;
            });
        }
    }

    /// Detect OS theme preference using dark-light crate (desktop) or Android JNI (Android)
    fn detect_os_theme() -> ThemeMode {
        #[cfg(target_os = "android")]
        {
            // Use Android JNI to get system theme mode
            match crate::android_contexttheme::get_ui_theme_mode() {
                Ok(crate::android_contexttheme::UiThemeMode::Dark) => {
                    log::debug!("Android system theme detected: Dark");
                    ThemeMode::Dark
                }
                Ok(crate::android_contexttheme::UiThemeMode::Light) => {
                    log::debug!("Android system theme detected: Light");
                    ThemeMode::Light
                }
                Ok(crate::android_contexttheme::UiThemeMode::Unspecified) | Err(_) => {
                    log::debug!("Android system theme unspecified or error, defaulting to Light");
                    ThemeMode::Light
                }
            }
        }

        #[cfg(not(target_os = "android"))]
        {
            // Use dark-light crate for desktop platforms
            match dark_light::detect() {
                dark_light::Mode::Dark => ThemeMode::Dark,
                dark_light::Mode::Light => ThemeMode::Light,
                dark_light::Mode::Default => ThemeMode::Light, // Default to Light if unspecified
            }
        }
    }

    /// Get theme data by name
    fn get_theme_data_by_name(name: &str) -> Option<&'static str> {
        match name {
            "green" => Some(THEME_GREEN),
            "lightblue" => Some(THEME_LIGHTBLUE),
            "lightpink" => Some(THEME_LIGHTPINK),
            "yellow" => Some(THEME_YELLOW),
            _ => None,
        }
    }

    /// Apply theme by name
    fn apply_theme_by_name(&self, name: &str) {
        if let Some(theme_data) = Self::get_theme_data_by_name(name) {
            if let Err(e) = load_theme_from_json_str(theme_data) {
                log::error!("Failed to load theme '{}': {}", name, e);
            }
        }
    }

    fn string_to_log_level(value: &str) -> LogLevel {
        match value {
            "Error" => LogLevel::Error,
            "Warn" => LogLevel::Warn,
            "Info" => LogLevel::Info,
            "Debug" => LogLevel::Debug,
            "Trace" => LogLevel::Trace,
            _ => LogLevel::Info,
        }
    }

    fn log_level_to_string(level: LogLevel) -> String {
        match level {
            LogLevel::Error => "Error".to_string(),
            LogLevel::Warn => "Warn".to_string(),
            LogLevel::Info => "Info".to_string(),
            LogLevel::Debug => "Debug".to_string(),
            LogLevel::Trace => "Trace".to_string(),
        }
    }

    fn string_to_text_style(value: &str) -> Option<egui::TextStyle> {
        if value.is_empty() {
            return None;
        }
        match value {
            "Small" => Some(egui::TextStyle::Small),
            "Body" => Some(egui::TextStyle::Body),
            "Button" => Some(egui::TextStyle::Button),
            "Heading" => Some(egui::TextStyle::Heading),
            "Monospace" => Some(egui::TextStyle::Monospace),
            _ => None,
        }
    }

    fn text_style_to_string(style: &Option<egui::TextStyle>) -> String {
        match style {
            None => String::new(),
            Some(s) => s.to_string(),
        }
    }

    pub fn ui_internal(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let available_width = ui.ctx().content_rect().width();
        let is_desktop = available_width >= crate::DESKTOP_MIN_WIDTH;
        // Apply theme at the start of UI rendering
        apply_theme(ui.ctx(), Some(Self::detect_os_theme));

        // Apply saved text style
        self.apply_saved_text_style(ui.ctx());

        // Top app bar removed - hamburger menu moved to browser toolbar

        // === notification render progress area start
        ui.horizontal(|ui| {
            egui::ScrollArea::horizontal()
                .id_salt(format!("notification_render_progress_area"))
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;

                    // Package loading progress
                });
        });
        // === notification render progress area end

        // === tab controller (shared logic for mobile and desktop)
        self.prepare_tabs_controller();

        // === main content area start
        let available_width = ui.available_width();
        let is_desktop = available_width >= crate::DESKTOP_MIN_WIDTH;

        if is_desktop {
            self.render_desktop_tabs(ui, frame);
        } else {
            // Mobile UI - use same layout for now
            self.render_desktop_tabs(ui, frame);
        }
        // === main content area end

        // === logs area start
        if self.settings.show_logs {
            // Add vertical spacer to push logs to bottom
            self.render_logs(ui);
        }
        // === logs area end

        // === settings dialog
        self.dlg_settings.show(ui.ctx(), &mut self.settings);
        // Handle save and theme changes from settings dialog
        if self.dlg_settings.save_clicked {
            self.dlg_settings.save_clicked = false; // Reset flag to prevent repeated saves
            self.save_settings();
        }
        if let Some(theme_name) = self.dlg_settings.theme_to_apply.take() {
            if theme_name == "default" {
                setup_local_theme(Some("resources/material-theme.json"));
                load_themes();
            } else {
                self.apply_theme_by_name(&theme_name);
            }
        }
        // === settings dialog end

        // === Update dialog (both desktop and Android)
        self.dlg_update.show(ui.ctx());
        // Handle update request from update dialog
        if self.dlg_update.do_update {
            self.perform_update();
        }
        // === Update dialog end

        // === About dialog
        self.dlg_about.show(
            ui.ctx(),
            self.update_checking,
            self.update_available,
            &self.update_status,
        );
        // Handle check update and perform update from about dialog
        if self.dlg_about.do_check_update {
            self.check_for_update();
        }
        if self.dlg_about.do_perform_update {
            self.perform_update();
        }
        // === About dialog end

        // === Install dialog (desktop only)
        #[cfg(not(target_os = "android"))]
        self.show_install_dialog(ui.ctx());
        // === Install dialog end

        // Handle settings button - open settings dialog
        if ui
            .ctx()
            .data(|data| data.get_temp::<bool>(egui::Id::new("settings_button_clicked")))
            .unwrap_or(false)
        {
            ui.ctx().data_mut(|data| {
                data.remove::<bool>(egui::Id::new("settings_button_clicked"));
            });

            // Open settings dialog
            self.dlg_settings.open();
        }
    }

    fn show_menus(&mut self, ctx: &egui::Context) {
        // Standard Menu with Items - opens below button (default positioning)
        if self.standard_menu_open {
            let close_menu = Cell::new(false);
            let should_exit = Cell::new(false);
            let open_settings = Cell::new(false);
            let open_about = Cell::new(false);
            #[cfg(not(target_os = "android"))]
            let do_install_action = Cell::new(false);
            let settings_text = tr!("settings");
            let about_text = tr!("about");
            let exit_text = tr!("exit");
            let settings_item = self.create_menu_item(&settings_text, "settings", || {
                println!("Settings clicked!");
                open_settings.set(true);
                close_menu.set(true);
            });
            let about_item = self.create_menu_item(&about_text, "info", || {
                println!("About clicked!");
                open_about.set(true);
                close_menu.set(true);
            });

            // Install/Uninstall menu item (desktop only)
            #[cfg(not(target_os = "android"))]
            let install_text = if self.install_status == InstallStatus::Installed {
                tr!("uninstall")
            } else {
                tr!("install")
            };
            #[cfg(not(target_os = "android"))]
            let install_item = self.create_menu_item(&install_text, "install", || {
                do_install_action.set(true);
                close_menu.set(true);
            });

            let exit_item = self.create_menu_item(&exit_text, "exit", || {
                close_menu.set(true);
                should_exit.set(true);
                println!("Exit clicked!");
            });

            #[cfg(not(target_os = "android"))]
            let menu_builder = menu("standard_menu", &mut self.standard_menu_open)
                .item(settings_item)
                .item(install_item)
                .item(about_item)
                .item(exit_item);

            #[cfg(target_os = "android")]
            let menu_builder = menu("standard_menu", &mut self.standard_menu_open)
                .item(settings_item)
                .item(about_item)
                .item(exit_item);

            let mut menu_builder = menu_builder
                .anchor_corner(self.anchor_corner)
                .menu_corner(self.menu_corner)
                .default_focus(self.default_focus)
                .positioning(self.positioning)
                .quick(self.quick)
                .has_overflow(self.has_overflow)
                .stay_open_on_outside_click(self.stay_open_on_outside_click)
                .stay_open_on_focusout(self.stay_open_on_focusout)
                .skip_restore_focus(self.skip_restore_focus)
                .x_offset(self.x_offset)
                .y_offset(self.y_offset)
                .no_horizontal_flip(self.no_horizontal_flip)
                .no_vertical_flip(self.no_vertical_flip)
                .typeahead_delay(self.typeahead_delay)
                .list_tab_index(self.list_tab_index);

            if let Some(rect) = self.items_button_rect {
                menu_builder = menu_builder.anchor_rect(rect);
            }

            menu_builder.show(ctx);

            if close_menu.get() {
                self.standard_menu_open = false;
            }

            if open_settings.get() {
                // Open settings dialog
                self.dlg_settings.open();
            }

            if open_about.get() {
                self.dlg_about.open();
            }

            #[cfg(not(target_os = "android"))]
            if do_install_action.get() {
                self.perform_install_action();
            }

            if should_exit.get() {
                ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                process::exit(1);
            }
        }
    }

    fn create_menu_item<'a, F>(
        &self,
        text: &'a str,
        _id: &str,
        on_click: F,
    ) -> egui_material3::MenuItem<'a>
    where
        F: Fn() + 'a,
    {
        let mut item = menu_item(text).on_click(on_click);
        if self.disabled {
            item = item.enabled(false);
        }
        item
    }

    /// Perform install or uninstall action based on current status
    #[cfg(not(target_os = "android"))]
    fn perform_install_action(&mut self) {
        use crate::install_stt::InstallResult;

        let result = if self.install_status == InstallStatus::Installed {
            install::do_uninstall()
        } else {
            install::do_install()
        };

        match result {
            InstallResult::Success(msg) => {
                self.install_message = msg;
                // Refresh install status
                self.install_status = install::check_install();
            }
            InstallResult::Error(err) => {
                self.install_message = format!("Error: {}", err);
            }
        }
        self.install_dialog_open = true;
    }

    /// Show installation result dialog
    #[cfg(not(target_os = "android"))]
    fn show_install_dialog(&mut self, ctx: &egui::Context) {
        if self.install_dialog_open {
            let title = if self.install_message.starts_with("Error") {
                tr!("install-error")
            } else if self.install_status == InstallStatus::Installed {
                tr!("install-success")
            } else {
                tr!("uninstall-success")
            };

            let mut should_close = false;

            dialog("install_dialog", &title, &mut self.install_dialog_open)
                .content(|ui| {
                    ui.label(&self.install_message);
                    ui.add_space(16.0);

                    ui.horizontal(|ui| {
                        if ui.add(MaterialButton::filled(tr!("ok"))).clicked() {
                            should_close = true;
                        }
                    });
                })
                .show(ctx);

            if should_close {
                self.install_dialog_open = false;
            }
        }
    }

    /// Check for updates from GitHub
    fn check_for_update(&mut self) {
        self.update_checking = true;
        self.update_status.clear();
        self.update_available = false;

        match install::check_update() {
            Ok(info) => {
                self.update_checking = false;
                if info.available {
                    self.update_available = true;
                    self.dlg_update.download_url = info.download_url;
                    self.dlg_update.current_version = info.current_version;
                    self.dlg_update.latest_version = info.latest_version.clone();
                    self.dlg_update.release_notes = info.release_notes;
                    self.update_status = format!(
                        "{} {} → {}",
                        tr!("update-available"),
                        self.dlg_update.current_version,
                        info.latest_version
                    );
                    // Open update dialog automatically
                    self.dlg_update.open();
                } else {
                    self.update_status = tr!("up-to-date").to_string();
                }
            }
            Err(e) => {
                self.update_checking = false;
                self.update_status = format!("{}: {}", tr!("update-error"), e);
            }
        }
    }

    /// Perform update
    fn perform_update(&mut self) {
        #[cfg(not(target_os = "android"))]
        {
            use crate::install_stt::InstallResult;

            let tmp_dir = if let Some(ref cfg) = self.config {
                cfg.tmp_dir.clone()
            } else {
                std::path::PathBuf::from("./tmp")
            };

            match install::do_update(
                &self.dlg_update.download_url,
                &self.dlg_update.latest_version,
                &tmp_dir,
            ) {
                InstallResult::Success(msg) => {
                    self.install_message = msg;
                    self.update_available = false;
                    self.update_status.clear();
                    self.dlg_update.close();
                }
                InstallResult::Error(err) => {
                    self.install_message = format!("Error: {}", err);
                }
            }
            self.install_dialog_open = true;
        }

        #[cfg(target_os = "android")]
        {
            // On Android, open browser to download page
            if !self.dlg_update.download_url.is_empty() {
                if let Err(e) = webbrowser::open(&self.dlg_update.download_url) {
                    log::error!("Failed to open browser for update download: {}", e);
                    self.update_status = format!("Failed to open browser: {}", e);
                } else {
                    log::info!("Opened browser for update download");
                    self.dlg_update.close();
                }
            }
        }
    }

    // Controller method: prepare tab state (shared by both mobile and desktop UI)
    fn prepare_tabs_controller(&mut self) {
        // No-op: legacy tabs removed for mycart browser
    }

    fn render_desktop_tabs(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // Browser UI replaces legacy Dure-Sijang tabs
        self.render_browser_ui(ui, frame);
    }

    fn render_logs(&mut self, ui: &mut egui::Ui) {
        // put blank space before logs
        // Calculate max height: leave 200px for log box if enabled
        let reserved_space = if self.settings.show_logs { 200.0 } else { 0.0 };
        let max_height = ui.available_height() - reserved_space;

        // put blank space if self.installed_packages is empty
        if max_height > 0.0 {
            ui.add_space(max_height);
        }

        // Read from global log buffer
        let log_text = if let Ok(buffer) = get_log_buffer().lock() {
            buffer.clone()
        } else {
            String::from("Unable to access logs")
        };

        // Use top_down layout within this section to keep content in correct order
        ui.label(tr!("logs"));
        // Create a scrollable text area for logs
        egui::ScrollArea::vertical()
            .id_salt("logs_scroll")
            .max_height(150.0)
            .min_scrolled_height(150.0)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut log_text.as_str())
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Monospace)
                        .interactive(false)
                        .desired_rows(10),
                );
            });
    }

    /// Extracts embedded data from GitHub HTML response
    /// Data is contained in: <script type="application/json" data-target="react-app.embeddedData">{data}</script>
    fn extract_github_embedded_data(html: &str) -> Option<String> {
        // Find the script tag with embedded data
        let script_start =
            html.find(r#"<script type="application/json" data-target="react-app.embeddedData">"#)?;
        let data_start = script_start
            + r#"<script type="application/json" data-target="react-app.embeddedData">"#.len();
        let data_end = html[data_start..].find("</script>")?;
        let json_str = &html[data_start..data_start + data_end];

        // Parse the embedded JSON to extract the actual file content
        let parsed: serde_json::Value = serde_json::from_str(json_str).ok()?;

        // Navigate to the payload > codeViewBlobLayoutRoute.StyledBlob > rawLines
        let raw_lines = parsed
            .get("payload")?
            .get("codeViewBlobLayoutRoute.StyledBlob")?
            .get("rawLines")?
            .as_array()?;

        // Join the lines to reconstruct the file content
        let content: Vec<String> = raw_lines
            .iter()
            .filter_map(|line| line.as_str().map(|s| s.to_string()))
            .collect();

        Some(content.join("\n"))
    }

    fn save_settings(&mut self) {
        // Sync theme selections into settings before persisting
        if let Ok(theme) = get_global_theme().lock() {
            self.settings.theme_mode = theme.theme_mode.to_string();
            self.settings.contrast_level = theme.contrast_level.to_string();
        }

        // Update log settings for in-app log display
        update_log_settings(LogSettings {
            show_logs: self.settings.show_logs,
            log_level: Self::string_to_log_level(&self.settings.log_level),
        });

        // Update log level in real-time
        log::info!("Updating log level to: {}", self.settings.log_level);
        crate::log_capture::update_log_level(&self.settings.log_level);

        // Save to file
        if let Some(ref config) = self.config {
            match config.save_settings(&self.settings) {
                Ok(_) => {
                    log::info!("Settings saved successfully");
                }
                Err(e) => {
                    log::error!("Failed to save settings: {}", e);
                }
            }
        } else {
            log::error!("Config not available, cannot save settings");
        }
    }
}

impl View for DureSijangApp {
    fn ui(&mut self, ui: &mut egui::Ui) {
        // View trait implementation - limited functionality without frame access
        // Full UI is rendered through eframe::App::update()
        ui.label("DureSijang Browser");
        ui.label("This view requires frame access. Use eframe::App::update() instead.");
    }
}

impl eframe::App for DureSijangApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Process GTK events for webview (Linux/OpenBSD)
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            while gtk::events_pending() {
                gtk::main_iteration_do(false);
            }
        }

        // Request continuous repainting for responsive webview
        ctx.request_repaint();

        // On first update - initialize webview and perform startup tasks
        if !self.first_update_done {
            self.first_update_done = true;

            // Initialize egui_webview for browser functionality (desktop-only)
            #[cfg(not(target_os = "android"))]
            {
                egui_webview::init_webview(ctx);
                log::info!("Initialized egui_webview for browser");

                // Create default tab if no tabs exist (first run)
                if self.browser_state.tabs.is_empty() {
                    self.add_browser_tab(ctx, frame, "https://dure.app");
                    log::info!("Created default browser tab");
                } else {
                    // Tabs were loaded from database - create webview widgets for them
                    // (tab metadata exists but webview_widgets HashMap is empty)
                    for tab in &self.browser_state.tabs {
                        // Only create webview if one doesn't already exist for this tab
                        if !self.webview_widgets.contains_key(&tab.id) {
                            use egui_webview::EguiWebView;
                            let view = EguiWebView::new(ctx, tab.id, frame, |builder| {
                                builder.with_url(&tab.url_bar)
                            });
                            self.webview_widgets.insert(tab.id, view);
                            log::info!("Created webview for loaded tab {:?} with URL: {}", tab.id, tab.url_bar);
                        }
                    }
                    log::info!("Initialized {} webviews for {} loaded tabs",
                               self.webview_widgets.len(),
                               self.browser_state.tabs.len());
                }
            }

            // Check for updates if autoupdate is enabled
            if self.settings.autoupdate {
                self.check_for_update();
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            #[cfg(target_os = "android")]
            {
                if let Some(multi_touch) = ctx.input(|i| i.multi_touch()) {
                    if multi_touch.num_touches >= 2 {
                        self.zoom_factor =
                            (self.zoom_factor * multi_touch.zoom_delta).clamp(0.25, 3.0);
                    }
                }
                ctx.set_zoom_factor(self.zoom_factor);
            }
            self.ui_internal(ui, frame);
        });

        // End webview frame (desktop-only)
        #[cfg(not(target_os = "android"))]
        egui_webview::webview_end_frame(ctx);
    }
}

// ===== WebView Management Methods (NEW - Task 5) =====
impl DureSijangApp {
    /// Navigate the webview back in history (desktop-only)
    ///
    /// # Arguments
    /// * `idx` - Tab index
    ///
    /// # Returns
    /// Ok(()) on success, Err if tab not found or navigation fails
    #[cfg(not(target_os = "android"))]
    pub fn navigate_back(&mut self, idx: usize) -> anyhow::Result<()> {
        if idx >= self.browser_state.tabs.len() {
            anyhow::bail!("Tab index {} out of bounds", idx);
        }

        let tab_id = self.browser_state.tabs[idx].id;
        if let Some(view) = self.webview_widgets.get(&tab_id) {
            view.back();
            Ok(())
        } else {
            anyhow::bail!("WebView not found for tab {:?}", tab_id)
        }
    }

    /// Navigate the webview forward in history (desktop-only)
    ///
    /// # Arguments
    /// * `idx` - Tab index
    ///
    /// # Returns
    /// Ok(()) on success, Err if tab not found or navigation fails
    #[cfg(not(target_os = "android"))]
    pub fn navigate_forward(&mut self, idx: usize) -> anyhow::Result<()> {
        if idx >= self.browser_state.tabs.len() {
            anyhow::bail!("Tab index {} out of bounds", idx);
        }

        let tab_id = self.browser_state.tabs[idx].id;
        if let Some(view) = self.webview_widgets.get(&tab_id) {
            view.forward();
            Ok(())
        } else {
            anyhow::bail!("WebView not found for tab {:?}", tab_id)
        }
    }

    /// Reload the current page in the webview (desktop-only)
    ///
    /// # Arguments
    /// * `idx` - Tab index
    ///
    /// # Returns
    /// Ok(()) on success, Err if tab not found or reload fails
    #[cfg(not(target_os = "android"))]
    pub fn navigate_reload(&mut self, idx: usize) -> anyhow::Result<()> {
        use anyhow::Context;

        if idx >= self.browser_state.tabs.len() {
            anyhow::bail!("Tab index {} out of bounds", idx);
        }

        let tab_id = self.browser_state.tabs[idx].id;
        if let Some(view) = self.webview_widgets.get(&tab_id) {
            // EguiWebView doesn't have reload(), use underlying wry webview
            view.view
                .evaluate_script("window.location.reload()")
                .context("Failed to reload webview")?;
            Ok(())
        } else {
            anyhow::bail!("WebView not found for tab {:?}", tab_id)
        }
    }

    // ===== Browser UI Methods (MVVM Browser Integration) =====

    /// Add a new browser tab (desktop-only)
    ///
    /// # Arguments
    /// * `ctx` - egui context
    /// * `frame` - eframe frame
    /// * `url` - Initial URL to load
    #[cfg(not(target_os = "android"))]
    pub fn add_browser_tab(&mut self, ctx: &egui::Context, frame: &eframe::Frame, url: &str) {
        // Create tab metadata
        let tab_id = self.browser_state.add_tab(url, url);

        // Create EguiWebView widget
        use egui_webview::EguiWebView;
        let view = EguiWebView::new(ctx, tab_id, frame, |builder| {
            builder.with_url(url)
        });

        // Store webview in HashMap
        self.webview_widgets.insert(tab_id, view);
        log::info!("Added browser tab {:?} with URL: {}", tab_id, url);
    }

    /// Add a new browser tab (Android - uses wry)
    ///
    /// # Arguments
    /// * `ctx` - egui context
    /// * `frame` - eframe frame
    /// * `url` - Initial URL to load
    #[cfg(target_os = "android")]
    pub fn add_browser_tab(&mut self, ctx: &egui::Context, frame: &eframe::Frame, url: &str) {
        // Create tab metadata
        let tab_id = self.browser_state.add_tab(url, url);

        // Create WebView using AndroidWebViewManager
        // Frame implements HasWindowHandle directly
        if let Err(e) = self.android_webview_manager.create_webview(
            tab_id,
            url,
            frame,
        ) {
            log::error!("Failed to create Android WebView: {}", e);
            // Remove tab metadata if WebView creation failed
            self.browser_state.close_tab(self.browser_state.tabs.len() - 1);
            return;
        }

        log::info!("Added Android browser tab {:?} with URL: {}", tab_id, url);
    }

    /// Close a browser tab by index
    ///
    /// # Arguments
    /// * `idx` - Tab index to close
    pub fn close_browser_tab(&mut self, idx: usize) {
        if idx >= self.browser_state.tabs.len() {
            return;
        }

        let tab_id = self.browser_state.tabs[idx].id;

        // Remove webview widget before removing tab metadata (desktop-only)
        #[cfg(not(target_os = "android"))]
        {
            if let Some(_view) = self.webview_widgets.remove(&tab_id) {
                log::info!("Destroyed webview for tab {:?}", tab_id);
                // Drop will handle cleanup
            }
        }

        // Remove Android WebView (Android-only)
        #[cfg(target_os = "android")]
        {
            self.android_webview_manager.remove_webview(&tab_id);
            log::info!("Destroyed Android WebView for tab {:?}", tab_id);
        }

        // Remove tab metadata
        self.browser_state.close_tab(idx);
        log::info!("Closed browser tab at index {}", idx);
    }

    // ===== Browser Helper Functions =====

    /// Add https:// protocol if missing from URL
    fn ensure_protocol(url: &str) -> String {
        if url.starts_with("http://") || url.starts_with("https://") {
            url.to_string()
        } else {
            format!("https://{}", url)
        }
    }

    /// Strip protocol prefix from URL for cleaner display
    fn strip_protocol(url: &str) -> &str {
        url.strip_prefix("https://")
            .or_else(|| url.strip_prefix("http://"))
            .unwrap_or(url)
    }

    /// Check if the active tab can navigate back
    fn check_can_navigate_back(app: &DureSijangApp) -> bool {
        let Some(idx) = app.browser_state.active_tab_index else {
            return false;
        };

        if idx >= app.browser_state.tabs.len() {
            return false;
        }

        #[cfg(not(target_os = "android"))]
        {
            let tab_id = app.browser_state.tabs[idx].id;
            // Check if webview exists (egui_webview doesn't expose can_go_back())
            return app.webview_widgets.get(&tab_id).is_some();
        }

        #[cfg(target_os = "android")]
        false
    }

    /// Check if the active tab can navigate forward
    fn check_can_navigate_forward(app: &DureSijangApp) -> bool {
        let Some(idx) = app.browser_state.active_tab_index else {
            return false;
        };

        if idx >= app.browser_state.tabs.len() {
            return false;
        }

        #[cfg(not(target_os = "android"))]
        {
            let tab_id = app.browser_state.tabs[idx].id;
            // Check if webview exists (egui_webview doesn't expose can_go_forward())
            return app.webview_widgets.get(&tab_id).is_some();
        }

        #[cfg(target_os = "android")]
        false
    }

    /// Render the bookmarks list in the sidebar
    fn render_bookmarks_list(app: &mut DureSijangApp, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        use egui::ScrollArea;

        let mut bookmark_to_navigate: Option<(String, String)> = None;
        let mut bookmark_to_delete: Option<usize> = None;

        ScrollArea::vertical().show(ui, |ui| {
            for (idx, bookmark) in app.browser_state.bookmarks.iter().enumerate() {
                ui.horizontal(|ui| {
                    if ui.button(&bookmark.title).clicked() {
                        bookmark_to_navigate = Some((bookmark.url.clone(), bookmark.title.clone()));
                    }
                    if ui.small_button("×").clicked() {
                        bookmark_to_delete = Some(idx);
                    }
                });
            }
        });

        // Handle bookmark actions after rendering (desktop-only)
        #[cfg(not(target_os = "android"))]
        if let Some((url, title)) = bookmark_to_navigate {
            // Create new tab with bookmarked URL
            app.add_browser_tab(ui.ctx(), frame, &url);

            // Update title for the newly created tab
            if let Some(last_idx) = app.browser_state.tabs.len().checked_sub(1) {
                app.browser_state.tabs[last_idx].title = title;
            }

            log::info!("Opened bookmark in new tab: {}", url);
        }
        if let Some(idx) = bookmark_to_delete {
            let _ = app.browser_state.delete_bookmark(idx);
        }
    }

    /// Render the browsing history list in the sidebar
    fn render_history_list(app: &mut DureSijangApp, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        use egui::ScrollArea;

        if app.browser_state.history_entries.is_empty() {
            ui.centered_and_justified(|ui| {
                ui.label("No browsing history yet");
            });
            return;
        }

        let mut url_to_navigate: Option<(String, i32)> = None;

        ScrollArea::vertical().show(ui, |ui| {
            for entry in &app.browser_state.history_entries {
                let title = entry.title.as_deref().unwrap_or(&entry.url);
                if ui.button(title).clicked() {
                    url_to_navigate = Some((entry.url.clone(), entry.tab_id));
                }
            }
        });

        // Handle history click after rendering
        if let Some((url, tab_id)) = url_to_navigate {
            if app.browser_state.should_open_new_tab(tab_id) {
                // Open in new tab (different tab or tab doesn't exist) - desktop-only
                #[cfg(not(target_os = "android"))]
                {
                    app.add_browser_tab(ui.ctx(), frame, &url);
                    log::info!("Opened history in new tab: {}", url);
                }
            } else {
                // Navigate current tab (same tab)
                if let Some(idx) = app.browser_state.active_tab_index {
                    app.browser_state.update_tab_url(idx, &url, None);
                    #[cfg(not(target_os = "android"))]
                    {
                        let tab_id = app.browser_state.tabs[idx].id;
                        if let Some(view) = app.webview_widgets.get(&tab_id) {
                            let _ = view.view.load_url(&url);
                            log::info!("Navigated to history URL: {}", url);
                        }
                    }
                }
            }
        }
    }

    /// Render the browser UI
    ///
    /// Shows sidebar, toolbar, tabs, and webview content
    pub fn render_browser_ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        use egui::{CentralPanel, SidePanel, TopBottomPanel, ScrollArea, TextEdit};

        // 1. Left Sidebar Panel (collapsible, resizable)
        if self.browser_state.sidebar_open {
            SidePanel::left("browser_sidebar")
                .resizable(true)
                .default_width(200.0)
                .show_separator_line(true)
                .show_inside(ui, |ui| {
                    // Material Design tab switcher
                    ui.horizontal(|ui| {
                        if ui.selectable_label(
                            self.browser_state.sidebar_tab == SidebarTab::Bookmarks,
                            "📚 Bookmarks"
                        ).clicked() {
                            self.browser_state.sidebar_tab = SidebarTab::Bookmarks;
                        }

                        if ui.selectable_label(
                            self.browser_state.sidebar_tab == SidebarTab::History,
                            "🕒 History"
                        ).clicked() {
                            self.browser_state.sidebar_tab = SidebarTab::History;
                        }
                    });
                    ui.separator();

                    // Content area based on active tab
                    match self.browser_state.sidebar_tab {
                        SidebarTab::Bookmarks => Self::render_bookmarks_list(self, ui, frame),
                        SidebarTab::History => Self::render_history_list(self, ui, frame),
                    }
                });
        }

        // 2. Top Toolbar Panel (navigation controls)
        TopBottomPanel::top("browser_toolbar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                // Hamburger menu (moved from top app bar)
                let items_button = ui.add(
                    MaterialButton::filled(&crate::material_symbol_icons::icon_text("menu"))
                        .small()
                );
                self.items_button_rect = Some(items_button.rect);
                if items_button.clicked() {
                    self.standard_menu_open = !self.standard_menu_open;
                }
                self.show_menus(ui.ctx());

                // Sidebar toggle
                let sidebar_icon = if self.browser_state.sidebar_open {
                    crate::material_symbol_icons::icon_text("chevron_left")
                } else {
                    crate::material_symbol_icons::icon_text("chevron_right")
                };
                if ui.add(MaterialButton::filled(&sidebar_icon).small()).clicked() {
                    self.browser_state.sidebar_open = !self.browser_state.sidebar_open;
                }

                // Back button
                let can_go_back = Self::check_can_navigate_back(self);
                let back_btn = ui.add_enabled(
                    can_go_back,
                    MaterialButton::filled(&crate::material_symbol_icons::icon_text("arrow_back"))
                        .small()
                );
                #[cfg(not(target_os = "android"))]
                if back_btn.clicked() {
                    if let Some(idx) = self.browser_state.active_tab_index {
                        if let Err(e) = self.navigate_back(idx) {
                            log::warn!("Back navigation failed: {}", e);
                        } else {
                            self.browser_state.refresh_history();
                        }
                    }
                }

                // Forward button
                let can_go_forward = Self::check_can_navigate_forward(self);
                let fwd_btn = ui.add_enabled(
                    can_go_forward,
                    MaterialButton::filled(&crate::material_symbol_icons::icon_text("arrow_forward"))
                        .small()
                );
                #[cfg(not(target_os = "android"))]
                if fwd_btn.clicked() {
                    if let Some(idx) = self.browser_state.active_tab_index {
                        if let Err(e) = self.navigate_forward(idx) {
                            log::warn!("Forward navigation failed: {}", e);
                        } else {
                            self.browser_state.refresh_history();
                        }
                    }
                }

                // URL input
                ui.label("URL:");
                let response = ui.add(
                    TextEdit::singleline(&mut self.browser_state.url_input)
                        .id(egui::Id::new("browser_url_input"))  // Stable ID for focus detection
                        .desired_width(ui.available_width() - 120.0),
                );

                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)) {
                    if let Some(idx) = self.browser_state.active_tab_index {
                        let url = Self::ensure_protocol(&self.browser_state.url_input);
                        // Update metadata
                        self.browser_state.update_tab_url(idx, &url, None);
                        // Update URL input to show full URL
                        self.browser_state.url_input = url.clone();

                        let tab_id = self.browser_state.tabs[idx].id;

                        // Navigate webview widget (desktop-only)
                        #[cfg(not(target_os = "android"))]
                        {
                            if let Some(view) = self.webview_widgets.get(&tab_id) {
                                let _ = view.view.load_url(&url);
                                log::info!("Navigated tab {} to {}", idx, url);
                            }
                        }

                        // Navigate Android WebView (Android-only)
                        #[cfg(target_os = "android")]
                        {
                            if let Err(e) = self.android_webview_manager.navigate(&tab_id, &url) {
                                log::error!("Failed to navigate Android WebView: {}", e);
                            } else {
                                log::info!("Navigated Android tab {} to {}", idx, url);
                            }
                        }

                        self.browser_state.refresh_history();
                    }
                }

                // Go button
                if ui.add(MaterialButton::filled("Go").small()).clicked() {
                    if let Some(idx) = self.browser_state.active_tab_index {
                        let url = Self::ensure_protocol(&self.browser_state.url_input);
                        // Update metadata
                        self.browser_state.update_tab_url(idx, &url, None);
                        // Update URL input to show full URL
                        self.browser_state.url_input = url.clone();

                        let tab_id = self.browser_state.tabs[idx].id;

                        // Navigate webview widget (desktop-only)
                        #[cfg(not(target_os = "android"))]
                        {
                            if let Some(view) = self.webview_widgets.get(&tab_id) {
                                let _ = view.view.load_url(&url);
                                log::info!("Navigated tab {} to {}", idx, url);
                            }
                        }

                        // Navigate Android WebView (Android-only)
                        #[cfg(target_os = "android")]
                        {
                            if let Err(e) = self.android_webview_manager.navigate(&tab_id, &url) {
                                log::error!("Failed to navigate Android WebView: {}", e);
                            } else {
                                log::info!("Navigated Android tab {} to {}", idx, url);
                            }
                        }

                        self.browser_state.refresh_history();
                    }
                }

                // Bookmark button
                let bookmark_icon = crate::material_symbol_icons::icon_text("bookmark");
                if ui.add(MaterialButton::filled(&bookmark_icon).small()).clicked() {
                    if let Some(idx) = self.browser_state.active_tab_index {
                        let title = self.browser_state.tabs[idx].title.clone();
                        let url = self.browser_state.tabs[idx].url_bar.clone();
                        let _ = self.browser_state.add_bookmark(&title, &url);
                    }
                }
            });
        });

        // 3. Central Panel (tabs + browser content)
        CentralPanel::default().show_inside(ui, |ui| {
            if self.browser_state.tabs.is_empty() {
                // Empty state
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("No tabs open");
                        ui.label("Click '+' to create a new tab.");
                        ui.add_space(20.0);
                        #[cfg(not(target_os = "android"))]
                        if ui.button("+ New Tab").clicked() {
                            self.add_browser_tab(ui.ctx(), frame, "https://dure.app");
                        }
                    });
                });
            } else {
                // Tab bar with horizontal scrolling
                ScrollArea::horizontal().show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let mut tab_to_close: Option<usize> = None;

                        for (idx, tab) in self.browser_state.tabs.iter().enumerate() {
                            ui.horizontal(|ui| {
                                // Tab button with stripped protocol for cleaner display
                                let display_title = Self::strip_protocol(&tab.title);
                                let is_active = self.browser_state.active_tab_index == Some(idx);

                                if ui
                                    .selectable_label(is_active, display_title)
                                    .clicked()
                                {
                                    log::info!("Switching active tab from {:?} to {}",
                                               self.browser_state.active_tab_index, idx);
                                    self.browser_state.active_tab_index = Some(idx);
                                    self.browser_state.url_input = tab.url_bar.clone();
                                }

                                // Close button
                                if ui.small_button("×").clicked() {
                                    tab_to_close = Some(idx);
                                }
                            });
                            ui.add_space(2.0); // Add space between tabs
                        }

                        // + button to add new tab (desktop-only)
                        #[cfg(not(target_os = "android"))]
                        if ui.button("+").clicked() {
                            self.add_browser_tab(ui.ctx(), frame, "https://dure.app");
                        }

                        // Close tab after iteration (avoid borrow conflict)
                        if let Some(idx) = tab_to_close {
                            self.close_browser_tab(idx);
                        }
                    });
                });

                ui.separator();

                // Browser content area - render all webviews, show only active (desktop-only)
                #[cfg(not(target_os = "android"))]
                {
                    // Collect URL updates first to avoid borrow conflicts
                    let mut url_updates: Vec<(usize, String)> = Vec::new();

                    // Check if address bar has focus or menu is open
                    let address_bar_focused = ui.memory(|mem| {
                        mem.focused().map_or(false, |id| id == egui::Id::new("browser_url_input"))
                    });
                    let menu_open = self.standard_menu_open;
                    let dialog_open = self.dlg_settings.open;

                    for (idx, tab) in self.browser_state.tabs.iter().enumerate() {
                        let is_active = Some(idx) == self.browser_state.active_tab_index;

                        // Size control: hide webviews if address bar focused, menu open, or dialog open (fixes keyboard/z-index issues)
                        let size = if is_active && !address_bar_focused && !menu_open && !dialog_open {
                            ui.available_size()
                        } else {
                            egui::vec2(0.0, 0.0)
                        };

                        // Render webview (if exists for this tab)
                        if let Some(view) = self.webview_widgets.get_mut(&tab.id) {
                            ui.push_id(tab.id, |ui| {
                                let response = view.ui(ui, size);

                                // Handle navigation events (active tab only)
                                if is_active {
                                    for event in response.events {
                                        if let egui_webview::WebViewEvent::Loaded(new_url) = event {
                                            url_updates.push((idx, new_url));
                                        }
                                    }
                                }
                            });
                        }
                    }

                    // Apply URL updates after iteration (tab title will show URL)
                    for (idx, new_url) in url_updates {
                        self.browser_state.update_tab_url(idx, &new_url, Some(&new_url));
                        log::info!("Tab {} navigated to {}", idx, new_url);
                    }
                }
            }
        });
    }
}
