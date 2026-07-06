use std::process::Command;
use rand::Rng;
use std::io::{self, Error, ErrorKind};

/// Generates a valid IEEE 802 Locally Administered Unicast MAC address.
pub fn generate_random_mac() -> String {
    let mut rng = rand::thread_rng();
    let first_byte: u8 = match rng.gen_range(0..4) {
        0 => 0x02,
        1 => 0x06,
        2 => 0x0A,
        _ => 0x0E,
    };
    
    format!(
        "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
        first_byte,
        rng.gen::<u8>(),
        rng.gen::<u8>(),
        rng.gen::<u8>(),
        rng.gen::<u8>(),
        rng.gen::<u8>()
    )
}

/// Applies the MAC address to the specified interface inside the namespace.
pub fn apply_chhadm(interface: &str, custom_mac: Option<&str>) -> io::Result<String> {
    let mac = match custom_mac {
        Some("random") | None => generate_random_mac(),
        Some(val) => val.to_string(),
    };

    let status = Command::new("ip")
        .args(&["link", "set", "dev", interface, "address", &mac])
        .status()?;

    if !status.success() {
        return Err(Error::new(ErrorKind::Other, "Failed to set MAC address"));
    }

    Ok(mac)
}

/// Configures iptables rules inside the container namespace to redirect TCP/DNS to Tor.
pub fn apply_gupt(trans_port: u16, dns_port: u16) -> io::Result<()> {
    let rules = vec![
        vec!["-t", "nat", "-F"],
        vec![
            "-t", "nat", "-A", "OUTPUT", "-p", "udp", "--dport", "53",
            "-j", "REDIRECT", "--to-ports", &dns_port.to_string(),
        ],
        vec![
            "-t", "nat", "-A", "OUTPUT", "-p", "tcp", "--dport", "53",
            "-j", "REDIRECT", "--to-ports", &dns_port.to_string(),
        ],
        vec!["-t", "nat", "-A", "OUTPUT", "-o", "lo", "-j", "RETURN"],
        vec![
            "-t", "nat", "-A", "OUTPUT", "-p", "tcp", "--syn",
            "-j", "REDIRECT", "--to-ports", &trans_port.to_string(),
        ],
    ];

    for rule in rules {
        let status = Command::new("iptables").args(&rule).status()?;
        if !status.success() {
            return Err(Error::new(
                ErrorKind::Other,
                format!("Failed to apply iptables rule: {:?}", rule),
            ));
        }
    }

    Ok(())
}

/// Flushes the NAT table rules inside the namespace during teardown.
pub fn cleanup_gupt() -> io::Result<()> {
    let status = Command::new("iptables").args(&["-t", "nat", "-F"]).status()?;
    if !status.success() {
        return Err(Error::new(ErrorKind::Other, "Failed to flush Gupt NAT rules"));
    }
    Ok(())
}
