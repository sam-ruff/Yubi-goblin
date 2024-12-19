use std::fs;

/// Lists all users on the system
pub fn list_system_users() -> anyhow::Result<Vec<String>> {
    let passwd_contents = fs::read_to_string("/etc/passwd")?;
    let mut users = Vec::new();

    for line in passwd_contents.lines() {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() >= 7 {
            let name = parts[0];
            let uid_str = parts[2];
            let shell = parts[6];

            if let Ok(uid) = uid_str.parse::<u32>() {
                if uid >= 1000 && !shell.ends_with("nologin") && !shell.ends_with("false") {
                    users.push(name.to_string());
                }
            }
        }
    }

    Ok(users)
}