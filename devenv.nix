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

    targets = [ "wasm32-unknown-unknown" ];
  };

  languages.javascript = {
    enable = true;
    pnpm.enable = true;
  };

  packages = with pkgs; [
    (cargo-llvm-cov.overrideAttrs (old: {
      doCheck = false;
    }))
    wasm-pack
  ];
}
