when defined(emcc):
  {.emit: "#include <emscripten.h>".}
  {.pragma: wasmPragma, cdecl, exportc, dynlib, codegenDecl: "EMSCRIPTEN_KEEPALIVE $# $#$#".}
else:
  {.pragma: wasmPragma, cdecl, exportc, dynlib.}

import /home/Emil/code/repos/metacraft-labs/DendrETH/src/nim-light-client/light_client_utils

from ssz_serialization/codec import readSszValue
from stew/ranges/ptr_arith import makeOpenArray
import serialization/object_serialization

export object_serialization

proc allocMemory*(size: int): pointer {.wasmPragma.} =
  alloc(size)

proc deserializeSSZType*[T](t: var T, memory: pointer, length: Natural) =
  readSszValue(makeOpenArray(memory, byte, length), t)

template fromSszBytes*(T: type Slot, bytes: openArray[byte]): T =
  T fromSszBytes(uint64, bytes)

