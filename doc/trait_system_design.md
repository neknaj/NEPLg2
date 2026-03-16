# NEPLg2 trait 設計指針

最終更新: 2026-03-16

## 1. 目的

- NEPLg2.1 の式指向・前置記法・型記法（`doc/type_notation_spec.md`）を崩さずに trait を拡張する。
- move/pure-impure/メモリ安全設計と矛盾しない trait システムを確立する。
- 場当たりの文字列分岐を減らし、trait 契約を型システム側で一貫管理する。

## 2. 設計原則

- trait は「メソッド集合」ではなく、型能力の契約として扱う。
- オーバーロード解決と trait 境界判定は同じ型同値判定で統一する。
- 実装一意性は `(trait, target type)` の組で保証する。
- move/effect 判定は trait 判定と独立させる。ただし `Copy/Clone` は move 規則へ接続する。

## 3. NEPLg2 での trait の役割

### 3.1 Interface 相当

- 共通メソッド契約を提供する。
- `let Type impl for Trait:` で実装する（NEPLg2.1 宣言構文）。

### 3.2 Type Class/Concept 相当

- 型引数境界を `.T: Trait` 形式または `where` 節で表現する（例: `.T: Eq`）。
- 呼び出し時に `trait_bound_satisfied` で充足判定する。

### 3.3 move/memory 相当

- `Copy` / `Clone` は所有権規則に接続する能力 trait として扱う。
- 将来導入する `MemReadable .T`, `MemWritable .T`, `RegionOwned` はメモリ能力の契約として扱う。

## 4. 一意性規則（coherence）

- 同一モジュール内では同一 `(trait, target type)` への重複 impl を禁止する。
- 判定は文字列化した型ではなく、構造的型同値（`same_type`）で行う。
- 重複検出後は後続パスで重複 impl を無視し、診断を安定化する。

## 5. シグネチャ整合

- trait メソッド実装の整合判定は構造型同値で行う。
- 文字列ベース比較は補助（mangle/デバッグ）に限定し、契約判定に使わない。

## 6. ハードコード最小化方針

- 型名ハードコード（例: 特定 struct 名での `Copy` 禁止）は禁止する。
- trait 参照の分岐は段階的に能力テーブルへ移す。
- 最終的には trait の「能力種別」を宣言側から供給し、コンパイラ側の名前比較を撤廃する。

### 6.1 移行段階

- 現段階では `Copy/Clone` の能力接続のため最小限の trait 名参照が残る。
- ただし判定対象型はすべて構造型同値ベースで扱い、特定型名の例外分岐は置かない。

## 7. 前置記法・オーバーロードとの整合

### 7.1 基本原則

- API 名は bare 名（prefix/suffix なし）で揃える。`get_opt` や `get_safe` のような接尾辞は使わない。
- trait 解決は既存の前置呼び出しモデルに従う。
- 型注釈は `%TypeExpr` 記法を使う（`<>` 囲みは廃止）。
- 暗黙 cast による trait/overload 解決は導入しない。

### 7.2 モジュール分離によるオーバーロード管理

オーバーロードの複雑化は、**同じ bare 名を持つ関数を別モジュールに配置し、`use` 対象を切り替えることで解決する**。

典型例: Core 層（依存型・証明必須）と Casual 層（Result/Option ラッパー）を別モジュールに分ける。

```nepl
// Core 層: 型安全を静的に保証
use core::collections::vec as *
get vec proof    // where %IsLess idx len が必要、戻り型 .T

// Casual 層: 実行時検査、Option で返す
use std::collections::vec as *
get vec 3        // 証明不要、戻り型 Option .T
```

モジュール修飾名により、両方を `use` している状態でも常に明示できる。

```nepl
core::vec::get vec proof
std::vec::get vec 3
```

### 7.3 両モジュールを use したときのオーバーロード解決

同一スコープに複数のオーバーロード候補が存在する場合、コンパイラは以下の順で解決を試みる。

**段階 1: 制約フィルタリング（where 節の充足可否）**

`where` 節を充足できない候補を除外する。これが最も強力なフィルタとなる。

```nepl
use core::collections::vec as *
use std::collections::vec as *

let x get vec 3
// core::vec::get の where %IsLess 3 len → scope に証明なし → 除外
// std::vec::get → 残る → Option .T に解決
```

```nepl
let x get vec proof   // proof : IsLess idx len が scope にある
// core::vec::get の where を充足 → 残る
// std::vec::get → i32 を期待するが proof は IsLess 型 → 型不一致 → 除外
// core::vec::get に解決
```

**段階 2: 期待型による絞り込み（双方向型推論）**

制約フィルタ後も複数候補が残った場合、呼び出し元の期待型を call site に伝播させて絞る。

```nepl
let x %Option .T get vec idx    // 期待型 Option .T → std::vec::get に解決
let x %.T        get vec idx    // 期待型 .T → core::vec::get に解決
```

**段階 3: 修飾名の要求**

フィルタ後に複数候補が残り、期待型でも絞れない場合はコンパイルエラーとし、修飾名を要求する。

```nepl
let x get vec idx
// ERROR: ambiguous — use `core::vec::get` or `std::vec::get`
```

### 7.4 effect とオーバーロード

同名オーバーロードは同一 effect を要求する。pure/impure を同名だけで分岐させる API 設計は採用しない。effect が異なる場合はモジュールを分けるか、別名関数を使う。

## 8. 今後の拡張順序

1. `Copy/Clone` の能力判定を trait 能力テーブル化する。
2. `MemReadable .T`, `MemWritable .T`, `RegionOwned` を導入する。
3. move_check と trait 能力を連携し、token 消費規則を強化する。
4. stdlib の `mem` / `std/streamio` を trait 境界ベース API へ統一する。
5. 依存型（Phase 8）導入時に、`where` 節の命題型充足を制約フィルタに組み込む。
