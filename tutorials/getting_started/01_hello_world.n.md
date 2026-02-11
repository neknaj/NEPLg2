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

`stdout`として示されている内容が、そのサンプルコードを実行したときに期待される結果です。
このサンプルコードでは、`Hello, NEPL!`が標準出力に表示されることが期待されます。

上に示されているサンプルコードの枠の中に、`▶ Run`と書かれたボタンがあります。
このボタンを押すと、ポップアップが開いて、そこで実行することができます。
サンプルコードを編集し、`Hello World!`など、他の文字列もプリントできることを確認してみてください。

## 最初につまずきやすい点

- `#target wasi` がないと、`std/stdio` の入出力が使えません。
- `#entry main` がないと、どの関数を実行するか決まりません。
- `#indent` の幅と実際のインデントがずれると、パースエラーになります。

まずはこの 3 つを固定してから、本文のロジックだけを編集する運用が安全です。
