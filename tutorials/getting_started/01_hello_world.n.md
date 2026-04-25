# Hello World

NEPLg2 で実行可能な最小プログラムです。
標準ライブラリを使う最小プログラムでは `#target std` と `#entry main` を指定し、`fn main <()*> ()> ():` を定義します。

ここでは `std/stdio` の `println` で 1 行出力します。

neplg2:test[stdio, normalize_newlines]
stdout: "Hello, NEPL!\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/stdio" as *

// 標準出力へ 1 行だけ表示する最小構成です。
fn main <()*> ()> ():
    println "Hello, NEPL!";
```

`stdout`として示されている内容が、そのサンプルコードを実行したときに期待される結果です。
このサンプルコードでは、`Hello, NEPL!`が標準出力に表示されることが期待されます。

上に示されているサンプルコードの枠の中に、`▶ Run`と書かれたボタンがあります。
このボタンを押すと、ポップアップが開いて、そこで実行することができます。
サンプルコードを編集し、`Hello World!`など、他の文字列もプリントできることを確認してみてください。

## 最初につまずきやすい点

- `#target std` がないと、`std/stdio` の入出力が使えません。
- `#entry main` がないと、どの関数を実行するか決まりません。
- `#indent` の幅と実際のインデントがずれると、パースエラーになります。

まずはこの 3 つを固定してから、本文のロジックだけを編集する運用が安全です。
