// build.rs

use std::error::Error;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn Error>> {
    // Windows-specific resource compilation
    #[cfg(target_os = "windows")]
    {
        extern crate winres;
        let mut res = winres::WindowsResource::new();
        res.set_icon("resources/logo.ico");
        res.compile()?;
    }

    // Font subsetting: Material Symbols (12 icons only)
    subset_material_symbols()?;

    // Android: Copy fonts to assets directory for runtime loading
    #[cfg(target_os = "android")]
    copy_fonts_to_android_assets()?;

    Ok(())
}

fn subset_material_symbols() -> Result<(), Box<dyn Error>> {
    use std::collections::HashSet;

    let source_font = Path::new("resources/MaterialSymbolsOutlined[FILL,GRAD,opsz,wght].ttf");
    let subset_font = Path::new("resources/MaterialSymbolsOutlined_subset.ttf");

    // Load original font binary data
    let font_data = fs::read(source_font)?;

    // Define the 12 Material Icons we use for browser UI
    let chars_to_keep: HashSet<char> = [
        '\u{E5D2}', // menu (hamburger menu)
        '\u{E8B8}', // settings
        '\u{E5C4}', // arrow_back (browser back)
        '\u{E5C8}', // arrow_forward (browser forward)
        '\u{E5D5}', // refresh (reload page)
        '\u{E5CD}', // close (close tab)
        '\u{E8B6}', // search
        '\u{E838}', // star (bookmark filled)
        '\u{E83A}', // star_border (bookmark outline)
        '\u{E88A}', // home
        '\u{E145}', // add (new tab)
        '\u{E88E}', // info
    ]
    .iter()
    .copied()
    .collect();

    // Perform font subsetting using fontcull (no OpenType features needed)
    let subsetted_bytes = fontcull::subset_font_data(&font_data, &chars_to_keep, &[])?;

    // Write the subsetted font
    fs::write(subset_font, subsetted_bytes)?;

    // Rebuild if source font changes
    println!("cargo:rerun-if-changed={}", source_font.display());

    Ok(())
}

#[cfg(target_os = "android")]
fn copy_fonts_to_android_assets() -> Result<(), Box<dyn Error>> {
    use std::env;
    use std::path::PathBuf;

    // Use CARGO_MANIFEST_DIR to find project root, then navigate to Android assets
    // This ensures fonts are copied to app/src/main/assets/ (Gradle source directory)
    // not target/.../assets/ (Cargo build directory)
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let assets_dir = PathBuf::from(&manifest_dir).join("app/src/main/assets");

    // Create assets directory if it doesn't exist
    fs::create_dir_all(&assets_dir)?;

    // Copy subset Material Symbols (~100KB vs 9.6MB)
    let subset_font = Path::new("resources/MaterialSymbolsOutlined_subset.ttf");
    let dest_material = assets_dir.join("MaterialSymbolsOutlined_subset.ttf");
    fs::copy(subset_font, &dest_material)?;

    // Rebuild if fonts change
    println!("cargo:rerun-if-changed={}", subset_font.display());

    Ok(())
}
