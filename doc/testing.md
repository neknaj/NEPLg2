# Testing and stdlib status

This document describes the current NEPL test feature, how standard library
tests are organized, and what is implemented in the stdlib today.

## Test execution

Stdlib tests are NEPL programs under `stdlib/tests`. Each test file is a
standalone program that must compile and exit with status 0.

Run all stdlib tests:

```
cargo run -p nepl-cli -- test
```

Enable verbose compiler logs (for debugging):

```
cargo run -p nepl-cli -- test --verbose
```

Filter by substring (path match):

```
cargo run -p nepl-cli -- test string
```

Notes:
- Tests are compiled and executed with the WASI target.
- A non-zero exit code is treated as a failure.
- The test runner loads stdlib from the repository `stdlib/` directory.
- The test runner passes fixed arguments `--flag value` (argv[1..]) to WASI programs.

## stdlib test module

The stdlib provides a small `std/test` module for assertions.

Exports:
- `assert <(bool)*()>`
- `assert_eq_i32 <(i32,i32)*()>`
- `assert_str_eq <(str,str)*()>`
- `assert_ok_i32 <(ResultI32)*()>`
- `assert_err_i32 <(ResultI32)*()>`

Failure behavior:
- On WASI targets, assertion failures print a red message and then call `trap`.
- On wasm targets, failures call `trap` without printing.

Important syntax rule:
- NEPL does not support parenthesized expressions for grouping; use prefix calls directly.
- Tuple literals are allowed using commas, e.g. `(a, b)`.

Example:

```
#import "std/test"
#use std::test::*
#import "std/math"
#use std::math::*

fn main <()*> ()> ():
    assert_eq_i32 3 add 1 2;
    assert lt 1 2;
    ()
```

Move rule reminder:
- `ResultI32` is not Copy. If a value is consumed by `unwrap_or` or similar,
  create a new value for additional checks.

## Where tests live

- Language core / compiler behavior: Rust tests under `nepl-core/tests/*.rs`
- Standard library behavior: NEPL tests under `stdlib/tests/*.nepl`

## Current stdlib scope (summary)

The current stdlib is intentionally minimal and i32-focused:

- `std/math`: i32 arithmetic and comparisons
- `std/mem`: linear memory alloc/load/store helpers
- `std/string`: length, equality, from_i32, to_i32 (ResultI32), find (stub)
- `std/result`: `ResultI32` and helpers
- `std/option`: `OptionI32` and helpers
- `std/list`: fixed-capacity list of i32 with bounds-checked get
- `std/stdio`: WASI `print`, `println`, `print_i32`, `println_i32`, `read_all`,
  `read_line`
- `std/cliarg`: WASI `args_sizes_get/args_get` argument access (`cliarg_count`,
  `cliarg_get`, `cliarg_program`)
- `std/test`: basic assertions for stdlib tests

If you extend stdlib behavior, add a matching `.nepl` test under
`stdlib/tests` and ensure `cargo run -p nepl-cli -- test` stays green.
