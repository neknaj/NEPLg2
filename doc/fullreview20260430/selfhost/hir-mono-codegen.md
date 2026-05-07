# selfhost HIR mono codegen review

確認対象 commit: `c5f93163 fix(selfhost): split hir expr payloads`

## 確認対象

- `stdlib/neplg2/core/hir/hir.nepl`
- `stdlib/neplg2/core/mono/mono.nepl`
- `stdlib/neplg2/core/codegen/wasm/binary.nepl`
- `stdlib/neplg2/core/codegen/llvm/text.nepl`

## 良い点

`hir.nepl` は flat table model を持ち、function、expr、child range、param range を ID/range として扱う方向に進んでいる。`6277239` で child/param range は `Empty` / `Range` payload へ分離され、`8ff05570` で expr absence は `Option<SelfhostHirExprId>` へ移った。これは selfhost で AST から typed IR へ移る基盤として必要である。

remote main の `0fcc4839` により、`SelfhostHirExprKind` equality helper は numeric tag 比較から direct nested match へ改善された。variant 追加時に equality helper の更新漏れを source policy で検出する形になった点は前進である。

`mono.nepl` は generic instance key、type arg range、deterministic seed を持つ。`b9e85f23` で instance ID の未割当は `Option<SelfhostMonoInstanceId>` へ移り、`SelfhostMonoInstanceId` 自体は stable table index に限定された。cache や symbol naming に使う identity model の入口として前進している。

`codegen/wasm/binary.nepl` と `codegen/llvm/text.nepl` はまだ placeholder に留まる。これは S5 未着手として妥当で、type/resource/mono が未確定な段階で backend を先に進めない判断は正しい。

## 問題とリスク

`SelfhostHirExprId(-1)` は `8ff05570` で解消された。ただし `SelfhostHirExpr` は kind に関係なく `first_child`、`child_count`、`name`、`int_value`、`bool_value` を同じ record に持つため、variant-specific payload の検査が効かない。enum equality の numeric tag 問題、child/param range の empty sentinel、expr ID の invalid sentinel は解消したが、expression payload model の問題は残る。

この HIR model のまま type/resource/codegen を積むと、どの expression kind がどの payload を持つかを match で保証できず、invalid field を後続 stage が読む危険が残る。

`mono.nepl` の `SelfhostMonoInstanceId(-1)` は `b9e85f23` で解消された。今後の risk は cache lookup の実装時に `Option<SelfhostMonoInstanceId>` / cache state enum を維持できるかであり、public invalid constructor や `is_valid` helper を再導入してはならない。

codegen は placeholder なので、WASM/LLVM layout、drop elaboration、panic/diagnostic lowering、string/collection ABI は未確認である。Rust backend の巨大 file 分割 issue と同じく、selfhost backend も最初から責務境界を分ける必要がある。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `hir/hir.nepl` | flat table HIR skeleton。kind equality は direct match 化済み。child/param range は `Empty` / `Range`、expr absence は `Option<SelfhostHirExprId>` へ分離済み。 | expression payload を variant-specific に直してから S3+ 拡張。 |
| `mono/mono.nepl` | generic instance key/seed。instance absence は `Option<SelfhostMonoInstanceId>` 化済み。 | key modelは良い。cache 実装時に typed absence を維持する。 |
| `codegen/wasm/binary.nepl` | placeholder。 | S5未着手。 |
| `codegen/llvm/text.nepl` | placeholder。 | S5未着手。 |

## 推奨対応

- HIR expression payload は `c5f93163` で `SelfhostHirExprPayload` enum へ分離済み。今後 expression kind を増やす場合も、flat field と shared placeholder を再導入せず payload variant と match accessor を増やす。
- empty child/param range の `Empty` / `Range` payload 分離を維持し、退行を source policy で監視する。
- Mono instance cache は invalid ID を返さず、lookup result の `Option` / `Result` 表現を維持する。
- WASM/LLVM backend は text/binary emitter、layout、symbol naming、drop lowering、diagnostic loweringを分け、Rust backend の巨大 file 問題を再発させない。
