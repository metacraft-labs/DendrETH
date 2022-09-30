when defined(emcc):
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "__attribute__((used)) $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}

func sumOfArrayElements*(arr: array[5, int]): int {.wasmPragma.} =
  var sum: int
  for val in arr:
    sum += val
  sum

func createNewArray*(el: int): array[5, int] {.wasmPragma.} =
  var res: array[5, int]
  for i in 0..4:
    res[i] = el
  res

func arrayMapAdd*(arr: array[5, int],
                  value: int): array[5, int] {.wasmPragma.} =
  var res: array[5, int]
  for i in 0..4:
    res[i] = arr[i] + value
  res
