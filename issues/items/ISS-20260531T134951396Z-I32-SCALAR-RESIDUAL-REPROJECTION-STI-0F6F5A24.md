---
id: ISS-20260531T134951396Z-I32-SCALAR-RESIDUAL-REPROJECTION-STI-0F6F5A24
title: "i32 scalar residual reprojection still recomputes RPN edit"
area: core
status: verified
resolved: true
priority: P1
type: performance
created: 2026-05-31
updated: 2026-06-01
target: "nepl-core/src/resource/initialized_scalar_flow.rs; nepl-core/src/resource/resource_summary_value_cache/i32_scalar.rs; nepl-core/src/resource/resource_summary_value_cache/stable_mirror.rs"
---

# ISS-20260531T134951396Z-I32-SCALAR-RESIDUAL-REPROJECTION-STI-0F6F5A24: i32 scalar residual reprojection still recomputes RPN edit

## 概要

RPN same-session string literal edit では、広い Resource summary preseed replay 後も
i32 scalar summary の残差再計算が残っている。kind 別 recomputed-op counter により、
残る `resource_summary_value_recomputed_ops=+16` は i32 scalar return facts に属し、
同じ edit delta の i32 scalar reprojection value bypass `+16` と一致することを確認した。

## 対象

- `nepl-core/src/resource/initialized_scalar_flow.rs`
- `nepl-core/src/resource/resource_summary_value_cache/i32_scalar.rs`

## 根拠

- `tmp/rpn_recomputed_ops_kind_counters_20260531.json` では、base `compile_ms=9237`、edit `compile_ms=3310`。
- base から edit への差分は `resource_summary_value_recomputed_ops=+16`。
- 同じ差分は `resource_summary_value_i32_scalar_return_facts_recomputed_ops=+16` だけに出ている。
- `resource_summary_value_raw_alias_return_entry_recomputed_ops`、`resource_summary_value_raw_init_param_facts_recomputed_ops`、`resource_summary_value_drop_traversal_forall_recomputed_ops` の差分は `0`。
- `resource_summary_value_i32_scalar_return_facts_bypasses=+16` と `resource_summary_value_i32_scalar_return_facts_reprojection_value_bypasses=+16` も同時に増えている。

## 問題

現在の aggregate `resource_summary_value_recomputed_ops` だけでは、残差がどの summary kind に
属するかを判断できない。今回追加した kind 別 counter により、残差は i32 scalar の
再投影失敗に限定されることが分かったが、entry 単位 / function 単位 / reason 単位の
詳細はまだ不足している。

このまま changed-function-only proof replay の実装へ進むと、実際には i32 stable mirror の
再投影境界が足りないだけの問題を、scope 設計の問題として誤って扱うおそれがある。

## 影響

- RPN code edit の `resource_static_check` はまだ秒単位であり、changed-function-only proof replay と並行して残差の内訳を正確に追う必要がある。
- i32 scalar summary は raw-init / collection slot / private cache proof の下流入力にもなるため、再投影できない stable mirror surface を放置すると、後続の cache 設計で同じ失敗が再発する。
- 残差は全体の支配項ではないが、aggregate counter に混ざると次の性能 issue 分割を誤らせる。

## 修正方針

- i32 scalar replay miss に、entry 単位 / function 単位 / reason 単位 counter を追加する。
- `ReturnProjection`、`ParameterProjection`、`ScalarType` などの再投影失敗理由を JSON で観測できるようにする。
- 必要なら debug-only 測定モードで、bypass した function name と fact count を出す。
- 原因が scalar type canonicalization など単一 surface に収束したら、dependency closure、source policy、type boundary key を弱めずに stable mirror boundary を修正する。

## 2026-05-31 reason counter checkpoint

i32 scalar residual の発生源を function count と fact count に分けて観測する counter を追加した。
既存の `resource_summary_value_i32_scalar_return_facts_recomputed_ops` は facts 数の aggregate のまま
維持し、replay の entry miss は entry 取得前で fact count を持たないため function count として
別 counter にした。

追加した主な観測点は次である。

- `resource_summary_value_i32_scalar_return_facts_misses`
- `resource_summary_value_i32_scalar_return_facts_reprojection_value_return_projection_bypasses`
- `resource_summary_value_i32_scalar_return_facts_reprojection_value_parameter_projection_bypasses`
- `resource_summary_value_i32_scalar_return_facts_reprojection_value_scalar_type_bypasses`
- `resource_summary_value_i32_scalar_return_facts_replay_entry_miss_functions`
- `resource_summary_value_i32_scalar_return_facts_replay_reprojection_value_*_functions`

`tmp/rpn_i32_residual_reason_counters_20260531.json` では、same-session string literal edit が
次の結果になった。

- base `compile_ms=8932`、`resource_static_check=8347.574ms`
- edit `compile_ms=2102`、`resource_static_check=1816.135ms`
- base から edit への差分は `resource_summary_value_i32_scalar_return_facts_recomputed_ops=+16`
- base から edit への差分は `resource_summary_value_i32_scalar_return_facts_reprojection_value_bypasses=+16`
- 内訳は `resource_summary_value_i32_scalar_return_facts_reprojection_value_scalar_type_bypasses=+10`
- 内訳は `resource_summary_value_i32_scalar_return_facts_reprojection_value_parameter_projection_bypasses=+6`
- `resource_summary_value_i32_scalar_return_facts_reprojection_value_return_projection_bypasses=0`
- `resource_summary_value_i32_scalar_return_facts_replay_entry_miss_functions=+7`
- `resource_summary_value_i32_scalar_return_facts_misses=0`

したがって、残差は「replay entry が見つからず 7 関数が再計算され、その再計算結果の
16 facts が stable entry として保存できない」形で発生している。次の修正は、entry key を
弱めるのではなく、i32 scalar stable mirror の `ParameterProjection` と `ScalarType`
再投影境界を精査し、source policy / dependency closure / type boundary を保ったまま
保存可能な surface を増やす。

## 2026-05-31 stable reprojection partial checkpoint

i32 scalar stable mirror の projection-derived scalar type 境界を修正した。raw-init / raw-alias
と同じく、構造 projection から現在の function signature 上で型を計算できる場合は現在の
signature を authority とし、raw address の terminal `Deref` や終端が open generic のまま
残る場合だけ保存済み stable scalar type key を proof boundary として使う。alias / offset /
relation では return 側と parameter 側の再投影後の scalar type が同じであることを維持する。

focused regression では、projection-derived open generic scalar type が現在の applied
signature から `i32` へ rebased されること、terminal raw `Deref` は保存済み scalar type を
使って再投影されること、non-final raw `Deref` は後続 layout を検証できないため拒否される
ことを確認した。

`tmp/rpn_i32_open_generic_reprojection_code_edit_20260531.json` では、same-session code edit が
次の結果になった。

- base `compile_ms=9231`、`resource_static_check=8606.798ms`
- edit `compile_ms=2126`、`resource_static_check=1841.527ms`
- edit delta は `resource_summary_value_i32_scalar_return_facts_recomputed_ops=+10`
- edit delta は `resource_summary_value_i32_scalar_return_facts_reprojection_value_bypasses=+10`
- 内訳は scalar type `+8`、parameter projection `+2`、return projection `0`
- `resource_summary_value_i32_scalar_return_facts_replay_entry_miss_functions=+5`

直前の reason counter checkpoint の `+16` / 7 functions からは改善したが、i32 residual は
まだ解消していない。この issue は open のまま継続し、次 checkpoint では fact kind /
function name をさらに分けるか、remaining scalar type / parameter projection の stable mirror
surface を狭く修正する。

なお、この修正は微小 edit 側の residual を削るものであり、base compile の
`resource_static_check=8606.798ms` は未解決である。base compile 0.5 秒未満の目標は
親 issue と per-program compile performance issue で別途追跡する。

## 2026-05-31 fact kind counter checkpoint

i32 scalar residual の `ReprojectionValue` bypass を fact 種別ごとに分解する counter を追加した。
既存の reason counter は「なぜ stable entry 化できなかったか」を示すが、alias / offset /
relation / condition のどの surface が失われたかまでは分からなかった。今回の counter は、
`ReprojectionValue` に限って同じ facts を種類別に集計し、Web / Node の
`CompilerSession.loader_cache_stats_json()` から読めるようにした。

追加した観測点は次である。

- `resource_summary_value_i32_scalar_return_facts_reprojection_value_alias_bypasses`
- `resource_summary_value_i32_scalar_return_facts_reprojection_value_offset_bypasses`
- `resource_summary_value_i32_scalar_return_facts_reprojection_value_relation_bypasses`
- `resource_summary_value_i32_scalar_return_facts_reprojection_value_constant_bypasses`
- `resource_summary_value_i32_scalar_return_facts_reprojection_value_return_condition_bypasses`
- `resource_summary_value_i32_scalar_return_facts_reprojection_value_parameter_condition_bypasses`

`tmp/rpn_i32_fact_kind_counters_final_code_edit_20260531.json` では、same-session unused local
追加 edit が次の結果になった。

- base `compile_ms=8931`、`resource_static_check=8318.313ms`
- edit `compile_ms=2219`、`resource_static_check=1922.104ms`
- edit delta は `resource_summary_value_i32_scalar_return_facts_recomputed_ops=+10`
- edit delta は `resource_summary_value_i32_scalar_return_facts_reprojection_value_bypasses=+10`
- reason 内訳は `scalar_type=+8`、`parameter_projection=+2`、`return_projection=0`
- fact kind 内訳は `alias=+5`、`offset=+5`
- `relation` / `constant` / `return_condition` / `parameter_condition` は `0`
- `resource_summary_value_i32_scalar_return_facts_replay_entry_miss_functions=+5`

したがって、残る i32 residual は condition 系ではなく、alias / offset の stable entry 化が
`ScalarType` と `ParameterProjection` で落ちる問題として扱う。試作として保存側で
構造 projection の終端型を優先する案も測定したが、RPN edit delta が `+12` へ悪化したため
採用しない。次の修正では entry key や dependency closure を弱めず、bypass した function /
fact の具体名を debug-only に分けてから、alias / offset のどの projection surface が
実際に失敗しているかを確認する。

## 2026-06-01 debug-only fact logging checkpoint

i32 scalar residual の次修正に向けて、通常 JSON stats へ配列や fact 本体を追加せず、
native debug-only の stderr ログで function / fact / reason を確認できるようにした。
`NEPL_RESOURCE_I32_STABLE_REPROJECTION_DEBUG=1` を指定した native 実行だけで出力し、
通常 compile path と Web playground の measurement schema には影響させない。

追加したログは次である。

- `resource-i32-stable-reprojection`: stable entry の再投影時に、最初に失敗した
  fact の `function`、fact kind、index、reason、stable fact 本体を出す。
- `resource-i32-candidate-bypass`: candidate 化に失敗した関数の aggregate reason と
  alias / offset / relation / condition の件数、元の facts を出す。

`tmp/i32_stable_reprojection_debug_unit_20260601.stderr` では、既存 focused regression の
wrong scalar type case で次のログを確認した。

- `function=i32_leaf`
- `kind=alias`
- `reason=ScalarType`
- `fact=ResourceSummaryStableI32ScalarReturnAlias { ... scalar_ty: ResourceSummaryStableTypeKey("u8") }`

subagent review では、entry-level の reason だけでは fact 単位の失敗位置が潰れるため、
`reproject_i32_scalar_return_facts_entry_result` 側で fact kind / index を持ったまま
ログを出すべきだと確認した。今回の変更はその観測境界を追加するものであり、cache key、
dependency closure、source capability policy、type boundary は変更していない。

次の実修正は、このログを RPN same-session edit 相当の native 再現または focused
regression に当て、残っている alias / offset の `ParameterProjection` / `ScalarType`
失敗を `reproject_i32_scalar_projection_suffix_with_stable_type` と
`reproject_i32_scalar_common_type` の狭い surface で修正する。

## 2026-06-01 stable reprojection completion checkpoint

RPN debug-only ログで確認した残差は、次の 3 surface に分かれていた。

- `load__i32__T__pure_Option_T_i32` の parameter projection `[Deref, EnumPayload(Some)]`。
- `string_addr` / `string_access_addr` / `string_byte_access_addr` 系の `str` 全体を raw address scalar carrier として扱う alias。
- `string_data_ptr` の `str` に対する `StorageOffset(Known(4))` を i32 raw address view として扱う offset。

修正では、i32 scalar alias / offset の `scalar_ty` を構造 projection の終端型と
同一視しないようにした。projection の `ty` は suffix replay の layout 検証に使う
構造型であり、i32 leaf としての proof boundary は stable scalar type key で別に持つ。
そのため、`str` のような raw address carrier や `StorageOffset` を含む suffix は、
stable scalar type が現在 session の `i32` へ再投影できる場合に限り受け入れる。

non-final `Deref` は引き続き原則拒否する。ただし、parameter suffix が先頭 `Deref` を
除いて return suffix と完全一致する alias / offset では、先に検証される function result
型を raw load container authority として使う。これにより、raw address から `Option<i32>`
を読み出して return 側と同じ `EnumPayload(Some)` leaf を参照する fact だけを
fail-closed に再投影できる。

focused regression として、次を追加した。

- leading raw `Deref` が return container と同じ suffix に anchored される場合だけ通る。
- unanchored non-final raw `Deref` は `ParameterProjection` のまま拒否される。
- `str` 全体の raw address carrier alias が `i32` scalar fact として再投影される。
- `str` の `StorageOffset(Known(4))` carrier offset が `MemPtr<u8>.addr` の `i32` として再投影される。
- raw address carrier ではない型の `StorageOffset(Known(_))` は `ScalarType` のまま拒否される。
- symbolic offset 内の place が callee 側 open generic でも、caller signature の field projection から `i32` に rebased される。

`tmp/rpn_i32_reprojection_fix_measure_20260601.json` では、same-session unused local edit が
次の結果になった。

- base `compile_ms=9583`
- edit `compile_ms=2292`
- edit delta は `resource_summary_value_i32_scalar_return_facts_recomputed_ops=0`
- edit delta は `resource_summary_value_i32_scalar_return_facts_bypasses=0`
- edit delta は `resource_summary_value_i32_scalar_return_facts_reprojection_value_bypasses=0`
- alias / offset / scalar type / parameter projection の i32 scalar reprojection bypass delta はすべて `0`
- `resource_summary_value_i32_scalar_return_facts_replay_entry_miss_functions` delta も `0`

これにより、この issue の検証条件である「RPN same-session edit の i32 scalar residual
delta を 0 にする」は満たした。base compile `compile_ms=9583` と edit compile
`compile_ms=2292` は依然として性能目標に届いていないため、初回 compile 0.5 秒未満、
式枝差し替え 0.1 秒以下、stdlib prechecked artifact / Resource proof template /
typed expression subtree query / codegen fragment cache は親 issue 群で継続する。

## 検証

- RPN same-session edit JSON で、i32 scalar recomputed op delta と i32 scalar reprojection bypass delta が `0` になる。
- `cargo test -p nepl-core resource_summary_value --lib -- --nocapture`
- RPN same-session edit 測定 `tmp/rpn_i32_reprojection_fix_measure_20260601.json`
