# GearClaw Desktop Client

## Requirements
- MacOS 12.0+ (Sequoia recommended for latest gpui)
- XCode Command Line Tools (Required for Metal shader compilation)
  ```bash
  xcode-select --install
  ```
- Rust 1.75+

## Troubleshooting
### Metal Shader Compilation Failed
If you see `xcrun: error: unable to find utility "metal"`, ensure Xcode Command Line Tools are installed and selected:
```bash
sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer
```

### Dependency Conflicts
If you encounter `mismatched types` in `core-graphics` or `zed-font-kit`:
1. Ensure you are using the latest `gpui` git dependency (already configured).
2. Try running `cargo update`.

2. Run the client:
   ```bash
   cargo run -p gearclaw_gui
   ```

## Structure
- Built with `gpui` (zed-industries).
- Connects to `gearclaw_core` for logic.
