# [競/きょう]プロ[向/む]け I/O と[演算/えんざん]

この章は、競技プログラミングで最初に使う入出力パターンを、`std/streamio` の scanner / open だけで短く書く練習です。

## 2 整数を読んで和を出力する

neplg2:test[stdio, normalize_newlines]
stdin: "3 4\n"
stdout: "7\n"
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as *
#import "core/result" as *
#import "std/streamio" as *
#import "std/iotarget" as *

fn main <()*> ()> ():
    // scanner で読んだ値を即座に合成し、writer へ流します。
    let sc <StreamScanner> unwrap_ok open ReadStream::Stdio;
    let ans <i32> add read sc read sc;
    close sc;
    unwrap_ok open WriteStream::Stdio
    |> writeln ans
    |> flush
    |> close
```

## i64 を読んで加算する

`10^12` 以上を扱うときは i64 を使います。

neplg2:test[stdio, normalize_newlines]
stdin: "1000000000000 7\n"
stdout: "1000000000007\n"
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as *
#import "core/result" as *
#import "std/streamio" as *
#import "std/iotarget" as *

fn main <()*> ()> ():
    // 型だけ i64 に広げ、入出力の流れは同じ形で保ちます。
    let sc <StreamScanner> unwrap_ok open ReadStream::Stdio;
    let ans <i64> add read sc read sc;
    close sc;
    unwrap_ok open WriteStream::Stdio
    |> writeln ans
    |> flush
    |> close
```

## 3 値を 1 行で空白区切り出力する

`write w " "` を使うと、出力フォーマットを崩さずに書けます。

neplg2:test[stdio, normalize_newlines]
stdin: "5 8 13\n"
stdout: "5 8 13\n"
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/result" as *
#import "std/streamio" as *
#import "std/iotarget" as *

fn main <()*> ()> ():
    // 空白と改行を明示し、提出形式どおりの出力を固定します。
    let sc <StreamScanner> unwrap_ok open ReadStream::Stdio;
    let a <i32> read sc;
    let b <i32> read sc;
    let c <i32> read sc;
    close sc;
    unwrap_ok open WriteStream::Stdio
    |> write a
    |> write " "
    |> write b
    |> write " "
    |> writeln c
    |> flush
    |> close
```
