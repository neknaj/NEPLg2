# NEPLg2.1 zero-argument function marker `void`

## 位置づけ

この文書は、NEPLg2.1 の関数型・関数定義・関数リテラルにおける 0 引数 marker を `unit` から `void` へ移す仕様と実装計画を定義する。

この変更は表層構文の整理であり、HIR、Resource IR、ownership、borrow、drop、monomorphize、codegen の意味論を変更しない。frontend は `void` marker を既存の空 parameter list へ正規化し、後続 phase へ `void` 専用 node を渡してはならない。

NEPLg2.1 の以前の checkpoint では、`unit` が次の 3 つを兼ねていた。

```text
1. unit 型
2. unit 値
3. 0 引数関数 marker
```

このうち 3 番目だけを `void` へ移す。変更後の役割は次の通りである。

```text
unit:
    unit 型
    unit 値

void:
    0 引数関数 marker
```

`void` は型ではなく値でもない。したがって `TypeExpr::Void`、HIR の void literal、Resource IR の void value、runtime void value は追加しない。

## 基本仕様

`void` は次の場所だけで使える。

```text
1. pure 関数型の第 1 引数位置
2. impure 関数型の第 1 引数位置
3. 関数定義・関数リテラルの lambda header
```

次の関数型は 0 引数で `unit` を返す pure 関数を表す。

```neplg2
%fn void unit
```

frontend 正規化後の意味は次である。

```text
Function {
    params = [],
    result = unit,
    effect = Pure,
}
```

次の関数型は `unit` 型の値を 1 個受け取り、`unit` を返す pure 関数を表す。

```neplg2
%fn unit unit
```

frontend 正規化後の意味は次である。

```text
Function {
    params = [unit],
    result = unit,
    effect = Pure,
}
```

0 引数関数定義は、型側に `fn void T`、lambda header 側に `\void` を書く。

```neplg2
fn main %fn void unit \void:
    unit
```

`\void` は仮引数名ではない。このため関数本体では `void` という名前を参照できない。

```neplg2
fn bad %fn void unit \void:
    void
```

上のコードは不正である。

## `unit` の扱い

`unit` は引き続き unit 型と unit 値を表す keyword である。

```neplg2
let marker %unit unit
```

`unit` 型の値を 1 個受け取る関数は、通常の仮引数名を使う。

```neplg2
fn id_unit %fn unit unit \a:
    a
```

0 引数関数から `unit` 引数関数を呼び出す場合は、実引数として `unit` 値を渡す。

```neplg2
fn main %fn void unit \void:
    id_unit unit
```

旧仕様の `\unit` は 0 引数 marker として受理しない。`unit` は予約語なので通常の仮引数名にも使えない。

## 関数型の区切り規則

NEPLg2.1 の関数型は前置形で書く。複数引数関数は、同じ effect を持つ隣接した関数型を既存の複数引数関数型へ正規化してよい。

```neplg2
%fn i32 fn i32 i32
```

正規化後:

```text
Function {
    params = [i32, i32],
    result = i32,
    effect = Pure,
}
```

ただし `void` は通常の引数型ではなく 0 引数 marker である。`void` を含む関数型では、`void` の位置で空引数関数として区切る。

```neplg2
%fn void fn unit unit
```

これは 0 引数で `fn unit unit` を返す関数である。

```neplg2
%fn unit fn void unit
```

これは `unit` 引数を 1 個取り、0 引数関数を返す関数である。

## 禁止事項

`void` は型ではないため、型注釈、返り値型、型引数として使えない。

```neplg2
let x %void unit
fn bad %fn void void \void:
    unit
let y %Option void none
```

`void` は値ではないため、式として使えない。

```neplg2
fn bad %fn void unit \void:
    void
```

`void` は通常の仮引数名ではないため、1 引数関数の parameter として使えない。

```neplg2
fn bad %fn unit unit \void:
    unit
```

## 旧構文からの移行

既存 NEPLg2.1 source に対して、0 引数 marker として使われていた `unit` は機械的に `void` へ移す。

```text
fn unit
    -> fn void

\unit
    -> \void
```

この置換は `unit` 型、`unit` 値、返り値型 `unit`、型引数 `unit` には適用しない。

旧コード:

```neplg2
fn main %fn unit unit \unit:
    sub unit
```

新コード:

```neplg2
fn main %fn void unit \void:
    sub unit
```

`sub unit` の `unit` は値なので残す。

## 診断方針

旧 0 引数 marker が残っている場合は、可能なら次の意味を含む診断を出す。

```text
`fn unit T` now denotes a function that takes one unit argument.
Use `fn void T` and `\void` for a zero-argument function.
```

`void` が型式や値式の位置に現れた場合は、次の意味を含む診断を出す。

```text
`void` is not a type or a value.
It is only allowed as a zero-argument function marker after `fn`, `impure fn`, or `\`.
```

## 実装計画

1. lexer に `VoidMarker` token と `void` keyword を追加する。
2. NEPLg2.1 prefix type parser で `fn void T` / `impure fn void T` を空 parameter list へ正規化する。
3. 通常 type expression parser では `void` を受理せず、`TypeExpr::Void` を追加しない。
4. lambda parser で `\void` を空 parameter list へ正規化し、`\unit` の 0 引数 marker 扱いを削除する。
5. `void` を予約語として `expect_ident` から拒否する。
6. `nodesrc/neplg21_syntax_migrate.js` を更新し、旧 0 引数 marker を `void` へ変換する。
7. 実行対象 corpus、stdlib doctest、tutorial、README、doc を新構文へ移行する。
8. positive / negative frontend tests を追加し、`fn void` と `fn unit` の意味差を固定する。

## 完了条件

この変更は次の条件をすべて満たしたら完了とする。

```text
1. `fn void T` が 0 引数関数型として parse される。
2. `\void` が 0 引数関数 marker として parse される。
3. `fn unit T` が `unit` 引数 1 個の関数型として parse される。
4. `\unit` が 0 引数 marker として受理されない。
5. `void` が型・値・返り値型・型引数として受理されない。
6. 既存 corpus の 0 引数関数が `fn void` / `\void` へ移行されている。
7. `unit` 値の実引数、`unit` 型注釈、返り値型 `unit` は保持されている。
8. `fn main %fn void unit \void:` が通る。
9. `fn sub %fn unit unit \a:` と `sub unit` が通る。
10. Resource IR 以降へ `void` 専用表現が追加されていない。
```
