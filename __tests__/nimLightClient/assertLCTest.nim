import light_client_utils

proc assertLCFailTest*(
    data: Natural
  ): void {.cdecl, exportc, dynlib} =
  assertLC(data == 0, BlockError.Invalid)
