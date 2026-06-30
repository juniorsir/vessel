use crate::{Sandbox, SandboxError, WorkloadSpec};
use async_trait::async_trait;

pub struct FirecrackerSandbox {
    spec: WorkloadSpec,
    vmm_pid: Option<u32>,
}

impl FirecrackerSandbox {
    pub fn new(spec: WorkloadSpec) -> Self {
        Self { spec, vmm_pid: None }
    }
}

#[async_trait]
impl Sandbox for FirecrackerSandbox {
    async fn construct(&mut self) -> Result<(), SandboxError> {
        // 1. Prepare disk drives into block files
        // 2. Generate Firecracker configuration payload (JSON)
        // 3. Bind virtual TAP device for guest networking
        println!("[Firecracker VM] Instantiating Firecracker VMM context for {}", self.spec.id);
        Ok(())
    }

    async fn start(&mut self) -> Result<(), SandboxError> {
        // Spawn the Firecracker VMM process and configure it via the socket API
        println!("[Firecracker VM] Booting guest kernel with args: {:?}", self.spec.exec_args);
        self.vmm_pid = Some(2048); // Simulated PID of the Firecracker process
        Ok(())
    }

    async fn pause(&mut self) -> Result<(), SandboxError> {
        println!("[Firecracker VM] Suspending virtual machine state execution...");
        Ok(())
    }

    async fn resume(&mut self) -> Result<(), SandboxError> {
        println!("[Firecracker VM] Resuming virtual machine CPU execution.");
        Ok(())
    }

    async fn destroy(&mut self) -> Result<(), SandboxError> {
        println!("[Firecracker VM] Shutting down microVM and reclaiming block files.");
        Ok(())
    }

    async fn get_pid(&self) -> Option<u32> {
        self.vmm_pid
    }
}
