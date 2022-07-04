{
  writeShellApplication,
  llvm,
  nim,
}:
writeShellApplication {
  name = "nim-wasm";
  runtimeInputs = [nim llvm.lld llvm.clang-unwrapped];
  text = ''
    input_file="$1"
    output_file="''${input_file%.nim}.wasm"
    set -x
    nim c \
      --skipCfg:on \
      --app:lib \
      --os:standalone \
      --cpu:wasm32 \
      --cc:clang \
      --noMain \
      --opt:size \
      --listCmd \
      --d:wasm \
      --d:noSignalHandler \
      --d:nimPreviewFloatRoundtrip \
      --exceptions:goto \
      --gc:none \
      --threads:off \
      --stackTrace:off \
      --lineTrace:off \
      --mm:none \
      --passC:"-w" \
      --passC:"-ferror-limit=3" \
      --passC:"-I${./include}" \
      --passC:"-fuse-ld=wasm-ld" \
      --passC:"--target=wasm32-unknown-unknown-wasm" \
      --passC:"-nostdinc -fno-builtin -fno-exceptions -fno-threadsafe-statics" \
      --passC:"-fvisibility=hidden -flto" \
      --passC:"-std=c99" \
      --passL:"--target=wasm32-unknown-unknown-wasm -nostdlib -Wl,--no-entry,--allow-undefined,--export-dynamic,--gc-sections,--strip-all" \
      "-o:$output_file" \
      "$input_file"
  '';
}
