# Rust コンパイラ静的検査 / ResourceIR レビュー

確認対象 commit: `e8a4e399 docs(review): add check ResourceIR gate issue`

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
| check-only API | `nepl-cli --check` は ResourceIR gate を通らない。 | 新規 issue 化済み。メモリ安全必達の観点では P1 blocker。 |

## 良い点

- ResourceIR が ownership / lifetime / effect / initialized-state の authority として明示されている。
- compiler pipeline に旧 `passes::move_check` と旧 `passes::insert_drops` を戻さない source policy がある。
- diagnostic code は `resource.cell.*` / `resource.owner.*` / `resource.borrow.*` / `resource.lower.*` に分類され、粗い bucket に戻っていない。
- `ResourceDropElaborationPlan` を HIR drop insertion の入力にしており、candidate plan や HIR scope walker を再 authority にしない方向へ進んでいる。
- recent regression は full compiler gate と `.n.md` compile_fail の両方で owner diagnostic 到達を確認しており、単体 ResourceIR fixture だけに閉じていない。

## 問題

### `--check` が ResourceIR を通らない

`check_module_with_source_map` は `run_typecheck` 後に成功を返す。現在の compile preparation が ResourceIR を safety authority にしているため、`--check` が成功しても memory/resource safety を確認したことにならない。これは `ISS-20260507T143850332Z-CLI-CHECK-DOES-NOT-RUN-RESOURCEIR-ME-D1F139FF` として追加した。

修正方針は、artifact を emit しない shared prepare/check phase を作り、target precheck、typecheck、ResourceIR 用 monomorphize、coverage、cell、drop plan、borrow、effect、owner、HIR bridge までを check-only でも実行すること。過去の deep HIR stack overflow を再発させないため、旧 artifact pipeline へ雑に戻すのではなく、現在の非再帰化された prepare phase を共有する必要がある。

### raw memory boundary が移行用 capability として残る

typecheck の raw body effect check と ResourceIR gate は、source map の raw memory boundary を見て一部 migration source を許可する。これは stdlib 内部実装を段階移行するには必要だが、最終設計では public safe API と internal raw API の境界を型で表すべきである。

### 責務分割の監視範囲が完全ではない

`test_resource_checker_responsibility.js` は主要 module の存在と一部 line limit を監視している。現在の確認では `lower.rs`、`owner_check.rs`、`effect_check.rs`、`lower_aggregate_projection.rs` などは上限内だった。一方、`owner_variant.rs`、`dump.rs`、`cell_state.rs`、`owner_control.rs`、`initialized_control.rs` のように大きいが line limit 対象外の module もある。直ちに不正とは断定しないが、ResourceIR の証明 boundary は肥大化しやすいため、個別レビューで「大きい理由が責務上必要か」を見る必要がある。

## issue 連携

- `ISS-20260507T143850332Z-CLI-CHECK-DOES-NOT-RUN-RESOURCEIR-ME-D1F139FF`: 新規。`--check` に ResourceIR gate がない。
- `ISS-20260427T152958303Z-MEMPTR-AND-REGIONTOKEN-LACK-COMPILER-0BC8ECDF`: open。pointer provenance の最終設計。
- `ISS-20260429T040748194Z-RUST-COMPILER-DIAGNOSTICS-ARE-NOT-AL-1617747D`: open。diagnostics taxonomy と ResourceIR/selfhost model の整合。

## 次に確認すること

- `stdlib/core/mem.nepl` と raw-memory-backed stdlib API が、ResourceIR の owner/effect/cell model と同じ不変条件を型で表せているか。
- selfhost typecheck / ResourceIR 設計が Rust 実装の旧 HIR 直走査 special-case を移植しない計画になっているか。
- `--check` issue の修正時に、deep HIR stack overflow 回避と ResourceIR gate 実行を両立する regression が追加されるか。
