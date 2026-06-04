---
id: ISS-20260604T034255467Z-SELFHOST-TYPE-AND-HIR-RANGES-ALLOW-I-A4509F7E
title: "selfhost Type and HIR ranges allow invalid raw i32 count invariants"
area: selfhost
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-06-04
updated: 2026-06-04
target: "stdlib/neplg2/core/hir/hir/range.nepl, stdlib/neplg2/core/ty/ty/record.nepl, stdlib/neplg2/core/ty/ty/eq.nepl"
---

# ISS-20260604T034255467Z-SELFHOST-TYPE-AND-HIR-RANGES-ALLOW-I-A4509F7E: selfhost Type and HIR ranges allow invalid raw i32 count invariants

## 概要

Subagent audit found HIR and type range constructors storing first/count as raw i32 without checked construction, and equality logic can treat negative counts as trivially equal. This violates the Zenn policy that invalid states should be excluded with typed constructors and Result.

## 対象

- `stdlib/neplg2/core/hir/hir/range.nepl, stdlib/neplg2/core/ty/ty/record.nepl, stdlib/neplg2/core/ty/ty/eq.nepl`

## 根拠

- `SelfhostHirChildRange` / `SelfhostHirParamRange` は `Empty` variant を持っていたが、nonempty range を作る public constructor が `first/count` を raw `i32` としてそのまま payload に保存していた。
- `SelfhostHirFunction` は parameter range を `first_param` / `param_count` の raw pair として再保存しており、typed range の改善が function record 境界で失われていた。
- `SelfhostFunctionTypeRecord` も function argument table 範囲を `first_arg` / `arg_count` として保存していたため、negative count を持つ record を equality へ流せた。
- `selfhost_type_arena_function_args_equal` は `idx >= n` を終了条件にしていたため、`n < 0` の invalid range が空列一致のように true になる余地があった。

## 問題

Subagent audit found HIR and type range constructors storing first/count as raw i32 without checked construction, and equality logic can treat negative counts as trivially equal. This violates the Zenn policy that invalid states should be excluded with typed constructors and Result.

## 影響

Negative or overflowing ranges can flow into arena/type comparisons and silently produce valid-looking equality results, undermining later static checks and diagnostics.

## 修正方針

Introduce checked range constructors returning Result or a validated range type, reject negative count/overflow/out-of-arena inputs, and update callers to match on typed errors.

## 解決内容

- HIR child / parameter range に `SelfhostHirRangeBuildError` と `*_new_result` / `*_new_bounded_result` を追加し、negative first、negative count、non-canonical empty、end overflow、out-of-bounds を typed error として返すようにした。
- 既存の direct constructor は `*_new_unchecked` に改名し、arena 内で table 長と追加件数から範囲が証明済みの場所だけが使う API として明示した。
- `SelfhostHirFunction` は raw `first_param` / `param_count` ではなく `SelfhostHirParamRange` を保持する形へ変更した。
- function type argument range に `SelfhostFunctionTypeArgRange` と `SelfhostFunctionTypeArgRangeBuildError` を追加し、`SelfhostFunctionTypeRecord` は typed argument range と result だけを持つ形へ変更した。
- type equality は `selfhost_function_type_arg_range_is_valid` を先に確認し、invalid range payload を false として扱う defensive boundary を追加した。

## 検証

- pass: `node nodesrc\test_selfhost_hir_range_payload.js`
- pass: `node nodesrc\test_selfhost_type_record_payload.js`
- pass: `node nodesrc\test_selfhost_hir_expr_id_absence.js`
- pass: `node nodesrc\test_selfhost_ty_split_contract.js`
- pass: `node nodesrc\test_selfhost_type_arena_report_contract.js`
- pass: `node nodesrc\tests.js -i tests\stdlib\neplg2_hir_ranges.n.md --no-tree -o tmp\selfhost-hir-range-validation.json -j 1 --assert-io --dist web\dist`
- pass: `node nodesrc\tests.js -i tests\stdlib\neplg2_type_ranges.n.md --no-tree -o tmp\selfhost-type-range-validation.json -j 1 --assert-io --dist web\dist`
- pass: `node nodesrc\tests.js -i tests\stdlib\neplg2_type_arena.n.md --no-tree -o tmp\selfhost-type-arena-after-range-validation.json -j 1 --assert-io --dist web\dist`
- pass: `node nodesrc\tests.js -i stdlib\neplg2\core\hir\hir.nepl --no-tree -o tmp\selfhost-hir-after-range-validation.json -j 1 --assert-io --dist web\dist`
