---
id: ISS-20260531T075621000Z-RAW-ALIAS-RESIDUAL-REPROJECTION-VAL-9A5D0C3E
title: "raw alias residual reprojection value bypasses keep 38 recomputations"
area: core
status: open
resolved: false
priority: P1
type: performance
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/resource/initialized_alias_flow.rs; nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs"
---

# ISS-20260531T075621000Z-RAW-ALIAS-RESIDUAL-REPROJECTION-VAL-9A5D0C3E: raw alias residual reprojection value bypasses keep 38 recomputations

## 概要

raw alias summary stable mirror / preseed cache により RPN same-session code edit の `resource_raw_alias_summary_recomputations` は 288 から 38 へ下がったが、まだ 13 件の `raw_alias_return_entry` reprojection value bypass が残っている。

## 対象

- `nepl-core/src/resource/initialized_alias_flow.rs`
- `nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs`
- `nepl-core/src/resource/resource_summary_value_cache/raw_alias.rs`

## 根拠

- `tmp/rpn_raw_alias_cache_code_edit_20260531.json` の edit delta では `resource_summary_value_raw_alias_return_entry_hits=65`、`stores=73`、`bypasses=13`、`reprojection_bypasses=13`、`reprojection_value_bypasses=13` だった。
- 同じ測定で `resource_summary_value_raw_alias_return_entry_unstable_key_bypasses=0`、`resource_summary_value_raw_alias_return_entry_unstable_entry_bypasses=0` であり、key 構築や stable entry 化の根本失敗ではなく、entry を現在 compile の projection / type surface へ戻す段階の残件である。
- bypass がある関数は通常 fixed-point に戻るため stale hit にはならないが、依存 propagation により raw alias summary recomputation が 38 件残る。

## 問題

raw alias summary の preseed cache は有効になったが、projection / type reprojection の残差により、RPN の実コード微小編集はまだ秒単位 compile の一部を raw alias fixed-point に使っている。

## 影響

RPN code edit の compile time は `7142ms` であり、0.5 秒未満 compile / 0.1 秒以下の式枝差し替え目標には届いていない。raw-init residual recomputation と並んで、次の Resource IR summary replay 支配項として扱う必要がある。

## 修正方針

- raw alias residual bypass の関数名、parameter projection、return projection、type mismatch reason を debug-only trace または細分 counter で取得する。
- `stable_mirror.rs` の raw alias projection reprojection が、現在の function signature / owner summary type boundary / generic instantiation に対して不足している surface を特定する。
- TypeCtx 全体検索や似た型名への緩和は行わず、必要な stable surface を key と entry に追加できる場合だけ replay 範囲を広げる。
- 修正後も `unstable_key=0`、`unstable_entry=0`、source policy / dependency closure miss の fail-closed 境界を維持する。

## 検証

RPN same-session code edit JSON で `resource_summary_value_raw_alias_return_entry_reprojection_value_bypasses` と `resource_raw_alias_summary_recomputations` がさらに下がり、existing Resource IR safety tests と `node nodesrc/issues.js check --dir issues` が通ることを確認する。
