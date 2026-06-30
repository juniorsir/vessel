use crate::{
    drivers::{os_native::OsNativeSandbox, firecracker::FirecrackerSandbox, wasm::WasmSandbox},
    Sandbox, SandboxError, SandboxType, WorkloadSpec, RuntimePlugin, HookType
};

pub struct SandboxManager {
    plugins: Vec<Arc<dyn RuntimePlugin>>,
}

impl SandboxManager {
    pub fn new() -> Self {
        Self { plugins: Vec::new() }
    }

    pub fn register_plugin(&mut self, plugin: Arc<dyn RuntimePlugin>) {
        self.plugins.push(plugin);
    }

    async fn run_hooks(&self, hook: HookType, spec: &WorkloadSpec) -> Result<(), SandboxError> {
        for plugin in &self.plugins {
            println!("Calling plugin '{}' hook...", plugin.name());
            plugin.call_hook(hook.clone(), spec).await?;
        }
        Ok(())
    }

    pub async fn create_sandbox(&self, spec: WorkloadSpec) -> Result<Box<dyn Sandbox>, SandboxError> {
        // 1. Execute PreCreate plugins (e.g., configuring networks or generating dynamic credentials)
        self.run_hooks(HookType::PreCreate, &spec).await?;

        // 2. Select and build the requested sandbox isolation driver
        let mut sandbox: Box<dyn Sandbox> = match spec.sandbox_type {
            SandboxType::OsNative | SandboxType::GVisor => {
                Box::new(OsNativeSandbox::new(spec.clone()))
            }
            SandboxType::MicroVM => {
                Box::new(FirecrackerSandbox::new(spec.clone()))
            }
            SandboxType::Wasm => {
                Box::new(WasmSandbox::new(spec.clone()))
            }
        };

        // 3. Setup isolation boundaries
        sandbox.construct().await?;

        // 4. Run PostStart hooks once the container starts running
        self.run_hooks(HookType::PostStart, &spec).await?;

        Ok(sandbox)
    }
}
