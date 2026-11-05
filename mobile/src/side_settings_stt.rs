pub struct SideSettings {
    // Font selector state
    pub selected_font_display: String,
    pub system_fonts: Vec<(String, String)>,
    pub system_fonts_loaded: bool,
    // Action results
    pub save_clicked: bool,
    pub theme_to_apply: Option<String>,
}

impl Default for SideSettings {
    fn default() -> Self {
        Self {
            selected_font_display: "Default (NotoSansKr)".to_string(),
            system_fonts: Vec::new(),
            system_fonts_loaded: false,
            save_clicked: false,
            theme_to_apply: None,
        }
    }
}
