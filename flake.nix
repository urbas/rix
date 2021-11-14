{
  description = "A reimplementation or nix in Rust.";

  inputs.nixpkgs.url = "nixpkgs/nixpkgs-unstable";

  outputs = { self, nixpkgs }: rec {
    supportedSystems = [ "x86_64-linux" "aarch64-linux" ];
    forAllSystems = f: nixpkgs.lib.genAttrs supportedSystems (system: f system);
    nixpkgsForAllSystems = forAllSystems (system:
      import nixpkgs { inherit system; }
    );

    devShell = forAllSystems (system:
      with nixpkgsForAllSystems.${system};

      stdenv.mkDerivation {
        name = "rix";
        buildInputs = [ nix rustup ];
        shellHook = ''
          PATH=$prefix/bin:$PATH
        '';
      }
    );
  };
}
