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
          cargo-watch
          nixpkgs-fmt
          rustup
        ];

        nixjs-rt-deps = with pkgs; [
          nodejs
          parallel
          prefetch-npm-deps
        ];

        pkgs' = pkgs.extend (pkgs-self: pkgs-super: {
          nixjs-rt = pkgs-self.callPackage ./nixjs-rt/pkg.nix {
            self = "${self}/nixjs-rt";
          };

          rix = pkgs-self.callPackage ./pkg.nix { rixSources = self; };
        });

      in
      with pkgs';
      {
        packages.${system} = { inherit nixjs-rt rix; default = rix; };
        devShells.${system}.default = pkgs.stdenv.mkDerivation {
          name = "rix";
          buildInputs = rix-deps ++ nixjs-rt-deps;
          shellHook = ''
            export RIX_NIXRT_JS_MODULE=nixjs-rt/dist/lib.mjs
          '';
        };
      });
}
