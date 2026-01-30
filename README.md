# NEPLG2

Prefix + off-side rule language targeting WebAssembly. Everything is prefix; blocks use `:` + indentation; almost everything is an expression.

## Quick example
```neplg2
#entry main
#indent 4

#import "std/math"
#use std::math::*
#import "std/stdio"
#use std::stdio::*

fn main <()*>()> ():
    let mut x <i32> 0;
    while lt x 5:
        print "count = ";
        print_i32 x;
        set x add x 1;
    ()
```

## CLI
Compile and/or run:
```bash
# run directly (no output file) targeting wasm
cargo run -p nepl-cli -- --input examples/counter.nepl --run

# write wasm and run
cargo run -p nepl-cli -- --input examples/counter.nepl --output target/counter.wasm --run

# choose target (wasm|wasi), default wasm
cargo run -p nepl-cli -- --input examples/counter.nepl --target wasi --output target/counter.wasm
```
`--run` with `--target wasi` is not supported in the embedded runner; use an external WASI runtime.

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
