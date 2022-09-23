finalNixPkgs: previousNixPkgs: {
  emscripten-inriched-cache = previousNixPkgs.emscripten.overrideAttrs (old: {
    postInstall = ''
      pushd $TMPDIR
      echo 'int __main_argc_argv() { return 42; }' >test.c
      for MEM in "-s ALLOW_MEMORY_GROWTH" ""; do
        for LTO in -flto ""; do
          for OPT in "-O2" "-O3" "-Oz" "-Os"; do
            $out/bin/emcc $MEM $LTO $OPT -s WASM=1 -s STANDALONE_WASM test.c
          done
        done
      done
    '';
  });
}
