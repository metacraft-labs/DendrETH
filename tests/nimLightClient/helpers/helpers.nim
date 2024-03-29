when defined(emcc):
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "__attribute__((used)) $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}

import light_client_utils

from ssz_serialization/codec import readSszValue
from stew/ranges/ptr_arith import makeOpenArray
import serialization/object_serialization

export object_serialization

proc allocMemory*(size: int): pointer {.wasmPragma.} =
  let res = alloc(size)
  if cast[int](res) == 0:
    quit 1
  return res

proc deserializeSSZType*[T](t: var T, memory: pointer, length: Natural) =
  readSszValue(makeOpenArray(memory, byte, length), t)

template fromSszBytes*(T: type Slot, bytes: openArray[byte]): T =
  T fromSszBytes(uint64, bytes)

