<div align="center">

# Roxy

**Minimal terminal-based HTTP/HTTPS intercepting proxy.**

https://github.com/user-attachments/assets/94f9c9ae-5842-4b19-9f5b-84dedb9430e6

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE) [![Rust 2024](https://img.shields.io/badge/rust-2024-orange.svg)](Cargo.toml)

</div>


## Overview

Roxy is a lightweight intercepting proxy for HTTP and HTTPS with a terminal user interface (TUI) written in Rust. 
It captures, inspects and modifies requests in real time, before they reach the server, from inside your terminal. 
Think of it as a minimal, terminal-native alternative to Burp Suite for everyday traffic analysis.

## Features

- **HTTP interception** — capture and inspect requests in real time before they reach the server.
- **HTTPS interception** — full MITM support through `CONNECT`, with certificates generated automatically.
- **Automatic certificate authority** — a local CA is created on first use and leaf certificates are issued per host on demand. Nothing to configure by hand.
- **Request editing** — modify intercepted requests on the fly with your `$EDITOR`.
- **Repeater** — resend and tweak requests manually, in separate tabs, like Burp Suite's Repeater.
- **Custom TUI** — clean and responsive interface built with [ratatui](https://ratatui.rs/).

## Requirements

To build it from source you need:

| Requirement | Notes |
| --- | --- |
| Rust 1.85 or newer | Roxy uses the 2024 edition. Check with `rustc --version`. Install from [rustup](https://rustup.rs). |
| A C compiler | `gcc` or `clang`. |
| CMake 3.18 or newer | Required to build the TLS backend. Install it with your package manager, e.g. `sudo apt install build-essential cmake` or `sudo dnf install gcc cmake`. |
| `$EDITOR` | Only needed for the request editing feature. |

> [!Note]
> CMake and a C compiler are not optional. Roxy relies on [rustls](https://github.com/rustls/rustls), which builds [AWS-LC](https://github.com/aws/aws-lc) from source through `aws-lc-sys`. 
> If the build fails while compiling `aws-lc-sys`, install `cmake` and a C compiler and run `cargo build --release` again.

## Installation

### Build from source

```bash
git clone https://github.com/vid4l-07/Roxy.git
cd Roxy
cargo build --release
```

The binary will be available at `target/release/roxy`.

### Install it straight from the repository

```bash
cargo install --git https://github.com/vid4l-07/Roxy.git
```
The binary will be available at `~/.cargo/bin/roxy`.

## HTTPS support

### Where the certificates live

| What | Location | Notes |
| --- | --- | --- |
| CA certificate and key | `~/.config/roxy/ca.crt`, `~/.config/roxy/ca.key` | Created lazily, the first time an HTTPS request is intercepted. |
| Leaf certificate per host | `/tmp/roxy_certs/<host>.crt`, `/tmp/roxy_certs/<host>.key` | Issued on demand and cached. The cache is wiped whenever a new CA is generated. |

### Trusting the Roxy CA

Roxy generates a self-signed CA named `Roxy CA`. 

>[!Warning]
> Every HTTPS request will fail validation until the CA is trusted by the client.

Install `~/.config/roxy/ca.crt` in your browser or system.

**Firefox / Chrome**:
1. `Settings` → `Privacy & Security` → `Certificates` → `View Certificates`.
2. Go to the **Authorities** tab and press **Import**.
3. Select `~/.config/roxy/ca.crt`.

**System CA store** 
For tools like curl.

```bash
# Debian / Ubuntu
sudo cp ~/.config/roxy/ca.crt /usr/local/share/ca-certificates/roxy.crt
sudo update-ca-certificates

# Fedora / RHEL / Arch
sudo cp ~/.config/roxy/ca.crt /etc/pki/ca-trust/source/anchors/roxy.crt
sudo update-ca-trust extract
```

To remove it again, delete the file and re-run the corresponding command.

### Rotating or removing the CA

```bash
rm -rf ~/.config/roxy
```

Roxy generates a new CA on the next HTTPS request and removes every leaf certificate it had cached. 
You will have to trust the new CA again. Remember to remove the old one from your browser and system store.

## Usage

### Global

| Key | Action |
| --- | --- |
| `q` | Quit |
| `Tab` | Switch between Proxy and Repeater screens |
| `?` | Open the help popup for the current screen |

### Proxy

| Key | Action |
| --- | --- |
| `i` | Toggle intercept ON/OFF |
| `Enter` | Forward the intercepted request |
| `e` | Edit request in external editor |
| `r` | Send request to Repeater |
| `↑`/`k` | Scroll up |
| `↓`/`j` | Scroll down |

### Repeater

| Key | Action |
| --- | --- |
| `Enter` | Send the current request |
| `e` | Edit request in external editor |
| `r` | Rename current tab |
| `n` | Next repeater tab |
| `p` | Previous repeater tab |
| `x` | Close current tab |
| `z` | Toggle zoom on the focused panel |
| `↑`/`k` | Scroll up |
| `↓`/`j` | Scroll down |
| `←`/`h` / `→`/`l` | Toggle focus between Request and Response panels |
| `H` | Decrease Request panel width |
| `L` | Increase Request panel width |

### External Editor

When you press `e`, the raw request is written to a temporary file and opened with the binary defined by `$EDITOR`. Edit it, save, and quit.
The modified request replaces the original and `Content-Length` is recalculated automatically.

## How it works

1. **The listener** accepts connections on `127.0.0.1:8080` and handles each one in its own async task.
2. **A plain `CONNECT` request** is answered with `200 Connection Established`, TLS is terminated locally with a certificate issued for the requested host, and the decrypted request is read as a regular HTTP request.
3. **Any other request** is read directly as plain HTTP, resolving the destination from the absolute target or the `Host` header.
4. **Intercept ON**: the request is pushed to the TUI over a `tokio::mpsc` channel and the connection waits on a `oneshot` channel until the user forwards it. The request can be forwarded as-is, edited first, or sent to the Repeater.
5. **Intercept OFF**: the request is forwarded immediately.
6. **Forwarding** opens a connection to the upstream server (plain TCP for HTTP, TLS verified against `webpki-roots` for HTTPS), writes the request, and streams the raw response back to the client.
7. **The Repeater** resends a stored request whenever you press `Enter`, choosing the transport based on the protocol the request was captured with, and shows the response side by side with the request.

The TUI and the proxy never share state directly. They only talk through the event channels, so the interface stays responsive while requests are being intercepted.

## Contributions

Contributions are always welcome. If you find a bug or want to help with new features, you can:

- Open an issue in the repository.
- Open a pull request.

## License

Roxy is released under the [MIT License](LICENSE). © 2026 Hugo Vidal Martinez.
