# Linking **WebAssembly object** files

Given two or more modules (written in potentially different languages), we can combine them in a single `WebAssembly` module by linking the object files produced from the compilation of the individual modules.

## Example

Let's take a look at linking two libraries - one written in Nim and one in C.

Here's [`add.nim`](./add.nim):

```nim
proc add*(a, b: int): int {.cdecl, exportc, dynlib.} =
  a + b
```

And here's the [`main.c`](./main.c) file:

```c
int add(int a, int b);
int test()
{
	return add(2, 3);
}
```

## Compiling to **WebAssembly object** files

### Compiling the `c` file

```
clang --target=wasm32-unknown-unknown-wasm  -nostdlib -c -o ./wasm-build/wasm-ccache/main.wasm main.c
```

### Compiling the `nim` file

```
nim c --noLinking --noMain --os:standalone --cpu:wasm32 --cc:clang --d:nimNoLibc --passC:"-fPIC" --passC:"--target=wasm32-unknown-unknown-wasm" --passC:"-I../../libs/nix/nim-wasm/include" --nimcache:./wasm-build/wasm-nimcache ./add.nim
```

## Linking the **WebAssembly object** files

```
wasm-ld --no-entry --export-dynamic ./wasm-build/wasm-nimcache/*.o  ./wasm-build/wasm-ccache/main.wasm -o ./wasm-build/linked-result.wasm
```

## Verify that it works

You can use tool like [wasm3](https://github.com/wasm3/wasm3) - a WebAssembly interpreter.
Run the `main` function directly:

```
wasm3 --func __main_void ./wasm-build/linked-result.wasm
```

The result should be:

> Result: 5
