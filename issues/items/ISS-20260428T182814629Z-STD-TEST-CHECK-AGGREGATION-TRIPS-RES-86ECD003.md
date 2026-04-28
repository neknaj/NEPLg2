---
id: ISS-20260428T182814629Z-STD-TEST-CHECK-AGGREGATION-TRIPS-RES-86ECD003
title: "std/test check aggregation trips Resource IR raw dealloc gate"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "stdlib/std/test.nepl; tests/stdlib/neplg2_type_arena.n.md"
---

# ISS-20260428T182814629Z-STD-TEST-CHECK-AGGREGATION-TRIPS-RES-86ECD003: std/test check aggregation trips Resource IR raw dealloc gate

## 概要

remote main `ba56490` の Resource IR raw alias 強化後、`std/test` の check 集約 helper が `Vec<Result<(),str>>` owner を raw 一時領域へ退避していたため、`RawMemoryDeallocCell` が初期化済み非 Copy owner の残存として検出されていました。

## 対象

- `stdlib/std/test.nepl; tests/stdlib/neplg2_type_arena.n.md`

## 根拠

- `node nodesrc\tests.js -i tests\stdlib\neplg2_type_arena.n.md --no-tree -o tmp\std-test-resource-gate-repro.json -j 1` が 3/3 compile fail。
- 先頭 error は `D3100 resource ir raw memory cell ownership violation` で、`checks_print_human__Vec_T_Result_T_E_unit_str...` の `RawMemoryDeallocCell` に集中していた。
- `checks_has_err` / `checks_summary` / `checks_print_human` / `checks_print_machine` / `finish_checks` が同じ raw 一時 owner 退避 pattern を共有していた。

## 問題

`std/test` は `Vec<Result<(),str>>` の len/data を読むために owner 全体を raw temporary cell へ `store` し、field を `load` したあと同じ cell を `dealloc_raw` していました。Resource IR が raw cell の初期化状態を追跡するようになったため、非 Copy owner を raw cell に残すこの実装は正しく拒否されます。

## 影響

`checks_print_report` / `checks_exit_code` を使う doctest が対象ライブラリの正誤と無関係に compile fail し、self-host stdlib の回帰テストが broad gate として機能しなくなります。

## 修正方針

raw temporary cell で owner を退避するのをやめ、`&checks` から `Vec` の `len_ref` / `data_ptr_ref` を読む実装に置き換えました。観測後は元の owner をそのまま返すか `free` し、線形所有権を保つようにしました。

## 検証

- `node nodesrc\tests.js -i stdlib\std\test.nepl --no-tree -o tmp\std-test-resource-gate-after.json -j 1`: total=12 passed=12
- `node nodesrc\tests.js -i tests\stdlib\neplg2_type_arena.n.md --no-tree -o tmp\std-test-resource-gate-focused.json -j 1`: total=3 passed=3
- `trunk build`: pass
- `node nodesrc\tests.js -i stdlib\std\test.nepl --no-tree -o tmp\std-test-resource-gate-after-trunk.json -j 1`: total=12 passed=12
- `node nodesrc\tests.js -i tests\stdlib\neplg2_type_arena.n.md --no-tree -o tmp\std-test-resource-gate-focused-after-trunk.json -j 1`: total=3 passed=3
- `NEPL_TEST_CASE_TIMEOUT_MS=30000 node nodesrc\tests.js -i tests\stdlib\neplg2_import_spec.n.md --no-tree -o tmp\selfhost-import-spec-timeout-probe.json -j 1`: D3100 は再発せず、別 issue `ISS-20260428T184502533Z-SELF-HOST-IMPORT-SPEC-TEST-OVERFLOWS-BDC6F326` の wasm codegen stack overflow まで進むことを確認。
- `origin/main` `9e74c6e` へ rebase 後、`trunk build`: pass
- rebase 後、`node nodesrc\tests.js -i stdlib\std\test.nepl --no-tree -o tmp\std-test-resource-gate-after-rebase.json -j 1`: total=12 passed=12
- rebase 後、`node nodesrc\tests.js -i tests\stdlib\neplg2_type_arena.n.md --no-tree -o tmp\std-test-resource-gate-focused-after-rebase.json -j 1`: total=3 passed=3
