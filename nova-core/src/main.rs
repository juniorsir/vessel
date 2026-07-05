use nova_core::config::Patra;
use nova_core::engine::Engine;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    
    let config_path = if args.len() > 1 {
        &args[1]
    } else {
        "patra.yaml"
    };

    println!("[Nova Core] Loading configuration from '{}'...", config_path);
    let patra = match Patra::load(config_path) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("[Error] {}", e);
            std::process::exit(1);
        }
    };

    let engine = Engine::new(patra);
    if let Err(e) = engine.launch() {
        eprintln!("[Fatal Error] {}", e);
        std::process::exit(1);
    }
}
