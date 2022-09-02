when defined(emcc):
  {.emit: "#include <emscripten.h>".}
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "EMSCRIPTEN_KEEPALIVE $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}

import light_client_utils

proc assertLCFailTest*(
    data: Natural
  ): void {.wasmPragma.} =
  assertLC(data == 0, BlockError.Invalid)
