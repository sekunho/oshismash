{
  description = "Smash or Pass: VTuber Edition";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
  };

  outputs = { self, nixpkgs }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages."${system}";
    in {

      devShells."${system}".default = pkgs.mkShell {
        buildInputs = with pkgs; [
          cargo
          cargo-watch
          rustc
          rust-analyzer
          rustfmt

          nodePackages.tailwindcss

          # Databse
          sqitchPg
          perl534Packages.TAPParserSourceHandlerpgTAP
          pkg-config
          openssl
        ];
      };

    };
}
