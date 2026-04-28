# Advanced: target と WASI notes

NEPLg2 の code は target によって利用できる API が変わります。

- `#target core`: compiler core や純粋な計算を確認する最小 target です。
- `#target std`: `std/test`、標準入出力、collection を使う tutorial の標準 target です。
- WASI / stdio / filesystem は外部 I/O として扱い、pure な関数へ混ぜません。

入門本文では、実行確認しやすいように多くの章で `#target std` を使います。一方、静的検査だけを確認する compile-fail 例では `#target core` を使うことがあります。

target 固有の低水準 API を tutorial の通常章で直接扱わないのは、self-host と stdlib の public API を同じ設計で読めるようにするためです。
