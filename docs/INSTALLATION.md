# Installation

Install Virust CLI from GitHub without needing crates.io publishing.

---

## Quick Install (Recommended)

### Using the install script (Linux/macOS):

```bash
curl -fsSL https://raw.githubusercontent.com/hangsiahong/virust/main/install.sh | bash
```

Or download and run:

```bash
# Download the script
curl -O https://raw.githubusercontent.com/hangsiahong/virust/main/install.sh

# Review it if you want
cat install.sh

# Run it
chmod +x install.sh
./install.sh
```

**This will:**
1. Clone the Virust repository
2. Build the CLI in release mode
3. Install to `~/.cargo/bin/virust`
4. Verify installation

---

## Manual Install

### Option 1: Clone and build

```bash
# Clone the repository
git clone https://github.com/hangsiahong/virust.git
cd virust

# Build in release mode
cargo build --release

# The binary will be at:
# ./target/release/virust

# Install it to your cargo bin directory
cp target/release/virust ~/.cargo/bin/virust

# Or add it to your PATH
export PATH="$PATH:$(pwd)/target/release"
```

### Option 2: Using cargo install with git

```bash
# Install directly from GitHub
cargo install --git https://github.com/hangsiahong/virust.git --bin virust

# Or specify a branch/tag
cargo install --git https://github.com/hangsiahong/virust.git --branch v0.4 --bin virust
```

**Note:** This method requires that the workspace dependencies are properly configured.

---

## Verify Installation

```bash
# Check version
virust --version

# See available commands
virust --help

# Create a new project
virust init my-project
```

---

## Requirements

- **Rust**: 1.70+ (install from [rustup.rs](https://rustup.rs/))
- **Operating System**: Linux, macOS, or Windows (WSL2)
- **Disk Space**: ~500MB for build artifacts
- **Build Time**: 2-5 minutes on modern hardware

---

## Platform-Specific Notes

### Linux

Most Linux distributions are supported:
- Ubuntu/Debian
- Fedora/RHEL
- Arch Linux
- Alpine Linux

```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Then install Virust
curl -fsSL https://raw.githubusercontent.com/hangsiahong/virust/main/install.sh | bash
```

### macOS

Works on both Intel (x86_64) and Apple Silicon (arm64/aarch64):

```bash
# Install Rust if not already installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Then install Virust
curl -fsSL https://raw.githubusercontent.com/hangsiahong/virust/main/install.sh | bash
```

### Windows (WSL2)

Windows users should use WSL2:

```powershell
# In PowerShell, enable WSL2
wsl --install

# Then in WSL2, install Rust and Virust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
curl -fsSL https://raw.githubusercontent.com/hangsiahong/virust/main/install.sh | bash
```

---

## Installation from Specific Version

To install a specific version (tag or branch):

```bash
# Using the install script
./install.sh v0.4

# Or with cargo install
cargo install --git https://github.com/hangsiahong/virust.git --tag v0.4 --bin virust
```

---

## Updating Virust

To update to the latest version:

```bash
# Re-run the install script
curl -fsSL https://raw.githubusercontent.com/hangsiahong/virust/main/install.sh | bash

# Or pull and rebuild manually
cd virust  # or wherever you cloned it
git pull origin main
cargo build --release
cp target/release/virust ~/.cargo/bin/virust
```

---

## Uninstalling

To remove Virust:

```bash
# Remove the binary
rm ~/.cargo/bin/virust

# Optionally, remove the cloned repository
rm -rf virust  # or wherever you cloned it
```

---

## Troubleshooting

### "command not found: virust"

This means `~/.cargo/bin` is not in your PATH. Add this to your `~/.bashrc` or `~/.zshrc`:

```bash
export PATH="$HOME/.cargo/bin:$PATH"
```

Then reload your shell:

```bash
source ~/.bashrc  # or source ~/.zshrc
```

### "error: linker not found"

On Linux, you may need to install build essentials:

```bash
# Ubuntu/Debian
sudo apt-get install build-essential

# Fedora/RHEL
sudo dnf groupinstall "Development Tools"

# Arch Linux
sudo pacman -S base-devel
```

### Build fails with "out of memory"

If you have limited RAM, you can build with fewer parallel jobs:

```bash
cd virust
CARGO_BUILD_JOBS=2 cargo build --release
```

### Permission denied when installing

Make sure your `~/.cargo/bin` directory is writable:

```bash
mkdir -p ~/.cargo/bin
chmod u+w ~/.cargo/bin
```

---

## Next Steps

After installing Virust:

1. **Create a new project:**
   ```bash
   virust init my-project
   cd my-project
   ```

2. **Start development server:**
   ```bash
   virust dev
   ```

3. **Build for production:**
   ```bash
   virust build
   ```

4. **Read the documentation:**
   - [Getting Started Guide](./GETTING_STARTED.md)
   - [API Documentation](./API.md)
   - [Examples](https://github.com/hangsiahong/virust/tree/main/examples)

---

## Advanced: Local Development Installation

If you're developing Virust itself and want to test your local changes:

```bash
# Clone your fork
git clone https://github.com/YOUR_USERNAME/virust.git
cd virust

# Make your changes
# ...

# Install your local version
cargo install --path crates/virust-cli

# Or link the binary
ln -sf $(pwd)/target/release/virust ~/.cargo/bin/virust
```

---

**Need help?** Open an issue at: https://github.com/hangsiahong/virust/issues
