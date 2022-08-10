import .../src/nim-light-client/light_client_utils

from ssz_serialization/codec import readSszValue
from stew/ranges/ptr_arith import makeOpenArray
import serialization/object_serialization

export object_serialization

proc allocMemory*(size: uint32): pointer {.cdecl, exportc, dynlib} =
  alloc(size)

proc deserializeSSZType*[T](t: var T, memory: pointer, length: Natural) =
  readSszValue(makeOpenArray(memory, byte, length), t)

template fromSszBytes*(T: type Slot, bytes: openArray[byte]): T =
  T fromSszBytes(uint64, bytes)

