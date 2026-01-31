# Runtime and Memory Model

This document summarizes how NEPLG2 targets runtime environments and manages
memory without GC.

## Targets

- **wasm**: pure WebAssembly without WASI imports. Intended to run on any
  conforming WASM runtime.
- **wasi**: WebAssembly with WASI syscalls. Intended to run on any WASI runtime.

The `#if[target=wasm]` gate is always allowed; `#if[target=wasi]` requires the
WASI target.

## Allocator (no GC)

NEPL uses explicit allocation and deallocation. The allocator is implemented
inside the wasm module so the compiled output does not require host-provided
imports.

### Linear memory layout

- `memory[0..4)`: `heap_ptr` (u32)
- `memory[4..8)`: `free_list_head` (u32)

### Block layout

Each allocated block has an 8-byte header:

- `u32 size`
- `u32 next`

The payload starts immediately after the header.

### Behavior

- `alloc(size)` returns a pointer to the payload.
- `dealloc(ptr, size)` returns a block to the free list.
- `realloc(ptr, old_size, new_size)` allocates a new block, copies bytes, and
  frees the old one.

The allocator grows memory using `memory.grow` when required. On failure, it
returns `0`.

## Ownership direction

NEPL is moving toward Rust-like ownership. The current stdlib APIs remain
explicit (`alloc`/`dealloc`) and are designed to be wrapped by owned container
types (e.g., `Vec`, `List`, `String`) that free their resources when dropped.
