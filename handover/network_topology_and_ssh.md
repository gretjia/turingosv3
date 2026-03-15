# Hardware Topology and Network SSH Info

Based on the canonical Omega Protocol Network Runbook and Handover Snapshot (2026-03-16).

## 1. Hardware Topology

### Primary Control Node: omega-vm
- **Location:** Google Cloud VM
- **Role:** Master control node. Primary site for code repository, Gemini CLI, and Git synchronization. No GPU, 16GB RAM. Not used for training.
- **Tailscale IP:** `100.122.223.27`
- **WireGuard IP:** `10.88.0.1`
- **Public IP:** `34.27.27.164`

### Central Command Console: zephrymac-studio (Mac)
- **Location:** Home LAN (`192.168.3.93`)
- **Role:** Chief Architect's primary operating machine. Used to access `omega-vm` and serve as the central console for deployment. Apple Silicon M4, 32GB RAM.
- **Tailscale IP:** `100.72.87.94`

### Worker Node: windows1-w1 (Windows Forge)
- **Location:** Home LAN (`192.168.3.112`)
- **Role:** High-performance data forge and deterministic computation.
- **Specs:** AMD AI Max 395, 128GB Unified Memory. Storage: 4TB Internal SSD + 8TB USB4 External SSD (containing Level-2 tick data 2023 - Jan 2026).
- **Tailscale IP:** `100.123.90.25`

### Worker Node: linux1-lx (Linux Forge)
- **Location:** Home LAN (`192.168.3.113`)
- **Role:** High-performance data forge, always-on jump host.
- **Specs:** AMD AI Max 395, 128GB Unified Memory. Storage: 4TB Internal SSD + 8TB USB4 External SSD (containing Level-2 tick data 2023 - Jan 2026).
- **Tailscale IP:** `100.64.97.113`

### HK Public Hub (vm-0-7-ubuntu)
- **Role:** Public hub, reverse-SSH landing point, WireGuard peer.
- **Public IP:** `43.161.252.57`
- **Tailscale IP:** `100.81.234.55`
- **WireGuard IP:** `10.88.0.2`

## 2. SSH and Network Routing Decisions

The network is in control-plane mode. The pure Tailscale mesh under GFW was too unstable. A controlled hybrid design is in place:
1. `MacStudio` connects to `omega-vm` via HK public SSH jump to WireGuard.
2. `omega-vm` reaches Shenzhen home devices via HK-hosted reverse SSH listeners.

### Routing from MacStudio to `omega-vm`
- **Primary:** `ssh omega-vm-hk` (MacStudio -> HK public `43.161.252.57` -> `10.88.0.1` over WireGuard wg0).
- **Secondary (Direct):** `ssh omega-vm` (direct Tailscale to `100.122.223.27`).
- **Fallback:** `ssh omega-vm-hk-ts` (MacStudio -> HK Tailscale `100.81.234.55` -> `10.88.0.1`).

### Routing from `omega-vm` to Shenzhen Devices
- **Primary for linux:** `ssh linux1-lx` (ProxyJump `hk-wg`, port `2226`).
- **Primary for Mac:** `ssh zephrymac-studio` (ProxyJump `hk-wg`, port `2227`).
- **Primary for Windows:** `ssh windows1-w1` (ProxyJump `hk-wg`, port `2228`).
- **Fallbacks via Linux jump:** `ssh zephrymac-studio-via-linux1` and `ssh windows1-via-linux1`.
- **Legacy Fallbacks:** `linux1-back` and `windows1-back` using older Mac reverse tunnel (`omega-mac-back.service` on `localhost:2223`, `2224`).

## 3. Credentials & Accounts

- **HK SSH user:** `ubuntu`
- **`omega-vm` SSH user:** `zephryj`
- **Linux SSH user:** `zepher`
- **Windows SSH user:** `jiazi`
- **`omega-vm` HK jump key:** `/home/zephryj/.ssh/hktail.pem`
- **Worker key (Linux/Windows):** `/home/zephryj/.ssh/id_ed25519_omega_workers`
- **Mac reverse-SSH key:** `/home/zephryj/.ssh/id_ed25519_mac_backssh`
