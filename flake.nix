{
  description = "A reimplementation or nix in Rust.";

  inputs.nixpkgs.url = "nixpkgs/nixpkgs-unstable";
  inputs.nixrt.url = "github:urbas/nixrt";

  outputs = { self, nixpkgs, nixrt }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
      forSupportedSystems = f: with nixpkgs.lib; foldl' (resultAttrset: system: recursiveUpdate resultAttrset (f { inherit system; pkgs = import nixpkgs { inherit system; }; })) {} supportedSystems;

    in forSupportedSystems ({ pkgs, system, ... }:
      let
        buildInputs = with pkgs; [
          busybox-sandbox-shell
          coreutils
          nix
          nixrt.packages.${system}.default
          rustup
        ];

      in {
        packages.${system} = { inherit nixrt pkgs; };
        devShells.${system}.default = pkgs.stdenv.mkDerivation {
          name = "rix";
          inherit buildInputs;
          shellHook = ''
            export RIX_NIXRT_JS_MODULE=${nixrt.packages.${system}.default}/lib/node_modules/nixrt/dist/lib.js
            export RUSTFLAGS=-Dwarnings
          '';
        };
      });
}
