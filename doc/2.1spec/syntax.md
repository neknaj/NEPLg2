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
// すべての構文要素は式（値を持つ）
// let / set は unit を返す式として <expr> に含まれる
<expr> :=
    <literal>
  | <ident>
  | @ <ident>                    // 強制値モード（forced value）— 変数を強制的に値として扱う
  | <expr> <expr>                // 前置適用（juxtaposition）— flat chain; 境界は arity/型情報で決定
  | <expr> |> <expr>             // パイプ演算子（左結合）
  | <expr> . <field_name>        // フィールドアクセス（特殊形式、中値演算子ではない）
  | & <expr>                     // 共有 borrow 生成（shared borrow; 型 &T）
  | &mut <expr>                  // 可変 borrow 生成（unique borrow; 型 &mut T）
  | \ <params> : <suite>         // クロージャリテラル
  | if <expr> : <suite> [else if <expr> : <suite>]* [else : <suite>]
  | match <expr> : <match_arms>
  | while <expr> : <suite>       // ループ（Phase 0–7: unit を返す。Phase 8: 証明付きで T を返す）
  | let <pattern> <expr>         // 変数束縛（unit を返す式）
  | set <ident> <expr>           // 可変束縛の更新（unit を返す式）
  | <block>

// suite: インライン式またはインデントブロック（: の直後に置く本体）
<suite> :=
    <expr>                       // インライン式（: の直後、同一行）
  | <block>                      // インデントブロック（: の後に改行してインデント）
```

NEPLg2.1 は純粋な式指向言語である。「文（statement）」という独立した構文カテゴリは存在しない。`let` と `set` は `unit` を返す式として他の式と同等に扱われる。ブロック内での典型的な使い方はシーケンスの前置要素だが、型規則上は他と区別しない。

### 4.1 前置適用（juxtaposition）

関数適用は引数を並べるだけ。パーサは flat なトークン列として受理し、型チェッカーが各関数の arity と型情報を用いて呼び出し境界を決定する。BNF 上の「`<expr> <expr>`」は内部的に flat list として蓄積され、型情報で分割される（通常の意味での「左結合 AST を先に作る」のではない）。

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

ブロックは `:` の後に改行してインデントして並べた式の列。最後の式がブロックの値になる。

```
<block> :=
    <indent>
        <expr>*
        <expr>               // ブロックの値（最後の式）
```

ブロック内に並ぶ各式はその値が評価される。`let`・`set`・`while` のように `unit` を返す式は副作用（束縛の生成・更新）のために使う。`<suite>` はインライン式（同行）またはインデントブロック（改行後インデント）の両方を受け付ける（§4 の BNF 参照）。

```nepl
let result
    let a 10     // unit を返す式：a を束縛
    let b 20     // unit を返す式：b を束縛
    add a b      // i32 を返す式：ブロックの値 = 30
```

---

## 6. if 式

```
if <cond> : <suite>
[else if <cond> : <suite>]*
[else : <suite>]
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
    <pattern> : <suite>
    <pattern> : <suite>
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

### 8.1 基本形（Phase 0–7）

```
while <cond> : <suite>
```

Phase 0–7 の `while` は **`unit` を返す**。ループ本体の型も `unit` でなければならない（最後の文・式が `unit` でない場合はコンパイルエラー）。条件は `bool` 型でなければならない。

**設計理由**: 0 回実行時に「何を返すか」が型安全に解決できないため、Phase 0–7 では `while` は値を生成しない。値を積み上げるには `let mut` + `while` + ループ後の読み出しパターンを使う。

```nepl
// 標準パターン: let mut で積み上げ、while 後で読む
let mut i   1
let mut acc 0
while le i n:
    set acc add acc i
    set i   add i 1
// while は unit を返す; acc はループ後に読む
acc         // ← ブロックの値（while の外）
```

ループ本体の最後は `set`（`unit`）か `unit` を返す式でなければならない:

```nepl
// OK: set は unit
while cond:
    set x next x

// NG: 本体末尾が non-unit → コンパイルエラー
// while cond:
//     compute_value unit    // i32 を返す → 本体末尾が unit でない
```

副作用規則: ループ本体が `Impure` 操作を含む場合、`while` 式全体が `Impure`。

### 8.2 値返し形（Phase 8：証明付き while）

Phase 8 では、「少なくとも 1 回実行されること」の証明オブジェクトを渡すことで `while` がボディの型 `T` を返せる。

```
while <cond> <proof> : <suite>
```

- `proof : %WillExecute <cond>` — 条件が最初の評価で真であることを示す証明オブジェクト。
- この形の `while` は本体の最後の式の型 `T` を返す（0 回実行は証明により除外されているため型安全）。

```nepl
// [Phase 8 example]
// n > 0 の証明があるとき: i から n まで合計し、最終 acc を返す
let proof gt_proof n    // proof : WillExecute (le 1 n)
let result
    let mut i   1
    let mut acc 0
    while le i n proof:
        set acc add acc i
        set i   add i 1
        acc              // ← 証明によりボディ型 i32 が while の型になる
// result : i32
```

`WillExecute` は Phase 8 で stdlib が提供する命題型（詳細は [phase8.md](./phase8.md) 参照）。

---

## 9. `set` 式（可変束縛の更新）

```
set <ident> <expr>
```

`let mut` で宣言された束縛を更新する。`set` は `unit` を返す式。

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
// std::string_builder モジュールの bare 名 API を使用
use std::string_builder as *

let buf new unit   // new : %fn unit -> StringBuilder
let f \ : buf      // buf を move キャプチャ
// let _ buf       // ← コンパイルエラー: buf は Moved
```

---

## 12. let 式

```
let <pattern> <expr>
let <pattern> %<TypeExpr> <expr>    // 型注釈付き
let mut <ident> <expr>              // 可変束縛
```

`let` は `unit` を返す式。`<pattern>` は網羅的でなければならない。詳細は [patterns.md](./patterns.md) を参照。

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

### 14.2 浮動小数点リテラル

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

## 15. Borrow 生成と Dereference

### 15.1 Borrow 生成式

```
& <expr>         // 共有 borrow（型: &T）
&mut <expr>      // 可変 borrow（型: &mut T）
```

`&` と `&mut` は式文脈の前置特殊形式。juxtaposition（関数適用）より先にパーサが認識する。

```nepl
let b  &x          // b : &i32  — x の共有 borrow
let bm &mut x      // bm : &mut i32  — x の可変 borrow
```

型規則:
- `e : T` ならば `& e : &T`
- `e : T`（`e` が可変束縛）ならば `&mut e : &mut T`

borrow のライフタイム終端規則（NLL）は [effects.md §3.2.1](./effects.md) を参照。

### 15.2 Dereference

dereference は stdlib の `deref` 関数で行う（特殊構文はない）。

```nepl
let y deref b      // b : &i32 → y : i32
```

`deref` は通常の前置適用（juxtaposition）として解析される。

---

## 16. 構文上の注意

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
