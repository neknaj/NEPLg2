---
id: ISS-20260425T000000Z-RV-EXAMPLE-003-AEE7BF2B
title: "legacy RPN example が raw memory と typo 名に依存している"
area: examples
status: verified
resolved: true
priority: P1
type: architecture
created: 2026-04-25
updated: 2026-04-26
target: "examples/rpn_legacy.nepl, doc/examples.md"
legacy_id: RV-EXAMPLE-003
source: "doc/review20260425/examples.md#rv-example-003"
---

# RV-EXAMPLE-003: legacy RPN example が raw memory と typo 名に依存している

旧 `doc/review20260425` から移行した Issue。新しい正の ID は `ISS-20260425T000000Z-RV-EXAMPLE-003-AEE7BF2B`。

## 要約

互換用の小さな RPN REPL が、現在推奨したい stdlib public API ではなく raw memory と内部 layout を見せる例になっていました。さらに typo を含むファイル名が doc と example 一覧に残り、利用者へ古い名称を案内していました。

## 影響

example が「低レベルメモリ操作を使わず stdlib を活用する」という方針に反します。文字列や stack の内部 layout が変わると壊れ、`read_line` の戻り値を呼び出し側が raw dealloc するという誤った利用方法も広めます。

## 修正方針

ファイル名を `rpn_legacy.nepl` へ改め、処理は `str_trim` / `str_split` / `to_i32` / `Stack` / `Vec` の public API に寄せます。stack の更新は `pop_ref` と `push` に限定し、raw allocation/load/store は使いません。

## 検証

確認済み:

## 旧レビュー本文

- 解決済: true
- 状態: verified
- 優先度: P1
- 種別: architecture
- 対象: `examples/rpn_legacy.nepl`, `doc/examples.md`

### 根拠

- `examples/rpn_regacy.nepl`: ファイル名と文書名が `legacy` ではなく `regacy` になっていた。
- `examples/rpn_regacy.nepl`: `core/mem` を import し、入力文字列の length / byte と stack 配列を `load_i32` / `load_u8` / `store_i32` で直接操作していた。
- `examples/rpn_regacy.nepl`: `read_line` の内部 buffer を `dealloc_raw input 1028` で明示解放しており、stdlib の文字列所有権境界を example 側で仮定していた。

### 問題

互換用の小さな RPN REPL が、現在推奨したい stdlib public API ではなく raw memory と内部 layout を見せる例になっていました。さらに typo を含むファイル名が doc と example 一覧に残り、利用者へ古い名称を案内していました。

### 影響

example が「低レベルメモリ操作を使わず stdlib を活用する」という方針に反します。文字列や stack の内部 layout が変わると壊れ、`read_line` の戻り値を呼び出し側が raw dealloc するという誤った利用方法も広めます。

### 修正方針

ファイル名を `rpn_legacy.nepl` へ改め、処理は `str_trim` / `str_split` / `to_i32` / `Stack` / `Vec` の public API に寄せます。stack の更新は `pop_ref` と `push` に限定し、raw allocation/load/store は使いません。

### 対応結果

`examples/rpn_regacy.nepl` を `examples/rpn_legacy.nepl` に rename し、実装を stdlib API ベースで書き直しました。`doc/examples.md` の収録一覧も新しいファイル名へ更新しました。

### 検証

確認済み:

- `node nodesrc/tests.js -i examples/rpn_legacy.nepl --no-tree -o tmp/rpn-legacy-example-tests.json -j 2` (`total=1`, `passed=1`, `failed=0`)
- `node nodesrc/tests.js -i examples --no-tree -o tmp/examples-after-rpn-legacy.json -j 4` (`total=12`, `passed=12`, `failed=0`)
