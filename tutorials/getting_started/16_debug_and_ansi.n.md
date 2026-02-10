# デバッグ出力と ANSI カラー

NEPLg2 の `std/stdio` には、通常出力に加えて ANSI カラー出力を補助する関数があります。
デバッグ時に重要な値を色で強調すると、ログの確認が速くなります。

## `print_color` / `println_color` の基本

neplg2:test[stdio, normalize_newlines]
stdout: "\u001b[31mERR\u001b[0m \u001b[32mOK\u001b[0m\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
| #import "std/stdio" as *
|
fn main <()*> ()> ():
    print_color ansi_red "ERR";
    print " ";
    println_color ansi_green "OK";
```

## `std/test` と組み合わせる

neplg2:test[stdio, normalize_newlines, strip_ansi]
stdout: "Checked color-ready\n"
```neplg2
| #entry main
| #indent 4
| #target wasi
|
| #import "std/test" as *
|
fn main <()*> ()> ():
    test_checked "color-ready";
```

## 注意点

- ANSI コードは端末依存です。対応していない環境ではエスケープ文字列として見えることがあります。
- テストでは `strip_ansi` を使うと、色の有無に依存せず安定して比較できます。
