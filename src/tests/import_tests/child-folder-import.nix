{
  dataPath = (builtins.import ./nested/basic.nix).data;
  dataString = (builtins.import (builtins.toString ./nested/basic.nix)).data;
}
