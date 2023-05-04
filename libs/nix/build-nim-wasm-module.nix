{
  nim-wasm,
  runCommand,
  lib,
}: {
  name,
  srcPath,
  srcFile ? name,
  buildInputs ? [],
  extraArgs ? "",
}: let
  nim2wasm = lib.getExe nim-wasm;
  outFileName = "${name}.wasm";
in
  runCommand name {inherit buildInputs;} ''
    mkdir -p $out/nimcache
    mkdir -p $out/lib
    ${nim2wasm} c --nimcache:$out/nimcache -o:$out/lib/${outFileName} ${extraArgs} ${srcPath}/${srcFile}.nim
  ''
