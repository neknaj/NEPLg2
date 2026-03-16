# NEPLg2.1 Pattern / Match / Let 仕様

最終更新: 2026-03-16

> 型記法は `doc/type_notation_spec.md`（NEPLg2.1 型記法仕様）に従う。
>
> NEPLg2 では中値記法はパイプ `|>` のみ存在する。式の文脈での中値演算子はない。
> パターンの `|`（OR パターン区切り）はパターン専用の構文であり、式の中値演算子とは別物である。

---

## 1. 前提：Tuple 廃止と Pair / Triple

言語組み込みの `Tuple` キーワードおよび `Tuple:` リテラル構文は廃止する。
代わりに、stdlib が通常の struct として `Pair<.A, .B>` と `Triple<.A, .B, .C>` を提供する。
言語に特別な構文はなく、他の struct と同様に扱う。

```nepl
// stdlib/core/pair.nepl
pub struct Pair<.A, .B>:
    fst .A
    snd .B

// stdlib/core/triple.nepl
pub struct Triple<.A, .B, .C>:
    fst .A
    snd .B
    trd .C
```

構築は他のコンストラクタ呼び出しと同じ前置記法：

```nepl
let p Pair 1 2         // Pair<i32, i32>
let t Triple 1 2 3     // Triple<i32, i32, i32>
```

---

## 2. パターンの種類

パターンは `let` 文と `match` 式の両方で使用できる。
コンストラクタパターンのアリティ（受け取るサブパターン数）は型推論によって静的に決定する。

### 2.1 識別子パターン (Identifier Pattern)

名前に値を束縛する。

```
<ident>
```

```nepl
let x 42
```

`mut` を付けると可変束縛になる（識別子パターンのみ）：

```nepl
let mut x 0
```

### 2.2 ワイルドカードパターン (Wildcard Pattern)

値を破棄する。束縛は生成しない。

```
_
```

```nepl
let _ expr
```

### 2.3 リテラルパターン (Literal Pattern)

整数・文字列・bool リテラルの値に一致する。`match` アームで使用できる。

```
<literal>
```

```nepl
match n:
    0:
        "zero"
    1:
        "one"
    _:
        "other"
```

### 2.4 範囲パターン (Range Pattern)

数値の連続範囲に一致する。`match` アームで使用できる。

範囲パターンの具体的な構文は、型前置記法の仕様確定と合わせて設計する（現時点では未確定）。
以下はプレースホルダ形式で概念を示す。

```
range_incl <start> <end>     // 閉区間 [start, end]
range <start> <end>          // 半開区間 [start, end)
```

```nepl
match score:
    range_incl 90 100:
        "A"
    range_incl 70 89:
        "B"
    _:
        "C"
```

> **注意**: 上記の `range_incl` / `range` はパターン専用の構文形式であり、最終構文は別途確定する。

### 2.5 コンストラクタパターン (Constructor Pattern)

struct または enum コンストラクタ名を前置し、フィールド数分のサブパターンを続ける（位置ベース）。
フィールド数はコンストラクタの定義から型推論によって静的に決定する。

```
<ConstructorName> <pattern...>
<EnumName>::<VariantName> <pattern...>
```

```nepl
let Pair a b p           // Pair を (a, b) に分解
let Triple a b c t       // Triple を (a, b, c) に分解
let Point x y pt         // struct Point を (x, y) に分解

match opt:
    Option::Some v:
        v
    Option::None:
        0

match res:
    Result::Ok val:
        val
    Result::Err e:
        handle_error e
```

### 2.6 ネストパターン (Nested Pattern)

コンストラクタパターンのサブパターンに任意のパターンを置ける。
アリティは型情報から静的に決定されるため、括弧を用いずに構造が決まる。

```nepl
let Pair Pair a b Pair c d nested_pair
// nested_pair : Pair<Pair<A,B>, Pair<C,D>> を a, b, c, d に分解

match pair_of_pairs:
    Pair Pair a b Pair c d:
        add add a b add c d   // add (add a b) (add c d) の前置形
```

型が確定できずアリティが不明な場合はコンパイルエラーとなる。型注釈で解消すること。

### 2.7 束縛付きパターン (`@` パターン)

値全体を名前に束縛しつつ、サブパターンでさらに検査する。

```
<ident> @ <pattern>
```

> **注意**: `@ident` はすでに強制値モード（`forced_value`）で使用されている。
> パターン文脈での `<ident> @ <pattern>` と式文脈での `@ident` は構文上別物として扱う。
> 衝突が生じる場合は別の記号への変更を検討する。

```nepl
match p:
    pair @ Pair a _:
        // a は fst、pair は Pair 全体
        use_both pair a
```

### 2.8 OR パターン (Or Pattern)

`|` で複数パターンを結合する。どれか 1 つに一致すればよい。
`|` はパターン専用の区切り構文であり、式の中値演算子とは別物である。
全ての選択肢で束縛する変数の集合と型が一致しなければならない。

```
<pattern> | <pattern> | ...
```

```nepl
match n:
    0 | 1 | 2:
        "small"
    _:
        "large"

match opt:
    Option::Some 0 | Option::None:
        "empty or zero"
    Option::Some v:
        v
```

### 2.9 参照パターン (Reference Pattern)

borrowed 値を分解する。将来: borrow checker（Resource IR）統合後に完全サポート。

```
& <pattern>
```

現時点では型システム上の対応のみ記述し、実装は Resource IR 導入後に完全対応する。

---

## 3. `let` 文でのパターン

### 3.1 構文

```
let <pattern> <expr>
let <pattern> <type_annotation> <expr>    // 型注釈は expr の前に付く
```

- `let` は文（評価値を持たない）。
- `<expr>` を評価し、`<pattern>` で分解してスコープに束縛する。
- パターンは**網羅的**でなければならない。コンストラクタパターンが網羅的でない場合（特定バリアントのみ等）は `let` で使用不可。

### 3.2 例

```nepl
let x 42
let Pair a b Pair 1 2
let Triple a _ c t              // t の中間フィールドを無視
let pair @ Pair a b Pair 1 2    // 束縛 + 分解
let mut x 0

// 型注釈付き
let Pair a b <Pair<i32,i32>> some_expr
```

### 3.3 制約

- パターンが非網羅的な場合はコンパイルエラー（リテラルパターンや非全域バリアントパターンは `let` 不可）。
- 同一パターン内で同名を複数回束縛するとエラー。
- `mut` は識別子パターンのみに付与できる。

---

## 4. `match` 式でのパターン

### 4.1 構文

```
match <scrutinee>:
    <pattern>:
        <block>
    <pattern>:
        <block>
    ...
```

`match` は式であり、全アームのブロックの型が一致しなければならない。

### 4.2 網羅性検査

コンパイラは全アームが scrutinee の型を網羅しているかを静的に検査する。
- enum はすべてのバリアントが覆われているか確認する。
- ワイルドカード `_` または識別子パターンがデフォルトアームとして使える。
- 網羅されていない場合はコンパイルエラー。

### 4.3 例

```nepl
// Option match
match opt:
    Option::Some v:
        v
    Option::None:
        0

// リテラル match
match n:
    0:
        "zero"
    1:
        "one"
    _:
        "other"

// ネストパターン
match result_pair:
    Pair Result::Ok a Result::Ok b:
        Pair a b
    Pair Result::Err e _:
        handle_error e
    Pair _ Result::Err e:
        handle_error e

// OR パターン
match x:
    0 | 1:
        "zero or one"
    n:
        n

// 束縛付き
match opt:
    original @ Option::Some v:
        use_both original v
    Option::None:
        default_val
```

### 4.4 アームの効果

`match` 式全体の効果は全アーム本体の効果の上限となる（いずれか Impure ならば全体 Impure）。

### 4.5 ガード条件（将来拡張）

現時点ではガード条件（`if` 節付きアーム）は仕様に含めない。将来追加する場合は本仕様を改訂する。

---

## 5. パターンと所有権

パターンによる分解は move を伴う。

| 型の種別 | パターン束縛の挙動 |
|---|---|
| `Copy` 型 | copy されて束縛される |
| `Owned` / `Linear` 型 | move されて束縛される（分解元は move 済み、以後使用不可） |
| `_` で受けたフィールド | 直ちに drop される（`Drop` 能力があれば） |

```nepl
let Pair file rest p
// p は move 済み
// file : File → move された (Owned)
// rest → move された
```

OR パターンで束縛された変数の所有権種別は、全選択肢で一致しなければならない。

---

## 6. 名前解決とコンストラクタアリティ

コンストラクタパターンのアリティは型推論の結果から静的に決定する。
アリティが確定できない場合はコンパイルエラー。型注釈で解消すること。

名前解決の優先順序：
1. `EnumName::VariantName` 形式 → enum バリアントとして解決
2. 型名と一致する struct コンストラクタ
3. 未解決の場合はエラー

---

## 7. 型前置記法への対応方針

型前置記法仕様は別途確定予定。本仕様のパターン構文は以下を満たすよう設計する：

- コンストラクタパターン `ConstructorName subpatterns...` は前置記法の型式と同形を維持する。
- 型注釈をパターン内に書く場合の構文は前置記法仕様に従う（現時点では `<T>` アノテーション形式）。
- 将来の型前置記法対応時にパターン構文の実質的な変更が不要なよう、位置ベースのアリティ駆動構造を採用している。
