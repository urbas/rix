{
  description = "A reimplementation or nix in Rust.";

  inputs.nixpkgs.url = "nixpkgs/nixpkgs-unstable";

  outputs = { self, nixpkgs }:
    let
      supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
      forSupportedSystems = f: with nixpkgs.lib; foldl' (resultAttrset: system: recursiveUpdate resultAttrset (f { inherit system; pkgs = import nixpkgs { inherit system; }; })) { } supportedSystems;

    in
    forSupportedSystems ({ pkgs, system, ... }:
      let
        rix-deps = with pkgs; [
          busybox-sandbox-shell
          coreutils
          cargo-watch
          nix
          nixpkgs-fmt
          rustup
        ];

        nixjs-rt-deps = with pkgs; [
          nodejs
          parallel
          prefetch-npm-deps
        ];

        nixjs-rt = import ./nixjs-rt/pkg.nix { inherit pkgs; self = "${self}/nixjs-rt"; };

      in
      {
        packages.${system} = { inherit nixjs-rt pkgs; };
        devShells.${system}.default = pkgs.stdenv.mkDerivation {
          name = "rix";
          buildInputs = rix-deps ++ nixjs-rt-deps;
        };
      });
}
