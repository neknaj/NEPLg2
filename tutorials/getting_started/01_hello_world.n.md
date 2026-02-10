# Hello World

NEPLg2 で実行可能な最小プログラムです。
WASI ターゲットでは `#target wasi` と `#entry main` を指定し、`fn main <()*> ()> ():` を定義します。

ここでは `std/stdio` の `println` で 1 行出力します。

neplg2:test[stdio, normalize_newlines]
stdout: "Hello, NEPL!\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
| #import "std/stdio" as *
|
fn main <()*> ()> ():
    println "Hello, NEPL!";
```

## 何が起きているか

- `#entry main`: 実行開始点を指定します。
- `#target wasi`: WASI 向けコード生成を選びます。
- `println`: `str` を改行付きで出力します。
