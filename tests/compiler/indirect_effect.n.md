# indirect effect

## pure から impure function value を間接呼び出しできない

neplg2:test[compile_fail]
diag_code: effect.pure.calls_impure
```neplg2

#entry main
#indent 4
#target core

fn impure_id <(i32)*>i32> (x):
    x

fn call_callback <((i32)*>i32, i32)->i32> (callback, value):
    callback value

fn main <()->i32> ():
    call_callback @impure_id 1
```
