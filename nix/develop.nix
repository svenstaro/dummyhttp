{
  pkgs,
  #nix-pre-commit-hooks,
  rustToolChain,
}:
pkgs.mkShell rec {
  nativeBuildInputs = with pkgs; [
    cargo
    clippy
    glib
    openssl
    #pkg-config
    pkgconfig
    rustc
    rustfmt
    rustToolChain
  ];

  #shellHook = ''
  #  # ${(import ./pre-commit.nix {inherit nix-pre-commit-hooks;}).pre-commit-check.shellHook}
  #'';
}
