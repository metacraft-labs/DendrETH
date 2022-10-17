import std/os
import std/terminal

import ../../vendor/nim-terminaltables/src/terminaltables

const lightClientPath = "./beacon-light-client/nim/light_client.nim"
discard execShellCmd("rm -rf ./src/measurements/build/")

const outputDir = "./src/measurements/build/"
const clangCompiledFileName = "clang_light_client.wasm"
const emccCompiledFileName = "emcc_light_client.wasm"

const compileLightClientWithClang = "nim-wasm c --lib:\"./vendor/nim/lib\" -o:" & outputDir & clangCompiledFileName & " " & lightClientPath
const compileLightClientWithEmcc = "nim-wasm c --lib:\"./vendor/nim/lib\" -d:emcc -o:" & outputDir & emccCompiledFileName & " " & lightClientPath

discard execShellCmd(compileLightClientWithClang)
discard execShellCmd(compileLightClientWithEmcc)

let results = newUnicodeTable()
results.setHeaders(@[newCell("Method used", pad = 2), newCell("Size", pad = 2)])
results.addRow(@["Compiled with `clang`", $getFileSize(outputDir & clangCompiledFileName)] )
results.addRow(@["Compiled with `emscripten`", $getFileSize(outputDir & emccCompiledFileName)])

stdout.styledWriteLine({styleBright, styleBlink},
                       "Size of Nim Light Client compiled to wasm")
printTable(results)
