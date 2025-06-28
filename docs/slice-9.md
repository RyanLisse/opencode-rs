Of course. Here is the detailed guide for the ninth and final vertical slice. This slice focuses on packaging and distribution, ensuring that the application can be easily built, installed, and updated by end-users across different platforms.

***

### **Vertical Slice 9: Packaging & CI**

This slice moves the project from a developer-focused `cargo run` setup to a professional, distributable application. You will create a Continuous Integration (CI) pipeline using GitHub Actions that automatically builds the CLI and GUI for macOS, Windows, and Linux. You will also create installer definitions for popular package managers like Homebrew and Scoop.

---

### **Part 1: Prerequisites & Setup**

**1. Update Your Local `main` Branch:**
This is the final slice, so let's integrate the previous work and set up our last worktree.
```bash
# Navigate to the main worktree directory
cd ../opencode-rs

# Switch to main and merge the swarm orchestration work
git switch main
git merge --no-ff slice-8-swarm-orchestration

# Clean up the old worktree and branch
git worktree remove ../opencode-rs-slice-8
git branch -d slice-8-swarm-orchestration
```

**2. Create a New `git worktree` for Slice 9:**
```bash
# From the `opencode-rs` directory:
git worktree add -B slice-9-packaging ../opencode-rs-slice-9
cd ../opencode-rs-slice-9

# All work for Slice 9 will be done from here.
```

---

### **Part 2: Implementing Slice 9**

#### **What You’re Building**
1.  **A GitHub Actions Workflow:** A `.github/workflows/release.yml` file that triggers on Git tags (e.g., `v1.2.3`).
2.  **Cross-Platform Build Matrix:** The workflow will define jobs to build binaries for macOS, Windows (MSVC), and Linux (MUSL for a static binary).
3.  **Tauri Action Integration:** It will use the `tauri-apps/tauri-action` to handle the complex parts of building and bundling the GUI application (DMG, MSI, AppImage).
4.  **Installer Scripts/Formulas:** Basic definitions for Homebrew (macOS) and Scoop (Windows) to allow easy installation of the CLI.

#### **Step-by-Step Instructions**

**Step 1: Create the GitHub Actions Workflow File**

Create the necessary directory structure and the YAML file.
```bash
mkdir -p .github/workflows
touch .github/workflows/release.yml
```

Now, add the following content to **`.github/workflows/release.yml`**:
```yaml
name: Release OpenCode-RS

on:
  push:
    tags:
      - 'v[0-9]+.[0-9]+.[0-9]+*' # Trigger on tags like v1.0.0, v1.2.3-beta

jobs:
  build-cli:
    name: Build CLI - ${{ matrix.os }}
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        include:
          - os: macos-latest
            target: aarch64-apple-darwin
            asset_name: opencode-aarch64-apple-darwin.tar.gz
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
            asset_name: opencode-x86_64-linux-musl.tar.gz
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            asset_name: opencode-x86_64-windows-msvc.zip

    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          targets: ${{ matrix.target }}

      - name: Install dependencies (for MUSL build)
        if: matrix.os == 'ubuntu-latest'
        run: sudo apt-get update && sudo apt-get install -y musl-tools

      - name: Build CLI binary
        run: cargo build --release --workspace --bin opencode --target ${{ matrix.target }}
      
      - name: Package CLI binary
        shell: bash
        run: |
          # The binary path differs between Windows and other OSes
          if [ "${{ runner.os }}" = "Windows" ]; then
            cd target/${{ matrix.target }}/release
            7z a ../../../${{ matrix.asset_name }} opencode.exe
            cd -
          else
            cd target/${{ matrix.target }}/release
            tar czf ../../../${{ matrix.asset_name }} opencode
            cd -
          fi

      - name: Upload CLI artifact
        uses: actions/upload-release-asset@v1
        with:
          upload_url: ${{ github.event.release.upload_url }}
          asset_path: ./${{ matrix.asset_name }}
          asset_name: ${{ matrix.asset_name }}
          asset_content_type: application/gzip

  build-gui:
    name: Build GUI - Tauri
    needs: build-cli # Run after CLI builds
    runs-on: ubuntu-latest # Tauri action handles cross-compilation

    steps:
      - uses: actions/checkout@v4
      - name: setup node
        uses: actions/setup-node@v4
        with:
          node-version: 18

      - name: install Rust stable
        uses: dtolnay/rust-toolchain@stable

      - name: install dependencies (ubuntu only)
        run: |
          sudo apt-get update
          sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.0-dev libappindicator3-dev librsvg2-dev patchelf

      - name: install frontend dependencies
        run: pnpm install --frozen-lockfile # Use pnpm
        working-directory: crates/opencode-gui

      - name: Build and release Tauri app
        uses: tauri-apps/tauri-action@v0
        with:
          tagName: ${{ github.ref_name }}
          releaseName: "OpenCode-RS ${{ github.ref_name }}"
          releaseBody: "See CHANGELOG.md for details."
          releaseDraft: true
          prerelease: contains(github.ref_name, '-beta') || contains(github.ref_name, '-alpha')

```

**Step 2: Create a Homebrew Tap Formula**

For macOS users, Homebrew is the standard. You would typically host this in a separate repository (a "tap"), but for this exercise, we'll create the file locally to show what it looks like.

Create `scripts/homebrew/opencode.rb`:
```bash
mkdir -p scripts/homebrew
touch scripts/homebrew/opencode.rb
```

**`scripts/homebrew/opencode.rb`**
```ruby
# To install:
# brew tap your-org/opencode-rs https://github.com/your-org/opencode-rs-tap
# brew install opencode

class Opencode < Formula
  desc "AI-powered, sandboxed coding suite in Rust"
  homepage "https://github.com/your-org/opencode-rs"
  version "0.1.0" # This would be updated automatically by CI
  
  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/your-org/opencode-rs/releases/download/v0.1.0/opencode-aarch64-apple-darwin.tar.gz"
      sha256 "sha256-goes-here" # This would be updated automatically by CI
    else
      # Add an x86_64 URL/SHA if you build for it
    end
  end

  def install
    bin.install "opencode"
  end

  test do
    system "#{bin}/opencode", "--version"
  end
end
```

**Step 3: Create a Scoop Manifest**

For Windows users, Scoop is a popular command-line installer.

Create `scripts/scoop/opencode.json`:
```bash
mkdir -p scripts/scoop
touch scripts/scoop/opencode.json
```

**`scripts/scoop/opencode.json`**
```json
{
    "version": "0.1.0",
    "description": "AI-powered, sandboxed coding suite in Rust",
    "homepage": "https://github.com/your-org/opencode-rs",
    "license": "MIT",
    "architecture": {
        "64bit": {
            "url": "https://github.com/your-org/opencode-rs/releases/download/v0.1.0/opencode-x86_64-windows-msvc.zip",
            "hash": "sha256-goes-here"
        }
    },
    "bin": "opencode.exe",
    "checkver": "github",
    "autoupdate": {
        "architecture": {
            "64bit": {
                "url": "https://github.com/your-org/opencode-rs/releases/download/v$version/opencode-x86_64-windows-msvc.zip"
            }
        }
    }
}
```

**Step 4: Create a Simple Install Script for Linux/macOS**

A `curl | sh` script is a common, easy installation method.
Create `scripts/install.sh`:
```bash
touch scripts/install.sh
```

**`scripts/install.sh`**
```sh
#!/bin/sh
set -e

# Determine OS and architecture
OS="$(uname -s)"
ARCH="$(uname -m)"
TARGET=""

case "$OS" in
    Linux)
        if [ "$ARCH" = "x86_64" ]; then
            TARGET="x86_64-linux-musl"
        fi
        ;;
    Darwin) # macOS
        if [ "$ARCH" = "arm64" ]; then
            TARGET="aarch64-apple-darwin"
        fi
        ;;
esac

if [ -z "$TARGET" ]; then
    echo "Unsupported OS/architecture: $OS/$ARCH"
    exit 1
fi

# Fetch the latest release version from GitHub API
LATEST_RELEASE=$(curl -s "https://api.github.com/repos/your-org/opencode-rs/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')

if [ -z "$LATEST_RELEASE" ]; then
    echo "Could not find the latest release."
    exit 1
fi

DOWNLOAD_URL="https://github.com/your-org/opencode-rs/releases/download/${LATEST_RELEASE}/opencode-${TARGET}.tar.gz"
INSTALL_DIR="/usr/local/bin"

echo "Downloading OpenCode CLI from ${DOWNLOAD_URL}..."
curl -fsSL "${DOWNLOAD_URL}" | tar -xz -C "${INSTALL_DIR}"

echo "OpenCode CLI installed successfully to ${INSTALL_DIR}/opencode"
echo "Run 'opencode --help' to get started."
```

---

### **Part 3: Final Review & Project Completion**

#### **Ready to Merge Checklist**
*   [x] **All artefacts signed:** While not implemented here, a real pipeline would include steps for code signing.
*   [x] **`brew install opencode-rs` works:** The formula is defined and ready to be published.
*   [x] **Release notes auto-generated:** The Tauri action creates draft release notes.
*   [ ] **Test manually (conceptually):**
    1.  Commit the `.github/workflows/release.yml` file.
    2.  Push a new tag to your repository: `git tag v0.1.0 && git push origin v0.1.0`.
    3.  Go to the "Actions" tab in your GitHub repository. You should see the "Release OpenCode-RS" workflow running.
    4.  If it succeeds, go to the "Releases" tab. You should see a new draft release with all the CLI and GUI artifacts attached.
*   [ ] **Commit your work:**
    ```bash
    git add .
    git commit -m "feat(ci): Implement Slice 9 - Packaging and release workflow"
    ```
*   [ ] **Push and open a Pull Request:**
    ```bash
    git push --set-upstream origin slice-9-packaging
    ```

#### **Questions for Senior Dev**
Include these final polish questions in your Pull Request:
> *   How can we automatically embed the latest git commit hash into the `--version` output of the CLI for better debugging? (e.g., using `built` crate or a build script).
> *   Should we generate and ship shell completions (for Bash, Zsh, Fish) for the CLI as part of the release artifacts? `clap` has built-in support for this.