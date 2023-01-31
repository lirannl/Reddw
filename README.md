Reddw - a light-yet-robust wallpaper manager

Compile instructions:
Install rustup/rust
```bash
cargo tool install cargo-cli
cd src-tauri
# For development
cargo tauri dev
# For release
npm build && cargo build -r
```
Release executable will be src-tauri/target/release/reddw.exe
