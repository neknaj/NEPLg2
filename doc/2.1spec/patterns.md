# NEPLg2.1 パターン・Match・Let 仕様

最終更新: 2026-03-16

> 型記法は [types.md](./types.md) に従う。
> 中値演算子は `|>` のみ。パターンの `|`（OR パターン区切り）は式の `|>` とは別物。

---

## 1. Pair / Triple

言語組み込みの Tuple キーワードおよびリテラル構文はない。stdlib が通常の struct として `Pair .A .B` と `Triple .A .B .C` を提供する。

```nepl
// stdlib/core/pair.nepl
pub let Pair struct .A .B:
    fst %.A
    snd %.B

// stdlib/core/triple.nepl
pub let Triple struct .A .B .C:
    fst %.A
    snd %.B
    trd %.C
```

構築は他のコンストラクタ呼び出しと同じ前置記法:

```nepl
let p Pair 1 2
let t Triple 1 2 3
```

---

## 2. パターンの種類

パターンは `let` 文と `match` 式の両方で使用できる。コンストラクタパターンのアリティ（受け取るサブパターン数）は型推論によって静的に決定する。

### 2.1 識別子パターン

名前に値を束縛する。

```nepl
let x 42
let mut x 0    // 可変束縛（識別子パターンのみ mut 可）
```

### 2.2 ワイルドカードパターン

値を破棄する。束縛は生成しない。

```nepl
let _ expr
```

### 2.3 リテラルパターン

整数・文字列・bool リテラルの値に一致する。`match` アームで使用できる。

```nepl
match n:
    0: "zero"
    1: "one"
    _: "other"
```

### 2.4 範囲パターン

> **仕様保留**: 範囲パターンの具体的な構文は未確定。型前置記法・kind-directed 解析との整合性が確定してから追加する。現時点では `if`/`else if` チェーンで代替すること。
>
> ```nepl
> let grade
>     if ge score 90: "A"
>     else if ge score 70: "B"
>     else: "C"
> ```

### 2.5 コンストラクタパターン

struct または enum コンストラクタ名を前置し、フィールド数分のサブパターンを続ける（位置ベース）。

```nepl
let Pair a b p           // Pair を (a, b) に分解
let Triple a b c t
let Point x y pt

match opt:
    Option::Some v: v
    Option::None:   0

match res:
    Result::Ok val:   val
    Result::Err e:    handle_error e
```

### 2.6 ネストパターン

コンストラクタパターンのサブパターンに任意のパターンを置ける。アリティは型情報から静的に決定されるため、括弧を用いずに構造が決まる。

```nepl
let Pair Pair a b Pair c d nested_pair
// nested_pair : Pair (Pair .A .B) (Pair .C .D) を a, b, c, d に分解

match pair_of_pairs:
    Pair Pair a b Pair c d:
        add add a b add c d
```

型が確定できずアリティが不明な場合はコンパイルエラー。型注釈で解消すること。

### 2.7 束縛付きパターン（`@` パターン）

値全体を名前に束縛しつつ、サブパターンでさらに検査する。

```
<ident> @ <pattern>
```

> **注意**: `@ident` はすでに強制値モード（`forced_value`）で使用されている。パターン文脈での `<ident> @ <pattern>` と式文脈での `@ident` は構文上別物として扱う。

```nepl
match p:
    pair @ Pair a _:
        use_both pair a
```

### 2.8 OR パターン

`|` で複数パターンを結合する。全ての選択肢で束縛する変数の集合と型が一致しなければならない。

```nepl
match n:
    0 | 1 | 2: "small"
    _:         "large"

match opt:
    Option::Some 0 | Option::None: "empty or zero"
    Option::Some v:                v
```

### 2.9 参照パターン

borrowed 値を分解する。Resource IR 統合後に完全サポート。

```
& <pattern>
```

---

## 3. `let` 文でのパターン

### 3.1 構文

```
let <pattern> <expr>
let <pattern> %<TypeExpr> <expr>    // 型注釈は expr の前に付く
```

- `let` は文（評価値を持たない）。
- パターンは**網羅的**でなければならない。コンストラクタパターンが網羅的でない場合はコンパイルエラー。

### 3.2 例

```nepl
let x 42
let Pair a b Pair 1 2
let Triple a _ c t
let pair @ Pair a b Pair 1 2
let mut x 0

// 型注釈付き
let Pair a b %Pair i32 i32 some_expr
```

### 3.3 制約

- 非網羅的パターン（リテラルパターンや非全域バリアントパターン）は `let` 不可。
- 同一パターン内で同名を複数回束縛するとエラー。
- `mut` は識別子パターンのみに付与できる。

---

## 4. `match` 式でのパターン

### 4.1 構文

```
match <scrutinee> :
    <pattern> : <suite>
    <pattern> : <suite>
    ...
```

`match` は式。全アームの型が一致しなければならない。`<suite>` はインライン式またはインデントブロック（[syntax.md §4](./syntax.md) 参照）。

### 4.2 網羅性検査

コンパイラは全アームが scrutinee の型を網羅しているかを静的に検査する。

- enum はすべてのバリアントが覆われているか確認する。
- ワイルドカード `_` または識別子パターンがデフォルトアームとして使える。
- 網羅されていない場合はコンパイルエラー。

### 4.3 例

```nepl
// Option match
match opt:
    Option::Some v: v
    Option::None:   0

// ネストパターン
match result_pair:
    Pair Result::Ok a Result::Ok b:  Pair a b
    Pair Result::Err e _:            handle_error e
    Pair _ Result::Err e:            handle_error e

// OR パターン
match x:
    0 | 1: "zero or one"
    n:     n

// 束縛付き
match opt:
    original @ Option::Some v: use_both original v
    Option::None:               default_val
```

### 4.4 アームの効果

`match` 式全体の効果は全アーム本体の効果の上限（いずれか Impure ならば全体 Impure）。

### 4.5 ガード条件

現時点ではガード条件（`if` 節付きアーム）は仕様に含めない。将来追加する場合は本仕様を改訂する。

---

## 5. パターンと所有権

パターンによる分解は move を伴う。

| 型の種別 | パターン束縛の挙動 |
|---|---|
| `Copy` 型 | copy されて束縛される |
| `Owned` / `Linear` 型 | move されて束縛される（分解元は以後使用不可） |
| `_` で受けたフィールド | 直ちに drop される（`Drop` 能力があれば） |

```nepl
let Pair file rest p
// p は move 済み
// file : File → move された (Linear)
// rest → move された
```

OR パターンで束縛された変数の所有権種別は、全選択肢で一致しなければならない。

---

## 6. 名前解決とコンストラクタアリティ

コンストラクタパターンのアリティは型推論の結果から静的に決定する。アリティが確定できない場合はコンパイルエラー。型注釈で解消すること。

名前解決の優先順序:

1. `EnumName::VariantName` 形式 → enum バリアントとして解決
2. 型名と一致する struct コンストラクタ
3. 未解決の場合はエラー

---

## 7. 型記法との関係

- コンストラクタパターン `ConstructorName subpatterns...` は前置記法の型式と同形を維持する。
- 型注釈をパターン内に書く場合は `%TypeExpr` 形式を使う（`%Pair i32 str` など）。
- 位置ベース・アリティ駆動構造を採用しているため、型記法変更の影響を受けない。

---

## 8. フィールドアクセスとクロージャリテラル

パターンではなく式に属するが、パターンと密接に関連するため本仕様に記載する。詳細は [syntax.md](./syntax.md) §4.3 と §8 を参照。
