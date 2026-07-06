use std::fs::File;
use std::io::Read;
use std::process::Command;

/// Generates a valid IEEE 802 locally administered unicast MAC address using standard library.
fn generate_random_mac() -> String {
    let mut bytes = [0u8; 6];
    if let Ok(mut file) = File::open("/dev/urandom") {
        let _ = file.read_exact(&mut bytes);
    } else {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(123456);
        let bytes_num = nanos.to_le_bytes();
        bytes[0] = bytes_num[0];
        bytes[1] = bytes_num[1];
        bytes[2] = bytes_num[2];
        bytes[3] = bytes_num[3];
    }
    bytes[0] = (bytes[0] & 0xFC) | 0x02;
    
    format!(
        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5]
    )
}

/// Applies MAC address randomization or specific spoofing to an interface.
pub fn apply_chhadm(iface: &str, mode: &str) -> Result<String, String> {
    let mac = if mode == "random" || mode.is_empty() {
        generate_random_mac()
    } else {
        mode.to_string()
    };

    let status = Command::new("ip")
        .args(&["link", "set", "dev", iface, "address", &mac])
        .status()
        .map_err(|e| format!("Failed to execute ip command on {}: {}", iface, e))?;

    if !status.success() {
        return Err(format!("Failed to set MAC address on interface {}", iface));
    }

    Ok(mac)
}

/// Applies Tor transparent proxying rules strictly to the container interface.
pub fn apply_gupt(iface: &str, mode: &str) -> Result<(), String> {
    println!("\n🔍 [Gupt Debug] Invoked apply_gupt with iface='{}' and mode='{}'", iface, mode);

    let rules = vec![
        // Intercept DNS queries leaving the container interface and divert to Tor DNSPort (5353)
        vec!["-t", "nat", "-I", "PREROUTING", "1", "-i", iface, "-p", "udp", "--dport", "53", "-j", "REDIRECT", "--to-ports", "5353"],
        vec!["-t", "nat", "-I", "PREROUTING", "1", "-i", iface, "-p", "tcp", "--dport", "53", "-j", "REDIRECT", "--to-ports", "5353"],
        // Intercept all outbound TCP SYN packets leaving the container interface and divert to Tor TransPort (9040)
        vec!["-t", "nat", "-I", "PREROUTING", "1", "-i", iface, "-p", "tcp", "--syn", "-j", "REDIRECT", "--to-ports", "9040"],
    ];

    for rule in rules {
        println!("🔍 [Gupt Debug] Executing: iptables {:?}", rule.join(" "));
        let status = Command::new("iptables")
            .args(&rule)
            .status()
            .map_err(|e| format!("Failed to run iptables: {}", e))?;
            
        if !status.success() {
            println!("❌ [Gupt Debug] Rule failed!");
            return Err(format!("iptables Tor rule failed: {:?}", rule));
        } else {
            println!("✅ [Gupt Debug] Rule applied successfully!");
        }
    }

    Ok(())
}

/// Removes NAT redirection rules when container terminates.
pub fn cleanup_gupt(iface: &str) -> Result<(), String> {
    let _ = Command::new("iptables")
        .args(&["-t", "nat", "-D", "PREROUTING", "-i", iface, "-p", "tcp", "--syn", "-j", "REDIRECT", "--to-ports", "9040"])
        .status();
    let _ = Command::new("iptables")
        .args(&["-t", "nat", "-D", "PREROUTING", "-i", iface, "-p", "udp", "--dport", "53", "-j", "REDIRECT", "--to-ports", "5353"])
        .status();
    let _ = Command::new("iptables")
        .args(&["-t", "nat", "-D", "PREROUTING", "-i", iface, "-p", "tcp", "--dport", "53", "-j", "REDIRECT", "--to-ports", "5353"])
        .status();

    Ok(())
}
