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
      --d:lightClientWASM
      --gc:destructors
      --threads:off
      --stackTrace:off
      --lineTrace:off
      --lib:"./vendor/nim/lib"
      @if emcc:
        --d:useMalloc
        --clang.exe:emcc
        --clang.linkerexe:emcc
        --clang.cpp.exe:emcc
        --clang.cpp.linkerexe:emcc
        --passL:"-Oz -s ALLOW_MEMORY_GROWTH -s WASM=1 -s ERROR_ON_UNDEFINED_SYMBOLS=0"
      @else:
        --clang.options.linker:""
        --clang.cpp.options.linker:""
        --passC:"-w"
        --passC:"-ferror-limit=3"
        --passC:"-I${./include}"
        --passC:"-fuse-ld=wasm-ld"
        --passC:"--target=wasm32-unknown-unknown-wasm"
        --passC:"-nostdinc -fno-builtin -fno-exceptions -fno-threadsafe-statics"
        --passC:"-fvisibility=hidden -flto"
        --passC:"-std=gnu99"
        --passC:"-mbulk-memory" # prevents clang from inserting calls to `memcpy`
        --passL:"--target=wasm32-unknown-unknown-wasm -nostdlib -Wl,--no-entry,--allow-undefined,--export-dynamic,--gc-sections,--strip-all"
      @end
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
