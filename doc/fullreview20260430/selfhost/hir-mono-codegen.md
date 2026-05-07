# selfhost HIR mono codegen review

確認対象 commit: `31291b37 fix(core): add parser backend responsibility policy`

## 確認対象

- `stdlib/neplg2/core/hir/hir.nepl`
- `stdlib/neplg2/core/mono/mono.nepl`
- `stdlib/neplg2/core/codegen/wasm/binary.nepl`
- `stdlib/neplg2/core/codegen/llvm/text.nepl`

## 良い点

`hir.nepl` は flat table model を持ち、function、expr、child range、param range を ID/range として扱う方向に進んでいる。これは selfhost で AST から typed IR へ移る基盤として必要である。

`mono.nepl` は generic instance key、type arg range、deterministic seed を持つ。以前の marker-only 状態からは前進しており、cacheや symbol naming に使う identity model の入口がある。

`codegen/wasm/binary.nepl` と `codegen/llvm/text.nepl` はまだ placeholder に留まる。これは S5 未着手として妥当で、type/resource/mono が未確定な段階で backend を先に進めない判断は正しい。

## 問題とリスク

`hir.nepl` は `SelfhostHirExprId(-1)`、`SelfhostHirChildRange(-1, 0)`、`SelfhostHirParamRange(-1, 0)` を持ち、empty/unset を数値 sentinel で表す。さらに `SelfhostHirExpr` は kind に関係なく `first_child`、`child_count`、`name`、`int_value`、`bool_value` を同じ record に持つため、variant-specific payload の検査が効かない。

この HIR model のまま type/resource/codegen を積むと、どの expression kind がどの payload を持つかを match で保証できず、invalid field を後続 stage が読む危険が残る。

`mono.nepl` は `SelfhostMonoInstanceId(-1)` を持つ。instance ID の未割当は `Option<SelfhostMonoInstanceId>` や pending/cache state enum で表すべきで、public invalid constructor を残すべきではない。

codegen は placeholder なので、WASM/LLVM layout、drop elaboration、panic/diagnostic lowering、string/collection ABI は未確認である。Rust backend の巨大 file 分割 issue と同じく、selfhost backend も最初から責務境界を分ける必要がある。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `hir/hir.nepl` | flat table HIR skeleton。 | payload/sentinel設計を直してから拡張。 |
| `mono/mono.nepl` | generic instance key/seed。 | key modelは良いが invalid IDを直す。 |
| `codegen/wasm/binary.nepl` | placeholder。 | S5未着手。 |
| `codegen/llvm/text.nepl` | placeholder。 | S5未着手。 |

## 推奨対応

- HIR expression は `SelfhostHirExprKind` + shared payload record ではなく、variant-specific payload table または payload enum へ再設計する。
- empty child/param range は `SelfhostHirRange::Empty` / `SelfhostHirRange::Range` のような enum にする。
- Mono instance cache は invalid ID を返さず、lookup result を `Option` / `Result` で表す。
- WASM/LLVM backend は text/binary emitter、layout、symbol naming、drop lowering、diagnostic loweringを分け、Rust backend の巨大 file 問題を再発させない。
