use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::process::Command;
use anyhow::Result;

struct DesktopEntryConfig {
    name: String,
    comment: String,
    exec: String,
    icon: String,
    terminal: bool,
    categories: Vec<String>,
}

impl DesktopEntryConfig {
    // Create a default configuration using Cargo.toml metadata
    fn default() -> Result<DesktopEntryConfig> {
        let name = env!("CARGO_PKG_NAME").to_string();
        let description = env!("CARGO_PKG_DESCRIPTION").to_string();
        let name_clone = name.clone();
        Ok(Self {
            name,
            comment: description,
            exec: format!("/usr/local/bin/{}", name_clone),
            icon: "icon.png".to_string(),
            terminal: false,
            categories: vec!["Utility".to_string(), "Development".to_string()],
        })
    }
}

fn title_case(input: &str) -> String {
    input
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first_char) => {
                    first_char.to_uppercase().collect::<String>() + chars.as_str().to_lowercase().as_str()
                }
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn add_desktop_entry(config: DesktopEntryConfig) -> Result<()> {
    // System-wide directory for .desktop files
    let applications_dir = PathBuf::from("/usr/share/applications");
    let desktop_path = applications_dir.join(format!("{}.desktop", config.name.to_lowercase()));

    // Create the directory if necessary
    fs::create_dir_all(&applications_dir)?;

    // System-wide icon directory for application icons
    let icon_dir = PathBuf::from("/usr/share/icons/hicolor/48x48/apps");
    fs::create_dir_all(&icon_dir)?;
    let icon_target = icon_dir.join(&config.icon);

    // Attempt to copy the icon from the project root directory
    let project_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source_icon_path = project_root.join(&config.icon);
    if source_icon_path.exists() {
        fs::copy(&source_icon_path, &icon_target)?;
        // Set readable and writable permissions for the owner, and readable for others
        fs::set_permissions(&icon_target, fs::Permissions::from_mode(0o644))?;
    } else {
        eprintln!("Warning: Icon file '{}' not found at {:?}", &config.icon, &source_icon_path);
    }

    // Construct the .desktop file content
    let desktop_content = format!(
        r#"[Desktop Entry]
Version=1.0
Type=Application
Name={name}
Comment={comment}
Exec={exec}
TryExec={exec}
X-Ubuntu-Gettext-Domain={exec}
SingleMainWindow=true
Icon={icon_name}
Terminal={terminal}
Categories={categories};
"#,
        name = title_case(&*config.name.replace("_", " ").replace(".", " ").replace("-", " ")),
        comment = config.comment,
        exec = config.exec,
        // For icons placed in hicolor theme directories, just use the basename without extension.
        icon_name = &icon_target.to_string_lossy(),
        terminal = if config.terminal { "true" } else { "false" },
        categories = config.categories.join(";")
    );

    // Write the .desktop file
    {
        let mut file = File::create(&desktop_path)?;
        file.write_all(desktop_content.as_bytes())?;
    }

    // Set permissions on the desktop file
    fs::set_permissions(&desktop_path, fs::Permissions::from_mode(0o644))?;

    println!("Desktop entry created at: {:?}", desktop_path);
    println!("Icon copied to: {:?}", icon_target);

    // Update desktop database
    let update_status = Command::new("update-desktop-database")
        .arg("/usr/local/share/applications")
        .status();

    match update_status {
        Ok(status) if status.success() => {
            println!("Desktop database updated successfully.");
        }
        Ok(status) => {
            eprintln!("update-desktop-database exited with non-zero status: {:?}", status);
        }
        Err(e) => {
            eprintln!("Failed to run update-desktop-database: {}", e);
        }
    }

    Ok(())
}

pub fn create_desktop() {
    // Create a default desktop entry config
    let config = match DesktopEntryConfig::default() {
        Ok(config) => config,
        Err(e) => {
            eprintln!("Failed to load default configuration: {}", e);
            return;
        }
    };

    // Add the desktop entry
    if let Err(e) = add_desktop_entry(config) {
        eprintln!("Failed to add desktop entry: {}", e);
    }
}
