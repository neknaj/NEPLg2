# NEPLg2 Getting Started

この tutorial は、現在の NEPLg2 で通常のアプリケーションと self-host 用の基礎コードを書くための入門です。古い章に残っていた競技プログラミング catalog、raw memory 依存、panic helper 前提の例は本文から外し、`Result` / `Option` / `match` / `char` / byte と text の区別 / collection ownership を順に扱います。

コード例は原則として `neplg2:test` で実行できます。入門本文では `alloc_raw`、`MemPtr`、`unwrap_ok` のような内部寄り・panic 寄りの入口を推奨しません。失敗しうる処理は `Result` を返し、呼び出し側で `match` して扱います。

## Part 0: 実行環境と最小構成

- [01 Hello World](01_hello_world.n.md)
- [02 test harness](02_test_harness.n.md)

## Part 1: 値、式、関数

- [03 値と型](03_values_and_types.n.md)
- [04 前置呼び出しと pipe](04_prefix_calls.n.md)
- [05 関数、block、末尾式](05_functions_and_blocks.n.md)
- [06 if と match](06_if_and_match.n.md)

## Part 2: 失敗を型で扱う

- [07 Option](07_option.n.md)
- [08 Result](08_result.n.md)
- [09 小さな検証 project](09_validation_project.n.md)

## Part 3: 文字列、byte、char

- [10 文字列と text](10_string_and_text.n.md)
- [11 ByteBuf と text I/O](11_bytebuf_and_text_io.n.md)
- [12 char と ASCII](12_char_and_ascii.n.md)

## Part 4: collection と所有権

- [13 Vec の基本](13_vec_basics.n.md)
- [14 collection の読み取り](14_collection_reads.n.md)
- [15 move と borrow](15_move_and_borrow.n.md)
- [16 cleanup と Drop 方針](16_drop_and_cleanup.n.md)

## Part 5: module、generic、trait

- [17 import と module](17_imports_and_modules.n.md)
- [18 generics](18_generics.n.md)
- [19 trait と bound](19_traits_and_bounds.n.md)
- [20 namespace と method 呼び出し](20_namespace_and_methods.n.md)

## Part 6: 実践 project

- [21 FizzBuzz](21_project_fizzbuzz.n.md)
- [22 小さな parser](22_project_parser_small.n.md)
- [23 config validator](23_project_config_validator.n.md)
- [24 byte output](24_project_byte_output.n.md)

## Advanced / Appendix

- [90 競技プログラミング導入](90_competitive_programming_intro.n.md)
- [91 sort / search / prefix sum](91_sort_search_prefixsum.n.md)
- [92 graph / BFS / DP](92_graph_bfs_dp.n.md)
- [95 target と WASI notes](95_target_and_wasi_notes.n.md)
- [99 旧 tutorial からの移行](99_migration_notes.n.md)

## 推奨する読み方

1. Part 0 で `#entry`、`#target`、`std/test` の形を固定します。
2. Part 1 で式、関数、`match` の書き方を先に覚えます。
3. Part 2 以降は、失敗や欠損を panic ではなく値として返す書き方を基本にします。
4. Part 3 と Part 4 は self-host の lexer / parser / stdlib を読むための前提です。
5. Advanced は入門本文を終えてから、必要な用途だけ参照します。
