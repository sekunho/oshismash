{
  description = "Smash or Pass: VTuber Edition";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixpkgs-unstable";
    nixos-unstable.url = "github:NixOS/nixpkgs/nixos-unstable";
    naersk.url = "github:nix-community/naersk";
  };

  outputs = { self, nixpkgs, nixos-unstable, naersk }:
    let
      system = "x86_64-linux";
      pkgs = nixpkgs.legacyPackages."${system}";
      unstablepkgs = nixos-unstable.legacyPackages."${system}";

      naersk-lib = naersk.lib.${system}.override {
        cargo = pkgs.cargo;
        rustc = pkgs.rustc;
      };

      shell = import ./nix/shell.nix { inherit pkgs; };

      oshismash = import ./nix/packages/oshismash.nix {
        inherit pkgs;
        inherit unstablepkgs;
        inherit naersk-lib;
      };
    in {
      packages.${system} = {
        oshismash-unwrapped = oshismash;

        oshismash = pkgs.symlinkJoin {
          name = "oshismash";
          paths = [ oshismash ];
          buildInputs = [ pkgs.makeWrapper ];

          # https://gist.github.com/CMCDragonkai/9b65cbb1989913555c203f4fa9c23374
          postBuild = ''
            wrapProgram $out/bin/oshismash \
              --set APP__STATIC_ASSETS "${oshismash}/bin/public"
          '';
        };
      };

      apps.${system}.oshismash = {
        type = "app";
        program = "${self.packages.${system}.oshismash}/bin/oshismash";
      };

      nixosModule = import ./nix/services/oshismash.nix;
      devShells."${system}".default = shell;
    };
}
