# 🚀 Mastodon to Memos Sync Bot

A lightweight, high-performance Rust bot that automatically syncs specific **Mastodon** posts to your self-hosted **Memos** instance.

It watches your Mastodon timeline for posts tagged with `#memos`, saves them to Memos with full context (media, polls, author info), and then cleans up the trigger post from Mastodon.

## ✨ Features

* **Trigger-based Syncing**: Only syncs posts (or reblogs) containing the `#memos` hashtag.
* **Rich Content Preservation**:
    * Preserves full text and formatting (HTML converted to Markdown).
    * Embeds images and videos.
    * Captures Poll options and vote counts.
    * 🔗 Saves the original source link.
* **Beautiful Formatting**: Adds a compact HTML header with the author's avatar, display name, and username to every memo.
* **Smart Auto-Tagging**: Automatically adds `#mastodon`, `#mastodon2memos`, and the author's username (e.g., `#gargron`) as tags in Memos.
* **Privacy First**: All synced memos are set to `PRIVATE` visibility by default.
* **Timeline Cleanup**: Automatically deletes the status (or undoes the reblog) from Mastodon after a successful sync to keep your feed clean.
* **Resource Efficient**: Written in Rust. Optimized for low-power devices like Raspberry Pi (polling interval: 60s).

## 🛠️ Prerequisites

* **Rust** (latest stable version)
* A **Mastodon** account and API Token.
* A **Memos** instance (v0.12+) and API Token.

## ⚙️ Configuration

1.  **Clone the repository:**
    ```bash
    git clone [https://github.com/your-username/mastodon2memos.git](https://github.com/your-username/mastodon2memos.git)
    cd mastodon2memos
    ```

2.  **Create a `.env` file** in the project root:
    ```bash
    touch .env
    ```

3.  **Add the following variables** to `.env`:

    ```ini
    # Mastodon Config
    MASTODON_URL=[https://mastodon.social](https://mastodon.social)
    MASTODON_TOKEN=your_mastodon_access_token

    # Memos Config
    MEMOS_URL=[https://memos.example.com](https://memos.example.com)
    MEMOS_TOKEN=your_memos_access_token
    ```

## 🏗️ Building

### Standard Build
```bash
cargo build --release
```
The binary will be located at `./target/release/mastodon2memos`.

### Cross-Compilation for Raspberry Pi (ARM)
If you are building on an x86 machine for a Raspberry Pi and encounter OpenSSL errors, use `cross`:

```bash
cargo install cross
cross build --target arm-unknown-linux-gnueabihf --release
```

## 🚀 Deployment (Systemd Service)

To run the bot in the background on Linux (e.g., Raspberry Pi, Ubuntu), set it up as a systemd service.

1.  **Create the service file:**
    ```bash
    sudo nano /etc/systemd/system/mastodon2memos.service
    ```

2.  **Paste the configuration** (adjust paths and user):

    ```ini
    [Unit]
    Description=Mastodon to Memos Sync Bot
    After=network.target

    [Service]
    Type=simple
    # REPLACE with your actual linux username
    User=your_linux_username
    Group=your_linux_username
    
    # Path to the folder containing your .env file
    WorkingDirectory=/home/your_linux_username/mastodon2memos
    
    # Path to the executable binary
    ExecStart=/home/your_linux_username/mastodon2memos/target/release/mastodon2memos
    
    # Restart policy
    Restart=always
    RestartSec=10
    
    # Logging
    StandardOutput=journal
    StandardError=journal

    [Install]
    WantedBy=multi-user.target
    ```

3.  **Enable and Start:**

    ```bash
    sudo systemctl daemon-reload
    sudo systemctl enable mastodon2memos
    sudo systemctl start mastodon2memos
    ```

4.  **Check Status:**
    ```bash
    sudo systemctl status mastodon2memos
    # View live logs
    journalctl -u mastodon2memos -f
    ```

## 📖 Usage

1.  Go to Mastodon (Web or App).
2.  Write a post (or reply/boost) and include the tag **`#memos`**.
3.  Wait ~60 seconds.
4.  The bot will:
    * Detect the post.
    * Send it to Memos with all attachments.
    * Delete the post from Mastodon.
5.  Open your Memos timeline to see the result!

## 📄 License

[MIT](https://choosealicense.com/licenses/mit/)
