# Release Checklist

## Version Bump (if needed)
```bash
# Update version in Cargo.toml
# Current: 0.2.0
```

## Commit and Push
```bash
git add -A
git commit -m "feat: complete TUI redesign with menu system

- Add main menu with scan/quick scan/options
- Add loading animation during scan
- Add navigation guide in all views  
- Fix ASCII art spelling (RESIKNO)
- Update README for new TUI-first interface
- Add MIT LICENSE
- Add CONTRIBUTING.md
- Add USER_GUIDE.md and CHEATSHEET.md"

git push origin main
```

## Tag Release
```bash
# Create annotated tag
git tag -a v0.2.0 -m "Release v0.2.0 - TUI Redesign

Major update with complete interface overhaul:
- Beautiful ASCII art header
- Main menu with multiple scan options
- In-TUI cleanup confirmation and execution
- Loading animation with progress bar
- Navigation guide on all screens
- Mouse/trackpad scroll support
- Help screen integrated

Full changelog in CHANGELOG.md"

# Push tag
git push origin v0.2.0
```

## Build Release Binary
```bash
# Build optimized release
cargo build --release

# Create release directory
mkdir -p release
cp target/release/resikno release/resikno-mac-v0.2.0

# Create tarball
tar -czf release/resikno-mac-v0.2.0.tar.gz -C release resikno-mac-v0.2.0

# Show binary info
ls -lh release/
file release/resikno-mac-v0.2.0
```

## GitHub Release (Manual)
1. Go to https://github.com/esmondo/resikno-mac/releases
2. Click "Draft a new release"
3. Choose tag: v0.2.0
4. Release title: "v0.2.0 - TUI Redesign"
5. Description:

```markdown
## What's New

### 🎨 Beautiful New Interface
- ASCII art header with RESIKNO branding
- Main menu for easy navigation
- Navigation guide on every screen

### ✨ New Features
- **Menu System**: Choose between Full Scan, Quick Scan, Review Results, Restore, Help
- **Loading Animation**: Braille spinner with progress bar during scan
- **In-TUI Cleanup**: Confirmation and execution without leaving the interface
- **Mouse Support**: Scroll with trackpad/mouse
- **Help Screen**: Press '5' or '?' for keyboard shortcuts

### 🚀 Improved UX
- Direct TUI launch with `resikno` command
- Return to menu anytime with 'm' key
- Better button navigation (← →) in dialogs
- Cleaner, more consistent styling

### 📚 Documentation
- Complete User Guide (USER_GUIDE.md)
- Quick Cheatsheet (CHEATSHEET.md)
- Contributing guidelines (CONTRIBUTING.md)

## Installation

### From Source
```bash
git clone https://github.com/esmondo/resikno-mac.git
cd resikno-mac
cargo install --path .
```

### Pre-built Binary
Download `resikno-mac-v0.2.0` from Assets below, then:
```bash
chmod +x resikno-mac-v0.2.0
sudo mv resikno-mac-v0.2.0 /usr/local/bin/resikno
```

## SHA256 Checksums
```
[TO BE FILLED AFTER BUILD]
```

## Contributors
- @esmondo - Initial development and TUI redesign
```

6. Upload `resikno-mac-v0.2.0.tar.gz` to release assets
7. Publish release

## Post-Release
- [ ] Update Homebrew formula (if applicable)
- [ ] Post on social media
- [ ] Update website/docs
