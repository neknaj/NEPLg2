# stdlib math traits review

確認対象 commit: `b350213c docs(review): add selfhost compiler review`

## 確認対象

- `stdlib/core/math/**`
- `stdlib/core/traits/**`
- `stdlib/core/rand/xorshift32.nepl`

## 良い点

math は `i32`、`i64`、`u8`、`f32`、`f64`、`int128`、conversion、reinterpret が分割されている。巨大 `math.nepl` へ戻さず、width/operation ごとに source policy で確認しやすい構成である。

`core/traits/eq.nepl`、`ord.nepl`、`hash.nepl`、`hash_key.nepl` は collection の generic constraint と整合している。BTree 系は `Ord`、HashMap/HashSet は `HashKey`/hasher へ依存し、i32 専用構造からは脱却している。

`Copy` trait は non-owning pointer と owner-bearing storage の区別に関わるため、ResourceIR との接続点として重要である。stdlib 側では多くの collection が `.T: Copy` を要求し、現状の drop contract 未完を明示している。

## 問題とリスク

`i32` が raw pointer representation と同じ幅で使われているため、ordinary scalar と raw pointer proof の混線を compiler 側で常に警戒する必要がある。これは Rust ResourceIR 側で recent fix が続いた領域であり、stdlib の math API 自体が悪いわけではないが、raw address proof と scalar arithmetic を分ける設計が必須である。

`int128` や float formatting/parsing は string builder と raw byte helper に依存する。stdlib raw-memory-backed API migration が完了するまでは、pure/effect boundary の監視対象である。

## 進捗状況

| 領域 | 状態 | 判定 |
|---|---|---|
| `core/math/i32` / `i64` | arithmetic/bitwise/compareに分割。 | 良い。 |
| `core/math/f32` / `f64` | unary/binary/compare/conversion。 | 良い。 |
| `core/math/int128` | i128/u128/typesに分割。 | selfhostの整数処理には有用。 |
| `core/traits` | Eq/Ord/Hash/Drop/Copyなど。 | collection設計の基盤。Drop側は未完。 |

## 推奨対応

- raw pointer proof は math helper ではなく ResourceIR の pointer provenance に閉じる。
- collection/string/hash の helper が ordinary `i32` arithmetic を raw pointer proof として使わない source policy を維持する。
- `Drop` trait と collection free/drop contract を設計し、`Copy` 制約を外せる collection と外せない collectionを明確に分ける。
