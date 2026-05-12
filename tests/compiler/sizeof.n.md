# sizeof の検証

`size_of<T>` が基本型とジェネリクスで正しく動作するかを確認します。

## sizeof_primitives

neplg2:test
ret: 0
```neplg2
#target core
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
#target core
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
#target core
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

## sizeof_multi_field_struct_regression

neplg2:test
ret: 0
```neplg2
#target core
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *

struct Pair:
    a <i32>
    b <i32>

struct WidePair:
    a <i64>
    b <i32>

fn main <()->i32> ():
    if:
        eq size_of<Pair> 8
        then:
            if eq size_of<WidePair> 12 0 2
        else:
            1
```

## sizeof_algebraic_types

neplg2:test
ret: 0
```neplg2
#target core
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *
#import "core/option" as *
#import "core/result" as *

fn main <()->i32> ():
    let s_i32 <i32> size_of<i32>;
    let s_str <i32> size_of<str>;
    let s_opt_i32 <i32> size_of<Option<i32>>;
    let s_opt_str <i32> size_of<Option<str>>;
    let s_res_i32_str <i32> size_of<Result<i32,str>>;
    if:
        lt s_opt_i32 s_i32
        then:
            1
        else:
            if:
                lt s_opt_str s_str
                then:
                    2
                else:
                    if:
                        lt s_res_i32_str s_opt_i32
                        then:
                            3
                        else:
                            0
```

## sizeof_nested_generic_struct

neplg2:test
ret: 0
```neplg2
#target core
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *
#import "core/option" as *
#import "core/result" as *

struct Cell<.T>:
    v <.T>

struct Node<.T>:
    head <.T>
    tail <Option<.T>>

fn main <()->i32> ():
    let s_cell_i64 <i32> size_of<Cell<i64>>;
    let s_i64 <i32> size_of<i64>;
    let s_node_i32 <i32> size_of<Node<i32>>;
    let s_res <i32> size_of<Result<Node<i32>, Cell<i64>>>;
    if:
        eq s_cell_i64 s_i64
        then:
            if:
                lt s_node_i32 s_i64
                then:
                    2
                else:
                    if:
                        lt s_res s_node_i32
                        then:
                            3
                        else:
                            0
        else:
            1
```

## sizeof_collection_structs

neplg2:test
ret: 0
```neplg2
#target std
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *
#import "core/option" as *
#import "core/traits/hash" as *
#import "alloc/collections/vec" as *
#import "alloc/collections/stack" as *
#import "alloc/collections/hashmap" as *
#import "alloc/collections/hashset" as *

fn main <()->i32> ():
    let vec_expected <i32> add (add 4 4) (add size_of<VecStorageState> size_of<MemPtr<i32>>);
    let stack_expected <i32> add (add 4 4) size_of<Vec<Option<i32>>>;
    if:
        eq size_of<Vec<i32>> vec_expected
        then:
            if:
                eq size_of<Stack<i32>> stack_expected
                then:
                    if:
                        gt size_of<HashMap<i32, i32, DefaultHash32>> 0
                        then:
                            if gt size_of<HashSet<i32, DefaultHash32>> 0 0 4
                        else:
                            3
                else:
                    2
        else:
            1
```

## sizeof_diag_structs

[目的/もくてき]:
- `alloc/diag` の[主要/しゅよう] struct が `size_of` の[対象/たいしょう]として[扱/あつか]えることを[確/たし]かめます。

[何/なに]を[確/たし]かめるか:
- `Span` の layout が 3 つの `i32` ぶんである。
- `Diag` / `Diags` / `Outcome` が[不正/ふせい]な zero-size 扱いになっていない。

neplg2:test
ret: 0
```neplg2
#target std
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *
#import "alloc/diag/error" as *

fn main <()->i32> ():
    if:
        eq size_of<Span> 12
        then:
            if:
                gt size_of<Diag> 0
                then:
                    if:
                        gt size_of<Diags> 0
                        then:
                            if:
                                gt size_of<Outcome<i32, StdErrorKind>> 0
                                then:
                                    0
                                else:
                                    4
                        else:
                            3
                else:
                    2
        else:
            1
```

## sizeof_generic_param_requires_dot

neplg2:test[compile_fail]
diag_codes: parser.type_expr.invalid
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
