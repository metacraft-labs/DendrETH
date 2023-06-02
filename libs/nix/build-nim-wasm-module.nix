{
  nim-wasm,
  runCommand,
  lib,
}: {
  name,
  src,
  srcFile ? name,
  outFileName ? "${name}.wasm",
  buildInputs ? [],
  extraArgs ? "",
}: let
  nim2wasm = lib.getExe nim-wasm;
in
  runCommand name {
    inherit buildInputs src;
  } ''
    mkdir -p $out/nimcache
    mkdir -p $out/lib
    ${nim2wasm} c --nimcache:$out/nimcache -o:$out/lib/${outFileName} ${extraArgs} $src/${srcFile}.nim
  ''
