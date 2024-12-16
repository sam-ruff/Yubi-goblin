# Yubi-goblin
A yubikey UI for managing 2FA yubikey settings for login with Ubuntu.

# Running
```bash
cargo run
```

## Installing the binary
```bash
cargo build --release
sudo cp target/release/yubi-goblin /usr/local/bin/yubi-goblin
sudo chmod +x /usr/local/bin/yubi-goblin
# You can then run 
yubi-goblin
# After the first run a desktop entry will be created
# Yubi goblin always needs to run as root
```