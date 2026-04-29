# cratesinfo

[![CI](https://github.com/cumulus13/crates-info/actions/workflows/ci.yml/badge.svg)](https://github.com/cumulus13/crates-info/actions/workflows/ci.yml)
[![Release](https://github.com/cumulus13/crates-info/actions/workflows/release.yml/badge.svg)](https://github.com/cumulus13/crates-info/releases)
[![Crates.io](https://img.shields.io/crates/v/crates-info.svg)](https://crates.io/crates/crates-info)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

A fast CLI tool to get detailed information about Rust crates from [crates.io](https://crates.io) — like `pipinfo` for PyPI or `npm info` for Node, but for the Rust/Cargo ecosystem.

**Author:** Hadi Cahyadi \<cumulus13@gmail.com\>  
**Homepage:** https://github.com/cumulus13/crates-info

---

## Features

- 📦 Full crate metadata — description, downloads, MSRV, license, dates
- 🔗 Links — docs.rs, homepage, repository
- 🏷️  Keywords & categories
- ⚙️  Feature flags with their dependency tree
- 📜 README rendered in the terminal (Markdown + HTML)
- 📋 Full version history with size, license, downloads
- 🔗 Dependency listing (normal / dev / build), grouped and coloured
- 🔍 Search crates.io
- 👤 Crate owners
- 🌍 **Cross-platform** — Windows, macOS, Linux (uses pure-Rust TLS, no OpenSSL needed)

---

## Install

### Pre-built binaries (recommended)

Download the latest binary for your platform from the [Releases page](https://github.com/cumulus13/crates-info/releases):

| Platform | File |
|---|---|
| Windows x64 | `cratesinfo-vX.Y.Z-windows-x86_64.zip` |
| macOS Intel | `cratesinfo-vX.Y.Z-macos-x86_64.tar.gz` |
| macOS Apple Silicon | `cratesinfo-vX.Y.Z-macos-arm64.tar.gz` |
| Linux x64 (static) | `cratesinfo-vX.Y.Z-linux-x86_64.tar.gz` |
| Linux ARM64 (static) | `cratesinfo-vX.Y.Z-linux-arm64.tar.gz` |

**Windows** — extract the `.zip`, place `cratesinfo.exe` somewhere on your `PATH`  
(e.g. `C:\Users\<you>\bin\` or `C:\Program Files\cratesinfo\`).

**macOS / Linux:**
```sh
tar -xzf cratesinfo-*.tar.gz
sudo mv cratesinfo /usr/local/bin/
```

### Via cargo (all platforms)

```sh
cargo install crates-info
```

### Build from source

```sh
git clone https://github.com/cumulus13/crates-info
cd crates-info
cargo build --release
# binary: target/release/cratesinfo  (or cratesinfo.exe on Windows)
```

---

## Usage

```
cratesinfo <COMMAND>
```

### Commands

| Command | Description |
|---|---|
| `info <crate>` | Full metadata + latest version details |
| `info <crate> --readme` | Same + README rendered inline |
| `versions <crate>` | Latest 20 published versions |
| `versions <crate> --all` | All versions |
| `deps <crate> [version]` | Dependencies (normal / dev / build) |
| `readme <crate>` | Render README in the terminal |
| `search <terms...>` | Search crates.io |
| `search <terms> --limit N` | Limit results (default: 10) |
| `owners <crate>` | Crate owners |

### Examples

```sh
cratesinfo info serde
cratesinfo info tokio --readme
cratesinfo versions actix-web --all
cratesinfo deps serde
cratesinfo deps serde 1.0.195
cratesinfo readme reqwest
cratesinfo search async runtime
cratesinfo search http client --limit 5
cratesinfo owners tokio
```

---

## GitHub Actions

Three workflows are included:

| Workflow | Trigger | What it does |
|---|---|---|
| `ci.yml` | Push / PR | Format check, Clippy lint, build+test on Linux/Windows/macOS |
| `release.yml` | `git push --tags` (e.g. `v0.2.0`) | Cross-compiles 5 platform binaries, creates GitHub Release with all assets + SHA256SUMS |
| `publish.yml` | GitHub Release published | `cargo publish` to crates.io |

### Releasing a new version

```sh
# 1. bump version in Cargo.toml
# 2. commit + tag
git add Cargo.toml Cargo.lock
git commit -m "chore: release v0.2.0"
git tag v0.2.0
git push origin main --tags
# → release.yml builds all 5 binaries and creates the GitHub Release automatically
```

For `cargo publish` to work, add your crates.io API token as a repository secret named `CARGO_REGISTRY_TOKEN` in **Settings → Secrets and variables → Actions**.

---

## License

MIT — see [LICENSE](LICENSE)

## 👤 Author
        
[Hadi Cahyadi](mailto:cumulus13@gmail.com)
    

[![Buy Me a Coffee](https://www.buymeacoffee.com/assets/img/custom_images/orange_img.png)](https://www.buymeacoffee.com/cumulus13)

[![Donate via Ko-fi](https://ko-fi.com/img/githubbutton_sm.svg)](https://ko-fi.com/cumulus13)
 
[Support me on Patreon](https://www.patreon.com/cumulus13)