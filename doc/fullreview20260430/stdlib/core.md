# stdlib core review

確認対象 commit: `b350213c docs(review): add selfhost compiler review`

## 確認対象

- `stdlib/core/char.nepl`
- `stdlib/core/mem.nepl`
- `stdlib/core/{option,result,test,cast,field}.nepl`
- `stdlib/core/traits/**`

## 良い点

`core/char.nepl` は `char` primitive を Unicode scalar value として扱い、ASCII 分類、UTF-8 length/byte helper、`char_from_i32_result` を提供している。`char` と `str` byte offset の責務が分けられており、selfhost lexer/parser が `'a'` などの char literal と byte scan を混同しにくい。

`core/result.nepl` と `core/option.nepl` は enum match を前提にした基本 API を持つ。`unwrap_ok` / `unwrap_err` は残るが、コメント上は unsafe helper として通常は match/unwrap_or を優先する方針が明示されている。

`std/test` へ分離された構造化 assertion は、`core/test` 時代の単純 return-value style から、stdout report と exit code を分ける設計へ移行している。

traits は `Copy` / `Drop` / `Eq` / `Ord` / `Hash` / `Serialize` / `Deserialize` / `Debug` / `Stringify` が分かれており、collection/hash/json/stringify の型制約の基盤になる。

## 問題とリスク

`core/mem.nepl` は 1134 行あり、allocator、raw pointer、MemPtr、RegionToken、bulk copy、load/store、typed allocation API が同居している。ResourceIR の safety-critical boundary として大きすぎる。

`MemPtr<T>` は non-owning pointer として扱いたい一方、stdlib の多くの collection/string/I/O は storage owner の実体としても使っている。`RegionToken<T>` は owner token の入口だが、compiler 側の provenance model と完全には統合されていない。

`mem_ptr_addr`、`alloc_raw`、`dealloc_raw`、`load<T>`、`store<T>` などの raw surface はまだ stdlib/user code から到達できる。これは open issue の通り、pure/effect boundary と initialized/drop obligation の根本課題である。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `core/char.nepl` | Unicode scalar/ASCII/UTF-8 helper。 | 良い。selfhost lexerに使える。 |
| `core/result.nepl` | Result enum + helper。 | match優先方針は良い。unsafe unwrapの通常利用は禁止方向。 |
| `core/option.nepl` | Option enum + helper。 | 基盤として妥当。 |
| `core/mem.nepl` | allocator/raw/typed memory API。 | P1 safety boundary。分割と owner model統合が必要。 |
| `core/traits/**` | trait surface。 | collection/drop/hash/jsonの基盤。Drop contractは未完。 |

## 推奨対応

- `core/mem` を allocator internals、raw ABI、typed owner token、borrowed pointer projection、bulk copy に分ける。
- `MemPtr` と `RegionToken` は compiler ResourceIR の owner/provenance model と同期し、safe public API から raw address escape を外す。
- `unwrap_ok` / `unwrap_err` は stdlib implementation の通常経路から排除し、source policy の監視対象を広げる。
- char API は lexer/string API と接続し、string byte offset と char scalar index の混同を source policy/doc で防ぐ。
