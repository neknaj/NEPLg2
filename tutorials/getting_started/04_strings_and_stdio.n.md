# 文字列と標準入出力

NEPL の文字列は `str` です。
結合には `concat` を使います。

`std/stdio` を import すると `print` / `println` / `read_line` が使えます。

## 文字列の結合（concat）

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
| #import "std/test" as *
| #import "core/mem" as *
|
fn main <()*> ()> ():
    let a <str> "Hello"
    let b <str> "World"
    let c <str> concat a ", "
    let d <str> concat c b
    assert_str_eq "Hello, World" d
    test_checked "concat"
```

## 入力を読む（read_line）→表示する（println）

ここでは stdin から 1 行読み、同じ内容を表示します。

neplg2:test[stdio, normalize_newlines]
stdin: "abc\n"
stdout: "abc\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
| #import "std/stdio" as *
|
fn main <()*> ()> ():
    let s <str> read_line
    println s
```
