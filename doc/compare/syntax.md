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

## 9. 移行インパクト

1. **Parser 拡張**: `%` 記法・juxtaposition 型適用・`unit` を受け付けつつ、旧記法は移行期間中に警告付きで受け付ける。
2. **stdlib 全体の書き換え**: すべての関数シグネチャ・struct/enum 定義を新形式に移行。
3. **チュートリアル・テストの更新**: ドキュメントとテストケースを新形式に更新。
4. **旧構文廃止**: 移行完了後に旧記法を削除。
