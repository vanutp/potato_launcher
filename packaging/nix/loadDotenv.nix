{ lib }:
path:
lib.pipe path [
  builtins.readFile
  (lib.splitString "\n")
  (builtins.filter (x: x != ""))
  (map (lib.splitString "="))
  (map (x: (lib.throwIf (lib.length x != 2) "Invalid environment file entry: ${x}") x))
  (map (x: {
    name = lib.elemAt x 0;
    value = lib.elemAt x 1;
  }))
  lib.listToAttrs
]
