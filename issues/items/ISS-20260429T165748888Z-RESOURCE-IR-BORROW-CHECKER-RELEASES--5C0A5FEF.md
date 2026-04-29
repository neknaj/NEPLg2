---
id: ISS-20260429T165748888Z-RESOURCE-IR-BORROW-CHECKER-RELEASES--5C0A5FEF
title: "Resource IR borrow checker releases local call argument tokens"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-29
updated: 2026-04-30
target: "nepl-core/src/resource/borrow_check.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260429T165748888Z-RESOURCE-IR-BORROW-CHECKER-RELEASES--5C0A5FEF: Resource IR borrow checker releases local call argument tokens

## 概要

ResourceOp::Call and IndirectCall currently release every argument borrow token after return-token propagation. A Resource IR producer or transform that passes a Local borrow token directly as a call argument can lose the live local borrow and miss a later Assign conflict.

## 対象

- `nepl-core/src/resource/borrow_check.rs, nepl-core/tests/resource_ir.rs`

## 根拠

- `ResourceOp::Call` / `ResourceOp::IndirectCall` は return-token propagation の後に全 `args` へ `release_token` を呼んでいた。
- これは lowering が call 引数を temporary として materialize している現状では drop overwrite の false positive を直すが、Resource IR model 上は `args` に `Local` place も入れられる。
- `Local` borrow token を直接 call 引数にした Resource IR では、call 後も local reference が生存しているのに borrow source が `Released` になり、後続の `Assign` conflict を見逃す。

## 問題

ResourceOp::Call and IndirectCall currently release every argument borrow token after return-token propagation. A Resource IR producer or transform that passes a Local borrow token directly as a call argument can lose the live local borrow and miss a later Assign conflict.

## 影響

This can make the Resource IR borrow checker unsound for non-temporary call argument places and weakens the memory-safety gate if lowering or future Resource IR transforms stop materializing all call arguments as temporaries.

## 修正方針

Release only call argument borrow tokens whose place root is Temporary after return-token propagation. Keep Local or projected non-temporary borrow tokens live, and add Resource IR regression tests for local call argument tokens.

## 修正内容

- `release_call_argument_tokens` を `release_call_temporary_argument_tokens` に変更し、call 後に解放する borrow token を `PlaceRoot::Temporary` の引数 place に限定した。
- return-token propagation は従来どおり先に行うため、`borrow_id(&x)` のように返り値へ移った token は output 側で生存し続ける。
- `Local` borrow token を直接 call 引数に渡した Resource IR で、call 後の `Assign` が borrow conflict として残る回帰テストを追加した。

## 検証

cargo test -p nepl-core --test resource_ir resource_ir_borrow_check -- --nocapture; cargo check -p nepl-core; trunk build; node nodesrc/tests.js -i tests/compiler/drop_overwrite.n.md --no-tree -o tmp/drop-overwrite-borrow-local-scope.json -j 1 --dist web/dist

- `cargo check -p nepl-core`: pass
- `cargo test -p nepl-core --test resource_ir resource_ir_borrow_check -- --nocapture`: 16 passed
- `cargo test -p nepl-core --test drop_overwrite -- --nocapture`: 1 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/compiler/drop_overwrite.n.md --no-tree -o tmp/drop-overwrite-borrow-local-scope.json -j 1 --dist web/dist`: total=1, passed=1
- `node nodesrc/tests.js -i tests/compiler --no-tree -o tmp/compiler-after-call-borrow-local-scope.json -j 4 --dist web/dist`: total=649, passed=636, failed=13。残りは既存の ResourceIR owner obligation 系。
