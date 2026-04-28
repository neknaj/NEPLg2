---
id: ISS-20260428T184502533Z-SELF-HOST-IMPORT-SPEC-TEST-OVERFLOWS-BDC6F326
title: "self-host import_spec test overflows wasm codegen stack"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "nepl-core/src/codegen_wasm.rs; tests/stdlib/neplg2_import_spec.n.md"
---

# ISS-20260428T184502533Z-SELF-HOST-IMPORT-SPEC-TEST-OVERFLOWS-BDC6F326: self-host import_spec test overflows wasm codegen stack

## 概要

`std/test` の D3100 回帰を取り除くと、`tests/stdlib/neplg2_import_spec.n.md::doctest#1` が wasm codegen まで進み、`codegen_wasm::gen_expr` / `gen_block` の再帰で `RangeError: Maximum call stack size exceeded` になります。

## 対象

- `nepl-core/src/codegen_wasm.rs; tests/stdlib/neplg2_import_spec.n.md`

## 根拠

- `NEPL_TEST_CASE_TIMEOUT_MS=30000 node nodesrc\tests.js -i tests\stdlib\neplg2_import_spec.n.md --no-tree -o tmp\selfhost-import-spec-timeout-probe.json -j 1`: total=3, passed=2, failed=1。
- `origin/main` `9e74c6e` へ rebase 後の `NEPL_TEST_CASE_TIMEOUT_MS=30000 node nodesrc\tests.js -i tests\stdlib\neplg2_import_spec.n.md --no-tree -o tmp\selfhost-import-spec-timeout-probe-after-rebase.json -j 1` でも total=3, passed=2, failed=1。
- failure は `tests\stdlib\neplg2_import_spec.n.md::doctest#1` の compile phase。
- stack は `nepl_core::codegen_wasm::gen_expr` と `nepl_core::codegen_wasm::gen_block` の往復に集中している。
- 同 suite は `ISS-20260428T141156276Z...` / `ISS-20260428T163153838Z...` の検証時点では 3/3 pass していたため、現 remote main で self-host broad validation の新しい blocker になっている。

## 問題

wasm artifact 生成側の HIR traversal が host stack に依存しており、self-host import spec の正当なテスト入力で `gen_expr` / `gen_block` の再帰が深くなりすぎます。D3100 に隠れていたため、`std/test` 修正後に顕在化しました。

追加調査では native CLI は同じ入力を約 21 秒で wasm 生成でき、`node --stack_size=32768` でも Node runner は pass しました。通常 Node stack だけが落ちるため、入力の意味論ではなく wasm-host 上の compiler stack 消費が根本原因です。HIR の最深箇所は self-host lexer の `lex_keyword_kind` にある else-if decision tree で、`gen_expr` が `If` の else chain を再帰的に下げていました。

## 影響

self-host module/import-spec regression suite が broad validation gate として完走できません。また、self-host stdlib の正当なプログラムが診断ではなく artifact 生成時の host stack exhaustion で落ちる可能性が残ります。

## 修正方針

`If` の else-if chain を wasm codegen 側で iterative に下げる `gen_if_else_chain` を追加しました。各条件と then branch は既存の `gen_expr` で下げつつ、else branch がさらに `If` の場合は Rust/WASM stack を積まずに loop で同じ instruction 列を生成します。これにより stdlib 側の keyword classifier を一時的に浅く書き換える回避ではなく、同種の decision tree 全体に効く compiler 側の修正にしました。

## 検証

- `target\debug\nepl-cli.exe -i tmp\selfhost_import_spec_case1.nepl --target std -o tmp\selfhost_import_spec_case1_direct --emit wasm`: pass（調査用一時入力、約 21 秒）
- `node --stack_size=32768 nodesrc\tests.js -i tests\stdlib\neplg2_import_spec.n.md --no-tree -o tmp\selfhost-import-spec-stack-size-probe.json -j 1`: total=3 passed=3（stack 依存の切り分け）
- `rustfmt --check nepl-core\src\codegen_wasm.rs`: pass
- `trunk build`: pass
- `node nodesrc\tests.js -i tests\stdlib\neplg2_import_spec.n.md --no-tree -o tmp\selfhost-import-spec-codegen-stack-after-if-chain.json -j 1`: total=3 passed=3
- `node nodesrc\tests.js -i stdlib\neplg2 -i tests\stdlib\neplg2_type_arena.n.md -i tests\stdlib\neplg2_stdlib_map.n.md -i tests\stdlib\neplg2_module_graph.n.md -i tests\stdlib\neplg2_module_loader.n.md -i tests\stdlib\neplg2_import_spec.n.md -i tests\stdlib\neplg2_parser.n.md -i tests\stdlib\neplg2_lexer.n.md --no-tree -o tmp\selfhost-broad-after-if-chain-codegen.json -j 1`: total=58 passed=58
- `cargo fmt --all --check`: 未変更の `nepl-core/src/lexer.rs` の既存 formatting drift で fail（今回触った `codegen_wasm.rs` は `rustfmt --check` 済み）
- `cargo test -p nepl-core --test check_pipeline compile_wasm_accepts_deep_prefix_chain_without_codegen_stack_overflow -- --nocapture`: remote main 由来の既存 native stack overflow が残るため fail。本 issue の import_spec/web stack overflow とは分離。
