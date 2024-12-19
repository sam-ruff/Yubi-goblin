use anyhow::{Context, Result};
use std::process::{Command, Stdio};
use log::debug;
use crate::models::models::Dependencies;

/// Checks if the given package is installed using `dpkg -s`.
fn is_package_installed(package: &str) -> Result<bool> {
    let status = Command::new("dpkg")
        .arg("-s")
        .arg(package)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .with_context(|| format!("Failed to run `dpkg -s {}`", package))?;

    Ok(status.success())
}

/// Checks if `apt` is available using `which`.
fn is_apt_available() -> Result<bool> {
    let status = Command::new("which")
        .arg("apt")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("Failed to run `which apt`")?;

    Ok(status.success())
}

/// Checks dependencies and returns a `Dependencies` struct.
pub fn check_dependencies() -> Result<Dependencies> {
    let apt_installed = is_apt_available()?;
    let libpam_u2f_installed = is_package_installed("libpam-u2f")?;
    let pamu2fcfg_installed = is_package_installed("pamu2fcfg")?;

    Ok(Dependencies {
        apt: apt_installed,
        libpam_u2f: libpam_u2f_installed,
        pamu2fcfg: pamu2fcfg_installed,
    })
}

/// Installs the given packages using `apt update` and `apt install`.
/// Returns Ok(()) on success, or an Err with context on failure.
pub fn install_packages(packages: &[&str]) -> Result<()> {
    // Run `apt update`
    Command::new("apt")
        .arg("update")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("Failed to run `apt update`")?
        .success()
        .then_some(())
        .ok_or_else(|| anyhow::anyhow!("`apt update` command did not complete successfully."))?;

    // Prepare `apt install -y <packages>`
    let mut install_cmd = Command::new("apt");
    install_cmd.arg("install").arg("-y");
    for pkg in packages {
        install_cmd.arg(pkg);
    }

    install_cmd
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("Failed to run `apt install`")?
        .success()
        .then_some(())
        .ok_or_else(|| anyhow::anyhow!("`apt install` command failed."))
}

/// Removes the given packages using `apt remove -y`.
/// Returns Ok(()) on success, or an Err with context on failure.
pub fn remove_packages(packages: &[&str]) -> Result<()> {
    // Prepare `apt remove -y <packages>`
    let mut remove_cmd = Command::new("apt");
    remove_cmd.arg("remove").arg("-y");
    for pkg in packages {
        debug!("Removing package {}", pkg);
        remove_cmd.arg(pkg);
    }

    remove_cmd
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .context("Failed to run `apt remove`")?
        .success()
        .then_some(())
        .ok_or_else(|| anyhow::anyhow!("`apt remove` command failed."))
}