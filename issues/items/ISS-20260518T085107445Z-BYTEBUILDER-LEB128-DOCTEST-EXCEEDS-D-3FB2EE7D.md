---
id: ISS-20260518T085107445Z-BYTEBUILDER-LEB128-DOCTEST-EXCEEDS-D-3FB2EE7D
title: "ByteBuilder LEB128 doctest exceeds default static-check timeout"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-18
updated: 2026-05-18
target: "tests/stdlib/byte_builder.n.md, nepl-core/src/resource/initialized_alias_flow.rs"
---

# ISS-20260518T085107445Z-BYTEBUILDER-LEB128-DOCTEST-EXCEEDS-D-3FB2EE7D: ByteBuilder LEB128 doctest exceeds default static-check timeout

## 概要

tests/stdlib/byte_builder.n.md::doctest#2 times out during compile at the default 60000ms limit on origin/main and on the ByteBufStorage branch. The failure is in compilation/static checking, not runtime execution. The doctest is small, so this suggests Resource IR/type/effect summary cost around ByteBuilder owner-returning paths is still excessive.

## 対象

- `tests/stdlib/byte_builder.n.md, nepl-core/src`

## 根拠

- `work/collection-drop-contract` 上で `node nodesrc/tests.js -i tests/stdlib/byte_builder.n.md -i tests/stdlib/text_utf8.n.md --no-tree -o tmp/agent1-bytebuf-storage-state-builder-text.json -j 1 --dist web/dist --assert-io` を実行すると、`tests\stdlib\byte_builder.n.md::doctest#2` が compile phase で `wasm test case timeout after 60000ms` になった。
- 同じ run では `byte_builder.n.md::doctest#1` と `#3`、および `text_utf8.n.md` の 9 件は通過しており、LEB128 known vector fixture だけが既定 timeout を超えている。
- `origin/main` の detached worktree `C:\neknaj\neplg2_1_baseline` でも `node nodesrc/tests.js -i tests/stdlib/byte_builder.n.md --no-tree -o tmp/baseline-byte-builder.json -j 1 --dist C:\neknaj\neplg2_1\web\dist --assert-io` が同じ doctest#2 timeout になった。したがって今回の `ByteBufStorage` 化による新規退行ではない。
- 300000ms 枠の単体確認も完了しなかったため、単なる 60 秒閾値の揺れではなく、静的検査の計算量か要約展開の問題として扱う。
- これは [静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md) Stage 6 の compiler/static-check performance 残件であり、timeout 値の引き上げだけで解決扱いにしない。

## 問題

tests/stdlib/byte_builder.n.md::doctest#2 times out during compile at the default 60000ms limit on origin/main and on the ByteBufStorage branch. The failure is in compilation/static checking, not runtime execution. The doctest is small, so this suggests Resource IR/type/effect summary cost around ByteBuilder owner-returning paths is still excessive.

## 影響

Full stdlib doctest runs can fail or stall even when ByteBuf/ByteBuilder semantics are correct. Raising the timeout would hide a compiler/static-check performance problem and does not explain whether the algorithmic cost is due to owner aggregate expansion, function summary recursion, or doctest fixture structure.

## 修正方針

Profile the failing doctest with compile-stage timings and Resource IR counters. Determine whether owner-backed aggregate summary expansion for ByteBuilder/ByteBuf, LEB128 recursion, or std/test report construction dominates. Fix the compiler/static-check algorithm or split only if the fixture is intrinsically too large; do not weaken static checks or use stdlib allowlists.

## 検証

2026-05-18 Agent 1:

- 根本原因は `byte_builder_push_leb_u32` 個別ではなく、raw-address return alias summary が再帰的な storage offset (`+1`, `+1`, ...) を有限の抽象値へ widen できず、`StorageOffset(Known(1))` の projection を何百段も連結し続けることだった。
- `initialized_alias_flow` で raw-address projection を正規化し、連続する storage offset を合成するようにした。
- summary 更新時に同じ構造の storage offset alias が異なる offset 値へ変化した場合は `StorageOffset(Unknown)` へ widen し、既存の `Unknown` alias がより具体的な offset alias を subsume するようにした。
- これは特定 stdlib 関数の allowlist ではなく、Resource IR の raw-address alias summary 全体へ適用される抽象解釈上の収束規則である。
- regression として、自己再帰で storage offset が増え続ける synthetic Resource IR 関数に対し、summary が `StorageOffset(Unknown)` を含む有限 projection に収束する unit test を追加した。
- `NEPL_COMPILE_STAGE_TIMING=1 target\debug\nepl-cli.exe --check --target std --profile debug --stdlib-root stdlib -i tmp\agent1-byte-builder-leb-probe.nepl`: `Check successful`。`resource_raw_alias_summary_recomputations=179 summaries=27`, `resource_initialized_raw_alias_summaries=74ms`, `resource_static_check=4451ms`。
- `cargo test -p nepl-core raw_alias_return_summaries_widen_recursive_storage_offsets -- --nocapture`: passed。
- `trunk build --release`: passed。
- `node nodesrc/tests.js -i tests/stdlib/byte_builder.n.md --no-tree -o tmp/agent1-bytebuilder-leb-timeout-fixed.json -j 1 --dist web/dist --assert-io`: total=3, passed=3。
- `node nodesrc/tests.js -i tests/stdlib/byte_builder.n.md -i tests/stdlib/text_utf8.n.md --no-tree -o tmp/agent1-bytebuilder-textutf8-fixed.json -j 1 --dist web/dist --assert-io`: total=12, passed=12。
