# Build definition for `copy`, shared by the flake's per-system package and the
# overlay (so the derivation lives in exactly one place). Invoke with
# `pkgs.callPackage ./nix/package.nix { }`.
{ lib, rustPlatform }:

let
  # Only the tracked sources — drop build outputs and VCS data so the store
  # path (and therefore the build) stays reproducible.
  src = lib.cleanSourceWith {
    name = "copy-source";
    src = ../.;
    filter = path: _type:
      let base = baseNameOf (toString path);
      in !(
        base == "target"
        || base == "result"
        || lib.hasPrefix "result-" base
        || base == ".git"
      );
  };
in
rustPlatform.buildRustPackage {
  pname = "copy";
  version = "0.1.4"; # keep in sync with Cargo.toml

  inherit src;

  cargoLock.lockFile = ../Cargo.lock; # committed lockfile -> no vendor hash

  # Default features need no system libraries. SELinux xattr preservation is
  # opt-in; to enable it, uncomment:
  #   buildFeatures = [ "selinux-support" ];
  #   buildInputs = [ libselinux ];

  meta = {
    description = "A modern, fast file copying tool";
    homepage = "https://github.com/UnbreakableMJ/copy";
    license = lib.licenses.mit;
    mainProgram = "copy";
    platforms = lib.platforms.linux;
  };
}
