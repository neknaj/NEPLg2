# selfhost core infra review

確認対象 commit: `31291b37 fix(core): add parser backend responsibility policy`

## 確認対象

- `stdlib/neplg2/core/infra/diag.nepl`
- `stdlib/neplg2/core/infra/outcome.nepl`
- `stdlib/neplg2/core/infra/span.nepl`
- `stdlib/neplg2/core/infra/text.nepl`
- `stdlib/neplg2/core/options.nepl`
- `stdlib/neplg2/core/pipeline.nepl`
- `doc/neplg2/compiler_diagnostics_redesign_plan.md`

## 良い点

diagnostic は `SelfhostDiagnosticCode` を階層 enum として持ち、stable string は `selfhost_diag_code_name` 境界でのみ生成する。これは Rust 側 diagnostic redesign の code-first 方針と整合している。

`SelfhostDiagnostic` は severity、code、message、primary label、note を値として保持し、CLI reporter へ表示責務を分離している。filesystem/stdout/stderr に依存しないため core compiler の pure boundary に置ける。

`SelfhostOutcome<T,E>` は result と diagnostics を分離して運ぶ。以前の raw pointer cell 依存から owned field へ寄せており、payload cleanup callback も明示されている。

`options.nepl` は target/profile を enum と `Option` で管理し、CLI alias 文字列を core へ持ち込まない。`pipeline.nepl` は VFS root load 入口だけを持ち、CLI driver が loader を直接組み立てすぎない境界になっている。

## 問題とリスク

`SelfhostDiagnosticCode` の taxonomy は S1/S2 向けで、`Type`、`Effect`、`Resource`、`Borrow`、`Drop`、`Mono`、`Backend` の code 階層がまだない。S3 以降で自由文字列や generic parser code に逃がすと、Rust/selfhost diagnostic parity と `compiler_diagnostics_redesign_plan.md` の方針から外れる。

`span` と `text` は Copy-friendly な foundation だが、line/column source map と multi-file diagnostic rendering はまだ薄い。CLI reporter は `file_id:start..end` に近い出力で、Rust compiler と同等の source label には達していない。

`pipeline.nepl` は root load までで、typecheck/resource/codegen の stage result composition をまだ持たない。S3 以降では stage ごとに `Result` と diagnostic collection の ownership rule を統一し、error path で AST/HIR/resource owner を落とさない設計が必要である。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `core/infra/diag.nepl` | enum-first diagnostic code、mandatory code、stable string match。 | 良い。S3+ taxonomy 追加が必要。 |
| `core/infra/outcome.nepl` | Result + diagnostics の owned field model。 | 良い。stage composition はこれから。 |
| `core/infra/span.nepl` | file_id/start/end の基本 span。 | S1/S2 には十分。line map は不足。 |
| `core/infra/text.nepl` | byte/line helper。 | lexer/parser supportとして有効。 |
| `core/options.nepl` | target/profile enum + Option。 | 良い。 |
| `core/pipeline.nepl` | root VFS load entry。 | S2まで。S3+ pipeline未接続。 |

## 推奨対応

- Rust 側 diagnostic redesign 後の code taxonomy に合わせ、selfhost でも `Type` / `Resource` / `Backend` 系 code enum を先に定義する。
- diagnostic code の追加時は stable string match に wildcard fallback を置かない。
- source map と reporter は、span の byte range だけでなく line/column/label/note を JSON と human の両方で検査できる形へ拡張する。
- stage outcome は、error path の cleanup callback を個別に散らさず、AST/HIR/diagnostic owner を扱う typed stage result として統一する。
