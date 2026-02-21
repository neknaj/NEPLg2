# tuple_old_syntax

旧タプル記法 `(a, b)` は廃止済みなので、コンパイルエラーになることを確認します。

## old_tuple_literal_call_is_rejected

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4
#target wasm

fn take <((i32,bool))->i32> (t):
    7

fn main <()->i32> ():
    take (1, true)
```

## old_tuple_literal_construct_is_rejected

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4
#target wasm

fn make <.A,.B> <(.A,.B)->(.A,.B)> (a,b):
    (a, b)

fn take_nested <(((i32,bool),i32))->i32> (t):
    9

fn main <()->i32> ():
    let t <(i32,bool)> make 3 true
    take_nested (t, 2)
```
