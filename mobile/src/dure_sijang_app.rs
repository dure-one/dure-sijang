#![doc(hidden)]

#[cfg(not(target_os = "android"))]
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use sys_locale::get_locale;

use eframe::egui;
use egui_i18n::tr;
use egui_material3::menu::{Corner, FocusState, Positioning};
use egui_material3::MaterialButton;
use egui_material3::{get_global_theme, ContrastLevel, ThemeMode};

use crate::LogLevel;

use crate::browser_stt::SidebarTab;
pub use crate::dure_sijang_app_stt::*;
use crate::material_symbol_icons::{ICON_INFO, ICON_SETTINGS};
use crate::{Config, Settings};

// Maximum number of tabs to prevent memory issues
const MAX_TABS: usize = 20;

// Maximum number of cached webviews (desktop-only, LRU eviction)
// Keeps memory usage bounded - each webview can use 50-200MB
#[cfg(not(target_os = "android"))]
const MAX_CACHED_WEBVIEWS: usize = 10;

// Active tab background alpha for better visibility (lighter than default)
const ACTIVE_TAB_BG_ALPHA: u8 = 60;

use crate::install;

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
    // Using subset font (12 icons, ~100KB) instead of full font (9.6MB)
    setup_local_fonts_from_bytes(
        "MaterialSymbolsOutlined",
        include_bytes!("../resources/MaterialSymbolsOutlined_subset.ttf"),
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
            side_settings: Default::default(),

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

            // WebView LRU cache tracking (desktop-only)
            #[cfg(not(target_os = "android"))]
            webview_last_access: HashMap::new(),

            // Android WebView manager
            #[cfg(target_os = "android")]
            android_webview_manager: crate::android_webview::AndroidWebViewManager::new(),

            // Browser state (MVVM)
            browser_state: crate::browser_stt::BrowserState::load_from_db(),

            // Tab switch debouncing and synchronization
            last_tab_switch_time: None,
            tab_switch_debounce_ms: 300, // 300ms debounce
            tab_switch_in_progress: false,
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

        // Initialize browser with default tabs
        app.initialize_browser();

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
        let is_desktop = available_width > crate::DESKTOP_MIN_WIDTH;
        log::trace!(
            "ui_internal called, available_width={}, is_desktop={}",
            available_width,
            is_desktop
        );

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
        let is_desktop = available_width > crate::DESKTOP_MIN_WIDTH;

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

        // Handle save and theme changes from settings sidebar
        if self.side_settings.save_clicked {
            self.side_settings.save_clicked = false; // Reset flag to prevent repeated saves
            self.save_settings();
        }
        if let Some(theme_name) = self.side_settings.theme_to_apply.take() {
            if theme_name == "default" {
                setup_local_theme(Some("resources/material-theme.json"));
                load_themes();
            } else {
                self.apply_theme_by_name(&theme_name);
            }
        }
        // === settings dialog end
    }

    // Controller method: prepare tab state (shared by both mobile and desktop UI)
    fn prepare_tabs_controller(&mut self) {
        // No-op: legacy tabs removed for mycart browser
    }

    fn render_desktop_tabs(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        log::trace!("render_desktop_tabs called");
        // Browser UI replaces legacy Dure-Sijang tabs
        self.render_browser_ui(ui, frame);
    }

    /// Parse RFC5424 log line to extract readable format
    /// Input: <134>1 2023-03-15T14:30:00.123Z - dure-sijang - - - [INFO] mobile: Started
    /// Output: [14:30:00.123] [INFO] mobile: Started
    fn parse_rfc5424_for_display(line: &str) -> String {
        // Try to parse RFC5424 format
        if let Some(msg_start) = line.find(" - - - ") {
            let msg_part = &line[msg_start + 7..]; // Skip " - - - "

            // Extract timestamp (between first space and third space)
            if let Some(first_space) = line.find(' ') {
                if let Some(second_space) = line[first_space + 1..].find(' ') {
                    let timestamp_full = &line[first_space + 1..first_space + 1 + second_space];

                    // Extract time portion (HH:MM:SS.mmm)
                    if let Some(time_start) = timestamp_full.find('T') {
                        if let Some(time_end) = timestamp_full.find('Z') {
                            let time = &timestamp_full[time_start + 1..time_end];
                            return format!("[{}] {}", time, msg_part);
                        }
                    }
                }
            }

            // Fallback: just show the message part
            return msg_part.to_string();
        }

        // Fallback: show original line if not RFC5424 format
        line.to_string()
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

        // Read from global log buffer and parse for pretty display
        let log_text = if let Ok(buffer) = get_log_buffer().lock() {
            // Parse each line from RFC5424 to readable format
            buffer
                .lines()
                .map(|line| Self::parse_rfc5424_for_display(line))
                .collect::<Vec<_>>()
                .join("\n")
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
            log::warn!("Config not available, cannot save settings");
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
        log::debug!(
            "UPDATE CALLED, first_update_done={}",
            self.first_update_done
        );

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
            if let Some(idx) = self.browser_state.active_tab_index {
                log::info!(
                    "First update - tabs: {}, active_tab_index is SOME: {}",
                    self.browser_state.tabs.len(),
                    idx
                );
            } else {
                log::info!(
                    "First update - tabs: {}, active_tab_index is NONE",
                    self.browser_state.tabs.len()
                );
            }
            self.first_update_done = true;

            // Initialize egui_webview for browser functionality (desktop-only)
            #[cfg(not(target_os = "android"))]
            {
                egui_webview::init_webview(ctx);
                log::info!("Initialized egui_webview");

                // Create default tab if no tabs exist (first run)
                if self.browser_state.tabs.is_empty() {
                    self.add_browser_tab(ctx, frame, "https://www.dure.app");
                    log::info!("Created default browser tab");
                } else {
                    // Tabs loaded from database - webviews created on-demand when switched to
                    log::info!(
                        "Loaded {} tabs from database",
                        self.browser_state.tabs.len()
                    );
                }
            }

            // Launch initial WebView for active tab (Android only)
            #[cfg(target_os = "android")]
            {
                if let Some(active_idx) = self.browser_state.active_tab_index {
                    if active_idx < self.browser_state.tabs.len() {
                        let url = self.browser_state.tabs[active_idx].url_bar.clone();
                        log::info!(
                            "First update - launching initial WebView for tab {} at {}",
                            active_idx,
                            url
                        );

                        if let Err(e) = crate::android_activity_embedding::launch_webview_activity(
                            &url,
                            active_idx as i32,
                        ) {
                            log::error!("Failed to launch initial WebView: {}", e);
                        } else {
                            log::info!("Successfully launched initial WebView");
                        }
                    }
                }
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

// ===== WebView Management Methods (Desktop) =====
impl DureSijangApp {
    /// Navigate webview back (desktop-only)
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

    /// Navigate webview forward (desktop-only)
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

    /// Evict least recently used webview from cache (desktop-only)
    #[cfg(not(target_os = "android"))]
    fn evict_lru_webview(&mut self) {
        if self.webview_last_access.is_empty() {
            return;
        }

        let lru_tab_id = self
            .webview_last_access
            .iter()
            .min_by_key(|(_, time)| *time)
            .map(|(id, _)| *id);

        if let Some(tab_id) = lru_tab_id {
            log::info!(
                "Evicting LRU webview {:?} (cache size: {})",
                tab_id,
                self.webview_widgets.len()
            );
            self.webview_widgets.remove(&tab_id);
            self.webview_last_access.remove(&tab_id);
        }
    }

    /// Add a new browser tab (desktop-only)
    #[cfg(not(target_os = "android"))]
    pub fn add_browser_tab(&mut self, ctx: &egui::Context, frame: &eframe::Frame, url: &str) {
        if self.browser_state.tabs.len() >= MAX_TABS {
            log::warn!("Cannot create tab: maximum {} tabs reached", MAX_TABS);
            return;
        }

        let tab_id = self.browser_state.add_tab(url, url);

        if self.webview_widgets.len() >= MAX_CACHED_WEBVIEWS {
            self.evict_lru_webview();
        }

        use egui_webview::EguiWebView;
        let view = EguiWebView::new(ctx, tab_id, frame, |builder| builder.with_url(url));
        self.webview_widgets.insert(tab_id, view);
        self.webview_last_access
            .insert(tab_id, std::time::SystemTime::now());
        log::info!(
            "Added browser tab {:?} with URL: {} (cache size: {})",
            tab_id,
            url,
            self.webview_widgets.len()
        );
    }
}

// ===== Browser UI Methods (MVVM Browser Integration) =====
impl DureSijangApp {
    /// Initialize browser with default tabs on first launch
    fn initialize_browser(&mut self) {
        // Load existing tabs from database (session restoration)
        self.browser_state = crate::browser_stt::BrowserState::load_from_db();

        // If no tabs exist (first launch or all closed), create defaults
        if self.browser_state.tabs.is_empty() {
            log::info!("No saved tabs - creating defaults");

            self.browser_state
                .add_tab("https://www.dure.one/business", "dure.one");
            self.browser_state
                .add_tab("https://www.dure.app", "dure.app");

            self.browser_state.active_tab_index = Some(0);
            self.browser_state.url_input = "https://www.dure.one/business".to_string();

            log::info!("Created {} default tabs", self.browser_state.tabs.len());
        } else {
            log::info!(
                "Restored {} tabs from database, active tab: {:?}",
                self.browser_state.tabs.len(),
                self.browser_state.active_tab_index
            );
        }
    }

    /// Add a new browser tab (Android - uses Activity Embedding)
    ///
    /// # Arguments
    /// * `ctx` - egui context
    /// * `frame` - eframe frame
    /// * `url` - Initial URL to load
    #[cfg(target_os = "android")]
    pub fn add_browser_tab(&mut self, ctx: &egui::Context, frame: &eframe::Frame, url: &str) {
        // Check tab limit
        if self.browser_state.tabs.len() >= MAX_TABS {
            log::warn!("Cannot create tab: maximum {} tabs reached", MAX_TABS);
            return;
        }

        // Create tab metadata
        let tab_id = self.browser_state.add_tab(url, url);

        // Android: Launch WebViewActivity via JNI
        let tab_index = self.browser_state.tabs.len() - 1;
        if let Err(e) =
            crate::android_activity_embedding::launch_webview_activity(url, tab_index as i32)
        {
            log::info!(
                "Failed to launch WebViewActivity for tab {}: {}",
                tab_index,
                e
            );
            // Remove tab metadata on failure
            self.browser_state.close_tab(tab_index);
        } else {
            log::info!("Launched WebViewActivity for new tab {}", tab_index);
        }
    }

    /// Close a browser tab by index
    ///
    /// # Arguments
    /// * `idx` - Tab index to close
    pub fn close_browser_tab(&mut self, idx: usize) {
        if idx >= self.browser_state.tabs.len() {
            log::warn!("Attempted to close invalid tab index {}", idx);
            return;
        }

        log::info!(
            "Closing tab {} (active: {:?})",
            idx,
            self.browser_state.active_tab_index
        );

        let tab_id = self.browser_state.tabs[idx].id;

        // Remove webview widget and last access tracking (desktop-only)
        #[cfg(not(target_os = "android"))]
        {
            if let Some(_view) = self.webview_widgets.remove(&tab_id) {
                self.webview_last_access.remove(&tab_id);
                log::info!("Destroyed webview for tab {:?}", tab_id);
            }
        }

        // If closing active tab on Android, destroy WebViewActivity first
        #[cfg(target_os = "android")]
        if self.browser_state.active_tab_index == Some(idx) {
            if let Err(e) = crate::android_activity_embedding::destroy_webview_activity() {
                log::warn!("Failed to destroy WebView when closing tab: {}", e);
            }
        }

        // Remove tab metadata
        self.browser_state.close_tab(idx);

        // If no tabs left, clear all cached webviews (desktop-only)
        #[cfg(not(target_os = "android"))]
        if self.browser_state.tabs.is_empty() {
            let count = self.webview_widgets.len();
            self.webview_widgets.clear();
            self.webview_last_access.clear();
            log::info!("Last tab closed - cleared all {} cached webviews", count);
        }

        // Calculate new active tab if we closed the active one
        if self.browser_state.active_tab_index == Some(idx) {
            let new_active = if idx > 0 {
                Some(idx - 1)
            } else if !self.browser_state.tabs.is_empty() {
                Some(0)
            } else {
                None
            };

            log::info!("New active tab after close: {:?}", new_active);

            // Launch WebView for new active tab on Android
            #[cfg(target_os = "android")]
            if let Some(new_idx) = new_active {
                let url = self.browser_state.tabs[new_idx].url_bar.clone();
                if let Err(e) =
                    crate::android_activity_embedding::launch_webview_activity(&url, new_idx as i32)
                {
                    log::error!("Failed to launch WebView for new active tab: {}", e);
                }
            }

            self.browser_state.active_tab_index = new_active;
            if let Some(idx) = new_active {
                self.browser_state.url_input = self.browser_state.tabs[idx].url_bar.clone();
            }
        }

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

    /// Render the menu content in the sidebar
    /// Render the about content in the sidebar
    fn render_about_content(app: &mut DureSijangApp, ui: &mut egui::Ui) {
        ui.heading(tr!("app-title"));
        ui.label(format!(
            "{}: {}",
            tr!("about-version"),
            env!("CARGO_PKG_VERSION")
        ));

        #[cfg(not(target_os = "android"))]
        {
            ui.horizontal(|ui| {
                if ui.button(tr!("about-check-update")).clicked() {
                    app.update_checking = true;
                    app.update_status = tr!("checking-update").to_string();
                    app.update_available = false;

                    let ctx = ui.ctx().clone();
                    std::thread::spawn(move || match crate::install::check_for_updates() {
                        Ok(update_info) => {
                            ctx.request_repaint();
                            log::info!(
                                "Update check complete: available={}",
                                update_info.available
                            );
                        }
                        Err(e) => {
                            log::error!("Update check failed: {}", e);
                        }
                    });
                }

                if app.update_checking {
                    ui.spinner();
                    ui.label(&app.update_status);
                } else if app.update_available {
                    ui.label(&app.update_status);
                } else if !app.update_status.is_empty() {
                    ui.label(&app.update_status);
                }
            });
        }

        ui.add_space(8.0);

        // Documentation and website links
        ui.horizontal(|ui| {
            ui.hyperlink_to(tr!("about-docs"), "https://dure.one/docs/");
            ui.label("|");
            ui.hyperlink_to(tr!("about-app-website"), "https://sijang.dure.app/");
        });

        ui.separator();

        // Cool tone markdown text color for dark/light mode
        let text_color = if ui.visuals().dark_mode {
            egui::Color32::from_rgb(210, 220, 230) // Cool light blue-gray
        } else {
            egui::Color32::from_rgb(40, 50, 60) // Cool dark blue-gray
        };

        // Code background color from theme
        let bg_color = ui.visuals().code_bg_color;

        // Set text color override BEFORE creating ScrollArea
        ui.ctx().style_mut(|style| {
            style.visuals.override_text_color = Some(text_color);
        });

        egui::Frame::new()
            .fill(bg_color)
            .inner_margin(egui::Margin::symmetric(12, 8))
            .show(ui, |ui| {
                egui::ScrollArea::both().show(ui, |ui| {
                    let credits_md = include_str!("../../CREDITS.md");

                    egui_markdown::MarkdownLabel::new(ui.id().with("credits_md"), credits_md)
                        .font(egui::FontId::proportional(14.0))
                        .selectable(true)
                        .show(ui);
                });
            });

        // Clear text color override after rendering
        ui.ctx().style_mut(|style| {
            style.visuals.override_text_color = None;
        });
    }

    /// Render the settings UI in the sidebar
    fn render_settings_content(app: &mut DureSijangApp, ui: &mut egui::Ui) {
        app.side_settings.render_ui(ui, &mut app.settings);
    }

    /// Render the bookmarks list in the sidebar
    fn render_bookmarks_list(
        app: &mut DureSijangApp,
        ui: &mut egui::Ui,
        frame: &mut eframe::Frame,
    ) {
        use egui::ScrollArea;

        let mut bookmark_to_navigate: Option<(String, String)> = None;
        let mut bookmark_to_delete: Option<usize> = None;

        ScrollArea::vertical().show(ui, |ui| {
            for (idx, bookmark) in app.browser_state.bookmarks.iter().enumerate() {
                ui.horizontal(|ui| {
                    if ui.button(&bookmark.title).clicked() {
                        bookmark_to_navigate = Some((bookmark.url.clone(), bookmark.title.clone()));
                    }
                    // Show creation timestamp
                    let time_str = bookmark.created_at.format("%m/%d %H:%M").to_string();
                    ui.label(format!("({})", time_str));
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
                ui.horizontal(|ui| {
                    if ui.button(title).clicked() {
                        url_to_navigate = Some((entry.url.clone(), entry.tab_id));
                    }
                    // Show visit timestamp
                    let time_str = entry.visited_at.format("%m/%d %H:%M").to_string();
                    ui.label(format!("({})", time_str));
                });
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
        use egui::{CentralPanel, ScrollArea, SidePanel, TextEdit, TopBottomPanel};

        // Process pending resize toggle (must happen before rendering sidebar)
        self.browser_state.process_resize_toggle();

        let is_desktop = ui.available_width() > crate::DESKTOP_MIN_WIDTH;

        if let Some(idx) = self.browser_state.active_tab_index {
            log::trace!(
                "render_browser_ui: {} tabs, active_tab_index=Some({})",
                self.browser_state.tabs.len(),
                idx
            );
        } else {
            log::trace!(
                "render_browser_ui: {} tabs, active_tab_index=None",
                self.browser_state.tabs.len()
            );
        }

        // on android put some top padding to avoid overlapping with system status bar
        #[cfg(target_os = "android")]
        {
            ui.add_space(30.0);
        }

        // 1. Left Sidebar (Menu | Bookmarks | History)
        if self.browser_state.sidebar_open {
            SidePanel::left("browser_sidebar")
                .resizable(true)
                .default_width(200.0)
                .show_separator_line(true)
                .show_inside(ui, |ui| {
                    // Tab switcher: Bookmarks | History | Settings | About
                    ui.horizontal(|ui| {
                        // Bookmarks tab
                        let is_bookmarks_active =
                            self.browser_state.sidebar_tab == SidebarTab::Bookmarks;
                        let mut bookmarks_button = egui::Button::new(tr!("bookmarks"));
                        if is_bookmarks_active {
                            let base_color = ui.style().visuals.widgets.inactive.bg_fill;
                            let active_color = egui::Color32::from_rgba_premultiplied(
                                base_color.r(),
                                base_color.g(),
                                base_color.b(),
                                ACTIVE_TAB_BG_ALPHA,
                            );
                            bookmarks_button = bookmarks_button.fill(active_color);
                        }
                        if ui.add(bookmarks_button).clicked() {
                            self.browser_state.sidebar_tab = SidebarTab::Bookmarks;
                        }

                        // History tab
                        let is_history_active =
                            self.browser_state.sidebar_tab == SidebarTab::History;
                        let mut history_button = egui::Button::new(tr!("history"));
                        if is_history_active {
                            let base_color = ui.style().visuals.widgets.inactive.bg_fill;
                            let active_color = egui::Color32::from_rgba_premultiplied(
                                base_color.r(),
                                base_color.g(),
                                base_color.b(),
                                ACTIVE_TAB_BG_ALPHA,
                            );
                            history_button = history_button.fill(active_color);
                        }
                        if ui.add(history_button).clicked() {
                            self.browser_state.sidebar_tab = SidebarTab::History;
                        }

                        // Settings tab
                        let is_settings_active =
                            self.browser_state.sidebar_tab == SidebarTab::Settings;
                        let mut settings_button =
                            egui::Button::new(format!("{} {}", ICON_SETTINGS, tr!("settings")));
                        if is_settings_active {
                            let base_color = ui.style().visuals.widgets.inactive.bg_fill;
                            let active_color = egui::Color32::from_rgba_premultiplied(
                                base_color.r(),
                                base_color.g(),
                                base_color.b(),
                                ACTIVE_TAB_BG_ALPHA,
                            );
                            settings_button = settings_button.fill(active_color);
                        }
                        if ui.add(settings_button).clicked() {
                            self.browser_state.sidebar_tab = SidebarTab::Settings;
                        }

                        // About tab
                        let is_about_active = self.browser_state.sidebar_tab == SidebarTab::About;
                        let mut about_button =
                            egui::Button::new(format!("{} {}", ICON_INFO, tr!("about")));
                        if is_about_active {
                            let base_color = ui.style().visuals.widgets.inactive.bg_fill;
                            let active_color = egui::Color32::from_rgba_premultiplied(
                                base_color.r(),
                                base_color.g(),
                                base_color.b(),
                                ACTIVE_TAB_BG_ALPHA,
                            );
                            about_button = about_button.fill(active_color);
                        }
                        if ui.add(about_button).clicked() {
                            self.browser_state.sidebar_tab = SidebarTab::About;
                        }
                    });
                    ui.separator();

                    // Content area based on active tab
                    match self.browser_state.sidebar_tab {
                        SidebarTab::Bookmarks => Self::render_bookmarks_list(self, ui, frame),
                        SidebarTab::History => Self::render_history_list(self, ui, frame),
                        SidebarTab::Settings => Self::render_settings_content(self, ui),
                        SidebarTab::About => Self::render_about_content(self, ui),
                    }
                });
        }

        // 2. tab header bar with horizontal scrolling
        ScrollArea::horizontal().show(ui, |ui| {
            ui.horizontal(|ui| {
                let mut tab_to_close: Option<usize> = None;

                for (idx, tab) in self.browser_state.tabs.iter().enumerate() {
                    ui.horizontal(|ui| {
                        // Tab button with stripped protocol for cleaner display
                        let display_title = Self::strip_protocol(&tab.title);
                        let is_active = self.browser_state.active_tab_index == Some(idx);

                        // Custom tab styling with lighter background for active tab
                        let mut tab_button = egui::Button::new(display_title);
                        if is_active {
                            // Lighter background for better visibility
                            let base_color = ui.style().visuals.widgets.inactive.bg_fill;
                            let active_color = egui::Color32::from_rgba_premultiplied(
                                base_color.r(),
                                base_color.g(),
                                base_color.b(),
                                ACTIVE_TAB_BG_ALPHA,
                            );
                            tab_button = tab_button.fill(active_color);
                        }

                        if ui.add(tab_button).clicked() {
                            // DEBOUNCE CHECK: Prevent rapid clicks
                            let now = std::time::Instant::now();
                            if let Some(last_switch) = self.last_tab_switch_time {
                                let elapsed = now.duration_since(last_switch).as_millis() as u64;
                                if elapsed < self.tab_switch_debounce_ms {
                                    log::warn!(
                                        "[TAB_SWITCH] Ignoring rapid click ({}ms < {}ms debounce)",
                                        elapsed,
                                        self.tab_switch_debounce_ms
                                    );
                                    return; // Skip this click
                                }
                            }

                            // CONCURRENT CHECK: Prevent overlapping switches
                            if self.tab_switch_in_progress {
                                log::warn!("[TAB_SWITCH] Switch already in progress, ignoring click");
                                return;
                            }

                            self.tab_switch_in_progress = true;
                            self.last_tab_switch_time = Some(now);

                            log::info!(
                                "[TAB_SWITCH] User clicked tab {} (current: {:?})",
                                idx,
                                self.browser_state.active_tab_index
                            );

                            #[cfg(target_os = "android")]
                            {
                                // Tab switching with wry WebView caching:
                                // - WebViewActivity is destroyed/recreated (Activity lifecycle)
                                // - wry WebView is cached and reused (AndroidWebViewManager)
                                // - SplitPlaceholderRule shows split dynamically on tab click
                                //
                                // Synchronization: ACTIVITY_LIFECYCLE_LOCK ensures destroy completes
                                // before launch starts, preventing race conditions

                                // Destroy old WebViewActivity
                                if let Err(e) =
                                    crate::android_activity_embedding::destroy_webview_activity()
                                {
                                    log::warn!("Failed to destroy old WebView: {}", e);
                                    self.tab_switch_in_progress = false; // Clear flag on error
                                    return;
                                }

                                // Small delay to let Android complete Activity.finish()
                                std::thread::sleep(std::time::Duration::from_millis(50));
                                log::debug!("[TAB_SWITCH] 50ms delay after destroy, before launch");

                                // Launch WebViewActivity for clicked tab
                                // onCreate() will get cached wry WebView from AndroidWebViewManager
                                let url = self.browser_state.tabs[idx].url_bar.clone();
                                if let Err(e) =
                                    crate::android_activity_embedding::launch_webview_activity(
                                        &url, idx as i32,
                                    )
                                {
                                    log::error!("Failed to switch to tab {}: {}", idx, e);
                                    self.tab_switch_in_progress = false; // Clear flag on error
                                    return;
                                } else {
                                    log::info!("[TAB_SWITCH] Switched to tab {} (wry WebView cached)", idx);
                                }
                            }

                            // Desktop: Destroy old webview (native windows can't be hidden)
                            #[cfg(not(target_os = "android"))]
                            {
                                // Destroy previous active webview if different tab
                                if let Some(prev_idx) = self.browser_state.active_tab_index {
                                    if prev_idx != idx && prev_idx < self.browser_state.tabs.len() {
                                        let prev_tab_id = self.browser_state.tabs[prev_idx].id;
                                        if self.webview_widgets.remove(&prev_tab_id).is_some() {
                                            self.webview_last_access.remove(&prev_tab_id);
                                            log::info!(
                                                "Destroyed webview for tab {} on switch",
                                                prev_idx
                                            );
                                        }
                                    }
                                }
                                log::info!(
                                    "Switched to tab {} (webview will be created on-demand)",
                                    idx
                                );
                            }

                            self.browser_state.active_tab_index = Some(idx);
                            // Use the clicked tab's URL, not the iterator's current tab
                            self.browser_state.url_input =
                                self.browser_state.tabs[idx].url_bar.clone();

                            // Clear in-progress flag
                            self.tab_switch_in_progress = false;
                        }

                        // Close button
                        if ui.small_button("×").clicked() {
                            tab_to_close = Some(idx);
                        }
                    });
                    ui.add_space(2.0); // Add space between tabs
                }

                // + button to add new tab
                let can_add_tab = self.browser_state.tabs.len() < MAX_TABS;
                let add_button = ui.add_enabled(can_add_tab, egui::Button::new("+"));
                if add_button.clicked() {
                    self.add_browser_tab(ui.ctx(), frame, "https://www.dure.app");
                }
                if !can_add_tab {
                    add_button.on_disabled_hover_text(
                        "Maximum 20 tabs. Close some tabs to create new ones.",
                    );
                }

                // Close tab after iteration (avoid borrow conflict)
                if let Some(idx) = tab_to_close {
                    self.close_browser_tab(idx);
                }
            });
        });

        ui.separator();

        // 3. Top Toolbar Panel (navigation controls)
        TopBottomPanel::top("browser_toolbar").show_inside(ui, |ui| {
            ui.horizontal(|ui| {
                // Hamburger menu button - toggles sidebar
                let items_button = ui.add(
                    MaterialButton::filled(&crate::material_symbol_icons::icon_text("menu"))
                        .small(),
                );
                if items_button.clicked() {
                    self.browser_state.toggle_sidebar();
                }

                // Back button
                let can_go_back = Self::check_can_navigate_back(self);
                let back_btn = ui.add_enabled(
                    can_go_back,
                    MaterialButton::filled(&crate::material_symbol_icons::icon_text("arrow_back"))
                        .small(),
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
                    MaterialButton::filled(&crate::material_symbol_icons::icon_text(
                        "arrow_forward",
                    ))
                    .small(),
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
                ui.label(format!("{}:", tr!("url-label")));
                let response = ui.add(
                    TextEdit::singleline(&mut self.browser_state.url_input)
                        .id(egui::Id::new("browser_url_input"))
                        .desired_width(ui.available_width() - 120.0),
                );

                #[cfg(target_os = "android")]
                {
                    if response.gained_focus() {
                        let _ = crate::android_inputmethod::show_soft_input();
                    }
                    if response.lost_focus() {
                        let _ = crate::android_inputmethod::hide_soft_input();
                    }
                }
                crate::clipboard_popup::show_clipboard_popup(
                    ui,
                    &response,
                    &mut self.browser_state.url_input,
                );

                // Handle Enter key to navigate (works when TextEdit has focus OR when it just lost focus)
                let enter_pressed =
                    response.has_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter));
                if (response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                    || enter_pressed
                {
                    let url = Self::ensure_protocol(&self.browser_state.url_input);

                    // If no tabs exist, create a new tab
                    if self.browser_state.tabs.is_empty() {
                        self.add_browser_tab(ui.ctx(), frame, &url);
                        log::info!("Created new tab for URL: {}", url);
                    } else if let Some(idx) = self.browser_state.active_tab_index {
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

                        // Navigate Android WebView via Activity Embedding
                        #[cfg(target_os = "android")]
                        {
                            if let Err(e) =
                                crate::android_activity_embedding::navigate_webview(&url)
                            {
                                log::error!("Failed to navigate WebView to {}: {}", url, e);
                            } else {
                                log::info!("WebView navigation successful to {}", url);
                            }
                        }

                        self.browser_state.refresh_history();
                    }
                }

                // Go button
                if ui
                    .add(MaterialButton::filled(tr!("url-go")).small())
                    .clicked()
                {
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

                        // Navigate Android WebView via Activity Embedding
                        #[cfg(target_os = "android")]
                        {
                            if let Err(e) =
                                crate::android_activity_embedding::navigate_webview(&url)
                            {
                                log::error!("Failed to navigate WebView to {}: {}", url, e);
                            } else {
                                log::info!("WebView navigation successful to {}", url);
                            }
                        }

                        self.browser_state.refresh_history();
                    }
                }

                // Bookmark button - toggle bookmark for active tab
                let is_bookmarked = if let Some(idx) = self.browser_state.active_tab_index {
                    self.browser_state
                        .is_bookmarked(&self.browser_state.tabs[idx].url_bar)
                } else {
                    false
                };
                let bookmark_icon = if is_bookmarked { "🔖" } else { "📑" };
                if ui
                    .add(MaterialButton::filled(bookmark_icon).small())
                    .clicked()
                {
                    if let Some(idx) = self.browser_state.active_tab_index {
                        let title = self.browser_state.tabs[idx].title.clone();
                        let url = self.browser_state.tabs[idx].url_bar.clone();
                        let _ = self.browser_state.add_bookmark(&title, &url);
                    }
                }
            });
        });

        // 4. Central Panel (tabs + browser content)
        CentralPanel::default().show_inside(ui, |ui| {
            if self.browser_state.tabs.is_empty() {
                // Empty state
                ui.centered_and_justified(|ui| {
                    ui.vertical_centered(|ui| {
                        ui.heading("No tabs open");
                        ui.label("Click '+' to create a new tab.");
                        ui.add_space(20.0);
                        let can_add_tab = self.browser_state.tabs.len() < MAX_TABS;
                        let add_button =
                            ui.add_enabled(can_add_tab, egui::Button::new("+ New Tab"));
                        if add_button.clicked() {
                            self.add_browser_tab(ui.ctx(), frame, "https://www.dure.app");
                        }
                        if !can_add_tab {
                            add_button.on_disabled_hover_text(
                                "Maximum 20 tabs. Close some tabs to create new ones.",
                            );
                        }
                    });
                });
            } else {
                // Browser content area - single active webview (desktop-only, mirrors Android pattern)
                #[cfg(not(target_os = "android"))]
                {
                    if let Some(active_idx) = self.browser_state.active_tab_index {
                        let active_tab_id = self.browser_state.tabs[active_idx].id;
                        let active_tab_url = self.browser_state.tabs[active_idx].url_bar.clone();

                        // Create webview if doesn't exist for active tab
                        if !self.webview_widgets.contains_key(&active_tab_id) {
                            // Check cache size and evict LRU if needed
                            if self.webview_widgets.len() >= MAX_CACHED_WEBVIEWS {
                                self.evict_lru_webview();
                            }

                            use egui_webview::EguiWebView;
                            let view =
                                EguiWebView::new(ui.ctx(), active_tab_id, frame, |builder| {
                                    builder.with_url(&active_tab_url)
                                });
                            self.webview_widgets.insert(active_tab_id, view);
                            self.webview_last_access
                                .insert(active_tab_id, std::time::SystemTime::now());
                            log::info!(
                                "Created webview for tab {} on demand (cache size: {})",
                                active_idx,
                                self.webview_widgets.len()
                            );
                        }

                        // Update last access time for active webview
                        self.webview_last_access
                            .insert(active_tab_id, std::time::SystemTime::now());

                        // Render ONLY active webview with size adjusted for log panel
                        // Inactive webviews are not rendered (hidden by not calling ui())
                        if let Some(view) = self.webview_widgets.get_mut(&active_tab_id) {
                            let mut size = ui.available_size();

                            // Reserve space for log panel when enabled (200px total: label + 150px scroll area + padding)
                            if self.settings.show_logs {
                                size.y = (size.y - 200.0).max(0.0);
                            }

                            log::trace!(
                                "Rendering webview for tab {} (id={:?}), size={:?}, show_logs={}",
                                active_idx,
                                active_tab_id,
                                size,
                                self.settings.show_logs
                            );

                            ui.push_id(active_tab_id, |ui| {
                                let response = view.ui(ui, size);

                                // Handle navigation events
                                for event in response.events {
                                    if let egui_webview::WebViewEvent::Loaded(new_url) = event {
                                        self.browser_state.update_tab_url(
                                            active_idx,
                                            &new_url,
                                            Some(&new_url),
                                        );
                                        log::info!("Tab {} navigated to {}", active_idx, new_url);
                                    }
                                }
                            });
                        }
                    }
                }
            }
        });
    }
}
