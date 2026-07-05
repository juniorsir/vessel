use crate::config::Patra;
use crate::network::{apply_chhadm, apply_gupt, cleanup_gupt};
use std::process::{Command, Stdio};
use std::path::Path;

pub struct Engine {
    patra: Patra,
}

impl Engine {
    pub fn new(patra: Patra) -> Self {
        Self { patra }
    }

    pub fn launch(&self) -> Result<(), String> {
        println!("[Engine] 🚀 Initializing Vessel Container: '{}'", self.patra.sangjna);
        
        let rootfs = Path::new(&self.patra.mool);
        if !rootfs.exists() {
            return Err(format!("Root filesystem not found at: {}", self.patra.mool));
        }

        let container_id = &self.patra.sangjna;
        let host_veth = format!("veth-h-{}", &container_id[..std::cmp::min(6, container_id.len())]);
        let container_veth = format!("veth-c-{}", &container_id[..std::cmp::min(6, container_id.len())]);

        let _ = Command::new("ip").args(&["link", "add", &host_veth, "type", "veth", "peer", "name", &container_veth]).output();
        let _ = Command::new("ip").args(&["link", "set", &host_veth, "up"]).output();

        // 🎭 Apply Chhadm (MAC Address Spoofing)
        if let Some(ref chhadm_mode) = self.patra.chhadm {
            if let Err(e) = apply_chhadm(&host_veth, chhadm_mode) {
                let _ = Command::new("ip").args(&["link", "delete", &host_veth]).output();
                return Err(e);
            }
        }

        // 🧅 Apply Gupt (Tor Anonymity Routing)
        if let Some(ref gupt_mode) = self.patra.gupt {
            if let Err(e) = apply_gupt(&host_veth, gupt_mode) {
                let _ = Command::new("ip").args(&["link", "delete", &host_veth]).output();
                return Err(e);
            }
        }

        let parts: Vec<&str> = self.patra.karya.split_whitespace().collect();
        let program = parts.get(0).ok_or("No executable defined in Karya")?;
        let args = &parts[1..];

        println!("[Engine] ⚡ Executing payload: {} inside isolated rootfs...", program);

        let mut child = Command::new(program)
            .args(args)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| format!("Failed to execute process inside container: {}", e))?;

        let status = child.wait().map_err(|e| format!("Process wait failed: {}", e))?;

        if self.patra.gupt.is_some() {
            println!("[Gupt] 🧹 Dismantling Tor transparent firewall rules...");
            cleanup_gupt(&host_veth);
        }

        let _ = Command::new("ip").args(&["link", "delete", &host_veth]).output();

        if status.success() {
            println!("[Engine] ✨ Vessel session finished cleanly.");
            Ok(())
        } else {
            Err(format!("Container exited with error code: {}", status.code().unwrap_or(-1)))
        }
    }
}
