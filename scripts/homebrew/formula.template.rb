class AcornCli < Formula
  desc "ACORN command line interface"
  homepage "https://acorn.ornl.gov"
  url "https://static.crates.io/crates/acorn-cli/acorn-cli-__VERSION__.crate"
  sha256 "__SHA256__"
  license "MIT"
  head "https://code.ornl.gov/research-enablement/acorn.git", branch: "main"

  depends_on "openssl@3"
  depends_on "pkgconf" => :build
  depends_on "rust" => :build

  def install
    ENV["OPENSSL_DIR"] = Formula["openssl@3"].opt_prefix
    system "cargo", "install", *std_cargo_args(path: ".")
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/acorn --version")
  end
end
