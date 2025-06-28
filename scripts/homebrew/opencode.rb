# To install:
# brew tap your-org/opencode-rs https://github.com/your-org/opencode-rs-tap
# brew install opencode

class Opencode < Formula
  desc "AI-powered, sandboxed coding suite in Rust"
  homepage "https://github.com/your-org/opencode-rs"
  version "0.1.0" # This would be updated automatically by CI
  license "MIT"
  
  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/your-org/opencode-rs/releases/download/v0.1.0/opencode-aarch64-apple-darwin.tar.gz"
      sha256 "sha256-goes-here" # This would be updated automatically by CI
    else
      url "https://github.com/your-org/opencode-rs/releases/download/v0.1.0/opencode-x86_64-apple-darwin.tar.gz"
      sha256 "sha256-goes-here" # This would be updated automatically by CI
    end
  end

  on_linux do
    if Hardware::CPU.intel?
      url "https://github.com/your-org/opencode-rs/releases/download/v0.1.0/opencode-x86_64-linux-musl.tar.gz"
      sha256 "sha256-goes-here" # This would be updated automatically by CI
    end
  end

  def install
    bin.install "opencode"
    
    # Install shell completions if available
    if (buildpath/"completions").exist?
      bash_completion.install "completions/opencode.bash" => "opencode"
      zsh_completion.install "completions/_opencode"
      fish_completion.install "completions/opencode.fish"
    end
  end

  test do
    system "#{bin}/opencode", "--version"
    system "#{bin}/opencode", "--help"
  end
end