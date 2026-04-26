---
id: ISS-20260426T061837095Z-WILDCARD-RESULT-I64-PATTERN-CAN-GENE-C5C0C655
title: "wildcard Result i64 pattern can generate invalid wasm"
area: compiler
status: verified
resolved: true
priority: P2
type: bug
created: 2026-04-26
updated: 2026-04-26
target: nepl-core/src/codegen_wasm.rs
---

# ISS-20260426T061837095Z-WILDCARD-RESULT-I64-PATTERN-CAN-GENE-C5C0C655: wildcard Result i64 pattern can generate invalid wasm

## 概要

`Result<i64, E>` を `Result::Ok _` で match し、payload を使わず bool / i32 を返すだけの arm を書くと、i64 payload 用 local が別 arm の i32 payload に再利用され、invalid wasm が生成される場合がある。stdlib numeric overflow の回帰テスト追加中に、`Result::Ok _` arm が単に `false` を返すだけで再現した。

## 対象

- `nepl-core/src/codegen_wasm.rs`
- `nepl-core/src/hir.rs`
- `nepl-core/src/passes/drop_insertion.rs`
- `nepl-core/src/typecheck.rs`

## 根拠

- `tests/stdlib/string_numeric_overflow.n.md` の作成中、`to_i64 "9223372036854775808"` を `match` し、`Result::Ok _:` で `false` を返す arm を置くと `invalid wasm generated: type mismatch: expected i64, found i32` で compile phase が失敗した。
- 同じ判定を `core/result.is_err<i64,i32>` に置き換えると compile/run は通るため、numeric parser の意味論ではなく wildcard pattern lowering / drop generation 側の問題と見ている。

## 問題

`Result<i64, E>` を `Result::Ok _` で match し、payload を使わず bool / i32 を返すだけの arm を書くと、wasm backend が arm 束縛 `_` を共通の外側スコープに作るため、最初の arm で確保した i64 local を後続 arm の i32 payload local として再利用していた。その結果、`local.set` の期待型と payload load の型がずれて invalid wasm になった。

## 影響

テストや利用者コードが、未使用の i64 payload を人工的に評価する workaround を必要とし得る。これは compiler の stack / drop bug を隠し、stdlib の例にも不自然な未使用値処理を増やす原因になる。

## 修正方針

pattern lowering / drop generation を修正し、非 i32 payload の wildcard pattern が payload を正しく消費し、match の各 arm が期待される stack type だけを残すようにする。

## 検証

`Result<i64,i32>` を `Result::Ok _` で match し、payload を使わず i32 / bool を返す compiler regression test を追加する。

## 対応結果

- wasm backend の match arm lowering で arm 束縛 local を arm ごとの scope に入れ、`_` など同名 bind が arm 間で異なる payload 型を持っても同じ wasm local を再利用しないようにした。
- `HirMatchArm` に `bind_ty` を追加し、drop insertion が arm body の戻り値型から payload 型を推測しないようにした。
- monomorphize で match arm の `bind_ty` も型置換するようにした。
- `tests/stdlib/string_numeric_overflow.n.md` の i64 overflow テストを `is_err` workaround から `Result::Ok _` / `Result::Err _` match へ戻した。

## 確認結果

- `cargo test -p nepl-core --test neplg2 result_i64_wildcard_match_does_not_reuse_arm_bind_local -- --nocapture`: pass
- `cargo test -p nepl-core --test neplg2 -- --nocapture`: 47/47 passed
- `cargo fmt --all --check`: pass
- `node nodesrc/issues.js check`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tests/stdlib/string_numeric_overflow.n.md --no-tree -o tmp/wildcard-result-i64-pattern-string-overflow.json -j 1`: `total=8`, `passed=8`, `failed=0`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-wildcard-result-i64-pattern.json`: 13/13 passed
