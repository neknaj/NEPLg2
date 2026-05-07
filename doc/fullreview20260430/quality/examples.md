# examples review

確認対象 commit: `c5f93163 fix(selfhost): split hir expr payloads`

## 確認対象

- `examples/bf.nepl`
- `examples/counter.nepl`
- `examples/counter2.nepl`
- `examples/fib.nepl`
- `examples/helloworld.nepl`
- `examples/nm.nepl`
- `examples/rpn.nepl`
- `examples/rpn_legacy.nepl`
- `examples/stdio.nepl`
- `web/examples/*.nepl`
- `.github/workflows/ci.yml`
- `nodesrc/test_examples_string_direct_imports.js`

## 良い点

`rpn.nepl` と `rpn_legacy.nepl` は、Stack の public API、string submodule direct import、`Result` による push failure handling、ANSI style enum API を使う形へ更新されている。過去の raw memory / internal layout 依存からは大きく改善している。

`stdio.nepl` は ASCII と UTF-8 の入力 doctest を持ち、`read_line` が UTF-8 text をそのまま扱う public example として分かりやすい。`nm.nepl` は CLI args、`read_all`、parser、HTML generator をつなぐやや大きい example として有効である。

`nodesrc/test_examples_string_direct_imports.js` は、examples が広い `alloc/string` facade alias へ戻らず、用途別 submodule を直接 import することを固定している。stdlib 分割方針に examples が追従している点は良い。

`web/examples` は build 時に examples を playground へ同期する入口になっており、playground 初期表示も `/examples/rpn.nepl` を優先する。つまり examples は単なるサンプルではなく、ユーザーが最初に触る統合面である。

## 問題とリスク

CI は examples doctest をまとめて実行していない。`nm` compile と `counter` emit smoke はあるが、RPN の color/REPL doctest、stdio UTF-8 doctest、BF interpreter doctest などは main branch gate から外れている。この gap は `ISS-20260507T153812328Z-EXAMPLES-DOCTESTS-ARE-NOT-RUN-BY-CI-13ED1895` として追加した。

`rpn.nepl` の operator dispatch は、現行言語では string token を `if` で判定している。`+` / `-` / `*` の小規模分岐としては実害は小さいが、ユーザーに見える example であり、将来的には tokenizer で `RpnTokenKind` enum を作り、`match` で分岐する方が開発方針に合う。

`examples/*.nepl` は public integration surface である一方、CI 上の artifact として test JSON が残っていない。失敗時に「どの example が、どの stdout mismatch を起こしたか」を GitHub Actions から追いにくい。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `examples/rpn.nepl` | ANSI style / Stack / string API の現行例。 | 良い。operator enum 化余地あり。 |
| `examples/rpn_legacy.nepl` | 控えめな互換 RPN 例。 | 良い。CI 実行が必要。 |
| `examples/stdio.nepl` | UTF-8 stdin/stdout smoke。 | 良い。 |
| `examples/nm.nepl` | Markdown parser/htmlgen CLI 例。 | CI compile あり。 |
| `examples/bf.nepl` | 大きめの interpreter 例。 | CI doctest gate が必要。 |
| `web/examples` | playground bundled examples。 | source と同期されているが generated 側は review noise に注意。 |

## 推奨対応

- examples doctest job を CI に追加し、JSON artifact と Pages final status に含める。
- RPN は将来、token classification を enum 化して `match` で operator dispatch する。今すぐ別 issue 化するほどの blocker ではないが、example quality 改善候補として扱う。
- examples を stdlib public API の integration contract と見なし、stdlib の I/O / string / collection / ANSI 変更時は focused examples run を必ず入れる。
