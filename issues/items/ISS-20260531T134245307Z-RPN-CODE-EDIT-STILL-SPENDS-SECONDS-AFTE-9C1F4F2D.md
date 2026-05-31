---
id: ISS-20260531T134245307Z-RPN-CODE-EDIT-STILL-SPENDS-SECONDS-AFTE-9C1F4F2D
title: "RPN code edit still spends seconds after raw-init replay"
area: core
status: investigating
resolved: false
priority: P1
type: performance
created: 2026-05-31
updated: 2026-06-01
target: "nepl-core/src/resource; nepl-web/src/lib.rs; nodesrc/run_test.js"
---

# ISS-20260531T134245307Z-RPN-CODE-EDIT-STILL-SPENDS-SECONDS-AFTE-9C1F4F2D: RPN code edit still spends seconds after raw-init replay

## 概要

Complete raw-init leaf replay の false miss は解消したが、RPN same-session code edit はまだ数秒かかる。`recomputed_ops=21` と Resource IR summary cache 外の固定費を分解し、0.5 秒未満の compile と 10ms 未満の微小再compileへ近づける必要がある。

## 対象

- `nepl-core/src/resource`
- `nepl-web/src/lib.rs`
- `nodesrc/run_test.js`

## 根拠

- 2026-05-31 の `tmp/rpn_return_type_canonicalization_code_edit_20260531.json` では、base `compile_ms=8861`、edit `compile_ms=6703`。
- 同 edit delta では `raw_init_param_facts_hits=205`、`resource_summary_value_replayed_ops=253`、`raw_init_param_facts_bypasses=0`、`raw_init_param_facts_reprojection_value_bypasses=0`、`param_cell_result_type=0` まで改善した。
- それでも edit delta に `resource_summary_value_recomputed_ops=21` が残り、compile time は秒単位である。
- raw-init complete leaf replay だけでは、stdlib-heavy workload の typecheck / monomorphize / Resource IR summary build / codegen の残り固定費を消せない。

## 問題

現在の timing は raw-init replay が効いたことは示すが、replay 後にどの stage / function / summary kind が秒単位の時間を消費しているかを十分に分解できていない。次の性能改善では、remaining 21 ops と Resource IR summary cache 外の固定費を測定し、根本原因ごとに issue を分ける必要がある。

## 修正方針

- RPN same-session code edit の stage timing と Resource IR per-function timing を再取得する。
- `resource_summary_value_recomputed_ops=21` の function / summary kind / dependency reason を観測できる counter または debug-only timing を追加する。
- raw-init 以外の summary kind、typecheck / monomorphize / codegen fragment cache、stdlib prechecked artifact のどれが次の支配項かを切り分ける。
- timing 追加は通常実行の重さやコメント増加を妨げないよう、明示的な測定モードまたは軽い集約 counter に限定する。

## 2026-05-31 調査更新

Native release CLI の `NEPL_COMPILE_STAGE_TIMING=1` / `NEPL_RESOURCE_PER_FUNCTION_TIMING=1`
で `examples/rpn.nepl` を測定した結果、`resource_static_check=6950ms` の主成分は
`resource_initialized_moves=6050ms` だった。内訳は
`resource_initialized_raw_init_summaries=2502ms`、
`resource_initialized_i32_scalar_summaries=1558ms`、
`resource_initialized_function_checks=1875ms` である。

per-function timing では、`raw_init_summary` は `apply_op` / `dealloc_raw` /
`byte_builder_*`、`i32_scalar_summary` は `sb_append_result` / `byte_builder_reserve`、
final initialized check は `str_trim` / `str_slice_result` / RPN entry 関数が上位だった。
したがって、raw-init complete leaf replay の false miss 解消後の次段階は、少なくとも
次の 3 系統に分けて扱う必要がある。

- i32 scalar summary の stable mirror / replay。
- final initialized function check の function-level stable result cache または stdlib prechecked artifact。
- raw-init summary cache maintenance / remaining recomputation の詳細分解。

この checkpoint では、`CompilerSession.loader_cache_stats_json()` に initialized-state
summary stage の再計算数、summary count、final function check の関数数と op 数を追加する。
これにより Web / Node の same-session code edit JSON でも、raw-init replay 後に
全関数の fixed-point / final check が残っているかを継続観測できる。

`tmp/rpn_stage_breakdown_code_edit_20260531.json` では、base `compile_ms=8919`、
unused local 追加 edit `compile_ms=6771` だった。edit delta は次の通り。

- `resource_raw_alias_summary_recomputations=288`, `resource_raw_alias_summary_count=54`
- `resource_i32_scalar_summary_recomputations=209`, `resource_i32_scalar_summary_count=87`
- `resource_raw_init_summary_recomputations=81`, `resource_raw_init_summary_count=78`
- `resource_collection_slot_summary_recomputations=0`, `resource_collection_slot_summary_count=0`
- `resource_initialized_function_checks=288`, `resource_initialized_function_check_ops=3642`
- `resource_summary_value_replayed_ops=253`, `resource_summary_value_recomputed_ops=21`

この結果に基づき、i32 scalar summary replay を
`ISS-20260531T050630951Z-I32-SCALAR-SUMMARY-NEEDS-STABLE-MIRR-E70E2D93`、
final initialized function check replay を
`ISS-20260531T050636303Z-INITIALIZED-FUNCTION-CHECK-NEEDS-STA-66734844`
へ分割した。

## 2026-05-31 i32 scalar replay 更新

`ISS-20260531T050630951Z-I32-SCALAR-SUMMARY-NEEDS-STABLE-MIRR-E70E2D93`
で `I32ScalarReturnFacts` の stable mirror / replay を実装した。facts が空の relevant
function も empty entry として cache し、no-fact function が worklist に戻る固定費を
削った。

`tmp/rpn_i32_scalar_empty_cache_code_edit_20260531.json` では、same-session code edit
delta が次の通りになった。

- `resource_i32_scalar_summary_recomputations=14`
- `resource_i32_scalar_summary_count=87`
- `resource_raw_init_summary_recomputations=81`
- `resource_initialized_function_checks=288`
- `resource_summary_value_i32_scalar_return_facts_hits=429`
- `resource_summary_value_replayed_ops=682`

i32 scalar summary の全関数規模 replay は解消したが、edit compile は `compile_ms=6496`
でまだ秒単位である。残る支配項は raw-init residual recomputation と final initialized
function check であり、この issue は open のまま継続する。

## 2026-05-31 raw alias replay 更新

`ISS-20260531T071945698Z-RAW-ALIAS-SUMMARIES-NEED-STABLE-MIRR-4DCE44A8`
で `RawCellAddressReturnSummary` の stable mirror / preseed cache を実装した。alias が空の
relevant function も empty entry として cache し、no-alias function が worklist に戻る
固定費を削った。

`tmp/rpn_raw_alias_cache_code_edit_20260531.json` では、same-session code edit delta が
次の通りになった。

- `resource_raw_alias_summary_recomputations=38`
- `resource_raw_alias_summary_count=54`
- `resource_summary_value_raw_alias_return_entry_hits=65`
- `resource_summary_value_raw_alias_return_entry_stores=73`
- `resource_summary_value_raw_alias_return_entry_reprojection_value_bypasses=13`
- `resource_raw_init_summary_recomputations=81`
- `resource_initialized_function_checks=13`

raw alias summary の全関数規模再計算は解消したが、edit compile は `compile_ms=7142`
でまだ秒単位である。残る raw alias 側の `reprojection_value` bypass は
`ISS-20260531T075621000Z-RAW-ALIAS-RESIDUAL-REPROJECTION-VAL-9A5D0C3E` に分離し、
この issue は raw-init residual recomputation、raw alias residual reprojection、
式枝差し替え query 化を追跡する親 issue として open のまま継続する。

## 2026-05-31 empty source capability policy 更新

`ISS-20260531T071956084Z-RAW-INIT-RESIDUAL-RECOMPUTATIONS-NEE-C36FBACE`
の部分対応として、capability proof が空の file では source text 全体を source
capability policy hash に混ぜないようにした。関数 semantics は Resource IR body hash
と typed signature/type boundary で検出し、source capability policy は privilege proof
surface の変化だけを見る。

`tmp/rpn_empty_source_policy_raw_init_code_edit_20260531.json` では、same-session code
edit の compile が `7142ms` から `6164ms` へ改善した。edit delta は次の通り。

- `resource_raw_alias_summary_recomputations=32`
- `resource_raw_init_summary_recomputations=73`
- `resource_initialized_function_checks=1`
- `resource_summary_value_raw_init_param_facts_hits=226`
- `resource_summary_value_raw_init_param_facts_stores=48`
- `resource_summary_value_recomputed_ops=29`
- `resource_summary_value_replayed_ops=4509`

raw-init replay bypass は引き続き `0` で、stale hit を避ける fail-closed 境界は維持している。
ただし RPN edit compile はまだ秒単位であり、full function-local capability policy、
raw alias residual reprojection、typed expression subtree query、codegen fragment cache が
残る支配項として継続する。

## 2026-05-31 raw-init residual resolved 更新

function-local source capability policy と raw-init empty entry replay を実装し、
`ISS-20260531T071956084Z-RAW-INIT-RESIDUAL-RECOMPUTATIONS-NEE-C36FBACE` は verified /
resolved にした。

`tmp/rpn_function_local_policy_empty_raw_init_filtered_code_edit_20260531.json` では、same-session code
edit の `resource_raw_init_summary_recomputations=0` になった。raw-init bypass は `0` のまま。
一方で edit compile は `6105ms` で、`resource_raw_alias_summary_recomputations=32`、
`resource_initialized_function_checks=1`、`resource_summary_value_recomputed_ops=29` が残る。
この親 issue は、raw-init 以外の支配項を追跡するため open のまま継続する。

## 2026-05-31 raw alias residual resolved 更新

`ISS-20260531T075621000Z-RAW-ALIAS-RESIDUAL-REPROJECTION-VAL-9A5D0C3E`
で raw alias return entry の projection / type replay 境界を修正した。

`tmp/rpn_raw_alias_projection_type_replay_code_edit_20260531.json` では、same-session code
edit の delta が次の通りになった。

- `resource_raw_alias_summary_recomputations=1`
- `resource_summary_value_raw_alias_return_entry_hits=146`
- `resource_summary_value_raw_alias_return_entry_stores=13`
- `resource_summary_value_raw_alias_return_entry_bypasses=0`
- `resource_summary_value_raw_alias_return_entry_reprojection_value_bypasses=0`
- `resource_raw_init_summary_recomputations=0`
- `resource_initialized_function_checks=1`
- `resource_summary_value_recomputed_ops=16`
- `resource_summary_value_replayed_ops=4522`

raw alias の residual reprojection miss は解消した。RPN edit compile は `compile_ms=5923` でまだ
秒単位だが、raw-init / raw alias fixed-point の false miss は支配項ではなくなっている。
次は typed expression subtree query、変更関数自身の summary 再計算、codegen fragment cache、
および Resource IR summary 外の固定費を分けて追跡する。

## 2026-05-31 stage timing JSON 更新

Web / Node の `CompilerSession.loader_cache_stats_json()` に、直近 compile の
`compile_stage_timings` を追加した。native の `NEPL_COMPILE_STAGE_TIMING=1` だけでは
playground / Node runner の same-session cache miss 後の支配 stage を JSON artifact として
残せないためである。compiled-output cache hit では `[]`、real compile では target precheck から
wasm validation までの stage 配列を返す。`compile_stage_timing_status` は
`not_started` / `cache_hit` / `compiled` / `failed` のいずれかで、stage 配列だけでは曖昧な
cache hit と早期失敗を分ける。Web / Node の clock は `performance.now()` を優先し、
fallback としてだけ `Date.now()` を使う。

`tmp/rpn_stage_timing_same_session_code_edit_20260531.json` では、同じ `CompilerSession` で RPN を
一度 compile した後、`main` 内の表示用 string literal だけを変えた。base は
`compile_ms=12549`、edit は `compile_ms=5779` だった。edit stage timing は次の通り。

- `resource_typecheck=154ms`
- `resource_monomorphize=5ms`
- `resource_static_check=5492ms`
- `codegen_precheck=5ms`
- `wasm_codegen=19ms`
- `wasm_validate=2ms`

同 edit の Resource summary delta は、`resource_summary_value_replayed_ops=4553`、
`resource_summary_value_recomputed_ops=16`、`resource_raw_alias_summary_recomputations=0`、
`resource_raw_init_summary_recomputations=0`、`resource_initialized_function_checks=0` だった。
したがって、現時点の秒単位コストは raw-init / raw-alias / final check の false miss ではなく、
大量の proof replay と Resource static check pipeline の固定費として残っている。
この issue では typed expression subtree query、changed function only の proof replay、
codegen fragment cache、binary intermediate artifact を次の分解対象として継続する。

## 2026-05-31 final check lazy pass 更新

final initialized function check の cache hit を、final cell / collection slot state の
materialized replay ではなく、diagnostic-free / auto-drop-free な checked pass として
戻すようにした。保存時点で diagnostics と auto drop points を持つ関数は no-store に
倒しているため、hit entry は cell gate と drop elaboration に必要な surface だけで
pass として扱える。

`tmp/rpn_lazy_initialized_check_code_edit_20260531.json` では、same-session string literal
edit が `compile_ms=5454`、`resource_static_check=5170ms` だった。直前の
`tmp/rpn_stage_timing_same_session_code_edit_20260531.json` の edit `compile_ms=5779` /
`resource_static_check=5492ms` より小さくなっている。

edit の累積差分は次の通り。

- `resource_raw_alias_summary_recomputations=0`
- `resource_raw_init_summary_recomputations=0`
- `resource_initialized_function_checks=0`
- `resource_summary_value_lazy_pass_hits=288`
- `resource_summary_value_lazy_pass_ops=3639`
- `resource_summary_value_replayed_ops=914`

final check 由来の全関数 final state materialize は削れたが、まだ 0.5 秒未満には届いて
いない。次は stage-local key / dependency closure hash の重複構築、changed function only
proof replay、typed expression subtree query を分けて進める。

## 2026-05-31 dependency closure base hash 更新

Resource summary dependency closure hash を、summary kind tag と kind 非依存の closure
base hash に分離した。closure base hash は reachable dependency closure の function
identity、Resource IR body hash、source capability policy hash、function-local type boundary
を含むため、stale hit を防ぐ invalidation 入力は維持している。

`ResourceSummaryValueCacheContext` は Resource static check stage 内だけで有効な
dependency closure base hash table を持つ。raw alias / i32 scalar / raw-init / final
initialized check が同じ module / dependency graph / function index を何度も問い合わせる場合、
同じ closure を再走査しない。table key は in-memory pointer identity を含むため、永続
artifact へは保存しない。source capability policy input の更新時には table を clear する。

`tmp/rpn_dependency_closure_base_cache_20260531.json` では、same-session string literal edit が
次の結果になった。

- base `compile_ms=9431`、`resource_static_check=8804ms`
- edit `compile_ms=3138`、`resource_static_check=2833ms`
- base から edit への差分は `resource_raw_alias_summary_recomputations=0`
- base から edit への差分は `resource_raw_init_summary_recomputations=0`
- base から edit への差分は `resource_initialized_function_checks=0`
- base から edit への差分は `resource_i32_scalar_summary_recomputations=+7`
- base から edit への差分は `resource_summary_value_recomputed_ops=+16`
- edit 累積値は `resource_summary_value_lazy_pass_hits=288`、`resource_summary_value_replayed_ops=914`

lazy final check checkpoint の edit `compile_ms=5454` / `resource_static_check=5170ms` からは
大きく改善したが、0.5 秒未満 compile / 0.1 秒以下の式枝差し替え budget には届いていない。
この issue は、changed function only proof replay、typed expression subtree query、
stdlib prechecked artifact、codegen fragment cache へ継続する。

## 2026-05-31 preseeded summary record skip 更新

Resource summary cache から worklist 前に replay 済みの entry を、同じ compile の末尾で
candidate として再記録しないようにした。replay 時点で key 作成、stable entry の存在確認、
現在の type / place boundary への fail-closed な再投影が済んでいるため、末尾の candidate 化は
安全性 proof を強めず、同じ stable mirror と dependency closure を再構築する固定費だけを増やす。

対象は raw alias、i32 scalar、raw-init complete leaf、collection slot complete leaf である。
ただし preseed 後に dependent として再び worklist へ入った関数は、再計算済み summary として
通常どおり candidate 化する。`SummaryWorklist::unrecomputed_initial_skips` により、初期 skip
されたまま一度も再計算されなかった関数だけを record から外す。

`tmp/rpn_skip_preseeded_summary_record_20260531.json` では、same-session string literal edit が
次の結果になった。

- base `compile_ms=9403`、`resource_static_check=8789ms`
- edit `compile_ms=2105`、`resource_static_check=1820ms`
- base から edit への差分は `resource_raw_alias_summary_recomputations=0`
- base から edit への差分は `resource_raw_init_summary_recomputations=0`
- base から edit への差分は `resource_initialized_function_checks=0`
- base から edit への差分は `resource_i32_scalar_summary_recomputations=+7`
- base から edit への差分は `resource_summary_value_recomputed_ops=+16`
- edit 累積値は `resource_summary_value_hits=0`
- edit 累積値は `resource_summary_value_replayed_ops=914`
- edit 累積値は `resource_summary_value_lazy_pass_hits=288`

`resource_summary_value_hits` と kind 別 hits は、通常 recompute 後に candidate 化した entry が
既存 stable value と一致した数を表す。preseed replay による実 reuse は
`resource_summary_value_replay_*` と `resource_summary_value_lazy_pass_*` で観測する。

dependency closure base hash checkpoint の edit `compile_ms=3138` / `resource_static_check=2833ms`
からは改善したが、0.5 秒未満 compile / 0.1 秒以下の式枝差し替え budget には届いていない。
この issue は、changed function only proof replay、typed expression subtree query、
stdlib prechecked artifact、codegen fragment cache へ継続する。

## 2026-05-31 recomputed ops kind counter 更新

Aggregate `resource_summary_value_recomputed_ops` を summary kind 別に分解する counter を追加した。
既存 aggregate counter は維持し、追加 counter は raw alias / i32 scalar / raw-init /
collection slot のどの stable mirror が残差再計算を持つかを観測するためだけに使う。

`tmp/rpn_recomputed_ops_kind_counters_20260531.json` では、same-session string literal edit が
次の結果になった。

- base `compile_ms=9237`、`resource_static_check=8625ms`
- edit `compile_ms=3310`、`resource_static_check=3013ms`
- base から edit への差分は `resource_summary_value_recomputed_ops=+16`
- base から edit への差分は `resource_summary_value_i32_scalar_return_facts_recomputed_ops=+16`
- base から edit への差分は raw alias / raw-init / drop traversal の kind 別 recomputed ops が `0`
- base から edit への差分は `resource_summary_value_i32_scalar_return_facts_bypasses=+16`
- base から edit への差分は `resource_summary_value_i32_scalar_return_facts_reprojection_value_bypasses=+16`

このため、残る `+16` は i32 scalar stable mirror の再投影失敗として扱う。
`ISS-20260531T134951396Z-I32-SCALAR-RESIDUAL-REPROJECTION-STI-0F6F5A24` を追加し、
entry / function / reason counter と再投影 surface の root cause を別 issue へ分離した。

一方で edit `resource_static_check` はまだ秒単位であり、i32 residual だけを直しても
0.5 秒未満には届かない。親 issue では changed function only proof replay、typed expression
subtree query、stdlib prechecked artifact、codegen fragment cache を継続する。

## 2026-05-31 i32 scalar reason counter 更新

i32 scalar residual は `ISS-20260531T134951396Z-I32-SCALAR-RESIDUAL-REPROJECTION-STI-0F6F5A24`
側で function / reason counter を追加した。親 issue では seconds-scale compile time の
全体支配項を追うが、i32 scalar stable mirror の false miss を changed-function-only proof
scope の問題と混同しないため、詳細は子 issue に分離して扱う。

`tmp/rpn_i32_residual_reason_counters_20260531.json` では、same-session string literal edit が
次の結果になった。

- base `compile_ms=8932`、`resource_static_check=8347.574ms`
- edit `compile_ms=2102`、`resource_static_check=1816.135ms`
- `resource_summary_value_i32_scalar_return_facts_recomputed_ops=+16`
- `resource_summary_value_i32_scalar_return_facts_reprojection_value_bypasses=+16`
- 内訳は scalar type `+10`、parameter projection `+6`、return projection `0`
- `resource_summary_value_i32_scalar_return_facts_replay_entry_miss_functions=+7`
- `resource_summary_value_i32_scalar_return_facts_misses=0`

この結果から、残差 `+16` は recompute 後の stable entry 化に失敗しており、新規 entry store
ではない。次は i32 stable mirror の parameter projection / scalar type 境界を修正する。
ただし edit compile の支配時間はまだ `resource_static_check=1816.135ms` であるため、親 issue
では引き続き changed-function-only proof replay、typed expression subtree query、stdlib
prechecked artifact、codegen fragment cache を継続する。

## 2026-05-31 i32 scalar stable reprojection partial 更新

`ISS-20260531T134951396Z-I32-SCALAR-RESIDUAL-REPROJECTION-STI-0F6F5A24`
で、i32 scalar stable mirror の projection-derived type replay 境界を raw-init / raw-alias
と同じ方針へ寄せた。構造 projection から現在の function signature 上で型が決まる場合は
現在の signature を使い、raw address terminal deref や open generic 終端だけ保存済み
stable scalar type key を使う。

`tmp/rpn_i32_open_generic_reprojection_code_edit_20260531.json` では、same-session code edit が
次の結果になった。

- base `compile_ms=9231`、`resource_static_check=8606.798ms`
- edit `compile_ms=2126`、`resource_static_check=1841.527ms`
- edit delta は `resource_summary_value_i32_scalar_return_facts_recomputed_ops=+10`
- edit delta の内訳は scalar type `+8`、parameter projection `+2`
- `resource_summary_value_i32_scalar_return_facts_replay_entry_miss_functions=+5`

直前の `+16` からは改善したが、edit compile はまだ秒単位で、base compile も
`resource_static_check=8606.798ms` と 0.5 秒未満から遠い。したがって、この親 issue では
i32 residual を子 issue に残しつつ、base compile 短縮のための stdlib prechecked artifact、
changed-function-only proof replay、typed expression subtree query、codegen fragment cache を
継続する。edit cache の改善だけで完了扱いにはしない。

## 2026-05-31 dependency graph sharing 更新

Resource static check の各 summary kind が同じ `ResourceModule` から dependency /
dependent / initial worklist order を作り直していたため、compile-local な
`ResourceSummaryDependencyGraph` を追加して共有するようにした。これは stale hit を防ぐ
body hash / source capability policy / typed boundary を変更せず、同じ graph construction の
固定費だけを削る。

`tmp/rpn_dependency_graph_share_code_edit_20260531.json` では、`trunk build --release` 後の
Web RPN same-session unused local 追加 edit が次の結果になった。

- base `compile_ms=9246`、`resource_static_check=8193.197ms`
- edit `compile_ms=2135`、`resource_static_check=1857.811ms`
- edit delta は `resource_raw_alias_summary_recomputations=+1`
- edit delta は `resource_i32_scalar_summary_recomputations=+5`
- edit delta は `resource_raw_init_summary_recomputations=0`
- edit delta は `resource_initialized_function_checks=+1`
- edit delta は `resource_summary_value_replayed_ops=+920`
- edit delta は `resource_summary_value_recomputed_ops=+10`

i32 scalar stable reprojection partial checkpoint の edit `compile_ms=2126` /
`resource_static_check=1841.527ms` と同程度で、まだ 0.1 秒以下の式枝差し替え
budget には届かない。この issue は引き続き changed-function-only proof replay、typed
expression subtree query、stdlib prechecked artifact、codegen fragment cache を追跡する。

## 2026-05-31 borrowed worklist dependents 更新

共有 `ResourceSummaryDependencyGraph` から作る `SummaryWorklist` が `dependents` を
clone せず借用するようにした。legacy constructor は owned dependents を保持するため、
既存の direct test 経路は維持している。これは proof key や replay 判定を変えず、同じ逆辺
リストの所有形態だけを変える follow-up である。

`tmp/rpn_borrowed_worklist_dependents_code_edit_20260531.json` では、`trunk build --release`
後の Web RPN same-session unused local 追加 edit が次の結果になった。

- base `compile_ms=9510`、`resource_static_check=8446.129ms`
- edit `compile_ms=2251`、`resource_static_check=1943.803ms`
- edit delta は `resource_raw_alias_summary_recomputations=+1`
- edit delta は `resource_i32_scalar_summary_recomputations=+5`
- edit delta は `resource_raw_init_summary_recomputations=0`
- edit delta は `resource_initialized_function_checks=+1`
- edit delta は `resource_summary_value_replayed_ops=+920`
- edit delta は `resource_summary_value_recomputed_ops=+10`

counter は dependency graph sharing checkpoint と同じ形で、raw-init / raw-alias / final check
の大きな false miss は戻っていない。一方で elapsed time はまだ秒単位であり、次は
changed-function-only proof replay と typed expression subtree query へ進む。

## 2026-05-31 i32 scalar fact kind 更新

`ISS-20260531T134951396Z-I32-SCALAR-RESIDUAL-REPROJECTION-STI-0F6F5A24`
で、i32 scalar residual の `ReprojectionValue` bypass を fact 種別へ分解した。最終測定
`tmp/rpn_i32_fact_kind_counters_final_code_edit_20260531.json` では、same-session unused local
追加 edit が `compile_ms=2219`、`resource_static_check=1922.104ms` だった。

edit delta は `resource_summary_value_i32_scalar_return_facts_recomputed_ops=+10` のままで、
fact kind 内訳は alias `+5` / offset `+5`、reason 内訳は scalar type `+8` /
parameter projection `+2` である。condition 系は `0` だったため、残差は i32 condition
propagation ではなく alias / offset の stable mirror surface として継続する。

この checkpoint は支配時間を直接削るものではない。親 issue では引き続き
changed-function-only proof replay、typed expression subtree query、stdlib prechecked
artifact、codegen fragment cache を進め、base compile `resource_static_check` の秒単位固定費も
別途削る。

## 2026-06-01 owner obligation pass cache 更新

owner obligation の function check は diagnostic-free pass cache として `ResourceSummaryValueCache`
へ保存するようにした。cache key は function body hash、dependency closure hash、source
capability policy hash、type boundary hash を含む。diagnostics を持つ関数は保存せず、cached
pass では `final_owners` を materialize しない。現在の owner obligation gate は diagnostics
だけを authority として読むため、この pass-only replay は後続 stage の入力を弱めない。

`tmp/rpn_owner_obligation_cache_probe_final_20260601.json` では、`trunk build --release` 後の
Web RPN same-session string literal edit が次の結果になった。

- base `compile_ms=10801`、`resource_static_owner_obligations=1780.175ms`
- edit `compile_ms=3006`、`resource_static_owner_obligations=1534.075ms`
- edit delta は `resource_owner_obligation_function_checks=0`
- edit delta は `resource_owner_obligation_function_check_ops=0`
- edit delta は `resource_summary_value_owner_obligation_check_replay_hit_functions=288`
- edit delta は `resource_summary_value_owner_obligation_check_replay_miss_functions=0`

これにより owner checker 本体の全関数再実行は消えた。一方で owner stage はまだ約 1.4 秒残る。
残りは `compute_owner_return_summaries` の全関数固定点計算であり、これは
`ISS-20260601T174500000Z-OWNER-RETURN-SUMMARY-NEEDS-STABLE-CACH-8A7B61D2` として分離した。
`OwnerReturnSummary` は `TypeId` や projection-rich な session-local state を含むため、単純な
in-memory summary reuse ではなく stable mirror value cache として設計する。

## 2026-06-01 owner summary lazy skip 更新

owner obligation pass cache が全関数で hit した compile では、owner return summary を作らずに
diagnostics-free report を返す lazy path を追加した。これは `OwnerReturnSummary` の直接 cache
ではなく、既に body hash / dependency closure hash / source capability policy hash / type
boundary hash で検証された pass entry が全関数に揃っている場合に、現在の gate が消費しない
summary 構築を省く変更である。

1 関数でも pass replay が miss した場合は、従来どおり `compute_owner_return_summaries` を
構築して miss 関数を checker に通す。このため all-hit warm edit の固定費は削れるが、
partial miss 時の owner summary reuse は未解決である。引き続き
`ISS-20260601T174500000Z-OWNER-RETURN-SUMMARY-NEEDS-STABLE-CACH-8A7B61D2` で per-function
stable mirror entry と dependency closure kind を実装する。

`tmp/rpn_owner_summary_lazy_skip_code_literal_20260601.json` では、RPN 実コード文字列 literal edit
が base `compile_ms=10254`、edit `compile_ms=2110` だった。edit の
`resource_static_owner_obligations` は `745.034ms` で、owner obligation pass cache checkpoint の
`1534.075ms` から改善した。edit delta は `resource_owner_return_summary_recomputations=0`、
`resource_owner_return_summary_count=0`、`resource_owner_return_summary_pass_cache_skip_functions=288`、
`resource_owner_obligation_function_checks=0`、`resource_summary_value_owner_obligation_check_replay_hit_functions=288`
である。RPN edit compile はまだ 0.1 秒以下ではないため、次は initialized side の残り固定費、
typed expression subtree query、stdlib prechecked artifact を継続する。

## 2026-06-01 final initialized changed-function plan 更新

final initialized function check の pass-only replay に、compile-local changed-function plan を追加した。前回 compile の diagnostic-free / auto-drop-free pass snapshot と現在の関数 local fingerprint を比較し、関数本文、type boundary、source capability policy、generic boundary が変わった関数から reverse dependents を辿って affected set を作る。

affected ではない関数は dependency closure hash の再構築と通常の replay probe を行わず、前回 pass snapshot の `ResourceCheckDeferred` だけを checked pass として戻す。snapshot は `TypeId`、`Span`、`SourceMap`、final state を保持しない。関数 order、namespace、fingerprint の構築に不整合があれば conservative-all に倒す。

この checkpoint は final check probe 固定費の削減であり、raw alias / i32 scalar / raw-init summary preseed loop にはまだ適用していない。summary fixed-point 側では callee summary materialization が必要なため、次段階で changed-function/dependency closure ごとの replay plan を別途設計する。

`tmp/rpn_final_initialized_pass_plan_20260601.json` では、release Web RPN same-session string literal edit が base `compile_ms=9998`、edit `compile_ms=2178` だった。edit delta は `resource_summary_value_initialized_function_check_plan_skip_functions=288`、`resource_summary_value_initialized_function_check_plan_skip_ops=3639`、`resource_summary_value_initialized_function_check_replay_probe_functions=0` である。

## 2026-06-01 Resource summary changed-function replay plan 更新

raw alias / i32 scalar / raw-init summary preseed loop に changed-function replay plan を追加した。
前回 compile の stable summary key と現在 compile の関数 local fingerprint を比較し、変更関数から
reverse dependents を辿って affected set を作る。affected ではない関数は dependency closure
hash の再構築と通常 replay probe を省くが、caller summary index が必要とするため summary
自体は現在の `TypeCtx` と function signature へ再投影して materialize する。

snapshot は stable key と fingerprint だけを持ち、`TypeId`、`Span`、`SourceMap`、summary
本体は保持しない。関数順序、namespace、source capability policy、body hash、type boundary、
generic boundary が合わない場合や再投影できない場合は通常 path へ戻る。

subagent review で、body hash が `__def{file}_{start}_{end}` 付き symbol をそのまま hash すると
span だけがずれた未変更 caller まで affected になる可能性が指摘された。このため
`ResourceCallTarget::User`、`EffectOp::UserCall`、`FunctionValueIdentity` の body hash 入力を
Resource summary key と同じ定義 span mangle 正規化に揃えた。

`tmp/rpn_summary_replay_plan_code_literal_20260601.json` では、release Web RPN same-session string
literal edit が次の結果になった。

- base `compile_ms=9593`
- edit `compile_ms=1521`
- edit `resource_static_check=1222.739ms`
- edit `resource_static_initialized_moves=301.490ms`
- edit `resource_static_owner_obligations=805.523ms`
- edit `resource_typecheck=163.275ms`
- edit delta は `raw_alias_replay_probe_functions=0`
- edit delta は `raw_alias_plan_skip_functions=288`
- edit delta は `i32_scalar_replay_probe_functions=0`
- edit delta は `i32_scalar_plan_skip_functions=209`
- edit delta は `raw_init_replay_probe_functions=0`
- edit delta は `raw_init_plan_skip_functions=151`
- edit delta は `initialized_function_check_plan_skip_functions=288`

RPN edit compile は `2178ms` から `1521ms` へ改善したが、まだ 0.1 秒以下ではない。
残りは owner obligation lazy path の固定費、typecheck、lowering / effect / borrow stage、
typed expression subtree query 未実装、stdlib prechecked artifact 未実装に分かれる。

## 検証

- RPN same-session code edit の compiled-output miss 測定で、支配 stage と function / summary kind を説明できる JSON を残す。
- 修正後の測定で `compile_ms`、`recomputed_ops`、または特定 stage timing が改善していることを確認する。
