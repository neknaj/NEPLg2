# Hello World

`std/stdio` を使う最小プログラムです。`#entry main` で入口を指定し、`#target std` で標準入出力を使う target を選びます。

neplg2:test[stdio, normalize_newlines]
stdout: "Hello, NEPL!\n"
```neplg2
#entry main
#indent 4
#target std

#import "std/stdio" as *

// 標準出力へ 1 行だけ書きます。
fn main %impure fn () () \():
    println "Hello, NEPL!";
```

`println` は外部 I/O なので、`main` は `()*` の関数として書きます。戻り値が不要な program では `()` を返し、テストの終了 code を返したい program では `i32` を返します。

## 最初に固定するもの

- `#indent 4`: この tutorial の block indent は 4 spaces です。
- `#target std`: 入出力や `std/test` を使う章で指定します。
- `#entry main`: runner が呼ぶ関数を明示します。
