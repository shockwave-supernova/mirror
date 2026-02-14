# 🛰️ Mastodon Mirror

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-linux%20%7C%20arm%20%7C%20x86__64-lightgrey)

A lightweight, asynchronous bot designed to mirror posts from one Mastodon account to another. 

Built with **Rust**, it is highly efficient and portable. While it is optimized to run on low-resource hardware like the **Raspberry Pi Zero W**, it works perfectly on **standard Linux servers, VPS, and desktops (x86_64)**.

## ✨ Features

- **Efficient Polling:** Checks for new posts periodically (default: every 2 minutes) to minimize CPU usage.
- **Media Support:** Downloads and re-uploads images and videos, preserving alt-text descriptions.
- **Smart Filtering:**
  - Ignores Replies and Reblogs (mirrors original content only).
  - Skips conversations (posts starting with `@user`).
- **State Persistence:** Remembers the ID of the last mirrored post to prevent duplicates after restarts.
- **Zero-Dependency TLS:** Uses `rustls` instead of OpenSSL, ensuring easy compilation on any Linux distro without dependency hell.
- **Systemd Ready:** Includes service configuration for automatic background execution.

## 🛠️ Configuration

1. **Clone the repository:**

    git clone https://github.com/your-username/mirror.git
    cd mirror

2. **Create a `.env` file** in the project root:

    # Source Account (Where to take posts from)
    SOURCE_URL=https://mastodon.social
    SOURCE_TOKEN=your_source_access_token

    # Target Account (Where to publish)
    TARGET_URL=https://mastodon.social
    TARGET_TOKEN=your_target_access_token

## 🏗️ Build & Install

You can build this project for any Linux machine. Choose the option that fits your hardware.

### Option 1: Standard Build (PC, Server, VPS)
For standard 64-bit Linux systems (Ubuntu, Debian, Fedora, Arch, etc.).

1. **Install Rust:**
   If you haven't already: https://rustup.rs/

2. **Build:**

    cargo build --release

3. **Run:**
   The binary will be created in `target/release/`.

    ./target/release/mirror

---

### Option 2: Cross-Compilation for Raspberry Pi Zero W (ARMv6)
If you are building on a powerful PC for an older Raspberry Pi (Model Zero/1). We recommend using `cargo-zigbuild`.

1. **Install dependencies:**

    cargo install cargo-zigbuild
    rustup target add arm-unknown-linux-gnueabihf

2. **Build for ARMv6:**
   *Note: We specify glibc 2.28 to ensure compatibility with Raspberry Pi OS.*

    cargo zigbuild --release --target arm-unknown-linux-gnueabihf.2.28

3. **Deploy:**
   Copy the binary to your Pi:

    scp target/arm-unknown-linux-gnueabihf/release/mirror pi@raspberrypi.local:/home/pi/
    scp .env pi@raspberrypi.local:/home/pi/

## 🚀 Deployment (Systemd)

To run the bot as a background service on Linux:

1. **Create a service file:**

    sudo nano /etc/systemd/system/mastodon-mirror.service

2. **Paste the configuration:**
   *(Adjust paths and username according to your setup)*

    [Unit]
    Description=Mastodon Mirror Service
    After=network-online.target
    Wants=network-online.target

    [Service]
    User=your_user
    WorkingDirectory=/path/to/your/mirror/folder
    ExecStart=/path/to/your/mirror/folder/mirror
    Restart=always
    RestartSec=15
    StandardOutput=journal
    StandardError=journal

    [Install]
    WantedBy=multi-user.target

3. **Enable and Start:**

    sudo systemctl daemon-reload
    sudo systemctl enable --now mastodon-mirror.service

4. **Check Logs:**

    journalctl -u mastodon-mirror.service -f

## 📜 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
