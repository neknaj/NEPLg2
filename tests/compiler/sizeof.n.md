# sizeof の検証

`size_of<T>` が基本型とジェネリクスで正しく動作するかを確認します。

## sizeof_primitives

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"sizeof_primitives\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"primitive size layout\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let actual %i32 if:
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
    let report:
        test_report_new "sizeof_primitives"
        |> test_report_push assert_eq_i32 "primitive size layout" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## sizeof_generic_function

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"sizeof_generic_function\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"generic function size_of\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *
#import "std/test" as *

fn size_of_t <.T> %fn () i32 \():
    size_of<.T>

fn main %impure fn () i32 \():
    let actual %i32 if:
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
    let report:
        test_report_new "sizeof_generic_function"
        |> test_report_push assert_eq_i32 "generic function size_of" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## sizeof_generic_struct_wrapper

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"sizeof_generic_struct_wrapper\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"generic struct wrapper size\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *
#import "std/test" as *

struct Wrap<.T>:
    value %.T

fn main %impure fn () i32 \():
    let actual %i32 if:
        eq size_of<i32> size_of<Wrap<i32>>
        then:
            if eq size_of<str> size_of<Wrap<str>> 0 2
        else:
            1
    let report:
        test_report_new "sizeof_generic_struct_wrapper"
        |> test_report_push assert_eq_i32 "generic struct wrapper size" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## sizeof_multi_field_struct_regression

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"sizeof_multi_field_struct_regression\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"multi-field struct size\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *
#import "std/test" as *

struct Pair:
    a %i32
    b %i32

struct WidePair:
    a %i64
    b %i32

fn main %impure fn () i32 \():
    let actual %i32 if:
        eq size_of<Pair> 8
        then:
            if eq size_of<WidePair> 12 0 2
        else:
            1
    let report:
        test_report_new "sizeof_multi_field_struct_regression"
        |> test_report_push assert_eq_i32 "multi-field struct size" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## sizeof_algebraic_types

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"sizeof_algebraic_types\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"algebraic type size\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let s_i32 %i32 size_of<i32>;
    let s_str %i32 size_of<str>;
    let s_opt_i32 %i32 size_of<Option<i32>>;
    let s_opt_str %i32 size_of<Option<str>>;
    let s_res_i32_str %i32 size_of<Result<i32,str>>;
    let actual %i32 if:
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
    let report:
        test_report_new "sizeof_algebraic_types"
        |> test_report_push assert_eq_i32 "algebraic type size" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## sizeof_nested_generic_struct

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"sizeof_nested_generic_struct\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"nested generic struct size\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *
#import "core/option" as *
#import "core/result" as *
#import "std/test" as *

struct Cell<.T>:
    v %.T

struct Node<.T>:
    head %.T
    tail %Option .T

fn main %impure fn () i32 \():
    let s_cell_i64 %i32 size_of<Cell<i64>>;
    let s_i64 %i32 size_of<i64>;
    let s_node_i32 %i32 size_of<Node<i32>>;
    let s_res %i32 size_of<Result<Node<i32>, Cell<i64>>>;
    let actual %i32 if:
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
    let report:
        test_report_new "sizeof_nested_generic_struct"
        |> test_report_push assert_eq_i32 "nested generic struct size" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## sizeof_collection_structs

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"sizeof_collection_structs\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"collection struct size\" expected=\"0\" actual=\"0\" message=\"\"\n"
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
#import "std/test" as *

fn main %impure fn () i32 \():
    let vec_expected %i32 add (add 4 4) size_of<VecStorage<i32>>;
    let stack_expected %i32 add (add 4 4) size_of<Vec<Option<i32>>>;
    let actual %i32 if:
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
    let report:
        test_report_new "sizeof_collection_structs"
        |> test_report_push assert_eq_i32 "collection struct size" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## sizeof_diag_structs

[目的/もくてき]:
- `alloc/diag` の[主要/しゅよう] struct が `size_of` の[対象/たいしょう]として[扱/あつか]えることを[確/たし]かめます。

[何/なに]を[確/たし]かめるか:
- `Span` の layout が 3 つの `i32` ぶんである。
- `Diag` / `Diags` / `Outcome` が[不正/ふせい]な zero-size 扱いになっていない。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"sizeof_diag_structs\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"diag struct size\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#target std
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *
#import "alloc/diag/error" as *
#import "std/test" as *

fn main %impure fn () i32 \():
    let actual %i32 if:
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
    let report:
        test_report_new "sizeof_diag_structs"
        |> test_report_push assert_eq_i32 "diag struct size" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## sizeof_generic_param_requires_dot

neplg2:test[compile_fail]
diag_codes: parser.type_expr.invalid
```neplg2
#target std
#entry main
#indent 4
#import "core/mem" as *

fn bad_sizeof %T %fn () i32 \():
    size_of<T>

fn main %impure fn () () \():
    bad_sizeof<i32>;
```
