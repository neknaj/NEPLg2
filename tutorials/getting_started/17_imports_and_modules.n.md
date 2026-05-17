# import と module

`#import` は他の module の名前を使うための宣言です。短い tutorial では `as *` を使いますが、実コードでは alias を付けると由来が読みやすくなります。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok]
    ##: [0] ok
    ##: [1] ok
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as math
#import "core/result" as *
#import "std/test" as *
#import "core/math" as *

fn main <()*>i32> ():
    let checks:
        checks_new
        |> checks_push assert_eq_i32 7 math::add 3 4
        |> checks_push assert_eq_i32 12 math::mul 3 4
    let shown checks_print_report checks
    checks_exit_code shown
```

`as *` は学習や小さな test では便利ですが、大きな module では alias を使って衝突を避けます。
