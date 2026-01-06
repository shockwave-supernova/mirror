# 🛰️ Mastodon Mirror

![Rust](https://img.shields.io/badge/rust-1.75%2B-orange?logo=rust)
![License](https://img.shields.io/badge/license-MIT-blue)
![Platform](https://img.shields.io/badge/platform-linux%20%7C%20armv6-lightgrey)

A lightweight, asynchronous bot designed to mirror posts from one Mastodon account to another. 

Built with **Rust**, it is optimized for low-resource hardware like the **Raspberry Pi Zero W**. It handles media attachments, preserves content formatting, and manages API rate limits intelligently.

## ✨ Features

- **Efficient Polling:** Checks for new posts periodically (default: every 2 minutes) to minimize CPU usage.
- **Media Support:** Downloads and re-uploads images and videos, preserving alt-text descriptions.
- **Smart Filtering:**
  - Ignores Replies and Reblogs (mirrors original content only).
  - Skips conversations (posts starting with `@user`).
- **State Persistence:** Remembers the ID of the last mirrored post to prevent duplicates after restarts.
- **Zero-Dependency TLS:** Uses `rustls` instead of OpenSSL, making cross-compilation for ARM/Linux painless.
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

### Local Development (x86_64)
To run the bot on your local machine:

    cargo run --release

### Cross-Compilation for Raspberry Pi Zero W (ARMv6)
This project uses `rustls` to avoid OpenSSL linking issues on older hardware. We recommend using `cargo-zigbuild`.

1. **Install dependencies:**

    cargo install cargo-zigbuild
    rustup target add arm-unknown-linux-gnueabihf

2. **Build for ARMv6:**
   *Note: We specify glibc 2.28 to ensure compatibility with Raspberry Pi OS.*

    cargo zigbuild --release --target arm-unknown-linux-gnueabihf.2.28

3. **Deploy:**
   The binary will be located at:
   `target/arm-unknown-linux-gnueabihf/release/mirror`

   Copy it to your Raspberry Pi:

    scp -P 22 target/arm-unknown-linux-gnueabihf/release/mirror pi@raspberrypi.local:/home/pi/
    scp -P 22 .env pi@raspberrypi.local:/home/pi/

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
    User=pi
    WorkingDirectory=/home/pi/mirror
    ExecStart=/home/pi/mirror/mirror
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
