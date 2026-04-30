# stdlib alloc string / io review

対象 commit: `f108cebd`

参照 Actions: `25157230630`

## 概要

`alloc/string.nepl` と `alloc/io.nepl` は selfhost の lexer / parser / diagnostic / file I/O に直結する。`StringBuilder` と `ByteBuilder` は `Option<MemPtr<u8>>` 化され、空 storage と owning storage を型に出す方向へ進んでいる。一方、`alloc/string.nepl` は 3290 行で、UTF-8、builder、numeric parser、slice/search が集中している。

## 良い点

- `str_starts_with` / `str_starts_with_at` / `str_find` があり、`#indent` のような manual byte comparison を置き換える標準 API が整ってきた。
- `str_next_char_result` / `str_char_count` / `str_char_at_result` / `str_slice_chars_result` があり、char support と string の連携がある。
- `StringBuilder.data` と `ByteBuilder.ptr` は `Option<MemPtr<u8>>` になり、empty を null owner として扱う旧設計より良い。
- `ByteBuf` / `ByteBuilder` は selfhost binary/text emitter の短期基盤として使える。

## Actions で確認した問題

`stdlib-test` artifact では、`from_f64_result__f64__Result_T_E_str_i32__pure` が `resource.cell.possibly_moved` を出している。これは `from_f64_result` が scratch buffer を確保し、`string_from_mem_unchecked_result scratch trim` と `dealloc_raw scratch_raw 6` の境界を跨ぐことで、Resource IR が scratch cell を `MaybeMoved` と見るためである。

この failure は HashMap/HashSet doctest の前段で表面化し、collection review signal を汚している。`ISS-20260430T140641137Z-FROM-F64-RESULT-SCRATCH-BUFFER-REINT-1D9324F1` として追加した。

## 設計評価

`StringBuilder` / `ByteBuilder` の Option 化は正しい方向だが、`MemPtr<u8>` 自体が owner token ではないため完成形ではない。理想は次である。

- `OwnedBytes` が storage owner と len/cap を持つ。
- `ByteBuf` は finalized owned byte sequence。
- `StringBuilder` は UTF-8 生成専用 builder。
- `str` への確定は `OwnedStringRegion` からだけ行う。
- unchecked raw conversion は internal boundary に閉じる。

## selfhost への示唆

selfhost S1/S2 では string prefix/search/slice と builders は使ってよい。owned `Vec<str>` を作る split API は廃止し、`str_split_next` のような allocation-free scanner、`str_find`、byte/char scanner を優先する。
