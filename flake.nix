{
  description = "aoike";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
    typst.url = "github:typst/typst";
  };

  outputs =
    {
      nixpkgs,
      rust-overlay,
      flake-utils,
      typst,
      ...
    }:
    flake-utils.lib.eachDefaultSystem (
      system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };
        rust-tools = pkgs.rust-bin.nightly.latest.default.override {
          extensions = [ "rust-src" ];
          targets = [ "wasm32-unknown-unknown" ];
        };
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = [ rust-tools ];
          packages = [
            typst.packages.${system}.default
          ]
          ++ (with pkgs; [
            bun
            wasm-bindgen-cli
            trunk
            leptosfmt
          ]);
        };
      }
    );
}
