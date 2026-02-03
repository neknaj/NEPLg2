# NEPLG2

Prefix + off-side rule language targeting WebAssembly. Everything is prefix; blocks use `:` + indentation; almost everything is an expression.

## Quick example
```neplg2
#entry main
#indent 4
#target wasi

#import "std/math"
#use std::math::*
#import "std/stdio"
#use std::stdio::*

fn main <()*> ()> ():
    let mut x <i32> 0;
    while lt x 5:
        print "count = ";
        println_i32 x;
        set x add x 1;
```

## CLI
Compile and/or run:
```bash
# run directly (no output file) targeting wasm
cargo run -p nepl-cli -- --input examples/counter.nepl --run

# write wasm and run with embedded runner
cargo run -p nepl-cli -- --input examples/counter.nepl --output target/counter --run

# compile to wasi and run with external WASI runtime
cargo run -p nepl-cli -- -i examples/rpn.nepl --run --target wasi

# choose target (wasm|wasi), default wasm
cargo run -p nepl-cli -- --input examples/counter.nepl --target wasi --output target/counter

# emit multiple outputs (wasm + pretty wat + minified wat)
cargo run -p nepl-cli -- -i examples/counter.nepl -o target/counter --emit wasm,wat,wat-min
```

Notes:
- `--output` is treated as a base path; extensions are added per `--emit`.
- `--emit` can be repeated or comma-separated; `all` expands to `wasm, wat, wat-min`.

### Running with external WASI runtimes

After compiling to WASM, you can run with `wasmtime` or `wasmer`:

```bash
# Compile to WASI binary
cargo run -p nepl-cli -- -i examples/counter.nepl -o counter --target wasi

# Run with wasmtime
wasmtime run counter.wasm

# Run with wasmer
wasmer run counter.wasm

# With stdin/stdout interaction (e.g., rpn.nepl)
cargo run -p nepl-cli -- -i examples/rpn.nepl -o rpn --target wasi
echo "3 5 +" | wasmtime run rpn.wasm
echo "3 5 +" | wasmer run rpn.wasm
```

`#entry` directive specifies which function serves as the entry point (exported as `_start` for WASI compliance).

Run stdlib tests:
```bash
cargo run -p nepl-cli -- test
```

## Imports
No built-in functions. Use std modules explicitly:
- `std/math` – i32 arithmetic/comparison (pure)
- `std/stdio` – `print` / `println` / `print_i32` / `println_i32` via WASI `fd_write`
- `std/test` – `assert` / `assert_eq_i32` / `assert_str_eq` helpers for stdlib tests

## Tests
```bash
cargo test --workspace --locked
```
