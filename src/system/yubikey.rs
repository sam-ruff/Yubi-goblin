use crate::install::apt::check_dependencies;
use crate::models::models::YubiKey;
use crate::system::backup::{backup_file, restore_file_from_backup};
use crate::system::users::list_system_users;
use anyhow::{anyhow, Context, Result};
use rusb::{Context as UsbContext, UsbContext as _};
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;

/// Function to fetch YubiKeys from the system
pub fn fetch_yubikeys() -> Result<Vec<YubiKey>> {
    // Initialize the USB context
    let ctx = UsbContext::new().map_err(|_| anyhow!("Failed to initialize USB context"))?;

    // List devices
    let devices = ctx.devices().map_err(|_| anyhow!("Failed to list USB devices"))?;

    let mut found_yubikeys = Vec::new();
    let yubikey_vendor_id = 0x1050;

    // Iterate over the devices and check for YubiKeys
    for device in devices.iter() {
        let device_desc = match device.device_descriptor() {
            Ok(dd) => dd,
            Err(_) => continue,
        };

        // Check if the device is a YubiKey based on the vendor ID
        if device_desc.vendor_id() == yubikey_vendor_id {
            // Attempt to open the device
            let address = device.address();
            let handle = match device.open() {
                Ok(h) => h,
                Err(_) => continue, // Could not open device, skip it
            };

            // Get product string if available, fallback to "YubiKey"
            let product_string = match device_desc.product_string_index() {
                Some(_) => handle.read_product_string_ascii(&device_desc)
                    .unwrap_or_else(|_| "YubiKey".to_string()),
                None => "YubiKey".to_string(),
            };

            let usb_port = address as i32;

            found_yubikeys.push(YubiKey {
                name: product_string,
                usb_port,
            });
        }
    }

    Ok(found_yubikeys)
}

/// Install a YubiKey for the specified user.
pub async fn install_yubikey_for_user(username: &str) -> Result<()> {
    // Check if the YubiKey is already installed
    let is_yubikey_installed = check_if_yubikey_is_installed_for_user(username)?;
    if is_yubikey_installed {
        return Err(anyhow!("YubiKey is installed already for user {}", username));
    }

    // Ensure Yubico folder exists
    ensure_folder_location(username)?;

    // Derive the user's home directory
    let user_home: PathBuf = ["/home", username].iter().collect();
    let config_path = user_home.join(".config").join("Yubico");

    // Check if user exists
    let known_users = list_system_users()?;
    if !known_users.contains(&username.to_string()) {
        return Err(anyhow!("User '{}' does not exist on the system", username));
    }

    // Check if a YubiKey is connected
    let yubikeys = fetch_yubikeys().context("Failed to enumerate YubiKeys")?;
    if yubikeys.is_empty() {
        return Err(anyhow!("No YubiKeys detected on the system"));
    }

    // Check dependencies
    let deps = check_dependencies()?;
    if !(deps.apt && deps.libpam_u2f && deps.pamu2fcfg) {
        return Err(anyhow!("Not all required dependencies are installed"));
    }

    let u2f_keys_path = config_path.join("u2f_keys");

    // Run pamu2fcfg and capture output
    let output = Command::new("pamu2fcfg")
        .arg("-n")
        .env("HOME", user_home.to_str().unwrap())
        .output()
        .context("Failed to run pamu2fcfg")?;

    if !output.status.success() {
        return Err(anyhow!("pamu2fcfg command failed with a non-zero exit code"));
    }

    // Append output to u2f_keys file
    {
        let mut file_out = OpenOptions::new()
            .write(true)
            .append(true)
            .create(true)
            .open(&u2f_keys_path)
            .context("Failed to open u2f_keys file for writing")?;
        file_out.write_all(&output.stdout)?;
    }

    // chmod 600 on the u2f_keys file
    let chmod_status = Command::new("chmod")
        .arg("600")
        .arg(u2f_keys_path.to_str().unwrap())
        .status()
        .context("Failed to run chmod on u2f_keys file")?;

    if !chmod_status.success() {
        return Err(anyhow!("chmod command failed with a non-zero exit code"));
    }

    // Backup and edit /etc/pam.d/sudo
    backup_file("/etc/pam.d/sudo")?;
    add_pam_line("/etc/pam.d/sudo", "auth required pam_u2f.so")?;

    // Backup and edit /etc/pam.d/gdm-password
    backup_file("/etc/pam.d/gdm-password")?;
    add_pam_line("/etc/pam.d/gdm-password", "auth required pam_u2f.so")?;

    Ok(())
}

fn add_pam_line(file_path: &str, line_to_add: &str) -> Result<()> {
    // Read the existing file
    let file = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read {}", file_path))?;

    // Check if the line is already present
    if file.contains(line_to_add) {
        // Line already present, nothing to do
        return Ok(());
    }

    let mut lines: Vec<String> = file.lines().map(|s| s.to_string()).collect();

    // Try to insert before "@include common-auth" if found, otherwise insert at the top
    let mut insert_index = 0;
    for (i, l) in lines.iter().enumerate() {
        if l.contains("@include common-auth") {
            insert_index = i;
            break;
        }
    }

    lines.insert(insert_index, line_to_add.to_string());

    // Open the file with write and truncate options
    let mut file_out = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(file_path)
        .with_context(|| format!("Failed to open {} for writing", file_path))?;

    // Write the modified lines back to the file
    for l in lines {
        writeln!(file_out, "{}", l)?;
    }

    Ok(())
}

/// Ensures the `~/.config/Yubico` folder exists for the given `username`.
pub fn ensure_folder_location(username: &str) -> Result<()> {
    let user_home: PathBuf = ["/home", username].iter().collect();
    let config_path = user_home.join(".config").join("Yubico");

    if !config_path.exists() {
        fs::create_dir_all(&config_path)
            .with_context(|| format!("Failed to create directory {:?}", config_path))?;
    }

    Ok(())
}

/// Remove the YubiKey configuration for the specified user.
pub fn remove_yubikey_for_user(username: &str) -> Result<()> {
    // Check if user exists
    let known_users = list_system_users()?;
    if !known_users.contains(&username.to_string()) {
        return Err(anyhow!("User '{}' does not exist on the system", username));
    }

    // Check if the YubiKey is installed
    let is_yubikey_installed = check_if_yubikey_is_installed_for_user(username)?;
    if !is_yubikey_installed {
        return Err(anyhow!("YubiKey not installed for user {}", username));
    }

    // Restore the PAM configuration files from backup, if available
    restore_file_from_backup("/etc/pam.d/sudo")?;
    restore_file_from_backup("/etc/pam.d/gdm-password")?;

    // Derive the user's home directory
    let user_home: PathBuf = ["/home", username].iter().collect();
    let config_path = user_home.join(".config").join("Yubico");
    let u2f_keys_path = config_path.join("u2f_keys");

    // Remove the u2f_keys file if it exists
    if u2f_keys_path.exists() {
        fs::remove_file(&u2f_keys_path)
            .with_context(|| format!("Failed to remove file {:?}", u2f_keys_path))?;
    }

    // If desired, remove the Yubico directory if it's now empty
    if config_path.is_dir() && fs::read_dir(&config_path)?.next().is_none() {
        fs::remove_dir(&config_path)
            .with_context(|| format!("Failed to remove directory {:?}", config_path))?;
    }

    Ok(())
}

/// Checks if a YubiKey is considered "installed" for the specified user.
pub fn check_if_yubikey_is_installed_for_user(username: &str) -> Result<bool> {
    let user_home: PathBuf = ["/home", username].iter().collect();
    let u2f_keys_path = user_home.join(".config").join("Yubico").join("u2f_keys");

    // Check for `u2f_keys` file existence
    if !u2f_keys_path.exists() {
        return Ok(false);
    }

    // Check for the presence of the pam_u2f.so line in /etc/pam.d/sudo
    if !file_contains_line("/etc/pam.d/sudo", "auth required pam_u2f.so")? {
        return Ok(false);
    }

    // Check for the presence of the pam_u2f.so line in /etc/pam.d/gdm-password
    if !file_contains_line("/etc/pam.d/gdm-password", "auth required pam_u2f.so")? {
        return Ok(false);
    }

    // If all checks passed, consider YubiKey installed
    Ok(true)
}

/// Helper function to check if a given file contains a specified line.
fn file_contains_line(file_path: &str, line_to_check: &str) -> Result<bool> {
    let file_content = fs::read_to_string(file_path)
        .with_context(|| format!("Failed to read {}", file_path))?;
    Ok(file_content.contains(line_to_check))
}
