# Template formula for https://github.com/Reeywhaar/homebrew-tap.
#
# Copy this file to:
#   Formula/diff-tgz.rb
# inside the Reeywhaar/homebrew-tap repository.
#
# The release workflow keeps the placeholder lines up to date automatically.

class DiffTgz < Formula
  desc "Compute and apply VCDIFF binary patches between tgz archives"
  homepage "https://github.com/Reeywhaar/diff-tgz"
  license "MIT"

  on_macos do
    on_intel do
      # url-mac-intel-placeholder
      url "https://github.com/Reeywhaar/diff-tgz/releases/download/0.0.0/diff-tgz-0.0.0-x86_64-apple-darwin.tar.gz"
      # sha256-mac-intel-placeholder
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end

    on_arm do
      # url-mac-arm-placeholder
      url "https://github.com/Reeywhaar/diff-tgz/releases/download/0.0.0/diff-tgz-0.0.0-aarch64-apple-darwin.tar.gz"
      # sha256-mac-arm-placeholder
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  on_linux do
    on_intel do
      # url-linux-intel-placeholder
      url "https://github.com/Reeywhaar/diff-tgz/releases/download/0.0.0/diff-tgz-0.0.0-x86_64-unknown-linux-gnu.tar.gz"
      # sha256-linux-intel-placeholder
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end

    on_arm do
      # url-linux-arm-placeholder
      url "https://github.com/Reeywhaar/diff-tgz/releases/download/0.0.0/diff-tgz-0.0.0-aarch64-unknown-linux-gnu.tar.gz"
      # sha256-linux-arm-placeholder
      sha256 "0000000000000000000000000000000000000000000000000000000000000000"
    end
  end

  # version-placeholder
  version "0.0.0"

  def install
    bin.install "diff-tgz"
  end

  test do
    system "#{bin}/diff-tgz", "--help"
  end
end
