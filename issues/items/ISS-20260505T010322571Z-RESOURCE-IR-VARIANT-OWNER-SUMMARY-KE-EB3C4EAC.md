---
id: ISS-20260505T010322571Z-RESOURCE-IR-VARIANT-OWNER-SUMMARY-KE-EB3C4EAC
title: "Resource IR variant owner summary keeps consumed projection also returned by same variant"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-05
updated: 2026-05-05
target: "nepl-core/src/resource/owner_summary_cleanup.rs, nepl-core/src/resource/owner_summary.rs, nepl-core/src/resource/owner_variant.rs, nepl-core/tests/resource_ir.rs"
---

# ISS-20260505T010322571Z-RESOURCE-IR-VARIANT-OWNER-SUMMARY-KE-EB3C4EAC: Resource IR variant owner summary keeps consumed projection also returned by same variant

## 概要

OwnerReturnSummary records the same owner projection as both variant_projection_returns and variant_consumed_parameter_sources for a single Result variant when different paths of that variant either return the parameter owner or replace it with a fresh owner.

## 対象

- `nepl-core/src/resource/owner_summary_cleanup.rs`
- `nepl-core/src/resource/owner_summary.rs`
- `nepl-core/src/resource/owner_variant.rs`
- `nepl-core/tests/resource_ir.rs`
- `doc/neplg2/static_check_complexity_reduction_plan.md` Stage 4 Resource Check

## 根拠

- `reserve(h, grow)` のように、同じ `Result::Ok` variant で `grow=false` は引数 owner を返し、`grow=true` は古い owner を解放して fresh owner を返す関数を Resource IR summary 化すると、同じ owner projection が「返却」と「消費」の両方に記録されていた。
- 呼び出し側の `match Result::Ok reserved` で payload binding 前に `reserved.data.Some` が `Reserved` と誤診断され、正しい owner-preserving flow がコンパイルできなくなっていた。

## 問題

OwnerReturnSummary records the same owner projection as both variant_projection_returns and variant_consumed_parameter_sources for a single Result variant when different paths of that variant either return the parameter owner or replace it with a fresh owner.

## 影響

Matching the Result payload can report a false resource.owner.reserved diagnostic before binding, blocking valid owner-preserving flows such as reserve/append helpers and hiding real ownership issues behind an impossible summary state.

## 修正方針

Normalize variant owner summaries so a parameter owner returned by a variant is not also recorded as consumed by that same variant; materialize the returned owner alias and keep unrelated variant consumptions intact.

## 対応内容

- variant owner summary の cleanup に、同一 variant で返却される parameter owner を同一 variant の consumed summary から除外する正規化を追加した。
- `Result::Ok` payload を match arm に入る時点で materialize した際、captured owner の raw alias と storage origin も戻り値 payload 側へ移すようにした。
- 非選択 variant の pending effect は arm 選択後に落とし、選択 arm の無関係な消費だけを残すようにした。
- fresh owner へ置き換える path と引数 owner をそのまま返す path が同じ variant に混在する regression を追加した。

## 検証

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_materializes_result_payload_owner_aliases_before_binding`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_mem_ptr_use_before_dealloc_result_refinement`
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_rejects_mem_ptr_use_before_realloc_result_refinement`
- `node nodesrc/tests.js -i tests/stdlib/features_tui.n.md --no-tree -o tmp/features-tui-after-owner-summary.json -j 1 --dist web/dist`: `resource.owner.reserved` は消えた。残る `get_terminal_size` の `maybe_freed` と StringBuilder の `maybe_leak` は `ISS-20260505T010829802Z-WASIX-GET-TERMINAL-SIZE-DEALLOCATES--A1302F57` と `ISS-20260429T142213822Z-BYTEBUILDER-AND-STRINGBUILDER-RESULT-4EB1D1EB` に分離した。
