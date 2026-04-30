# stdlib nm / kp / platforms review

対象 commit: `f108cebd`

## nm

`stdlib/nm/parser.nepl` と `stdlib/nm/html_gen.nepl` は gloss/nm markdown processing を担う。以前の raw AST storage から、source を再走査して `StringBuilder` へ出す設計に寄せており、non-Copy AST container を raw storage に置かない点は良い。

良い点:

- JSON/HTML escape classifier に enum と `match` が使われている。
- `str_starts_with` / `str_find` / `str_slice_trim_suffix_cr` など、stdlib string helper を利用する方向へ進んでいる。
- source policy により raw aggregate detour を戻さない監視がある。

問題:

- `parser.nepl` は 824 行、`html_gen.nepl` は 519 行で、block parser / inline parser / JSON/HTML rendering がまだ密である。
- inline delimiter 判定には nested `if` が残る。有限分岐は `match` / enum classifier へさらに寄せる余地がある。
- StringBuilder / `sb_build_result` の owner failure に引きずられる。

## kp

`kp` modules は graph / search / DSU / Fenwick / prefix helper を提供する。教育・競プロ用途として有用だが、selfhost compiler 中核の優先度は collections/mem/string より低い。

注意点:

- 多くは `Vec<i32>` や raw numeric storage に依存する。
- selfhost の algorithm helper として使うなら Copy payload に限定する。

## platforms / features

`features/tui.nepl` と `platforms/wasix/tui.nepl` は terminal UI helper を持つ。ANSI / terminal escape は char literal 導入後も、binary escape byte と text char の区別が必要である。

## 結論

nm は「stdlib 不足による不自然な文字列比較」を減らす改善が進んでいるが、parser/html の責務分割と `match` 化は継続対象である。kp/platforms は selfhost core の blocker ではないが、source policy と char/string API の追従対象として review を続ける。
