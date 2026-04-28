---
id: ISS-20260428T142719860Z-RESOURCE-CHECKER-IS-BECOMING-A-NEW-M-5F78F7E4
title: "Resource checker is becoming a new monolithic static-check pass"
area: core
status: open
resolved: false
priority: P1
type: architecture
created: 2026-04-28
updated: 2026-04-29
target: "nepl-core/src/resource/borrow_check.rs, nepl-core/src/resource/effect.rs, nepl-core/src/resource/effect_summary.rs, nepl-core/src/resource/initialized.rs, nepl-core/src/resource/mod.rs, nepl-core/src/resource/owner_check.rs, nepl-core/src/resource/shadow.rs, nepl-core/src/resource/summary.rs"
---

# ISS-20260428T142719860Z-RESOURCE-CHECKER-IS-BECOMING-A-NEW-M-5F78F7E4: Resource checker is becoming a new monolithic static-check pass

## 概要

The static-check migration split typecheck.rs and move_check.rs, but Resource IR enforcement is now accumulating cell state, owner obligation, borrow lifetime, effect boundary, summaries, merge helpers, and raw memory cell utilities inside nepl-core/src/resource/check.rs. The file is already 2674 lines after Stage 4 raw storage fixes.

## 対象

- `nepl-core/src/resource/check.rs, nepl-core/src/resource/effect.rs, nepl-core/src/resource/mod.rs`

## 根拠

- [NEPLg2 静的検査の複雑化解消計画](../../doc/neplg2/static_check_complexity_reduction_plan.md)
- 2026-04-28 の Stage 4 raw storage cell 修正後、`nepl-core/src/resource/check.rs` は 2674 行になっている。
- 同じ時点で `nepl-core/src/resource/effect.rs` は 1197 行、`nepl-core/src/resource/lower.rs` は 731 行であり、Resource IR 周辺の責務はすでに分離対象として十分大きい。
- `check.rs` には `ResourceCheckEngine`、`ResourceOwnerCheckEngine`、`ResourceBorrowCheckEngine`、function owner summary、borrow table、owner table、cell table、raw cell helper、merge helper、shadow report が同居している。

## 問題

The static-check migration split typecheck.rs and move_check.rs, but Resource IR enforcement is now accumulating cell state, owner obligation, borrow lifetime, effect boundary, summaries, merge helpers, and raw memory cell utilities inside nepl-core/src/resource/check.rs. The file is already 2674 lines after Stage 4 raw storage fixes.

## 影響

If Stage 4/5 keeps adding checks in one file, the project will recreate the same ad-hoc responsibility concentration that the Resource IR migration is intended to remove. Authoritative Resource IR gating will become harder to audit, and future self-host work will depend on another oversized static-check pass.

## 修正方針

Split resource checking by responsibility before enabling broader authoritative gates: cell_state, owner_obligation, borrow_lifetime, raw_cell, function_summary, state_merge, and shadow_report modules. Keep public exports stable through resource/mod.rs and move tests only when behavior is unchanged.

## 検証

After splitting, run the full resource_ir test suite, rustfmt on the resource modules, node nodesrc/issues.js check, and trunk build. Add a source-level check or documentation note so Resource IR check responsibilities do not collapse back into one file.

## 2026-04-28 place utility split

最初の分割として、Resource checker 内で CellState / OwnerState / BorrowState が共有している `Place` 操作 helper を `nepl-core/src/resource/place_utils.rs` へ切り出した。

- `should_track`
- `raw_memory_cell_place`
- `place_suffix_after_prefix`
- `replace_place_prefix`
- `place_with_suffix`
- `places_overlap`
- `push_unique_place`
- `push_unique_usize`

この commit は behavior を変えず、次に `cell_state` / `owner_obligation` / `borrow_lifetime` を分けるための共通依存を先に作る。issue はまだ open のままとし、`check.rs` から engine と table をさらに分割する。

確認:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `rustfmt --check nepl-core\src\resource\check.rs nepl-core\src\resource\mod.rs nepl-core\src\resource\place_utils.rs`
- `node nodesrc/issues.js check`
- `git diff --check`
- `trunk build`

## 2026-04-28 CellState table split

`CellTable`、raw cell obligation helper、`CellState` merge helper を `nepl-core/src/resource/cell_state.rs` へ分離した。`check.rs` には `ResourceCheckEngine` の traversal / diagnostic emission を残し、cell state storage と projection-aware availability 判定を別 module に移した。

この分割で `check.rs` は 2413 行、`cell_state.rs` は 233 行、`place_utils.rs` は 49 行になった。issue はまだ open のままとし、次は owner obligation / borrow lifetime / function summary の分割を続ける。

確認:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `rustfmt --check nepl-core\src\resource\cell_state.rs nepl-core\src\resource\check.rs nepl-core\src\resource\mod.rs nepl-core\src\resource\place_utils.rs`
- `node nodesrc/issues.js check`
- `git diff --check`
- `trunk build`

## 2026-04-28 OwnerState table split

`OwnerTable` と owner state merge helper を `nepl-core/src/resource/owner_state.rs` へ分離した。`ResourceOwnerCheckEngine` は traversal / diagnostic emission / function summary application に集中し、owner storage id allocation、live entry lookup、descendant owner transfer、branch merge は owner table module の責務にした。

この分割で `check.rs` は 2288 行、`cell_state.rs` は 233 行、`owner_state.rs` は 131 行、`place_utils.rs` は 49 行になった。issue はまだ open のままとし、次は borrow lifetime table / function summary の分割を続ける。

確認:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `rustfmt --check nepl-core\src\resource\owner_state.rs nepl-core\src\resource\check.rs nepl-core\src\resource\mod.rs`
- `node nodesrc/issues.js check`
- `git diff --check`
- `trunk build`

## 2026-04-28 BorrowState table split

`BorrowTable`、borrow token binding、`BorrowState` merge helper を `nepl-core/src/resource/borrow_state.rs` へ分離した。`ResourceBorrowCheckEngine` は traversal / diagnostic emission / function alias application に集中し、borrow token の複製・移動・解放、source state merge、branch/loop/match 後の binding retain は borrow table module の責務にした。

この分割で `check.rs` は 2200 行、`borrow_state.rs` は 211 行、`cell_state.rs` は 256 行、`owner_state.rs` は 143 行、`place_utils.rs` は 59 行になった。issue はまだ open のままとし、次は function summary / engine traversal の分割、または分割後の責務境界を固定する check を追加する。

確認:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `rustfmt --check nepl-core\src\resource\borrow_state.rs nepl-core\src\resource\check.rs nepl-core\src\resource\mod.rs`
- `node nodesrc/issues.js check`
- `git diff --check`
- `trunk build`

## 2026-04-28 post-BorrowState review

BorrowState table split 後に Resource checker 周辺を再確認した。`check.rs` は 2674 行から 2200 行まで縮小し、`place_utils.rs`、`cell_state.rs`、`owner_state.rs`、`borrow_state.rs` へ state table / merge helper は分離済みである。

ただし issue はまだ完了ではない。`check.rs` には次が残っている。

- `ResourceCheckEngine` / `ResourceOwnerCheckEngine` / `ResourceBorrowCheckEngine` の traversal と diagnostic emission。
- `BorrowTokenReturnSummary` / `OwnerReturnSummary` / `OwnerProjectionReturnSummary` の function boundary summary。
- `FunctionAliasTable` と aggregate field alias propagation。
- shadow report public struct と compiler pipeline 接続用の entry point。

また `effect.rs` は 1273 行で、raw identity / raw pointer / function alias / raw memory identity / pointer alias の state table と summary computation が同居している。Stage 5 の authoritative gate を広げる前に、`check.rs` だけでなく effect boundary checker も同じ分割方針で整理する必要がある。

次の候補:

- `function_summary.rs` を作り、borrow / owner の return summary と `FunctionAliasTable` を `check.rs` から分離する。
- `effect_identity.rs` / `effect_alias.rs` を作り、`effect.rs` の raw identity table と pointer alias table を分離する。
- 分割後に source-level responsibility check または doc note を追加し、Resource IR checker が再び単一巨大 pass に戻らないようにする。

## 2026-04-28 FunctionAlias table split

`FunctionAliasTable`、function alias entry、function list dedupe、constructed aggregate field への alias propagation を `nepl-core/src/resource/function_alias.rs` へ分離した。borrow / owner checker は traversal と diagnostic emission を続けて担当し、function value alias state と aggregate field alias propagation は専用 module に閉じた。

あわせて owner field transfer と function alias propagation が共有していた aggregate field place 構築を `place_utils::construct_aggregate_field_place` に移した。これにより function alias module は `check.rs` の private helper に依存せず、owner checker 側も同じ helper を使う。

この分割で `check.rs` は 2068 行、`function_alias.rs` は 102 行、`place_utils.rs` は 95 行になった。issue はまだ open のままとし、次は owner / borrow return summary computation、または `effect.rs` の raw identity / pointer alias table 分割を続ける。

確認:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `rustfmt --check nepl-core\src\resource\function_alias.rs nepl-core\src\resource\place_utils.rs nepl-core\src\resource\check.rs nepl-core\src\resource\mod.rs`
- `node nodesrc/issues.js check`
- `git diff --check`
- `trunk build`

## 2026-04-28 Effect FunctionAlias reuse

`nepl-core/src/resource/effect.rs` に残っていた duplicate `FunctionAliasTable`、function alias entry、dedupe、constructed aggregate field alias propagation を削除し、`function_alias.rs` の共通 module を使うようにした。

raw identity field propagation と pointer alias field propagation も `place_utils::construct_aggregate_field_place` を使うようにし、`effect.rs` 側の duplicate aggregate field place builder を削除した。これにより Stage 4 borrow/owner checker と Stage 5 effect checker が同じ function alias state table と aggregate field place construction を共有する。

この分割で `effect.rs` は 1273 行から 1143 行になった。issue はまだ open のままとし、次は `effect.rs` の raw identity / raw memory identity / pointer alias table の分割、または owner / borrow return summary computation の分割を続ける。

確認:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `rustfmt --check nepl-core\src\resource\effect.rs nepl-core\src\resource\function_alias.rs nepl-core\src\resource\place_utils.rs`
- `node nodesrc/issues.js check`
- `git diff --check`
- `trunk build`

## 2026-04-28 Effect identity state split

`effect.rs` から raw identity table、raw memory identity table、raw pointer alias table、raw memory identity propagation helper、aggregate field propagation helper を `nepl-core/src/resource/effect_identity.rs` へ分離した。

`ResourceEffectBoundaryEngine` は traversal、summary application、diagnostic emission を担当し、raw identity / pointer alias の state storage と merge / prefix replacement は `effect_identity.rs` の責務にした。Stage 5 の raw address escape gate は同じ state table を使い続けるため、検査意味論は変更していない。

この分割で `effect.rs` は 1143 行から 785 行になり、`effect_identity.rs` は 366 行になった。issue はまだ open のままとし、次は owner / borrow return summary computation、effect return summary computation、または shadow report entry point の分割を続ける。

確認:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `rustfmt --check nepl-core\src\resource\effect_identity.rs nepl-core\src\resource\effect.rs nepl-core\src\resource\mod.rs`
- `node nodesrc/issues.js check`
- `git diff --check`
- `trunk build`

## 2026-04-28 Report type split

`ResourceSafetyShadowReport`、Cell / Owner / Borrow check report、deferred counter、diagnostic enum、operation enum を `nepl-core/src/resource/report.rs` へ分離した。

`check.rs` は Resource IR traversal と diagnostic emission の実装を担当し、public report shape と compiler / test 向けの diagnostic data structure は `report.rs` の責務にした。`resource/mod.rs` の public export は維持しているため、外部 API は変えていない。

この分割で `check.rs` は 2068 行から 1875 行になり、`report.rs` は 211 行になった。issue はまだ open のままとし、次は owner / borrow return summary computation、effect return summary computation、または checker engine traversal の分割を続ける。

確認:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `rustfmt --check nepl-core\src\resource\report.rs nepl-core\src\resource\check.rs nepl-core\src\resource\mod.rs`
- `node nodesrc/issues.js check`
- `git diff --check`
- `trunk build`

## 2026-04-29 Function return summary split

`BorrowTokenReturnSummary`、`OwnerReturnSummary`、`OwnerProjectionReturnSummary` と、それらを固定点で計算する owner / borrow return summary logic を `nepl-core/src/resource/summary.rs` へ分離した。

`check.rs` 側には Resource IR traversal、diagnostic emission、summary application を残した。`summary.rs` は既存の owner / borrow checker engine を使って関数境界の戻り値 summary だけを計算する責務に限定し、summary state が `check.rs` の middle section に埋もれ続けないようにした。

この分割で `check.rs` は 1875 行から 1676 行になり、`summary.rs` は 219 行になった。issue はまだ open のままとし、次は engine traversal の分割、または `effect.rs` の return summary / boundary engine の責務整理を続ける。

確認:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `rustfmt --check nepl-core\src\resource\summary.rs nepl-core\src\resource\check.rs nepl-core\src\resource\mod.rs`
- `node nodesrc/issues.js check`
- `git diff --check`
- `trunk build`

## 2026-04-29 Effect return summary split

`RawIdentityReturnSummary`、`RawPointerReturnSummary` と、それらを固定点で計算する raw identity / pointer alias return summary logic を `nepl-core/src/resource/effect_summary.rs` へ分離した。

`effect.rs` 側には boundary traversal、effect count、diagnostic emission、summary application を残した。`effect_summary.rs` は既存の `ResourceEffectBoundaryEngine` を使って関数境界の raw identity / pointer alias propagation summary だけを計算する責務に限定した。

この分割で `effect.rs` は 785 行から 632 行になり、`effect_summary.rs` は 166 行になった。issue はまだ open のままとし、次は effect boundary engine traversal の分割、または `check.rs` 側の engine traversal / shadow report entry point の責務整理を続ける。

確認:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `rustfmt --check nepl-core\src\resource\effect_summary.rs nepl-core\src\resource\effect.rs nepl-core\src\resource\mod.rs`
- `node nodesrc/issues.js check`
- `git diff --check`
- `trunk build`

## 2026-04-29 Shadow entry point split

HIR から Resource IR へ lower して shadow report を組み立てる `check_hir_resource_safety_shadow` を `nepl-core/src/resource/shadow.rs` へ分離した。

`check.rs` 側には `ResourceModule` に対する initialized move / owner obligation / borrow lifetime checker の entry point と engine implementation を残した。これにより HIR lowering coverage、effect boundary checker、shadow report assembly への依存が `check.rs` から外れ、ResourceModule checker と compiler pipeline 接続部の責務境界が明確になった。

この分割で `check.rs` は 1676 行から 1658 行になり、`shadow.rs` は 25 行になった。issue はまだ open のままとし、次は `check.rs` の engine traversal 分割、または `effect.rs` の boundary engine 分割を続ける。

確認:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `rustfmt --check nepl-core\src\resource\shadow.rs nepl-core\src\resource\check.rs nepl-core\src\resource\mod.rs`
- `node nodesrc/issues.js check`
- `git diff --check`
- `trunk build`

## 2026-04-29 Initialized move checker split

`ResourceCheckEngine` と `check_resource_initialized_moves` を `nepl-core/src/resource/initialized.rs` へ分離した。これは Resource IR の `InitializedCell` / moved / dropped state を検査する Stage 4 component であり、owner obligation / borrow lifetime checker とは独立した責務として扱う。

`check.rs` 側には owner obligation と borrow lifetime の engine / entry point を残した。`shadow.rs` と `resource/mod.rs` は `initialized.rs` の public entry point を参照するように更新し、外部 API の `check_resource_initialized_moves` export は維持した。

この分割で `check.rs` は 1658 行から 1054 行になり、`initialized.rs` は 619 行になった。issue はまだ open のままとし、次は owner / borrow checker engine の分割、または effect boundary engine の分割を続ける。

確認:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `rustfmt --check nepl-core\src\resource\initialized.rs nepl-core\src\resource\check.rs nepl-core\src\resource\mod.rs nepl-core\src\resource\shadow.rs`
- `node nodesrc/issues.js check`
- `git diff --check`
- `trunk build`

## 2026-04-29 Borrow lifetime checker split

`ResourceBorrowCheckEngine` と `check_resource_borrow_lifetimes` を `nepl-core/src/resource/borrow_check.rs` へ分離した。これは Resource IR の borrow token / borrow lifetime state を検査する Stage 4 component であり、owner obligation checker とは独立した責務として扱う。

`summary.rs` の borrow token return summary 計算は、新しい `borrow_check.rs` の engine を参照するように更新した。`shadow.rs` と `resource/mod.rs` は `borrow_check.rs` の public entry point を参照するように更新し、外部 API の `check_resource_borrow_lifetimes` export は維持した。

この分割で `check.rs` は 1054 行から 661 行になり、`borrow_check.rs` は 406 行になった。issue はまだ open のままとし、次は owner checker engine の分離、または effect boundary engine の分離を続ける。

確認:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `rustfmt --check nepl-core\src\resource\borrow_check.rs nepl-core\src\resource\check.rs nepl-core\src\resource\mod.rs nepl-core\src\resource\shadow.rs nepl-core\src\resource\summary.rs`
- `node nodesrc/issues.js check`
- `git diff --check`
- `trunk build`

## 2026-04-29 Owner obligation checker split

`ResourceOwnerCheckEngine` と `check_resource_owner_obligations` を `nepl-core/src/resource/owner_check.rs` へ移動し、旧 `nepl-core/src/resource/check.rs` を削除した。これは Resource IR の owner obligation / free obligation state を検査する Stage 4 component であり、initialized / borrow checker とは独立した責務として扱う。

`summary.rs` の owner return summary 計算は、新しい `owner_check.rs` の engine を参照するように更新した。`shadow.rs` と `resource/mod.rs` は `owner_check.rs` の public entry point を参照するように更新し、外部 API の `check_resource_owner_obligations` export は維持した。

これにより Resource IR enforcement の中心だった `check.rs` は消滅し、Stage 4 の主要 checker は `initialized.rs`、`borrow_check.rs`、`owner_check.rs` に分離された。`owner_check.rs` は 661 行で、issue はまだ open のままとし、次は effect boundary engine の分割または責務境界の回帰 guard を続ける。

確認:

- `cargo test -p nepl-core --test resource_ir -- --nocapture`
- `rustfmt --check nepl-core\src\resource\owner_check.rs nepl-core\src\resource\mod.rs nepl-core\src\resource\shadow.rs nepl-core\src\resource\summary.rs`
- `node nodesrc/issues.js check`
- `git diff --check`
- `trunk build`
