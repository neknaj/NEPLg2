# stdlib core review

対象 commit: `f108cebd`

## 概要

`stdlib/core` は no-std 相当の基礎層であり、`math`、`cast`、`char`、`mem`、`option`、`result`、`test`、traits を持つ。selfhost の土台としては必須だが、`mem` が public safe API と internal raw API をまだ分離できていない。

## 良い点

- `Option<T>` と `Result<T,E>` は enum で表現され、通常の分岐は `match` で書ける。
- `StdErrorKind` は enum で、stdlib error boundary の stable representation になっている。
- `char` primitive と `core/char` helper が追加され、ASCII classifier、UTF-8 encode helper、scalar validation がある。
- traits は `Copy` / `Drop` / `Eq` / `Ord` / `Hash` / `HashKey` / stringify など、collection と selfhost に必要な capability を分け始めている。

## 問題

### `core/mem` の役割混在

`core/mem.nepl` は次を同じ module に持つ。

- raw allocator: `alloc_raw` / `dealloc_raw` / `realloc_raw`
- raw load/store: `load_i32` / `store_i32` / `load<T>` / `store<T>`
- typed pointer wrapper: `MemPtr<T>`
- region wrapper: `RegionToken<T>`
- public-ish checked wrapper: `alloc_ptr` / `dealloc_ptr` / `load_u8(MemPtr<u8>)`

`MemPtr<T>` はコメント上は non-owning pointer だが、現実には `Vec.data`、`StringBuilder.data`、`ByteBuf.ptr` などで owner field としても使われる。この二重意味が Resource IR の alias summary を複雑にしている。

### `RegionToken<T>` は compiler-issued capability ではない

`region_new` で stdlib code から token を作れるため、`RegionToken<T>` は「所有権を証明する compiler-issued token」ではない。短期的には builder/string の owner flow を整理する助けになっているが、最終設計としては forge 可能な struct では不十分である。

### `core/test` と `std/test` の意味が違う

`core/test` は trap helper、`std/test` は stdout report helper である。selfhost / `.n.md` 共通運用では、同じ「assert」という名前でも、trap 型と report 型を混同しない doc と API 境界が必要である。

## selfhost への示唆

selfhost core では `MemPtr` / raw address を public compiler data structure に持ち込まない。token id、span id、kind enum などは `Copy` 値として扱い、storage owner は後続の `OwnedBuffer` 設計を待つ。`char` / `Option` / `Result` / traits は現在の方向で利用可能だが、`mem` は最終設計ではない。
