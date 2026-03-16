# NEPLg2.1 型記法 仕様

最終更新: 2026-03-16

---

## 1. 設計方針

NEPLg2 の式は括弧なし・前置 juxtaposition で書かれ、呼び出し境界は型チェッカーが kind/type 情報を用いて決定する。
**型式も完全に同じ原則に従う。括弧は一切使わない。グループ化構文はない。**

### 1.1 現状の問題

| 箇所 | 現在 | 問題 |
|------|------|------|
| 関数型 | `(A, B) -> C` | 括弧・カンマ・中値 `->` |
| 型適用 | `Name<A, B>` | `<>` と カンマ |
| 型パラメータ宣言 | `<.T, .U>` | `<>` と カンマ |
| unit 型 | `()` | 空括弧という特殊記法 |
| 型注釈 | `<TypeExpr>` | `<>` ブラケット |

### 1.2 変更方針

| 変更 | 変更前 | 変更後 |
|------|--------|--------|
| unit 型のキーワード化 | `()` | `unit` |
| 関数型キーワード化 | `(A, B) -> C` | `fn A B -> C` |
| 副作用関数型 | `(A, B) *> C` | `fn* A B -> C` |
| 型適用を juxtaposition へ | `Name<A, B>` | `Name A B` |
| グループ化 → 廃止 | `<TypeExpr>` | なし（kind-directed で解決） |
| 型注釈記号の変更 | `<TypeExpr>` | `%TypeExpr` |
| 型パラメータ宣言の `<>` 廃止 | `<.T, .U>` | `.T .U` |
| 宣言キーワードを `let` に統一 | `fn`/`struct`/`enum`/`trait`/`impl` | `let Name [kind]` |

---

## 2. 型式の文法

```
TypeExpr :=
    unit                              // unit型
  | never                             // never型
  | i32 | u8 | f32 | bool | str      // プリミティブ型
  | .T                                // 型変数（ラベル）
  | Name                              // 型コンストラクタ（0引数）
  | TypeExpr TypeExpr                 // 型適用（juxtaposition、左結合）
  | fn TypeExpr* -> TypeExpr          // 純粋関数型
  | fn* TypeExpr* -> TypeExpr         // 副作用関数型
  | &TypeExpr                         // 共有参照
  | &mut TypeExpr                     // 可変参照
```

括弧によるグループ化は存在しない。境界は kind-directed アルゴリズムで決定する。

---

## 3. Kind-Directed 型解決アルゴリズム

### 3.1 式の呼び出し解決との対応

式において `add sub 1 2 3` は括弧なしで `add (sub 1 2) 3` に解決される。
型チェッカーが各関数の arity と型から呼び出し境界を決定するためである。

型式も同じアルゴリズムを用いる。型コンストラクタの **kind** が arity に相当する。

| 式での概念 | 型での対応 |
|------------|-----------|
| 関数の型（arity） | 型コンストラクタの kind |
| `fn A B -> C` の戻り型 | `* -> * -> *` の最終 `*` |
| 型チェッカーによる境界確定 | kind チェッカーによる境界確定 |

### 3.2 種（Kind）による型適用境界の決定

型コンストラクタは kind を持つ。

| 型 | Kind |
|----|------|
| `i32`, `bool`, `str`, `unit` | `*` |
| `Option` | `* -> *` |
| `Vec` | `* -> *` |
| `Result` | `* -> * -> *` |
| `Pair` | `* -> * -> *` |
| `.T`（型変数） | `*`（デフォルト）または kind 推論 |

kind が `*` になるまで juxtaposition で型を適用する。

```
Vec Option i32
  └─ Vec : * -> *   → 1引数必要
  └─ Option : * -> *（kind * でない）→ i32 を適用
  └─ Option i32 : * ✓
  └─ Vec (Option i32) : * ✓

Result i32 str
  └─ Result : * -> * -> *  → 2引数必要
  └─ i32 : *、str : *
  └─ Result i32 str : * ✓
```

### 3.3 `fn`/`fn*` における `->` の帰属

式で innermost の `fn` が `->` を先取りするのと同様に、
型式でも innermost の `fn`/`fn*` が **最初に出現する `->` を先取りする**。

```
fn fn i32 -> i32 -> i32
  ├─ 外側 fn: args を集める
  │   └─ 内側 fn: i32 を引数として取り、-> i32 を先取り → fn i32 -> i32 : *
  ├─ 外側 fn の唯一の引数: fn i32 -> i32
  └─ 外側 -> i32 が続く → fn (fn i32 -> i32) -> i32 ✓

fn fn i32 -> i32 fn i32 -> i32 -> i32
  ├─ 外側 fn: 2つの引数
  │   ├─ 内側 fn1: i32 -> i32 → fn i32 -> i32 : *
  │   └─ 内側 fn2: i32 -> i32 → fn i32 -> i32 : *
  └─ 外側 -> i32 → fn (fn i32 -> i32) (fn i32 -> i32) -> i32 ✓
```

`fn`/`fn*` の引数列は、自身に属する `->` が出現するまで続く。

### 3.4 関数型を引数に持つ型適用

型コンストラクタの引数として `fn` 型を置く場合も、kind-directed で自然に解決される。

```
Result fn i32 -> i32 str
  ├─ Result : * -> * -> *
  ├─ 第1引数: fn i32 -> i32（fn が -> を先取り → : *）
  └─ 第2引数: str : *
  → Result (fn i32 -> i32) str ✓

fn Option i32 -> i32
  ├─ 外側 fn の引数列: Option i32（Option : * -> *、i32 で kind * に）
  └─ -> i32 → fn (Option i32) -> i32 ✓
```

---

## 4. 型注釈記号 `%`

`%TypeExpr` は **型注釈の開始を示す接頭辞**である。
`<TypeExpr>` を完全に置き換える。`%` の後は型式が続き、その境界は kind-directed アルゴリズムで決定する。
閉じ記号はない。

### 4.1 使用場所

| 用途 | 旧 | 新 |
|------|-----|-----|
| 関数シグネチャ注釈 | `<fn A -> B>` | `%fn A -> B` |
| let 型注釈 | `<Option i32>` | `%Option i32` |
| struct フィールド型 | `fname <TypeExpr>` | `fname %TypeExpr` |
| enum バリアントペイロード | `Variant <TypeExpr>` | `Variant %TypeExpr` |
| 式中の型注釈 | `expr<TypeArgs>` | `expr %TypeArgs`（TBD） |

### 4.2 `%` の終端

`%` に続く型式は kind-directed アルゴリズムで kind `*` になった時点で終端する。
宣言構文の文脈（改行・`:`・値引数の `(`）が自然な区切りとなる。

---

## 5. 各宣言での記法

### 5.1 関数定義

`fn` キーワードは**宣言キーワードとして廃止**する。

**廃止の理由**: 旧仕様では型記法に `fn` が現れなかったため `fn` を宣言キーワードとして使えた。
新仕様では型注釈が `%fn ...` / `%fn* ...` の形で `fn` を含むため、宣言キーワードとしての `fn` は紛らわしい。

関数定義はすべて `let` を使う。型注釈 `%fn ...` が関数型であることを示す。

```nepl
// 旧（fn エイリアス）
fn main <()*>i32> ():
fn id <.T> <(.T)->.T> (x):
fn add2 <(i32,i32)->i32> (a, b):
fn map <.T,.U> <(Option<.T>, (.T)->.U)->Option<.U>> (o, f):
fn and_then <.T,.U> <(Option<.T>, (.T)->Option<.U>)->Option<.U>> (o, f):

// 新（let に統一）
let main %fn* -> i32 ():
let id .T %fn .T -> .T (x):
let add2 %fn i32 i32 -> i32 (a, b):
let map .T .U %fn Option .T fn .T -> .U -> Option .U (o, f):
let and_then .T .U %fn Option .T fn .T -> Option .U -> Option .U (o, f):
```

`let map .T .U %fn Option .T fn .T -> .U -> Option .U` の解析：
- `.T .U` = 型パラメータ（`.` 始まりで識別子と区別）
- `%fn ...` = 型シグネチャ注釈の開始
- `fn` = 外側の関数型コンストラクタ
  - 引数1: `Option .T`（`Option : * -> *`、`.T` で kind `*` に）
  - 引数2: `fn .T -> .U`（inner fn が `->` を先取り → kind `*`）
  - 外側 `->`: 区切り
  - 戻り型: `Option .U`（kind `*`）

**巻き上げ（hoisting）**: `let` バインディングのうち型注釈が `fn`/`fn*` 型のものは、旧 `fn` と同様にスコープ内で巻き上げが有効になる。

### 5.2 struct 定義

`let Name struct` 形式。型パラメータは `.Ident` で列挙。フィールド型は `fname %TypeExpr`。

```nepl
// 旧
struct Pair<.A, .B>:
    fst <.A>
    snd <.B>

// 新
let Pair struct .A .B:
    fst %.A
    snd %.B
```

```nepl
// 旧
struct Node<.T>:
    val <.T>
    next <Option<Node<.T>>>

// 新
let Node struct .T:
    val %.T
    next %Option Node .T
```

`%Option Node .T` の解析：`Option : * -> *`、引数として `Node .T`（`Node : * -> *`、`.T` で kind `*` に → `Node .T : *`）→ `Option (Node .T) : *` ✓

### 5.3 enum 定義

`let Name enum` 形式。バリアントは `Name %TypeExpr`（ペイロードなしは `Name` のみ）。

```nepl
// 旧
enum Result<.T, .E>:
    Ok <.T>
    Err <.E>

// 新
let Result enum .T .E:
    Ok %.T
    Err %.E
```

```nepl
// 旧
enum Option<.T>:
    Some <.T>
    None

// 新
let Option enum .T:
    Some %.T
    None
```

### 5.4 let 型注釈（値バインディング）

```nepl
// 旧
let a <Option<i32>> some<i32> 10
let checks <Vec<Result<(),str>>>

// 新
let a %Option i32 some 10
let checks %Vec Result unit str
```

`%Vec Result unit str`：`Vec : * -> *`、引数として `Result unit str`（`Result : * -> * -> *`、`unit : *`、`str : *` → `Result unit str : *`）→ `Vec (Result unit str) : *` ✓

### 5.5 trait 定義・impl

`let Name trait` 形式。`let Type impl for Trait` 形式。

```nepl
// 旧
trait Eq:
    fn eq <(Self, Self)->bool> (a, b):
        ...

impl Eq for i32:
    fn eq <(i32, i32)->bool> (a, b):
        ...

// 新
let Eq trait:
    let eq %fn Self Self -> bool (a, b):
        ...

let i32 impl for Eq:
    let eq %fn i32 i32 -> bool (a, b):
        ...
```

### 5.6 trait 境界

型パラメータに制約を付ける場合は `.T: Trait` 形式を使う。

```nepl
// 旧
fn sort <.T: Ord> <(Vec<.T>) *> Vec<.T>> (v):

// 新
let sort .T: Ord %fn* Vec .T -> Vec .T (v):
```

複数の型パラメータで一部に制約がある場合：

```nepl
let zip .T .U: Show %fn Vec .T Vec .U -> Vec Pair .T .U (a, b):
```

---

## 6. 基本型の変化まとめ

| 現在 | 新 | 説明 |
|------|----|------|
| `()` | `unit` | unit 型 |
| `Vec<i32>` | `Vec i32` | 単一型引数 |
| `Option<i32>` | `Option i32` | 単一型引数 |
| `Result<i32, str>` | `Result i32 str` | 2引数 |
| `Vec<Option<i32>>` | `Vec Option i32` | ネスト（kind-directed） |
| `Vec<Result<(), str>>` | `Vec Result unit str` | ネスト |
| `(i32) -> i32` | `fn i32 -> i32` | 関数型 |
| `(i32, i32) -> i32` | `fn i32 i32 -> i32` | 2引数 |
| `() -> i32` | `fn -> i32` | 0引数 |
| `() *> i32` | `fn* -> i32` | 副作用 |
| `((i32)->i32) -> i32` | `fn fn i32 -> i32 -> i32` | 高階（括弧不要） |
| `(Option<.T>, (.T)->.U)->Option<.U>` | `fn Option .T fn .T -> .U -> Option .U` | 高階ネスト |
| `Result<(i32)->str, str>` | `Result fn i32 -> str str` | 型引数内の関数型 |

---

## 7. 変更しないもの

- `.T`、`.label` — 型変数（ラベル記法）
- `&T`、`&mut T` — 参照型（既に前置的）
- `fn`/`fn*` の `->` セパレータ（型式文脈内のみ）
- 値引数リスト `(a, b)` の括弧（型記法ではなく構文）

---

## 8. 移行インパクト

1. **Parser 拡張（両対応期）**: `%` 記法・juxtaposition 型適用・`unit` を受け付けつつ旧記法も警告付きで受け付ける
2. **stdlib 全体の書き換え**: すべての関数シグネチャ・struct/enum 定義を新形式に移行
3. **チュートリアル・テストの更新**: ドキュメントとテストケースを新形式に更新
4. **旧構文廃止**: 移行完了後に旧記法を削除

---

## 付録: 字句解析・構文解析上の注意

### `%` の扱い

- `%` は型注釈コンテキストへの切り替えを示す接頭辞トークン
- `%` の後はトップレベル型式が続く（kind `*` になるまで）
- 閉じ記号なし

### `fn` の使用範囲

`fn` は**型式文脈のみ**で使用する。宣言キーワードとしての `fn` は廃止。

- 型式文脈（`%fn ...` の内部）: 関数型コンストラクタ
- 宣言文脈: `let` のみ使用（`fn` は使わない）

### `->` の使用範囲

`->` は型式文脈の `fn`/`fn*` 内のみで有効。式文脈では使用しない。
innermost の `fn`/`fn*` が最初の `->` を先取りする。

### 型パラメータの認識

宣言（`let`/`struct`/`enum`）において、名前の後ろに続く `.Ident` または `.Ident: Trait` のトークン列を型パラメータとして認識する。
`%` または `:` が現れた時点で型パラメータ列が終了する。
