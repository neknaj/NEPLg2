# 名前空間と `::` 呼び出し

NEPLg2 では、関数や enum バリアントを `名前空間::識別子` で参照できます。
`#import "... " as alias` を使うと、`alias::name` で明示的に呼べます。

## alias 経由で関数を呼ぶ

`m::add` のように書くと、「どのモジュールの関数か」をコード上で明確にできます。

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/math" as m
#import "std/test" as *

fn main <()*>()> ():
    assert_eq_i32 9 m::add 4 5
    assert_eq_i32 6 m::mul 2 3
    test_checked "namespace function call"
```

## enum バリアントも `::` で参照する

`Option::Some` / `Option::None` のように、型の名前空間でバリアントを指定します。

neplg2:test
```neplg2
| #entry main
| #indent 4
| #target wasi
|
#import "core/option" as *
#import "std/test" as *

fn unwrap_or_zero <(Option<i32>)->i32> (v):
    match v:
        Option::Some x:
            x
        Option::None:
            0
|
fn main <()*>()> ():
    let v1 <Option<i32>> Option::Some 12
    let v2 <Option<i32>> Option::None
    assert_eq_i32 12 unwrap_or_zero v1
    assert_eq_i32 0 unwrap_or_zero v2
    test_checked "enum variant path"
```

## 使い分けの目安

- import を `as *` にすると短く書けますが、識別子の衝突に注意します。
- alias を使うと少し長くなる代わりに、参照元が明確になります。
- 大きめのコードでは、衝突しやすい名前を alias 経由に寄せると読みやすくなります。
