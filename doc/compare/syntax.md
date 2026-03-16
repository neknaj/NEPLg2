# NEPLg2.0 → NEPLg2.1 構文変更対照表

---

## 1. 宣言キーワードの統一

NEPLg2.1 ではすべての宣言を `let` で行う。

| 構文 | NEPLg2.0（旧） | NEPLg2.1（新） |
|------|----------------|----------------|
| 関数定義 | `fn name <TypeParams> <Sig> (params):` | `let name [type_params] %fn ... -> ... \ params :` |
| struct 定義 | `struct Name<.T>:` | `let Name struct .T:` |
| enum 定義 | `enum Name<.T>:` | `let Name enum .T:` |
| trait 定義 | `trait Name:` | `let Name trait:` |
| impl 定義 | `impl Trait for Type:` | `let Type impl for Trait:` |

### 具体例

```nepl
// NEPLg2.0
fn id <.T> <(.T)->.T> (x):
    x

fn add2 <(i32,i32)->i32> (a, b):
    ...

struct Pair<.A, .B>:
    fst <.A>
    snd <.B>

enum Result<.T, .E>:
    Ok <.T>
    Err <.E>

impl Eq for i32:
    fn eq <(i32, i32)->bool> (a, b):
        ...
```

```nepl
// NEPLg2.1
let id .T %fn .T -> .T \ x :
    x

let add2 %fn i32 i32 -> i32 \ a b :
    ...

let Pair struct .A .B:
    fst %.A
    snd %.B

let Result enum .T .E:
    Ok %.T
    Err %.E

let i32 impl for Eq:
    let eq %fn i32 i32 -> bool \ a b :
        ...
```

---

## 2. 型記法の変更

| 箇所 | NEPLg2.0（旧） | NEPLg2.1（新） |
|------|----------------|----------------|
| 型適用 | `Name<A, B>` | `Name A B`（juxtaposition） |
| 型パラメータ宣言 | `<.T, .U>` | `.T .U`（スペース区切り） |
| 関数型（純粋） | `(A, B) -> C` | `fn A B -> C` |
| 関数型（副作用） | `(A, B) *> C` | `fn* A B -> C` |
| unit 型 | `()` | `unit` |
| 型注釈 | `<TypeExpr>` | `%TypeExpr` |

### 具体例

| NEPLg2.0 | NEPLg2.1 |
|----------|----------|
| `Vec<i32>` | `Vec i32` |
| `Option<i32>` | `Option i32` |
| `Result<i32, str>` | `Result i32 str` |
| `Vec<Option<i32>>` | `Vec Option i32` |
| `Vec<Result<(), str>>` | `Vec Result unit str` |
| `(i32) -> i32` | `fn i32 -> i32` |
| `(i32, i32) -> i32` | `fn i32 i32 -> i32` |
| `() -> i32` | `fn unit -> i32` |
| `() *> i32` | `fn* unit -> i32` |
| `((i32)->i32) -> i32` | `fn fn i32 -> i32 -> i32` |
| `(Option<.T>, (.T)->.U)->Option<.U>` | `fn Option .T fn .T -> .U -> Option .U` |

---

## 3. 引数リスト記法の変更

| NEPLg2.0 | NEPLg2.1 |
|----------|----------|
| `(a, b):` | `\ a b :` |
| `():` | `\ :` |
| `(x):` | `\ x :` |

---

## 4. Tuple の廃止

言語組み込みの Tuple キーワードおよびリテラル構文を廃止。stdlib の `Pair .A .B` と `Triple .A .B .C` で代替する。

---

## 5. `fn` キーワードの使用範囲の変更

| NEPLg2.0 | NEPLg2.1 |
|----------|----------|
| 宣言キーワードとして使用 | 宣言キーワードとしての `fn` は廃止。`let` のみ使用 |
| 型注釈には使わなかった | 型式文脈（`%fn ...`）の関数型コンストラクタとしてのみ使用 |

---

## 6. `where` 節の追加

NEPLg2.0 には `where` 節はなかった。NEPLg2.1 では複数制約を型シグネチャと引数リストの間に記述する。

```nepl
// NEPLg2.1 での where 節
let merge .T .K .V %fn Vec .T Vec .T -> Vec .T
    where .T: Ord .K: Hash .V: Eq
    \ a b :
    ...
```

---

## 7. 効果記法の変更

| NEPLg2.0 | NEPLg2.1 |
|----------|----------|
| `->` で pure、`*>` で副作用 | `%fn ... -> ...` で pure |
| （型記法内の `->` と式文脈で混在） | `%fn* ... -> ...` で副作用 |

entry 関数の強制 Impure 特例は廃止。署名どおりに effect を判定する。

---

## 8. シャドウイング・オーバーロード

NEPLg2.0 にはなかった `noshadow` が追加される。

```nepl
noshadow let eq %fn i32 i32 -> bool \ a b :
    ...
```

---

---

## 9. `if` / `while` の補助マーカー廃止

NEPLg2.0 の parser には `cond` / `then` / `else` / `do` / `block` の補助マーカーが存在し、`if cond then A else B` または indent-based layout によって制御構造を表現していた。

NEPLg2.1 では `:` + インデント（または `:` + インライン式）に統一する。

| NEPLg2.0 | NEPLg2.1 |
|----------|----------|
| `if cond then ... else ...` | `if <cond> : <suite> [else : <suite>]` |
| `while cond do ...` | `while <cond> : <suite>` |
| `block: ...` | `<block>`（インデントブロック） |

`then`・`do`・`block` キーワードは廃止。

---

## 10. 括弧によるグループ化の廃止

NEPLg2.0 の AST には `Group`（括弧グループ）が存在し、`(expr)` 形式を解析できた。

NEPLg2.1 では**式文脈・型文脈ともに括弧によるグループ化構文は存在しない**。呼び出し境界はすべて型推論（arity / kind-directed 解析）で決定する。

| NEPLg2.0 | NEPLg2.1 |
|----------|----------|
| `add (mul 2 3) 4` | `add mul 2 3 4`（型推論で境界確定） |
| `(i32, str) -> bool` | `fn i32 str -> bool` |

---

## 11. セミコロンの廃止

NEPLg2.0 では一部の文脈でセミコロン `;` が区切り文字として使われていた。

NEPLg2.1 ではセミコロンは不要・無効。式の区切りはすべて改行とインデントによる。

---

## 12. バリアント参照記法の変更

NEPLg2.0 では enum バリアントはグローバル関数として登録され、bare 名で参照できた。

NEPLg2.1 では次の 2 形式が使える（[declarations.md §4.1](../2.1spec/declarations.md) 参照）:

| 形式 | 例 |
|------|----|
| 修飾形 | `Option::Some 10`、`Result::Ok val` |
| bare 形 | `Some 10`（期待型が確定している場合のみ） |

`::` はモジュール修飾ではなく enum 型名による修飾。型検査器が処理する。

**破壊的変更（移行注意）**: NEPLg2.0 で bare 名バリアントを期待型なしで使っていたコードは、NEPLg2.1 では型注釈を追加するか修飾形に書き換える必要がある。複数の enum で同名バリアントが衝突している場合はコンパイルエラーとなる。

---

## 13. 移行インパクト

1. **Parser 拡張**: `%` 記法・juxtaposition 型適用・`unit` を受け付けつつ、旧記法は移行期間中に警告付きで受け付ける。
2. **補助マーカー削除**: `then`・`do`・`block` キーワードを parser から除去。
3. **括弧グループ削除**: `Group` AST ノードと `LParen` のグループ解析を廃止。
4. **セミコロン廃止**: 文区切りセミコロンのパースを除去。
5. **stdlib 全体の書き換え**: すべての関数シグネチャ・struct/enum 定義を新形式に移行。
6. **チュートリアル・テストの更新**: ドキュメントとテストケースを新形式に更新。
7. **旧構文廃止**: 移行完了後に旧記法を削除。
