# Zavro Client

A native Linux desktop proxy client for **Fedora 45 (GNOME)**, built with
[Tauri v2](https://tauri.app/) (Rust + React + TypeScript) and powered by
[Xray-core](https://github.com/XTLS/Xray-core).

Zavro Client connects to your own proxy servers and routes your traffic
through them — with a clean native-feel UI, one-click connect, per-server
ping checks, a live speed graph, and optional system-wide proxy or TUN
routing.

> **Status:** v1.0.0. Built and released by GitHub Actions on Fedora 45.
> The supported build path is CI on Fedora — see
> [Build from source](#build-from-source).

## Features

- **Protocols:** VLESS, VMess, Trojan, Shadowsocks
- **Subscriptions:** import server lists from a subscription URL
- **System proxy:** sets the GNOME system proxy via `gsettings`
  (SOCKS5 on port **10808**, HTTP on port **10809**)
- **TUN mode:** routes all traffic through a `tun0` interface via Xray's
  `tun` inbound (requires a polkit/`pkexec` prompt — see below)
- **Routing modes:** Rule (proxy by geo rules) / Global (proxy everything)
  / Direct (bypass proxy)
- **Ping check:** TCP-connect latency to each node's host:port
- **Live telemetry:** speed graph sampled from the network interface,
  connection uptime, per-session traffic totals
- **Activity log:** connect/disconnect events, Xray output, errors

## Install (Fedora 45)

Download the latest release from the
[Releases page](https://github.com/AbolfazlNasiriDev/ZavroClient/releases)
(`v1.0.0` and later).

### Option A — .rpm (recommended)

```bash
sudo dnf install ./zavro-client-1.0.0-1.x86_64.rpm
zavro-client
```

### Option B — .AppImage (no install)

```bash
chmod +x Zavro_Client-1.0.0_x86_64.AppImage
./Zavro_Client-1.0.0_x86_64.AppImage
```

## Build from source

The supported build path is **GitHub Actions on Fedora 45**
(`.github/workflows/release.yml`). A local build on Fedora 45 works too;
a local build on Ubuntu (the dev VM) is **not** supported — Tauri needs
extra `apt` packages there and the resulting `.rpm`/`.AppImage` would not
be Fedora builds anyway.

### Prerequisites (Fedora 45)

```bash
# Rust (stable) + Node.js 24
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
curl -fsSL https://rpm.nodesource.com/setup_24.x | sudo bash -
sudo dnf install -y nodejs

# Tauri v2 system dependencies
sudo dnf install -y \
  webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel \
  librsvg2-devel libxdo-devel openssl-devel \
  rpm-build patchelf curl git jq unzip

cargo install tauri-cli --version "^2" --locked
```

### Build

```bash
git clone https://github.com/AbolfazlNasiriDev/ZavroClient.git
cd ZavroClient
npm ci
npm run build

# Fetch Xray-core (latest stable) and stage it — CI does this automatically;
# for a local build, do it by hand:
XRAY_TAG=$(curl -fsSL https://api.github.com/repos/XTLS/Xray-core/releases/latest | jq -r .tag_name)
curl -fsSL -o /tmp/Xray-linux-64.zip "https://github.com/XTLS/Xray-core/releases/download/${XRAY_TAG}/Xray-linux-64.zip"
curl -fsSL -o src-tauri/resources/geoip.dat "https://github.com/XTLS/Xray-core/releases/download/${XRAY_TAG}/geoip.dat"
curl -fsSL -o src-tauri/resources/geosite.dat "https://github.com/XTLS/Xray-core/releases/download/${XRAY_TAG}/geosite.dat"
unzip -o /tmp/Xray-linux-64.zip -d /tmp/xray-dl
cp /tmp/xray-dl/xray src-tauri/binaries/xray-x86_64-unknown-linux-gnu
chmod +x src-tauri/binaries/xray-x86_64-unknown-linux-gnu

cargo tauri build --bundles rpm,appimage
# Bundles land in src-tauri/target/release/bundle/{rpm,appimage}/
```

To run the app in development mode (no bundle):

```bash
cargo tauri dev
```

## Xray-core & geo files — provenance

Zavro Client does **not** vendor Xray-core in git. At CI build time the
workflow queries the GitHub API for the latest stable release of
[XTLS/Xray-core](https://github.com/XTLS/Xray-core) and downloads:

- `Xray-linux-64.zip` → the `xray` binary, staged at
  `src-tauri/binaries/xray-x86_64-unknown-linux-gnu`
- `geoip.dat` and `geosite.dat` → shipped **inside** that zip in current
  Xray-core releases (not separate download assets), staged at
  `src-tauri/resources/`

The binary is verified to run (`./xray version`) before the build
continues, and the workflow fails loudly if any asset is missing or empty.

> **Version note:** Xray-core upstream moved from `Xray-core` to `XTLS`.
> The latest stable release is **v26.11.9** (2026-11-09). The reference
> design image showed `1.8.8` — that is an old version and is **not** what
> this app ships. The CI workflow always resolves the current latest
> stable release at build time, so the shipped version is whatever is
> latest when the release is cut.

## How it works

### System proxy (default)

When you connect with system proxy enabled, Zavro Client sets the GNOME
proxy settings via `gsettings`:

- `org.gnome.system.proxy mode = 'manual'`
- SOCKS proxy `127.0.0.1:10808` (Xray `socks` inbound)
- HTTP proxy `127.0.0.1:10809` (Xray `http` inbound)

Disconnecting restores the proxy mode to `none`. Only GNOME settings are
touched; no system files are modified.

### TUN mode

TUN mode creates a `tun0` interface and routes traffic through Xray's
`tun` inbound, so even apps that ignore the system proxy are covered.
Enabling it triggers a **polkit (`pkexec`) prompt** — you must approve it
as an administrator. It requires the kernel `tun` module to be available
(standard on Fedora 45). TUN takes effect for the Xray `tun` inbound on
the next connect; it is an independent toggle from system-proxy mode, so
you can run either or both — TUN covers apps that ignore the system proxy.

### Routing

- **Rule** — Xray's default routing rules (geoip/geosite): Iranian/local
  traffic direct, the rest through the proxy.
- **Global** — everything through the proxy.
- **Direct** — everything bypasses the proxy (Xray still runs, but
  traffic is not proxied).

## Security notes

- Your server credentials (UUIDs, passwords, subscription URLs) are
  stored **locally** on your machine only. They are never sent anywhere
  except to the proxy servers you configure.
- The app never phones home. The only network requests it makes on its
  own are the ones you trigger: subscription updates and ping checks.
- TUN mode asks for admin rights via `pkexec` only when you enable it.
- No real API keys or accounts are involved — the app talks directly to
  your Xray servers.

## Honest limitations

- **TUN needs polkit + the kernel `tun` module.** If polkit is not
  running or the `tun` module is unavailable, TUN mode will fail with an
  error in the log. System-proxy mode works without admin rights.
- **The speed graph is interface-sampled from `/proc/net/dev`**, not
  measured inside Xray. It reflects traffic on the active interface
  (including non-proxied traffic) and is an approximation.
- **Ping is a TCP connect to the node's host:port**, not Xray's built-in
  URL test. It measures reachability/latency to the server, not proxied
  throughput.
- **Seed nodes are the user's own Zavro servers.** They ship with no
  measurements — latency appears only after you run **Check Ping**.
- **CI on Fedora is the supported build path.** A local Ubuntu build of
  Tauri may need extra `apt` dependencies and is not how releases are
  produced.
- Only **x86_64** Linux is built and tested. ARM builds are not provided.

## Project layout

```
zavro-client/
├── src/                    # React + TypeScript frontend
├── src-tauri/
│   ├── src/                # Rust backend (Tauri commands, Xray manager)
│   ├── binaries/           # xray sidecar binary (CI-staged, gitignored)
│   ├── resources/          # geoip.dat / geosite.dat (CI-staged, gitignored)
│   ├── icons/              # app icon
│   └── tauri.conf.json
├── .github/workflows/
│   └── release.yml         # Fedora 45 CI: build .rpm + .AppImage, publish release
├── BUILD_LOG.md
└── README.md
```

## License

MIT — see [LICENSE](LICENSE). Copyright (c) 2026 Abolfazl Nasiri.
