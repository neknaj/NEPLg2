# NEPLg2.1 コア構文仕様

最終更新: 2026-03-16

---

## 1. 基本原則

NEPLg2.1 の構文は次の原則を貫く。

- **前置記法（prefix notation）**: 関数適用は `f a b` の形。括弧によるグループ化は式文脈に存在しない。
- **オフサイドルール**: ブロックはインデントで表現する。閉じ括弧・閉じキーワードはない。
- **式指向**: `if`・`match`・`while`・ブロックはすべて値を返す式。
- **中値演算子は `|>` のみ**: 算術・比較演算子はすべて前置関数。

---

## 2. リテラル

```
<literal> :=
    <integer>    // 10進 整数リテラル: 0, 42, -1 等
  | <float>      // 浮動小数点: 3.14, -0.5 等
  | <bool>       // true, false
  | <string>     // ダブルクォート文字列: "hello"
  | unit         // unit 値（唯一の unit 型の値）
```

---

## 3. 識別子と型変数

- 識別子: `[a-zA-Z_][a-zA-Z0-9_]*`
- 型変数: `.Ident`（ドット始まり）。宣言文脈での型パラメータとして使う。

---

## 4. 式（Expression）

```
<expr> :=
    <literal>
  | <ident>
  | @ <ident>                    // 強制値モード（forced value）— 変数を強制的に値として扱う
  | <expr> <expr>                // 前置適用（juxtaposition）— 左結合
  | <expr> |> <expr>             // パイプ演算子（左結合）
  | <expr> . <field_name>        // フィールドアクセス（特殊形式、中値演算子ではない）
  | \ <params> : <expr>          // クロージャリテラル
  | if <expr> : <block> [else if <expr> : <block>]* [else : <block>]
  | match <expr> : <match_arms>
  | while <expr> : <block>       // ループ（unit を返す）
  | let <pattern> <expr>         // 式ブロック内の let 文（文として扱う）
  | set <ident> <expr>           // 可変束縛の更新（文として扱う）
  | <block>
```

### 4.1 前置適用（juxtaposition）

関数適用は引数を並べるだけ。引数の数（arity）は型情報から決定する。

```nepl
add 1 2           // add(1, 2)
mul add 1 2 3     // mul(add(1, 2), 3)  — 'add' が arity 2 なので '1 2' を消費
neg x             // neg(x)
```

括弧は存在しない。グループ化は型推論（arity）によって解決される。

### 4.2 パイプ演算子 `|>`

`|>` は NEPLg2.1 唯一の中値演算子。左結合。

```
a |> f       ≡   f a
a |> f |> g  ≡   g (f a)   // 左結合: (a |> f) |> g
```

型規則: `a : A`、`f : %fn A -> B` ならば `a |> f : B`。
効果規則: パイプ全体の effect は適用する関数の effect の上限（いずれか Impure ならば Impure）。

```nepl
// 例: 読み取り → パース → バリデーション
input |> trim |> parse_int |> validate

// 部分適用との組み合わせ
scores |> map \ x : mul x 2 |> filter \ x : gt x 0

// 定数インデックスのクランプ
n |> clamp 0 100
```

`|>` はパターン文脈には現れない。パターン内の `|` は OR パターン区切りであり別物。

### 4.3 フィールドアクセス `e.field`

struct のフィールドには `.` でアクセスする。これは中値演算子ではなく、パーサが特別扱いする特殊形式。

```nepl
let p Point 3 4
let x p.x          // Point の x フィールド
let y p.y

// ネストアクセス
let r Rect Point 0 0 Point 10 20
let tx r.top_left.x
```

フィールドアクセスは純粋（side-effect なし）。

```nepl
// |> との組み合わせ: p.x |> f は f p.x と等価
p.x |> to_float
```

フィールドアクセス vs パターン分解の使い分け:

| 操作 | 構文 | 適した場面 |
|------|------|-----------|
| 一部フィールドを参照 | `p.x` | 1〜2 フィールドを点在して使う |
| 複数フィールドを一度に束縛 | `let Point x y p` | 多数のフィールドを使う・move が必要 |

---

## 5. ブロック（Block）

ブロックは `:` の後にインデントして並べた文・式の列。最後の式がブロックの値になる。

```
<block> :=
    : <indent>
        <stmt_or_expr>*
        <expr>               // ブロックの値（最後の式）
```

```nepl
let result
    let a 10
    let b 20
    add a b        // ブロックの値 = 30
```

let 文はブロック内では文（値を持たない）として扱う。

---

## 6. if 式

```
if <cond> :
    <block>
[else if <cond> :
    <block>]*
[else :
    <block>]
```

`if` は式。全アームの型が一致しなければならない。`else` がない場合は `unit` を返す（全アームが `unit` の場合のみ省略可）。

```nepl
let grade
    if ge score 90: "A"
    else if ge score 70: "B"
    else: "C"

// unit を返す場合（else 省略可）
if is_debug:
    print_debug info
```

---

## 7. match 式

```
match <scrutinee> :
    <pattern> :
        <block>
    <pattern> :
        <block>
    ...
```

`match` は式。全アームの型が一致しなければならない。コンパイラは網羅性を静的に検査する。詳細は [patterns.md](./patterns.md) を参照。

```nepl
match opt:
    Option::Some v:
        v
    Option::None:
        0
```

---

## 8. while 式

```
while <cond> : <block>
```

`while` は式。**ブロック末尾式の型** `T` が `while` 式の型になる。最終イテレーションで評価されたブロックの値が `while` 式全体の値となる。条件は `bool` 型でなければならない。

```nepl
let mut i   1
let mut acc 0
while le i n:
    set acc add acc i
    set i   add i 1
    acc         // ← ブロック末尾式。最終ループでの acc が while 式の値になる
```

条件が最初から偽の場合（0 回実行）の値は型 `T` のデフォルト値または `unit` とする（`T = unit` の場合が最も単純）。条件が偽になりうる場合、型推論は 0 回実行時の値も考慮する必要がある。

> **仕様保留**: 0 回実行時の初期値の扱い（`Option T` で包む・`T` のデフォルト値・専用の `loop`/`while_result` 構文を別途設けるか等）は未確定。型安全性との整合を確認後に決定する。現時点では末尾 `unit` で使うのが安全。

副作用規則: ループ本体が `Impure` 操作を含む場合、`while` 式全体が `Impure`。

---

## 9. `set` 文（可変束縛の更新）

```
set <ident> <expr>
```

`let mut` で宣言された束縛を更新する。`set` は文（`unit` を返す）。

```nepl
let mut x 0
set x add x 1    // x を更新
```

純粋性条件（詳細は [effects.md §3.5](./effects.md) 参照）:

- 更新対象が unique local state であること
- その状態への参照が外へ escape しないこと
- 共有 borrow が存在しないこと
- 更新の結果が観測可能な raw identity を漏らさないこと

これらを満たす場合、`set` を含む関数は `Pure` のままで扱える。

```nepl
// Pure 関数の内部で set を使う例
let sum_to %fn i32 -> i32 \ n :
    let mut i   1
    let mut acc 0
    while le i n:
        set acc add acc i
        set i   add i 1
    acc    // acc は関数外に漏れない → Pure のまま
```

---

## 10. `@ident`（強制値モード）

式文脈で `@ident` と書くと、その識別子を強制的に値として扱う。パターン文脈での束縛付きパターン `ident @ pattern` とは別物（[patterns.md §2.7](./patterns.md) 参照）。

---

## 11. クロージャリテラル `\ params : body`

匿名関数は `\ params : body` で書く。

```
\ <param...> : <expr>
```

```nepl
\ x : mul x 2              // 引数 1 つ
\ a b : add a b            // 引数 2 つ
\ : 42                     // 引数なし

// 高階関数への渡し方
map xs \ x : mul x 2
filter xs \ x : gt x 0
fold_left xs 0 \ acc x : add acc x
```

型は呼び出し側の期待型から推論される。型を明示する場合は `let` を使う（[declarations.md](./declarations.md) 参照）。

キャプチャ規則:

| 変数の種別 | キャプチャの挙動 |
|-----------|----------------|
| `Copy` 型 | copy されてキャプチャ（元変数も使用可） |
| `Owned` 型 | move されてキャプチャ（クロージャ生成後は元変数使用不可） |
| `Linear` 型 | キャプチャ**不可**（コンパイルエラー） |

`Linear` 型（`File`、`Socket` など）はクロージャに渡せない。明示的な引数として渡すこと。

キャプチャ時点の値が固定される:

```nepl
let mut x 10
let f \ : x        // x の値（10）を copy してキャプチャ
let mut x 20       // 後からの再代入はクロージャのキャプチャに影響しない
f unit             // → 10（キャプチャ時点の値）
```

`Owned` 型の場合、クロージャ生成後は元変数が `Moved` 状態になるため再代入も使用も不可。

```nepl
let buf StringBuilder::new unit
let f \ : buf      // buf を move キャプチャ
// let _ buf       // ← コンパイルエラー: buf は Moved
```

---

## 12. let 文（ブロック内）

```
let <pattern> <expr>
let <pattern> %<TypeExpr> <expr>    // 型注釈付き
let mut <ident> <expr>              // 可変束縛
```

ブロック内の `let` は文（評価値を持たない）。`<pattern>` は網羅的でなければならない。詳細は [patterns.md](./patterns.md) を参照。

---

## 13. 演算子優先度・結合性

NEPLg2.1 の式文脈に現れる演算子は次の 2 種類のみ。優先度は下ほど高い。

| 優先度 | 演算子 | 結合性 | 備考 |
|--------|--------|--------|------|
| 低     | `\|>`  | 左結合 | 唯一の中値演算子 |
| 高     | juxtaposition（`f a`）| 左結合 | 関数適用。arity は型情報で決定 |

フィールドアクセス `.field` は演算子ではなく特殊形式。juxtaposition よりさらに強く結合する（パーサが先に解析する）。

```nepl
f a b |> g c    // 意味: g c (f a b)
                // 解析: (f a b) |> (g c) = g c (f a b)
p.x |> to_float // 意味: to_float p.x
                // 解析: (p.x) |> to_float = to_float p.x
```

算術・比較などすべての演算子は前置関数（`add`・`gt` など）であり、中値演算子ではない。

---

## 14. リテラル詳細

### 14.1 整数リテラル

10 進数のみ。負数は `-` を前置する（`neg` 関数との糖衣構文ではなくリテラル値として解析する）。

```
<integer> := [-] [0-9]+
```

### 11.2 浮動小数点リテラル

10 進浮動小数点。科学記法・`nan`・`inf` も対応する。

```
<float> := [-] [0-9]+ '.' [0-9]+
         | [-] [0-9]+ ('e' | 'E') [+-]? [0-9]+
         | [-] [0-9]+ '.' [0-9]+ ('e' | 'E') [+-]? [0-9]+
         | 'nan'
         | 'inf'
         | '-inf'
```

`nan` と `inf` は浮動小数点型のリテラルとしてのみ有効。

### 14.3 文字列リテラル

UTF-8 ダブルクォート文字列。以下のエスケープシーケンスを認識する。

| エスケープ | 意味 |
|-----------|------|
| `\\`      | バックスラッシュ |
| `\"`      | ダブルクォート |
| `\n`      | 改行（LF） |
| `\r`      | キャリッジリターン |
| `\t`      | タブ |
| `\0`      | NUL |
| `\u{XXXX}` | Unicode コードポイント（16 進 1〜6 桁） |

---

## 15. 構文上の注意

### 15.1 括弧なし

式文脈に括弧は存在しない。arity は型情報から決定されるため、括弧なしで構造が一意に定まる。

```nepl
// 正しい前置記法
add mul 2 3 4     // add(mul(2, 3), 4) = 10

// NEPLg2.1 に括弧構文はない
// add (mul 2 3) 4  ← このような括弧構文はない
```

### 15.2 `|>` は式文脈のみ

`|>` は式文脈でのみ有効。型式・パターン文脈には現れない。

### 15.3 `->` は型式文脈のみ

`->` は `%fn ... -> ...` の型式文脈でのみ有効。式の中値演算子ではない。

### 15.4 コメント

```nepl
// 行コメント
```
