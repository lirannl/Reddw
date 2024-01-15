Reddw - a light and robust wallpaper manager

Compile instructions:
Install rustup/rust
```bash
cargo tool install cargo-cli
cd src-tauri
# For development
cargo tauri dev
# For release
cargo tauri build
```
Release executable will be src-tauri/target/release/reddw(.exe)

Todo:
- [ ] Split config updates into discrete types of updates
- [ ] Make config file updates use comparisons to determine what type of update to dispatch
- [ ] React to config updates differently based on the type of update