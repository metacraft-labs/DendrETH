# Tool for comparing the size of the Nim Light Client compiled to wasm with different methods

## How to use

Run the following command in this dir:

```bash
nim c -r sizeCompare.nim
```

## Example output

```
Size of Nim Light Client compiled to wasm
┌──────────────────────────────┬─────────┐
│  Method used                 │  Size   │
├──────────────────────────────┼─────────┤
│  Compiled with `clang`       │  62887  │
├──────────────────────────────┼─────────┤
│  Compiled with `emscripten`  │  58646  │
└──────────────────────────────┴─────────┘
```
