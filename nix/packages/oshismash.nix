{ pkgs, unstablepkgs, naersk-lib }:

(naersk-lib.buildPackage {
  pname = "oshismash";
  version = "0.1.0";
  root = ../../.;
  nativeBuildInputs = with pkgs; [ ];
  buildInputs = with pkgs; [ openssl pkg-config ];
}).overrideAttrs (old: {
  nativeBuildInputs = old.nativeBuildInputs ++ [
    unstablepkgs.nodePackages.tailwindcss
    unstablepkgs.esbuild
  ];

  doCheck = true;

  buildInputs = old.buildInputs;

  buildPhase = old.buildPhase + ''
    tailwindcss \
      --input assets/app.css \
      --config assets/tailwind.config.js \
      --output public/app.css \
      --minify
  '';

  installPhase = old.installPhase + ''
    mv public $out/bin
  '';
})
