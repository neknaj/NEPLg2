# NEPLg2.1 パターン・Match・Let 仕様

最終更新: 2026-03-27

---

## 1. 方針

Zenn #2 を基準とし、`match` arm と `let` 分解は Rust 風の発想を前置記法へ写したものとして定義する。

---

## 2. パターンの種類

### 2.1 識別子パターン

```nepl
let x 42
```

### 2.2 ワイルドカード

```nepl
let _ expr
```

### 2.3 リテラルパターン

```nepl
match n:
    0 println "zero"
    _ println "other"
```

### 2.4 OR パターン

Zenn #2 に従い、`or` は guard ではなく pattern 自体として扱う。

```nepl
match x:
    or 1 2 println "one or two"
    _ println "other"
```

### 2.5 span パターン

range pattern は保留ではなく、`span` を用いた pattern として扱う。

```nepl
match c:
    span 'a' 'j' println "early ASCII letter"
    span 'k' 'z' println "late ASCII letter"
    _ println "other"
```

### 2.6 struct field pattern

struct 分解は位置ベースではなく field 名付きで書く。

```nepl
let Point x: a y: b p
```

```nepl
match p:
    Point x: a y: b println a
```

---

## 3. `let` による分解

`let` は単純束縛だけでなく分解にも使える。

```nepl
let Point x: a y: b p
```

このとき `Point` は struct pattern、`x:` と `y:` は対象 field 名、`a` と `b` は新しい束縛である。

型注釈が必要なら、右辺側の式に `%TypeExpr` を前置する。

```nepl
let Point x: a y: b %Point p_expr
```

---

## 4. `match`

### 4.1 構文

```
match <expr>:
    <pattern> <expr>
    <pattern> <expr>
```

arm は `:<suite>` ではなく `<pattern> <expr>` である。

### 4.2 例

```nepl
match x:
    or 1 2 println "one or two"
    3 println "three"
    _ println "anything"
```

```nepl
match c:
    span 'a' 'j' println "early ASCII letter"
    span 'k' 'z' println "late ASCII letter"
    _ println "something else"
```

### 4.3 網羅性

Zenn #2 に従い、`_` を書いた時点で網羅される。網羅されていない `match` はエラーとする。

---

## 5. guard について

旧案にあった `pattern if pred: expr` をコア構文とはしない。Zenn #2 では `or` や `span` を pattern として直接導入しているため、少なくとも現時点の基準文書では guard を中心に説明しない。

guard やより複雑な pattern 合成は将来拡張候補として別途検討する。

---

## 6. enum / constructor pattern

Zenn #2 は主に `or`、`span`、struct field pattern を明示している。enum variant pattern 自体は従来どおり利用できるものとして扱うが、詳細な bare variant 解決規則は [declarations.md](./declarations.md) に従う。

```nepl
match opt:
    Option::Some v v
    Option::None 0
```

---

## 7. 所有権との関係

pattern による分解が move か borrow かは、対象型の所有権規則に従う。ここで重要なのは表層構文であり、構文自体は field 名付き前置記法に統一する。

---

## 8. 旧案からの変更点

- OR は guard ではなく `or` pattern
- range は保留ではなく `span`
- match arm は `pattern: suite` ではなく `pattern expr`
- struct 分解は位置ベースではなく field 名付き
- 部分適用を前提にした guard 記述は採用しない
