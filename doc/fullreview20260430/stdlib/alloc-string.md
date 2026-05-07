# stdlib alloc string review

確認対象 commit: `b350213c docs(review): add selfhost compiler review`

## 確認対象

- `stdlib/alloc/string.nepl`
- `stdlib/alloc/string/**`
- `nodesrc/test_stdlib_string_*.js`
- `nodesrc/test_stdlib_byte_scanner_helpers_boundary.js`

## 良い点

string は `storage`、`access`、`builder`、`search`、`slice`、`split`、`integer`、`float`、`utf8`、`scanner`、`char_offsets` へ分割されている。selfhost lexer/parser で必要な `str_starts_with_at`、line scanner、slice、UTF-8 helper が stdlib API として揃ってきた。

`StringBuilder` は `Option<MemPtr<u8>>` で empty/owned storage を分け、raw pointer sentinel ではなく typed absence を使っている。source policy も builder owner boundary を監視している。

`alloc/string/search` は `str_eq`、`str_starts_with`、`str_starts_with_at`、byte find などを持ち、`#indent` や directive 文字列比較を byte-by-byte if chain から解消する基盤になっている。

`char_offsets` と `core/char` により、byte offset と char scalar の連携を段階的に整えられる状態になった。

## 問題とリスク

string storage は raw memory backed であり、`string_from_addr_unchecked`、`mem_ptr_addr`、bulk copy、byte load/store の境界を完全には隠せていない。これは open issue `STDLIB-RAW-MEMORY-BACKED-APIS` の対象である。

`alloc/string.nepl` の root facade は実質 re-export と古い概要コメントだけで、module map と安全性契約の説明が薄い。利用者は submodule doc まで読まないと、byte offset/char offset/owner boundary を把握しにくい。

integer/float formatting/parsing は selfhost diagnosticsやcompiler outputで重要になる。raw scratch buffer や builder failure path が source policy で守られているか、Actions の stdlib-test で継続確認が必要である。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `string/storage` | raw-backed str construction。 | P1 migration対象。 |
| `string/access` | len/data/byte access。 | selfhostで有用。raw escape監視が必要。 |
| `string/builder/**` | typed owned buffer。 | 良い。 |
| `string/search/**` | compare/boundary/byte_find。 | lexer/parser改善に有効。 |
| `string/slice/**` | byte/char/trim。 | byte vs char境界を文書化する。 |
| `string/integer` / `float` | format/parse分割。 | selfhost diagnosticsに必要。 |
| `string/utf8` / `char_offsets` | UTF-8 validation/offset。 | char連携の基盤。 |

## 推奨対応

- string facade に module map、byte/char offset、owner boundary を明記する。
- raw-backed storage helper は internal boundary として扱い、selfhost/nm/json などの利用側から直接 raw pointer を受け取らない設計を維持する。
- directive/keyword/classifier などで必要な文字列比較は `str_starts_with_at` や scanner helperに集約し、手書き byte-by-byte 比較を戻さない。
