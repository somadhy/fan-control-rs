# CPU Fan control on rust (fan-control-rs)
CPU fan control util with GPIO with RUST

## 🛠 Setup Steps

1. Build and install your binary:

```bash
cargo build --release
sudo cp target/release/fan-control-rs /usr/local/bin/
```

2. Update service file and deploy it:

Use next command line params for `ExecStart`
```
  --chip /dev/gpiochip0  # Chip path
  --offset 50            # GPIO pin offset GPIO50
  --on-temp 60           # Fan on temerature in C
  --off-temp 45          # Fan off temperature in C  
  --interval 2000        # Temperature check interval in mS
  --verbose              # Verbose mode
```

```bash
sudo cp fan-control-rs.service /etc/systemd/system/
```

3. Enable & start the service:

```bash
sudo systemctl daemon-reexec
sudo systemctl daemon-reload
sudo systemctl enable fan-control-rs.service
sudo systemctl start fan-control-rs.service
```

Check status:

```bash
sudo systemctl status fan-control.service
```

# ✅ Permissions Note

If your binary doesn’t run due to permission issues:

Add your user to the gpio group:
```bash
    sudo usermod -aG gpio $USER
```
Or run with elevated capabilities (as shown above via `AmbientCapabilities`).


