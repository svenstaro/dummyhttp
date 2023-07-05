let
  sources = import ./nix/sources.nix {};
  pkgs = import sources.nixpkgs {
    overlays = [(import sources.rust-overlay)];
  };
  #nix-pre-commit-hooks = import sources."pre-commit-hooks.nix";
  rustToolChain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;
in
  pkgs.callPackage ./nix/develop.nix {inherit pkgs rustToolChain;}
