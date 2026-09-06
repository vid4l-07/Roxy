<div align="center">

# Roxy

Minimal terminal-based HTTP intercepting proxy.

</div>

## Overview

Roxy is a lightweight HTTP intercepting proxy with a terminal user interface (TUI) built in Rust. It allows you to capture, inspect, and modify HTTP requests in real time before they reach the server. Think of it as a minimal, terminal-native alternative to Burp Suite for HTTP traffic analysis.

## Features

- **HTTP Interception** — Capture and inspect requests in real time before they reach the server.
- **Request Editing** — Modify intercepted requests on the fly using your preferred external editor.
- **Repeater** — Resend and tweak requests manually, similar to Burp Suite's Repeater.
- **Multiple Tabs** — Organize different requests in separate Repeater tabs.
- **Vim-style Navigation** — Scroll with `j`/`k` for efficient keyboard-driven workflow.
- **Custom TUI** — Clean, responsive interface built with [ratatui](https://github.com/ratatui/ratatui).

## Installation

### Build from source

```bash
git clone https://github.com/vid4l-07/Roxy.git
cd Roxy
cargo build --release
```

The binary will be available at `target/release/roxy`.

## Usage

Configure your browser or HTTP client to use `127.0.0.1:8080` as a proxy.

### Global

| Key | Action |
| --- | --- |
| `q` | Quit |
| `Tab` | Switch between Proxy and Repeater screens |

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
| `n` | Next repeater tab |
| `p` | Previous repeater tab |
| `x` | Close current tab |
| `↑`/`k` | Scroll up |
| `↓`/`j` | Scroll down |
| `←`/`h` / `→`/`l` | Toggle focus between Request and Response panels |
| `H` | Decrease Request panel width |
| `L` | Increase Request panel width |

### External Editor

When you press `e`, the request is opened in the editor defined by `$EDITOR`. If `$EDITOR` is not set, the command will fail with an error message. Edit the raw request, save, and exit. The modified request will replace the original, and `Content-Length` is automatically recalculated.

### How it works

1. **Proxy** listens on `127.0.0.1:8080` for incoming HTTP connections.
2. When intercept is **ON**, incoming requests are captured and displayed in the TUI.
3. The user can **forward** the request as-is, **edit** it in an external editor, or **send it to Repeater**.
4. **Repeater** allows resending requests independently and viewing responses side-by-side.
5. Communication between the TUI and the proxy happens via async channels (`tokio::mpsc`).

## Contributions

Contributions are always welcome. If you find a bug or want to help with new features, you can:

- Open an issue in the repository.
- Open a pull request.

