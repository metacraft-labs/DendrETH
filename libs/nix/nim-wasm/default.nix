{
  writeTextFile,
  writeShellApplication,
  llvm,
  nim,
}: let
  nimcfg = writeTextFile {
    name = "nim-cfg";
    text = ''
      # We allow the user to override the Nim system library being used.
      # This makes it easier to introduce patches to the system library
      # during development, which make take some time to be upstreamed:
      let localNimLib = getEnv("LOCAL_NIM_LIB")
      if localNimLib != "":
        switch("lib", localNimLib)

      # By default, the compiler will produce a wasm sitting next to the
      # compiled project main module:
      switch("out", projectName() & ".wasm")

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

      if defined(emcc):
        --d:useMalloc
        --clang.exe:emcc
        --clang.linkerexe:emcc
        --clang.cpp.exe:emcc
        --clang.cpp.linkerexe:emcc
        --passL:"-Oz -s ALLOW_MEMORY_GROWTH -s WASM=1 -s ERROR_ON_UNDEFINED_SYMBOLS=0"
      else:
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
    '';
    destination = "/nim/config.nims";
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
