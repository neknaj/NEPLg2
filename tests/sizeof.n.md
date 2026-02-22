# sizeof の検証

`size_of<T>` が基本型とジェネリクスで正しく動作するかを確認します。

## sizeof_primitives

neplg2:test
ret: 0
```neplg2
#target wasm
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *

fn main <()->i32> ():
    if:
        eq size_of<i32> 4
        then:
            if:
                eq size_of<i64> 8
                then:
                    if:
                        eq size_of<f32> 4
                        then:
                            if:
                                eq size_of<f64> 8
                                then:
                                    if eq size_of<str> 4 0 5
                                else:
                                    4
                        else:
                            3
                else:
                    2
        else:
            1
```

## sizeof_generic_function

neplg2:test
ret: 0
```neplg2
#target wasm
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *

fn size_of_t <.T> <()->i32> ():
    size_of<.T>

fn main <()->i32> ():
    if:
        eq size_of<i32> size_of_t<i32>
        then:
            if:
                eq size_of<i64> size_of_t<i64>
                then:
                    if eq size_of<str> size_of_t<str> 0 3
                else:
                    2
        else:
            1
```

## sizeof_generic_struct_wrapper

neplg2:test
ret: 0
```neplg2
#target wasm
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *

struct Wrap<.T>:
    value <.T>

fn main <()->i32> ():
    if:
        eq size_of<i32> size_of<Wrap<i32>>
        then:
            if eq size_of<str> size_of<Wrap<str>> 0 2
        else:
            1
```

## sizeof_generic_param_requires_dot

neplg2:test[compile_fail]
```neplg2
#target std
#entry main
#indent 4
#import "core/mem" as *

fn bad_sizeof <T> <()->i32> ():
    size_of<T>

fn main <()*>()> ():
    bad_sizeof<i32>;
```
