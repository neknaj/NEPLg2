# Quality Review: Examples

対象 commit: `f108cebd`

## 対象

- `examples/*.nepl`
- `web/examples` generation
- `nm-compile` and CLI smoke paths

## 概要

examples は hello world、counter、fib、stdio、RPN、BF、nm などを含む。過去 issue で raw memory 依存、Stack API 変更への追従漏れ、標準 header の不統一は修正済みで、examples 自体は現行 stdlib API に寄せられている。

CI では examples 全体を直接走らせる専用 job は見えないが、`nm-compile` と `wasi-test` の CLI multi-emit smoke、web examples sync が関連する。

## Actions 根拠

Actions run `25157230630`:

- `nm-compile`: failure。`examples/nm.nepl` が stdlib/nm/string builder owner failure の影響を受ける。
- `wasi-test`: failure。ただし CLI multi-emit step へ到達する前に doctest step が failure している可能性がある。
- Pages build/deploy: success。web artifact generation は通っている。

## 良い点

- examples の旧 raw memory / removed Stack API issue は verified/fixed になっている。
- `nodesrc/sync_web_examples.js` により web examples の clean checkout build 問題は対処済み。
- examples は tutorial より実アプリ寄りの smoke として重要な位置にある。

## 問題

- `examples/nm.nepl` は CI の NM compile smoke で失敗しており、stdlib/nm と string builder owner contract の影響を強く受ける。
- examples 専用の Actions artifact がないため、どの examples が現在 green かを remote main から一目で確認しにくい。
- web deploy が成功しても、runtime examples が全部成功するとは限らない。

## 必要な設計

- examples は CI で `nodesrc/tests.js -i examples --no-tree` を独立 artifact として残す。
- nm example は selfhost/docs validation の smoke でもあるため、stdlib/nm/string owner failure と明確に紐付ける。
- web examples sync は source examples との drift を継続検査する。

## 進捗状況

- source examples modernization: かなり完了。
- web examples sync: あり。
- examples dedicated CI artifact: 不足。
- `nm-compile`: failure。
