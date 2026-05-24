# parser_debug.rs 由来の doctest

このファイルは Rust テスト `parser_debug.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## debug_parse_string_nepl

neplg2:test[skip]
```neplg2
#entry main
#indent 4
fn main %fn () i32():
    0
```
