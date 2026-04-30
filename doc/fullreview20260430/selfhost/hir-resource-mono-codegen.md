# Selfhost Compiler Review: HIR, Resource, Mono, Codegen

対象 commit: `f108cebd`

## 対象

- `stdlib/neplg2/core/hir/hir.nepl`
- `stdlib/neplg2/core/resource/move_state.nepl`
- `stdlib/neplg2/core/mono/mono.nepl`
- `stdlib/neplg2/core/codegen/wasm/binary.nepl`
- `stdlib/neplg2/core/codegen/llvm/text.nepl`
- `stdlib/neplg2/core/pipeline.nepl`
- `stdlib/neplg2/core/options.nepl`

## 設計評価

HIR は flat table と stable id を持つ方向にあり、deep traversal を再帰ではなく explicit stack へ移す前提をコメントで明記している。これは selfhost compiler で stack safety を確保する方向として妥当である。

しかし S4 の中核である Resource IR / move / borrow / drop はまだ実装されていない。`resource/move_state.nepl` は stage0 marker の段階であり、Rust 側の Resource IR final authority と同等の安全性はない。selfhost で最も妥協してはいけない範囲なので、ここを急いで旧 move_check 風に作るのは危険である。

mono は instance key と seed があるが、generic type arg table、instance cache、mangling、layout とは未接続である。codegen は WASM/LLVM とも placeholder に近く、S5 には未到達である。

## Actions 根拠

Actions run `25157230630` では HIR/type/resource 周辺に次の selfhost failure がある。

- `core/hir/hir.nepl::doctest#1`: `sb_build_result` owner maybe leak
- `core/hir/hir.nepl::doctest#2`: `resource.owner.reserved` on local `child_id`
- `core/hir/hir.nepl::doctest#3`: `params0` owner maybe leak
- `core/ty/ty.nepl::doctest#1`: arena owner maybe leak
- `core/pipeline.nepl::doctest#1`: timeout

この review では local runtime test ではなく Actions artifact/log を根拠にした。

## 良い点

- HIR id は `SelfhostHirFunctionId` / `SelfhostHirExprId` の typed wrapper になっている。
- HIR module は function / param / expr / child table を分けており、backend 前の stable id 参照に向いている。
- mono key は `SelfhostMonoDefId` と type arg range を分けている。
- options は target / profile を enum として扱う方向。

## 問題

- HIR の doctest が strict owner gate で失敗しており、table construction の ownership contract がまだ安定していない。
- `resource/move_state.nepl` は実質未実装で、move/borrow/drop の final authority になっていない。
- HIR lowering が存在しないため、AST/typecheck result から HIR へ接続できない。
- mono は instance cache / substitution / layout / codegen naming と接続されていない。
- WASM / LLVM codegen は実行可能 backend ではない。
- pipeline は load root までで、S3-S5 を統合する compile pipeline ではない。

## 必要な設計

- selfhost Resource IR は Rust 側の `doc/neplg2/static_check_complexity_reduction_plan.md` と diagnostic redesign の方針を反映する。
- move/borrow/drop は HIR 後付け special-case ではなく、Resource IR operation と owner/path/state merge を正とする。
- owner state / cell state / value condition は enum と typed ID で扱い、数値 sentinel や string key にしない。
- branch merge、match exhaustiveness、Result variant refinement、drop authority は first-class pass にする。
- codegen は static check 完了済み HIR / Resource plan だけを入力にする。

## 進捗状況

- `core/hir/hir`: 初期実装。flat HIR table はあるが owner failure が残る。
- `core/resource/move_state`: 未実装相当。
- `core/mono/mono`: 初期実装。key/id helper。
- `core/codegen/wasm/binary`: 未実装相当。
- `core/codegen/llvm/text`: 未実装相当。
- `core/pipeline`: 初期実装。load root まで。
- `core/options`: 実装中。target/profile/options。

## 判定

S4-S5 はまだ通常実装へ進むべきではない。Rust 側 Resource IR が安定した範囲を設計として取り込み、selfhost でも型安全・メモリ安全が static check で保証される構造にしてから実装する必要がある。
