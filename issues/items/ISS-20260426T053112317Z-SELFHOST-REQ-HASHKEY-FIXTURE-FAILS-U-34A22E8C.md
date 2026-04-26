---
id: ISS-20260426T053112317Z-SELFHOST-REQ-HASHKEY-FIXTURE-FAILS-U-34A22E8C
title: "selfhost_req HashKey fixture loses struct keys in generic HashMap"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/src/typecheck.rs, nepl-core/src/codegen_wasm.rs, nepl-core/src/codegen_llvm.rs, nepl-core/tests/selfhost_req.rs, nepl-core/tests/neplg2.rs"
---

# ISS-20260426T053112317Z-SELFHOST-REQ-HASHKEY-FIXTURE-FAILS-U-34A22E8C: selfhost_req HashKey fixture loses struct keys in generic HashMap

## 概要

`nepl-core/tests/selfhost_req.rs::test_req_trait_extensions` が `HashMap<Point,str,DefaultHash32>` の `get` で `None` を返し、期待値 5 ではなく 0 になっていた。
fixture 自体は `#target std` を含むため Rust harness は WASI runner を使う必要があるが、WASI runner に直しても `HashMap<Point,...>` の key 書き込みが `store<i32>` として monomorphize され、struct key の実体ではなく i32 pointer 表現だけが bucket に保存されていた。

## 対象

- `nepl-core/tests/selfhost_req.rs`
- `nepl-core/tests/neplg2.rs`
- `nepl-core/src/typecheck.rs`
- `nepl-core/src/codegen_wasm.rs`
- `nepl-core/src/codegen_llvm.rs`

## 根拠

- `cargo test -p nepl-core --test selfhost_req test_req_trait_extensions` は現在の `main` 相当の `nepl-core/src/monomorphize.rs` に戻しても `left: 0, right: 5` で失敗する。
- 同 fixture は `#target std` を含むが、`run_main_i32` は `CompileTarget::Wasm` を指定している。
- `HashMap<Point,i32,DefaultHash32>` の最小回帰で、bucket の status と len は更新される一方、保存された key は `Point` ではなく pointer 由来の i32 として読み戻された。
- verbose typecheck では `store<.K> hashmap_key_ptr<.K,.V> entries size slot key` の内側 `hashmap_key_ptr` を reduce するとき、外側 `store` の次引数 `.K` と、まだ内側 call の引数である `entries: i32` を誤って unify していた。
- WAT では `insert__...Point...` 内の key 保存が `store__i32...` になり、`store__...Point` が生成されていなかった。

## 問題

`infer_expected_from_outer_consumer` は、内側 call の戻り値に外側 call の期待型を伝える際、stack 上で現在の引数より後ろに見える entry も外側 call の sibling 引数だと仮定していた。
しかし prefix call では `store<.K> hashmap_key_ptr<.K,.V> entries size slot key` のように、現在 reduce 中の内側 call の引数が stack 上で後続位置に残る。
このため `entries: i32` が外側 `store` の value 引数として扱われ、`.K` が `i32` に束縛され、`HashMap<Point,...>` の key storage が破壊された。

## 影響

user-defined struct を `HashKey` として使う self-host 要件が壊れ、HashMap / HashSet ベースの symbol table 設計が信頼できない。
また `#target std` fixture を bare wasm runner で実行していたため、stdlib/WASI 前提の selfhost_req が正しい検証 gate になっていなかった。

## 修正方針

`infer_expected_from_outer_consumer` は現在の外側引数より前に確定済みの sibling 引数だけを使って期待型を推論し、現在 reduce 中の内側 call の後続引数を外側 call の引数として unify しない。
さらに generic `load` / `store` intrinsic は注釈 type arg が未解決 TypeVar の場合、HIR 上の式型から concrete storage type を選び、aggregate storage lowering が type arg 解決漏れで i32 fallback しないようにする。
`#target std` の selfhost_req は WASI runner へ戻し、`HashMap<Point,...>` の Rust 回帰を追加する。

## 対応

- `nepl-core/src/typecheck.rs` の外側 call 期待型推論を、現在引数より前にある確定済み sibling だけで行うようにした。
- WASM / LLVM backend の `load` / `store` intrinsic lowering で、未解決 type arg を式型から補正する `intrinsic_storage_type` を追加した。
- `nepl-core/tests/neplg2.rs` に generic aggregate `load` / `store`、`HashKey` 経由の aggregate 保存、`HashMap<Point,...>` roundtrip の回帰テストを追加した。
- `nepl-core/tests/selfhost_req.rs` の `#target std` fixture を `run_main_wasi_i32` で実行するようにした。

## 検証

- `cargo test -p nepl-core --test neplg2 -- --nocapture`: 46/46 passed
- `cargo test -p nepl-core --test selfhost_req -- --nocapture`: 6/6 passed
- `cargo test -p nepl-core --test neplg2 hashmap_custom_struct_key_roundtrips_value -- --nocapture`: pass
- `cargo test -p nepl-core --test selfhost_req test_req_trait_extensions -- --nocapture`: pass
- `node nodesrc/run_doctest.js -i tests/stdlib/selfhost_req.n.md -n 6 --dist dist`: pass
- `trunk build`: pass（既存 Rust warning は残存）
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-selfhost-req-hashkey.json`: 13/13 passed
