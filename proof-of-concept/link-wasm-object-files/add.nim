proc add*(a, b: int): int {.cdecl, exportc, dynlib.} =
  a + b
