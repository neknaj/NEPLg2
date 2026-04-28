# NEPLg2 tutorial 全面書き直し計画

作成日: 2026-04-28

## 目的

`tutorials/getting_started/` は章数が多く、過去の NEPLg2 実装に合わせた説明やコード例が混在している。現在の compiler / stdlib では、`Result` / `Option`、`std/test`、match literal patterns、borrow / move、raw memory を避ける stdlib public API、self-host に向けた char / text 方針が変わっているため、既存 tutorial を部分修正で延命するより、現在の NEPLg2 に合わせて章立てから作り直す。

この文書は、tutorial を「今の NEPLg2 の正しい使い方」を示す教材として再設計するための計画である。

## 現状の問題

- 章数が 28 本あり、入門と競プロ catalog が同じ `getting_started` に混在している。
- 古い関数 signature 表記や、戻り値 `()` / `i32` の扱いが章ごとに揺れている。
- `std/test` の使い方が章ごとに異なり、現在推奨したい `checks_*` / `Result` ベースの書き方として統一されていない。
- `unwrap` / panic helper を避ける方針、`match` で失敗を扱う方針、raw memory に直接触れない方針が tutorial 全体に通っていない。
- match literal pattern、borrow / move / Drop、collection ownership、byte と text の区別、future `char` 方針など、現在の NEPLg2 で重要な概念が体系化されていない。
- 競技プログラミング向けの章は有用だが、入門導線としては密度が高く、標準 tutorial とは別 track に分けた方がよい。

## 書き直し方針

### 読者

主対象は、NEPLg2 の構文と stdlib を初めて使う開発者とする。compiler 内部や unsafe memory model の詳細は扱わない。ただし、所有権、`Result`、`Option`、byte / text の区別など、通常コードを書くために必要な安全設計は早い段階で説明する。

### コード例の標準形

すべてのコード例は実行可能な `neplg2:test` にする。説明だけの疑似コードは使わない。

標準の test 形:

```neplg2
| #entry main
| #indent 4
| #target std
|
#import "std/test" as *

fn main <()->i32> ():
    let checks <Vec<Result<(),str>>>:
        checks_new
        |> checks_push check_eq_i32 "label" 1 1
    let shown <Vec<Result<(),str>>> checks_print_report checks
    checks_exit_code shown
```

標準の I/O 形:

```neplg2
#entry main
#indent 4
#target std

#import "std/stdio" as *

fn main <()->()> ():
    println "Hello, NEPL!";
```

古い signature 表記、不要な raw memory、内部 layout 依存、panic 前提の `unwrap` は tutorial から除外する。

### 文章スタイル

- 1 章 1 テーマにする。
- 1 つの章に実行例は 1〜3 個までにする。
- 先に「何の問題を解くか」を示し、その後にコードを置く。
- コード内コメントは日本語で短く、処理目的を説明する。
- 未実装予定機能は tutorial 本文には入れず、予定章として appendix に置く。

## 新しい章立て

### Part 0: 実行環境と最小構成

1. `00_index.n.md`: tutorial 全体の案内、NEPLg2 の基本方針。
2. `01_hello_world.n.md`: `#entry` / `#target std` / `println`。
3. `02_test_harness.n.md`: `std/test`、`checks_new`、`checks_exit_code`。

### Part 1: 値、式、関数

4. `03_values_and_types.n.md`: `i32` / `u8` / `bool` / `str` / `()`、型注釈。
5. `04_prefix_calls.n.md`: prefix call、nested call、pipe を使わない基本形。
6. `05_functions_and_blocks.n.md`: `fn`、block、tail expression、`;` の意味。
7. `06_if_and_match.n.md`: `if` expression、`match`、bool / integer literal match。

### Part 2: 失敗を型で扱う

8. `07_option.n.md`: `Option`、`some` / `none`、`match`。
9. `08_result.n.md`: `Result`、`Ok` / `Err`、panic しない失敗処理。
10. `09_validation_project.n.md`: 小さな入力検証関数を `Result` で組む。

### Part 3: 文字列、byte、char

11. `10_string_and_text.n.md`: `str`、byte length、UTF-8 boundary、`str_slice_result`。
12. `11_bytebuf_and_text_io.n.md`: `ByteBuf`、checked UTF-8 conversion、binary と text の区別。
13. `12_char_and_ascii.n.md`: `char` 実装後に追加。char literal、ASCII classifier、`str_char_*`。

`char` が実装されるまでは、12 章は計画章として index に載せず、実装後に追加する。

### Part 4: collection と所有権

14. `13_vec_basics.n.md`: `Vec` の作成、push、len、free。
15. `14_collection_reads.n.md`: Copy read、borrowed read、owned remove の違い。
16. `15_move_and_borrow.n.md`: move、shared borrow、mutable borrow、use-after-move が compile error になる例。
17. `16_drop_and_cleanup.n.md`: free / Drop / container cleanup の基本。

raw memory、`MemPtr`、`alloc_raw` は通常 tutorial では扱わない。必要なら unsafe/internal appendix に分ける。

### Part 5: module、trait、generic

18. `17_imports_and_modules.n.md`: `#import`、alias、名前空間。
19. `18_generics.n.md`: type parameter、`Vec<T>`、`Option<T>`。
20. `19_traits_and_bounds.n.md`: `Copy` / `Clone` / `Drop` / custom trait の最小例。
21. `20_namespace_and_methods.n.md`: `Trait::method` / enum constructor / module alias。

### Part 6: 実践プロジェクト

22. `21_project_fizzbuzz.n.md`: stdout project。
23. `22_project_parser_small.n.md`: `str` を走査して token-like value を作る。char 実装後に char literal 版へ更新。
24. `23_project_config_validator.n.md`: `Result` と diagnostics の小 project。
25. `24_project_byte_output.n.md`: `ByteBuilder` で小さな binary/text output を作る。

### Part 7: Advanced / Appendix

26. `90_competitive_programming_intro.n.md`: 競プロ track の導入。
27. `91_sort_search_prefixsum.n.md`: sort / binary search / prefix sum。
28. `92_graph_bfs_dp.n.md`: BFS / DP。
29. `95_target_and_wasi_notes.n.md`: std / wasix / llvm の注意。
30. `99_migration_notes.n.md`: 古い tutorial からの移行表。

既存の 22〜27 の競プロ章は、Advanced track として再編集する。入門本編には混ぜない。

## 既存章からの移行

| 既存章 | 新章 |
|---|---|
| `01_hello_world` | `01_hello_world` |
| `02_numbers_and_variables`, `02b_type_conversion...` | `03_values_and_types`, `10_string_and_text` |
| `03_functions`, `07_while_and_block`, `08_if_layouts` | `05_functions_and_blocks`, `06_if_and_match` |
| `05_option`, `06_result` | `07_option`, `08_result` |
| `09_import_and_structure`, `17_namespace_and_alias` | `17_imports_and_modules`, `20_namespace_and_methods` |
| `10_project_fizzbuzz`, `11_testing_workflow` | `02_test_harness`, `21_project_fizzbuzz` |
| `12`〜`14` | `09_validation_project`, `23_project_config_validator` |
| `15_match_patterns` | `06_if_and_match` |
| `18_recursion_and_termination` | Advanced または project 内に統合 |
| `19_pipe_operator` | `04_prefix_calls` の後半または appendix |
| `20_generics`, `21_trait_bounds` | `18_generics`, `19_traits_and_bounds` |
| `22`〜`27` | Advanced track `90`〜`92` |

## 実装手順

### Stage 0: baseline 固定

- 現行 `tutorials/getting_started` の doctest 実行結果を確認する。
- 失敗がある場合は、tutorial rewrite issue に記録する。
- index と章名の現状 snapshot を残す。

### Stage 1: 新 index と章 template

- `00_index.n.md` を新章立てへ更新する。
- 新章共通の code style を固定する。
- 古い章は一時的に残し、リンクから外すか `99_migration_notes` へ移す。

### Stage 2: Core 入門章

- Part 0〜2 を先に書く。
- `std/test` と `Result` を早期に導入し、以後の章で共通 test pattern を使う。

### Stage 3: Text / char 章

- `str` と byte length の章を先に書く。
- `char` 実装後、`char` 章と既存 text examples を更新する。
- `char` 未実装の間は decimal character code を tutorial 本文で推奨しない。

### Stage 4: Collection / ownership 章

- raw memory を使わず、現在の collection public API で書く。
- move / borrow / cleanup の compile_fail examples を focused に入れる。

### Stage 5: Advanced track

- 競プロ章は現在の stdlib API と ownership 方針で再実装する。
- catalog ではなく、少数の実用パターンに絞る。

## 検証

- `node nodesrc/tests.js -i tutorials/getting_started --no-tree -o tmp/tutorials-rewrite.json -j 4`
- `node nodesrc/tests.js -i tutorials --no-tree -o tmp/tutorials-all.json -j 4`
- 変更した章ごとの focused run。
- markdown link check は既存 tool がないため、`rg` でリンク先存在確認を行う。

## 完了条件

1. `00_index.n.md` が新章立てに更新されている。
2. 本編章の全 code example が現在の NEPLg2 syntax で実行できる。
3. tutorial から raw memory / old signature / panic helper 推奨が消えている。
4. `Result` / `Option` / `match` / collection ownership / string byte-vs-char の説明が一貫している。
5. char 実装後、char literal と stdlib char API を使う章が追加されている。
6. `tutorials/getting_started` の doctest が通る。
