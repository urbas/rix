{
  dataPath = (builtins.import ./nested/basic.nix).data;
  dataString = (builtins.import "./nested/basic.nix").data;
}
