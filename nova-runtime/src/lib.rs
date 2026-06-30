use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::collections::HashMap;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum SandboxType {
    OsNative,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub memory_bytes: u64,
    pub cpu_cores: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct WorkloadSpec {
    pub id: String,
    pub sandbox_type: SandboxType,
    pub rootfs_path: PathBuf,
    pub exec_args: Vec<String>,
    pub limits: ResourceLimits,
    pub env_vars: HashMap<String, String>,
    pub port_mappings: Vec<String>,
    pub volume_mounts: Vec<String>,
}

pub mod drivers {
    pub mod os_native {
        use crate::WorkloadSpec;
        use nix::sched::{unshare, CloneFlags};
        use nix::mount::{mount, MsFlags};
        use nix::unistd::{pivot_root, chdir, execve, fork, ForkResult};
        use std::ffi::CString;
        use std::fs;
        use std::path::Path;
        use std::collections::HashMap; // <-- Added to solve submodule scope error

        pub struct OsNativeSandbox {
            spec: WorkloadSpec,
            pid: Option<u32>,
        }

        impl OsNativeSandbox {
            pub fn new(spec: WorkloadSpec) -> Self {
                Self { spec, pid: None }
            }

            pub async fn start(&mut self) -> Result<(), String> {
                let rootfs = self.spec.rootfs_path.clone();
                let exec_args = self.spec.exec_args.clone();
                let env_vars = self.spec.env_vars.clone();

                println!("[nova-runtime] Forking process space for sandbox isolation...");

                match unsafe { fork() } {
                    Ok(ForkResult::Parent { child }) => {
                        self.pid = Some(child.as_raw() as u32);
                        Ok(())
                    }
                    Ok(ForkResult::Child) => {
                        if let Err(e) = self.isolate_and_exec(&rootfs, &exec_args, &env_vars) {
                            eprintln!("[nova-runtime-child] Sandbox execution panic: {}", e);
                            std::process::exit(1);
                        }
                        std::process::exit(0);
                    }
                    Err(e) => Err(format!("Fork system call failed: {}", e)),
                }
            }

            fn isolate_and_exec(
                &self,
                rootfs: &Path,
                exec_args: &[String],
                env_vars: &HashMap<String, String>,
            ) -> Result<(), String> {
                // 1. Unshare Namespaces (PID, Mount, Net, IPC, UTS)
                unshare(
                    CloneFlags::CLONE_NEWPID
                        | CloneFlags::CLONE_NEWNS
                        | CloneFlags::CLONE_NEWNET
                        | CloneFlags::CLONE_NEWIPC
                        | CloneFlags::CLONE_NEWUTS,
                )
                .map_err(|e| format!("Failed to unshare namespaces: {}", e))?;

                // 2. Bind-mount the rootfs directory onto itself (pivot_root requirement)
                mount(
                    Some(rootfs),
                    rootfs,
                    None::<&str>,
                    MsFlags::MS_BIND | MsFlags::MS_REC,
                    None::<&str>,
                )
                .map_err(|e| format!("Failed to self bind-mount rootfs: {}", e))?;

                // 3. Create the old-root placeholder directory
                let old_root_dir = rootfs.join("old_root");
                if !old_root_dir.exists() {
                    fs::create_dir(&old_root_dir)
                        .map_err(|e| format!("Failed to create old_root directory: {}", e))?;
                }

                // 4. Swap filesystems using pivot_root
                pivot_root(rootfs, &old_root_dir)
                    .map_err(|e| format!("pivot_root system call failed: {}", e))?;

                // 5. Change active directory context to "/"
                chdir("/")
                    .map_err(|e| format!("Failed to chdir to new root /: {}", e))?;

                // 6. Mount the isolated /proc filesystem
                let proc_path = Path::new("/proc");
                if !proc_path.exists() {
                    let _ = fs::create_dir(proc_path);
                }
                mount(
                    Some("proc"),
                    proc_path,
                    Some("proc"),
                    MsFlags::empty(),
                    None::<&str>,
                )
                .map_err(|e| format!("Failed to mount isolated /proc filesystem: {}", e))?;

                // 7. Compile C-compatible execution parameters
                let binary_c = CString::new(exec_args[0].as_str())
                    .map_err(|e| format!("Invalid binary path: {}", e))?;
                
                let args_c: Vec<CString> = exec_args
                    .iter()
                    .map(|s| CString::new(s.as_str()).unwrap())
                    .collect();

                let envs_c: Vec<CString> = env_vars
                    .iter()
                    .map(|(k, v)| CString::new(format!("{}={}", k, v)).unwrap())
                    .collect();

                // 8. Hand-off execution to target binary
                execve(&binary_c, &args_c, &envs_c)
                    .map_err(|e| format!("execve system call failed: {}", e))?;

                Ok(())
            }

            pub async fn get_pid(&self) -> Option<u32> {
                self.pid
            }
        }
    }
}
