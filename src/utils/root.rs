use std::process::{Command, Stdio};
use nix::unistd::Uid;

/// Only allow the program to be run as root
pub fn get_root_privs() {
    if Uid::current().is_root() {
        println!("We are root.");
        return;
    }
    let current_exe = std::env::current_exe().expect("Unable to get current executable path");
    let display = std::env::var("DISPLAY").unwrap_or_default();
    let xauthority = std::env::var("XAUTHORITY").unwrap_or_default();

    let args_str = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let command_line = format!(
        "DISPLAY={} XAUTHORITY={} {} {}",
        display,
        xauthority,
        current_exe.display(),
        args_str
    );

    let mut child = Command::new("pkexec")
        .arg("bash")
        .arg("-c")
        .arg(&command_line)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("Failed to start pkexec.");

    let status = child.wait().expect("Failed to wait on pkexec child process");

    if status.success() {
        std::process::exit(0);
    } else {
        eprintln!(
            "Failed to acquire root privileges via pkexec. Status: {:?}",
            status
        );
        std::process::exit(1);
    }
}