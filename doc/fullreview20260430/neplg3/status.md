# NEPLg3 status review

確認対象 commit: `c5f93163 fix(selfhost): split hir expr payloads`

## 確認対象

- `doc/neplg3/README.md`
- `doc/neplg3/spec/*.md`
- `doc/neplg3/impl/*.md`
- `stdlib/neplg3/cli/main.nepl`
- `stdlib/neplg3/core/{ast,diagnostic,parser,span,typecheck}.nepl`

## 良い点

NEPLg3 は `doc/neplg3/spec` と `doc/neplg3/impl` に分かれ、言語仕様と実装設計の責務が明確である。spec は syntax、types、declarations、patterns、effects、memory、traits、modules、stdlib、platform、errors、compiler を持ち、設計目標を広く整理している。

`doc/neplg3/spec/compiler.md` は Resource IR、ownership / borrow / region / drop、内部効果と surface effect の分離を明記している。現行 NEPLg2 の ResourceIR 改修と方向性は近く、selfhost / future compiler の静的検査設計で参照できる。

`doc/neplg3/spec/memory.md` と `effects.md` は、GC なし、内部メモリ操作の Pure 畳み込み、Owned / Linear resource、Drop Elaboration、Region Inference を掲げている。ユーザー提示の「型安全・メモリ安全は必達」と整合する。

## 問題とリスク

`stdlib/neplg3` の実装は placeholder である。`cli/main.nepl`、`core/ast.nepl`、`diagnostic.nepl`、`parser.nepl`、`span.nepl`、`typecheck.nepl` は skip doctest 付きの skeleton で、実際の compiler logic はまだない。NEPLg3 実装を現行 selfhost 進捗として扱ってはいけない。

NEPLg3 spec は、compiler spec の test requirement を `diag_code` / `diag_codes` へ更新し、現行 NEPLg2 の diagnostic code redesign と同じ typed diagnostic enum / stable string code contract に揃えた。将来実装時も数値 ID や自由文字列 code を内部主キーに戻してはならない。

NEPLg3 の memory/effect 仕様は理想に近いが、現行 NEPLg2 selfhost はまだ S1/S2 が中心で、S3/S4 type/resource 実装は未成熟である。NEPLg3 仕様を NEPLg2 selfhost にそのまま前倒しするより、現行 ResourceIR と stdlib safety design を完成させる方が優先である。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `doc/neplg3/spec` | 広い仕様文書あり。 | 方針整理として有効。 |
| `doc/neplg3/impl` | compiler structure と移行戦略あり。 | 設計文書段階。 |
| `stdlib/neplg3/cli` | skeleton。 | 未実装。 |
| `stdlib/neplg3/core/ast` | skeleton。 | 未実装。 |
| `stdlib/neplg3/core/parser` | skeleton。 | 未実装。 |
| `stdlib/neplg3/core/typecheck` | skeleton。 | 未実装。 |
| `stdlib/neplg3/core/diagnostic/span` | skeleton。 | 未実装。 |

## 推奨対応

- NEPLg3 は現時点で仕様・設計文書として扱い、NEPLg2 selfhost の実装進捗と混同しない。
- NEPLg3 diagnostic の追加仕様は `diag_code` / typed diagnostic enum contract に沿って記述する。
- NEPLg3 memory/effect 仕様のうち、現行 NEPLg2 にすぐ役立つ内容は ResourceIR / stdlib safety / selfhost type model の設計レビューへ取り込む。
- `stdlib/neplg3` skeleton を増やす前に、実装開始条件と CI target を明文化する。
