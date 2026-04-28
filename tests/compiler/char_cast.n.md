# char_cast

`char` 変数は暗黙には整数へ変換されないため、stdlib の UTF-8 decoder は `core/cast` の明示変換だけを使う。

## char_variable_casts_to_code_point

neplg2:test
ret: 65
```neplg2
#target std
#entry main
#indent 4

#import "core/cast" as *

fn main <()*>i32> ():
    let c <char> 'A'
    cast c
```

## checked_code_point_can_cast_to_char

neplg2:test
ret: 1
```neplg2
#target std
#entry main
#indent 4

#import "core/cast" as *

fn main <()*>i32> ():
    let c <char> cast 65
    match c:
        'A':
            1
        _:
            0
```
