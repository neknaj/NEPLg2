# Debug Builds

> **対象実装**: このドキュメントは現行 Bootstrap 実装（`nepl-core` / `nepl-cli`）の debug build 挙動を記述する。
> NEPLg3 文書は未確定 draft として扱い、現行 NEPLg2.1 の正仕様とは区別する。

This document describes debug-only output and profile gates.

## Profile gates

Use `#if[profile=debug]` or `#if[profile=release]` to include code only for a
given compiler profile.

The active profile is controlled by the CLI option:
- `--profile debug`
- `--profile release`

If omitted, the compiler uses the build profile it was compiled with.

## Debug output helpers

The stdlib provides debug output helpers that only emit in debug builds:
- `std/stdio::debug`
- `std/stdio::debugln`
- `std/diag::diag_debug_print`
- `std/diag::diag_debug_println`

In release builds these functions are no-ops.
