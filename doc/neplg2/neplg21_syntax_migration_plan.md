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
- unit 型・unit 値は `()` ではなく `unit` と書く。
- 0 引数関数 marker は `unit` ではなく `void` と書く。
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
impure fn void i32
```

`fn i32 fn i32 i32` は表記としてはカリー化されているが、NEPLg2.1 では部分適用を導入しない。frontend は既存の複数引数関数型へ正規化し、`add 1` のような unsaturated call を有効な関数値として扱わない。

```text
surface:
    %fn i32 fn i32 i32

frontend normalized:
    Function(params = [i32, i32], result = i32, effect = Pure)
```

関数を返す関数型は、戻り値側の関数型を括弧で明示する。括弧で grouping された戻り値関数型は複数引数関数へ flatten しない。

```text
surface:
    %fn bool (fn i32 fn i32 i32)

frontend normalized:
    Function(params = [bool], result = Function(params = [i32, i32], result = i32, effect = Pure), effect = Pure)
```

副作用を持つ関数型は NEPLg2.1 では `impure fn A B` を正とする。過去の draft や `doc/examples/` に残る `%fn*` は NEPLg2.1 の正規表記ではなく、移行対象である。

`unit` は NEPLg2.1 の unit 型・unit 値を表す keyword である。0 引数関数 marker は `void` を使う。`fn void T` は「0 引数で `T` を返す関数」として正規化し、`\void` は 0 引数の関数リテラルを表す。`fn unit T` は `unit` 型の引数を 1 個取る関数型である。この詳細は [zero_arg_void_marker_spec.md](./zero_arg_void_marker_spec.md) を正とする。

### 関数リテラル

```neplg2
let add_2 %fn i32 fn i32 i32 \arg1\arg2:
    add arg1 arg2

let now %impure fn void i32 \void:
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
4. unit 型・unit 値の `()` 表記を `unit` へ変換し、0 引数 marker は `void` へ分離する。
5. 実行対象 corpus の `<T>` 型注釈と型式を変換する。
6. generic postfix call を semantic rewrite で撤廃する。
7. README、doc index、tutorial、stdlib doc comment を NEPLg2.1 と NEPLg3 参考扱いに同期する。
8. selfhost 設計は NEPLg2.1 実装を踏まえて更新してから再開する。

### 2026-05-24 実装 checkpoint

- `%` 型注釈、prefix 型式、`\` lambda 引数は Rust frontend で受理し、既存 AST/HIR へ正規化している。
- `#extern` の `%...` signature は受理する。`#intrinsic` の型引数は compiler-owned directive syntax として `<...>` を維持する。
- `Stack<.T>.items : Vec<Option<.T>>` から `vec::free` を呼ぶ経路で、overload 候補選択が `Option<.T>: Copy` を generic impl pattern から認識できるようにした。
- 実行対象 corpus の `<TypeExpr>` 型注釈と function/lambda 外形は `nodesrc/neplg21_syntax_migrate.js` で大半を変換済みである。
- prefix 型適用境界は、現 checkpoint では parser-local arity hints を使っている。これは移行用の足場であり、kind resolver 化は `ISS-20260524T193635695Z-NEPLG2-1-PREFIX-TYPE-APPS-NEED-KIND-RESOLVER-A13F0C92` で扱う。
- explicit generic postfix はまだ正規構文として撤廃できていない。これは `ISS-20260524T085928138Z-NEPLG2-1-CORPUS-MIGRATION-NEEDS-SEMA-42A21754` の semantic rewrite として継続する。

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
