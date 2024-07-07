{ self, pkgs }:

let

  inherit (pkgs) buildNpmPackage;

in
buildNpmPackage rec {
  name = "nixjs-rt";
  src = self;
  npmDepsHash = "sha256-ev5i6sL2mQgxu7kuLVaMIJfUVEXcP27n8u2RiGE0Cd8=";
}
