class Resikno < Formula
  desc "A lightweight, transparent, and safe disk cleanup CLI tool for macOS"
  homepage "https://github.com/esmondo/resikno-mac"
  url "https://github.com/esmondo/resikno-mac/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "REPLACE_WITH_ACTUAL_SHA256_AFTER_RELEASE"
  license "MIT"
  head "https://github.com/esmondo/resikno-mac.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "resikno", shell_output("#{bin}/resikno --version")
  end
end
