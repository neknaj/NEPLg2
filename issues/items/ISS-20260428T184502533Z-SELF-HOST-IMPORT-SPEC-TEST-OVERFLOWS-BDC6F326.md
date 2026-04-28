---
id: ISS-20260428T184502533Z-SELF-HOST-IMPORT-SPEC-TEST-OVERFLOWS-BDC6F326
title: "self-host import_spec test overflows wasm codegen stack"
area: core
status: open
resolved: false
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

## 影響

self-host module/import-spec regression suite が broad validation gate として完走できません。また、self-host stdlib の正当なプログラムが診断ではなく artifact 生成時の host stack exhaustion で落ちる可能性が残ります。

## 修正方針

failing doctest#1 を縮約して該当 HIR shape を特定し、core 側に wasm codegen regression を追加します。そのうえで `gen_expr` / `gen_block` の該当 traversal を explicit stack または bounded traversal へ置き換え、host stack depth に依存しない生成にします。

## 検証

node nodesrc/tests.js -i tests/stdlib/neplg2_import_spec.n.md --no-tree -o tmp/selfhost-import-spec-codegen-stack-after.json -j 1; include the reduced Rust/core regression once isolated.
