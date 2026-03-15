# Error and Diagnostic Model

This document describes the standard error types and reporting utilities used
by NEPLg2. The goal is a consistent, structurally rich diagnostic flow that works safely across all targets without GC.

## Core Types

NEPLg2 standardizes error handling around three core concepts (detailed in `stdlib_breaking_reboot.md`):

- **`Result<T, StdErrorKind>`**: Used for lightweight, expected failures without rich diagnostics.
- **`Diag`**: A structurally rich diagnostic value containing `kind` (Log/Info/Warn/Error), `message`, `span`, `notes`, `help`, and `source`.
- **`Outcome<T, E>`**: A combination of a `Result` and a collection of `Diags`. Used when warnings or detailed errors must be carried alongside the result.

## Source Locations

`callsite_span` is an intrinsic that returns a `Span` for the current call site.
Helpers attach this span automatically to `Diag` instances.

## Reporting

- Ecosystem-wide standard: `Diag` is not just for stdlib errors but for compiler, tooling, and DSL diagnostics.
- Reporting/formatting is decoupled from the `Diag` data structure itself and handled by the `Stringify` / `Debug` / `Serialize` traits and renderer tools.

## Memory Management / No GC

All diagnostic values are explicit. There is no hidden global error state.
Under the new memory safety model (`purity_ownership_memory_spec.md`), `Diag` and `Outcome` structures are managed as standard values. Depending on their usage, their allocation scopes are automatically handled by **Region Inference** (if pure and persistent) or **Drop Elaboration**, without needing explicit manual heap allocations or GC.
