# NEPLg3 宣言構文仕様

最終更新: 2026-03-27

---

## 1. 宣言の基本原則

NEPLg3 では、定義の基本形を `let <name> <expr>` に統一する。関数定義は「関数宣言」という別構文ではなく、lambda 式を束縛した `let` である。

```nepl
let zero 0
let inc \a add a 1
let mut counter 0
```

---

## 2. 関数定義

### 2.1 基本形

```nepl
let <name> <expr>
```

関数を定義する場合、`<expr>` に lambda を置く。

```nepl
let double \a add a a
let const \a \b a
```

### 2.2 lambda

lambda の基本形は次の 2 つである。

```nepl
\arg <expr>
\arg:
    <exprs>
```

0 引数は `\()` を使う。

```nepl
let main \() 0

let main \():
    let p Point x: 0 y: 7
    0
```

複数引数関数はカリー化して入れ子にする。

```nepl
let add2 \a \b add a b
```

### 2.3 部分適用

関数はカリー化して定義するが、部分適用を値として残すことはサポートしない。したがって、定義側では `\a \b ...` の形を使って 2 引数関数を構成し、呼び出し側では必要な引数をすべて与える。

---

## 3. let と mut

### 3.1 不変束縛

```nepl
let answer 42
```

### 3.2 可変束縛

```nepl
let mut counter 0
```

`let mut` で束縛できる式の制約や更新規則は [effects.md](./effects.md) を参照。

### 3.3 型注釈付き束縛

型注釈は `let` 専用構文ではなく、右辺の式に `%TypeExpr` を前置する。

```nepl
let x %i32 add 1 2
let p %Pair i32 bool Pair 1 true
```

---

## 4. struct 定義

Zenn #2 の表記に合わせ、struct 定義ではフィールドを `name: TypeExpr` で書く。

```nepl
let Point struct:
    x: i32
    y: i32

let Pair struct .A .B:
    first: .A
    second: .B
```

### 4.1 struct 値の構築

フィールド名付きの前置記法を用いる。

```nepl
let p Point x: 0 y: 7
```

---

## 5. enum 定義

enum も `let` に統一する。

```nepl
let Option enum .T:
    Some .T
    None

let Result enum .T .E:
    Ok .T
    Err .E
```

バリアント参照は、従来どおり修飾形 `Option::Some` を基本に説明する。bare 形の解決は型文脈に依存するため、曖昧になる場合は修飾形を要求する。

---

## 6. trait 定義

trait / impl / where などは Zenn #1 / #2 ではまだ全面確定していない。ここでは `let` に統一する原則だけを固定し、表層構文は前置型記法に合わせる。

```nepl
let Eq trait:
    let eq %fn Self fn Self bool \a \b ...
```

`trait` メソッドの型も `fn A B` 形式を使う。

---

## 7. impl 定義

```nepl
let i32 impl for Eq:
    let eq %fn i32 fn i32 bool \a \b i32_eq a b
```

詳細な coherence や where 節の扱いは [traits.md](./traits.md) を参照。

---

## 8. 可視性

可視性キーワード自体は従来どおり使えるが、宣言の核は常に `let` である。

```nepl
pub let zero 0
pub let Point struct:
    x: i32
    y: i32
```
