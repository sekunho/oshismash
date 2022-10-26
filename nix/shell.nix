{ pkgs, ... }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    cargo
    cargo-watch
    rustc
    rust-analyzer
    rustfmt

    nodePackages.tailwindcss

    # Database
    sqitchPg
    perl534Packages.TAPParserSourceHandlerpgTAP
    pkg-config
    openssl
  ];

  PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
}
