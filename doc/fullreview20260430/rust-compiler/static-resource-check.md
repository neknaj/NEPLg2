# Rust コンパイラ静的検査 / ResourceIR レビュー

確認対象 commit: `3742a1a7 fix(cli): run Resource IR gates for check-only`

## 確認範囲

- `nepl-core/src/compiler.rs`
- `nepl-core/src/resource/**`
- `nepl-core/src/passes/drop_insertion.rs`
- `nepl-core/tests/resource_ir.rs`
- `tests/stdlib/memory_safety.n.md`
- `nodesrc/test_static_check_boundary_responsibility.js`
- `nodesrc/test_resource_gate_order.js`
- `nodesrc/test_resource_checker_responsibility.js`
- `nodesrc/test_resource_ir_test_harness_policy.js`

このレビューは、型安全・メモリ安全を ResourceIR が静的検査できる構造になっているか、旧 HIR 直接検査へ戻る経路がないか、enum / match による検査可能性が保たれているかを確認した。

## 現在の pipeline

`prepare_module_for_codegen_with_source_map` は次の順序で source semantics を検査している。

1. target/profile precheck
2. typecheck
3. ResourceIR 用 monomorphize
4. `run_resource_static_check`
5. drop elaboration HIR bridge validation
6. `passes::insert_resource_drops`
7. codegen 用 monomorphize

`run_resource_static_check` は、ResourceIR lowering 後に coverage、initialized cell、drop elaboration plan、borrow lifetime、effect boundary、owner obligation の gate を通す。`nodesrc/test_resource_gate_order.js` は、この順序を「drop-free source semantics を検査してから generated drops を挿入する」責務境界として固定している。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| ResourceIR model | `ResourceOp`、`CellState`、`OwnerState`、`BorrowState`、`StorageOrigin`、`EffectOp` が enum で表現されている。 | 方針に合う。数値 sentinel ではなく typed state と match で検査できる。 |
| lowering coverage | HIR と ResourceIR の coverage 比較 gate があり、最近の `region_ptr` helper regression も coverage と owner diagnostic の到達を固定した。 | 強化中。coverage が owner/cell 診断を覆い隠さないことを引き続き監視する。 |
| initialized cell | 通常 read/move/drop/call/construct/return と raw memory cell を ResourceIR cell gate で扱う。 | old move checker 依存から前進。summary 系 module が多く、責務分割の維持が重要。 |
| owner obligation | `NoFreeObligation`、`Reserved`、`UseAfterMove`、`DoubleFree`、`Leak` などが enum diagnostic へ対応している。 | 中核は実装済み。`MemPtr` / `RegionToken` の最終 provenance model は open。 |
| borrow lifetime | borrow checker は ResourceIR 側へ分離され、operation / state に応じた diagnostic mapping を持つ。 | 実装済み。selfhost S3 以降で function boundary と aggregate projection の追加確認が必要。 |
| effect boundary | raw memory / external IO / nondet / indirect call の effect count と pure boundary gate がある。 | 方針に合う。raw memory boundary の暫定許可を最終設計へ固定しないこと。 |
| drop elaboration | `ResourceDropElaborationPlan` を作り、HIR bridge で検証してから `insert_resource_drops` が消費する。 | 旧 VarState walker から脱却している。plan が checked live fact 由来であることは維持する。 |
| source policy | `test_static_check_boundary_responsibility.js` と `test_resource_checker_responsibility.js` が旧 move_check 再導入や巨大化を監視する。 | 有効。line limit は現在おおむね収まっているが、未監視の大きな module も残る。 |
| check-only API | `nepl-cli --check` は `3742a1a7` で compile preparation を共有し、ResourceIR gate と drop insertion bridge を実行する。 | fixed。artifact emission なしで safety authority を共有する regression が追加された。 |

## 良い点

- ResourceIR が ownership / lifetime / effect / initialized-state の authority として明示されている。
- compiler pipeline に旧 `passes::move_check` と旧 `passes::insert_drops` を戻さない source policy がある。
- diagnostic code は `resource.cell.*` / `resource.owner.*` / `resource.borrow.*` / `resource.lower.*` に分類され、粗い bucket に戻っていない。
- `ResourceDropElaborationPlan` を HIR drop insertion の入力にしており、candidate plan や HIR scope walker を再 authority にしない方向へ進んでいる。
- recent regression は full compiler gate と `.n.md` compile_fail の両方で owner diagnostic 到達を確認しており、単体 ResourceIR fixture だけに閉じていない。
- review 中に remote main へ `cd44312f fix(resource): preserve region_ptr_at non-owning provenance` が入り、`region_ptr_at` の `Result::Ok(MemPtr<U>)` payload が owner token に昇格しないことも ResourceIR / `.n.md` regression で固定された。

## 問題

### `--check` ResourceIR gate は修正済み

`check_module_with_source_map` は `3742a1a7` で `prepare_module_for_codegen_with_source_map` を共有する形に修正された。これにより target precheck、typecheck、ResourceIR 用 monomorphize、coverage、cell、drop plan、borrow、effect、owner、HIR bridge、drop insertion までを check-only でも実行する。成果物 emission は行わないため、過去の deep HIR stack overflow 回避と safety gate 実行を両立している。

この項目は `ISS-20260507T143850332Z-CLI-CHECK-DOES-NOT-RUN-RESOURCEIR-ME-D1F139FF` として fixed になった。今後は `check_module_with_source_map` が再び typecheck-only convenience API へ退行しないことを regression で監視する必要がある。

### raw memory boundary が移行用 capability として残る

typecheck の raw body effect check と ResourceIR gate は、source map の raw memory boundary を見て一部 migration source を許可する。これは stdlib 内部実装を段階移行するには必要だが、最終設計では public safe API と internal raw API の境界を型で表すべきである。

### 責務分割の監視範囲が完全ではない

`test_resource_checker_responsibility.js` は主要 module の存在と一部 line limit を監視している。現在の確認では `lower.rs`、`owner_check.rs`、`effect_check.rs`、`lower_aggregate_projection.rs` などは上限内だった。一方、`owner_variant.rs`、`dump.rs`、`cell_state.rs`、`owner_control.rs`、`initialized_control.rs` のように大きいが line limit 対象外の module もある。直ちに不正とは断定しないが、ResourceIR の証明 boundary は肥大化しやすいため、個別レビューで「大きい理由が責務上必要か」を見る必要がある。

## issue 連携

- `ISS-20260507T143850332Z-CLI-CHECK-DOES-NOT-RUN-RESOURCEIR-ME-D1F139FF`: fixed。`--check` が ResourceIR gate を共有する。
- `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF`: open。pointer provenance の最終設計。
- `ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D`: open。diagnostics taxonomy と ResourceIR/selfhost model の整合。
- `ISS-20260507T143247279Z-RESOURCE-IR-OWNER-CHECKER-LOSES-NON--66D5734F`: fixed。`region_ptr_at` Ok payload の non-owning provenance regression。

## 次に確認すること

- `stdlib/core/mem.nepl` と raw-memory-backed stdlib API が、ResourceIR の owner/effect/cell model と同じ不変条件を型で表せているか。
- selfhost typecheck / ResourceIR 設計が Rust 実装の旧 HIR 直走査 special-case を移植しない計画になっているか。
- `--check` regression が Actions で完了し、deep HIR stack overflow 回避と ResourceIR gate 実行を継続的に固定できているか。
