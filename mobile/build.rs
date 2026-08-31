// build.rs

use std::error::Error;
use std::fs;
use std::io::Read;
use std::path::Path;
use std::time::SystemTime;

fn main() -> Result<(), Box<dyn Error>> {
    // Download fallback resources for offline usage
    download_fallback_resources();

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

fn download_fallback_resources() {
    const UAD_LISTS_URL: &str = "https://cdn.jsdelivr.net/gh/0x192/universal-android-debloater@latest/resources/assets/uad_lists.json";
    const STALKERWARE_IOC_URL: &str =
        "https://raw.githubusercontent.com/AssoEchap/stalkerware-indicators/master/ioc.yaml";

    let resources_dir = Path::new("resources");

    // Ensure resources directory exists
    if let Err(e) = fs::create_dir_all(resources_dir) {
        eprintln!("Warning: Failed to create resources directory: {}", e);
        return;
    }

    // Download UAD lists
    download_if_needed(
        UAD_LISTS_URL,
        &resources_dir.join("uad_lists.json"),
        "UAD lists",
    );

    // Download Stalkerware IoC
    download_if_needed(
        STALKERWARE_IOC_URL,
        &resources_dir.join("stalkerware_ioc.yaml"),
        "Stalkerware IoC",
    );

    // Tell Cargo to rerun this build script if the files are deleted
    println!("cargo:rerun-if-changed=resources/uad_lists.json");
    println!("cargo:rerun-if-changed=resources/stalkerware_ioc.yaml");
}

fn download_if_needed(url: &str, file_path: &Path, description: &str) {
    // Check if file exists and is recent (less than 7 days old)
    let should_download = if file_path.exists() {
        match fs::metadata(file_path) {
            Ok(metadata) => {
                match metadata.modified() {
                    Ok(modified) => {
                        match SystemTime::now().duration_since(modified) {
                            Ok(age) => {
                                // Download if older than 7 days
                                age.as_secs() > 7 * 24 * 60 * 60
                            }
                            Err(_) => false, // Can't determine age, keep existing
                        }
                    }
                    Err(_) => false, // Can't get modification time, keep existing
                }
            }
            Err(_) => true, // Can't read metadata, try to download
        }
    } else {
        true // File doesn't exist, download
    };

    if !should_download {
        println!(
            "cargo:warning={} is up-to-date at {:?}",
            description, file_path
        );
        return;
    }

    println!("cargo:warning=Downloading {} from {}", description, url);

    match ureq::get(url).set("User-Agent", "dure-sijang/1.0").call() {
        Ok(response) => {
            let mut buffer = Vec::new();
            if let Err(e) = response.into_reader().read_to_end(&mut buffer) {
                eprintln!("Warning: Failed to read {} response: {}", description, e);
                if !file_path.exists() {
                    eprintln!("ERROR: {} not available and download failed!", description);
                }
                return;
            }

            match fs::write(file_path, &buffer) {
                Ok(_) => println!(
                    "cargo:warning=Successfully downloaded {} to {:?}",
                    description, file_path
                ),
                Err(e) => {
                    eprintln!(
                        "Warning: Failed to write {} to {:?}: {}",
                        description, file_path, e
                    );
                    if !file_path.exists() {
                        eprintln!(
                            "ERROR: {} not available and cannot write file!",
                            description
                        );
                    }
                }
            }
        }
        Err(e) => {
            eprintln!("Warning: Failed to download {}: {}", description, e);
            if !file_path.exists() {
                eprintln!("ERROR: {} not available and download failed!", description);
                eprintln!("       Please ensure network connectivity or manually download:");
                eprintln!("       curl -o {:?} \"{}\"", file_path, url);
            } else {
                eprintln!("       Using existing file at {:?}", file_path);
            }
        }
    }
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
