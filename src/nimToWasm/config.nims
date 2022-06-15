--app:lib
--os:standalone
--cpu:wasm32
--cc:clang
--noMain
--opt:size
--listCmd
--d:wasm
--d:noSignalHandler
--d:nimPreviewFloatRoundtrip
--exceptions:goto
--gc:none
--threads:off
--stackTrace:off
--lineTrace:off
--mm:none

switch("passC", "-fuse-ld=wasm-ld")
switch("passC", "--target=wasm32-unknown-unknown-wasm")
switch("passC", "-nostdinc -fno-builtin -fno-exceptions -fno-threadsafe-statics")
switch("passC", "-fvisibility=hidden -flto")
switch("passC", "-std=c99")

switch("clang.options.linker", "--target=wasm32-unknown-unknown-wasm -nostdlib -Wl,--no-entry,--allow-undefined,--export-dynamic,--gc-sections,--strip-all")
