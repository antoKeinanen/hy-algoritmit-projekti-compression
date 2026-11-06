{ pkgs, ... }:

{
  languages.rust = {
    enable = true;
    channel = "stable";

    components = [
      "rustc"
      "cargo"
      "clippy"
      "rustfmt"
      "rust-analyzer"
      "llvm-tools-preview"
    ];
  };

  packages = with pkgs; [
    (cargo-llvm-cov.overrideAttrs (old: {
      doCheck = false;
    }))
  ];
}
