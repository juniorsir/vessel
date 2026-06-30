use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct NetworkEndpoint {
    pub ip_address: String,
    pub gateway: String,
    pub interface_name: String,
    pub vlan_id: Option<u16>,
}

#[async_trait]
pub trait NetworkEngine: Send + Sync {
    /// Provisions network links, routing policies, and eBPF bypass points.
    async fn create_endpoint(&self, container_id: &str) -> Result<NetworkEndpoint, String>;
    
    /// Tears down networks and cleans up allocated interfaces and eBPF maps.
    async fn destroy_endpoint(&self, container_id: &str) -> Result<(), String>;
}
