# Vessel

Vessel is a Linux-first sandbox runtime written in Rust. It launches isolated workloads from a `Patra` manifest, supports image pull/build workflows, and includes security-oriented runtime modes (Tor routing, MAC spoofing, seccomp/capability controls).

## Current status

- Primary implementation: `nova-cli` (`/home/runner/work/vessel/vessel/nova-cli/src/main.rs`)
- Workspace members: `nova-cli`, `nova-core`, `nova-runtime`
- Platform support in current code: **Linux**
- Termux/Android support is currently not enabled in runtime or installer (both print “not yet supported”).

## Main capabilities

- Patra manifest parsing and sandbox launch (`vessel`, `vessel direct`, `vessel local`)
- Image pull/remove (`vessel prapt|pull`, `vessel nishkaas|rm|remove|delete`)
- Local filesystem chunk build (`vessel sanchay|build <dir>`)
- Quick operational commands (`vessel tor`, `vessel test`, `vessel clean`)
- AI-style helper flows in CLI:
  - Manifest generation (`vessel generate|srijan`)
  - Diagnostics (`vessel doctor|vaidya`)
  - Manifest risk audit (`vessel audit|drishti`)

## Installation

### Option 1: Install released binary

```bash
wget https://github.com/juniorsir/vessel/releases/latest/download/vessel-linux-amd64
chmod +x vessel-linux-amd64
sudo mv vessel-linux-amd64 /usr/local/bin/vessel
```

### Option 2: Build from source

```bash
cd /home/runner/work/vessel/vessel
cargo build --release --bin nova-cli
sudo cp target/release/nova-cli /usr/local/bin/vessel
```

## Quick start

1. Pull a base image:

```bash
sudo vessel prapt ubuntu
```

2. Start from local `Patra` file in current directory:

```bash
sudo vessel
```

3. Or run with natural-language prompt:

```bash
sudo vessel direct "run htop"
```

## Patra example

```yaml
Mool: "/var/lib/vessel/bases/ubuntu-rootfs"
Karya: "/bin/bash"
Smriti: "1GB"
Shakti: 1.0
Sangjna: "vessel-node"
Sadasya: "sir"
Suraksha: "ephemeral"
Kavach: "strict"
Adhikar: "-SYS_BOOT -SYS_TIME"
Sanket:
  - "1.1.1.1"
  - "8.8.8.8"
```

## CLI command reference

| Command | Aliases | Purpose |
|---|---|---|
| `vessel help` | `--help`, `-h`, `sahayata` | Show help |
| `vessel suchi` | `list` | List available/installed images |
| `vessel prapt <image>` | `pull <image>` | Pull base/custom rootfs |
| `vessel nishkaas <image>` | `rm/remove/delete <image>` | Remove installed rootfs |
| `vessel direct [prompt...]` | `local`, `run` | Launch from Patra or prompt |
| `vessel sanchay <dir>` | `build <dir>` | Build chunk catalog from directory |
| `vessel generate <intent...>` | `srijan` | Generate Patra from intent text |
| `vessel doctor [log]` | `vaidya` | Diagnose runtime/log issues |
| `vessel audit [Patra]` | `drishti` | Audit manifest security posture |
| `vessel tor` | - | Launch Tor-oriented preset |
| `vessel test` | - | Launch diagnostic test preset |
| `vessel clean` | - | Flush temporary networking/cache artifacts |

## Repository layout

- `/home/runner/work/vessel/vessel/nova-cli` — main CLI/runtime behavior
- `/home/runner/work/vessel/vessel/nova-core` — core config + engine scaffolding
- `/home/runner/work/vessel/vessel/nova-runtime` — runtime abstractions
- `/home/runner/work/vessel/vessel/builder` — chunking/compression/encryption utilities
- `/home/runner/work/vessel/vessel/sdk` — multi-language client wrappers (Rust, Go, Python, JS, C#, Java)

## Notes

- Most runtime commands require root privileges (`sudo`).
- The CLI reads `Patra` (capital P) from the working directory for default launches.
