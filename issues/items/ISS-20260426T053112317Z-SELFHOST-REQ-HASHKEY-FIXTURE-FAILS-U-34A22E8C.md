---
id: ISS-20260426T053112317Z-SELFHOST-REQ-HASHKEY-FIXTURE-FAILS-U-34A22E8C
title: "selfhost_req HashKey fixture loses struct keys in generic HashMap"
area: core
status: verified
resolved: true
priority: P1
type: bug
created: 2026-04-26
updated: 2026-04-27
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

## CI 再発確認 2026-04-27

GitHub Actions run `24967172989` の `rust-test` で、この issue の回帰テスト群が再び失敗している。

- `nepl-core/tests/neplg2.rs::generic_hashkey_eq_after_load_uses_concrete_impl`
- `nepl-core/tests/neplg2.rs::generic_hashkey_value_survives_hash_before_store`
- `nepl-core/tests/neplg2.rs::generic_store_after_generic_trait_probe_preserves_struct`
- `nepl-core/tests/neplg2.rs::hashmap_custom_struct_key_roundtrips_value`
- `nepl-core/tests/selfhost_req.rs::test_req_trait_extensions`

今回の直接の失敗内容は、前回の runtime key storage 破壊ではなく、`HashKey` から旧 `clone` method / 独自 copy capability を外した後も Rust fixture 側が `impl HashKey for Point` に `fn clone` を残し、さらに `HashMap<Point,...>` に必要な `Copy` bound を満たしていないことによる compile failure。

代表診断:

- `TypeImplMethodNotFoundInTrait`: `method 'clone' not found in trait 'HashKey'`
- `TypeTraitBoundUnsatisfied`: `type does not satisfy trait bound 'Copy'`

`ISS-20260425T000000Z-RV-STDLIB-012-C31422D8` の HashKey/Hasher cleanup 後に、`nepl-core/tests/neplg2.rs` と `nepl-core/tests/selfhost_req.rs` の fixture 更新が不足した再発として扱う。修正時は fixture を表面だけ消すのではなく、self-host 要件として `HashKey` と `Copy` / `Clone` の責務分離が正しく表現されていることを確認する。

## 再発修正 2026-04-27

Rust 側 fixture の `impl HashKey for Point` から旧 `fn clone <(Point)->Point>` を削除し、`HashKey` は `eq` / `hash32` のみを持つ現在の trait 仕様へ合わせた。
`Point` は `HashMap` / generic probe の key として値を複数回使う fixture なので、`core/traits/copy` を import し、標準 `Clone` / `Copy` を別 impl として明示した。
また、`hash_then_store` と `write_after_probe` のように `hashkey_hash32` / `hashkey_eq` の後で同じ key を保存する generic helper には `.T: HashKey&Copy` bound を明記し、HashKey が copy capability を含むかのような旧前提を残さない形にした。

検証:

- `cargo test -p nepl-core --test neplg2 hashkey -- --nocapture`: 2/2 passed
- `cargo test -p nepl-core --test neplg2 hashmap_custom_struct_key_roundtrips_value -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 generic_store_after_generic_trait_probe_preserves_struct -- --nocapture`: pass
- `cargo test -p nepl-core --test selfhost_req test_req_trait_extensions -- --nocapture`: pass
- `cargo fmt --all --check`: pass
- `cargo test -p nepl-core --test selfhost_req -- --nocapture`: 6/6 passed
- `cargo test -p nepl-core --test neplg2 -- --nocapture`: 55/55 passed
- `node nodesrc/tests.js -i tests/stdlib/selfhost_req.n.md --no-tree -o tmp/selfhost-req-hashkey-copy-fixture.json -j 1`: 6/6 passed
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/stdlib/selfhost_req.n.md --no-tree -o tmp/selfhost-req-hashkey-copy-fixture-after-trunk.json -j 1`: 6/6 passed
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-hashkey-copy-fixture.json`: 13/13 passed
- `node nodesrc/issues.js check`: pass
- `git diff --check`: pass
