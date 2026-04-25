use anyhow::{anyhow, Result};
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::path::{Path, PathBuf};

/// Pack a skill directory into a .cogtome archive
pub fn pack(skill_name: &str, skills_dir: &Path, output: Option<PathBuf>) -> Result<PathBuf> {
    let source = skills_dir.join(skill_name);
    if !source.exists() {
        anyhow::bail!("Skill '{}' not found at {}", skill_name, source.display());
    }

    let output = output.unwrap_or_else(|| PathBuf::from(format!("{skill_name}.cogtome")));

    let file = std::fs::File::create(&output)?;
    let encoder = GzEncoder::new(file, Compression::default());
    let mut tar = tar::Builder::new(encoder);

    tar.append_dir_all(skill_name, &source)?;
    tar.finish()?;

    Ok(output)
}

/// Install a .cogtome archive to the skills directory
pub fn install(package_path: &Path, skills_dir: &Path) -> Result<()> {
    let file = std::fs::File::open(package_path)?;
    let decoder = GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);

    // Unpack to a temp directory first, then move
    let temp_dir = std::env::temp_dir().join("cogtome-install-".to_string() + &unique_id());
    std::fs::create_dir_all(&temp_dir)?;

    // Safe unpacking: validate each entry path to prevent zip slip
    for entry in archive.entries()? {
        let mut entry = entry.map_err(|e| anyhow!("Failed to read archive entry: {}", e))?;
        let entry_path = entry.path()
            .map_err(|e| anyhow!("Failed to get entry path: {}", e))?
            .into_owned();

        // Security: prevent path traversal attacks
        let dest_path = temp_dir.join(&entry_path);
        if !dest_path.starts_with(&temp_dir) {
            anyhow::bail!("Archive contains invalid path that escapes target directory: {}", entry_path.display());
        }

        entry.unpack(&dest_path).map_err(|e| anyhow!("Failed to unpack entry: {}", e))?;
    }

    // Find the unpacked directory (should be the skill name directory)
    let entries = std::fs::read_dir(&temp_dir)?;
    let mut skill_dir = None;
    for entry in entries.flatten() {
        if entry.path().is_dir() {
            skill_dir = Some(entry.path());
            break;
        }
    }

    let skill_dir = skill_dir.ok_or_else(|| anyhow::anyhow!("Invalid package: no directory found"))?;
    let skill_name = skill_dir.file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid package: cannot determine skill name"))?;

    // Move to skills directory
    let dest = skills_dir.join(skill_name);
    if dest.exists() {
        anyhow::bail!("Skill '{}' already exists at {}", skill_name, dest.display());
    }

    std::fs::rename(&skill_dir, &dest)?;
    std::fs::remove_dir_all(&temp_dir)?;

    Ok(())
}

/// Generate a unique ID for temp directory naming
fn unique_id() -> String {
    uuid::Uuid::new_v4().to_string()
}
