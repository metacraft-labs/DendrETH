when defined(emcc):
  {.emit: "#include <emscripten.h>".}
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "EMSCRIPTEN_KEEPALIVE $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}

proc print(value: cdouble) {.importc, cdecl}

proc sum*(a,b: float64): float64 {.wasmPragma.} =
  a + b

proc printAdd*(a,b: float64) {.wasmPragma.} =
  print(sum(a, b))
