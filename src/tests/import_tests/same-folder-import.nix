{
  dataPath = (builtins.import ./basic.nix).data;
  dataString = (builtins.import (builtins.toString ./basic.nix)).data;
}
