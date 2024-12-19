use std::fs;
use std::path::PathBuf;

/// Backup the given file by copying it to `<original_path>.bak`.
pub fn backup_file(file_path: &str) -> anyhow::Result<()> {
    let backup_path = format!("{}.bak", file_path);
    fs::copy(file_path, &backup_path)?;
    Ok(())
}


/// Restore the given file from `<original_path>.bak`.
pub fn restore_file_from_backup(file_path: &str) -> anyhow::Result<()> {
    let backup_path = format!("{}.bak", file_path);

    if !PathBuf::from(&backup_path).exists() {
        return Err(anyhow::anyhow!(
            "Backup file '{}' does not exist, cannot restore.",
            backup_path
        ));
    }

    fs::copy(&backup_path, file_path)?;
    Ok(())
}

