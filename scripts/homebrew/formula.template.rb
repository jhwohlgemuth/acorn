class AcornCli < Formula
  desc "ACORN command line interface"
  homepage "https://acorn.ornl.gov"
  url "https://static.crates.io/crates/acorn-cli/acorn-cli-__VERSION__.crate"
  sha256 "__SHA256__"
  license "MIT"
  head "https://code.ornl.gov/research-enablement/acorn.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "acorn-cli")
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/acorn --version")
  end
end
