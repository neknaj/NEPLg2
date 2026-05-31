---
id: ISS-20260531T132755602Z-RAW-INIT-COMPLETE-LEAF-REPROJECTION-TYPE-CANON-4E8A1A2C
title: "Raw-init complete leaf replay still has return/byte-range type canonicalization misses"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs; nepl-core/src/resource/initialized_summary*.rs"
---

# ISS-20260531T132755602Z-RAW-INIT-COMPLETE-LEAF-REPROJECTION-TYPE-CANON-4E8A1A2C: Raw-init complete leaf replay still has return/byte-range type canonicalization misses

## 概要

Complete raw-init leaf mirror により `raw_init_param_facts_incomplete_leaf_bypasses` は 0 になったが、RPN same-session code edit では `raw_init_param_facts_reprojection_value_bypasses` が残っている。

## 対象

- `nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs`
- `nepl-core/src/resource/initialized_summary*.rs`

## 根拠

- 2026-05-31 の complete raw-init leaf checkpoint で、初回 RPN compile は `stores=259`、`incomplete_leaf=0`、`reprojection_value=15`、`param_cell_result_type=15` だった。
- 同一 `CompilerSession` 内で `examples/rpn.nepl` に local `i32` binding を追加した code edit 測定では、compiled-output cache は miss し、edit delta は `raw_init_param_facts_hits=205`、`resource_summary_value_replayed_ops=238`、`raw_init_param_facts_incomplete_leaf_bypasses=0` だった。
- 同 code edit の edit delta でも `raw_init_param_facts_reprojection_value_bypasses=15`、`param_cell_result_type=15` が残る。
- これは byte-range / variant / return facts を保存していなかった partial mirror 問題ではなく、保存対象を complete leaf に広げたことで露出した value type 再投影の canonicalization 問題である。

## 問題

`RawCellInitializationFunctionSummary` の return / byte-range / variant surface は stable entry に入るようになったが、一部の replay で保存済み stable type key と現在 compile の projection result type の対応が一致せず、fail-closed に bypass している。

## 影響

raw-init summary value cache は code edit 時に有効になり始めているが、残る type canonicalization miss の分だけ summary replay できない関数が残り、RPN の code edit compile はまだ秒単位である。

## 修正方針

- 失敗箇所を return cell / return byte-range / param byte-range / variant cell / variant byte-range / variant condition ごとに分けて観測する。
- 通常 projection から型が決まる surface は現在 signature と suffix を authority にし、raw address `Deref` のような typed projection だけでは値型を得られない surface だけ保存済み stable type を proof boundary にする。
- boundary 外 labelled generic を `TypeCtx` 全体検索で拾う緩和は行わない。必要な generic provenance / ordinal は owner summary type boundary または stable entry に明示する。
- final raw `Deref` fallback は raw-address cell view に限定し、途中 `Deref` で後続 projection の layout 検証を弱めない。

## 検証

- RPN same-session code edit の compiled-output miss 測定で `raw_init_param_facts_reprojection_value_bypasses=0` を確認する。
- focused regression で return / byte-range / variant の corrupted layout が fail-closed になること、non-final raw `Deref` fallback が拒否されることを維持する。
