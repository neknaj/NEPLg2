# NEPLg3 Trait システム仕様

最終更新: 2026-03-27

---

## 1. 設計原則

- trait は型能力の契約として扱う
- オーバーロード解決と trait 境界判定を統一する
- API 名は bare 名で揃える
- 表層構文は Zenn #1 / #2 の let / lambda / 前置型記法に合わせる

---

## 2. trait / impl

### 2.1 trait 定義

```nepl
let Eq trait:
    let eq %fn Self fn Self bool \a \b ...
```

### 2.2 impl 定義

```nepl
let i32 impl for Eq:
    let eq %fn i32 fn i32 bool \a \b i32_eq a b
```

---

## 3. trait 境界

trait 境界の詳細構文は今後も調整がありうるが、関数型そのものは `fn A B` 形式へ揃える。

```nepl
let sort .T: Ord %fn Vec .T Vec .T \v ...
```

複数引数ならカリー化して書く。

```nepl
let merge .T %fn Vec .T fn Vec .T Vec .T
    where .T: Ord
    \a \b ...
```

---

## 4. Coherence

- 同一 `(trait, target type)` への重複 impl を禁止する
- Orphan Rule を適用する
- impl の重複は修飾名では救済せず compile error とする

この設計自体は維持し、表記のみ新構文へ追従させる。

---

## 5. `Copy` / `Clone` / `Drop`

- `Copy` は暗黙複製可能な型のみ
- `Owned` / `Linear` 型に `Copy` は実装不可
- `Clone` は明示複製
- `Drop` は自動破棄規則に接続する

---

## 6. モジュール分離によるオーバーロード

Core 層と Casual 層を別モジュールに分け、同じ bare 名を使い分ける設計は維持する。

```nepl
use core::collections::vec as *
get vec proof

use std::collections::vec as *
get vec 3
```

---

## 7. オーバーロード解決

### 7.1 制約で絞る

`where` 節や trait 境界を満たさない候補を落とす。

### 7.2 期待型で絞る

```nepl
let x %Option i32 get vec idx
let y %i32 get vec idx
```

### 7.3 それでも曖昧なら修飾名を要求する

```nepl
core::vec::get vec proof
std::vec::get vec 3
```

ここで修飾名で解決できるのは **bare 名の候補集合** の曖昧性だけである。`impl Eq for i32` のような同一 `(trait, target type)` への重複 impl は Coherence 違反であり、修飾名では解決できない。

---

## 8. 標準 trait

| Trait | 意味 |
|---|---|
| `Eq` | 等値比較 |
| `Ord` | 全順序比較 |
| `Hash` | ハッシュ |
| `Stringify` | 人間向け文字列化 |
| `Debug` | デバッグ表現 |
| `Serialize .F` | 直列化 |
| `Deserialize .F` | 逆直列化 |
| `Parse` | 文字列からのパース |
| `Default` | デフォルト値 |
| `Clone` | 明示複製 |
| `Copy` | 暗黙複製 |
| `Add .U .R` | 加算演算 |
| `Drop` | 自動破棄対象 |
| `Reader` | バイト列の読み出し |
| `Writer` | バイト列の書き込み |
| `Seekable` | シーク操作 |
| `Buffered` | バッファリング |
| `Allocator` | 領域確保・解放ポリシー |
| `EventSource .E` | イベント発行源 |
| `EventSink .E` | イベント消費先 |

---

## 9. 将来導入する capability trait

`compiler.md` で言及している次の capability trait は、現時点ではコア構文として凍結していない将来拡張である。

- `MemReadable .T`
- `MemWritable .T`
- `RegionOwned`

これらは Resource IR 導入後のメモリ API / 解放 API を trait 境界で表す候補であり、本章では名前と役割だけを予約する。`stdlib.md` の trait 一覧でも将来導入候補として同じ位置づけで扱う。
