# tests review

確認対象 commit: `c5f93163 fix(selfhost): split hir expr payloads`

## 確認対象

- `tests/compiler/*.n.md`
- `tests/compiler/tree/*.js`
- `tests/stdlib/*.n.md`
- `tests/playground_editor/**`
- `stdlib/**/*.nepl` の `neplg2:test`
- `tutorials/**/*.n.md`
- `nodesrc/tests.js`
- `nodesrc/run_doctest.js`
- `nodesrc/run_test.js`
- `.github/workflows/ci.yml`

## 良い点

`.n.md` doctest は compiler / stdlib / tutorial を同じ形式で扱う共通資産になっている。`diag_code`、`diag_span`、`stdout`、`stderr`、`ret`、`exit_code`、`argv`、`stdin` を持てるため、Rust compiler と selfhost compiler の将来共通運用へ寄せやすい。

`nodesrc/tests.js` は worker pool、case timeout、changed-only collection、WASM / LLVM / all runner、dual backend strict comparison を持つ。CI でも `tests`、`tutorials`、`stdlib`、LLVM compile-only、LLVM dual backend を分けて実行しており、言語仕様と stdlib public behavior の回帰検出範囲は広い。

`nodesrc/run_doctest.js` は focused reproduction の入口として有効である。`run_test.js` は WASI/WASIX fallback、timing metadata、stdout/stderr expectation、exit code metadata を扱い、test result JSON の調査可能性が上がっている。

source policy regression は stdlib / ResourceIR / selfhost / diagnostics / editor contract まで広がっている。これは通常の doctest では見つけにくい「不自然な中間変数」「unsafe unwrap 再導入」「enum/match coverage 退行」「責務境界の肥大化」を検出する補助線になっている。

## 問題とリスク

`.n.md` の assertion suite は、`std/test` の structured report API へ移行済み部分がある一方、`ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD` が残っている。return value を exit code 相当に使う運用が残ると、失敗詳細が stdout contract として固定されず、Rust/selfhost 共通 runner の比較単位が曖昧になる。

CI は `node nodesrc/tests.js -i examples` を実行していない。`examples/rpn.nepl` や `examples/rpn_legacy.nepl` は色付き出力と REPL behavior の doctest を持つが、main branch CI では `nm` compile と `counter` emit smoke 以外が直接検査されない。このため `ISS-20260507T153812328Z-EXAMPLES-DOCTESTS-ARE-NOT-RUN-BY-CI-13ED1895` を追加した。

source policy regressions は CI で `--warn-only` として実行される。これは後続 job を止めない設計としては妥当だが、型安全・メモリ安全・enum/match policy の最終 gate にはならない。安全性に直結する policy は、warn-only の結果を無視せず issue として即時追跡する運用が必須である。

`tests/compiler/tree` の stage-level JS fixture と `.n.md` stage metadata はまだ完全には統合されていない。selfhost parity runner を作るには、tree fixture のうち外部 contract 化できるものを `.n.md` manifest へ寄せ、Rust 内部 API 固有のものだけ JS test に残す線引きが必要である。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `tests/compiler/*.n.md` | 言語仕様、diagnostic、move/resource、LLVM target を広く保持。 | 良い。stage parity metadata への拡張余地あり。 |
| `tests/compiler/tree` | parser / resolve / semantics の stage JSON を JS で確認。 | 有効だが `.n.md` 共通化計画とは未統合。 |
| `tests/stdlib/*.n.md` | stdlib public behavior と selfhost library smoke を保持。 | 良い。assert report 移行を継続。 |
| `stdlib/**/*.nepl` doctest | API 近傍の小さい例を保持。 | 良い。大型 regression は `tests/stdlib` へ置く方針を維持。 |
| `tutorials/**/*.n.md` | 現行 tutorial の executable docs。 | 良い。CI job あり。 |
| `examples/*.nepl` doctest | RPN / stdio / nm などが doctest を持つ。 | CI job がないため issue 追加。 |
| `nodesrc/tests.js` | aggregate runner。 | 機能は十分。expectation logic の共通化は残る。 |
| `nodesrc/run_doctest.js` | focused runner。 | 良い。aggregate runner との責務共通化を継続。 |
| GitHub Actions | tests/tutorials/stdlib/LLVM/pages を実行。 | examples job と latest run completion の追跡が必要。 |

## 推奨対応

- `ISS-20260429T102425370Z-N-MD-TESTS-RELY-ON-RETURN-VALUES-INS-9B49EDAD` を進め、assertion suite は stdout report + `exit_code` へ移行する。
- `ISS-20260507T153812328Z-EXAMPLES-DOCTESTS-ARE-NOT-RUN-BY-CI-13ED1895` を修正し、examples doctest JSON を CI artifact と Pages status に含める。
- Rust/selfhost 共通 `.n.md` runner は、manifest schema、backend enum、stage JSON canonicalization、skip policy を先に固定する。
- source policy warning は CI failure でなくても、レビューと issue 運用上は未解決安全リスクとして扱う。
