# Option（値がある / ない）

**Option** (`/ˈɑːpʃən/`, 選択肢; [オプション]; ラテン語: optio, `/ˈɔp.ti.oː/`, 選ぶこと) は「値が **ある**（Some）か、**ない**（None）か」を表す型です。

NEPL では `core/option` に Option と基本操作が入っています。

## Some / None と match

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
| #import "core/option" as *
| #import "std/test" as *
|
fn main <()*> ()> ():
    let a <Option<i32>> some<i32> 10
    let b <Option<i32>> none<i32>

    assert is_some<i32> a
    assert is_none<i32> b

    match a:
        Option::Some v:
            assert_eq_i32 10 v
        Option::None:
            test_fail "a was None"

    match b:
        Option::Some v:
            test_fail "b was Some"
        Option::None:
            ()
    test_checked "option"
```

## unwrap は注意

`unwrap` は None だと `unreachable` になり、プログラムが落ちます。
None の可能性があるなら `option_unwrap_or` などを使います。

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
| #import "core/option" as *
| #import "std/test" as *
|
fn main <()*> ()> ():
    let b <Option<i32>> none<i32>
    assert_eq_i32 123 option_unwrap_or<i32> b 123
    test_checked "unwrap_or"
```
