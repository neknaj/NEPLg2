# 旧 tutorial からの移行

旧 `getting_started` は、入門、実践、競技プログラミング catalog が同じ順序に混ざっていました。現在の構成では、入門本文を Part 0 から Part 6 へ整理し、競技向けの内容は Advanced track に移しました。

主な対応:

| 旧章 | 新しい扱い |
|---|---|
| `02_numbers_and_variables` / `02b_type_conversion...` | `03_values_and_types`、`10_string_and_text` |
| `03_functions` / `07_while_and_block` / `08_if_layouts` | `05_functions_and_blocks`、`06_if_and_match` |
| `05_option` / `06_result` | `07_option`、`08_result` |
| `09_import_and_structure` / `17_namespace_and_alias` | `17_imports_and_modules`、`20_namespace_and_methods` |
| `10_project_fizzbuzz` / `11_testing_workflow` | `02_test_harness`、`21_project_fizzbuzz` |
| `15_match_patterns` | `06_if_and_match` と parser project |
| `20_generics_basics` / `21_trait_bounds_basics` | `18_generics`、`19_traits_and_bounds` |
| `22` から `27` の競プロ章 | `90` 以降の Advanced track |

削除した古い章にあった raw memory、panic helper、古い signature 説明、owner 再利用に見えるサンプルは、current NEPLg2 の入門としては採用しません。必要な低水準内容は stdlib / compiler の設計文書または issue で扱います。
