# NEPLg2.1 Trait システム仕様

最終更新: 2026-03-16

---

## 1. 設計原則

- trait は「メソッド集合」ではなく、型能力の契約として扱う。
- オーバーロード解決と trait 境界判定は同じ型同値判定で統一する。
- 実装一意性は `(trait, target type)` の組で保証する。
- move / effect 判定は trait 判定と独立させる。ただし `Copy`/`Clone` は move 規則へ接続する。
- API 名は bare 名（prefix/suffix なし）で揃える。`get_opt`・`get_safe` のような接尾辞は使わない。
- 暗黙 cast による trait / overload 解決は導入しない。

---

## 2. Trait の役割

### 2.1 Interface 相当

共通メソッド契約を提供する。`let Type impl for Trait:` で実装する。

```nepl
let Eq trait:
    let eq %fn Self Self -> bool \ a b :
        ...

let i32 impl for Eq:
    let eq %fn i32 i32 -> bool \ a b :
        i32_eq a b
```

### 2.2 Type Class / Concept 相当

型引数境界を `.T: Trait` 形式または `where` 節で表現する。

```nepl
let sort .T: Ord %fn* Vec .T -> Vec .T \ v :
    ...

let merge .T .K .V %fn Vec .T Vec .T -> Vec .T
    where .T: Ord .K: Hash .V: Eq
    \ a b :
    ...
```

### 2.3 move / memory 相当

- `Copy`/`Clone` は所有権規則に接続する能力 trait として扱う。
- `MemReadable .T`, `MemWritable .T`, `RegionOwned`（将来導入）はメモリ能力の契約として扱う。

---

## 3. 一意性規則（Coherence）

### 3.1 同一モジュール内

- 同一モジュール内では同一 `(trait, target type)` への重複 impl を禁止する。
- 判定は文字列化した型ではなく、構造的型同値（`same_type`）で行う。
- 重複検出後は後続パスで重複 impl を無視し、診断を安定化する。

### 3.2 クロスモジュール Coherence（Orphan Rule）

別モジュールの impl も含めたグローバルな一意性を保証するため、以下の Orphan Rule を適用する。

**Orphan Rule**: impl の定義は次のいずれかを満たさなければならない。

1. `trait` が自モジュールで定義されている、または
2. `target type` が自モジュールで定義されている

両方が外部モジュールで定義された組み合わせ（外部 trait × 外部型）の impl は禁止する。

```nepl
// 自モジュール定義の型 → 外部 trait への impl → OK
let MyType impl for std::Eq: ...

// 外部 trait × 外部型 → NG
let std::Vec i32 impl for other::SomeTrait: ...
// ERROR: orphan impl — neither trait nor type is defined in this module
```

**`use` 時の衝突検出**: 複数のモジュールを `use` した結果、同一 `(trait, target type)` の impl が競合する場合はコンパイルエラーとし、修飾名での明示を要求する。Orphan Rule はこの衝突を事前に防ぐ主な機構だが、stdlib の改訂など許可された状況下での衝突は import 時エラーとして扱う。

---

## 4. シグネチャ整合

- trait メソッド実装の整合判定は構造型同値で行う。
- 文字列ベース比較は補助（mangle / デバッグ）に限定し、契約判定に使わない。

---

## 5. ハードコード最小化

- 型名ハードコード（例: 特定 struct 名での `Copy` 禁止）は禁止する。
- trait 参照の分岐は段階的に能力テーブルへ移す。
- 最終的には trait の「能力種別」を宣言側から供給し、コンパイラ側の名前比較を撤廃する。

---

## 6. モジュール分離によるオーバーロード管理

オーバーロードの複雑化は、**同じ bare 名を持つ関数を別モジュールに配置し、`use` 対象を切り替えることで解決する**。

典型例: Core 層（依存型・証明必須）と Casual 層（Result/Option ラッパー）を別モジュールに分ける。

```nepl
// Core 層: 型安全を静的に保証（Phase 8 以降）
use core::collections::vec as *
get vec proof    // where %IsLess idx len が必要、戻り型 .T

// Casual 層: 実行時検査、Option で返す
use std::collections::vec as *
get vec 3        // 証明不要、戻り型 Option .T
```

モジュール修飾名により、両方を `use` している状態でも常に明示できる:

```nepl
core::vec::get vec proof    // Core 版
std::vec::get vec 3         // Casual 版
```

---

## 7. オーバーロード解決の 3 段階

同一スコープに複数のオーバーロード候補が存在する場合、次の順で解決する。この仕組みは Phase 0–7 の trait 境界（`where .T: Ord` 等）から有効であり、Phase 8 の命題型制約（`where %IsLess idx len` 等）へも自然に拡張される。

### 段階 1: 制約フィルタリング（`where` 節の充足可否）

`where` 節を充足できない候補を除外する。

Phase 0–7 での典型例（trait 境界）:

```nepl
let x sort unsorted_vec
// sort .T: Ord の where を充足するか → .T が Ord を実装していなければ除外
```

Phase 8 以降の例（証明オブジェクト — 依存型導入後に有効）:

```nepl
// [Phase 8 example]
use core::collections::vec as *
use std::collections::vec as *

let x get vec 3
// core::vec::get の where %IsLess 3 len → scope に証明なし → 除外
// std::vec::get → 残る → Option .T に解決

let x get vec proof   // proof : IsLess idx len が scope にある
// core::vec::get の where を充足 → 残る
// std::vec::get → i32 を期待するが proof は IsLess 型 → 型不一致 → 除外
// core::vec::get に解決
```

### 段階 2: 期待型による絞り込み（双方向型推論）

制約フィルタ後も複数候補が残った場合、呼び出し元の期待型を call site に伝播させて絞る。

```nepl
// [Phase 8 example] — core/std vec の分離は Phase 8 以降のもの
let x %Option .T get vec idx    // 期待型 Option .T → std::vec::get に解決
let x %.T        get vec idx    // 期待型 .T → core::vec::get に解決
```

### 段階 3: 修飾名の要求

フィルタ後に複数候補が残り、期待型でも絞れない場合はコンパイルエラーとし、修飾名を要求する。

```nepl
// [Phase 8 example]
let x get vec idx
// ERROR: ambiguous — use `core::vec::get` or `std::vec::get`
```

---

## 8. effect とオーバーロード

同名オーバーロードは同一 effect を要求する。pure / impure を同名だけで分岐させる API 設計は採用しない。effect が異なる場合はモジュールを分けるか、別名関数を使う。

---

## 9. 標準 trait 一覧

### 9.1 基本 trait

| Trait | 意味 |
|-------|------|
| `Eq` | 等値比較 |
| `Ord` | 全順序比較 |
| `Hash` | ハッシュ計算 |
| `Stringify` | 人間可読文字列変換 |
| `Debug` | デバッグ表現 |
| `Serialize .F` | 指定フォーマットへの直列化 |
| `Deserialize .F` | 指定フォーマットからの逆直列化 |
| `Parse` | 文字列からのパース |
| `Default` | デフォルト値生成 |
| `Clone` | 明示的複製 |
| `Copy` | 暗黙複製（`Clone` の特殊ケース） |
| `Add .U .R` | 加算演算（`a + b` の演算子オーバーロードは |> 経由） |

### 9.2 move / memory 系 trait

| Trait | 意味 |
|-------|------|
| `Drop` | scope exit / overwrite で自動 drop |
| `RegionOwned` | region 管理下の所有者 |
| `MemReadable .T` | メモリから .T を読み出す能力（将来導入） |
| `MemWritable .T` | メモリへ .T を書き込む能力（将来導入） |

### 9.3 IO 系 trait

| Trait | 意味 |
|-------|------|
| `Reader` | バイト列の読み出し |
| `Writer` | バイト列の書き込み |
| `Seekable` | シーク操作 |
| `Buffered` | バッファリング |
| `EventSource .E` | イベント発行源 |
| `EventSink .E` | イベント消費先 |
