use std::fs;

use std::path::Path;
use anyhow::Result;

// Embedded themes archive as a byte array
const THEMES_ARCHIVE: &[u8] = include_bytes!("../themes.zip");

pub fn extract_themes_if_needed() -> Result<()> {
    if let Some(mut themes_dir) = dirs::config_dir() {
        themes_dir.push("halo/themes");
        
        // Only extract if themes directory doesn't exist or is empty
        if !themes_dir.exists() || themes_dir.read_dir()?.next().is_none() {
            extract_themes_archive(&themes_dir)?;
        }
    }
    Ok(())
}

fn extract_themes_archive(themes_dir: &Path) -> Result<()> {
    // Create themes directory
    fs::create_dir_all(themes_dir)?;
    
    // Read the zip archive
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(THEMES_ARCHIVE))?;
    
    // Extract each file
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let file_path = themes_dir.join(file.name());
        
        // Create parent directories if needed
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        
        // Write the file
        let mut output_file = fs::File::create(&file_path)?;
        std::io::copy(&mut file, &mut output_file)?;
    }
    
    Ok(())
}

pub fn refresh_themes() -> Result<()> {
    if let Some(mut themes_dir) = dirs::config_dir() {
        themes_dir.push("halo/themes");
        
        // Remove existing themes directory
        if themes_dir.exists() {
            fs::remove_dir_all(&themes_dir)?;
        }
        
        // Extract fresh themes
        extract_themes_archive(&themes_dir)?;
    }
    Ok(())
}
