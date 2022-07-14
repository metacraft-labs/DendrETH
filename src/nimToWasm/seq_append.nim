proc print(value: int) {.importc, cdecl}

proc createSeq*(a, b: int): seq[int]  {.cdecl, exportc, dynlib} =
  @[a,b,a,b,a]

proc printCreateSeqLen*(a,b: int) {.cdecl, exportc, dynlib} =
  print(createSeq(a, b).len)

proc start*() {.exportc: "_start".} =
  discard

