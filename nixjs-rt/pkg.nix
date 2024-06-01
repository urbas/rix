{ self, pkgs }:

let

  inherit (pkgs) buildNpmPackage;

in
buildNpmPackage rec {
  name = "nixjs-rt";
  src = self;
  npmDepsHash = "sha256-bzi3/pHxDYgKKZsV3zVIwZ6cwGjAz0lolzeCotao+9I=";
}
