# CLI Output

This document describes NEPL CLI output options and file naming.

## Output base

`--output` is treated as a base path. Extensions are added depending on `--emit`.

Examples:
- `--output out/a` writes `out/a.wasm`, `out/a.wat`, `out/a.min.wat` (depending on `--emit`).
- `--output out/a.wasm` is treated as base `out/a`.
- `--output out/a.wat` is treated as base `out/a`.
- `--output out/a.min.wat` is treated as base `out/a`.

## Emit formats

`--emit` accepts one or more values (comma-separated or repeated):
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

## WAT generation

- Pretty WAT uses the default formatting from `wasmprinter`.
- Minified WAT compresses whitespace after printing.
