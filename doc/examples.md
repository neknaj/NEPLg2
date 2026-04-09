# Examples

`examples/` は実行可能サンプル集です。短い文法例から、REPL や簡易 CLI のような少し大きめの例まで置いています。

## 方針

- 各 `.nepl` は読めるサンプルであると同時に、`neplg2:test` で回帰確認できる状態を保ちます。
- `examples/rpn.nepl` のように、ファイル先頭に日本語のドキュメントコメントと doctest をまとめる形式を基準にします。
- サンプル内コメントは処理の目的や構造を説明し、変更履歴や場当たり的な注釈は書きません。

## 確認コマンド

examples だけを focused に確認したいときは、まず `trunk build` を行ったうえで次を使います。

```bash
trunk build
node nodesrc/tests.js -i examples --no-tree -o tmp/examples-tests.json -j 4
```

`tmp/examples-tests.json` には各 example の doctest 結果がまとまります。標準出力比較を含むので、example の表示仕様を変えたときはこの JSON と実行結果を両方確認してください。

## 収録例

- `helloworld.nepl`
  最小の標準出力例
- `counter.nepl`
  while と可変変数の基本例
- `counter2.nepl`
  carriage return と ANSI を使った簡易アニメーション例
- `fib.nepl`
  状態を持つ逐次計算の例
- `stdio.nepl`
  1 行入力の基本例
- `bf.nepl`
  Brainfuck 実行器
- `rpn.nepl`
  高水準 stdlib を使った RPN REPL
- `rpn_regacy.nepl`
  生メモリ中心の旧式 RPN REPL
- `nm.nepl`
  Markdown 変換 CLI
