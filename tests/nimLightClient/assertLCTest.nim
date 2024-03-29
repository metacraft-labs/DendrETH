when defined(emcc):
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "__attribute__((used)) $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}

import light_client_utils

proc assertLCFailTest*(
    data: Natural
  ): void {.wasmPragma.} =
  assertLC(data == 0, BlockError.Invalid)
