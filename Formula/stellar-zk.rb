# typed: false
# frozen_string_literal: true

# This is a template formula. The CI pipeline replaces the placeholders
# and pushes the rendered file to the salazarsebas/homebrew-tap repository.
class StellarZk < Formula
  desc "ZK DevKit for Stellar/Soroban — unified CLI for Groth16, UltraHonk, and RISC Zero"
  homepage "https://github.com/salazarsebas/stellar-zk"
  version "VERSION_PLACEHOLDER"
  license "MIT"

  on_macos do
    on_intel do
      url "https://github.com/salazarsebas/stellar-zk/releases/download/v#{version}/stellar-zk-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "SHA256_MAC_X86_64_PLACEHOLDER"
    end

    on_arm do
      url "https://github.com/salazarsebas/stellar-zk/releases/download/v#{version}/stellar-zk-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "SHA256_MAC_ARM64_PLACEHOLDER"
    end
  end

  on_linux do
    on_intel do
      url "https://github.com/salazarsebas/stellar-zk/releases/download/v#{version}/stellar-zk-v#{version}-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "SHA256_LINUX_X86_64_PLACEHOLDER"
    end
  end

  def install
    bin.install "stellar-zk"
  end

  test do
    assert_match "stellar-zk", shell_output("#{bin}/stellar-zk --version")
  end
end
