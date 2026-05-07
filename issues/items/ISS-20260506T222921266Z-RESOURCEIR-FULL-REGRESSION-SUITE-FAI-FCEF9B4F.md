---
id: ISS-20260506T222921266Z-RESOURCEIR-FULL-REGRESSION-SUITE-FAI-FCEF9B4F
title: "ResourceIR full regression suite fails on origin/main baseline"
area: core
status: fixed
resolved: true
priority: P1
type: bug
created: 2026-05-06
updated: 2026-05-07
target: "nepl-core/src/resource, nepl-core/tests/resource_ir.rs"
---

# ISS-20260506T222921266Z-RESOURCEIR-FULL-REGRESSION-SUITE-FAI-FCEF9B4F: ResourceIR full regression suite fails on origin/main baseline

## 概要

On clean origin/main 3ba24e72, cargo test -p nepl-core --test resource_ir -- --nocapture fails 10 existing ResourceIR tests: branch borrow merge, returned raw address helper/function/aggregate-field alias, borrowed region ptr alias, literal zero offset raw helper, unknown-offset region dealloc, lowering skeleton dump, double dealloc, and returned aggregate raw cell owner field alias. The current owner-summary fix branch shows the same 10 baseline failures while its newly added tests pass.

## 対象

- `nepl-core/src/resource, nepl-core/tests/resource_ir.rs`

## 根拠

- 2026-05-07 時点で full ResourceIR suite は段階的に 217 passed / 11 failed まで縮小していたが、owner summary 系の残件により green ではなかった。
- `ISS-20260507T124325905Z-RESOURCE-OWNER-SUMMARY-MISSES-STRUCT-D34092E5` の修正後、`cargo test -p nepl-core --test resource_ir -- --nocapture` は 228 passed / 0 failed になった。

## 問題

On clean origin/main 3ba24e72, cargo test -p nepl-core --test resource_ir -- --nocapture fails 10 existing ResourceIR tests: branch borrow merge, returned raw address helper/function/aggregate-field alias, borrowed region ptr alias, literal zero offset raw helper, unknown-offset region dealloc, lowering skeleton dump, double dealloc, and returned aggregate raw cell owner field alias. The current owner-summary fix branch shows the same 10 baseline failures while its newly added tests pass.

## 影響

A non-clean ResourceIR baseline weakens static-check verification for type and memory safety. New owner/raw-address changes cannot rely on the full ResourceIR suite as a regression signal, and real regressions can be hidden among known failures.

## 修正方針

Triage the 10 failures by root cause and fix them without expected-fail masking. Preserve explicit raw-address proof requirements, restore initialized-cell/owner summaries for legitimate raw pointer flows, and repair borrow merge reporting so memory and borrow safety diagnostics remain authoritative.

## 検証

cargo test -p nepl-core --test resource_ir -- --nocapture must pass on clean origin/main; keep focused regressions for each root cause and run source-policy resource checker guards.

## 2026-05-07 Agent 2 progress: freed raw owner alias

原因:

- `resource_ir_owner_check_reports_double_dealloc` は `alloc_raw` の owner を `%p` に移したあと、1 回目の `dealloc_raw %p` で owner state は `Freed` になっていた。
- しかし `release_owner` が free 成功時に raw alias を消していたため、2 回目の `dealloc_raw` 用に `%p` を read した temporary が `%p` の `Freed` state へ解決できず、診断なしで通っていた。
- free 後も pointer value の provenance は stale use の診断に必要であり、owner obligation は `Freed` state で表現すべきである。raw alias を消すと「解放済み owner」ではなく「証明のない ordinary i32」に落ちてしまう。

修正:

- `release_owner` の successful free path は owner state を `Freed` に更新しつつ raw alias を保持するようにした。
- 既存 `resource_ir_owner_check_reports_stale_owned_alias_dealloc_after_free` は、stale alias が canonical freed owner `%p` へ解決されることを期待する形へ更新した。これは従来の `NoFreeObligation` より正確な診断である。
- `resource_ir_owner_check_reports_double_dealloc` には失敗時に diagnostics と ResourceIR dump を出す message を追加し、再発時の切り分けを容易にした。

検証:

- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_reports_double_dealloc -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_reports_stale_owned_alias_dealloc_after_free -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir resource_ir_owner_check_allows_raw_pointer_read_before_dealloc -- --nocapture`: passed
- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 194 passed / 9 failed。double dealloc は解消済み。残りは borrow merge、raw address proof/test drift、lowering dump、returned aggregate raw-cell owner alias。

## 2026-05-07 Agent 2 fixed: ResourceIR full baseline

原因:

- `region_token_ptr_ref` が `RegionToken` 内の `MemPtr.raw` と返却された `&MemPtr<T>` の参照先 raw field を結び付けておらず、`region_ptr_at` / `region_ptr` の borrowed region pointer owner が `dealloc_region` まで追跡できなかった。
- raw-address return summary の symbolic offset が callee parameter のまま caller へ適用され、`region_ptr_at token 0` のような既知 0 offset が base address と alias しなかった。
- `RawAddressAlias` は semantic metadata なのに coverage が alias target 内の synthetic `Deref` まで HIR deref coverage として数え、正しい lowering を `Lower(Incomplete)` と誤診断していた。
- helper/aggregate/function-value の raw-address alias 回帰テストの一部が、以前の緩い挙動に依存して ordinary `i32` literal を raw pointer proof として使っていた。これは raw-address proof を明示する設計に反する。
- borrow merge テストは、branch 内で作った borrow token が後続で使われないため non-lexical lifetime として解放される正しい挙動を、branch merge failure として期待していた。
- lowering skeleton dump expectation が現在の `end_scope` lowering と一致していなかった。
- aggregate field 経由で raw owner を返すケースでは、i32 aggregate field read が raw address identity を保持せず、さらに branch merge の owner canonicalization が `Deref` cell より scalar alias を優先するため、returned aggregate の raw-cell owner が一時 place 側へ漏れていた。

修正:

- `region_token_ptr_ref` の core mem wrapper semantics に `RawAddressAlias` を追加し、`RegionToken.ptr.raw` から返却 reference target の `MemPtr.raw` へ provenance を渡すようにした。
- raw-address return summary に callee parameter list を持たせ、summary 適用時に symbolic offset を actual arg / known i32 fact / caller symbolic place へ置換するようにした。
- symbolic/unknown offset prefix が implicit zero base と may-overlap する場合を `CellTable` 側で保守的に扱うようにした。
- `RawAddressAlias` の coverage は unknown place 検出だけを維持し、synthetic deref projection を HIR deref count に含めないようにした。
- stale test fixture は `alloc_raw` 由来の explicit raw pointer proof を使うように更新し、ordinary `i32` を raw proof として復活させない方針を維持した。
- borrow merge test は branch 後に token を読む形にし、本当に live borrow が merge されるケースを検査するようにした。
- skeleton dump expectation に `end_scope` 出力を追加した。
- aggregate i32 field read は raw address identity を保つ構造的 read として initialized / owner checker の alias table へ明示し、owner-cell canonicalization では `Deref` を scalar alias より優先するようにした。
- scalar alias 側に正規化された raw-cell owner も、alias 先 suffix に raw-cell projection がある場合は aggregate return/move の aliased descendant として扱うようにした。

検証:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 203 passed / 0 failed
- focused tests:
  - `resource_ir_owner_check_reports_double_dealloc`
  - `resource_ir_owner_check_reports_stale_owned_alias_dealloc_after_free`
  - `borrowed_region_ptr`
  - `resource_ir_cell_check_preserves_literal_arithmetic_helper_zero_offset`
  - `resource_ir_borrow_merge_rejects_mutation_after_branch_borrow`
  - `resource_ir_lowering_skeleton_tracks_locals_and_dump`
  - `resource_ir_owner_check_returns_aggregate_with_raw_cell_owner_stored_through_field_alias`

## 2026-05-07 Agent 2 rebase integration: bounded raw summary と Result payload alias

原因:

- `origin/main@97d07b31` は raw alias return summary を `SummaryWorklist` 化し、ordinary scalar parameter を raw seed にしない設計へ修正していた。
- この設計は self-host lexer timeout と bogus scalar raw alias を防ぐために必要だが、Result wrapper のような straight-line payload forwarding では、caller 側に既にある raw proof を Ok payload bind へ渡す summary が不足していた。
- plain `i32` identity helper を raw proof source に戻すと timeout 修正を壊すため、raw proof の authority は `RawAddressAlias` / typed wrapper / Result payload summary に限定する必要があった。

修正:

- raw seed summary とは別に、`Result` enum を返す straight-line value forwarder だけを対象にした value projection summary を追加した。
- summary 適用時は caller actual が既に raw alias として tracked されている場合だけ raw proof が伝播するため、ordinary scalar を pointer proof として扱わない。
- manual ResourceIR の plain `i32` helper fixture は、helper call 後に lowering authority 相当の `RawAddressAlias` を明示する形へ更新した。
- `origin/main` の `type_can_seed_raw_address_alias` / `SummaryWorklist` 設計は維持し、Result payload bind の raw field alias だけを補完した。

検証:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 203 passed / 0 failed
- `cargo test -p nepl-core resource::initialized_alias_flow::tests:: -- --nocapture`: passed
- `trunk build`: passed
- `NEPL_TEST_CASE_TIMEOUT_MS=60000 node nodesrc/tests.js -i tmp/probe_lex_empty.n.md --no-tree -o tmp/probe_lex_empty_after_trunk_result_value_summary.json -j 1 --assert-io`: timeout せず既知の `resource.owner.maybe_leak` diagnostic で compile fail

## 2026-05-07 Agent 1 再発確認

`fix/resource-retag-fill-initialized-cell` の検証中に `cargo test -p nepl-core --test resource_ir -- --nocapture` を実行し、現在の `main` baseline でも ResourceIR full suite が再び失敗していることを確認した。

切り分けとして WIP を stash し、clean `main` で `cargo test -p nepl-core --test resource_ir resource_ir_cell_check_preserves_dynamic_fill_origin_across_local_reads -- --nocapture` を実行したところ、同じ `RawMemoryLoadCell` / `resource.cell.uninit` で失敗した。したがって今回の `MemPtr` retag fill summary 修正による新規 regression ではない。

現在観測した full suite の状態:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 213 passed / 15 failed

失敗テスト:

- `resource_ir_cell_check_preserves_dynamic_fill_origin_across_local_reads`
- `resource_ir_cell_check_preserves_dynamic_fill_across_impure_i32_reads`
- `resource_ir_cell_check_rekeys_raw_cells_after_loading_raw_address_cell`
- `resource_ir_cell_check_summarizes_initialized_cells_behind_returned_header_pointer`
- `resource_ir_owner_check_consumes_only_used_aggregate_owner_projection`
- `resource_ir_owner_check_moves_result_payload_field_owner_to_match_bind`
- `resource_ir_owner_check_moves_stored_tail_owner_under_new_raw_node`
- `resource_ir_owner_check_reinitializes_self_update_aggregate_return`
- `resource_ir_owner_check_reinitializes_self_update_fresh_projection_return`
- `resource_ir_owner_check_rejects_mem_ptr_use_before_dealloc_result_refinement`
- `resource_ir_owner_check_rejects_mem_ptr_use_before_realloc_result_refinement`
- `resource_ir_owner_check_returns_aggregate_with_raw_cell_owner_stored_through_field_alias`
- `resource_ir_owner_check_transfers_aggregate_owner_descendants_returned_by_helper`
- `resource_ir_owner_check_transfers_owner_returned_by_function_value`
- `resource_ir_owner_summary_consumes_owned_err_payload_from_unreachable_arm`

優先度は P1 のまま維持する。静的検査大規模修正の完了条件として、これらは expected-fail 化せず root cause ごとに解消する必要がある。

## 2026-05-07 Agent 1 dynamic fill 2 本解消

`ISS-20260506T172012873Z-RESOURCE-IR-DYNAMIC-RAW-ADDRESS-VIEW-77E94B53` の再発修正として、raw address view origin と scalar value origin を分離した。

解消した失敗:

- `resource_ir_cell_check_preserves_dynamic_fill_origin_across_local_reads`
- `resource_ir_cell_check_preserves_dynamic_fill_across_impure_i32_reads`

現在観測した full suite の状態:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 215 passed / 13 failed

残り失敗:

- `resource_ir_cell_check_rekeys_raw_cells_after_loading_raw_address_cell`
- `resource_ir_cell_check_summarizes_initialized_cells_behind_returned_header_pointer`
- `resource_ir_owner_check_consumes_only_used_aggregate_owner_projection`
- `resource_ir_owner_check_moves_result_payload_field_owner_to_match_bind`
- `resource_ir_owner_check_moves_stored_tail_owner_under_new_raw_node`
- `resource_ir_owner_check_reinitializes_self_update_aggregate_return`
- `resource_ir_owner_check_reinitializes_self_update_fresh_projection_return`
- `resource_ir_owner_check_rejects_mem_ptr_use_before_dealloc_result_refinement`
- `resource_ir_owner_check_rejects_mem_ptr_use_before_realloc_result_refinement`
- `resource_ir_owner_check_returns_aggregate_with_raw_cell_owner_stored_through_field_alias`
- `resource_ir_owner_check_transfers_aggregate_owner_descendants_returned_by_helper`
- `resource_ir_owner_check_transfers_owner_returned_by_function_value`
- `resource_ir_owner_summary_consumes_owned_err_payload_from_unreachable_arm`

## 2026-05-07 Agent 1 returned range literal count 解消

`ISS-20260507T121931125Z-RESOURCE-IR-RETURNED-RANGE-SUMMARY-D-AAC77B80` として、returned initialized range summary が callee 内部 literal count を表現できない問題を修正した。

原因:

- `return_byte_ranges` / `param_byte_ranges` は count を projection suffix としてしか持てず、`fill_i32 buf 4 0` のような内部 literal bound は summary 収集時に落ちていた。
- 直接 ResourceIR fixture の一部は `ResourceOffset::Known` を element index として使っていたが、現行設計では byte offset なので `Known(2)` / `Known(1)` の i32 load は非整列 byte offset として未初期化診断になるのが正しい。

修正:

- range summary count source を `RawCellInitializationReturnCount` / `RawCellInitializationParamCount` enum に分離し、projection count と known i32 count を exhaustive `match` で扱うようにした。
- `PlaceRoot::I32Constant(i32)` を追加し、known literal bound を first-class scalar place として `InitializedRawByteRange` に保持できるようにした。
- direct ResourceIR fixture の fixed offset は byte offset に合わせた。

現在観測した full suite の状態:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 217 passed / 11 failed

解消した失敗:

- `resource_ir_cell_check_rekeys_raw_cells_after_loading_raw_address_cell`
- `resource_ir_cell_check_summarizes_initialized_cells_behind_returned_header_pointer`

残り失敗:

- `resource_ir_owner_check_consumes_only_used_aggregate_owner_projection`
- `resource_ir_owner_check_moves_result_payload_field_owner_to_match_bind`
- `resource_ir_owner_check_moves_stored_tail_owner_under_new_raw_node`
- `resource_ir_owner_check_reinitializes_self_update_aggregate_return`
- `resource_ir_owner_check_reinitializes_self_update_fresh_projection_return`
- `resource_ir_owner_check_rejects_mem_ptr_use_before_dealloc_result_refinement`
- `resource_ir_owner_check_rejects_mem_ptr_use_before_realloc_result_refinement`
- `resource_ir_owner_check_returns_aggregate_with_raw_cell_owner_stored_through_field_alias`
- `resource_ir_owner_check_transfers_aggregate_owner_descendants_returned_by_helper`
- `resource_ir_owner_check_transfers_owner_returned_by_function_value`
- `resource_ir_owner_summary_consumes_owned_err_payload_from_unreachable_arm`

残件は owner / returned raw cell summary / result refinement に集中しており、dynamic raw range scalar proof とは別 root cause として扱う。

## 2026-05-07 Agent 1 full ResourceIR suite green

`ISS-20260507T124325905Z-RESOURCE-OWNER-SUMMARY-MISSES-STRUCT-D34092E5` で structured `i32` raw owner projection summary を修正し、stale regression を現在の ResourceIR authority に合わせた。これにより 2026-05-07 時点の full ResourceIR regression suite は全件通過した。

解消した残り失敗:

- `resource_ir_owner_check_consumes_only_used_aggregate_owner_projection`
- `resource_ir_owner_check_moves_result_payload_field_owner_to_match_bind`
- `resource_ir_owner_check_moves_stored_tail_owner_under_new_raw_node`
- `resource_ir_owner_check_reinitializes_self_update_aggregate_return`
- `resource_ir_owner_check_reinitializes_self_update_fresh_projection_return`
- `resource_ir_owner_check_rejects_mem_ptr_use_before_dealloc_result_refinement`
- `resource_ir_owner_check_rejects_mem_ptr_use_before_realloc_result_refinement`
- `resource_ir_owner_check_returns_aggregate_with_raw_cell_owner_stored_through_field_alias`
- `resource_ir_owner_check_transfers_aggregate_owner_descendants_returned_by_helper`
- `resource_ir_owner_check_transfers_owner_returned_by_function_value` は current design に合わせて `resource_ir_owner_check_transfers_aggregate_owner_returned_by_function_value` へ更新
- `resource_ir_owner_summary_consumes_owned_err_payload_from_unreachable_arm`

検証:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`: 228 passed / 0 failed
