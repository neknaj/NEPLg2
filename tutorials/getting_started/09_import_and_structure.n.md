# import と小さな分割

この章では、`#import` を使って機能を取り込み、関数を小さく分けて読みやすくする基本を学びます。

NEPLg2 では巨大な1関数よりも、目的ごとに関数を分離したほうが型や挙動を追いやすくなります。

## `#import` と関数分割

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "std/test" as *

fn twice <(i32)->i32> (x):
    mul x 2

fn add_one <(i32)->i32> (x):
    add x 1

fn pipeline_like <(i32)->i32> (x):
    add_one twice x

fn main <()*> ()> ():
    assert_eq_i32 9 pipeline_like 4
    test_checked "import and split functions"
```

## 標準I/Oと組み合わせる

neplg2:test[stdio, normalize_newlines]
stdout: "42\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as *
#import "std/stdio" as *

fn calc <()->i32> ():
    add 40 2

fn main <()*> ()> ():
    println_i32 calc
```
