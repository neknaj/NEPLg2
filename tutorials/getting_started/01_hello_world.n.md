# Hello World

NEPL のプログラムは、WASI で[動/うご]かすなら `#target wasi` を指定し、`#entry main` の `fn main` を[用意/ようい]します。

ここでは `std/stdio` の `println` を使って 1 行表示します。

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

## 何が起きているか（ざっくり）

- `#entry main`：この関数がプログラムの[入口/いりぐち]です。
- `#target wasi`：WASI として[実行/じっこう]できる形に[コンパイル/こんぱいる]します。
- `println`：文字列（`str`）を[改行/かいぎょう]つきで表示します。
