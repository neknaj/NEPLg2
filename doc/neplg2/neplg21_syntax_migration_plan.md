# NEPLg2.1 surface syntax migration plan

## 位置づけ

この文書は、現行 Rust 実装の NEPLg2 を NEPLg2.1 の表層構文へ切り替えるための設計・実装計画である。

NEPLg2.1 は現行 `nepl-core/`、`stdlib/`、`tests/`、`tutorials/` をそのまま発展させる移行であり、NEPLg3 実装ではない。`doc/neplg3/` は今後も大きく変わり得る未確定の参考資料として扱う。NEPLg2.1 の実装中に、NEPLg3 文書を現在の正仕様として扱ってはならない。

2026-05-24 時点の開発方針は Zenn 記事「私のソフトウェアの設計指針: マルチプラットフォーム対応と静的検査の活用」に従う。試作段階で免除されるのは後方互換の保証であり、設計品質・静的検査・ドキュメント・issue 管理は免除されない。

## 目的

NEPLg2.1 では、型注釈、型式、関数リテラルを NEPL の前置記法に揃える。

- 型注釈は `<T>` ではなく `%T` と書く。
- 型適用は `Vec<i32>` ではなく `Vec i32` と書く。
- 関数型は `fn A B` の前置形で書き、複数引数は `fn A fn B C` と表示する。
- 関数リテラルの引数は `(a,b):` ではなく `\a\b:` と書く。
- 関数呼び出し側の明示 generic postfix `f<T>` は撤廃し、周辺の型注釈・引数・戻り値期待型から解決する。

これは表層構文の移行であり、Resource IR、ownership、borrow、drop、monomorphize、codegen の意味論を変えるものではない。frontend は NEPLg2.1 構文を既存の typed HIR へ正規化し、後続 phase へ構文差分を漏らさない。

## NEPLg2.1 構文

### 型注釈

```neplg2
let n %i32 40
let values %Vec i32 new |> add 1
%Result i32 str some_result
```

`%T expr` は「`expr` が `T` である」という期待型境界である。実行時の値や命令は生成しない。既存の `PrefixItem::TypeAnnotation` と `TypeExpectation::ExplicitAscription` に正規化する。

### 型式

NEPLg2.1 の型式は前置形で書く。

```text
i32
Vec i32
Result i32 str
&Vec i32
&mut Vec i32
fn i32 i32
fn i32 fn i32 i32
impure fn () i32
```

`fn i32 fn i32 i32` は表記としてはカリー化されているが、NEPLg2.1 では部分適用を導入しない。frontend は既存の複数引数関数型へ正規化し、`add 1` のような unsaturated call を有効な関数値として扱わない。

```text
surface:
    %fn i32 fn i32 i32

frontend normalized:
    Function(params = [i32, i32], result = i32, effect = Pure)
```

副作用を持つ関数型は NEPLg2.1 では `impure fn A B` を正とする。過去の draft や `doc/examples/` に残る `%fn*` は NEPLg2.1 の正規表記ではなく、移行対象である。

### 関数リテラル

```neplg2
let add_2 %fn i32 fn i32 i32 \arg1\arg2:
    add arg1 arg2

let now %impure fn () i32 \():
    clock_now_i32
```

関数リテラルは、当面は既存 parser と同じく private `FnDef` と関数値式へ desugar する。NEPLg2.1 では記法だけを変え、capture semantics や closure 表現を新しく導入しない。

### `fn` 宣言

NEPLg2.1 でも `fn name` は `let name` の関数定義向け糖衣構文として残す。

```neplg2
fn add_2 %fn i32 fn i32 i32 \arg1\arg2:
    add arg1 arg2
```

この方針は現行 NEPLg2 self-host 計画との整合性を優先する。将来の NEPLg3 が `let` のみに統一するかどうかは、この移行の正仕様ではない。

### 明示 generic postfix の撤廃

呼び出し側の `f<T>` / `Module::f<T,U>` / `Variant<T>` は NEPLg2.1 の正規構文ではない。

旧:

```neplg2
let a <Option<i32>> some<i32> 7
unwrap_ok<Vec<i32>, Diag> new<i32>
```

新:

```neplg2
let a %Option i32 some 7
%Vec i32 unwrap_ok new
```

単純な削除で意味が保てない場合は、周辺へ `%T` 注釈を追加する。これは regex 置換ではなく、呼び出し先 signature、期待戻り値、引数型、trait bound を見た semantic rewrite として扱う。

## frontend 境界

NEPLg2.1 移行で後続 phase を変えないため、frontend は次の境界を守る。

- lexer/parser は `%`、`\`、prefix 型式を受け付ける。
- parser/type frontend は NEPLg2.1 型式を既存 `TypeExpr` へ正規化する。
- 関数リテラルは既存 `FnDef` desugar を使う。
- call postfix generic は通常 source では禁止または移行診断にする。
- intrinsic や compiler-owned primitive の内部型引数処理は、source syntax とは分けて維持する。
- Resource IR へ NEPLg2.1 表層構文専用の node を持ち込まない。

## 移行単位

1. lexer/parser に `%` と `\` を追加する。
2. `%T expr` と `\a\b:` を既存 AST/HIR へ正規化する。
3. prefix 型式を実装し、関数型 chain を既存複数引数型へ flatten する。
4. 実行対象 corpus の `<T>` 型注釈と型式を変換する。
5. generic postfix call を semantic rewrite で撤廃する。
6. README、doc index、tutorial、stdlib doc comment を NEPLg2.1 と NEPLg3 参考扱いに同期する。
7. selfhost 設計は NEPLg2.1 実装を踏まえて更新してから再開する。

## 検証

- `cargo test -p nepl-core --test typeannot`
- `cargo test -p nepl-core --test generics`
- `cargo test -p nepl-core --test functions`
- `cargo test -p nepl-core --test overload`
- `node nodesrc/issues.js check --dir issues`
- `git diff --check`
- `trunk build`
- `node nodesrc/cli.js -i tests/playground_editor --playground-editor-tests -o json=tmp/playground-editor-neplg21-20260524.json`

大規模移行の途中で一時的に旧構文 fixture が落ちることはあり得る。ただし落ちている理由が NEPLg2.1 への正しい移行途中なのか、frontend 正規化や semantic rewrite の誤りなのかは個別に確認する。
