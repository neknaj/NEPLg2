# NEPLg3 型システム仕様

最終更新: 2026-03-27

---

## 1. 設計方針

NEPLg3 では、式だけでなく型も括弧なし前置記法で書く。Zenn #1 のカリー化方針を型記法にもそのまま適用する。

- `.T -> .S` は `fn .T .S`
- `i32 -> i32 -> i32` は `fn i32 fn i32 i32`
- `Pair (.T, .S)` は `Pair .T .S`

---

## 2. 型式の文法

```
TypeExpr :=
    ()
  | never
  | i32 | u8 | f32 | bool | str
  | .T
  | Name
  | TypeExpr TypeExpr
  | fn TypeExpr TypeExpr
  | fn* TypeExpr TypeExpr
  | &TypeExpr
  | &mut TypeExpr
```

`TypeExpr TypeExpr` は型適用であり、kind-directed に境界を決定する。

`fn` は unary function type constructor である。複数引数関数は、戻り値側にさらに `fn ... ...` を置く。

```nepl
fn i32 i32
fn i32 fn i32 i32
fn Pair i32 bool i32
```

`fn*` は副作用を持つ関数型を表すための拡張であり、本仕様書では effect 章との整合のために同じ前置形式で扱う。

---

## 3. kind-directed 型解決

型の読み取りも、式の読み取りと同様に左から右へ進める。型コンストラクタの kind が決まっていれば、どこまでが 1 個の型なのかを一意に定められる。

### 3.1 例

```nepl
Pair i32 bool
```

- `Pair` は 2 引数の型コンストラクタ
- `i32` を 1 個目に適用
- `bool` を 2 個目に適用
- 結果は 1 個の `TypeExpr`

```nepl
fn Pair i32 bool i32
```

- 外側の `fn` は 2 個の `TypeExpr` を取る
- 第 1 引数は `Pair i32 bool`
- 第 2 引数は `i32`

```nepl
fn i32 fn i32 i32
```

- 外側の `fn` の第 1 引数は `i32`
- 第 2 引数は `fn i32 i32`
- したがって `i32 -> i32 -> i32`

---

## 4. `%` による型注釈

`%` は宣言専用記号ではなく、**続く 1 個の式に型注釈を与える前置演算子**である。

```nepl
%i32 1
%i32 add 1 2
add %i32 1 2
```

上の 3 例は、それぞれ次のように読む。

- `%i32 1`
- `%i32 (add 1 2)`
- `add (%i32 1) 2`

### 4.1 意味

Zenn #2 に従い、`%T e` は型検査上 `T -> T` 型の恒等関数を 1 回適用したものとして扱う。

```nepl
%.T x
```

これは `.T -> .T` の恒等関数を通したのと同様に扱う。

### 4.2 使用箇所

`%` は式ならどこにでも現れうる。

- let 束縛の右辺
- 関数引数位置
- block の途中
- if / match の各 arm の式

このため、「let の型注釈」「関数宣言の型注釈」といった専用構文を基本に置かない。

---

## 5. 基本型

| 型 | 意味 |
|---|---|
| `()` | unit 型 |
| `never` | 値を持たない型 |
| `bool` | 真偽値 |
| `i32` | 32 bit 符号付き整数 |
| `u8` | 8 bit 符号なし整数 |
| `f32` | 32 bit 浮動小数点 |
| `str` | UTF-8 不変文字列 |

`()` は型名でもあり、その唯一の値の表記でもある。

---

## 6. 複合型

### 6.1 struct / enum / 型コンストラクタ

通常の型コンストラクタもすべて前置で書く。

```nepl
Point
Option i32
Result i32 str
Pair i32 bool
```

### 6.2 関数型

1 引数関数:

```nepl
fn i32 bool
```

2 引数関数:

```nepl
fn i32 fn i32 i32
```

0 引数関数:

```nepl
fn () i32
```

---

## 7. 型パラメータ

型変数は `.T` のように書く。

```nepl
let id .T %fn .T .T \a a
let Pair struct .A .B:
    first: .A
    second: .B
```

trait 境界や `where` 節など、Zenn #1 / #2 で未確定の領域は、本仕様書の他章で将来設計として扱う。

---

## 8. ジェネリクスの変位

NEPLg3 では、ジェネリック型パラメータは不変として扱う。

```nepl
Vec i32
Vec str
```

これらの間に部分型関係はない。

---

## 付録: 字句上の注意

### `fn`

`fn` は型式にだけ現れる。定義キーワードとしての `fn` は使わず、定義は `let` に統一する。

### `%`

`%` の適用先は「続く 1 個の式」である。`%` から始まる区間を宣言専用の注釈領域として扱ってはならない。
