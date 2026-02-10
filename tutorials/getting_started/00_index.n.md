# Getting Started（入門）

このチュートリアルは、NEPLg2 を初めて触る人向けに「仕様どおりに動く最小コード」を段階的に学ぶための資料です。

章立ては Rust Book のように「概念章」と「実践章」を交互に置く方針を参考にしています。
短い章で前提を積み上げ、途中で小さな実践章を挟んで手を動かしながら進みます。
後半は Elm / Lean のチュートリアルで重視される「関数中心」「型で仕様を表す」流れを取り入れています。

NEPLg2 の中核は次の 3 つです。

- 式指向: `if` や `match` も式として値を返します。
- 前置記法: `add 1 2` のように関数を前に書きます。
- オフサイドルール: インデントでブロックや複数行引数を表現します。

各ページに `neplg2:test` のコード例を埋め込み、`nodesrc/cli.js` のテスト実行でそのまま検証できます。

## 学習ロードマップ

### Part 1: 基礎（概念章）
- [01 Hello World](01_hello_world.n.md)
- [02 数値と変数（前置記法と型注釈）](02_numbers_and_variables.n.md)
- [03 関数と if（inline と block）](03_functions.n.md)
- [04 文字列と標準入出力](04_strings_and_stdio.n.md)
- [05 Option（値がある/ない）](05_option.n.md)
- [06 Result（成功/失敗）](06_result.n.md)

### Part 2: 制御構文と構造化（概念章）
- [07 while と block（オフサイドルール）](07_while_and_block.n.md)
- [08 if の書式バリエーション](08_if_layouts.n.md)
- [09 import と小さな分割](09_import_and_structure.n.md)

### Part 3: 実践（小プロジェクト章）
- [10 ミニプロジェクト: FizzBuzz](10_project_fizzbuzz.n.md)
- [11 テスト駆動で関数を固める](11_testing_workflow.n.md)

### Part 4: 関数型・型駆動スタイル（Elm / Lean 風）
- [12 純粋関数の合成（状態を持たない変換）](12_pure_function_pipeline.n.md)
- [13 型で失敗を表す（Option / Result の徹底）](13_type_driven_error_modeling.n.md)
- [14 等式的リファクタと回帰テスト](14_refactor_with_properties.n.md)

### Part 5: 実装で頻出の書き方
- [15 match で分岐を明示する](15_match_patterns.n.md)
- [16 デバッグ出力と ANSI カラー](16_debug_and_ansi.n.md)
- [17 名前空間と `::` 呼び出し](17_namespace_and_alias.n.md)
- [18 再帰と停止条件](18_recursion_and_termination.n.md)
- [19 パイプ演算子 `|>`](19_pipe_operator.n.md)
- [20 ジェネリクスの基本](20_generics_basics.n.md)
- [21 trait 制約の基本](21_trait_bounds_basics.n.md)

### Part 6: 競技プログラミング実践
- [22 競プロ向け I/O と演算](22_competitive_io_and_arith.n.md)
- [23 sort と二分探索の型](23_competitive_sort_and_search.n.md)
- [24 DP の基本パターン](24_competitive_dp_basics.n.md)
