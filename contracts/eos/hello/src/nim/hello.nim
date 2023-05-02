when defined(emcc):
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "__attribute__((used)) $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}

# For debugging purposes. This way we can call print from the nim code in the EOS contract
when defined(DEBUG):
  const irr = "</nix/store/88nw85nvskb71hsch8m22bka1hckq3k5-cdt/cdt/include/eosiolib/core/eosio/print.hpp>"
  proc print(value: cstring) {. header:irr, importc:"eosio::internal_use_do_not_use::prints", cdecl}


proc helloFromNim(): cstring {.wasmPragma.} =
  return "Hello from Nim"
