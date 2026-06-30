#![allow(dead_code, unused_imports, unused_variables, unused_mut)]

pub mod termux;

use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::collections::HashMap;
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
    sadasya: Option<String>,
    sanket: Vec<String>,
    sangjna: Option<String>,
    suraksha: Option<String>,
    tejas: Option<String>,
    kavach: Option<String>,
    kala: Option<String>,
    vayu: Vec<String>,
    kendra: Option<String>,
    gati: Option<String>,
    bhaar: Option<String>,
    adhikar: Option<String>,
    sthapana: Option<String>,
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

    if let Ok(term_val) = std::env::var("TERM") {
        paryavaran.insert("TERM".to_string(), term_val);
    } else {
        paryavaran.insert("TERM".to_string(), "xterm-256color".to_string());
    }

    let mool = if lower_prompt.contains("ubuntu") {
        format!("{}/bases/ubuntu-rootfs", crate::termux::get_base_dir())
    } else if lower_prompt.contains("alpine") {
        format!("{}/bases/alpine-rootfs", crate::termux::get_base_dir())
    } else {
        format!("{}/bases/alpine-rootfs", crate::termux::get_base_dir())
    };

    let mut karya = vec!["/bin/sh".to_string()]; 
    if lower_prompt.contains("shell") || lower_prompt.contains("bash") {
        if mool.contains("ubuntu") {
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
        sangjna: Some("god-mode-node".to_string()),
        suraksha: None,
        tejas: None,
        kavach: None,
        kala: None,
        vayu: Vec::new(),
        kendra: None,
        gati: None,
        bhaar: None,
        adhikar: None,
        sthapana: None,
    })
}

async fn run_sanchay_compile(target_dir: &str) -> Result<(u64, u64), Box<dyn std::error::Error>> {
    let registry_path = PathBuf::from(format!("{}/nova-registry/chunks", crate::termux::get_tmp_dir()));
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
        let mut f = File::create(Path::new(&thread_target_dir).join("nova-catalog.json")).unwrap();
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
        "ubuntu" => format!("{}/bases/ubuntu-rootfs", crate::termux::get_base_dir()),
        "alpine" => format!("{}/bases/alpine-rootfs", crate::termux::get_base_dir()),
        _ => mool,
    };

    let mut karya_input = String::new();
    print!("  \x1b[1mTask Execution (Karya) [default: /bin/bash]:\x1b[0m ");
    let _ = stdout().flush();
    let _ = stdin().read_line(&mut karya_input);
    let mut karya = karya_input.trim().to_string();
    if karya.is_empty() { karya = "/bin/bash".to_string(); }

    let mut smriti_input = String::new();
    print!("  \x1b[1mMemory Limit (Smriti) [default: 2GB]:\x1b[0m ");
    let _ = stdout().flush();
    let _ = stdin().read_line(&mut smriti_input);
    let mut smriti_str = smriti_input.trim().to_string();
    if smriti_str.is_empty() { smriti_str = "2GB".to_string(); }
    let smriti = parse_size_to_bytes(&smriti_str);

    let mut shakti_input = String::new();
    print!("  \x1b[1mCPU Core Allocation (Shakti) [default: 2.0]:\x1b[0m ");
    let _ = stdout().flush();
    let _ = stdin().read_line(&mut shakti_input);
    let mut shakti_str = shakti_input.trim().to_string();
    if shakti_str.is_empty() { shakti_str = "2.0".to_string(); }
    let shakti = shakti_str.parse::<f32>().unwrap_or(2.0);

    let mut suraksha_input = String::new();
    print!("  \x1b[1mSecurity Policy (Suraksha) [default: ephemeral]:\x1b[0m ");
    let _ = stdout().flush();
    let _ = stdin().read_line(&mut suraksha_input);
    let mut suraksha_str = suraksha_input.trim().to_string();
    if suraksha_str.is_empty() { suraksha_str = "ephemeral".to_string(); }

    let mut paryavaran = HashMap::new();
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
        sanket: vec!["1.1.1.1".to_string(), "8.8.8.8".to_string()],
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
        sthapana: Some("sudo curl wget nano htop git neofetch".to_string()),
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
        sthapana,
    })
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        println!("                            - \x1b[90mWith prompt: compiles the natural language intent on the fly\x1b[0m\n");

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

        let alpine_installed = Path::new(&format!("{}/bases/alpine-rootfs", crate::termux::get_base_dir())).exists();
        let ubuntu_installed = Path::new(&format!("{}/bases/ubuntu-rootfs", crate::termux::get_base_dir())).exists();

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
                            let is_installed = Path::new(&format!("{}/bases/{}-rootfs", crate::termux::get_base_dir(), name)).exists();

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
            let my_ubuntu_installed = Path::new(&format!("{}/bases/my-ubuntu-node-rootfs", crate::termux::get_base_dir())).exists();
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
        if !crate::termux::is_termux() && !nix::unistd::geteuid().is_root() {
            println!("\x1b[1m\x1b[31m[vessel rm] Error: Root privileges required. Please run this command as: sudo vessel rm {}\x1b[0m", args[2]);
            std::process::exit(1);
        }

        let distro = args[2].to_lowercase();
        let rootfs_dir = if distro.contains('/') {
            let parts: Vec<&str> = distro.split('/').collect();
            let repo = parts[1];
            format!("{}/bases/{}-rootfs", crate::termux::get_base_dir(), repo)
        } else {
            format!("{}/bases/{}-rootfs", crate::termux::get_base_dir(), distro)
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

    if args.len() == 3 && (args[1] == "prapt" || args[1] == "pull") {
        if !crate::termux::is_termux() && !nix::unistd::geteuid().is_root() {
            println!("\x1b[1m\x1b[31m[vessel prapt] Error: Root privileges required. Please run this command as: sudo vessel prapt {}\x1b[0m", args[2]);
            std::process::exit(1);
        }

        let distro = args[2].to_lowercase();
        
        let (url, rootfs_dir) = if distro.contains('/') {
            let parts: Vec<&str> = distro.split('/').collect();
            let username = parts[0];
            let repo = parts[1];
            let download_url = format!("https://github.com/{}/{}/releases/latest/download/rootfs.tar.gz", username, repo);
            let target_dir = format!("{}/bases/{}-rootfs", crate::termux::get_base_dir(), repo);
            (download_url, target_dir)
        } else {
            match distro.as_str() {
                "alpine" => (
                    "https://dl-cdn.alpinelinux.org/alpine/v3.19/releases/x86_64/alpine-minirootfs-3.19.1-x86_64.tar.gz".to_string(),
                    format!("{}/bases/alpine-rootfs", crate::termux::get_base_dir())
                ),
                "ubuntu" => (
                    "https://partner-images.canonical.com/core/jammy/current/ubuntu-jammy-core-cloudimg-amd64-root.tar.gz".to_string(),
                    format!("{}/bases/ubuntu-rootfs", crate::termux::get_base_dir())
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

        let temp_tar = format!("{}/nova-download.tar.gz", crate::termux::get_tmp_dir());
        fs::create_dir_all(&rootfs_dir)?;

        show_spinner("  [1/4] Downloading minimal OS base over network...", 1200).await;
        let _ = Command::new("curl").args(["-L", "-o", &temp_tar, &url]).status()?;

        show_spinner("  [2/4] Unpacking base files into secure system vault...", 800).await;
        let _ = Command::new("tar").args(["-xpf", &temp_tar, "-C", &rootfs_dir]).status()?;
        let _ = fs::remove_file(&temp_tar);

        show_spinner("  [3/4] Resetting directory system permissions...", 400).await;
        if !crate::termux::is_termux() {
            let _ = Command::new("chown").args(["-R", "root:root", &rootfs_dir]).status()?;
        }

        show_spinner("  [4/4] Compiling image structure to deduplicated NCI registry...", 1500).await;
        let (total_chunks, saved_bytes) = run_sanchay_compile(&rootfs_dir).await?;

        println!("\x1b[1m\x1b[32m✔ Retrived Successfully!\x1b[0m Built {} chunks inside registry. Saved {} bytes.", total_chunks, saved_bytes);
        println!("Installed securely at path: {}", rootfs_dir);

        let patra_path = Path::new("Patra");
        if !patra_path.exists() {
            // ADVANCED GOD-MODE PATRA GENERATION
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

# Pre-Install essential hacking/dev tools automatically
Sthapana: \"sudo curl wget nano htop neofetch git build-essential\"

Sadasya: sir                   # Run as unprivileged user 'sir' with sudo access
Sanket: 1.1.1.1 8.8.4.4
Sangjna: god-mode-node         # Elegant Hostname
Suraksha: ephemeral              # Enabled writeable copy-on-write overlay

# Advanced Host Integrations (Uncomment to use)
# Dwar: \"8080:80 3000:3000\"      # Port Forwarding
# Sanchay: \"/data/data/com.termux/files/home:/host_home\" # Map Termux home to sandbox
# Gati: \"100mbit\"                # Bandwidth Throttling
# Kendra: \"0,1\"                  # CPU Core Pinning
", rootfs_dir);
            if let Ok(mut f) = File::create(patra_path) {
                let _ = f.write_all(default_patra.as_bytes());
                println!("\n\x1b[1m\x1b[36m[vessel prapt]\x1b[0m An advanced 'Patra' manifest file was auto-generated in your current directory.");
                println!("You can safely run \x1b[1m`sudo vessel`\x1b[0m (or just `vessel` in Termux) right now to enter your new sandbox environment!\n");
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

    if args.len() == 1 || (args.len() >= 2 && (args[1] == "direct" || args[1] == "local" || args[1] == "run")) {
        
        if !crate::termux::is_termux() && !nix::unistd::geteuid().is_root() {
            println!("\x1b[1m\x1b[31m[vessel] Error: Root privileges required to construct sandbox namespaces.\x1b[0m");
            println!("        Please execute this command as: \x1b[1msudo vessel\x1b[0m\n");
            std::process::exit(1);
        }

        let config = if args.len() >= 3 {
            let prompt = args[2..].join(" ");
            println!("\x1b[1m\x1b[36m[vessel direct]\x1b[0m Initiating native, zero-latency local terminal hand-off...\n");
            
            show_spinner("  [1/4] Compiling semantic intent via local NCE...", 600).await;
            compile_natural_language_intent(&prompt)?
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
        println!("    ├── Mool (Base Environment): \x1b[32m{}\x1b[0m", config.mool);
        println!("    ├── Karya (Task Execution):  \x1b[33m{:?}\x1b[0m", config.karya.join(" "));
        println!("    ├── Seema (Resource Limits):");
        println!("        ├── Smriti (Memory):     \x1b[35m{} MB\x1b[0m", config.smriti / (1024 * 1024));
        println!("        └── Shakti (CPU):        \x1b[35m{} Cores\x1b[0m", config.shakti);
        
        if let Some(ref sthapana) = config.sthapana {
            println!("    ├── Sthapana (Pre-Install):  \x1b[36m{}\x1b[0m", sthapana);
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

        // =========================================================================
        // FILE INJECTIONS (Must happen for BOTH PRoot Termux and Native Linux)
        // Resolves .bashrc coloring, aliases, user accounts and Sudo access.
        // =========================================================================
        let sandbox_id = format!("vessel-{}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
        let mut rootfs_path = PathBuf::from(&config.mool);

        // 1. Inject God-Mode Colors & Aliases
        let bashrc_path = rootfs_path.join("etc/bash.bashrc");
        let _ = fs::create_dir_all(rootfs_path.join("etc"));
        if let Ok(content) = fs::read_to_string(&bashrc_path).or_else(|_| Ok::<String, ()>("".to_string())) {
            if !content.contains("god-mode") {
                if let Ok(mut file) = fs::OpenOptions::new().create(true).append(true).open(&bashrc_path) {
                    let wrapper_code = "
# [god-mode] Terminal Colors & Advanced Aliases
export TERM=xterm-256color
export COLORTERM=truecolor
export FORCE_COLOR=1
alias ls='ls --color=auto'
alias ll='ls -la --color=auto'
alias grep='grep --color=auto'
alias ip='ip -color=auto'
PS1='\\[\\033[01;32m\\]\\u@\\h\\[\\033[00m\\]:\\[\\033[01;34m\\]\\w\\[\\033[00m\\]\\$ '

# [vessel-security] Automatic Package Manager Warning Wrapper
apt() {
    if [ \"$EUID\" -ne 0 ]; then
        echo -e \"\\x1b[1m\\x1b[31m[vessel-security]\\x1b[0m Permission Denied: Please run using 'sudo apt'.\"
        return 1
    fi
    command apt \"$@\"
}
apt-get() {
    if [ \"$EUID\" -ne 0 ]; then
        echo -e \"\\x1b[1m\\x1b[31m[vessel-security]\\x1b[0m Permission Denied: Please run using 'sudo apt-get'.\"
        return 1
    fi
    command apt-get \"$@\"
}
";
                    let _ = writeln!(file, "{}", wrapper_code);
                }
            }
        }

        // 2. Resolve Users and setup Sudoers cleanly
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
                if !crate::termux::is_termux() {
                    let _ = Command::new("chown").args(["-R", "1000:1000", user_home.to_str().unwrap()]).status();
                }
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

            // Sudoers specific injection for the unprivileged user
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
        // =========================================================================

        // -------------------------------------------------------------
        // TERMUX SMART ROUTING
        // Intercepts flow if unprivileged user-space emulation is required
        // -------------------------------------------------------------
        if crate::termux::is_termux() {
            crate::termux::run_direct(config).await?;
            return Ok(());
        }

        show_spinner("  [2/4] Resolving virtual storage layers & device mounts...", 800).await;

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

                let _ = Command::new("ip").args(["link", "add", &host_veth, "type", "veth", "peer", "name", &guest_veth]).status();
                let _ = Command::new("ip").args(["link", "set", &guest_veth, "netns", &pid_str]).status();
                let _ = Command::new("ip").args(["addr", "add", "10.0.0.1/24", "dev", &host_veth]).status();
                let _ = Command::new("ip").args(["link", "set", &host_veth, "up"]).status();
                let _ = Command::new("sysctl").args(["-w", "net.ipv4.ip_forward=1"]).status();
                
                let _ = Command::new("iptables").args(["-A", "FORWARD", "-i", &host_veth, "-j", "ACCEPT"]).status();
                let _ = Command::new("iptables").args(["-A", "FORWARD", "-o", &host_veth, "-j", "ACCEPT"]).status();
                let _ = Command::new("iptables").args(["-t", "nat", "-A", "POSTROUTING", "-s", "10.0.0.0/24", "-j", "MASQUERADE"]).status();

                for port_spec in &config.dwar {
                    if let Some((host_port, guest_port)) = port_spec.split_once(':') {
                        let _ = Command::new("iptables").args([
                            "-t", "nat", "-A", "PREROUTING", 
                            "-p", "tcp", "--dport", host_port, 
                            "-j", "DNAT", "--to-destination", &format!("10.0.0.2:{}", guest_port)
                        ]).status();

                        let _ = Command::new("iptables").args([
                            "-t", "nat", "-A", "OUTPUT", 
                            "-p", "tcp", "-o", "lo", "--dport", host_port, 
                            "-j", "DNAT", "--to-destination", &format!("10.0.0.2:{}", guest_port)
                        ]).status();
                    }
                }

                if let Some(ref speed) = config.gati {
                    let _ = Command::new("tc").args([
                        "qdisc", "add", "dev", &host_veth, "root", 
                        "handle", "1:", "htb", "default", "11"
                    ]).status();
                    
                    let _ = Command::new("tc").args([
                        "class", "add", "dev", &host_veth, "parent", "1:", 
                        "classid", "1:11", "htb", "rate", speed
                    ]).status();

                    let _ = Command::new("tc").args([
                        "filter", "add", "dev", &host_veth, "parent", "1:", 
                        "protocol", "ip", "prio", "1", "u32", 
                        "match", "ip", "dst", "0.0.0.0/0", "flowid", "1:11"
                    ]).status();
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

                let mut status = 0;
                unsafe { libc::waitpid(pid as i32, &mut status, 0); }

                for port_spec in &config.dwar {
                    if let Some((host_port, guest_port)) = port_spec.split_once(':') {
                        let _ = Command::new("iptables").args([
                            "-t", "nat", "-D", "PREROUTING", 
                            "-p", "tcp", "--dport", host_port, 
                            "-j", "DNAT", "--to-destination", &format!("10.0.0.2:{}", guest_port)
                        ]).status();

                        let _ = Command::new("iptables").args([
                            "-t", "nat", "-D", "OUTPUT", 
                            "-p", "tcp", "-o", "lo", "--dport", host_port, 
                            "-j", "DNAT", "--to-destination", &format!("10.0.0.2:{}", guest_port)
                        ]).status();
                    }
                }

                let _ = Command::new("iptables").args(["-D", "FORWARD", "-i", &host_veth, "-j", "ACCEPT"]).status();
                let _ = Command::new("iptables").args(["-D", "FORWARD", "-o", &host_veth, "-j", "ACCEPT"]).status();
                let _ = Command::new("ip").args(["link", "delete", &host_veth]).stderr(Stdio::null()).status();
                let _ = Command::new("iptables").args(["-t", "nat", "-D", "POSTROUTING", "-s", "10.0.0.0/24", "-j", "MASQUERADE"]).status();

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
                if let Ok(mut f) = fs::File::create(&resolv_path) {
                    if !config.sanket.is_empty() {
                        for ns in &config.sanket {
                            let _ = writeln!(f, "nameserver {}", ns);
                        }
                    } else {
                        let _ = f.write_all(b"nameserver 1.1.1.1\nnameserver 8.8.8.8\n");
                    }
                }

                let binary_path = &config.karya[0];
                let exec_args = &config.karya;

                let binary_c = CString::new(binary_path.as_str()).unwrap();
                let args_c: Vec<CString> = exec_args.iter().map(|s| CString::new(s.as_str()).unwrap()).collect();
                
                let mut envs = config.paryavaran.clone();
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

                    let _ = Command::new("ip").args(["link", "set", "lo", "up"]).status();
                    let _ = Command::new("ip").args(["addr", "add", "10.0.0.2/24", "dev", &guest_veth]).status();
                    let _ = Command::new("ip").args(["link", "set", &guest_veth, "up"]).status();
                    let _ = Command::new("ip").args(["route", "add", "default", "via", "10.0.0.1"]).status();

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

                    // --- STHAPANA: Child-side native install loop ---
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
                                        print!("\r\x1b[2K"); // Erase spinner line cleanly on install complete
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

                    let _ = execve(&binary_c, &args_c, &envs_c);
                }

                std::process::exit(0);
            }
            Err(e) => {
                return Err(format!("Fork boundary failed: {}", e).into());
            }
        }
    }

    Ok(())
}
