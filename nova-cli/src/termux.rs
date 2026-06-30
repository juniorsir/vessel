use std::process::Command;
use std::env;
use crate::PatraConfig;

/// Detects if the CLI is running inside a Termux environment on Android
pub fn is_termux() -> bool {
    env::var("PREFIX").map_or(false, |p| p.contains("com.termux"))
}

/// Dynamically relocates the rootfs storage to the Termux Prefix to bypass Android Read-Only limitations
pub fn get_base_dir() -> String {
    if is_termux() {
        format!("{}/var/lib/vessel", env::var("PREFIX").unwrap_or_default())
    } else {
        "/var/lib/vessel".to_string()
    }
}

/// Dynamically locates a writeable temporary directory
pub fn get_tmp_dir() -> String {
    if is_termux() {
        env::var("TMPDIR").unwrap_or_else(|_| "/data/data/com.termux/files/usr/tmp".to_string())
    } else {
        "/tmp".to_string()
    }
}

/// Routes sandbox execution through PRoot when root-level namespaces are unavailable
pub(crate) async fn run_direct(config: PatraConfig) -> Result<(), Box<dyn std::error::Error>> {
    println!("\x1b[1m\x1b[33m[vessel-termux]\x1b[0m Termux environment detected. Adapting to unprivileged PRoot execution...");
    
    // Ensure PRoot is installed inside Termux
    if Command::new("proot").arg("--help").output().is_err() {
        println!("\x1b[1m\x1b[31m[vessel-termux] Error: Core package 'proot' is not installed.\x1b[0m");
        println!("Please install it by running: \x1b[1mpkg install proot\x1b[0m\n");
        std::process::exit(1);
    }

    // Warn the user about kernel features that must be bypassed in user-space
    if config.shakti != 1.0 || config.smriti != 512 * 1024 * 1024 {
        println!("\x1b[33m  ⚠ Kernel Limits (Smriti/Shakti) are unavailable in unprivileged user-space. Ignored.\x1b[0m");
    }
    if !config.dwar.is_empty() {
        println!("\x1b[33m  ⚠ Network bridging (Dwar) is unavailable. Falling back to host networking.\x1b[0m");
    }
    
    println!("\x1b[1m\x1b[32m✔ Termux adapter ready. Booting Virtual Environment...\x1b[0m\n");

    let mut proot_cmd = Command::new("proot");
    
    // Build the PRoot Virtualization Chain
    proot_cmd.arg("--link2symlink")
             .arg("-0")               // Fake root context (UID/GID 0)
             .arg("-r").arg(&config.mool)
             .arg("-b").arg("/dev")
             .arg("-b").arg("/proc")
             .arg("-w").arg("/");

    // Add Android's system resolver if DNS is needed
    if std::path::Path::new("/system/etc/resolv.conf").exists() {
        proot_cmd.arg("-b").arg("/system/etc/resolv.conf:/etc/resolv.conf");
    }

    // Resolve any custom 'Sanchay' (mount) definitions mapped by the user
    for mount_spec in &config.sanchay {
        if let Some((host_path, guest_path)) = mount_spec.split_once(':') {
            proot_cmd.arg("-b").arg(format!("{}:{}", host_path, guest_path));
        }
    }
    
    // Inject the 'Paryavaran' (Environment variables)
    for (k, v) in &config.paryavaran {
        proot_cmd.env(k, v);
    }

    // Hand-off execution to the requested task
    proot_cmd.arg(&config.karya[0]);
    if config.karya.len() > 1 {
        proot_cmd.args(&config.karya[1..]);
    }

    let mut child = proot_cmd.spawn()?;
    child.wait()?;

    Ok(())
}
