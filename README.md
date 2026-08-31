<div align="center">

# Roxy

Minimal terminal-based HTTP intercepting proxy.

[Features](#features) · [Installation](#installation) · [Usage](#usage) · [Roadmap](#roadmap)

</div>

## Features

- **HTTP interception** — capture and inspect requests in real time before they reach the server.
- **Request editing** — modify intercepted requests on the fly using an external editor.
- **Repeater** — resend and tweak requests manually, just like Burp Suite's Repeater.
- Custom TUI built with ```ratatui```.

---

## Installation

### Build from source

```bash
git clone https://github.com/vid4l-07/Roxy.git
cd Roxy
cargo build --release
```

The binary will be available at ```target/release/miburp```.

---

## Usage

The proxy starts immediately as an interactive TUI application. Configure your browser or HTTP client to use ```127.0.0.1:8080``` as a proxy.

### Global

| Key        | Action               |
| -------    | -------------------- |
| ```q```    | Quit                 |
| ```Tab```  | Switch between Proxy and Repeater screens |

### Proxy

| Key              | Action                          |
| ---------------- | ------------------------------- |
| ```i```          | Toggle intercept ON/OFF         |
| ```Enter```      | Forward the intercepted request |
| ```e```          | Edit request in external editor |
| ```r```          | Send request to Repeater        |

### Repeater

| Key              | Action                           |
| ---------------- | -------------------------------- |
| ```Enter```      | Send the current request         |
| ```e```          | Edit request in external editor  |
| ```n```          | Next repeater tab                |
| ```p```          | Previous repeater tab            |

### External editor

When you press ```e```, the request is opened in the editor defined by ```$EDITOR``` (falls back to ```vi```).
Edit the raw request, save, and exit. The modified request will replace the original.

---

## Contributions

Contributions are always welcome. If you find a bug or want to help with new features, you can:

- Open an issue in the repository.
- Open a pull request.
