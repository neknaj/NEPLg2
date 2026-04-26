---
id: ISS-20260426T162754192Z-DEREF-OF-NON-COPY-REFERENCE-CAN-SHAL-E939B2BB
title: "deref of non-Copy reference can shallow-copy owned aggregate values"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src/passes/move_check.rs, nepl-core/src/types.rs, nepl-core/tests/move_check.rs, tests/compiler/move_check.n.md, tests/compiler/move_effect.n.md, stdlib/core/option.nepl, stdlib/alloc/collections/vec.nepl, stdlib/alloc/collections/stack.nepl, stdlib/neplg2/cli/args.nepl, stdlib/tests/option.n.md"
---

# ISS-20260426T162754192Z-DEREF-OF-NON-COPY-REFERENCE-CAN-SHAL-E939B2BB: deref of non-Copy reference can shallow-copy owned aggregate values

## 概要

`HirExprKind::Deref` は aggregate dereference を、参照先 address から新しい storage へ byte-copy する形で lower している。move checker は参照を含む返り値の borrow origin は伝播するが、shared reference から value を作る deref に `Copy` を要求していない。そのため `*field::get_ref &owner "owned"` のような式で、元 owner を生かしたまま owned non-Copy field の shallow copy を作れる可能性がある。

## 対象

- `nepl-core/src/passes/move_check.rs, nepl-core/src/types.rs, nepl-core/tests/move_check.rs, tests/compiler/move_check.n.md, tests/compiler/move_effect.n.md, stdlib/core/option.nepl, stdlib/alloc/collections/vec.nepl, stdlib/alloc/collections/stack.nepl, stdlib/neplg2/cli/args.nepl, stdlib/tests/option.n.md`

## 根拠

- `nepl-core/src/codegen_wasm.rs` の `HirExprKind::Deref` は aggregate storage type に対して `emit_alloc_call` 後、1 byte ずつ参照元から destination へ copy している。
- `nepl-core/src/passes/move_check.rs` の `HirExprKind::Deref` は返り値が reference を含む場合の borrow origin escape は見るが、value-producing deref 自体が `Copy` 可能かを検査していない。
- `ISS-20260426T142242010Z-BORROWED-FIELD-PROJECTION-API-MISSIN-3010781E` の `get_ref` 実装中に、field reference 自体は load/copy しない一方、呼び出し側が `*ref` すると既存 deref semantics で non-Copy aggregate を materialize できる経路が残ることを確認した。

## 問題

shared reference の deref が、Copy read と non-Copy move-out を区別せずに値を作る。`&T` は所有権を持たない借用であり、そこから non-Copy `T` を値として取り出すには、明示 clone、exclusive place からの move-out、または borrowed pattern matching / borrowed projection のいずれかが必要になる。現状はこの区別がなく、borrow/lifetime 検査が「参照は安全」という形だけになり得る。

## 影響

Borrow/lifetime checks can be bypassed by turning a shared reference into a shallow owned aggregate copy. For owning structs, enums, or buffers this can create aliasing, double-free paths, or use-after-free style behavior, and it makes get_ref look safe while callers can still copy the referenced value with *.

## 修正方針

Separate copy dereference from move-out semantics. Dereferencing a shared reference to produce a value should require Copy, while non-Copy values need explicit clone, owned move from an exclusive place, or borrowed pattern matching/projection. Add diagnostics before codegen and keep byte-copy lowering only for Copy values or a future explicit move-out operation.

## 検証

Add compiler tests where *&non_copy and *field::get_ref of a non-Copy field are compile_fail, while *&i32 and get_ref of Copy scalar fields continue to compile. Add stdlib regressions for self-host option/enum field inspection once borrowed enum matching or Copy impls are designed.

## 解決内容

- move checker の `HirExprKind::Deref` で、value-producing deref の結果型が `Copy` でない場合は `D3051 TypeMoveFromSharedBorrowedValue` を出すようにした。
- `*&i32` や Copy impl 済み aggregate の clone/copy は許可し、`*&non_copy` と `*field::get_ref &owner "non_copy_field"` は拒否する回帰テストを追加した。
- `TypeCtx::is_copy_eligible_inner` が unbound type variable の `copy_cap` を見るようにし、`impl<.T: Copy> Copy for Option<.T>` のような正しい generic Copy impl を受理できるようにした。
- `Option<.T: Copy>` に `Clone` / `Copy` impl を追加し、`Option<i32>` / `Option<str>` などの軽量値を shared reference から安全に読めるようにした。
- `Vec` / `Stack` の ref helper は `*v` / `*stk` による owner header の shallow copy をやめ、`field::get_ref` で Copy field だけを読む形にした。
- self-host CLI args の単純 enum (`SelfhostCliTarget` / `SelfhostCliEmit` / `SelfhostCliProfile`) を `Copy` として明示し、`Option<enum>` field の観察が新しい deref 制約下でも成立するようにした。

## 検証結果

- `cargo fmt --all --check`: pass
- `cargo check --workspace`: pass
- `cargo test -p nepl-core --test move_check -- --nocapture`: 30/30 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/move_check.n.md -i tests/compiler/move_effect.n.md --no-tree -o tmp/non-copy-deref-compiler-after-09e9533.json -j 1`: 58/58 passed
- `node nodesrc/tests.js -i stdlib/core/option.nepl -i stdlib/tests/option.n.md --no-tree -o tmp/non-copy-deref-option-after-09e9533.json -j 1`: 4/4 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/vec.nepl -i stdlib/tests/vec.n.md --no-tree -o tmp/non-copy-deref-vec-after-09e9533.json -j 1`: 41/41 passed
- `node nodesrc/tests.js -i stdlib/alloc/collections/stack.nepl -i stdlib/tests/stack.n.md -i tests/stdlib/stack_collections.n.md --no-tree -o tmp/non-copy-deref-stack-after-09e9533.json -j 1`: 32/32 passed
- `node nodesrc/tests.js -i tests/stdlib/selfhost_cliarg_parser.n.md -i stdlib/neplg2/cli/args.nepl --no-tree -o tmp/non-copy-deref-selfhost-cliargs-after-09e9533.json -j 1`: 10/10 passed
- `node nodesrc/tests.js -i stdlib/neplg2 --no-tree -o tmp/non-copy-deref-neplg2-after-09e9533.json -j 2`: 22/22 passed
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-non-copy-deref-after-09e9533.json`: 13/13 passed
- `node nodesrc/tests.js -i stdlib --no-tree -o tmp/non-copy-deref-stdlib-full-after-09e9533.json -j 4`: 411/411 passed
