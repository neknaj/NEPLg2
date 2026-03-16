# NEPLg2.1 宣言構文仕様

最終更新: 2026-03-16

---

## 1. 宣言の統一原則

NEPLg2.1 ではすべての宣言を `let` キーワードで行う。関数・struct・enum・trait・impl はすべて `let` で始まる。型注釈は `%TypeExpr` 記法を使う。

---

## 2. 関数定義

```
let <name> [<type_params>] %fn [<param_types>] -> <ret_type>
    [where <constraints>]
    \ [<params>] :
    <body>
```

### 2.1 基本形

```nepl
let add2 %fn i32 i32 -> i32 \ a b :
    add_impl a b

let id .T %fn .T -> .T \ x :
    x

let not %fn bool -> bool \ x :
    if x: false else: true

let new %fn unit -> StringBuilder \ :
    sb_alloc unit
```

### 2.2 副作用関数（Impure）

```nepl
let print_line %fn* str -> unit \ s :
    ...

let open_file %fn* Path Mode -> Result File IoError \ path mode :
    ...
```

### 2.3 型パラメータと制約

```nepl
// 型パラメータのみ
let map .T .U %fn Option .T fn .T -> .U -> Option .U \ o f :
    match o:
        Option::Some x: Option::Some f x
        Option::None:   Option::None

// 型パラメータ + インライン制約
let sort .T: Ord %fn* Vec .T -> Vec .T \ v :
    ...
```

### 2.4 `where` 節

複数の制約または複雑な制約は `where` 節で分離して書く。

- 配置: 型シグネチャ（`%fn ... -> RetType`）の直後、引数リスト（`\ params :`）の直前
- 制約の区切りはスペースのみ（カンマなし）
- `where` 節自体に `:` は付けない（引数リスト `\ params :` の `:` が本体の開始を示す）

```nepl
let merge .T .K .V %fn Vec .T Vec .T -> Vec .T
    where .T: Ord .K: Hash .V: Eq
    \ a b :
    ...
```

Phase 8（依存型）では、型変数束縛（`.T: Trait`）に加えて命題制約（`%PropType`）も `where` に書ける（詳細は [phase8.md](./phase8.md)）。

```nepl
// Phase 8 example: 証明オブジェクトが where に現れる
let get .T .len .idx %fn Vec .T .len .idx -> .T
    where %IsLess .idx .len
    \ vec index :
    ...
```

### 2.5 引数リスト記法 `\ params :`

- 引数の束縛は `\ param1 param2 ... :` の形で書く。
- 括弧・カンマは使わない。
- 引数なし（unit 引数）は `\ :` と書く。
- `\` は型注釈の後ろに続き、`:` の後がボディになる。

### 2.6 巻き上げ（hoisting）

`let` バインディングのうち型注釈が `fn`/`fn*` 型のものは、宣言されたスコープ内で巻き上げが有効。相互再帰も可能。

---

## 3. struct 定義

```
let <Name> struct [<type_params>] :
    <field_name> %<TypeExpr>
    ...
```

```nepl
let Point struct:
    x %i32
    y %i32

let Pair struct .A .B:
    fst %.A
    snd %.B

let Node struct .T:
    val %.T
    next %Option Node .T    // ネスト型: Option (Node .T)
```

`%Option Node .T` の解析: `Option : * -> *`、引数として `Node .T`（`Node : * -> *`、`.T` で kind `*` に → `Node .T : *`）→ `Option (Node .T) : *` ✓

---

## 4. enum 定義

```
let <Name> enum [<type_params>] :
    <VariantName> [%<TypeExpr>]    // ペイロードなしは名前のみ
    ...
```

```nepl
let Option enum .T:
    Some %.T
    None

let Result enum .T .E:
    Ok %.T
    Err %.E

let Mode enum:
    Read
    Write
    Append
```

---

## 5. trait 定義

```
let <Name> trait [<type_params>] :
    let <method_name> %<fn_type> \ <params> :
        <default_body_or_...>
```

```nepl
let Eq trait:
    let eq %fn Self Self -> bool \ a b :
        ...

let Ord trait:
    let cmp %fn Self Self -> Ordering \ a b :
        ...
    let lt %fn Self Self -> bool \ a b :
        // デフォルト実装
        is_lt cmp a b
```

---

## 6. impl 定義

```
let <Type> impl for <Trait> [where <constraints>] :
    let <method_name> %<fn_type> \ <params> :
        <body>
```

```nepl
let i32 impl for Eq:
    let eq %fn i32 i32 -> bool \ a b :
        i32_eq a b

let Vec .T impl for Eq
    where .T: Eq :
    let eq %fn Vec .T Vec .T -> bool \ a b :
        vec_eq a b
```

---

## 7. let 型注釈（値バインディング）

```nepl
let a %Option i32 some 10
let checks %Vec Result unit str
```

`%Vec Result unit str`: `Vec : * -> *`、引数として `Result unit str`（`Result : * -> * -> *`、`unit : *`、`str : *` → `Result unit str : *`）→ `Vec (Result unit str) : *` ✓

---

## 8. `pub` と可視性

| 記法 | 意味 |
|------|------|
| （なし） | モジュール内のみ（暗黙 private） |
| `private` | 同上（明示版。省略しても同じ） |
| `pub let ...` | モジュール外から `use` で参照可能 |
| `pub use path` | 他モジュールの item を再エクスポート |

```nepl
pub let map .T .U %fn Option .T fn .T -> .U -> Option .U \ o f :
    ...
```

---

## 9. シャドウイングとオーバーロード

- 同名でもシグネチャが異なる宣言は**オーバーロード**として許可。
- 同名かつシグネチャが同一の宣言は warning を出し、後者が優先（shadowing）。
- `noshadow let` を付けた宣言は保護対象。同一シグネチャでの再定義はエラー。

```nepl
noshadow let eq %fn i32 i32 -> bool \ a b :   // 保護
    ...
```

---

## 10. `#module` / `#entry` / `#part` ヘッダ

ファイルの役割はヘッダで宣言する（詳細は [modules.md](./modules.md)）。

```nepl
#module    // 独立モジュールの anchor ファイル
#entry     // エントリポイント
#part      // merge されるパートファイル
```
