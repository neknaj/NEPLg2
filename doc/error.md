# Error Model

This document describes the standard error types and reporting utilities used
by NEPLG2. The goal is a consistent, Result-first error flow that works in both
WASM and WASI targets without relying on GC.

## Core Types

`std/error` defines:

- `ErrorKind`: classification (Failure, IoError, ParseError, etc.).
- `Span`: `(file_id, start, end)` byte range.
- `Error`: heap-backed record referenced by a pointer (`{kind,msg,span}` inline).

Errors are values carried through `Result<T, Error>`. Source chaining is not
implemented yet; context is represented by creating a new `Error`.

## Source Locations

`callsite_span` is an intrinsic that returns a `Span` for the current call site.
Helpers like `fail` and `context` attach this span automatically.

## Reporting

`std/diag` provides:

- `diag_to_string(e) -> str`: build a human-readable report string.
- `diag_print(e)` / `diag_println(e)` (WASI only): print via stdio.

On the WASM target, diagnostics are returned as strings (no I/O). On WASI,
diagnostics can be printed to stdout/stderr via `std/stdio`.

## Ownership / No GC

All error values are explicit. There is no hidden global error state. Error
records live in the heap via `std/mem::alloc` and do not rely on GC.
