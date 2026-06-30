pub mod engine {
    use nova_runtime::WorkloadSpec;

    pub struct Engine;

    impl Engine {
        pub fn new() -> Self {
            Self
        }

        pub fn process_workload(&self, spec: &WorkloadSpec) {
            println!("[nova-core] Coordinating sandbox allocation for: {}", spec.id);
        }
    }
}
