proc print(value: cdouble) {.importc, cdecl}

proc sum*(a,b: float64): float64 {.cdecl, exportc, dynlib.} =
  a + b

proc printAdd*(a,b: float64) {.cdecl, exportc, dynlib} =
  print(sum(a, b))

proc start*() {.exportc: "_start".} =
  discard
