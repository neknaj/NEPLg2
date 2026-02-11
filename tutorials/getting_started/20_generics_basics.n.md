# ジェネリクスの[基本/きほん]

ジェネリクスは「型を後で決める関数・型」を定義する仕組みです。
NEPLg2 では型パラメータを `<.T>` のように書きます。

## 汎用関数 `id`

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "std/test" as *

fn id <.T> <(.T)->.T> (x):
    x
|
fn main <()*>()> ():
    assert_eq_i32 42 id 42
    assert_str_eq "nepl" id "nepl"
    test_checked "generic id"
```

## ジェネリックな enum を扱う

`Option<.T>` のように、enum 側にも型パラメータを持たせられます。

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/option" as *
#import "std/test" as *

fn keep_or_default <.T> <(Option<.T>,.T)->.T> (opt, default):
    match opt:
        Option::Some v:
            v
        Option::None:
            default
|
fn main <()*>()> ():
    let a <Option<i32>> Option::Some 7
    let b <Option<i32>> Option::None
    assert_eq_i32 7 keep_or_default a 0
    assert_eq_i32 9 keep_or_default b 9
    test_checked "generic option"
```

## 補足

- `<.T, .U>` のように複数型パラメータも指定できます。
- 具体的な型注釈は必要な箇所だけに絞ると読みやすくなります。
