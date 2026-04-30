# stdlib alloc diag / json / hash review

対象 commit: `f108cebd`

## diag / outcome

`alloc/diag/error.nepl` は `DiagLevel`、`DiagKind`、`Diag`、`Diags`、`Outcome<T,E>` を持つ。selfhost diagnostic 集約に近い層として重要である。

良い点:

- `DiagLevel` は enum。
- `Outcome<T,E>` は value と diagnostics を分ける方向で、selfhost pipeline に合う。
- `Diags` は `Vec<Diag>` を内部に持ち、diagnostic list の基盤になる。

問題:

- `DiagKind` は `StdErrorKind` と `local_kind: Option<str>` を併用しており、selfhost compiler diagnostic id の主表現としては弱い。
- `Diags` は `Vec<Diag>` に依存するため、non-Copy owned diagnostic payload の Drop/free contract は collections の完成を待つ。

## JSON

`alloc/encoding/json.nepl` は `JsonValue` enum、`JsonMember` struct、`JsonEscapeKind` enum を持つ。過去の raw payload issue から大きく改善している。

良い点:

- JSON value が typed enum で表現されている。
- escape 分岐は `JsonEscapeKind` + `match` で表現され、以前の深い `if` nest から前進している。
- `json_quote_string` は diagnostic / report JSON に再利用できる。

問題:

- `JsonValue::Array(Vec<JsonValue>)` / `Object(Vec<JsonMember>)` は owning collection に依存する。
- serializer は string builder に依存するため、`sb_build_result` / StringBuilder owner issue の影響を受ける。

## hash

`alloc/hash` は FNV-1a、generic `hash32`、SHA256 を持つ。selfhost symbol table / cache / fingerprint に必要な層である。

良い点:

- `Hasher<T>` trait と `HashKey` capability があり、HashMap/HashSet と接続している。
- `hash32(str)` は byte scan + FNV-1a で selfhost symbol table に使いやすい。
- SHA256 は pure NEPL 実装として存在し、binary dependency を避けられる。

問題:

- SHA256 は `Vec<i32>` に依存するため、strict collection owner model の影響を受ける。
- `hash32` は 32-bit hash なので selfhost symbol table では collision handling を HashMap 側で正しく扱う必要がある。

## 結論

diag / json / hash の型設計は概ね良い。selfhost へ持ち込む場合は、diagnostic id を free string にしないこと、JSON/Diag の owning payload list が `Vec` の未完設計に依存していることを明示して進める。
