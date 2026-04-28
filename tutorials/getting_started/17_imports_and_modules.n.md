# import と module

`#import` は他の module の名前を使うための宣言です。短い tutorial では `as *` を使いますが、実コードでは alias を付けると由来が読みやすくなります。

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "core/math" as math
#import "core/result" as *
#import "std/test" as *

fn main <()*>i32> ():
    let checks:
        checks_new
        |> checks_push check_eq_i32 7 math::add 3 4
        |> checks_push check_eq_i32 12 math::mul 3 4
    checks_exit_code checks
```

`as *` は学習や小さな test では便利ですが、大きな module では alias を使って衝突を避けます。
