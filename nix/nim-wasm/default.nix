{
  writeTextFile,
  writeShellApplication,
  llvm,
  nim,
}: let
  nimcfg = writeTextFile {
    name = "nim-cfg";
    text = ''
      --skipCfg:on
      --app:lib
      --os:standalone
      --cpu:wasm32
      --cc:clang
      --noMain
      --opt:size
      --listCmd
      --d:nimNoLibc
      --d:wasm
      --d:noSignalHandler
      --d:nimPreviewFloatRoundtrip
      --d:lightClientEmbedded
      --exceptions:goto
      --gc:destructors
      --threads:off
      --stackTrace:off
      --lineTrace:off
      --lib:"./vendor/nim/lib"
      --clang.options.linker:""
      --clang.cpp.options.linker:""
      --passC:"-w"
      --passC:"-ferror-limit=3"
      --passC:"-I${./include}"
      --passC:"-fuse-ld=wasm-ld"
      --passC:"--target=wasm32-unknown-unknown-wasm"
      --passC:"-nostdinc -fno-builtin -fno-exceptions -fno-threadsafe-statics"
      --passC:"-fvisibility=hidden -flto"
      --passC:"-std=c99"
      --passL:"--target=wasm32-unknown-unknown-wasm -nostdlib -Wl,--no-entry,--allow-undefined,--export-dynamic,--gc-sections,--strip-all"
    '';
    destination = "/nim/nim.cfg";
  };
in
  writeShellApplication {
    name = "nim-wasm";
    runtimeInputs = [nim llvm.lld llvm.clang-unwrapped];
    text = ''
      export XDG_CONFIG_HOME="${nimcfg}"
      set -x
      nim "$@"
    '';
  }
