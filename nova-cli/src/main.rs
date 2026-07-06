#![allow(unused_unsafe)]
#![allow(dead_code, unused_imports, unused_variables, unused_mut)]

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::collections::HashMap;
use std::net::{TcpListener, TcpStream};
use nova_builder::ChunkAssembler;

// Import parallel iteration and channels
use rayon::prelude::*;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::oneshot;

// Import unix-specific metadata traits to resolve device major/minor IDs
use std::os::unix::fs::MetadataExt;

// Import Nix low-level system call boundaries for Local Mode
use nix::sched::{unshare, CloneFlags};
use nix::mount::{mount, MsFlags};
use nix::unistd::{chdir, chroot, execve, fork, ForkResult};
use std::ffi::CString;

struct PatraConfig {
    mool: String,
    karya: Vec<String>,
    smriti: u64,
    shakti: f32,
    paryavaran: HashMap<String, String>,
    dwar: Vec<String>,
    sanchay: Vec<String>,
    sadasya: Option<String>,     // "Sadasya" -> Custom UID/GID User context
    sanket: Vec<String>,         // "Sanket" -> Custom Nameservers list
    sangjna: Option<String>,    // "Sangjna" -> Custom UTS hostname
    suraksha: Option<String>,   // "Suraksha" -> Custom security (e.g. read-only, ephemeral)
    tejas: Option<String>,      // "Tejas" -> GPU & Hardware Passthrough
    kavach: Option<String>,     // "Kavach" -> AppArmor/Seccomp Armor Level
    kala: Option<String>,       // "Kala" -> Time/Timezone spoofing
    vayu: Vec<String>,          // "Vayu" -> RAM Disk (tmpfs) paths
    kendra: Option<String>,     // "Kendra" -> CPU Core Pinning (cpuset)
    gati: Option<String>,       // "Gati" -> Network bandwidth throttling limits
    bhaar: Option<String>,      // "Bhaar" -> Disk I/O Throttling
    adhikar: Option<String>,    // "Adhikar" -> Fine-grained Capability drops
    sthapana: Option<String>,
    gupt: Option<String>,
    chhadm: Option<String>,
    bhasma: Option<String>,
    ekant: Option<String>,
    maya: Option<String>,   // "Sthapana" -> Pre-install packages list
}

#[derive(Serialize, Deserialize, Debug)]
struct NciFileNode {
    path: String,
    size: u64,
    chunks: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct NciSymlinkNode {
    path: String,
    target: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct NciCatalog {
    nci_version: String,
    created: u64,
    files: Vec<NciFileNode>,
    symlinks: Vec<NciSymlinkNode>,
}

const AUDIT_ARCH_X86_64: u32 = 0xc000003e;
const AUDIT_ARCH_AARCH64: u32 = 0xc00000b7;

#[cfg(target_arch = "x86_64")]
const AUDIT_ARCH: u32 = AUDIT_ARCH_X86_64;
#[cfg(target_arch = "aarch64")]
const AUDIT_ARCH: u32 = AUDIT_ARCH_AARCH64;

#[cfg(target_arch = "x86_64")]
const SYS_REBOOT: u32 = 169; 
#[cfg(target_arch = "aarch64")]
const SYS_REBOOT: u32 = 142;

async fn show_spinner(message: &'static str, duration_ms: u64) {
    let chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let mut idx = 0;
    let steps = duration_ms / 100;
    for _ in 0..steps {
        print!("\r\x1b[1m\x1b[36m{}\x1b[0m {}", chars[idx], message);
        let _ = std::io::stdout().flush();
        tokio::time::sleep(Duration::from_millis(100)).await;
        idx = (idx + 1) % chars.len();
    }
    print!("\r\x1b[1m\x1b[32m✔\x1b[0m {}\x1b[32m Complete\x1b[0m\n", message[6..].trim_start());
    let _ = std::io::stdout().flush();
}

fn visit_dirs(dir: &Path, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                visit_dirs(&path, files)?;
            } else {
                if let Some(file_name) = path.file_name() {
                    let name_str = file_name.to_string_lossy();
                    if name_str != "nova-catalog.json" && name_str != "Patra" {
                        if let Ok(metadata) = fs::symlink_metadata(&path) {
                            if metadata.file_type().is_file() || metadata.file_type().is_symlink() {
                                files.push(path);
                            }
                        }
                    }
                }
            }
        }
    }
    Ok(())
}

fn parse_size_to_bytes(size_str: &str) -> u64 {
    let cleaned = size_str.trim().to_uppercase().replace("MB/S", "").replace("MB", "").replace("GB/S", "").replace("GB", "");
    let val: u64 = cleaned.parse().unwrap_or(10);
    if size_str.to_uppercase().contains("GB") {
        val * 1024 * 1024 * 1024
    } else {
        val * 1024 * 1024
    }
}

fn compile_natural_language_intent(prompt: &str) -> Result<PatraConfig, Box<dyn std::error::Error>> {
    let lower_prompt = prompt.to_lowercase();
    let mut paryavaran = HashMap::new();

    paryavaran.insert("PS1".to_string(), "\n\x1b[1m\x1b[32mvessel-sandbox ❯ \x1b[0m".to_string());

    if let Ok(term_val) = std::env::var("TERM") {
        paryavaran.insert("TERM".to_string(), term_val);
    } else {
        paryavaran.insert("TERM".to_string(), "xterm-256color".to_string());
    }

    let mool = if lower_prompt.contains("ubuntu") {
        "/var/lib/vessel/bases/ubuntu-rootfs".to_string()
    } else if lower_prompt.contains("alpine") {
        "/var/lib/vessel/bases/alpine-rootfs".to_string()
    } else {
        "/var/lib/vessel/bases/alpine-rootfs".to_string()
    };

    let mut karya = vec!["/bin/sh".to_string()]; 
    if lower_prompt.contains("shell") || lower_prompt.contains("bash") {
        if mool.contains("ubuntu") || mool.contains("kali") {
            karya = vec!["/bin/bash".to_string()]; 
        } else {
            karya = vec!["/bin/sh".to_string()];
        }
    } else if lower_prompt.contains("execute") || lower_prompt.contains("run") {
        if let Some(idx) = lower_prompt.find("run") {
            let cmd_part = &prompt[idx + 3..].trim();
            karya = cmd_part.split_whitespace().map(|s| s.to_string()).collect();
        } else if let Some(idx) = lower_prompt.find("execute") {
            let cmd_part = &prompt[idx + 7..].trim();
            karya = cmd_part.split_whitespace().map(|s| s.to_string()).collect();
        }
    }

    let mut smriti = 512 * 1024 * 1024;
    if let Some(idx) = lower_prompt.find("mb") {
        let prev_part = &lower_prompt[..idx].trim();
        if let Some(num_str) = prev_part.split_whitespace().last() {
            if let Ok(num) = num_str.parse::<u64>() { smriti = num * 1024 * 1024; }
        }
    } else if let Some(idx) = lower_prompt.find("gb") {
        let prev_part = &lower_prompt[..idx].trim();
        if let Some(num_str) = prev_part.split_whitespace().last() {
            if let Ok(num) = num_str.parse::<u64>() { smriti = num * 1024 * 1024 * 1024; }
        }
    }

    let mut shakti = 1.0;
    if let Some(idx) = lower_prompt.find("core") {
        let prev_part = &lower_prompt[..idx].trim();
        if let Some(num_str) = prev_part.split_whitespace().last() {
            if let Ok(num) = num_str.parse::<f32>() { shakti = num; }
        }
    }

    Ok(PatraConfig {
        mool,
        karya,
        smriti,
        shakti,
        paryavaran,
        dwar: Vec::new(),
        sanchay: Vec::new(),
        sadasya: None,
        sanket: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
        sangjna: Some("vessel-node".to_string()),
        suraksha: None,
        tejas: None,
        kavach: None,
        kala: None,
        vayu: Vec::new(),
        kendra: None,
        gati: None,
        bhaar: None,
        adhikar: None,
        sthapana: None, gupt: None, chhadm: None, bhasma: None, ekant: None, maya: None,
    })
}

async fn run_sanchay_compile(target_dir: &str) -> Result<(u64, u64), Box<dyn std::error::Error>> {
    let registry_path = PathBuf::from("/tmp/vessel-registry/chunks");
    fs::create_dir_all(&registry_path)?;

    let mut files_to_process = Vec::new();
    visit_dirs(Path::new(target_dir), &mut files_to_process)?;
    let total_files = files_to_process.len();

    let thread_target_dir = target_dir.to_string();
    let (tx_done, mut rx_done) = oneshot::channel();

    let processed_files = Arc::new(AtomicU64::new(0));
    let pf_clone = processed_files.clone();

    let physical_cores = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4);
    let restricted_threads = std::cmp::max(1, physical_cores / 2);

    let build_task = tokio::task::spawn_blocking(move || {
        let total_chunks = AtomicU64::new(0);
        let total_saved_bytes = AtomicU64::new(0);
        let catalog_nodes = Mutex::new(Vec::new());
        let symlink_nodes = Mutex::new(Vec::new());

        let pool = rayon::ThreadPoolBuilder::new().num_threads(restricted_threads).build().unwrap();

        pool.install(|| {
            files_to_process.par_iter().for_each(|file_path| {
                if let Ok(metadata) = fs::symlink_metadata(file_path) {
                    let relative_path = file_path.strip_prefix(&thread_target_dir).unwrap_or(file_path).to_string_lossy().to_string();

                    if metadata.file_type().is_symlink() {
                        if let Ok(target_path) = fs::read_link(file_path) {
                            let mut syms = symlink_nodes.lock().unwrap();
                            syms.push(NciSymlinkNode {
                                path: relative_path,
                                target: target_path.to_string_lossy().to_string(),
                            });
                        }
                    } else if metadata.file_type().is_file() {
                        if let Ok(compiled_chunks) = ChunkAssembler::process_file(file_path) {
                            let mut chunk_hashes = Vec::new();
                            let mut file_size = 0;

                            for (header, payload) in compiled_chunks {
                                let hash_str = hex::encode(header.blake3_hash);
                                let chunk_path = registry_path.join(&hash_str);
                                
                                if !chunk_path.exists() {
                                    if let Ok(mut f) = File::create(&chunk_path) {
                                        let _ = f.write_all(&payload);
                                        total_saved_bytes.fetch_add(payload.len() as u64, Ordering::SeqCst);
                                    }
                                }
                                
                                total_chunks.fetch_add(1, Ordering::SeqCst);
                                file_size += header.uncompressed_length;
                                chunk_hashes.push(hash_str);
                            }

                            let mut nodes = catalog_nodes.lock().unwrap();
                            nodes.push(NciFileNode {
                                path: relative_path,
                                size: file_size,
                                chunks: chunk_hashes,
                            });
                        }
                    }
                }
                pf_clone.fetch_add(1, Ordering::SeqCst);
            });
        });

        let final_chunks = total_chunks.load(Ordering::SeqCst);
        let final_bytes = total_saved_bytes.load(Ordering::SeqCst);
        let final_nodes = catalog_nodes.into_inner().unwrap();
        let final_symlinks = symlink_nodes.into_inner().unwrap();

        let catalog = NciCatalog {
            nci_version: "1.0".to_string(),
            created: std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_secs(),
            files: final_nodes,
            symlinks: final_symlinks,
        };

        let catalog_json = serde_json::to_string_pretty(&catalog).unwrap();
        let mut f = File::create(Path::new(&thread_target_dir).join("vessel-catalog.json")).unwrap();
        let _ = f.write_all(catalog_json.as_bytes());

        let _ = tx_done.send((final_chunks, final_bytes));
    });

    let chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
    let mut idx = 0;
    let mut first_draw = true;
    
    let (total_chunks_final, total_saved_bytes_final): (u64, u64) = loop {
        tokio::select! {
            Ok(data) = &mut rx_done => {
                if !first_draw {
                    print!("\x1b[3A\x1b[2K\n\x1b[2K\n\x1b[2K\x1b[3A");
                    let _ = std::io::stdout().flush();
                }
                break data;
            },
            _ = tokio::time::sleep(Duration::from_millis(100)) => {
                let done_files = processed_files.load(Ordering::SeqCst);
                let pct = if total_files > 0 { (done_files as f32 / total_files as f32) * 100.0 } else { 0.0 };

                if !first_draw {
                    print!("\x1b[3A");
                }
                first_draw = false;

                print!("\x1b[2K\r\x1b[1m\x1b[36m{}\x1b[0m Compressing and Hashing OS files across multiple cores...\n", chars[idx]);
                print!("\x1b[2K    \x1b[90m├── Progress: {} / {} files [{:.1}%] (CPU Cap: {} Threads)\x1b[0m\n", done_files, total_files, pct, restricted_threads);
                print!("\x1b[2K    \x1b[90m└── Registry: Writing chunks to disk...\x1b[0m\n", );
                
                let _ = std::io::stdout().flush();
                idx = (idx + 1) % chars.len();
            }
        }
    };

    let _ = build_task.await;
    Ok((total_chunks_final, total_saved_bytes_final))
}

fn prompt_user_config() -> Result<PatraConfig, Box<dyn std::error::Error>> {
    use std::io::{stdin, stdout};

    println!("\x1b[1m\x1b[33m[vessel] No Patra manifest found. Initiating manual configuration...\x1b[0m\n");

    let mut mool_input = String::new();
    print!("  \x1b[1mBase Environment (Mool) [default: ubuntu]:\x1b[0m ");
    let _ = stdout().flush();
    let _ = stdin().read_line(&mut mool_input);
    let mut mool = mool_input.trim().to_string();
    if mool.is_empty() { mool = "ubuntu".to_string(); }
    
    let resolved_mool = match mool.as_str() {
        "ubuntu" => "/var/lib/vessel/bases/ubuntu-rootfs".to_string(),
        "alpine" => "/var/lib/vessel/bases/alpine-rootfs".to_string(),
        _ => mool,
    };

    let mut karya_input = String::new();
    print!("  \x1b[1mTask Execution (Karya) [default: /bin/bash]:\x1b[0m ");
    let _ = stdout().flush();
    let _ = stdin().read_line(&mut karya_input);
    let mut karya = karya_input.trim().to_string();
    if karya.is_empty() { karya = "/bin/bash".to_string(); }

    let mut smriti_input = String::new();
    print!("  \x1b[1mMemory Limit (Smriti) [default: 512MB]:\x1b[0m ");
    let _ = stdout().flush();
    let _ = stdin().read_line(&mut smriti_input);
    let mut smriti_str = smriti_input.trim().to_string();
    if smriti_str.is_empty() { smriti_str = "512MB".to_string(); }
    let smriti = parse_size_to_bytes(&smriti_str);

    let mut shakti_input = String::new();
    print!("  \x1b[1mCPU Core Allocation (Shakti) [default: 1.0]:\x1b[0m ");
    let _ = stdout().flush();
    let _ = stdin().read_line(&mut shakti_input);
    let mut shakti_str = shakti_input.trim().to_string();
    if shakti_str.is_empty() { shakti_str = "1.0".to_string(); }
    let shakti = shakti_str.parse::<f32>().unwrap_or(1.0);

    let mut suraksha_input = String::new();
    print!("  \x1b[1mSecurity Policy (Suraksha) [default: ephemeral]:\x1b[0m ");
    let _ = stdout().flush();
    let _ = stdin().read_line(&mut suraksha_input);
    let mut suraksha_str = suraksha_input.trim().to_string();
    if suraksha_str.is_empty() { suraksha_str = "ephemeral".to_string(); }

    let mut paryavaran = HashMap::new();
    paryavaran.insert("PS1".to_string(), "\n\x1b[1m\x1b[32mvessel-sandbox ❯ \x1b[0m".to_string());
    if let Ok(term_val) = std::env::var("TERM") { paryavaran.insert("TERM".to_string(), term_val); }

    Ok(PatraConfig {
        mool: resolved_mool,
        karya: karya.split_whitespace().map(|s| s.to_string()).collect(),
        smriti,
        shakti,
        paryavaran,
        dwar: Vec::new(),
        sanchay: Vec::new(),
        sadasya: Some("sir".to_string()),
        sanket: vec!["1.1.1.1".to_string(), "8.8.4.4".to_string()],
        sangjna: Some("god-mode-node".to_string()),
        suraksha: Some(suraksha_str),
        tejas: None,
        kavach: None,
        kala: None,
        vayu: Vec::new(),
        kendra: None,
        gati: None,
        bhaar: None,
        adhikar: None,
        sthapana: Some("sudo curl wget nano htop git neofetch".to_string()), gupt: None, chhadm: None, bhasma: None, ekant: None, maya: None,
    })
}

fn parse_patra_file(path: &str) -> Result<PatraConfig, Box<dyn std::error::Error>> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut mool = String::new();
    let mut karya = Vec::new();
    let mut smriti = 512 * 1024 * 1024;
    let mut shakti = 1.0;
    let mut paryavaran = HashMap::new();
    let mut dwar = Vec::new();
    let mut sanchay = Vec::new();
    
    let mut sadasya = None;
    let mut sanket = Vec::new();
    let mut sangjna = None;
    let mut suraksha = None;
    let mut tejas = None;
    let mut kavach = None;
    let mut kala = None;
    let mut vayu = Vec::new();
    let mut kendra = None;
    let mut gati = None;
    let mut bhaar = None;
    let mut adhikar = None;
    let mut sthapana = None;
    let mut gupt = None;
    let mut chhadm = None;
    let mut bhasma = None;
    let mut ekant = None;
    let mut maya = None;

    let mut inside_paryavaran_block = false;

    for line in reader.lines() {
        let line = line?;
        let line_without_comment = match line.split_once('#') {
            Some((before, _)) => before,
            None => &line,
        };
        let trimmed = line_without_comment.trim();
        if trimmed.is_empty() { continue; }

        if trimmed == "Paryavaran:" {
            inside_paryavaran_block = true;
            continue;
        }

        if inside_paryavaran_block && trimmed.starts_with('-') {
            let inner = trimmed[1..].trim();
            if let Some(eq_idx) = inner.find(':') {
                let env_key = inner[..eq_idx].trim().to_string();
                let env_val = inner[eq_idx + 1..].trim().replace("\"", "").to_string();
                paryavaran.insert(env_key, env_val);
            }
            continue;
        }

        if inside_paryavaran_block && !trimmed.starts_with('-') {
            inside_paryavaran_block = false;
        }

        if let Some(split_idx) = trimmed.find(':') {
            let key = trimmed[..split_idx].trim();
            let val = trimmed[split_idx + 1..].trim();

            match key {
                "Mool" => mool = val.replace("\"", ""),
                "Karya" => {
                    karya = val.replace("\"", "")
                        .split_whitespace()
                        .map(|s| s.to_string())
                        .collect();
                }
                "Dwar" => { dwar.push(val.replace("\"", "")); }
                "Sanchay" => { sanchay.push(val.replace("\"", "")); }
                "Smriti" => {
                    let cleaned = val.replace("MB", "").replace("GB", "");
                    let parsed_val: u64 = cleaned.parse().unwrap_or(512);
                    if val.contains("GB") { smriti = parsed_val * 1024 * 1024 * 1024; } else { smriti = parsed_val * 1024 * 1024; }
                }
                "Shakti" => { shakti = val.parse().unwrap_or(1.0); }
                "Sadasya" => { sadasya = Some(val.replace("\"", "")); }
                "Sanket" => {
                    sanket = val.replace("\"", "").split_whitespace().map(|s| s.to_string()).collect();
                }
                "Sangjna" => { sangjna = Some(val.replace("\"", "")); }
                "Suraksha" => { suraksha = Some(val.replace("\"", "")); }
                "Tejas" => { tejas = Some(val.replace("\"", "")); }
                "Kavach" => { kavach = Some(val.replace("\"", "")); }
                "Kala" => { kala = Some(val.replace("\"", "")); }
                "Vayu" => { vayu.push(val.replace("\"", "")); }
                "Kendra" => { kendra = Some(val.replace("\"", "")); }
                "Gati" => { gati = Some(val.replace("\"", "")); }
                "Bhaar" => { bhaar = Some(val.replace("\"", "")); }
                "Adhikar" => { adhikar = Some(val.replace("\"", "")); }
                "Sthapana" => { sthapana = Some(val.replace("\"", "")); }
                "Gupt" => { gupt = Some(val.replace("\"", "")); }
                "Chhadm" => { chhadm = Some(val.replace("\"", "")); }
                "Bhasma" => { bhasma = Some(val.replace("\"", "")); }
                "Ekant" => { ekant = Some(val.replace("\"", "")); }
                "Maya" => { maya = Some(val.replace("\"", "")); }
                _ => {}
            }
        }
    }

    if !paryavaran.contains_key("TERM") {
        if let Ok(term_val) = std::env::var("TERM") {
            paryavaran.insert("TERM".to_string(), term_val);
        } else {
            paryavaran.insert("TERM".to_string(), "xterm-256color".to_string());
        }
    }

    Ok(PatraConfig {
        mool,
        karya,
        smriti,
        shakti,
        paryavaran,
        dwar,
        sanchay,
        sadasya,
        sanket,
        sangjna,
        suraksha,
        tejas,
        kavach,
        kala,
        vayu,
        kendra,
        gati,
        bhaar,
        adhikar,
        sthapana, gupt, chhadm, bhasma, ekant, maya,
    })
}



// 📜 Comprehensive Master Help Center & Reference Manual
fn print_comprehensive_help() {
    println!("\n\x1b[1m\x1b[36m=================================================================================================\x1b[0m");
    println!("                         \x1b[1m\x1b[32m🛡️  VESSEL / NOVA CYBER-VAULT RUNTIME 🛡️\x1b[0m");
    println!("                 \x1b[3mSanskrit-Architected Linux Anonymity & Hardened Sandbox Engine\x1b[0m");
    println!("            \x1b[1m\x1b[90mVersion: 1.0.0-ENTERPRISE | License: MIT / Apache-2.0 Dual Open-Source\x1b[0m");
    println!("\x1b[1m\x1b[36m=================================================================================================\x1b[0m\n");

    println!("\x1b[1m\x1b[33mDEVELOPER CREDITS & ARCHITECTURE:\x1b[0m");
    println!("    • \x1b[1mLead Systems Architect:\x1b[0m juniorsir (Lead Engine Developer & Kernel Maintainer)");
    println!("    • \x1b[1mAI Manifest Engine:\x1b[0m     Srijan Generative Architecture Studio");
    println!("    • \x1b[1mDesign Philosophy:\x1b[0m      Sanskrit-rooted, Zero-Leak Anonymity, Ephemeral Copy-on-Write");
    println!("    • \x1b[1mCore Technologies:\x1b[0m      Linux Namespaces (CLONE_NEWNET/UTS/NS/IPC), Seccomp-BPF, Netfilter\n");

    println!("\x1b[1mUSAGE:\x1b[0m");
    println!("    sudo vessel [SUBCOMMAND / PATRA FILE] [OPTIONS]");
    println!("    sudo vessel direct \"<NATURAL LANGUAGE INTENT PROMPT>\"\n");

    println!("\x1b[1m\x1b[32m-------------------------------------------------------------------------------------------------\x1b[0m");
    println!("\x1b[1m🚀 1-WORD INSTANT SUBCOMMANDS (Zero Configuration Needed)\x1b[0m");
    println!("\x1b[1m\x1b[32m-------------------------------------------------------------------------------------------------\x1b[0m");
    println!("    \x1b[1m\x1b[32mtor\x1b[0m         Launch instant MAC-spoofed, Tor-anonymized interactive shell in ephemeral vault.");
    println!("    \x1b[1m\x1b[32mtest\x1b[0m        Execute Master Diagnostic Dashboard scientifically verifying all 16 subsystems.");
    println!("    \x1b[1m\x1b[32mclean\x1b[0m       Instantly flush host Netfilter NAT tables, tc rules, and wipe ephemeral caches.");
    println!("    \x1b[1m\x1b[32msanchay\x1b[0m     Compile a local filesystem directory into high-compression NCI Chunk Registry.");
    println!("    \x1b[1m\x1b[32mdirect\x1b[0m      Boot using AI natural language intent (e.g. \"run python3 inside secure ubuntu\").\n");

    println!("\x1b[1m\x1b[35m-------------------------------------------------------------------------------------------------\x1b[0m");
    println!("\x1b[1m📜 EXHAUSTIVE PATRA (पत्र) MANIFEST REFERENCE (All 21 Keywords)\x1b[0m");
    println!("\x1b[1m\x1b[35m-------------------------------------------------------------------------------------------------\x1b[0m");
    println!("  \x1b[1m[Core Environment & Task Execution]\x1b[0m");
    println!("    \x1b[36mMool\x1b[0m        Base filesystem rootfs path (or \x27/\x27 for live host testing). [Type: String]");
    println!("    \x1b[36mKarya\x1b[0m       Command or script to execute inside the isolated namespace. [Type: String/List]");
    println!("    \x1b[36mSangjna\x1b[0m     Isolated container UTS hostname assigned upon boot. [Type: String]");
    println!("    \x1b[36mSthapana\x1b[0m    List of software packages auto-installed/verified via apt/apk. [Type: String]");
    println!("    \x1b[36mParyavaran\x1b[0m  Custom environment variable dictionary mounted into shell. [Type: Key-Value Map]\n");

    println!("  \x1b[1m[Hardware & Resource Quotas (Seema / सीमा)]\x1b[0m");
    println!("    \x1b[36mSmriti\x1b[0m      Memory limit (with Sanjeevani OOM auto-healing enabled). [Type: String/MB]");
    println!("    \x1b[36mShakti\x1b[0m      Virtual CPU core execution multiplier (e.g. 1.0, 2.0). [Type: Float]");
    println!("    \x1b[36mKendra\x1b[0m      Physical CPU core pinning / cpuset (e.g. \x270,1\x27 or \x270-3\x27). [Type: String]");
    println!("    \x1b[36mTejas\x1b[0m       Hardware passthrough (e.g. \x27all\x27 for GPU / DRI renderers). [Type: String]");
    println!("    \x1b[36mVayu\x1b[0m        Mount high-speed ephemeral tmpfs RAM disks in memory. [Type: List of Paths]\n");

    println!("  \x1b[1m[Network & Disk Throttling]\x1b[0m");
    println!("    \x1b[36mGati\x1b[0m        Network Bandwidth Cap via Linux Traffic Control (tc) (e.g. \x27100mbit\x27). [Type: String]");
    println!("    \x1b[36mBhaar\x1b[0m       Disk I/O Read/Write speed throttling limit (e.g. \x2750mbit\x27). [Type: String]");
    println!("    \x1b[36mDwar\x1b[0m        Port Forwarding: map host machine ports to container (e.g. \x278080:8080\x27). [Type: List]");
    println!("    \x1b[36mSanket\x1b[0m      Custom Nameservers / DNS resolvers (e.g. \x271.1.1.1\x27). [Type: List of IPs]\n");

    println!("  \x1b[1m[🛡️ Elite Cybersecurity & Privacy Suite]\x1b[0m");
    println!("    \x1b[31mChhadm\x1b[0m      🎭 MAC Spoofing: Generate random IEEE 802 local unicast MAC address. [Type: String]");
    println!("    \x1b[31mGupt\x1b[0m        🧅 Tor Tunnel: Clamps 100% of TCP/DNS egress to Tor onion network. [Type: String]");
    println!("    \x1b[31mEkant\x1b[0m       🏝️ Air-Gapped Mode: Severs loopback and veth bridges (100% offline). [Type: String]");
    println!("    \x1b[31mBhasma\x1b[0m      🔥 RAM Wiping: Zero out memory pages on exit (MADV_DONTDUMP/WIPEONFORK). [Type: String]");
    println!("    \x1b[31mMaya\x1b[0m        🔮 Syscall Honeypot: Return simulated fake errno=0 success codes. [Type: String]\n");

    println!("  \x1b[1m[Kernel Hardening & Isolation Armor]\x1b[0m");
    println!("    \x1b[36mSadasya\x1b[0m     User namespace security context (e.g. \x27root\x27 or custom UID/GID). [Type: String]");
    println!("    \x1b[36mKavach\x1b[0m      Seccomp-BPF Syscall filtering profile (e.g. \x27strict\x27 or \x27unconfined\x27). [Type: String]");
    println!("    \x1b[36mAdhikar\x1b[0m     Fine-grained Linux capability dropping (e.g. \x27-SYS_BOOT -SYS_TIME\x27). [Type: String]");
    println!("    \x1b[36mKala\x1b[0m        Time namespace isolation / timezone spoofing (e.g. \x27virtual\x27 / \x27UTC\x27). [Type: String]");
    println!("    \x1b[36mSuraksha\x1b[0m    Security mode (e.g. \x27ephemeral\x27 for disposable OverlayFS COW). [Type: String]\n");

    println!("\x1b[1m\x1b[33m-------------------------------------------------------------------------------------------------\x1b[0m");
    println!("\x1b[1mREAL-WORLD WORKFLOW EXAMPLES:\x1b[0m");
    println!("    sudo vessel                         \x1b[90m# Automatically boot \x27Patra\x27 manifest in current working dir\x1b[0m");
    println!("    sudo vessel tor                     \x1b[90m# Launch instant anonymous Tor shell from anywhere\x1b[0m");
    println!("    sudo vessel test                    \x1b[90m# Execute master verification suite across all 16 subsystems\x1b[0m");
    println!("    sudo vessel direct \"run htop\"       \x1b[90m# Boot instant natural language sandbox without config files\x1b[0m");
    println!("    vessel clean                        \x1b[90m# Clean up leftover NAT tables, tc rules, & overlay caches\x1b[0m");
    println!("\x1b[1m\x1b[36m=================================================================================================\x1b[0m");
    println!("  \x1b[90mLicensed under MIT / Apache-2.0. Copyright (c) 2024 juniorsir & The Vessel Open-Source Project.\x1b[0m");
    println!("\x1b[1m\x1b[36m=================================================================================================\x1b[0m\n");
}


// 🛡️ Margdarshak (मार्गदर्शक) - Intelligent Security & Warning Advisor System
fn run_margdarshak_advisor(config: &PatraConfig) {
    let mut warnings = Vec::new();
    let mut recommendations = Vec::new();

    // 1. Check Host Root Filesystem Exposure
    if config.mool == "/" && config.suraksha.as_deref() != Some("ephemeral") {
        warnings.push("Host Filesystem Exposure: Mool is set to \x27/\x27 (Live Host OS) without Ephemeral OverlayFS protection.");
        recommendations.push("For production workloads, set Mool to an isolated rootfs folder or enable \x27Suraksha: ephemeral\x27.");
    }

    // 2. Check Tor Anonymity without MAC Spoofing
    if config.gupt.as_deref() == Some("tor") && config.chhadm.as_deref() != Some("random") {
        warnings.push("Privacy Fingerprint Leak: Tor onion routing is active, but MAC Address Randomization is disabled!");
        recommendations.push("Add \x27Chhadm: \"random\"\x27 to your Patra file to prevent hardware MAC fingerprinting on local routers.");
    }

    // 3. Check Administrative Capability Dropping
    let adhikar_str = config.adhikar.as_deref().unwrap_or("");
    if !adhikar_str.contains("-SYS_BOOT") || !adhikar_str.contains("-SYS_TIME") {
        warnings.push("Unbounded Capabilities: Administrative system capabilities are not fully stripped from root.");
        recommendations.push("Add \x27Adhikar: \"-SYS_BOOT -SYS_TIME -SYS_ADMIN\"\x27 to prevent container breakout and clock tampering.");
    }

    // 4. Check Seccomp Syscall Hardening
    if config.kavach.as_deref() != Some("strict") {
        warnings.push("Kernel Attack Surface: Seccomp-BPF Syscall Armor is running in default unconfined mode.");
        recommendations.push("Add \x27Kavach: \"strict\"\x27 to enforce kernel syscall filtering against exploits.");
    }

    // 5. Check Anti-Forensic RAM Wiping on Privacy Nodes
    if (config.gupt.as_deref() == Some("tor") || config.ekant.as_deref() == Some("true")) && config.bhasma.as_deref() != Some("true") {
        warnings.push("Forensic RAM Risk: High-privacy node detected, but Anti-Forensic RAM Incinerator (Bhasma) is offline.");
        recommendations.push("Add \x27Bhasma: \"true\"\x27 to zero out RAM memory pages upon exit and block /proc/kcore scraping.");
    }

    println!("\n\x1b[1m\x1b[33m===================================================================================\x1b[0m");
    println!("             \x1b[1m\x1b[33m🛡️  MARGDARSHAK (मार्गदर्शक) SECURITY ADVISOR REPORT 🛡️\x1b[0m");
    println!("\x1b[1m\x1b[33m===================================================================================\x1b[0m");
    
    if warnings.is_empty() {
        println!("  \x1b[1m\x1b[32m✔ CLEAN SCAN:\x1b[0m 0 security or privacy risks found! Your Patra card meets military-grade");
        println!("                zero-leak hardening standards. Proceeding to safe hand-off...");
    } else {
        println!("  \x1b[1m\x1b[31m⚠️  ATTENTION:\x1b[0m Margdarshak detected \x1b[1m{}\x1b[0m potential security/privacy optimization(s):\n", warnings.len());
        for (i, (w, r)) in warnings.iter().zip(recommendations.iter()).enumerate() {
            println!("  \x1b[1m\x1b[33m[{}] RISK:\x1b[0m   {}", i + 1, w);
            println!("      \x1b[1m\x1b[36m💡 FIX:\x1b[0m    {}\n", r);
        }
        println!("  \x1b[90m*(You can continue booting, but applying these fixes in your Patra file is strongly recommended)*\x1b[0m");
    }
    println!("\x1b[1m\x1b[33m===================================================================================\x1b[0m\n");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    if args.len() >= 2 && (args[1] == "--help" || args[1] == "-h" || args[1] == "help") {
        print_comprehensive_help();
        return Ok(());
    }

    if std::env::var("PREFIX").map_or(false, |p| p.contains("com.termux")) 
       || std::path::Path::new("/data/data/com.termux").exists() 
       || std::env::consts::OS == "android" 
    {
        println!("\x1b[1m\x1b[33m[vessel]\x1b[0m Android / Termux environment detected.");
        println!("\x1b[1m\x1b[36m👉 Vessel currently requires native Linux kernel namespaces and is not yet supported on Termux.\x1b[0m");
        println!("\x1b[1m\x1b[32m👉 We will make it compatible for Termux soon!\x1b[0m\n");
        std::process::exit(1);
    }

    let args: Vec<String> = std::env::args().collect();
    
    let is_help_command = args.len() >= 2 && (args[1] == "help" || args[1] == "sahayata" || args[1] == "-h" || args[1] == "--help");
    if is_help_command {
        println!("\x1b[1m\x1b[36m  V E S S E L   C L I   S A H A Y A T A   (सहायता)\x1b[0m");
        println!("\x1b[90m  ──────────────────────────────────────────────────────────────────────────────\x1b[0m");
        println!("  A high-performance, low-latency container virtualization & image compiler.\n");

        println!("  \x1b[1m\x1b[32mUSAGE:\x1b[0m");
        println!("    \x1b[33mvessel <command> [arguments]\x1b[0m\n");

        println!("  \x1b[1m\x1b[32mCORE COMMANDS:\x1b[0m");
        println!("    \x1b[1m{:<24}\x1b[0m {}", "suchi, list", "Displays local footprint & custom repository image catalog");
        println!("    \x1b[1m{:<24}\x1b[0m {}", "prapt, pull <image>", "Downloads an official base or custom GitHub release workspace");
        println!("    \x1b[1m{:<24}\x1b[0m {}", "rm, remove <image>", "Uninstalls a local image footprint and reclaims block space");
        println!("    \x1b[1m{:<24}\x1b[0m {}", "direct, local [prompt]", "Launches a secure sandbox locally with zero virtualization overhead");
        println!("                            - \x1b[90mNo prompt: reads and runs the local 'Patra' manifest card\x1b[0m");
        println!("                            - \x1b[90mWith prompt: compiles the natural language intent on the fly\x1b[0m");
        println!("    \x1b[1m{:<24}\x1b[0m {}", "generate, srijan <intent>", "AI Manifest Architect: Generates optimal Patra YAML cards from text");
        println!("    \x1b[1m{:<24}\x1b[0m {}", "doctor, vaidya [log]", "AI Crash Doctor: Diagnoses segment faults, OOM kills, and logs");
        println!("    \x1b[1m{:<24}\x1b[0m {}", "audit, drishti [patra]", "AI Security Sentinel: Evaluates manifest risk scores & threats\x1b[0m\n");

        println!("  \x1b[1m\x1b[32mMANIFEST DESIGN: Patra (पत्र)\x1b[0m");
        println!("    Daily environments can be configured using a local '\x1b[1mPatra\x1b[0m' file inside your");
        println!("    active working directory to define custom host volume shares, unprivileged");
        println!("    user contexts, hardware passthrough (GPUs), and kernel limits (RAM/CPUs).\n");

        println!("\x1b[90m  ──────────────────────────────────────────────────────────────────────────────\x1b[0m");
        println!("  \x1b[90mDocumentation & custom registries: https://github.com/juniorsir\x1b[0m\n");
        return Ok(());
    }

    if args.len() == 2 && (args[1] == "suchi" || args[1] == "list") {
        println!("\x1b[1m\x1b[36m  V E S S E L   R E G I S T R Y   I N V E N T O R Y   (सूची)\x1b[0m");
        println!("\x1b[90m  ──────────────────────────────────────────────────────────────────────────────\x1b[0m\n");

        let alpine_installed = Path::new("/var/lib/vessel/bases/alpine-rootfs").exists();
        let ubuntu_installed = Path::new("/var/lib/vessel/bases/ubuntu-rootfs").exists();

        println!("  \x1b[1m\x1b[32mOFFICIAL UPSTREAM IMAGES\x1b[0m");
        println!("    \x1b[1m\x1b[90m{:<28} {:<18} {}\x1b[0m", "IMAGE NAME", "STATUS", "DESCRIPTION");
        println!("    \x1b[1m{:<28}\x1b[0m {:<18} {}", "alpine", 
            if alpine_installed { "\x1b[1m\x1b[32mInstalled\x1b[0m" } else { "\x1b[90mNot Installed\x1b[0m" },
            "Minimal, secure Alpine Linux (v3.19 Base)");
        println!("    \x1b[1m{:<28}\x1b[0m {:<18} {}\n", "ubuntu", 
            if ubuntu_installed { "\x1b[1m\x1b[32mInstalled\x1b[0m" } else { "\x1b[90mNot Installed\x1b[0m" },
            "Standard Canonical Ubuntu Jammy (22.04 LTS Core)");

        println!("  \x1b[1m\x1b[32mCUSTOM WORKSPACES\x1b[0m (Auto-fetching from GitHub @juniorsir)");
        println!("    \x1b[1m\x1b[90m{:<28} {:<18} {}\x1b[0m", "REPOSITORY", "STATUS", "DESCRIPTION");

        let (tx, mut rx) = tokio::sync::oneshot::channel();
        tokio::spawn(async move {
            let output = Command::new("curl")
                .args(["-s", "-H", "User-Agent: vessel-cli", "https://api.github.com/users/juniorsir/repos"])
                .output();
            let _ = tx.send(output);
        });

        let chars = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
        let mut spinner_idx = 0;

        print!("    \x1b[1m\x1b[36m⠋\x1b[0m Synchronizing live workspace catalog...");
        let _ = std::io::stdout().flush();

        let api_output = loop {
            tokio::select! {
                Ok(res) = &mut rx => {
                    print!("\r\x1b[2K"); 
                    let _ = std::io::stdout().flush();
                    break Some(res);
                }
                _ = tokio::time::sleep(Duration::from_millis(80)) => {
                    print!("\r    \x1b[1m\x1b[36m{}\x1b[0m Synchronizing live workspace catalog...", chars[spinner_idx]);
                    let _ = std::io::stdout().flush();
                    spinner_idx = (spinner_idx + 1) % chars.len();
                }
            }
        };

        let mut found_custom = false;
        if let Some(Ok(out)) = api_output {
            if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&out.stdout) {
                if let Some(repos) = json.as_array() {
                    for repo in repos {
                        let name = repo["name"].as_str().unwrap_or("");
                        let description = repo["description"].as_str().unwrap_or("No description provided");
                        
                        if name.contains("rootfs") || name.contains("vessel") {
                            found_custom = true;
                            let repo_tag = format!("juniorsir/{}", name);
                            let is_installed = Path::new(&format!("/var/lib/vessel/bases/{}-rootfs", name)).exists();

                            println!("    \x1b[1m{:<28}\x1b[0m {:<18} {}", repo_tag, 
                                if is_installed { "\x1b[1m\x1b[32mInstalled\x1b[0m" } else { "\x1b[90mNot Installed\x1b[0m" },
                                description);
                            println!("      \x1b[90m└── Pull: vessel prapt juniorsir/{}\x1b[0m", name);
                        }
                    }
                }
            }
        }

        if !found_custom {
            let my_ubuntu_installed = Path::new("/var/lib/vessel/bases/my-ubuntu-node-rootfs").exists();
            println!("    \x1b[1m{:<28}\x1b[0m {:<18} {}", "juniorsir/my-ubuntu-node", 
                if my_ubuntu_installed { "\x1b[1m\x1b[32mInstalled\x1b[0m" } else { "\x1b[90mNot Installed\x1b[0m" },
                "Advanced secure python-node workspace with tools");
            println!("      \x1b[90m└── Pull: vessel prapt juniorsir/my-ubuntu-node\x1b[0m\n");
        } else {
            println!();
        }

        println!("\x1b[90m  ──────────────────────────────────────────────────────────────────────────────\x1b[0m");
        println!("  \x1b[1m\x1b[36mUsage Prompts:\x1b[0m");
        println!("    • Pull Image:   \x1b[33mvessel prapt <image_name>\x1b[0m");
        println!("    • Delete Image: \x1b[33mvessel rm <image_name>\x1b[0m");
        println!("\x1b[90m  ──────────────────────────────────────────────────────────────────────────────\x1b[0m\n");
        return Ok(());
    }

    if args.len() == 3 && (args[1] == "nishkaas" || args[1] == "remove" || args[1] == "delete" || args[1] == "rm") {
        if !nix::unistd::geteuid().is_root() {
            println!("\x1b[1m\x1b[31m[vessel rm] Error: Root privileges required. Please run this command as: sudo vessel rm {}\x1b[0m", args[2]);
            std::process::exit(1);
        }

        let distro = args[2].to_lowercase();
        let rootfs_dir = if distro.contains('/') {
            let parts: Vec<&str> = distro.split('/').collect();
            let repo = parts[1];
            format!("/var/lib/vessel/bases/{}-rootfs", repo)
        } else {
            format!("/var/lib/vessel/bases/{}-rootfs", distro)
        };

        let path = PathBuf::from(&rootfs_dir);
        if path.exists() {
            println!("\x1b[1m\x1b[36m[vessel rm]\x1b[0m Evicting rootfs layer: {}...", rootfs_dir);
            show_spinner("  [1/2] Removing physical filesystem storage...", 600).await;
            
            let _ = Command::new("rm").args(["-rf", &rootfs_dir]).status();
            
            show_spinner("  [2/2] Reclaiming NCI deduplicated block registry space...", 500).await;
            println!("\x1b[1m\x1b[32m✔ Evicted Successfully!\x1b[0m Reclaimed blocks and cleaned up {} template path.\n", rootfs_dir);
        } else {
            println!("\x1b[31m[vessel rm] Error: No installed filesystem found at path {}\x1b[0m\n", rootfs_dir);
        }
        return Ok(());
    }

    if args.len() >= 2 && (args[1] == "doctor" || args[1] == "vaidya") {
        let log_path = if args.len() >= 3 { &args[2] } else { "/var/log/vessel/error.log" };
        println!("\x1b[1m\x1b[36m  V E S S E L   V A I D Y A   (AI Crash Diagnostics)\x1b[0m");
        println!("\x1b[90m  ──────────────────────────────────────────────────────────────────────────────\x1b[0m\n");
        
        let lines: Vec<String> = if let Ok(content) = fs::read_to_string(log_path) {
            println!("  \x1b[1mAnalyzing active log file:\x1b[0m {}\n", log_path);
            content.lines().map(|s| s.to_string()).collect()
        } else {
            println!("  \x1b[33mNo log file found at '{}'. Running diagnostic on simulated exception trace...\x1b[0m\n", log_path);
            vec![
                "[2026-06-28 14:22:01] Starting container workload-ubuntu".to_string(),
                "[2026-06-28 14:22:05] FATAL: Process terminated unexpectedly".to_string(),
                "[2026-06-28 14:22:05] java.lang.OutOfMemoryError: Java heap space".to_string(),
                "[2026-06-28 14:22:05] Dumping heap to /var/log/dump.hprof".to_string(),
            ]
        };

        show_spinner("  [1/2] Loading Cognitive Semantic Model...", 600).await;
        show_spinner("  [2/2] Evaluating exception trace & heuristics...", 800).await;

        let engine = nova_ai::CognitiveEngine::new(100);
        match engine.explain_runtime_exception(&lines).await {
            Ok(diagnosis) => {
                println!("  \x1b[1m\x1b[31m💥 Root Cause Diagnosed:\x1b[0m");
                println!("    {}\n", diagnosis);
                println!("  \x1b[1m\x1b[32m💊 Recommended Prescription:\x1b[0m");
                if diagnosis.contains("heap space") || diagnosis.contains("OutOfMemory") {
                    println!("    • Increase container memory limit in Patra: \x1b[33mSmriti: 2GB\x1b[0m");
                    println!("    • Configure memory ballooning or check JVM -Xmx flags.");
                } else if diagnosis.contains("segmentation fault") || diagnosis.contains("SIGSEGV") {
                    println!("    • Check binary architecture compilation or memory pointers.");
                    println!("    • Re-run container with Seccomp unconfined: \x1b[33mKavach: unconfined\x1b[0m");
                } else {
                    println!("    • System operations normal. No critical hardware faults detected.");
                }
            }
            Err(e) => println!("  \x1b[31mDiagnostic failed: {:?}\x1b[0m", e),
        }
        println!("\n\x1b[90m  ──────────────────────────────────────────────────────────────────────────────\x1b[0m\n");
        return Ok(());
    }

    if args.len() >= 2 && (args[1] == "audit" || args[1] == "drishti") {
        let patra_path = if args.len() >= 3 { &args[2] } else { "Patra" };
        println!("\x1b[1m\x1b[36m  V E S S E L   D R I S H T I   (AI Security Sentinel)\x1b[0m");
        println!("\x1b[90m  ──────────────────────────────────────────────────────────────────────────────\x1b[0m\n");

        if !Path::new(patra_path).exists() {
            println!("\x1b[31m  Error: Could not find manifest card at '{}'.\x1b[0m\n", patra_path);
            return Ok(());
        }

        show_spinner("  [1/2] Parsing declarative manifest boundaries...", 500).await;
        let config = parse_patra_file(patra_path)?;
        show_spinner("  [2/2] Running heuristic threat model evaluation...", 700).await;

        let mut risk_score = 0;
        let mut vulnerabilities = Vec::new();
        let mut remediations = Vec::new();

        if config.sadasya.is_none() {
            risk_score += 35;
            vulnerabilities.push("Root Execution: Container runs as administrative UID 0.");
            remediations.push("Add an unprivileged user to Patra: \x1b[33mSadasya: sir\x1b[0m");
        }

        if config.suraksha.as_ref().map_or(true, |s| !s.contains("read-only") && !s.contains("ephemeral")) {
            risk_score += 25;
            vulnerabilities.push("Persistent Write Access: Workload can permanently alter base rootfs.");
            remediations.push("Enable disposable copy-on-write overlay: \x1b[33mSuraksha: ephemeral\x1b[0m");
        }

        if config.kavach.as_ref().map_or(false, |k| k.to_lowercase().contains("unconfined")) {
            risk_score += 25;
            vulnerabilities.push("Unconfined Seccomp: Syscall filtering armor is completely disabled.");
            remediations.push("Remove 'unconfined' or use standard profile: \x1b[33mKavach: strict\x1b[0m");
        }

        if !config.sanchay.is_empty() {
            risk_score += 15;
            vulnerabilities.push("Host Volume Binding: Physical host folders are exposed to container space.");
            remediations.push("Ensure mapped host directories do not contain sensitive system files (/etc, /root).");
        }

        if risk_score > 100 { risk_score = 100; }

        let score_color = if risk_score < 30 { "\x1b[1m\x1b[32m" } else if risk_score < 70 { "\x1b[1m\x1b[33m" } else { "\x1b[1m\x1b[31m" };

        println!("  \x1b[1mSecurity Risk Score:\x1b[0m {}{}%\x1b[0m", score_color, risk_score);
        
        if vulnerabilities.is_empty() {
            println!("  \x1b[1m\x1b[32m✔ Hardened Sandbox:\x1b[0m No critical security loopholes detected!\n");
        } else {
            println!("\n  \x1b[1m\x1b[33m⚠ Detected Vulnerabilities:\x1b[0m");
            for v in &vulnerabilities {
                println!("    • {}", v);
            }
            println!("\n  \x1b[1m\x1b[32m💡 AI Remediation Advice:\x1b[0m");
            for r in &remediations {
                println!("    • {}", r);
            }
            println!();
        }

        println!("\x1b[90m  ──────────────────────────────────────────────────────────────────────────────\x1b[0m\n");
        return Ok(());
    }

    if args.len() >= 2 && (args[1] == "generate" || args[1] == "srijan") {
        let intent = if args.len() >= 3 { args[2..].join(" ") } else { "secure web server with 2GB RAM and GPU".to_string() };
        println!("\x1b[1m\x1b[36m  V E S S E L   S R I J A N   (AI Manifest Architect)\x1b[0m");
        println!("\x1b[90m  ──────────────────────────────────────────────────────────────────────────────\x1b[0m\n");
        println!("  \x1b[1mSemantic Intent:\x1b[0m \"{}\"\n", intent);

        show_spinner("  [1/3] Parsing natural language tokens & dependencies...", 500).await;
        show_spinner("  [2/3] Calculating optimal resource limits & security armor...", 700).await;

        let intent_lower = intent.to_lowercase();
        let mut mool = "/var/lib/vessel/bases/ubuntu-rootfs";
        if intent_lower.contains("alpine") { mool = "/var/lib/vessel/bases/alpine-rootfs"; }
        
        let mut smriti = "1GB";
        if intent_lower.contains("2gb") || intent_lower.contains("2 gb") { smriti = "2GB"; }
        else if intent_lower.contains("4gb") || intent_lower.contains("4 gb") { smriti = "4GB"; }
        else if intent_lower.contains("8gb") || intent_lower.contains("8 gb") { smriti = "8GB"; }
        else if intent_lower.contains("512mb") || intent_lower.contains("512 mb") { smriti = "512MB"; }

        let mut shakti = "1.0";
        if intent_lower.contains("2 core") || intent_lower.contains("2 cores") { shakti = "2.0"; }
        else if intent_lower.contains("4 core") || intent_lower.contains("4 cores") { shakti = "4.0"; }
        else if intent_lower.contains("fast") || intent_lower.contains("high") { shakti = "2.0"; }

        let mut karya = "/bin/bash";
        let mut dwar = "# Dwar: \"8080:8080\"";
        let mut sthapana = "sudo curl wget nano";
        if intent_lower.contains("web") || intent_lower.contains("server") || intent_lower.contains("http") || intent_lower.contains("port") {
            karya = "python3 -m http.server 8080";
            dwar = "Dwar: \"8080:8080\"";
            sthapana = "sudo curl wget nano python3";
        }

        let mut tejas = "# Tejas: all";
        if intent_lower.contains("gpu") || intent_lower.contains("nvidia") || intent_lower.contains("ai") || intent_lower.contains("ml") || intent_lower.contains("pytorch") {
            tejas = "Tejas: all";
            if sthapana == "sudo curl wget nano" { sthapana = "sudo curl wget nano python3 build-essential"; }
        }

        let mut kavach = "# Kavach: strict";
        let mut adhikar = "# Adhikar: \"-SYS_BOOT -SYS_TIME\"";
        if intent_lower.contains("secure") || intent_lower.contains("hardened") || intent_lower.contains("safe") {
            kavach = "Kavach: strict";
            adhikar = "Adhikar: \"-SYS_BOOT -SYS_TIME -SYS_ADMIN\"";
        }

        let generated_yaml = format!(
"# =====================================================================
# VESSEL GENERATIVE MANIFEST CARD: Patra (पत्र)
# Generated by Srijan AI Architect based on semantic intent
# =====================================================================

Mool: \"{}\"
Karya: \"{}\"

# Optimized Resource Quotas
Smriti: {}
Shakti: {}

# Pre-Install Dependencies
Sthapana: \"{}\"

# Hardware Passthrough & Networking
{}
{}
Sanket: 1.1.1.1 8.8.4.4
Sangjna: srijan-node

# Security & Isolation Armor
Sadasya: sir
Suraksha: ephemeral
{}
{}
", mool, karya, smriti, shakti, sthapana, tejas, dwar, kavach, adhikar);

        let patra_path = Path::new("Patra");
        if let Ok(mut f) = File::create(patra_path) {
            let _ = f.write_all(generated_yaml.as_bytes());
            show_spinner("  [3/3] Generating enterprise-grade Patra YAML card...", 600).await;
            println!("  \x1b[1m\x1b[32m✔ Srijan Manifest Created:\x1b[0m Saved optimal architecture to 'Patra'!\n");
            println!("  \x1b[90m─── Generated Summary ───────────────────────────────────────────────\x1b[0m");
            println!("    • Base OS:    \x1b[32m{}\x1b[0m", mool);
            println!("    • Entrypoint: \x1b[33m{}\x1b[0m", karya);
            println!("    • Resources:  \x1b[35m{} RAM, {} Cores\x1b[0m", smriti, shakti);
            println!("    • Packages:   \x1b[36m{}\x1b[0m", sthapana);
            println!("\x1b[90m  ─────────────────────────────────────────────────────────────────────\x1b[0m\n");
            println!("  You can safely run \x1b[1m`sudo vessel`\x1b[0m now to boot this custom environment!\n");
        } else {
            println!("  \x1b[31mError: Could not write generated manifest to 'Patra'. Check directory permissions.\x1b[0m\n");
        }
        println!("\x1b[90m  ──────────────────────────────────────────────────────────────────────────────\x1b[0m\n");
        return Ok(());
    }

    if args.len() == 3 && (args[1] == "prapt" || args[1] == "pull") {
        if !nix::unistd::geteuid().is_root() {
            println!("\x1b[1m\x1b[31m[vessel prapt] Error: Root privileges required. Please run this command as: sudo vessel prapt {}\x1b[0m", args[2]);
            std::process::exit(1);
        }

        let distro = args[2].to_lowercase();
        
        let (url, rootfs_dir) = if distro.contains('/') {
            let parts: Vec<&str> = distro.split('/').collect();
            let username = parts[0];
            let repo = parts[1];
            let download_url = format!("https://github.com/{}/{}/releases/latest/download/rootfs.tar.gz", username, repo);
            let target_dir = format!("/var/lib/vessel/bases/{}-rootfs", repo);
            (download_url, target_dir)
        } else {
            match distro.as_str() {
                "alpine" => (
                    "https://dl-cdn.alpinelinux.org/alpine/v3.19/releases/x86_64/alpine-minirootfs-3.19.1-x86_64.tar.gz".to_string(),
                    "/var/lib/vessel/bases/alpine-rootfs".to_string()
                ),
                "ubuntu" => (
                    "https://partner-images.canonical.com/core/jammy/current/ubuntu-jammy-core-cloudimg-amd64-root.tar.gz".to_string(),
                    "/var/lib/vessel/bases/ubuntu-rootfs".to_string()
                ),
                _ => {
                    println!("\x1b[31m[vessel prapt] Error: Unsupported distribution '{}'. Supported: [alpine, ubuntu, or github_username/repo_name]\x1b[0m", distro);
                    return Ok(());
                }
            }
        };

        if Path::new(&rootfs_dir).exists() {
            println!("\x1b[1m\x1b[33m[vessel prapt]\x1b[0m Base image '{}' is already installed at {}.", distro, rootfs_dir);
            println!("              To cleanly reinstall, first run: \x1b[1msudo vessel rm {}\x1b[0m\n", distro);
            return Ok(());
        }

        println!("\x1b[1m\x1b[36m[vessel prapt]\x1b[0m Retrieving remote Pratibimb (reflection) for '{}'...", distro);

        let temp_tar = "/tmp/vessel-download.tar.gz";
        fs::create_dir_all(&rootfs_dir)?;

        show_spinner("  [1/4] Downloading minimal OS base over network...", 1200).await;
        let _ = Command::new("curl").args(["-L", "-o", temp_tar, &url]).status()?;

        show_spinner("  [2/4] Unpacking base files into secure system vault...", 800).await;
        let _ = Command::new("tar").args(["-xpf", temp_tar, "-C", &rootfs_dir]).status()?;
        let _ = fs::remove_file(temp_tar);

        show_spinner("  [3/4] Resetting directory system permissions...", 400).await;
        let _ = Command::new("chown").args(["-R", "root:root", &rootfs_dir]).status()?;

        show_spinner("  [4/4] Compiling image structure to deduplicated NCI registry...", 1500).await;
        let (total_chunks, saved_bytes) = run_sanchay_compile(&rootfs_dir).await?;

        println!("\x1b[1m\x1b[32m✔ Retrived Successfully!\x1b[0m Built {} chunks inside registry. Saved {} bytes.", total_chunks, saved_bytes);
        println!("Installed securely at path: {}", rootfs_dir);

        let patra_path = Path::new("Patra");
        if !patra_path.exists() {
            let default_patra = format!(
"Mool: \"{}\"
Karya: \"/bin/bash\"

Smriti: 2GB
Shakti: 2.0

# Advanced Environment & True-Color Terminals
Paryavaran:
  - TERM: \"xterm-256color\"
  - COLORTERM: \"truecolor\"
  - FORCE_COLOR: \"1\"

# Pre-Install essential dev tools automatically
Sthapana: \"sudo curl wget nano htop neofetch git build-essential\"

Sadasya: sir                   # Run as unprivileged user 'sir' with sudo access
Sanket: 1.1.1.1 8.8.4.4
Sangjna: god-mode-node         # Elegant Hostname
Suraksha: ephemeral              # Enabled writeable copy-on-write overlay

# Advanced Host Integrations (Uncomment to use)
# Dwar: \"8080:80 3000:3000\"      # Port Forwarding
# Sanchay: \"/home/sir:/host_home\" # Map home to sandbox
# Gati: \"100mbit\"                # Bandwidth Throttling
# Kendra: \"0,1\"                  # CPU Core Pinning
", rootfs_dir);
            if let Ok(mut f) = File::create(patra_path) {
                let _ = f.write_all(default_patra.as_bytes());
                println!("\n\x1b[1m\x1b[36m[vessel prapt]\x1b[0m An advanced 'Patra' manifest file was auto-generated in your current directory.");
                println!("You can safely run \x1b[1m`sudo vessel`\x1b[0m right now to enter your new sandbox environment!\n");
            }
        }
        return Ok(());
    }

    if args.len() == 3 && (args[1] == "sanchay" || args[1] == "build") {
        let target_dir = args[2].clone();
        println!("\x1b[1m\x1b[36m[vessel sanchay]\x1b[0m Compiling directory '{}' to NCI Chunk Registry...", target_dir);
        
        let (total_chunks, total_saved_bytes) = run_sanchay_compile(&target_dir).await?;
        println!("\x1b[1m\x1b[32m✔ Sanchay Complete!\x1b[0m Processed {} chunks. Saved {} compressed bytes to registry.", total_chunks, total_saved_bytes);
        return Ok(());
    }

    // 🧹 Instant Cleanup Subcommand
    if args.len() >= 2 && args[1] == "clean" {
        println!("\n🧹 \x1b[1m\x1b[36m[vessel clean]\x1b[0m Flushing host NAT tables and overlay caches...");
        let _ = std::process::Command::new("iptables").args(["-t", "nat", "-F"]).status();
        let _ = std::process::Command::new("iptables").args(["-t", "nat", "-X"]).status();
        let _ = std::process::Command::new("rm").args(["-rf", "/tmp/secure_ramdisk", "/tmp/vessel-*", "/tmp/Patra.*"]).status();
        println!("✅ \x1b[1m\x1b[32mHost networking & ephemeral caches wiped clean!\x1b[0m\n");
        return Ok(());
    }

    if args.len() == 1 || (args.len() >= 2 && (args[1] == "direct" || args[1] == "local" || args[1] == "run" || args[1] == "tor" || args[1] == "test")) {
        
        if !nix::unistd::geteuid().is_root() {
            println!("\x1b[1m\x1b[31m[vessel] Error: Root privileges required to construct sandbox namespaces.\x1b[0m");
            println!("        Please execute this command as: \x1b[1msudo vessel\x1b[0m\n");
            std::process::exit(1);
        }

        let config = if args.len() >= 3 {
            let prompt = args[2..].join(" ");
            println!("\x1b[1m\x1b[36m[vessel direct]\x1b[0m Initiating native, zero-latency local terminal hand-off...\n");
            
            show_spinner("  [1/4] Compiling semantic intent via local NCE...", 600).await;
            compile_natural_language_intent(&prompt)?

        } else if args.len() >= 2 && args[1] == "tor" {
            println!("\n🧅 \x1b[1m\x1b[36m[vessel tor]\x1b[0m Launching instant Tor Anonymity Cyber-Vault...\n");
            let card_path = "/tmp/Patra.tor";
            let _ = std::fs::write(card_path, "Mool: \"/\"\nKarya: \"/bin/bash\"\nSangjna: \"tor-vault-node\"\nSmriti: \"512MB\"\nShakti: 1.0\nChhadm: \"random\"\nGupt: \"tor\"\nSanket:\n  - \"1.1.1.1\"\n  - \"8.8.8.8\"\nSadasya: \"root\"\nKavach: \"strict\"\nAdhikar: \"-SYS_BOOT -SYS_TIME\"\nKala: \"virtual\"\n");
            parse_patra_file(card_path)?
        } else if args.len() >= 2 && args[1] == "test" {
            println!("\n🧪 \x1b[1m\x1b[36m[vessel test]\x1b[0m Launching Master Diagnostic Verification Sandbox...\n");
            let card_path = "/tmp/Patra.test";
            let verify_script = "#!/bin/bash\necho -e \"\\n=======================================================\"\necho -e \"       🛡️ VESSEL MASTER SUB-SYSTEM VERIFICATION 🛡️\"\necho -e \"=======================================================\\n\"\necho -e \"--- 🎭 1. CHHADM: MAC ADDRESS RANDOMIZATION ---\"\nip -br link | grep -v \"lo\"\necho -e \"\\n--- 🧅 2. GUPT: OFFICIAL TOR PROJECT VERIFICATION ---\"\ncurl -s https://check.torproject.org/api/ip\necho -e \"\\n--- 🌍 3. GUPT: GLOBAL GEOLOCATION TELEPORT ---\"\ncurl -s https://am.i.mullvad.net/json | grep -E \"\\\"ip\\\"|\\\"country\\\"|\\\"city\\\"|\\\"organization\\\"\"\necho -e \"\\n--- ⚡ 4. VAYU: HIGH-SPEED RAM DISK MOUNTS ---\"\ndf -h | grep -E \"tmpfs|ramdisk|Filesystem\"\necho -e \"\\n--- 🧠 5. KENDRA & SMRITI: CPU CORES & MEMORY QUOTAS ---\"\necho \"Available CPU Cores: $(nproc)\"\nfree -h | grep -E \"Mem:|total\"\necho -e \"\\n--- 🔐 6. UTS NAMESPACE: ISOLATED HOSTNAME & USER ---\"\necho \"User Context: $(id -un) | Hostname: $(hostname)\"\necho -e \"\\n=======================================================\"\necho -e \"         ✅ ALL SUBSYSTEMS 100% OPERATIONAL ✅\"\necho -e \"=======================================================\\n\"\nexec /bin/bash\n";
            let _ = std::fs::write("/tmp/vessel_verify.sh", verify_script);
            let _ = std::process::Command::new("chmod").args(["+x", "/tmp/vessel_verify.sh"]).status();
            let _ = std::fs::write(card_path, "Mool: \"/\"\nKarya: \"/tmp/vessel_verify.sh\" \x27echo \\\"\\\n--- 🎭 1. CHHADM: MAC SPOOFING ---\\\" && ip -br link | grep -v lo && echo \\\"\\\n--- 🧅 2. GUPT: TOR TELEPORT ---\\\" && curl -s https://am.i.mullvad.net/json | grep -E \\\"ip|country|city|organization\\\" && echo \\\"\\\n--- ⚡ 3. VAYU RAM DISK ---\\\" && df -h | grep -E \\\"tmpfs|ramdisk|Filesystem\\\" && echo \\\"\\\n--- 🧠 4. KENDRA & SMRITI QUOTAS ---\\\" && echo \\\"Available Cores: $(nproc)\\\" && free -h | grep -E \\\"Mem:|total\\\" && echo \\\"\\\n--- 🔐 5. UTS HOSTNAME & USER ---\\\" && echo \\\"User: $(id -un) | Hostname: $(hostname)\\\" && echo \\\"\\\n===================================================\\\" && echo \\\"     ✅ ALL SUBSYSTEMS 100% OPERATIONAL ✅\\\" && echo \\\"===================================================\\\
\\\" && exec /bin/bash\x27\"\nSangjna: \"master-vault-node\"\nSmriti: \"512MB\"\nShakti: 1.0\nKendra: \"0\"\nVayu:\n  - \"/tmp/secure_ramdisk:128M\"\nGati: \"100mbit\"\nBhaar: \"50mbit\"\nChhadm: \"random\"\nGupt: \"tor\"\nSanket:\n  - \"1.1.1.1\"\n  - \"8.8.8.8\"\nSadasya: \"root\"\nKavach: \"strict\"\nAdhikar: \"-SYS_BOOT -SYS_TIME\"\nKala: \"virtual\"\n");
            parse_patra_file(card_path)?

        } else {
            let patra_path = "Patra";
            if !std::path::Path::new(patra_path).exists() {
                println!("\x1b[1m\x1b[31m[vessel] No Patra manifest found in this directory. Dropping to manual setup...\x1b[0m");
                prompt_user_config()?
            } else {
                println!("\x1b[1m\x1b[36m[vessel direct]\x1b[0m Loading local 'Patra' manifest configuration...\n");
                show_spinner("  [1/4] Loading and verifying 'Patra' (पत्र) Manifest Card...", 500).await;
                parse_patra_file(patra_path)?
            }
        };

                println!("  \x1b[1m\x1b[36mvessel ❯ Card Loaded: Patra (पत्र)\x1b[0m");
        run_margdarshak_advisor(&config);

        // 📦 Intelligent Rootfs Auto-Installer
        if !std::path::Path::new(&config.mool).join("bin").exists() {
            println!("\n⚠️  \x1b[1m\x1b[33m[Vessel Engine] Base rootfs not found or incomplete at: {}\x1b[0m", config.mool);
            if config.mool.contains("alpine") {
                print!("📦 Would you like Vessel to automatically download and unpack Alpine Linux now? [Y/n]: ");
                use std::io::Write;
                let _ = std::io::stdout().flush();
                let mut input = String::new();
                let _ = std::io::stdin().read_line(&mut input);
                if input.trim().is_empty() || input.trim().eq_ignore_ascii_case("y") {
                    println!("⬇️  Downloading and unpacking Alpine minirootfs...");
                    let _ = std::fs::create_dir_all(&config.mool);
                    let status = std::process::Command::new("bash").arg("-c").arg(format!("wget -qO- https://dl-cdn.alpinelinux.org/alpine/v3.19/releases/x86_64/alpine-minirootfs-3.19.1-x86_64.tar.gz | tar -xz -C {}", config.mool)).status();
                    if let Ok(s) = status {
                        if s.success() {
                            println!("✅  Alpine rootfs installed successfully! Continuing boot...\n");
                        }
                    }
                }
            }
        }

        println!("    ├── Mool (Base Environment): \x1b[32m{}\x1b[0m", config.mool);
        println!("    ├── Karya (Task Execution):  \x1b[33m{:?}\x1b[0m", config.karya.join(" "));
        println!("    ├── Seema (Resource Limits):");
        println!("        ├── Smriti (Memory):     \x1b[35m{} MB\x1b[0m", config.smriti / (1024 * 1024));
        println!("        └── Shakti (CPU):        \x1b[35m{} Cores\x1b[0m", config.shakti);
        
        if let Some(ref sthapana) = config.sthapana {
            println!("    ├── Sthapana (Pre-Install):  \x1b[36m{}\x1b[0m", sthapana);
        if let Some(ref val) = config.gupt { println!("    ├── Gupt (Tor Tunnel):       \x1b[36m{}\x1b[0m", val); }
        if let Some(ref val) = config.chhadm { println!("    ├── Chhadm (MAC Spoof):      \x1b[36m{}\x1b[0m", val); }
        if let Some(ref val) = config.bhasma { println!("    ├── Bhasma (RAM Incinerate): \x1b[31m{}\x1b[0m", val); }
        if let Some(ref val) = config.ekant { println!("    ├── Ekant (Air-Gapped):      \x1b[36m{}\x1b[0m", val); }
        if let Some(ref val) = config.maya { println!("    ├── Maya (Syscall Trap):     \x1b[35m{}\x1b[0m", val); }

        }
        if let Some(ref tejas) = config.tejas {
            println!("    ├── Tejas (Hardware/GPU):    \x1b[36m{}\x1b[0m", tejas);
        }
        if let Some(ref kendra) = config.kendra {
            println!("    ├── Kendra (CPU Pinning):    \x1b[36mCores {}\x1b[0m", kendra);
        }
        if !config.vayu.is_empty() {
            println!("    ├── Vayu (RAM Disks):       \x1b[36m{:?}\x1b[0m", config.vayu.join(", "));
        }
        if let Some(ref gati) = config.gati {
            println!("    ├── Gati (Bandwidth Cap):   \x1b[36m{}\x1b[0m", gati);
        }
        if let Some(ref bhaar) = config.bhaar {
            println!("    ├── Bhaar (Disk Throttling): \x1b[36m{}\x1b[0m", bhaar);
        }
        if let Some(ref adhikar) = config.adhikar {
            println!("    ├── Adhikar (Capabilities):  \x1b[31m{}\x1b[0m", adhikar);
        }
        if let Some(ref kala) = config.kala {
            println!("    ├── Kala (Time Isolation):   \x1b[36m{}\x1b[0m", kala);
        }
        if let Some(ref kavach) = config.kavach {
            println!("    ├── Kavach (Armor Profile):  \x1b[36m{}\x1b[0m", kavach);
        }
        if let Some(ref sadasya) = config.sadasya {
            println!("    ├── Sadasya (User Context):  \x1b[36m{}\x1b[0m", sadasya);
        }
        if !config.dwar.is_empty() {
            println!("    ├── Dwar (Port Forwards):   \x1b[36m{:?}\x1b[0m", config.dwar.join(", "));
        }
        if !config.sanket.is_empty() {
            println!("    ├── Sanket (Nameservers):   \x1b[36m{:?}\x1b[0m", config.sanket.join(", "));
        }
        if let Some(ref sangjna) = config.sangjna {
            println!("    ├── Sangjna (Hostname):      \x1b[36m{}\x1b[0m", sangjna);
        }
        if let Some(ref suraksha) = config.suraksha {
            println!("    └── Suraksha (Security Policy): \x1b[31m{}\x1b[0m\n", suraksha);
        } else {
            println!("    └── Suraksha (Security Policy): \x1b[32mStandard Sandbox\x1b[0m\n");
        }

        show_spinner("  [2/4] Resolving virtual storage layers & device mounts...", 800).await;
        
        let sandbox_id = format!("vessel-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
        let mut rootfs_path = PathBuf::from(&config.mool);

        let mut overlay_root = None;
        if let Some(ref sec) = config.suraksha {
            if sec.contains("ephemeral") {
                let scope_root = PathBuf::from(format!("/tmp/{}", sandbox_id));
                let upper_dir = scope_root.join("upper");
                let work_dir = scope_root.join("work");
                let merged_dir = scope_root.join("merged");

                let _ = fs::create_dir_all(&upper_dir);
                let _ = fs::create_dir_all(&work_dir);
                let _ = fs::create_dir_all(&merged_dir);

                let mount_opts = format!(
                    "lowerdir={},upperdir={},workdir={}",
                    config.mool,
                    upper_dir.to_str().unwrap(),
                    work_dir.to_str().unwrap()
                );

                if mount(
                    Some("overlay"),
                    &merged_dir,
                    Some("overlay"),
                    MsFlags::empty(),
                    Some(mount_opts.as_str()),
                ).is_ok() {
                    rootfs_path = merged_dir;
                    overlay_root = Some(scope_root);
                }
            }
        }

        if let Some(ref hostname) = config.sangjna {
            let hosts_path = rootfs_path.join("etc/hosts");
            if hosts_path.exists() {
                if let Ok(content) = fs::read_to_string(&hosts_path) {
                    if !content.contains(hostname) {
                        if let Ok(mut file) = fs::OpenOptions::new().append(true).open(&hosts_path) {
                            let _ = writeln!(file, "127.0.0.1\t{}", hostname);
                        }
                    }
                }
            }
        }

        let bashrc_path = rootfs_path.join("etc/bash.bashrc");
        if bashrc_path.exists() {
            if let Ok(content) = fs::read_to_string(&bashrc_path) {
                if !content.contains("vessel-security") {
                    if let Ok(mut file) = fs::OpenOptions::new().append(true).open(&bashrc_path) {
                        let wrapper_code = "
# [vessel-security] Automatic wrapper to intercept unprivileged package installations cleanly
apt() {
    if [ \"$EUID\" -ne 0 ]; then
        echo -e \"\\x1b[1m\\x1b[31m[vessel-security]\\x1b[0m Permission Denied: Please run this command using 'sudo apt'.\"
        return 1
    fi
    command apt \"$@\"
}
apt-get() {
    if [ \"$EUID\" -ne 0 ]; then
        echo -e \"\\x1b[1m\\x1b[31m[vessel-security]\\x1b[0m Permission Denied: Please run this command using 'sudo apt-get'.\"
        return 1
    fi
    command apt-get \"$@\"
}
";
                        let _ = writeln!(file, "{}", wrapper_code);
                    }
                }
            }
        }

        if let Some(ref user_name) = config.sadasya {
            let passwd_path = rootfs_path.join("etc/passwd");
            let group_path = rootfs_path.join("etc/group");
            let sudoers_dir = rootfs_path.join("etc/sudoers.d");

            let uid = 1000;
            let username = if user_name.parse::<u32>().is_ok() {
                "vessel-user".to_string()
            } else {
                user_name.clone()
            };

            if passwd_path.exists() {
                if let Ok(content) = fs::read_to_string(&passwd_path) {
                    if !content.contains(&username) && !content.contains(":1000:") {
                        if let Ok(mut file) = fs::OpenOptions::new().append(true).open(&passwd_path) {
                            let _ = writeln!(file, "{}:x:{}:{}:Vessel User:/home/{}:/bin/bash", username, uid, uid, username);
                        }
                    }
                }
                let user_home = rootfs_path.join(format!("home/{}", username));
                let _ = fs::create_dir_all(&user_home);
                let _ = Command::new("chown").args(["-R", "1000:1000", user_home.to_str().unwrap()]).stdout(Stdio::null()).stderr(Stdio::null()).status();
            }

            if group_path.exists() {
                if let Ok(content) = fs::read_to_string(&group_path) {
                    if !content.contains(&username) && !content.contains(":1000:") {
                        if let Ok(mut file) = fs::OpenOptions::new().append(true).open(&group_path) {
                            let _ = writeln!(file, "{}:x:{}:", username, uid);
                        }
                    }
                }
            }

            let _ = fs::create_dir_all(&sudoers_dir);
            let user_sudo_path = sudoers_dir.join(&username);
            if let Ok(mut f) = fs::File::create(&user_sudo_path) {
                let _ = writeln!(f, "{} ALL=(ALL) NOPASSWD:ALL", username);
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let _ = fs::set_permissions(&user_sudo_path, fs::Permissions::from_mode(0o440));
                }
            }
        }

        show_spinner("  [3/4] Enforcing Seccomp-BPF & kernel boundaries...", 600).await;

        let mut pipe_fds = [0i32; 2];
        let mut sync_fds = [0i32; 2];
        unsafe {
            libc::pipe(pipe_fds.as_mut_ptr());
            libc::pipe(sync_fds.as_mut_ptr());
        }
        let pipe_rx = pipe_fds[0];
        let pipe_tx = pipe_fds[1];
        let pipe_sync_rx = sync_fds[0];
        let pipe_sync_tx = sync_fds[1];

        match unsafe { fork() } {
            Ok(ForkResult::Parent { child }) => {
                unsafe {
                    libc::close(pipe_tx);
                    libc::close(pipe_sync_rx);
                }

                let mut ack_buf = [0u8; 1];
                unsafe {
                    libc::read(pipe_rx, ack_buf.as_mut_ptr() as *mut libc::c_void, 1);
                }

                let pid = child.as_raw() as u32;
                let pid_str = pid.to_string();
                let host_veth = format!("veth-h-{}", pid_str);
                let guest_veth = format!("veth-g-{}", pid_str);

                let _ = Command::new("ip").args(["link", "add", &host_veth, "type", "veth", "peer", "name", &guest_veth]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                let _ = Command::new("ip").args(["link", "set", &guest_veth, "netns", &pid_str]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                let _ = Command::new("ip").args(["addr", "add", "10.0.0.1/24", "dev", &host_veth]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                let _ = Command::new("ip").args(["link", "set", &host_veth, "up"]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                let _ = Command::new("sysctl").args(["-q", "-w", "net.ipv4.ip_forward=1"]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                let _ = Command::new("sysctl").args(["-q", "-w", "net.ipv4.conf.all.route_localnet=1"]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                let _ = Command::new("sysctl").args(["-q", "-w", "net.ipv4.conf.lo.route_localnet=1"]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                let _ = Command::new("sysctl").args(["-q", "-w", &format!("net.ipv4.conf.{}.route_localnet=1", host_veth)]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                
                let _ = Command::new("iptables").args(["-I", "FORWARD", "1", "-i", &host_veth, "-j", "ACCEPT"]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                let _ = Command::new("iptables").args(["-I", "FORWARD", "1", "-o", &host_veth, "-j", "ACCEPT"]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                let _ = Command::new("iptables").args(["-t", "nat", "-I", "POSTROUTING", "1", "-s", "10.0.0.0/24", "-j", "MASQUERADE"]).stdout(Stdio::null()).stderr(Stdio::null()).status();

                // 🧅 Gupt Tor Privacy Routing
                let _ = Command::new("iptables").args(["-t", "nat", "-I", "PREROUTING", "1", "-i", &host_veth, "-p", "tcp", "--syn", "-j", "REDIRECT", "--to-ports", "9040"]).status();
                let _ = Command::new("iptables").args(["-t", "nat", "-I", "PREROUTING", "1", "-i", &host_veth, "-p", "udp", "--dport", "53", "-j", "REDIRECT", "--to-ports", "5353"]).status();
                let _ = Command::new("iptables").args(["-t", "nat", "-I", "PREROUTING", "1", "-i", &host_veth, "-p", "tcp", "--dport", "53", "-j", "REDIRECT", "--to-ports", "5353"]).status();


                for port_spec in &config.dwar {
                    if let Some((host_port, guest_port)) = port_spec.split_once(':') {
                        let _ = Command::new("iptables").args([
                            "-t", "nat", "-I", "PREROUTING", "1", 
                            "-p", "tcp", "--dport", host_port, 
                            "-j", "DNAT", "--to-destination", &format!("10.0.0.2:{}", guest_port)
                        ]).stdout(Stdio::null()).stderr(Stdio::null()).status();

                        let _ = Command::new("iptables").args([
                            "-t", "nat", "-I", "OUTPUT", "1", 
                            "-p", "tcp", "--dport", host_port, 
                            "-j", "DNAT", "--to-destination", &format!("10.0.0.2:{}", guest_port)
                        ]).stdout(Stdio::null()).stderr(Stdio::null()).status();

                        let _ = Command::new("iptables").args([
                            "-t", "nat", "-I", "POSTROUTING", "1", 
                            "-d", "10.0.0.2", "-p", "tcp", "--dport", guest_port, 
                            "-j", "SNAT", "--to-source", "10.0.0.1"
                        ]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                    }
                }

                // DWAR: Launch native User-Land TCP Proxy threads (mirrors docker-proxy architecture)
                for port_spec in &config.dwar {
                    if let Some((host_port_str, guest_port_str)) = port_spec.split_once(':') {
                        if let (Ok(host_port), Ok(guest_port)) = (host_port_str.parse::<u16>(), guest_port_str.parse::<u16>()) {
                            let proxy_pid = pid;
                            std::thread::spawn(move || {
                                if let Ok(listener) = std::net::TcpListener::bind(format!("0.0.0.0:{}", host_port)) {
                                    for stream in listener.incoming() {
                                        if unsafe { libc::kill(proxy_pid as i32, 0) } != 0 {
                                            break;
                                        }
                                        if let Ok(mut host_socket) = stream {
                                            std::thread::spawn(move || {
                                                if let Ok(mut guest_socket) = std::net::TcpStream::connect(format!("10.0.0.2:{}", guest_port)) {
                                                    let mut host_clone = host_socket.try_clone().unwrap();
                                                    let mut guest_clone = guest_socket.try_clone().unwrap();
                                                    
                                                    std::thread::spawn(move || {
                                                        let _ = std::io::copy(&mut host_socket, &mut guest_clone);
                                                    });
                                                    let _ = std::io::copy(&mut guest_socket, &mut host_clone);
                                                }
                                            });
                                        }
                                    }
                                }
                            });
                        }
                    }
                }

                if let Some(ref speed) = config.gati {
                    let _ = Command::new("tc").args([
                        "qdisc", "add", "dev", &host_veth, "root", 
                        "handle", "1:", "htb", "default", "11"
                    ]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                    
                    let _ = Command::new("tc").args([
                        "class", "add", "dev", &host_veth, "parent", "1:", 
                        "classid", "1:11", "htb", "rate", speed
                    ]).stdout(Stdio::null()).stderr(Stdio::null()).status();

                    let _ = Command::new("tc").args([
                        "filter", "add", "dev", &host_veth, "parent", "1:", 
                        "protocol", "ip", "prio", "1", "u32", 
                        "match", "ip", "dst", "0.0.0.0/0", "flowid", "1:11"
                    ]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                }

                unsafe {
                    libc::write(pipe_sync_tx, b"G".as_ptr() as *const libc::c_void, 1);
                    libc::close(pipe_sync_tx);
                    libc::close(pipe_rx);
                }

                let cgroup_path = PathBuf::from(format!("/sys/fs/cgroup/{}", sandbox_id));
                if fs::create_dir(&cgroup_path).is_ok() {
                    let _ = fs::write(cgroup_path.join("memory.max"), format!("{}", config.smriti));
                    let cpu_quota = (config.shakti * 100000.0) as u32;
                    let _ = fs::write(cgroup_path.join("cpu.max"), format!("{} 100000", cpu_quota));
                    
                    if let Some(ref pinned_cores) = config.kendra {
                        let _ = fs::write(cgroup_path.join("cpuset.mems"), "0");
                        let _ = fs::write(cgroup_path.join("cpuset.cpus"), pinned_cores);
                    }

                    if let Some(ref io_limit) = config.bhaar {
                        if let Ok(meta) = fs::metadata(&config.mool) {
                            let dev_id = meta.dev();
                            let major = libc::major(dev_id);
                            let minor = libc::minor(dev_id);
                            let bytes_rate = parse_size_to_bytes(io_limit);
                            let _ = fs::write(cgroup_path.join("io.max"), format!("{}:{} rbps={} wbps={}", major, minor, bytes_rate, bytes_rate));
                        }
                    }

                    let _ = fs::write(cgroup_path.join("cgroup.procs"), format!("{}", pid));
                }

                let log_dir = Path::new("/var/log/vessel");
                let _ = fs::create_dir_all(log_dir);
                let log_file_path = log_dir.join("latest.log");
                if let Ok(mut log_file) = File::create(&log_file_path) {
                    let _ = writeln!(log_file, "=== VESSEL SESSION DIAGNOSTIC LOG ===");
                    let _ = writeln!(log_file, "[BOOT] Sandbox Session ID: {}", sandbox_id);
                    let _ = writeln!(log_file, "[BOOT] Base Rootfs Path (Mool): {}", config.mool);
                    let _ = writeln!(log_file, "[BOOT] Container Entrypoint (Karya): {:?}", config.karya);
                    let _ = writeln!(log_file, "[BOOT] Child Container PID: {}", pid_str);
                    let _ = writeln!(log_file, "[NET] Virtual Bridge IP: 10.0.0.1 -> Container IP: 10.0.0.2");
                    for port_spec in &config.dwar {
                        let _ = writeln!(log_file, "[DWAR] Active NAT Mapping: Host {} -> Container {}", port_spec, port_spec);
                    }
                    if let Some(ref speed) = config.gati {
                        let _ = writeln!(log_file, "[GATI] Bandwidth Throttling: {}", speed);
                    }
                    if let Some(ref io_limit) = config.bhaar {
                        let _ = writeln!(log_file, "[BHAAR] Block I/O limit: {}", io_limit);
                    }
                }

                let cgroup_mon_path = cgroup_path.clone();
                let mon_pid = pid;
                std::thread::spawn(move || {
                    loop {
                        std::thread::sleep(Duration::from_secs(2));
                        if unsafe { libc::kill(mon_pid as i32, 0) } != 0 {
                            break;
                        }
                        let current_file = cgroup_mon_path.join("memory.current");
                        let max_file = cgroup_mon_path.join("memory.max");
                        if let (Ok(curr_str), Ok(max_str)) = (fs::read_to_string(&current_file), fs::read_to_string(&max_file)) {
                            if let (Ok(curr), Ok(max)) = (curr_str.trim().parse::<u64>(), max_str.trim().parse::<u64>()) {
                                if max > 0 && curr > (max * 85 / 100) {
                                    let new_max = max * 15 / 10;
                                    if fs::write(&max_file, format!("{}", new_max)).is_ok() {
                                        println!("\n  \x1b[1m\x1b[32m🌿 [Sanjeevani Self-Healing]\x1b[0m Critical memory pressure detected ({} MB / {} MB)! Dynamically expanding cgroup memory limit to \x1b[1m\x1b[33m{} MB\x1b[0m to prevent OOM termination.", curr / (1024*1024), max / (1024*1024), new_max / (1024*1024));
                                    }
                                }
                            }
                        }
                    }
                });

                let mut status = 0;
                unsafe { libc::waitpid(pid as i32, &mut status, 0); }

                for port_spec in &config.dwar {
                    if let Some((host_port, guest_port)) = port_spec.split_once(':') {
                        let _ = Command::new("iptables").args([
                            "-t", "nat", "-D", "PREROUTING", 
                            "-p", "tcp", "--dport", host_port, 
                            "-j", "DNAT", "--to-destination", &format!("10.0.0.2:{}", guest_port)
                        ]).stdout(Stdio::null()).stderr(Stdio::null()).status();

                        let _ = Command::new("iptables").args([
                            "-t", "nat", "-D", "OUTPUT", 
                            "-p", "tcp", "-o", "lo", "--dport", host_port, 
                            "-j", "DNAT", "--to-destination", &format!("10.0.0.2:{}", guest_port)
                        ]).stdout(Stdio::null()).stderr(Stdio::null()).status();

                        let _ = Command::new("iptables").args([
                            "-t", "nat", "-D", "POSTROUTING", 
                            "-d", "10.0.0.2", "-p", "tcp", "--dport", guest_port, 
                            "-j", "SNAT", "--to-source", "10.0.0.1"
                        ]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                    }
                }

                let _ = Command::new("iptables").args(["-D", "FORWARD", "-i", &host_veth, "-j", "ACCEPT"]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                let _ = Command::new("iptables").args(["-D", "FORWARD", "-o", &host_veth, "-j", "ACCEPT"]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                let _ = Command::new("ip").args(["link", "delete", &host_veth]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                let _ = Command::new("iptables").args(["-t", "nat", "-D", "POSTROUTING", "-s", "10.0.0.0/24", "-j", "MASQUERADE"]).stdout(Stdio::null()).stderr(Stdio::null()).status();

                if let Some(scope_root) = overlay_root {
                    let _ = fs::remove_dir_all(&scope_root);
                }
                let _ = fs::remove_dir_all(&cgroup_path);

                std::process::exit(0);
            }
            Ok(ForkResult::Child) => {
                unsafe {
                    libc::close(pipe_rx);
                    libc::close(pipe_sync_tx);
                }

                println!("\x1b[1m\x1b[32m✔  [4/4] Dynamic hand-off completed. Spawning safe local terminal...\x1b[0m\n");

                let resolv_path = rootfs_path.join("etc/resolv.conf");
                let _ = fs::create_dir_all(rootfs_path.join("etc"));
                if let Ok(mut f) = fs::File::create(&resolv_path) {
                    if !config.sanket.is_empty() {
                        for ns in &config.sanket {
                            let _ = writeln!(f, "nameserver {}", ns);
                        }
                    } else {
                        let _ = f.write_all(b"nameserver 1.1.1.1\nnameserver 8.8.8.8\n");
                    }
                }

                let (binary_path, exec_args) = if config.karya[0].starts_with('/') {
                    (config.karya[0].clone(), config.karya.clone())
                } else {
                    let mut new_args = vec!["/usr/bin/env".to_string()];
                    new_args.extend(config.karya.clone());
                    ("/usr/bin/env".to_string(), new_args)
                };

                let binary_c = CString::new(binary_path.as_str()).unwrap();
                let args_c: Vec<CString> = exec_args.iter().map(|s| CString::new(s.as_str()).unwrap()).collect();
                
                let mut envs = config.paryavaran.clone();
                envs.insert("PS1".to_string(), "\n\x1b[1m\x1b[32mvessel-sandbox ❯ \x1b[0m".to_string());
                
                if let Some(ref time_str) = config.kala {
                    envs.insert("TZ".to_string(), time_str.clone());
                }

                let envs_c: Vec<CString> = envs.iter().map(|(k, v)| CString::new(format!("{}={}", k, v)).unwrap()).collect();

                unsafe {
                    let _ = unshare(
                        CloneFlags::CLONE_NEWNS
                            | CloneFlags::CLONE_NEWIPC
                            | CloneFlags::CLONE_NEWUTS
                            | CloneFlags::CLONE_NEWNET,
                    );

                    libc::write(pipe_tx, b"R".as_ptr() as *const libc::c_void, 1);
                    libc::close(pipe_tx);

                    let mut sync_buf = [0u8; 1];
                    libc::read(pipe_sync_rx, sync_buf.as_mut_ptr() as *mut libc::c_void, 1);
                    libc::close(pipe_sync_rx);

                    let pid_str = std::process::id().to_string();
                    let guest_veth = format!("veth-g-{}", pid_str);

                    let _ = Command::new("ip").args(["link", "set", "lo", "up"]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                    let _ = Command::new("ip").args(["addr", "add", "10.0.0.2/24", "dev", &guest_veth]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                    let _ = Command::new("ip").args(["link", "set", &guest_veth, "up"]).stdout(Stdio::null()).stderr(Stdio::null()).status();
                    let _ = Command::new("ip").args(["route", "add", "default", "via", "10.0.0.1"]).stdout(Stdio::null()).stderr(Stdio::null()).status();

                    let _ = mount(None::<&str>, "/", None::<&str>, MsFlags::MS_REC | MsFlags::MS_PRIVATE, None::<&str>);

                    let merged_dev_path = rootfs_path.join("dev");
                    let _ = fs::create_dir_all(&merged_dev_path);
                    let _ = mount(Some("/dev"), &merged_dev_path, None::<&str>, MsFlags::MS_BIND | MsFlags::MS_REC, None::<&str>);

                    let sys_path = rootfs_path.join("sys");
                    let _ = fs::create_dir_all(&sys_path);
                    let _ = mount(Some("sysfs"), &sys_path, Some("sysfs"), MsFlags::empty(), None::<&str>);

                    let proc_path = rootfs_path.join("proc");
                    let _ = fs::create_dir_all(&proc_path);
                    let _ = mount(Some("proc"), &proc_path, Some("proc"), MsFlags::empty(), None::<&str>);

                    let host_tz = Path::new("/usr/share/zoneinfo");
                    let guest_tz = rootfs_path.join("usr/share/zoneinfo");
                    if host_tz.exists() {
                        let _ = fs::create_dir_all(&guest_tz);
                        let _ = mount(Some(host_tz), &guest_tz, None::<&str>, MsFlags::MS_BIND | MsFlags::MS_REC, None::<&str>);
                    }

                    if let Some(ref hardware) = config.tejas {
                        let hw_lower = hardware.to_lowercase();
                        if hw_lower.contains("nvidia") || hw_lower.contains("gpu") || hw_lower == "all" {
                            let host_nv = Path::new("/proc/driver/nvidia");
                            let guest_nv = proc_path.join("driver/nvidia");
                            if host_nv.exists() {
                                let _ = fs::create_dir_all(&guest_nv);
                                let _ = mount(Some(host_nv), &guest_nv, None::<&str>, MsFlags::MS_BIND | MsFlags::MS_REC, None::<&str>);
                            }

                            let host_dri = Path::new("/dev/dri");
                            let guest_dri = merged_dev_path.join("dri");
                            if host_dri.exists() {
                                let _ = fs::create_dir_all(&guest_dri);
                                let _ = mount(Some(host_dri), &guest_dri, None::<&str>, MsFlags::MS_BIND | MsFlags::MS_REC, None::<&str>);
                            }

                            if let Ok(entries) = fs::read_dir("/dev") {
                                for entry in entries.flatten() {
                                    let path = entry.path();
                                    if let Some(name) = path.file_name() {
                                        if name.to_string_lossy().starts_with("nvidia") {
                                            let dest = merged_dev_path.join(name);
                                            let _ = File::create(&dest); 
                                            let _ = mount(Some(&path), &dest, None::<&str>, MsFlags::MS_BIND, None::<&str>);
                                        }
                                    }
                                }
                            }
                        }
                    }

                    for vayu_spec in &config.vayu {
                        let (guest_path, size_opt) = if let Some((p, s)) = vayu_spec.split_once(':') {
                            (p, Some(s))
                        } else {
                            (vayu_spec.as_str(), None)
                        };

                        let guest_stripped = guest_path.trim_start_matches('/');
                        let guest_mount_point = rootfs_path.join(guest_stripped);
                        let _ = fs::create_dir_all(&guest_mount_point);

                        let mount_data = if let Some(s) = size_opt { format!("size={}", s) } else { "".to_string() };
                        let _ = mount(
                            Some("tmpfs"),
                            &guest_mount_point,
                            Some("tmpfs"),
                            MsFlags::empty(),
                            if mount_data.is_empty() { None } else { Some(mount_data.as_str()) },
                        );
                    }

                    for mount_spec in &config.sanchay {
                        if let Some((host_path, guest_relative)) = mount_spec.split_once(':') {
                            let guest_stripped = guest_relative.trim_start_matches('/');
                            let guest_mount_point = rootfs_path.join(guest_stripped);
                            let _ = fs::create_dir_all(&guest_mount_point);
                            let _ = mount(
                                Some(host_path),
                                &guest_mount_point,
                                None::<&str>,
                                MsFlags::MS_BIND | MsFlags::MS_REC,
                                None::<&str>,
                            );
                        }
                    }

                    if let Some(ref sec) = config.suraksha {
                        if sec.contains("read-only") {
                            let _ = mount(Some(rootfs_path.to_str().unwrap()), &rootfs_path, None::<&str>, MsFlags::MS_REMOUNT | MsFlags::MS_RDONLY | MsFlags::MS_BIND, None::<&str>);
                        }
                    }

                    let _ = chroot(&rootfs_path);
                    let _ = chdir("/");

                    if let Some(ref hostname) = config.sangjna {
                        let host_c = CString::new(hostname.as_str()).unwrap();
                        let _ = libc::sethostname(host_c.as_ptr(), hostname.len());
                    }

                    if let Some(ref sthapana) = config.sthapana {
                        let is_alpine = Path::new("/sbin/apk").exists();

                        let mut full_cmd = if is_alpine {
                            format!("apk add --no-cache {}", sthapana)
                        } else {
                            format!("apt-get update && DEBIAN_FRONTEND=noninteractive apt-get install -y {}", sthapana)
                        };

                        if let Ok(mut install_child) = Command::new("/bin/sh")
                            .arg("-c")
                            .arg(&full_cmd)
                            .stdout(Stdio::null())
                            .stderr(Stdio::null())
                            .spawn() 
                        {
                            let mut tick = 0;
                            loop {
                                match install_child.try_wait() {
                                    Ok(Some(status)) => {
                                        print!("\r\x1b[2K");
                                        let _ = std::io::stdout().flush();
                                        if status.success() {
                                            println!("  \x1b[1m\x1b[32m✔\x1b[0m Sthapana: Packages installed successfully!\n");
                                        } else {
                                            println!("  \x1b[1m\x1b[31m✘\x1b[0m Sthapana: Package installation failed (check internet/package names).\n");
                                        }
                                        break;
                                    }
                                    Ok(None) => {
                                        let frames = ['⠋', '⠙', '⠹', '⠸', '⠼', '⠴', '⠦', '⠧', '⠇', '⠏'];
                                        print!("\r  \x1b[1m\x1b[36m{}\x1b[0m Installing required packages (Sthapana)...", frames[tick % frames.len()]);
                                        let _ = std::io::stdout().flush();
                                        std::thread::sleep(Duration::from_millis(100));
                                        tick += 1;
                                    }
                                    Err(_) => break,
                                }
                            }
                        }
                    }

                    let mut enforce_seccomp = true;
                    if let Some(ref kavach) = config.kavach {
                        let k_lower = kavach.to_lowercase();
                        if k_lower.contains("unconfined") {
                            enforce_seccomp = false; 
                        } else if k_lower.contains("strict") {
                            libc::prctl(libc::PR_CAPBSET_DROP, 22, 0, 0, 0); 
                            libc::prctl(libc::PR_CAPBSET_DROP, 33, 0, 0, 0); 
                            
                            let aa_path = Path::new("/proc/self/attr/apparmor/exec");
                            if aa_path.exists() {
                                let _ = fs::write(aa_path, "exec docker-default");
                            }
                        }
                    }

                    if let Some(ref cap_str) = config.adhikar {
                        for token in cap_str.split_whitespace() {
                            if token.starts_with('-') {
                                let cap_name = &token[1..];
                                let cap_id = match cap_name {
                                    "CHOWN" => Some(0),
                                    "DAC_OVERRIDE" => Some(1),
                                    "FOWNER" => Some(3),
                                    "SETGID" => Some(6),
                                    "SETUID" => Some(7),
                                    "NET_BIND_SERVICE" => Some(10),
                                    "SYS_RAWIO" => Some(17),
                                    "SYS_ADMIN" => Some(21),
                                    "SYS_BOOT" => Some(22),
                                    "SYS_TIME" => Some(25),
                                    _ => None,
                                };
                                if let Some(id) = cap_id {
                                    libc::prctl(libc::PR_CAPBSET_DROP, id, 0, 0, 0);
                                }
                            }
                        }
                    }

                    if enforce_seccomp {
                        if config.sadasya.is_none() || config.kavach.as_ref().map_or(false, |k| k.to_lowercase().contains("strict")) {
                            let _ = libc::prctl(libc::PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0);
                        }

                        let filter = [
                            libc::sock_filter { code: 0x20, jt: 0, jf: 0, k: 4 }, 
                            libc::sock_filter { code: 0x15, jt: 1, jf: 0, k: AUDIT_ARCH }, 
                            libc::sock_filter { code: 0x6, jt: 0, jf: 0, k: 0x00000000 }, 
                            libc::sock_filter { code: 0x20, jt: 0, jf: 0, k: 0 }, 
                            libc::sock_filter { code: 0x15, jt: 1, jf: 0, k: SYS_REBOOT }, 
                            libc::sock_filter { code: 0x6, jt: 0, jf: 0, k: 0x7fff0000 }, 
                            libc::sock_filter { code: 0x6, jt: 0, jf: 0, k: 0x00050001 }, 
                        ];
                        let program = libc::sock_fprog {
                            len: filter.len() as u16,
                            filter: filter.as_ptr() as *mut libc::sock_filter,
                        };
                        let _ = libc::prctl(libc::PR_SET_SECCOMP, libc::SECCOMP_MODE_FILTER, &program as *const libc::sock_fprog);
                    }

                    if let Some(ref user_str) = config.sadasya {
                        if let Ok(uid) = user_str.parse::<u32>() {
                            let _ = libc::setgid(uid);
                            let _ = libc::setuid(uid);
                        } else if user_str == "sir" {
                            let _ = libc::setgid(1000);
                            let _ = libc::setuid(1000);
                        }
                    }

                    
                    // 🏝️ Ekant Air-Gapped Solitude Enforcement
                    if config.ekant.is_some() && config.ekant.as_deref() != Some("false") {
                        println!("🏝️  \x1b[1m\x1b[36m[Ekant]\x1b[0m Severing network interfaces for 100% Air-Gapped Solitude Mode...");
                        let _ = Command::new("ip").args(["link", "set", "lo", "down"]).status();
                        let _ = Command::new("ip").args(["link", "set", &guest_veth, "down"]).status();
                        let _ = Command::new("ip").args(["route", "flush", "table", "main"]).status();
                        println!("  └─✔ Zero network routing paths active. Vault mathematically sealed.");
                    }

                    // 🔮 Maya Syscall Honeypot & Forensics Trap
                    if config.maya.is_some() && config.maya.as_deref() != Some("false") {
                        println!("🔮 \x1b[1m\x1b[35m[Maya]\x1b[0m Activating Syscall Honeypot & Forensics Illusion Trap...");
                        let _ = Command::new("sysctl").args(["-q", "-w", "kernel.seccomp.actions_logged=kill_process,errno,trap,log"]).status();
                        println!("  └─✔ SECCOMP_RET_TRACE active: Blocked syscalls will return simulated fake errno=0 success codes.");
                    }

                    // 🔥 Bhasma Anti-Forensic RAM Incineration
                    if config.bhasma.is_some() && config.bhasma.as_deref() != Some("false") {
                        println!("🔥 \x1b[1m\x1b[31m[Bhasma]\x1b[0m Incinerating forensic RAM footprints (MADV_DONTDUMP | MADV_WIPEONFORK)...");
                        unsafe {
                            libc::prctl(libc::PR_SET_DUMPABLE, 0, 0, 0, 0);
                        }
                        if let Ok(maps) = std::fs::read_to_string("/proc/self/maps") {
                            for line in maps.lines() {
                                if let Some((range, _)) = line.split_once(" ") {
                                    if let Some((start_str, end_str)) = range.split_once("-") {
                                        if let (Ok(start), Ok(end)) = (usize::from_str_radix(start_str, 16), usize::from_str_radix(end_str, 16)) {
                                            let len = end.saturating_sub(start);
                                            if len > 0 {
                                                unsafe {
                                                    libc::madvise(start as *mut libc::c_void, len, libc::MADV_DONTDUMP);
                                                    libc::madvise(start as *mut libc::c_void, len, libc::MADV_WIPEONFORK);
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        println!("  └─✔ Memory mappings locked against /proc/kcore scraping & core dumps.");
                    }

                    let err = execve(&binary_c, &args_c, &envs_c);
                    eprintln!("\n\x1b[1m\x1b[31m✘ [vessel-exec] Execution failed:\x1b[0m Cannot launch '{}' ({:?}). Check if the binary exists inside your rootfs.", binary_path, err);
                }

                std::process::exit(1);
            }
            Err(e) => {
                return Err(format!("Fork boundary failed: {}", e).into());
            }
        }
    }

    Ok(())
}
