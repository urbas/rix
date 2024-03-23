{ callPackage
, nixjs-rt
, path
, rixSources
, rustPlatform
}:

rustPlatform.buildRustPackage {
  pname = "rix";
  src = rixSources;
  version = (builtins.fromTOML (builtins.readFile ./Cargo.toml)).package.version;

  cargoLock = {
    lockFileContents = builtins.readFile ./Cargo.lock;
  };

  RIX_NIXRT_JS_MODULE = "${nixjs-rt}/lib/node_modules/nixjs-rt/dist/lib.mjs";
  RUSTY_V8_ARCHIVE = callPackage "${path}/pkgs/development/web/deno/librusty_v8.nix" { };
}
