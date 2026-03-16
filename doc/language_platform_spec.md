# NEPLg2.1 Language Platform Specification v1.0

NEPLg2.1 is defined as a **Language Platform**, a foundation for building and processing various languages and DSLs (JSON, Markdown, CSV, custom DSLs).

## 1. Vision and Goals

The platform's primary goal is to provide a unified infrastructure for:
- Lexing/Parsing (Toolkit-based).
- Transformation and Formatting.
- Integrated Language Islands (Embedded DSLs).
- Strong static verification.

## 2. Two-Layer Architecture

1.  **Bootstrap Host (Rust)**:
    - Located in `/nepl-core`.
    - Responsible for the bootstrap compiler pipeline.
    - Provides targets, backend (Wasm/LLVM), and minimal runtime.
2.  **Platform Stdlib (NEPL)**:
    - Located in `/stdlib`.
    - The actual platform implementation.
    - Contains parser toolkits, DSL implementations, and the **Self-host Compiler** (`stdlib/neplg2`).

## 3. Layered Standard Library Structure

The stdlib is organized into distinct layers to isolate dependencies:

1.  **core (`stdlib/core`)**: Zero-heap, target-independent foundation (Spans, Diags, Outcomes).
2.  **alloc (`stdlib/alloc`)**: Heap-dependent, target-independent utilities (Collections, String processing).
3.  **runtimes (`stdlib/runtimes`)**: Target-specific adapters (WASI, Native).
4.  **std (`stdlib/std`)**: The primary safe facade for users (I/O, FS).
5.  **features (`stdlib/features`)**: High-level, specialized features (TUI, Language processors for JSON/MD).

## 4. Platform Libraries

The platform must provide:
- **Syntax Processing**: PEG runtime, layout-aware parsers, streaming support.
- **Document Processing**: Unified models for JSON, XML, HTML, Markdown, LaTeX.
- **Language Islands**: First-class support for embedding one language within another.
- **Query & Transform**: Selector-based querying and structured rewriting.

## 5. Self-hosting and Stability

The ultimate goal is for the NEPLg2 compiler to be written in NEPLg2 (`stdlib/neplg2`).
**Bootstrap Compatibility** must be maintained:
1. Rust host builds `stdlib`.
2. Rust host builds self-host compiler.
3. Self-host compiler builds itself and `stdlib`.
