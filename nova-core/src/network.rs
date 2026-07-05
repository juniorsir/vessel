use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
use std::process::id as pid;

/// Generates a randomized IEEE 802 locally administered MAC address using 
/// zero external dependencies (hashing nanosecond timestamps and Process IDs).
pub fn generate_random_mac() -> String {
    let start = SystemTime::now();
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap_or_default();
    let seed = since_the_epoch.as_nanos() as u64 ^ (pid() as u64);
    
    // Setting the second nibble to '2', '6', 'A', or 'E' ensures switches/bridges 
    // recognize it as a valid unicast local address.
    let local_nibbles = [0x02, 0x06, 0x0A, 0x0E];
    let first_byte = local_nibbles[((seed >> 3) % 4) as usize];
    
    format!(
        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        first_byte,
        ((seed >> 8) & 0xff) as u8,
        ((seed >> 16) & 0xff) as u8,
        ((seed >> 24) & 0xff) as u8,
        ((seed >> 32) & 0xff) as u8,
        ((seed >> 40) & 0xff) as u8,
    )
}

/// Applies Chhadm (MAC & Fingerprint Spoofing) to a network interface.
pub fn apply_chhadm(interface_name: &str, mode: &str) -> Result<String, String> {
    let target_mac = if mode.eq_ignore_ascii_case("random") {
        generate_random_mac()
    } else {
        mode.to_string()
    };

    println!("[Chhadm] 🎭 Spoofing network fingerprint for '{}' -> MAC: {}", interface_name, target_mac);

    let output = Command::new("ip")
        .args(&["link", "set", "dev", interface_name, "address", &target_mac])
        .output()
        .map_err(|e| format!("Failed to execute ip command for MAC spoofing: {}", e))?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Chhadm MAC spoofing failed: {}", err));
    }

    Ok(target_mac)
}

/// Applies Gupt (Tor Anonymity Routing) by setting up transparent NAT redirect rules.
pub fn apply_gupt(host_veth: &str, mode: &str) -> Result<(), String> {
    if !mode.eq_ignore_ascii_case("tor") {
        return Ok(());
    }

    println!("[Gupt] 🧅 Engaging Tor Anonymity Tunnel on interface '{}'", host_veth);
    println!("[Gupt] 🔒 Routing 100% of container TCP/DNS traffic through Tor nodes...");

    let dns_rule = Command::new("iptables")
        .args(&[
            "-t", "nat", "-A", "PREROUTING",
            "-i", host_veth,
            "-p", "udp", "--dport", "53",
            "-j", "REDIRECT", "--to-ports", "5300"
        ])
        .output()
        .map_err(|e| format!("Failed to set iptables DNS rule for Gupt: {}", e))?;

    if !dns_rule.status.success() {
        return Err(format!("Gupt DNS routing failed: {}", String::from_utf8_lossy(&dns_rule.stderr)));
    }

    let tcp_rule = Command::new("iptables")
        .args(&[
            "-t", "nat", "-A", "PREROUTING",
            "-i", host_veth,
            "-p", "tcp", "--syn",
            "-j", "REDIRECT", "--to-ports", "9040"
        ])
        .output()
        .map_err(|e| format!("Failed to set iptables TCP rule for Gupt: {}", e))?;

    if !tcp_rule.status.success() {
        let _ = Command::new("iptables")
            .args(&["-t", "nat", "-D", "PREROUTING", "-i", host_veth, "-p", "udp", "--dport", "53", "-j", "REDIRECT", "--to-ports", "5300"])
            .output();
        return Err(format!("Gupt TCP Tor routing failed: {}", String::from_utf8_lossy(&tcp_rule.stderr)));
    }

    let _ = Command::new("iptables")
        .args(&["-A", "FORWARD", "-i", host_veth, "-p", "udp", "!", "--dport", "53", "-j", "DROP"])
        .output();

    println!("[Gupt] ✅ Tor Anonymity Tunnel active. Container public IP is now hidden.");
    Ok(())
}

/// Cleans up Gupt iptables rules when the container shuts down.
pub fn cleanup_gupt(host_veth: &str) {
    let _ = Command::new("iptables")
        .args(&["-t", "nat", "-D", "PREROUTING", "-i", host_veth, "-p", "udp", "--dport", "53", "-j", "REDIRECT", "--to-ports", "5300"])
        .output();
    let _ = Command::new("iptables")
        .args(&["-t", "nat", "-D", "PREROUTING", "-i", host_veth, "-p", "tcp", "--syn", "-j", "REDIRECT", "--to-ports", "9040"])
        .output();
    let _ = Command::new("iptables")
        .args(&["-D", "FORWARD", "-i", host_veth, "-p", "udp", "!", "--dport", "53", "-j", "DROP"])
        .output();
}
