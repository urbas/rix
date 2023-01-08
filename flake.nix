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

        devEnv = pkgs.buildEnv {
          name = "devEnv";
          paths = buildInputs;
        };

      in {
        packages.${system} = { inherit devEnv nixrt pkgs; };
        devShells.${system}.default = pkgs.stdenv.mkDerivation {
          name = "rix";
          inherit buildInputs;
          shellHook = ''
            export RIX_NIXRT_JS_MODULE=${nixrt.packages.${system}.default}/lib/node_modules/nixrt/src/lib.js
          '';
        };
      });
}
