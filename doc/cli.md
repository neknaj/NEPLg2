# CLI Output

> **対象実装**: このドキュメントは NEPLg2.0（現行 `nepl-core`）の CLI 仕様を記述する。
> NEPLg3 では ownership check と Resource IR analysis などの検査が追加される想定であり、ここで説明する現在の CLI 挙動とは一致しない部分がある（`doc/neplg3/impl/compiler_structure.md` 参照）。

This document describes NEPL CLI output options and file naming.

## Output base

`--output` is treated as a base path. Extensions are added depending on `--emit`.

Examples:
- `--output out/a` writes `out/a.wasm`, `out/a.wat`, `out/a.min.wat` (depending on `--emit`).
- `--output out/a.wasm` is treated as base `out/a`.
- `--output out/a.wat` is treated as base `out/a`.
- `--output out/a.min.wat` is treated as base `out/a`.

## Targets

`--target` selects the compilation target (default: `wasm`):

| Target | Description |
|--------|-------------|
| `wasm` | Pure WebAssembly (no WASI imports) |
| `wasi` | WebAssembly with WASI syscalls |
| `llvm` | Native binary via LLVM |

The current compiler keeps typecheck / effect-check behavior consistent across targets. Only the physical layout and allocator differ (absorbed by `#if[target="..."]` in stdlib). NEPLg3 is planned to add ownership check and Resource IR analysis uniformly across all targets.

## Emit formats

`--emit` accepts one or more values (comma-separated or repeated). Applies to Wasm targets:
- `wasm` outputs the binary `.wasm`.
- `wat` outputs a readable WAT.
- `wat-min` outputs a minified WAT.
- `all` expands to `wasm`, `wat`, `wat-min`.

Examples:
```
nepl-cli --input examples/counter.nepl --output target/counter --emit wasm
nepl-cli --input examples/counter.nepl --output target/counter --emit wat
nepl-cli --input examples/counter.nepl --output target/counter --emit wasm,wat,wat-min
```

## Profile

`--profile` controls `#if[profile=...]` gates in source:
- `--profile debug` enables `#if[profile=debug]`
- `--profile release` enables `#if[profile=release]`

If omitted, the compiler uses the build profile it was compiled with.

## Run and program arguments

When `--run` is used, arguments after `--` are passed to the WASI program.
The program name is always provided as `argv[0]` (input path or `<stdin>`).

Example:
```
nepl-cli --input examples/counter.nepl --run -- --flag value
```

## WAT generation

- Pretty WAT uses the default formatting from `wasmprinter`.
- Minified WAT compresses whitespace after printing.
