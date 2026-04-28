# NEPLg2 stdlib char 整備計画

作成日: 2026-04-28

## 目的

NEPLg2 に `'a'` 形式の char literal と `char` primitive type を追加するだけでは、stdlib の可読性と text safety は十分に改善しない。`string`、UTF-8 検証、byte buffer、lexer/parser、JSON/HTML escape が `i32` の文字コードを直接扱い続けると、`char` は表面上の構文だけになり、self-host compiler でも magic number が増える。

この文書は、stdlib 側で `char` をどう公開し、既存の byte-oriented API とどう共存させるかを定める。

関連 issue:

- [language lacks char type and single-quoted char literals](../../issues/items/ISS-20260428T004316513Z-LANGUAGE-LACKS-CHAR-TYPE-AND-SINGLE--63768838.md)
- [stdlib and selfhost should replace character code magic numbers with char literals](../../issues/items/ISS-20260428T004329925Z-STDLIB-AND-SELFHOST-SHOULD-REPLACE-C-CD0357FB.md)

## 基本方針

`char` は Unicode scalar value を表す Copy 型とする。`str` は引き続き UTF-8 byte sequence であり、既存の `len` は byte length のまま維持する。これは既存 stdlib、WASI I/O、byte builder、WASM binary emitter が byte count を前提にしているためである。

したがって、stdlib は次の区別を API 名で明示する。

| 概念 | API 方針 |
|---|---|
| byte length | 既存 `len s` / 将来 alias `str_byte_len` |
| byte index | 既存 `str_slice_result s start end` |
| UTF-8 boundary | 既存 `str_utf8_is_boundary` |
| char value | `char` / `char_*` |
| char iteration | `str_char_*` |
| ASCII byte classifier | `char_is_ascii_*` または `byte_is_ascii_*` |
| raw binary byte | `u8` / `i32` byte API のまま |

## `core/char` の追加

新規 module として `stdlib/core/char.nepl` を追加する。

### 必須 API

| API | 目的 |
|---|---|
| `char_to_i32(c) -> i32` | Unicode scalar value の code point を返す。 |
| `char_from_i32_result(code) -> Result<char,str>` | code point から `char` を作る。surrogate / 範囲外は Err。 |
| `char_to_u8_result(c) -> Result<i32,str>` | ASCII / byte として扱える `0..=255` の場合だけ返す。 |
| `char_is_ascii(c) -> bool` | `0..=127` 判定。 |
| `char_is_ascii_digit(c) -> bool` | `'0'..='9'`。 |
| `char_is_ascii_alpha(c) -> bool` | `'A'..='Z'` / `'a'..='z'`。 |
| `char_is_ascii_alnum(c) -> bool` | digit or alpha。 |
| `char_is_ascii_whitespace(c) -> bool` | `' '` / `'\t'` / `'\n'` / `'\r'` など。 |
| `char_utf8_len(c) -> i32` | UTF-8 encode 時の byte count。 |
| `char_encode_utf8_result(c) -> Result<ByteBuf,str>` | char を UTF-8 bytes へ encode する。 |

`char` の分類は、`match` と range predicate を組み合わせて実装する。単一文字の有限分岐では char literal を使い、decimal code を避ける。

## `string` 連携

既存 `alloc/string.nepl` は byte-oriented implementation を維持する。ただし public API には char-oriented helper を追加する。

### 追加 API

| API | 目的 |
|---|---|
| `str_byte_len(s) -> i32` | `len s` と同じ意味を明示名で提供する。 |
| `str_char_count(s) -> i32` | UTF-8 を走査し、Unicode scalar 数を数える。O(byte_len)。 |
| `str_char_at_result(s, char_index) -> Result<char,str>` | char index で 1 scalar を読む。O(byte_len)。 |
| `str_char_byte_index_result(s, char_index) -> Result<i32,str>` | char index から byte offset を得る。 |
| `str_next_char_result(s, byte_index) -> Result<(char,i32),str>` | byte offset から次 char と次 byte offset を返す。iterator の基礎。 |
| `str_slice_chars_result(s, start_char, end_char) -> Result<str,str>` | char index 範囲を byte boundary に変換して slice する。 |
| `str_starts_with_char(s, c) -> bool` | 先頭 char 判定。 |
| `str_contains_char(s, c) -> bool` | char 単位の contains。 |

### 既存 API の扱い

- `len s` の意味は変えない。byte length のままとする。
- `str_slice_result` は byte index API のままとする。
- `str_utf8_is_boundary` は `str_slice_chars_result` の下位 API として維持する。
- 既存 `string_byte_at_unchecked` などの byte API は binary / low-level 用として残すが、doc に「文字ではなく byte」と明記する。

## `StringBuilder` / `ByteBuilder` 連携

`StringBuilder` は `str` 片を集める API として維持し、char 追加 helper を増やす。

| API | 目的 |
|---|---|
| `sb_append_char_result(sb, c) -> Result<StringBuilder,str>` | char を UTF-8 encode して append。 |
| `sb_append_ascii_result(sb, c) -> Result<StringBuilder,str>` | ASCII char だけを 1 byte として append。 |
| `byte_builder_push_char_utf8(builder, c)` | UTF-8 bytes を `ByteBuilder` へ追加。 |
| `byte_builder_push_ascii(builder, c)` | ASCII char だけを 1 byte 追加。 |

WASM binary emitter のような binary format では、text magic 以外の byte は `u8` / integer のままにする。`'\0'` や `'a'` のように意味が文字である箇所だけ char literal に置き換える。

## `std/text` 連携

`std/text.nepl` は外部 byte sequence を checked UTF-8 `str` へ変換する役割を維持する。`char` 導入後は、UTF-8 decoder の共通化を進める。

追加候補:

- `text_utf8_decode_next(data, byte_len, i) -> Result<(char,i32), StdErrorKind>`
- `text_utf8_encode_char(c) -> Result<ByteBuf, StdErrorKind>`
- `text_is_valid_scalar(code) -> bool`

`alloc/string.nepl` と `std/text.nepl` に重複 UTF-8 decoder が増えないよう、最終的には core text helper を共有する。

## 既存コードの置き換え範囲

char support 実装後、次の順で decimal character code を char literal に置き換える。

1. `stdlib/alloc/encoding/json.nepl`: JSON escape classifier。
2. `stdlib/nm/parser.nepl`: JSON escape、inline parser の delimiter 判定。
3. `stdlib/nm/html_gen.nepl`: HTML escape classifier。
4. `stdlib/neplg2/core/syntax/lexer.nepl`: punctuation / newline / quote / slash / comment 判定。
5. `stdlib/alloc/string.nepl`: numeric parser、escape helper、UTF-8 boundary の ASCII fast path。
6. `stdlib/std/stdio.nepl` / `stdlib/std/env/cliarg.nepl`: sign、NUL、C-string doctest。
7. `tests/stdlib/byte_builder.n.md`: text magic bytes だけを char literal に置換。
8. `stdlib/platforms/wasix/tui.nepl`: ESC / bracket / backslash などの terminal sequence。

置換してはいけないもの:

- size、offset、capacity、alignment、tag、length。
- WASM binary encoding の section id や LEB128 byte。
- hash / crypto / binary protocol の固定 byte。
- UTF-8 continuation range など、範囲値としての byte。

## 実装順

### Stage 0: 仕様固定

- `char` literal / type の compiler issue を先に閉じる。
- `char` の storage、literal contextual typing、match pattern、diagnostic を確定する。

### Stage 1: `core/char` 基本 API

- `char_to_i32` / `char_from_i32_result` / ASCII classifier を追加する。
- doctest は ASCII と non-ASCII、surrogate rejected、範囲外 rejected を含める。

### Stage 2: UTF-8 encode / decode

- `char_utf8_len` と UTF-8 encode helper を追加する。
- `str_next_char_result` を実装し、UTF-8 decoder を `string` / `std/text` で共有できる形へ整理する。

### Stage 3: `string` char API

- `str_char_count`、`str_char_at_result`、`str_char_byte_index_result`、`str_slice_chars_result` を追加する。
- byte index API と char index API の名前を doc で明確化する。

### Stage 4: builder 連携

- `sb_append_char_result`、`byte_builder_push_char_utf8`、ASCII push helper を追加する。
- JSON / HTML escape など、文字を追加する API を builder へ寄せる。

### Stage 5: 既存コード移行

- escape classifier と lexer punctuation から順に char literal へ置換する。
- 置換後に静的検索で decimal character code の再発を監視する。

## 検証計画

- `tests/compiler/char_literals.n.md`: literal / typecheck / match / invalid literal。
- `stdlib/core/char.nepl` doctest: conversion / ASCII classifier / UTF-8 length。
- `tests/stdlib/string_char.n.md`: char count / char at / char slice / invalid index。
- `tests/stdlib/text_utf8.n.md`: decode / encode shared behavior。
- `tests/stdlib/nm.n.md`、JSON / HTML escape tests: char literal 移行後の出力一致。
- static test: classifier 関数が `10` / `34` / `92` などの decimal character code に戻らないこと。

## self-host への影響

self-host lexer / parser では、token punctuation と escape の判定に char literal を使う。source position は byte offset のまま維持し、`char` index を diagnostic span の単位にはしない。

つまり、`char` は source text の値を扱うための型であり、source span / byte buffer / WASM binary の単位を byte から置き換えるものではない。
