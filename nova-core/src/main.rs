use tokio::net::TcpListener;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("\x1b[1m\x1b[36m[novad]\x1b[0m Booting Systems Orchestrator...");

    // The standard TCP Sandbox Listener Thread
    let exec_listener = TcpListener::bind("127.0.0.1:9000").await?;
    println!("\x1b[1m\x1b[32m[novad-exec]\x1b[0m TCP Execution Engine listening on 127.0.0.1:9000");

    loop {
        let (mut socket, addr) = exec_listener.accept().await?;
        println!("[novad-exec] Connection received from: {}", addr);
        
        tokio::spawn(async move {
            let mut buffer = [0u8; 4096];
            let _ = socket.read(&mut buffer).await;
            let _ = socket.write_all(b"Connected to novad execution engine cleanly!\n").await;
        });
    }
}
