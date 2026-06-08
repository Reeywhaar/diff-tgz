#!/usr/bin/env ruby
# Generates the diff-tgz Homebrew formula for https://github.com/Reeywhaar/homebrew-tap.
#
# Usage:
#   update_formula.rb <formula_path> <version> \
#     <mac_intel_sha> <mac_arm_sha> \
#     <linux_intel_sha> <linux_arm_sha>

abort "Usage: #{$0} <formula> <version> <mac_intel_sha> <mac_arm_sha> <linux_intel_sha> <linux_arm_sha>" \
  unless ARGV.length == 6

formula_path, version, mac_intel_sha, mac_arm_sha, linux_intel_sha, linux_arm_sha = ARGV

repo = "Reeywhaar/diff-tgz"

[mac_intel_sha, mac_arm_sha, linux_intel_sha, linux_arm_sha].each do |sha|
  abort "Invalid SHA-256: #{sha}" unless sha.match?(/\A[a-f0-9]{64}\z/)
end

def asset_url(repo, version, target)
  "https://github.com/#{repo}/releases/download/#{version}/diff-tgz-#{target}.tar.gz"
end

content = <<~RUBY
  class DiffTgz < Formula
    desc "Compute and apply VCDIFF binary patches between tgz archives"
    homepage "https://github.com/#{repo}"
    license "MIT"
    version "#{version}"

    on_macos do
      on_intel do
        url "#{asset_url(repo, version, "x86_64-apple-darwin")}"
        sha256 "#{mac_intel_sha}"
      end

      on_arm do
        url "#{asset_url(repo, version, "aarch64-apple-darwin")}"
        sha256 "#{mac_arm_sha}"
      end
    end

    on_linux do
      on_intel do
        url "#{asset_url(repo, version, "x86_64-unknown-linux-gnu")}"
        sha256 "#{linux_intel_sha}"
      end

      on_arm do
        url "#{asset_url(repo, version, "aarch64-unknown-linux-gnu")}"
        sha256 "#{linux_arm_sha}"
      end
    end

    def install
      bin.install "diff-tgz"
    end

    test do
      system "\#{bin}/diff-tgz", "--help"
    end
  end
RUBY

File.write(formula_path, content)
puts "Written #{formula_path} (diff-tgz #{version})"

