# NEPLg2.0 → NEPLg2.1 構文変更対照表

---

## 1. 前提

この文書では、NEPLg2.1 のうち Zenn #1「カリー化」と Zenn #2「型と制御構文」で明示された内容を正として、NEPLg2.0 との差分を整理する。

---

## 2. 宣言構文

| 構文 | NEPLg2.0（旧） | NEPLg2.1（新） |
|---|---|---|
| 関数定義 | `fn name <TypeParams> <Sig> (params):` | `let name <expr>` |
| lambda | `(a, b): body` | `\a \b body` |
| 0 引数関数 | `fn main <()->i32> ():` | `let main \(): ...` |
| struct 定義 | `struct Name<.T>:` | `let Name struct .T:` |
| enum 定義 | `enum Name<.T>:` | `let Name enum .T:` |
| trait 定義 | `trait Name:` | `let Name trait:` |
| impl 定義 | `impl Trait for Type:` | `let Type impl for Trait:` |

### 例

```nepl
// NEPLg2.0
fn id <.T> <(.T)->.T> (x):
    x
```

```nepl
// NEPLg2.1
let id \x x
```

---

## 3. 型記法

| 箇所 | NEPLg2.0（旧） | NEPLg2.1（新） |
|---|---|---|
| 型適用 | `Name<A, B>` | `Name A B` |
| 型パラメータ宣言 | `<.T, .U>` | `.T .U` |
| 1 引数関数型 | `A -> B` | `fn A B` |
| 2 引数関数型 | `(A, B) -> C` | `fn A fn B C` |
| 1 引数 Impure 関数型 | `A *> B` | `fn* A B` |
| unit 型 / 値 | `()` | `()` |
| 型注釈 | `<TypeExpr>` | `%TypeExpr <expr>` |

### 例

| NEPLg2.0 | NEPLg2.1 |
|---|---|
| `Vec<i32>` | `Vec i32` |
| `Result<i32, str>` | `Result i32 str` |
| `(i32) -> i32` | `fn i32 i32` |
| `(i32, i32) -> i32` | `fn i32 fn i32 i32` |
| `() -> i32` | `fn () i32` |

---

## 4. `%` の意味

これは旧 2.1 案からも変わった点である。

| 旧案 | Zenn #1 / #2 を正とした 2.1 |
|---|---|
| 宣言で型注釈を始める記号 | 続く 1 個の式に作用する前置演算子 |

```nepl
%i32 add 1 2
add %i32 1 2
```

---

## 5. 関数と部分適用

| 項目 | NEPLg2.0 | NEPLg2.1 |
|---|---|---|
| 関数の内部モデル | 通常の多引数関数 | カリー化 |
| 部分適用 | 実質的に使える場面がある | 不採用 |

```nepl
add 1
```

NEPLg2.1 ではこれは値ではなくエラーとして扱う。

---

## 6. 制御構文

| 構文 | NEPLg2.0（旧） | NEPLg2.1（新） |
|---|---|---|
| if | `if cond then a else b` | `if cond a b` または `if cond then a else b` |
| match arm | `pattern: suite` | `pattern expr` |
| block | 暗黙 block が中心 | `block:` を明示 |
| 中間式の破棄 | 文区切り中心 | 前置 `; expr` |

### 例

```nepl
if true 1 2
if true then 1 else 2

match x:
    or 1 2 println "one or two"
    _ println "other"

block:
    ; println "side effect"
    1
```

---

## 7. pattern

| 項目 | 旧案 | Zenn #2 を正とした 2.1 |
|---|---|---|
| OR | guard 側に寄せる | `or` pattern |
| range | 保留 | `span` pattern |
| struct 分解 | 位置ベース | field 名付き |

```nepl
match c:
    span 'a' 'j' println "early"
    _ println "other"

let Point x: a y: b p
```

---

## 8. 旧比較文書からの読み替え

この比較ディレクトリの旧文書では、`fn A B -> C`、`unit`、`\ a b :`、`if <cond> : <suite>` など、Zenn #1 / #2 確定前の 2.1 案を新仕様として記していた。現在はそれらを採用しない。

参照先:

- 現在のコア構文: [syntax.md](../2.1spec/syntax.md)
- 現在の型記法: [types.md](../2.1spec/types.md)
- 現在の宣言構文: [declarations.md](../2.1spec/declarations.md)
- 現在の pattern: [patterns.md](../2.1spec/patterns.md)
