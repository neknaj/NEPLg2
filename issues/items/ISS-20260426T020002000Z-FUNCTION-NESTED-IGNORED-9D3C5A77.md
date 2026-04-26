---
id: ISS-20260426T020002000Z-FUNCTION-NESTED-IGNORED-9D3C5A77
title: "nested function regression test remains ignored even though it passes"
area: core
status: verified
resolved: true
priority: P2
type: test
created: 2026-04-26
updated: 2026-04-26
target: "nepl-core/tests/functions.rs, tests/compiler/functions.n.md"
source: doc/neplg2/pre_selfhost_audit_20260426.md
---

# ISS-20260426T020002000Z-FUNCTION-NESTED-IGNORED-9D3C5A77: nested function regression test remains ignored even though it passes

## 概要

`nepl-core/tests/functions.rs` の `function_nested` は `#[ignore]` のまま残っている。
コメントは「Nested functions are not yet fully supported in codegen」だが、現在は ignored test を明示実行すると pass する。

## 根拠

- `cargo test -p nepl-core --test functions function_nested -- --ignored` は `1 passed`。
- 同じファイルの `function_nested_capture_variable` は通常 test として実行されている。

## 問題

実装済みになった機能の regression test が ignored のままだと、将来の parser / typecheck / codegen 変更で nested function が壊れても CI が検出しない。
セルフホスト compiler は pass 内 helper を局所化する場面が多いため、nested function の安定性は実装 ergonomics に影響する。

## 影響

セルフホスト実装中に nested function を避ける不要な制約が残る。
また、実装済み機能を未実装と誤認し、設計や branch 分割が過度に保守的になる。

## 修正方針

`function_nested` の `#[ignore]` を外し、コメントを現在の仕様に合わせる。
もし target や backend によってまだ未対応の条件がある場合は、通常 pass するケースと未対応ケースを分け、未対応側だけを具体的な理由付きで ignore する。

## 対応結果

`nepl-core/tests/functions.rs` の `function_nested` から `#[ignore]` と古い未対応コメントを削除し、CI の通常実行対象へ戻した。
同じ内容を持つ `tests/compiler/functions.n.md` の `function_nested` も `neplg2:test[skip]` から通常の `neplg2:test` へ戻し、説明文を現在の挙動に合わせた。

明示 ignored 実行と通常実行の両方で pass することを確認し、nested function と capture あり nested function が同じ回帰テスト群で保護される状態にした。

## 検証

- `cargo test -p nepl-core --test functions function_nested`
- `cargo test -p nepl-core --test functions`
- 修正前確認: `cargo test -p nepl-core --test functions function_nested -- --ignored` (`1 passed`)
- `node nodesrc/tests.js -i tests/compiler/functions.n.md --no-tree -o tmp/function-nested-tests.json -j 1` (`total=22`, `passed=22`, `failed=0`)
- `node nodesrc/issues.js check`
