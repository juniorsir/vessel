Here is the complete `README.md` updated with a sleek, professional **Developer Card** near the bottom. 

Copy and paste this block into your terminal to overwrite and update your `README.md`:


<div align="center">

# 🚢 Vessel (NEXUS Engine)
**High-Performance, Zero-Latency Local Containerization & Virtualization Engine**

[![NPM Version](https://img.shields.io/npm/v/@juniorsir/vessel.svg?style=for-the-badge&color=cb3837)](https://www.npmjs.com/package/@juniorsir/vessel)
[![OS Support](https://img.shields.io/badge/OS-Linux%20%7C%20Termux%20(Android)-blue?style=for-the-badge)](#)
[![Built in Rust](https://img.shields.io/badge/Built_in-Rust-dea584?style=for-the-badge&logo=rust)](#)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=for-the-badge)](#)

</div>

**Vessel** is a next-generation sandbox engine built in Rust. It bypasses the heavy daemons of traditional Docker/Podman to offer **instant, zero-latency local terminal hand-offs**. 

By utilizing low-level Linux Namespaces, Cgroups v2, eBPF, and FastCDC BLAKE3 block deduplication natively, it achieves unparalleled performance. **For Android/Termux users**, Vessel features a smart-routing backend that seamlessly switches to `PRoot` unprivileged user-space emulation, giving you a full Linux environment directly on your phone.

---

## ⚡ Features

- **Zero-Latency Boot**: Spawns environments instantly directly into your current terminal tree.
- **NCI Deduplication**: Uses Content-Defined Chunking (FastCDC) to save up to 80% of disk space across different Linux distributions.
- **Advanced God-Mode**: Auto-injects true-color terminal support, custom `.bashrc` aliases, and seamless `sudo` wrappers into the guest OS.
- **Hardware Passthrough**: Direct GPU (Nvidia/DRI) mapping into the container.
- **Smart Termux Support**: Automatically detects Android environments and adapts volume mounts and networking to bypass Android's read-only restrictions without needing root.

---

## 📦 Installation

Vessel is distributed via NPM but downloads native, highly optimized Rust binaries for your specific architecture (x64 / arm64) during installation.

```bash
npm install -g @juniorsir/vessel
```
*Note: On Linux, ensure you have basic build tools available. On Termux, ensure `proot` is installed (`pkg install proot`).*

---

## 🚀 Quick Start

1. **Pull an official base image** (Requires `sudo` on Linux, run normally on Termux):
   ```bash
   sudo vessel prapt ubuntu
   ```
2. **Launch the Sandbox directly** (This automatically reads the generated `Patra` manifest):
   ```bash
   sudo vessel direct
   ```
3. **List installed images**:
   ```bash
   vessel suchi
   ```

---

## 📜 The `Patra` (पत्र) Manifest

The heart of Vessel is the **Patra Configuration Card**. Instead of writing complex `docker run` commands or multi-stage `Dockerfiles`, Vessel uses a declarative, highly-readable manifest placed in your active directory. 

When you type `vessel direct`, Vessel reads the `Patra` file and constructs the exact kernel boundaries, limits, and network topologies requested.

### 🛠️ Advanced `Patra` Example

Create a file named `Patra` in your working directory:

```yaml
# ---------------------------------------------------------
# CORE ENVIRONMENT
# ---------------------------------------------------------
Mool: "/var/lib/vessel/bases/ubuntu-rootfs"     # Base Image Path
Karya: "/bin/bash"                              # Entrypoint Execution
Sthapana: "sudo curl wget nano htop python3"    # Pre-boot package install injection

# ---------------------------------------------------------
# SYSTEM LIMITS (Cgroups v2)
# ---------------------------------------------------------
Smriti: 2GB                                     # Hard RAM Limit (OOM killer enabled)
Shakti: 2.0                                     # Max CPU cores allowed
Kendra: "0,1"                                   # CPU Core Pinning (Pin to core 0 and 1)
Bhaar: "50MB"                                   # Disk I/O Throttling (Read/Write max)
Gati: "100mbit"                                 # Network Bandwidth egress throttling

# ---------------------------------------------------------
# NETWORKING & CONTEXT
# ---------------------------------------------------------
Sangjna: "god-mode-node"                        # Container Hostname
Sanket: 1.1.1.1 8.8.8.8                         # Custom DNS Resolvers
Dwar: "8080:80 3000:3000"                       # Host -> Guest Port Forwarding
Sadasya: "sir"                                  # Boot into this unprivileged user (with auto sudo)

# ---------------------------------------------------------
# VOLUMES & HARDWARE
# ---------------------------------------------------------
Sanchay: "/home/user/code:/workspace"           # Bind mount host directories
Vayu: "/tmp/ramdisk:500m"                       # In-memory tmpfs creation
Tejas: "nvidia"                                 # Hardware Passthrough (Nvidia GPU, DRI)

# ---------------------------------------------------------
# SECURITY BOUNDARIES
# ---------------------------------------------------------
Suraksha: "ephemeral"                           # 'ephemeral' (CoW overlay) or 'read-only'
Kavach: "strict"                                # Enforce Seccomp-BPF & Drop CAP_SYS_BOOT
Adhikar: "-SYS_ADMIN -SYS_TIME"                 # Granularly drop specific Linux Capabilities
Kala: "America/New_York"                        # Isolated Timezone illusion

# ---------------------------------------------------------
# INJECTED ENVIRONMENT VARIABLES
# ---------------------------------------------------------
Paryavaran:
  - TERM: "xterm-256color"
  - NODE_ENV: "development"
  - CUSTOM_KEY: "secret_value"
```

### 📖 `Patra` Field Dictionary

| Field | Meaning / Translation | Functionality | Supported OS |
|-------|-----------------------|---------------|--------------|
| **`Mool`** | *Root / Base* | The path to the unpacked NCI root filesystem. | Linux, Termux |
| **`Karya`** | *Task / Work* | The primary binary/script to execute upon boot. | Linux, Termux |
| **`Sthapana`**| *Establishment* | Intercepts the boot, runs the native package manager (`apt`/`apk`), and installs these tools dynamically. | Linux, Termux |
| **`Smriti`** | *Memory* | Enforces physical RAM boundaries via `memory.max` cgroups. | Linux Only |
| **`Shakti`** | *Power / Energy* | Limits CPU cycles via `cpu.max` cgroups constraints. | Linux Only |
| **`Kendra`** | *Center / Core* | Binds the container process strictly to specific CPU cores via `cpuset.cpus`. | Linux Only |
| **`Bhaar`** | *Weight / Load* | Limits IOPS / Disk Read-Write limits in MB/s via `io.max`. | Linux Only |
| **`Gati`** | *Speed* | Injects Traffic Control (`tc`) token buckets on the veth interface to cap network speed. | Linux Only |
| **`Sangjna`** | *Name / Identity* | Sets the UNIX hostname (`sethostname`) and auto-updates `/etc/hosts`. | Linux, Termux |
| **`Sanket`** | *Signal* | Overwrites the container's `/etc/resolv.conf` with custom nameservers. | Linux, Termux |
| **`Dwar`** | *Door / Gateway* | Translates Host ports to Guest IP ports via isolated `iptables` NAT routing. | Linux Only |
| **`Sadasya`** | *Member / User* | Spawns a mapped unprivileged UNIX user, configures `sudoers.d`, and alters UID/GID via `setuid`. | Linux, Termux |
| **`Sanchay`** | *Storage / Vault* | Generates `MS_BIND` mount points to share host files with the container. | Linux, Termux |
| **`Vayu`** | *Air / Wind* | Mounts blazing-fast volatile `tmpfs` RAM-disks inside the container. | Linux Only |
| **`Tejas`** | *Radiance / Fire* | Securely mounts `/dev/dri` and `/proc/driver/nvidia` into the guest. | Linux Only |
| **`Suraksha`**| *Security* | `ephemeral` sets up an OverlayFS. Sandbox destruction wipes all writes instantly. | Linux, Termux |
| **`Kavach`** | *Armor* | `strict` engages a custom Seccomp-BPF matrix blocking dangerous system calls. | Linux Only |
| **`Adhikar`** | *Rights* | Manipulates `PR_CAPBSET_DROP` to explicitly deny Kernel Capabilities. | Linux Only |
| **`Kala`** | *Time* | Generates a time-isolated illusion overriding system timezones locally. | Linux, Termux |
| **`Paryavaran`**| *Environment* | Key-value pairs mapped natively into the execution context. | Linux, Termux |

---

## 🛠️ CLI Command Reference

| Command | Alias | Description |
|---------|-------|-------------|
| `vessel suchi` | `list` | Lists locally installed OS bases and synchronizes with remote GitHub custom workspaces. |
| `vessel prapt <image>` | `pull` | Downloads, extracts, and compiles an image into the deduplicated local NCI chunk registry. |
| `vessel nishkaas <img...>` | `rm` | Evicts the image footprint and reclaims compressed block storage. |
| `vessel direct [prompt]` | `run` | Boots a sandbox using the local `Patra` file. If a natural language prompt is passed (e.g., `vessel direct ubuntu run bash`), it compiles an ephemeral manifest on the fly. |
| `vessel sanchay <dir>` | `build` | Compresses a target directory using FastCDC+BLAKE3, outputting a `nova-catalog.json` tree. |

---

## 🏗️ Architecture Under The Hood

Vessel is composed of two distinct operational modes:

1. **Native Linux Execution (Requires Root):**
   When executing as `sudo` on Linux, Vessel forks the current process and triggers a `CLONE_NEWNS | CLONE_NEWIPC | CLONE_NEWUTS | CLONE_NEWNET` unshare syscall. It sets up private virtual ethernet bridges, generates isolated `cgroups v2` structures for resource limits, enforces `seccomp` bytecode filters, sets up a Copy-On-Write OverlayFS, and pivots the root directory (`pivot_root` / `chroot`).

2. **Termux Android Execution (Unprivileged Fallback):**
   Because modern Android strictly prohibits unprivileged users from using `mount` or `chroot`, Vessel dynamically detects the `$PREFIX` variable. If Termux is found, Vessel intelligently redirects configuration files and paths into the Termux user space, skipping cgroups, and invokes **PRoot** to trap system calls and emulate a root filesystem user-space boundary.

---

## 👨‍💻 About the Developer

<div align="center">
  <table>
    <tr>
      <td align="center">
        <a href="https://github.com/juniorsir">
          <img src="https://github.com/juniorsir.png?size=100" width="100px;" alt="JuniorSir" style="border-radius:50%"/>
        </a>
        <br />
        <b>JuniorSir</b>
        <br />
        <i>Systems & Low-Level Engineer</i>
      </td>
      <td>
        Passionate about high-performance virtualization, Rust, and pushing the limits of local computing. <br><br>
        🌍 <b>GitHub:</b> <a href="https://github.com/juniorsir">@juniorsir</a><br>
        💡 <b>Mission:</b> Bringing enterprise-grade, zero-latency sandboxing to everyone—from massive Linux servers down to Android phones via Termux.
      </td>
    </tr>
  </table>
</div>

---

## 📝 License & Contributing

Vessel is open-source software licensed under the [MIT License](LICENSE). 

Contributions are heavily encouraged! Because Vessel touches low-level OS primitives, any PRs optimizing FastCDC chunking, expanding Seccomp rules, or adding Mac/Windows compatibility drivers are welcome.
