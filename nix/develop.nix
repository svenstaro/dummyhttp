{
  pkgs,
  rustToolChain,
}:
pkgs.mkShell rec {
  nativeBuildInputs = with pkgs; [
    cargo
    clippy
    rustc
    rustfmt
    rustToolChain
  ];
}
