# NEPLg2 Self-Host Compiler

`stdlib/neplg2/` は NEPLg2.0 の現行 Rust コンパイラを NEPLg2.0 自身で再実装するための正規ソースツリーです。

このツリーは NEPLg3 の設計実験ではありません。`doc/neplg3/impl/compiler_structure.md` の分割方針を参考にしつつ、構文、型注釈、import、HIR、WASM/LLVM backend は現行 NEPLg2.0 を正とします。

## 層

- `core/`: filesystem、stdio、argv に依存しない純粋な compiler core。
- `cli/`: WASI / stdlib interface を使い、入力、diagnostic 表示、artifact 書き出しを担当する CLI 層。

## Stage 0 Skeleton

Stage 0 では各 pipeline stage の所有境界だけを固定し、各ファイルに実行可能な最小 doctest を置きます。実処理の移植は `selfhost/s0-infra-span-diag` 以降の issue で、依存順を崩さず追加します。

## 検証

```powershell
node nodesrc/tests.js -i stdlib/neplg2 --no-tree -o tmp/neplg2-selfhost-placeholder.json -j 2
```
