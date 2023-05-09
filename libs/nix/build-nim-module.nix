{
  nim,
  runCommand,
  lib,
}: {
  name,
  src,
  srcFile ? name,
  outFileName ? "nimcache/",
  buildInputs ? [],
  extraArgs ? "",
}: let
  nimC = "${nim}/bin/nim";
in
  runCommand name {
    inherit buildInputs src;
  } ''
    mkdir -p $out/nimcache
    mkdir -p $out/lib
    ${nimC} c --nimcache:$out/nimcache -o:$out/${outFileName} ${extraArgs} $src/${srcFile}.nim
  ''
