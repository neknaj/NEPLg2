# stdlib alloc hash json diag io review

確認対象 commit: `b350213c docs(review): add selfhost compiler review`

## 確認対象

- `stdlib/alloc/hash/**`
- `stdlib/alloc/encoding/json/**`
- `stdlib/alloc/diag/**`
- `stdlib/alloc/io/**`

## 良い点

hash は `fnv1a32`、`hash32`、`sha256` があり、SHA-256 は `api/compress/digest/padding/round/schedule/types` に分割済みである。`Sha256State` は Copy word state、`Sha256` は input buffer owner として分けられている。

JSON は `JsonValue` と `JsonEscapeKind` を enum で持ち、escape 分類を文字列/数値 sentinel ではなく match で扱う。string escape は stdlib string builder へ寄せられている。

diag/error は types/diag/diags/outcome に分割され、`Diag` と structured error outcome を collection/string 系の API が共有できる。

ByteBuf/ByteBuilder は raw pointer を public caller に押し出さず、byte buffer owner boundary を集約する方向になっている。

## 問題とリスク

`JsonValue::Array(Vec<JsonValue>)` と `JsonValue::Object(Vec<JsonMember>)` は recursive non-Copy payload を持つ。これは collection element drop contract が未完の現状では、free/drop 設計上の重要な入力である。

SHA-256 の `Sha256` は `Vec<i32>` buffer に入力 byte を蓄積する。mutable byte buffer がないための暫定実装としては理解できるが、大量入力では memory/copy cost が高く、selfhost の hashing用途では streaming ByteBuf/ByteBuilder と役割を分ける必要がある。

ByteBuf/ByteBuilder と JSON/string escape は raw-memory-backed API migration に依存する。source policy は増えているが、compiler ResourceIR gate と Actions 結果で継続確認が必要である。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `alloc/hash` | FNV/hash32/SHA-256 split。 | selfhost hash基盤として有用。 |
| `alloc/encoding/json` | enum value/escape + serializer。 | typed designは良い。drop contract依存。 |
| `alloc/diag` | structured Diag/outcome。 | stdlib error surfaceとして有用。 |
| `alloc/io` | ByteBuf/ByteBuilder/traits。 | raw boundary migration対象。 |

## 推奨対応

- JSON recursive payload を使う場合は、collection drop/remove contract が確立してから selfhost の大規模データに使う。
- SHA-256 は短期の hash/checksum用途なら良いが、selfhost symbol/hash table は `HashKey`/DefaultHash32 と用途を分ける。
- ByteBuf/ByteBuilder は safe owner API と internal raw implementation の境界を保ち、nm/json/string利用側へ raw pointerを漏らさない。
