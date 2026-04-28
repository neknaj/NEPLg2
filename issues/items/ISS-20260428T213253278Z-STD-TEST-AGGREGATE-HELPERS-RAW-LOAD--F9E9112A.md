---
id: ISS-20260428T213253278Z-STD-TEST-AGGREGATE-HELPERS-RAW-LOAD--F9E9112A
title: "std/test aggregate helpers raw-load Vec backing store under RawMemoryLoadCell gate"
area: stdlib
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-04-28
updated: 2026-04-29
target: "stdlib/std/test.nepl, nepl-core/src/resource/lower.rs, nepl-core/src/resource/cell_state.rs"
---

# ISS-20260428T213253278Z-STD-TEST-AGGREGATE-HELPERS-RAW-LOAD--F9E9112A: std/test aggregate helpers raw-load Vec backing store under RawMemoryLoadCell gate

## 概要

After remote main enabled the RawMemoryLoadCell gate, std/test doctests fail because checks_has_err_loop, checks_summary_loop, and checks_print_human_loop read Vec<Result<(),str>> elements through raw data pointers with load<Result<(),str>>. The resource checker cannot prove those temporary raw cells are initialized.

## 対象

- `stdlib/std/test.nepl`

## 根拠

- `stdlib/std/test.nepl` の旧実装は `Vec<Result<(),str>>` の data pointer と len を取り出し、`checks_has_err_loop` / `checks_summary_loop` / `checks_print_human_loop` で `load<Result<(),str>>` を使って要素を再読込していた。
- `Result<(),str>` は Copy として扱える型ではあるが、raw backing store から aggregate payload を後読みする設計は Resource IR の初期化状態を失いやすく、RawMemoryLoadCell gate の検査対象になっていた。
- `std/test` は多くの doctest が import する基盤なので、この helper が落ちると対象 module の検査前に compile failure になる。
- compiler 側の同根問題は [ISS-20260428T214527171Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-T-F609F5AB](ISS-20260428T214527171Z-RESOURCE-IR-RAWMEMORYLOADCELL-GATE-T-F609F5AB.md) で扱い、function-external initialized raw root と direct raw address alias を分離した。

## 問題

After remote main enabled the RawMemoryLoadCell gate, std/test doctests fail because checks_has_err_loop, checks_summary_loop, and checks_print_human_loop read Vec<Result<(),str>> elements through raw data pointers with load<Result<(),str>>. The resource checker cannot prove those temporary raw cells are initialized.

## 影響

All std/test doctests fail on current main, and any doctest importing std/test can fail before testing its own module. This blocks stdlib and self-host regression tests.

## 修正方針

`std/test` の API は raw backing-store scan を持たない accumulator に移行し、compiler 側では function-external initialized raw root と direct raw address arithmetic alias を Resource IR に表現する。stdlib だけで隠すのではなく、public helper と compiler model の両方を根本原因に合わせる。

## 解決

`std/test` は `Checks` accumulator により raw scan 自体を削除した。加えて compiler 側では親 issue として external initialized raw root と direct raw address arithmetic alias を扱うようにし、同種の valid external raw load が未初期化 temporary へ落ちないようにした。

## 修正内容

- `Checks` value accumulator を追加し、`checks_push` の時点で件数、失敗 flag、machine summary、人間向け report を更新する形に変更した。
- `checks_has_err` / `checks_summary` / `checks_print_human` / `checks_print_machine` / `finish_checks` / `checks_exit_code` は `Checks` を受け取る API に変更し、`Vec<Result<(),str>>` backing store の raw scan を削除した。
- `stdlib/std/test.nepl` から `core/mem` と `alloc/collections/vec` 依存を外し、非 Copy payload を raw memory から `load<Result<(),str>>` する経路をなくした。
- 既存 doctest / stdlib test / tutorial の `checks_new` call-site は型推論に任せる形へ更新し、helper signature 上で必要な箇所だけ `Checks` を使うようにした。

## 検証

- `git diff --check`: pass
- `rg -n "load<Result<(),str>>|Vec<Result<(),str>>|checks_has_err_loop|checks_summary_loop|checks_print_human_loop" stdlib/std/test.nepl stdlib tests tutorials`: no match
- `node nodesrc/tests.js -i stdlib\std\test.nepl --no-tree -o tmp\std-test-checks-accumulator-final.json -j 1`: total=12 failed=12。失敗は既知の `ISS-20260428T213732897Z-RAWMEMORYLOADCELL-GATE-REJECTS-INITI-6041ACF2` 由来の `alloc/string.nepl` / collection raw memory D3100 で、`std/test` の `load<Result<(),str>>` 経路ではない。
- `node nodesrc/tests.js -i tests\stdlib\std_test_collect.n.md --no-tree -o tmp\std-test-collect-checks-accumulator-final.json -j 1`: total=3 failed=3。失敗は同じ既知の raw memory gate ブロッカー。
- `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_direct_arithmetic_external_raw_load -- --nocapture`: pass
- `trunk build`: pass
- `node nodesrc/tests.js -i tutorials --no-tree -o tmp/raw-load-cell-external-roots-after-main-sync-tutorials.json -j 1`: `checks_has_err_loop` / `checks_summary_loop` の D3100 は再発せず。残り tutorials failure は別の `RegionToken` / scratch cell moved issue。
