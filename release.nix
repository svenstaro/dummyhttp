let
  sources = import ./nix/sources.nix {};
  pkgs = import sources.nixpkgs {
    overlays = [(import sources.rust-overlay)];
  };
  crane = pkgs.callPackage sources.crane {};
  advisory-db = sources.advisory-db;

  rustToolChain = pkgs.rust-bin.fromRustupToolchainFile ./rust-toolchain.toml;

  dummyhttp = pkgs.callPackage ./nix/build.nix {inherit crane rustToolChain advisory-db;};
in {
  inherit (dummyhttp) default doc;
  checks = {
    inherit (dummyhttp.checks) fmt clippy test audit;
  };
}
