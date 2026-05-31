---
id: ISS-20260531T012124326Z-RESOURCE-SUMMARY-RAW-INIT-VALUE-REPR-88633148
title: "Resource summary raw-init value reprojection needs projection canonicalization"
area: core
status: verified
resolved: true
priority: P1
type: performance
created: 2026-05-31
updated: 2026-05-31
target: "nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs; nepl-core/src/resource/initialized_summary_build_value_cache.rs"
---

# ISS-20260531T012124326Z-RESOURCE-SUMMARY-RAW-INIT-VALUE-REPR-88633148: Resource summary raw-init value reprojection needs projection canonicalization

## 概要

After instantiated generic nominal mapping and signature duplicate reprojection fixes, RPN same-session code edit still reports raw_init_param_facts_reprojection_value_bypasses=25 on first compile and 50 cumulatively after the second compile. Context construction is now 0, so the residual failure is inside stable raw-init value-to-summary replay.

## 対象

- `nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs; nepl-core/src/resource/initialized_summary_build_value_cache.rs`

## 根拠

- `ISS-20260528T123956303Z-RESOURCE-SUMMARY-TYPE-REPROJECTION-N-78929A8E` で `ResourceSummaryTypeReprojection` の context 構築失敗は 0 まで減った。
- 同 checkpoint の RPN same-session code edit 測定では、初回 compile の `raw_init_param_facts_reprojection_bypasses=25` はすべて `raw_init_param_facts_reprojection_value_bypasses=25` だった。
- 2 回目 compile では `raw_init_param_facts_hits=142` / `resource_summary_value_replay_hits=142` まで伸びたが、累積 `raw_init_param_facts_reprojection_value_bypasses=50` が残った。
- `raw_init_param_facts_unstable_entry_bypasses=0` と `raw_init_param_facts_unstable_key_bypasses=0` は維持されているため、残件は stable entry 生成や key 生成ではなく、entry から現在 compile の summary surface へ戻す value replay 境界にある。

## 問題

After instantiated generic nominal mapping and signature duplicate reprojection fixes, RPN same-session code edit still reports raw_init_param_facts_reprojection_value_bypasses=25 on first compile and 50 cumulatively after the second compile. Context construction is now 0, so the residual failure is inside stable raw-init value-to-summary replay.

## 影響

Raw-init param facts store/hit coverage improved substantially, but remaining value reprojection misses still force recomputation for some stdlib functions and keep code-edit compile time around 8.5 seconds.

## 修正方針

Split reproject_raw_init_param_facts_leaf_entry value failures by param cell projection/type and release requirement projection/type, then canonicalize the replayable projection/type cases without weakening open generic ambiguity checks.

## 2026-05-31 checkpoint

- value reprojection の失敗理由を `param_cell_projection` / `param_cell_type` / `param_release_projection` / `param_release_type` に分割した。
- raw address 上の raw-init param cell `Deref` は通常の reference dereference ではないため、stable entry に保存した cell 型を復元先として使う再投影境界を追加した。通常の field / tuple / enum payload projection の layout 検証は弱めていない。
- release Web RPN same-session code edit 測定では、初回 `raw_init_param_facts_reprojection_value_bypasses=25` が `23` へ下がり、`param_cell_projection=2` は `0` になった。
- 残る `23` 件は `param_cell_stable_type` であり、raw cell value type が function signature / owner summary type boundary に現れない labelled open generic として残っている。open generic は stable key だけで同名衝突を解決できないため、TypeCtx 全体検索では解決しない。
- non-signature nominal value type は現在の TypeCtx 内 stable key から再投影できるようにしたが、boundary 外 open generic は引き続き fail-closed に拒否する。

## 2026-05-31 owner boundary checkpoint

- `owner_summary_type_params` が function signature/result だけではなく、raw memory load/store/fill の value type、user call type arguments、indirect call signature、collection slot lifecycle/drop/transform の value type を owner summary boundary へ含めるようにした。
- これは raw-init summary replay に必要な open generic を cache key の type boundary hash と `ResourceSummaryTypeReprojection` の strict boundary に通すための足場であり、単なる local type や TypeCtx 全体検索を authority にはしない。
- release Web RPN same-session 測定 `tmp/rpn_owner_boundary_20260531.json` では、初回 `stores=165`、`bypasses=60`、`incomplete_leaf=37`、`reprojection_value=23`、`param_cell_stable_type=23` で、数値改善はまだ出ていない。
- subagent review と実測から、残る `23` 件は単純な owner boundary 追加ではなく、同名 labelled generic の provenance / ordinal を stable entry と key に持たせる設計が必要と判断した。`var(T:...)` だけを TypeCtx 全体から拾う緩和は stale hit を招くため行わない。

## 2026-05-31 follow-up review

- compile performance 側の独立 review でも、現在の主戦場は到達不能 stdlib、prelude allocator graph、Node/WASM stdlib VFS、LoaderSessionCache ではなく Resource summary value cache の raw-init replay と確認した。
- `tmp/rpn_owner_boundary_20260531.json` の初回 `compile_ms=9615`、`raw_init_param_facts_stores=165`、`bypasses=60`、`recomputed_ops=188` を基準にする。
- 残件は `incomplete_leaf=37` と `reprojection_value=23` であり、`reprojection_value` はすべて `param_cell_stable_type=23` である。`unstable_key=0`、`unstable_entry=0`、`reprojection_context=0` は維持されている。
- 次 checkpoint は labelled open generic の provenance / ordinal を stable entry と key に含め、`param_cell_stable_type` を減らす。`incomplete_leaf=37` は byte-range / variant / return facts を含む別 checkpoint として扱う。

## 2026-05-31 projection-derived replay completion

- raw-init param cell の型が base parameter と suffix から通常の layout 規則で決まる場合、stable entry に保存された cell 型を replay authority にしないようにした。
- 保存済み cell 型は raw address `Deref` のように typed projection だけでは値型を得られない場合の proof boundary としてのみ使う。
- direct user call の `type_args` を raw-init summary replay に渡し、param/return/release/variant summary 内の型を summary type parameter boundary から instantiation するようにした。
- release Web RPN same-session code edit 測定 `tmp/rpn_projection_authoritative_raw_init_type_20260531.json` では、初回 `raw_init_param_facts_reprojection_value_bypasses=0`、`param_cell_stable_type=0`、`param_cell_result_type=0` になった。
- 同測定の 2 回目 compile では `raw_init_param_facts_hits=144`、`resource_summary_value_replay_hits=167`、`recomputed_ops=21` となり、value reprojection 残件は解消した。
- 残る `raw_init_param_facts_incomplete_leaf_bypasses=37` は、byte-range / variant / return facts を complete mirror 化する [ISS-20260528T123956163Z-RESOURCE-SUMMARY-RAW-INIT-CACHE-NEED-245DC1A5](./ISS-20260528T123956163Z-RESOURCE-SUMMARY-RAW-INIT-CACHE-NEED-245DC1A5.md) で継続する。

## 検証

- `cargo test -p nepl-core owner_summary_type_params --lib -- --nocapture`
- `cargo test -p nepl-core initialized_summary_apply_param --lib -- --nocapture`
- `cargo test -p nepl-core initialized_summary_build_value_cache --lib -- --nocapture`
- `cargo test -p nepl-core stable_raw_init_param_cell --lib -- --nocapture`
- `cargo test -p nepl-core stable_raw_init_reprojection --lib -- --nocapture`
- `trunk build --release`
- RPN same-session code edit: `tmp/rpn_projection_authoritative_raw_init_type_20260531.json`
