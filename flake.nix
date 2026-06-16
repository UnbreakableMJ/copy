{
  description = "copy — a modern, fast, parallel file copying tool for Linux";

  inputs = {
    # nixos-unstable carries a Rust new enough for edition 2024 (rustc >= 1.85).
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    # copy is Linux-only (it uses copy_file_range), so only expose Linux systems.
    flake-utils.lib.eachSystem [ "x86_64-linux" "aarch64-linux" ] (system:
      let
        pkgs = import nixpkgs { inherit system; };
        copy = pkgs.callPackage ./nix/package.nix { };
      in
      {
        packages.default = copy;
        packages.copy = copy;

        apps.default = flake-utils.lib.mkApp {
          drv = copy;
          name = "copy";
        };

        devShells.default = pkgs.mkShell {
          inputsFrom = [ copy ];
          packages = with pkgs; [ cargo rustc clippy rustfmt rust-analyzer ];
        };
      })
    // {
      # Usable from other flakes: inputs.copy.overlays.default
      overlays.default = final: _prev: {
        copy = final.callPackage ./nix/package.nix { };
      };
    };
}
