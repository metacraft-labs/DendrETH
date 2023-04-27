{
  writeTextFile,
  writeShellApplication,
  llvm,
  nim,
  emscripten,
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

      --d:release

      --skipCfg:on
      --app:lib
      --os:standalone
      --cpu:wasm32
      --cc:clang
      --noMain
      --opt:size
      --listCmd
      --d:nimNoLibc
      --d:useMalloc
      --d:wasm
      --d:noSignalHandler
      --d:nimPreviewFloatRoundtrip
      --d:lightClientEmbedded
      --gc:destructors
      --threads:off
      --stackTrace:off
      --lineTrace:off

      if defined(emcc):
        --clang.exe:emcc
        --clang.linkerexe:emcc
        --clang.cpp.exe:emcc
        --clang.cpp.linkerexe:emcc
        --passL:"-Oz -s ALLOW_MEMORY_GROWTH -s WASM=1 -s ERROR_ON_UNDEFINED_SYMBOLS=0"
      else:
        --passC:"--target=wasm32-unknown-unknown-wasm"
        --passC:"-fuse-ld=wasm-ld"

        --passC:"-std=gnu99"  # necessary beacause blst lib uses GCC-style inline assembly

        --passC:"-flto=thin"
        --passC:"-fvisibility=hidden"

        # Disable unused language features to reduce runtime overhead
        --passC:"-fno-exceptions"
        --passC:"-fno-threadsafe-statics"
        --passC:"-fno-inline-functions"

        # Path to custom libc headers to be used in place of the standard ones
        --passC:"-nostdinc"
        --passC:"-I${./include}"

        # Prevent Nim from passing additional unneeded and unsupported libraries
        --clang.options.linker:""
        --clang.cpp.options.linker:""

        # Configure warnings
        --passC:"-Werror"
        --passC:"-Wall"

        # General LLD options: https://man.archlinux.org/man/extra/lld/ld.lld.1.en
        --passL:"--target=wasm32-unknown-unknown-wasm"
        --passL:"-nostdlib"
        --passL:"-Wl,--no-entry"
        --passL:"-Wl,--strip-debug"
        --passL:"-Wl,-z,stack-size=5242880"

        # Link libraries
        --passL:"-Wl,-L${emscripten}/share/emscripten/cache/sysroot/lib/wasm32-emscripten"
        --passL:"-Wl,-lstubs,-lc"
        --passL:"-Wl,-ldlmalloc"

        # Wasm specific LLD options: https://lld.llvm.org/WebAssembly.html
        --passL:"-Wl,--export-dynamic"
        --passL:"-Wl,--import-undefined"
        --passL:"-Wl,--initial-memory=16777216"
        --passL:"-Wl,--max-memory=2147483648"
        --passL:"-Wl,--global-base=1024"

    '';
    destination = "/nim/config.nims";
  };
in
  writeShellApplication {
    name = "nim-wasm";
    runtimeInputs = [nim llvm.lld llvm.clang-unwrapped];
    text = ''
      export XDG_CONFIG_HOME="${nimcfg}"
      USE_EMCC=0
      OUTPUT_FILE=""
      for a in "$@"; do
        if [[ "$a" == "-d:emcc" ]] ; then
          USE_EMCC=1
        fi
        if [[ "$a" == "-o:"* ]] ; then
          OUTPUT_FILE="''${a##-o:}"
        fi
      done
      if [[ $USE_EMCC = 1 ]] ; then
        nim "$@"
      else
        nim "$@"
        # Run optimizations separately when compiling with clang
        wasm-opt --strip-dwarf -Oz --low-memory-unused --zero-filled-memory --strip-debug --strip-producers "$OUTPUT_FILE" -o "$OUTPUT_FILE" --mvp-features
      fi
    '';
  }
