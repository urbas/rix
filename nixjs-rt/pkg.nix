{ self, pkgs }:

pkgs.buildNpmPackage rec {
  name = "nixjs-rt";
  src = self;
  npmDepsHash = "sha256-0J/nRg9cwMoBfLm3F540lIYcIa/LVHM5JopAdbDHCTQ=";
}
