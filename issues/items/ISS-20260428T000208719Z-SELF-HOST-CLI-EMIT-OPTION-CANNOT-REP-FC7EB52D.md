---
id: ISS-20260428T000208719Z-SELF-HOST-CLI-EMIT-OPTION-CANNOT-REP-FC7EB52D
title: "self-host CLI emit option cannot represent multiple artifacts"
area: selfhost
status: fixed
resolved: true
priority: P2
type: architecture
created: 2026-04-28
updated: 2026-04-28
target: "stdlib/neplg2/cli/args.nepl, tests/stdlib/selfhost_cliarg_parser.n.md"
---

# ISS-20260428T000208719Z-SELF-HOST-CLI-EMIT-OPTION-CANNOT-REP-FC7EB52D: self-host CLI emit option cannot represent multiple artifacts

## 概要

SelfhostCliOptions は emit を単一の SelfhostCliEmit として保持し、--emit の comma 区切り複数指定を後続 issue に回すコメントだけを持っている。現行 Rust CLI は wasm/wat/wat-min/llvm/llvm-min/all の artifact 選択と複数出力を扱う必要がある。

## 対象

- `stdlib/neplg2/cli/args.nepl, stdlib/neplg2/cli/driver.nepl`

## 根拠

- `stdlib/neplg2/cli/args.nepl` の `SelfhostCliOptions` は `emit <SelfhostCliEmit>` を単一値として持つ。
- `SelfhostCliEmit` の doc comment と `selfhost_cli_parse_emit_value` の comment は、Rust CLI の comma 区切り複数指定を後続 issue で `Vec<SelfhostCliEmit>` 化すると明記している。
- `doc/neplg2/self_host_plan.md` の S7 は artifact、diagnostic JSON、exit code 比較を成功条件としており、複数 artifact を扱う CLI 契約が必要になる。

## 問題

SelfhostCliOptions は emit を単一の SelfhostCliEmit として保持し、--emit の comma 区切り複数指定を後続 issue に回すコメントだけを持っている。現行 Rust CLI は wasm/wat/wat-min/llvm/llvm-min/all の artifact 選択と複数出力を扱う必要がある。

## 影響

S6 の CLI parity と bootstrap comparison で、同じ入力から複数 artifact と diagnostic JSON を安定して生成・比較できない。driver 実装時に単一 emit 前提を広げると、args/parser/reporter/artifact writer に横断変更が発生する。

## 修正方針

emit を Vec<SelfhostCliEmit> または small set 表現へ変更し、comma separated value を parser で分解・重複排除する。All の展開規則と output path の対応を driver/artifact writer の契約として明文化する。

## 検証

selfhost CLI args doctest と driver smoke で emit wasm,wat、emit all、重複指定、不正要素、-o との組み合わせを確認する。

## 解決

- `SelfhostCliOptions.emit` を単一 `SelfhostCliEmit` から固定 bool field の `SelfhostCliEmitSet` へ変更した。
- `--emit wasm,wat,llvm-min` の comma separated value を `selfhost_cli_parse_emit_set_value` で解析し、重複指定は同じ field を `true` にするだけの O(1) state として扱うようにした。
- `all` は parser 内で全 artifact set に展開する。空文字列、空要素、未知要素は `InvalidEmit` として返す。
- `selfhost_cli_emit_is_wasm` を emit set でも使えるよう overload し、`wat` / `wat-min` / `llvm` / `llvm-min` の確認 helper を追加した。
- 現時点では `stdlib/neplg2/cli/driver.nepl` がまだ存在しないため、driver 実装へ渡す args contract を先に複数 artifact 対応へ固定した。output path と複数 artifact の file naming は artifact writer 側の責務として `SelfhostCliEmitSet` comment に明記した。
- `tests/stdlib/selfhost_cliarg_parser.n.md` に emit list、`all` 展開、重複排除、不正空要素を確認する回帰 test を追加した。

## 検証結果

- `node nodesrc/tests.js -i tests/stdlib/selfhost_cliarg_parser.n.md --no-tree -o tmp/selfhost-cli-emit-set.json -j 1`
- `node nodesrc/tests.js -i stdlib/neplg2/cli/args.nepl --no-tree -o tmp/selfhost-cli-emit-set-docs.json -j 1`
- `node nodesrc/test_stdlib_match_decision_trees.js`
