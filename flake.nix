{
  description = "Figure out how many people would smash your oshis!";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs";
  };

  outputs = { self, nixpkgs }: (
    let
      pkgs = nixpkgs.legacyPackages.x86_64-linux;
    in {
      packages.x86_64-linux.hello = pkgs.hello;

      defaultPackage.x86_64-linux = pkgs.hello;

      devShell.x86_64-linux = pkgs.mkShell {
        buildInputs = with pkgs; [
          cargo
          rustc
          rust-analyzer

          sqitchPg
          perl534Packages.TAPParserSourceHandlerpgTAP
        ];
      };
    }
  );
}
