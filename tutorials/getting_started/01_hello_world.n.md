# Hello World

NEPLg2 で実行可能な最小プログラムです。
WASI ターゲットでは `#target wasi` と `#entry main` を指定し、`fn main <()*> ()> ():` を定義します。

ここでは `std/stdio` の `println` で 1 行出力します。

neplg2:test[stdio, normalize_newlines]
stdout: "Hello, NEPL!\n"
```neplg2
// 諸々を設定します
#entry main
#indent 4
#target wasi

// stdioをインポートします。
#import "std/stdio" as *

fn main <()*> ()> ():
    println "Hello, NEPL!";
```