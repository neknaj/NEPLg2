# NEPLg2.1 コア構文仕様

最終更新: 2026-03-27

---

## 1. 基本原則

NEPLg2.1 のコア構文は次の原則を貫く。

- 前置記法
- カリー化
- オフサイドルール
- `let <name> <expr>` への統一
- `%` を「次の 1 個の式に掛かる型注釈」として扱う
- 部分適用を不採用とする

---

## 2. リテラル

```
<literal> :=
    <integer>
  | <float>
  | true
  | false
  | <string>
  | ()
```

`()` は unit 型の唯一の値である。

---

## 3. 式

```
<expr> :=
    <literal>
  | <ident>
  | %<TypeExpr> <expr>
  | <expr> <expr>
  | <expr> |> <expr>
  | \ <param> <expr>
  | \ <param> : <block>
  | if <expr> <expr> <expr>
  | if <expr> then <expr> else <expr>
  | match <expr> : <match_arms>
  | let <name> <expr>
  | let mut <name> <expr>
  | <block_expr>
  | ; <expr>
```

0 引数 lambda では `<param>` に `()` を使う。

```nepl
\() 0
\():
    0
```

---

## 4. 前置適用

関数適用は引数を並べるだけで表す。式境界は、カリー化された関数型と型情報から決定する。

```nepl
add 1 2
mul add 1 2 3
```

部分適用はサポートしないため、最終的に関数型が残る式はエラーである。

```nepl
add 1
```

---

## 5. 型注釈 `%`

`%` は続く 1 個の式に作用する。

```nepl
%i32 add 1 2
add %i32 1 2
```

したがって、`%` は宣言文法の一部ではなく通常の式演算子として扱う。

---

## 6. `let`

```
let <name> <expr>
let mut <name> <expr>
```

`let` 式の型は `()` とする。

```nepl
let zero 0
let double \a add a a
let mut n 0
```

---

## 7. lambda

### 7.1 単一行

```nepl
\a add a a
```

### 7.2 複数行

```nepl
\a:
    ; println a
    add a a
```

### 7.3 複数引数

```nepl
\a \b add a b
```

---

## 8. `if`

Zenn #2 の current core syntax は次の 2 形である。

```nepl
if <cond_expr> <then_expr> <else_expr>
if <cond_expr> then <then_expr> else <else_expr>
```

```nepl
if true 1 2
if true then 1 else 2
```

`if` は 3 つの式を取る前置式であり、旧案の `else if ...:` 連鎖を基本文法とはしない。

---

## 9. `match`

```
match <expr>:
    <pattern> <expr>
    <pattern> <expr>
```

arm は `:<suite>` ではなく `<pattern> <expr>` で書く。

```nepl
match x:
    or 1 2 println "one or two"
    3 println "three"
    _ println "anything"
```

---

## 10. pattern の基本形

現時点で Zenn #2 が明示しているもの:

- `_`
- リテラル
- `or p q`
- `span a b`
- struct field pattern

```nepl
match c:
    span 'a' 'j' println "early ASCII letter"
    _ println "other"

let Point x: a y: b p
```

詳細は [patterns.md](./patterns.md) を参照。

---

## 11. オフサイドルール

Zenn #2 で明示されているオフサイドルールは次の通り。

- 1 個の式の改行は、インデント規則に反しない限り許可
- `block` 式、関数本体、`struct`、`enum`、`match` では `:` を使う
- `:` の次の行からインデントが増え、`:` を書いた行以下に戻ったらその範囲が終わる
- `#indent 4` のような指定がある場合、行頭インデントはその倍数でなければならない

---

## 12. `;`

前置 `;` は「式を評価して `()` にする」演算子として扱う。

```nepl
let unit_expr %() ; add 1 2
```

`block:` 内では途中式を捨てる用途に使う。

---

## 13. `block:`

`block:` は複数の式を一つの式にまとめる。

```nepl
block:
    ; expr1
    ; expr2
    expr3
```

最後の式の値が block 全体の値になる。

---

## 14. パイプ `|>`

`|>` は継続して採用するが、左辺・右辺とも**完全な式**でなければならない。部分適用を前提にした説明は採用しない。

```nepl
value |> f
value |> f |> g
```

右辺は `f` のような完全な関数値であり、`add 1` のような不完全な関数適用を置いてはならない。

---

## 15. 保留事項

`while`、`set`、高度な let pattern の網羅性、borrow 構文の最終形など、Zenn #1 / #2 でまだ確定していないものは他文書で将来設計として扱う。コア構文の正は本書の 1 から 14 とする。
