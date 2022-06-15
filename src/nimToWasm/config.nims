# --os:linux
--os:standalone
--cpu:wasm32
--cc:clang
--d:release
--noMain
--opt:size
--listCmd
--d:wasm
--d:noSignalHandler
--d:nimPreviewFloatRoundtrip # Avoid using sprintf as it's not available in wasm
--exceptions:goto
--app:lib
--gc:none
--threads:off
--stackTrace:off
--lineTrace:off
--mm:none

switch("passC", "-nostdinc -fuse-ld=wasm-ld -DNDEBUG -O -std=c99 -D__EMSCRIPTEN__ -D_LIBCPP_ABI_VERSION=2 -fvisibility=hidden -fno-builtin -fno-exceptions -fno-threadsafe-statics -flto -I/home/Emil/code/repos/DendrETH/src/nimToWasm")
switch("passL", "--target=wasm32-unknown-unknown-wasm ")
let llTarget = "wasm32-unknown-unknown-wasm"

switch("passC", "--target=" & llTarget)

var linkerOptions = " -nostdlib -Wl,--no-entry,--allow-undefined,--export-dynamic,--gc-sections,--strip-all"

switch("clang.options.linker", linkerOptions)
switch("clang.cpp.options.linker", linkerOptions)