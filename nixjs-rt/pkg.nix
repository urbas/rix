{ self
, buildNpmPackage
}:

buildNpmPackage rec {
  name = "nixjs-rt";
  src = self;
  npmDepsHash = "sha256-Irgqy5OTXg6vAdyOvRuYPpwlwYoNLdGh4Ps4U9lKiSc=";
}
