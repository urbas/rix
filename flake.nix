{
  description = "A reimplementation or nix in Rust.";

  inputs.nixpkgs.url = "nixpkgs/nixpkgs-unstable";

  outputs = { self, nixpkgs }:
    let
      forAllSystems = f: nixpkgs.lib.genAttrs [ "x86_64-linux" "aarch64-linux" ] (system: f { pkgs = import nixpkgs { inherit system; }; });

    in {
      packages = forAllSystems ({pkgs}: pkgs);
      devShells = forAllSystems ({pkgs}: with pkgs; {
        default = stdenv.mkDerivation {
          name = "rix";
          buildInputs = [
            busybox-sandbox-shell
            coreutils
            nix
            rustup
          ];
        };
      });
    };
}
