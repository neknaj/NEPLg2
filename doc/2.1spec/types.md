# NEPLg2.1 型システム仕様

最終更新: 2026-03-16

---

## 1. 設計方針

式は括弧なし・前置 juxtaposition で書かれ、呼び出し境界は型チェッカーが kind/type 情報を用いて決定する。**型式も完全に同じ原則に従う。括弧は一切使わない。グループ化構文はない。**

---

## 2. 型式の文法

```
TypeExpr :=
    unit                              // unit 型
  | never                             // never 型
  | i32 | u8 | f32 | bool | str      // プリミティブ型
  | .T                                // 型変数
  | Name                              // 型コンストラクタ（0 引数）
  | TypeExpr TypeExpr                 // 型適用（juxtaposition、左結合）
  | fn TypeExpr+ -> TypeExpr          // 純粋関数型（引数は 1 つ以上; 入力不要な場合は fn unit -> T）
  | fn* TypeExpr+ -> TypeExpr         // 副作用関数型（引数は 1 つ以上）
  | &TypeExpr                         // 共有参照
  | &mut TypeExpr                     // 可変参照
```

括弧によるグループ化は存在しない。境界は kind-directed アルゴリズムで決定する。

---

## 3. Kind-Directed 型解決アルゴリズム

### 3.1 式の呼び出し解決との対応

式において `add sub 1 2 3` は括弧なしで `add (sub 1 2) 3` に解決される。型チェッカーが各関数の arity と型から呼び出し境界を決定するためである。型式も同じアルゴリズムを用いる。型コンストラクタの **kind** が arity に相当する。

| 式での概念 | 型での対応 |
|------------|-----------|
| 関数の型（arity） | 型コンストラクタの kind |
| `fn A B -> C` の戻り型 | `* -> * -> *` の最終 `*` |
| 型チェッカーによる境界確定 | kind チェッカーによる境界確定 |

### 3.2 Kind テーブル

型コンストラクタは kind を持つ。

| 型 | Kind |
|----|------|
| `i32`, `bool`, `str`, `unit` | `*` |
| `Option` | `* -> *` |
| `Vec` | `* -> *` |
| `Result` | `* -> * -> *` |
| `Pair` | `* -> * -> *` |
| `Triple` | `* -> * -> * -> *` |
| `.T`（型変数） | `*`（デフォルト）または kind 推論 |

kind が `*` になるまで juxtaposition で型を適用する。

```
Vec Option i32
  └─ Vec : * -> *   → 1 引数必要
  └─ Option : * -> *（kind * でない）→ i32 を適用
  └─ Option i32 : * ✓
  └─ Vec (Option i32) : * ✓

Result i32 str
  └─ Result : * -> * -> *  → 2 引数必要
  └─ i32 : *、str : *
  └─ Result i32 str : * ✓
```

### 3.3 `fn`/`fn*` における `->` の帰属

型式でも innermost の `fn`/`fn*` が **最初に出現する `->` を先取りする**。

```
fn fn i32 -> i32 -> i32
  ├─ 外側 fn: args を集める
  │   └─ 内側 fn: i32 を引数として取り、-> i32 を先取り → fn i32 -> i32 : *
  ├─ 外側 fn の唯一の引数: fn i32 -> i32
  └─ 外側 -> i32 が続く → fn (fn i32 -> i32) -> i32 ✓

fn fn i32 -> i32 fn i32 -> i32 -> i32
  ├─ 外側 fn: 2 つの引数
  │   ├─ 内側 fn1: i32 -> i32 → fn i32 -> i32 : *
  │   └─ 内側 fn2: i32 -> i32 → fn i32 -> i32 : *
  └─ 外側 -> i32 → fn (fn i32 -> i32) (fn i32 -> i32) -> i32 ✓
```

`fn`/`fn*` の引数列は、自身に属する `->` が出現するまで続く。

### 3.4 関数型を引数に持つ型適用

```
Result fn i32 -> i32 str
  ├─ Result : * -> * -> *
  ├─ 第 1 引数: fn i32 -> i32（fn が -> を先取り → : *）
  └─ 第 2 引数: str : *
  → Result (fn i32 -> i32) str ✓

fn Option i32 -> i32
  ├─ 外側 fn の引数列: Option i32（Option : * -> *、i32 で kind * に）
  └─ -> i32 → fn (Option i32) -> i32 ✓
```

---

## 4. 型注釈記号 `%`

`%TypeExpr` は **型注釈の開始を示す接頭辞**。`%` の後は型式が続き、その境界は kind-directed アルゴリズムで決定する。閉じ記号はない。

### 4.1 使用場所

| 用途 | 記法 |
|------|------|
| 関数シグネチャ注釈 | `%fn A -> B` |
| let 型注釈 | `%Option i32` |
| struct フィールド型 | `fname %TypeExpr` |
| enum バリアントペイロード | `Variant %TypeExpr` |

### 4.2 `%` の終端

`%` に続く型式は kind-directed アルゴリズムで kind `*` になった時点で終端する。宣言構文の文脈（改行・`:`・引数リスト `\`）が自然な区切りとなる。

---

## 5. 基本型一覧

| 型 | 意味 |
|----|------|
| `unit` | 唯一の値しか持たない型（`unit` そのものが値） |
| `never` | 値を持たない型（発散する計算の戻り型） |
| `bool` | `true` / `false` |
| `i32` | 32 bit 符号付き整数 |
| `u8` | 8 bit 符号なし整数 |
| `f32` | 32 bit 浮動小数点 |
| `str` | UTF-8 不変文字列（Pure Persistent Value） |
| `Option .T` | 値がある（`Some .T`）または存在しない（`None`） |
| `Result .T .E` | 成功（`Ok .T`）または失敗（`Err .E`） |

---

## 6. 複合型

### 6.1 Pair と Triple

言語組み込みの Tuple 構文はない。stdlib が `Pair .A .B` と `Triple .A .B .C` を通常の struct として提供する。

```nepl
let Pair struct .A .B:
    fst %.A
    snd %.B

let Triple struct .A .B .C:
    fst %.A
    snd %.B
    trd %.C
```

構築は他のコンストラクタと同じ前置記法:

```nepl
let p Pair 1 2
let t Triple 1 2 3
```

### 6.2 参照型

| 型 | 意味 |
|----|------|
| `&.T` | 共有参照（shared borrow） |
| `&mut .T` | 可変参照（unique borrow） |

---

## 7. 型パラメータ

宣言（`let`）において、名前の後ろに続く `.Ident` または `.Ident: Trait` のトークン列を型パラメータとして認識する。`%` または `:` が現れた時点で型パラメータ列が終了する。

```nepl
let id .T %fn .T -> .T \ x : x          // .T が型パラメータ

let sort .T: Ord %fn* Vec .T -> Vec .T \ v :   // .T: Ord は制約付き型パラメータ
    ...
```

---

## 8. trait 境界

型パラメータに制約を付ける場合は `.T: Trait` 形式を使う。

```nepl
let sort .T: Ord %fn* Vec .T -> Vec .T \ v :
    ...
```

複数の型パラメータで一部に制約がある場合:

```nepl
let zip .T .U: Show %fn Vec .T Vec .U -> Vec Pair .T .U \ a b :
    ...
```

複数制約または複雑な制約は `where` 節で分離する（詳細は [declarations.md](./declarations.md) §4）。

---

## 9. ジェネリクスの変位（Variance）

NEPLg2.1 では、ジェネリック型のパラメータは**不変（invariant）**である。`Vec i32` と `Vec str` には部分型関係がない（サブタイピングはない）。

```
Vec i32  ≠  Vec str  — 型として完全に別物
Option i32  ≠  Option str
```

これにより変位に起因する型安全バグを構造的に排除し、型推論を単純化する。

Phase 8（依存型）以降で co-/contravariance の限定的な導入を検討するが、Phase 0–7 ではすべてのジェネリック型パラメータが不変として扱われる。

---

## 付録: 字句解析上の注意

### `%` の扱い

- `%` は型注釈コンテキストへの切り替えを示す接頭辞トークン。
- `%` の後はトップレベル型式が続く（kind `*` になるまで）。
- 閉じ記号なし。

### `fn` の使用範囲

`fn` は**型式文脈のみ**で使用する。宣言キーワードとしての `fn` は存在しない。

- 型式文脈（`%fn ...` の内部）: 関数型コンストラクタ
- 宣言文脈: `let` のみ使用

### `->` の使用範囲

`->` は型式文脈の `fn`/`fn*` 内のみで有効。式文脈では使用しない。innermost の `fn`/`fn*` が最初の `->` を先取りする。
