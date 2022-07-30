func sumOfArrayElements*(arr: array[5, int]): int {.cdecl, exportc, dynlib} =
  var sum: int
  for val in arr:
    sum += val
  sum

func createNewArray*(el: int): array[5, int] {.cdecl, exportc, dynlib} =
  var res: array[5, int]
  for i in 0..4:
    res[i] = el
  res

func arrayMapAdd*(arr: array[5, int],
                  value: int): array[5, int] {.cdecl, exportc, dynlib} =
  var res: array[5, int]
  for i in 0..4:
    res[i] = arr[i] + value
  res

proc start*() {.exportc: "_start".} =
  discard
