---
id: ISS-20260513T003230273Z-EXTERN-DECLARATIONS-IGNORE-IMPORT-VI-FA18F3C6
title: "extern declarations ignore import visibility and break raw syscall facades"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-13
updated: 2026-05-13
target: "nepl-core/src/lexer.rs, nepl-core/src/ast.rs, nepl-core/src/typecheck/driver.rs, stdlib/std/fs/raw/wasi.nepl"
---

# ISS-20260513T003230273Z-EXTERN-DECLARATIONS-IGNORE-IMPORT-VI-FA18F3C6: extern declarations ignore import visibility and break raw syscall facades

## 概要

import visibility enforcement records fn/enum/struct visibility, but #extern declarations are always inserted as private bindings and the lexer rejects pub #extern. std/fs/raw/wasi is re-exported through raw facades, so wasi_path_open, wasi_fd_read, wasi_fd_write, and wasi_fd_close become undefined when used from std/fs/fd and std/fs/raw/fd_io.

## 対象

- `nepl-core/src/lexer.rs, nepl-core/src/ast.rs, nepl-core/src/typecheck/driver.rs, stdlib/std/fs/raw/wasi.nepl`

## 根拠

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_fs_and_stdio_scratch_cleanup -- --nocapture` が、ResourceIR owner check に入る前に `wasi_path_open` / `wasi_fd_read` / `wasi_fd_write` / `wasi_fd_close` の `IdentifierUndefined` で停止した。
- 該当 symbol は `stdlib/std/fs/raw/wasi.nepl` の `#extern` 由来で、`std/fs/raw` facade が `pub #import "std/fs/raw/wasi" as *` で再公開する設計になっている。
- `Binding` visibility は `fn` / `enum` / `struct` へは反映済みだったが、`Directive::Extern` には visibility field がなく、typecheck で常に `Visibility::Private` として登録されていた。

## 問題

import visibility enforcement records fn/enum/struct visibility, but #extern declarations are always inserted as private bindings and the lexer rejects pub #extern. std/fs/raw/wasi is re-exported through raw facades, so wasi_path_open, wasi_fd_read, wasi_fd_write, and wasi_fd_close become undefined when used from std/fs/fd and std/fs/raw/fd_io.

## 影響

WASI fs/stdout resource regression tests cannot typecheck, and future module boundary work would either leak raw syscall APIs by disabling visibility or break any facade that needs to re-export extern ABI symbols.

## 修正方針

Add explicit visibility to Directive::Extern, support pub #extern in the lexer/parser path, register extern bindings with that visibility, and mark only facade-exported std/fs/raw/wasi externs public while keeping unrelated syscall implementation details private.

## 修正

- `pub #extern` を lexer で `pub #import` と同じ prefix directive として受理し、`Directive::Extern` / `TokenKind::DirExtern` に `Visibility` を持たせた。
- typecheck の extern binding 登録を `Visibility::Private` 固定から directive の visibility へ変更した。
- `stdlib/std/fs/raw/wasi.nepl` の facade 経由で使う WASI ABI symbol だけを `pub #extern` にした。`std/stdio/raw` や LLVM syscall fallback の private implementation detail は公開していない。
- `import_clause` regression で、`pub #extern` は `pub #import` facade 経由で参照でき、private `#extern` は alias import から見えないことを固定した。
- この修正後、`resource_ir_owner_check_accepts_fs_and_stdio_scratch_cleanup` は undefined identifier では止まらず、既存の `ISS-20260506T172100644Z-FS-AND-STDIO-SCRATCH-RAW-DEALLOC-LOS-57895CB4` の ResourceIR owner diagnostics まで到達する。

## 検証結果

- `cargo fmt -p nepl-core --check`: passed
- `cargo check -p nepl-core --tests`: passed
- `cargo test -p nepl-core --test import_clause extern_visibility_controls_imported_abi_symbols -- --nocapture`: passed
- `cargo test -p nepl-core --test import_clause -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_accepts_fs_and_stdio_scratch_cleanup -- --nocapture`: undefined identifier は解消。残りは既存の fs/stdio scratch owner diagnostics。
