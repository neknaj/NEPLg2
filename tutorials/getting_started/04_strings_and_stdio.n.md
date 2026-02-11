# [文字列/もじれつ]と[標準/ひょうじゅん][入出力/にゅうしゅつりょく]

文字列型は `str` です。
連結は `concat`、入出力は `std/stdio` の `print` / `println` / `read_line` を使います。

## 文字列の結合（concat）

neplg2:test[stdio, normalize_newlines]
stdout: "Hello, World\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/mem" as *
#import "alloc/string" as *
#import "std/stdio" as *

fn main <()*> ()> ():
    let hello <str> "Hello"
    let world <str> "World"
    let left <str> concat hello ", "
    let line <str> concat left world
    println line
```

## `print` と `println`

neplg2:test[stdio, normalize_newlines]
stdout: "A=10\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "std/stdio" as *

fn main <()*> ()> ():
    print "A="
    println_i32 10
```

## 入力を読む（`read_line`）→表示する（`println`）

ここでは stdin から 1 行読み、同じ内容を表示します。

neplg2:test[stdio, normalize_newlines]
stdin: "abc\n"
stdout: "abc\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "std/stdio" as *

fn main <()*> ()> ():
    let s <str> read_line
    println s
```

## 入出力の整理ポイント

- 単発の行入力は `read_line` が簡単です。
- 多数の整数入力を処理する場面では `kp/kpread` に切り替えると実装が安定します。
- 出力は `print`（改行なし）と `println`（改行あり）を用途で分けると、フォーマット崩れを防げます。
