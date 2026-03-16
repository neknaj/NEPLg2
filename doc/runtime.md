# Runtime and Memory Model

This document summarizes how NEPLG2 targets runtime environments and manages
memory without GC.

## Targets & Runtime Differences

- **wasm**: pure WebAssembly without WASI imports. Intended to run on any conforming WASM runtime.
- **wasi**: WebAssembly with WASI syscalls. Intended to run on any WASI runtime.
- **llvm**: Native binary compilation via LLVM (POSIX environments).

According to the new specifications (`purity_ownership_memory_spec.md`), the compiler's safety semantics (ownership, borrowing, memory bounds) are completely unified across targets. Target-specific differences (such as pointer representations, allocator implementations, and system calls) are absorbed in the NEPL source code using `#if[target=...]` conditional compilation rather than compiler-internal branching.

## Two-Tier Memory Management (No GC)

NEPLg2 strictly avoids Garbage Collection. Instead, memory is managed via a combination of two strict models, enforced statically by the compiler:

1. **Region Inference (Pure Persistent Values)**
   - Used for `str` (immutable UTF-8 strings), `List .T`, and immutable trees.
   - The compiler infers allocation scopes and inserts batch-free operations (`region_free_all`) upon exiting the determined region.
   - Safe to share freely without ownership constraints.

2. **Drop Elaboration (Owned & Linear Resources)**
   - Used for `OwnedBuf<T>`, `ByteBuf`, `StringBuilder`, `File`, `Socket`, etc.
   - Operated under strict Resource IR move semantics.
   - The compiler automatically inserts `drop` calls at scope exits or overwrites for non-moved resources.

The raw explicit `alloc` and `dealloc` operations belong to the unsafe/internal layer (`core/mem`) and are wrapped securely behind these abstractions.
