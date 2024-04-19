when defined(emcc):
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "__attribute__((used)) $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}

proc print(value: int) {.importc, cdecl}

proc createSeq*(a, b: int): seq[int]  {.wasmPragma.} =
  @[a,b,a,b,a]

proc printCreateSeqLen*(a,b: int) {.wasmPragma.} =
  print(createSeq(a, b).len)

