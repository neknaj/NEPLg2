# overload.rs 由来の doctest

このファイルは Rust テスト `overload.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## test_overload_cast_like

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_overload_cast_like\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"return type selects overload\" expected=\"10\" actual=\"10\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

// val_cast: Same name, same input type, different return type.
// Case 1: i32 -> i32 (identity)
fn val_cast <(i32)->i32> (v):
    v

// Case 2: i32 -> bool (non-zero check)
fn val_cast <(i32)->bool> (v):
    ne v 0

fn main <()*>i32> ():
    let v <i32> 10

    // Use type annotation on variable to select overload
    let res_i32 <i32> val_cast v
    let res_bool <bool> val_cast v

    // res_i32 should be 10, res_bool should be true
    let actual <i32> if:
        res_bool
        then res_i32
        else 0
    let report:
        test::test_report_new "test_overload_cast_like"
        |> test::test_report_push test::assert_eq_i32 "return type selects overload" 10 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## test_overload_print_like

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_overload_print_like\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"argument type selects overload\" expected=\"3\" actual=\"3\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

// my_print: Same name, different input types.
// Case 1: i32 -> i32 (returns 1 to signal "printed i32")
fn my_print <(i32)->i32> (v):
    1

// Case 2: bool -> i32 (returns 2 to signal "printed bool")
fn my_print <(bool)->i32> (v):
    2

fn main <()*>i32> ():
    let s1 <i32> my_print 100
    let s2 <i32> my_print true

    let actual <i32> add s1 s2
    let report:
        test::test_report_new "test_overload_print_like"
        |> test::test_report_push test::assert_eq_i32 "argument type selects overload" 3 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## test_explicit_type_annotation_prefix

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"test_explicit_type_annotation_prefix\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"explicit annotation selects overload\" expected=\"11\" actual=\"11\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

// magic: Same input, different return types
fn magic <(i32)->i32> (v):
    add v 1

fn magic <(i32)->bool> (v):
    true

fn main <()*>i32> ():
    // Use <type> prefix expression to explicitly select overload
    // This is useful when type cannot be inferred from context

    // Force selection of (i32)->i32
    let v1 <i32> magic 10

    // Force selection of (i32)->bool
    let v2 <bool> magic 10

    let actual <i32> if:
        v2
        then v1
        else 0
    let report:
        test::test_report_new "test_explicit_type_annotation_prefix"
        |> test::test_report_push test::assert_eq_i32 "explicit annotation selects overload" 11 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## overload_new_selected_by_let_annotation

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"overload_new_selected_by_let_annotation\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"zero argument overload selected by let annotation\" expected=\"7\" actual=\"7\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

fn new <()->i32> ():
    7

fn new <()->bool> ():
    true

fn main <()*>i32> ():
    let a <i32> new;
    let b <bool> new;
    let actual <i32> if b a 0
    let report:
        test::test_report_new "overload_new_selected_by_let_annotation"
        |> test::test_report_push test::assert_eq_i32 "zero argument overload selected by let annotation" 7 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## overload_new_ambiguous_without_expected_type

neplg2:test[compile_fail]
diag_code: type.overload.ambiguous
```neplg2
#entry main
#indent 4
#target core

fn new <()->i32> ():
    1

fn new <()->bool> ():
    true

fn main <()->i32> ():
    let v new
    0
```

## overload_zero_arg_result_selected_by_expected_type

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"overload_zero_arg_result_selected_by_expected_type\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"result overload selected by expected type\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/result" as *
#import "core/math" as *
#import "std/test" as test

fn build <()->Result<i32,str>> ():
    Result::Ok 9

fn build <()->Result<bool,str>> ():
    Result::Ok true

fn main <()*>i32> ():
    let a <Result<i32,str>> build;
    let b <Result<bool,str>> build;

    let ok_a <bool>:
        match a:
            Result::Ok v:
                eq v 9
            Result::Err _e:
                false
    let ok_b <bool>:
        match b:
            Result::Ok v:
                v
            Result::Err _e:
                false
    let actual <i32> if and ok_a ok_b 1 0
    let report:
        test::test_report_new "overload_zero_arg_result_selected_by_expected_type"
        |> test::test_report_push test::assert_eq_i32 "result overload selected by expected type" 1 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## overload_zero_arg_result_ambiguous_without_expected_type

neplg2:test[compile_fail]
diag_code: effect.pure.calls_impure
```neplg2
#entry main
#indent 4
#target core
#import "core/result" as *

fn build <()*>Result<i32,str>> ():
    Result::Ok 1

fn build <()*>Result<bool,str>> ():
    Result::Ok true

fn main <()->i32> ():
    let x build
    0
```

## overload_len_for_string_and_vec

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"overload_len_for_string_and_vec\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"Vec len overload selected\" expected=\"2\" actual=\"2\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"str len overload selected\" expected=\"1001\" actual=\"1001\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "alloc/collections/vec" as v
#import "core/result" as *
#import "core/math" as *
#import "std/test" as test

fn size <(str)->i32> (s):
    add 1000 1

fn size <(Vec<i32>)->i32> (vec):
    let n <i32> v::len<i32> &vec;
    v::free<i32> vec;
    n

fn main <()*>i32> ():
    let v <Vec<i32>>:
        v::new<i32>
        |> uwok
        |> v::push<i32> 3 |> uwok
        |> v::push<i32> 5 |> uwok
    let a <i32> size v;
    let b <i32> size "x";
    let report:
        test::test_report_new "overload_len_for_string_and_vec"
        |> test::test_report_push test::assert_eq_i32 "Vec len overload selected" 2 a
        |> test::test_report_push test::assert_eq_i32 "str len overload selected" 1001 b
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## overload_new_with_pipe_vec

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"overload_new_with_pipe_vec\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"pipe keeps Vec constructor overload\" expected=\"2\" actual=\"2\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as v
#import "core/result" as *
#import "std/test" as test

fn new <()*>Vec<i32>> ():
    unwrap_ok v::new<i32>

fn new <()->bool> ():
    true

fn main <()*>i32> ():
    let v <Vec<i32>>:
        <Vec<i32>> new
        |> v::push 1 |> uwok
        |> v::push 2 |> uwok
    let n <i32> v::len<i32> &v;
    v::free<i32> v;
    let report:
        test::test_report_new "overload_new_with_pipe_vec"
        |> test::test_report_push test::assert_eq_i32 "pipe keeps Vec constructor overload" 2 n
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## overload_pair_field_from_generic_result_keeps_tuple_type

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"overload_pair_field_from_generic_result_keeps_tuple_type\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"generic Result tuple field keeps Vec type\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as v
#import "core/field" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as test

fn pair_with_empty <.T: Copy> <(Vec<.T>)->Result<.Pair, StdErrorKind>> (left):
    let right <Vec<.T>> uwok v::new<.T>;
    Result::Ok<.Pair, StdErrorKind> Tuple:
        left
        right

fn main <()*>i32> ():
    let xs <Vec<i32>>:
        v::new<i32>
        |> uwok
        |> v::push<i32> 1 |> uwok
    let parts unwrap_ok pair_with_empty<i32> xs;
    let evens <Vec<i32>> get parts 0;
    let rest <Vec<i32>> get parts 1;
    let n <i32> v::len<i32> &evens;
    v::free<i32> evens;
    v::free<i32> rest;
    let report:
        test::test_report_new "overload_pair_field_from_generic_result_keeps_tuple_type"
        |> test::test_report_push test::assert_eq_i32 "generic Result tuple field keeps Vec type" 1 n
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## overload_result_inferred_from_outer_arg_context

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"overload_result_inferred_from_outer_arg_context\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"outer bool argument context selects bool overload\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

fn choice <(i32)->i32> (v):
    v

fn choice <(i32)->bool> (v):
    ne v 0

fn use_bool <(bool)->i32> (b):
    if b 1 0

fn main <()*>i32> ():
    let actual <i32> use_bool choice 7;
    let report:
        test::test_report_new "overload_result_inferred_from_outer_arg_context"
        |> test::test_report_push test::assert_eq_i32 "outer bool argument context selects bool overload" 1 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## overload_star_import_prefers_concrete_over_generic_new

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"overload_star_import_prefers_concrete_over_generic_new\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"let annotation selects Vec new overload\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "alloc/collections/vec" as v
#import "core/result" as *
#import "std/test" as test

fn new <()*>Vec<i32>> ():
    unwrap_ok v::new<i32>

fn main <()*>i32> ():
    let v <Vec<i32>> <Vec<i32>> new;
    let n <i32> v::len<i32> &v;
    v::free<i32> v;
    let report:
        test::test_report_new "overload_star_import_prefers_concrete_over_generic_new"
        |> test::test_report_push test::assert_eq_i32 "let annotation selects Vec new overload" 0 n
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## overload_different_arity_is_error

neplg2:test[compile_fail]
diag_code: type.overload.ambiguous
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn calc <(i32)->i32> (a):
    add a 1

fn calc <(i32,i32)->i32> (a, b):
    add a b

fn use_binary <(i32,i32,(i32,i32)->i32)->i32> (a, b, f):
    f a b

fn main <()->i32> ():
    let a <i32> calc 5;
    let b <i32> use_binary 3 4 calc;
    add a b
```

## overload_different_arity_unary_simple_is_error

neplg2:test[compile_fail]
diag_code: type.overload.ambiguous
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn calc <(i32)->i32> (a):
    add a 1

fn calc <(i32,i32)->i32> (a, b):
    add a b

fn main <()->i32> ():
    calc 5
```

## overload_nested_len_with_stack_and_string

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"overload_nested_len_with_stack_and_string\" count=2 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"str len overload selected\" expected=\"3\" actual=\"3\" message=\"\"\nassertion index=1 status=ok kind=eq_i32 label=\"Stack len overload selected\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "alloc/collections/stack" as *
#import "core/math" as *
#import "core/result" as *
#import "std/test" as test

fn main <()*>i32> ():
    let s <str> "abc";
    let st <Stack<i32>> unwrap_ok<Stack<i32>, Diag> new;
    let n1 <i32> len s;
    let n2 <i32> len<i32> &st;
    free<i32> st;
    let report:
        test::test_report_new "overload_nested_len_with_stack_and_string"
        |> test::test_report_push test::assert_eq_i32 "str len overload selected" 3 n1
        |> test::test_report_push test::assert_eq_i32 "Stack len overload selected" 0 n2
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## overload_nested_call_arg_position_len

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"overload_nested_call_arg_position_len\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"nested str_trim result selects str len overload\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "alloc/collections/stack" as *
#import "core/math" as *
#import "std/test" as test

fn main <()*>i32> ():
    let t <str> str_trim "  x  ";
    let actual <i32> len t;
    let report:
        test::test_report_new "overload_nested_call_arg_position_len"
        |> test::test_report_push test::assert_eq_i32 "nested str_trim result selects str len overload" 1 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## overload_nested_call_arg_position_bool_chain

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"overload_nested_call_arg_position_bool_chain\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"nested bool chain keeps comparison overloads\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

fn main <()*>i32> ():
    let actual <i32> if and eq 1 1 lt 2 3 1 0;
    let report:
        test::test_report_new "overload_nested_call_arg_position_bool_chain"
        |> test::test_report_push test::assert_eq_i32 "nested bool chain keeps comparison overloads" 1 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## overload_nested_call_arg_position_bool_chain_literals

neplg2:test[stdio, normalize_newlines]
stdout: "test_report name=\"overload_nested_call_arg_position_bool_chain_literals\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"literal locals keep comparison overloads\" expected=\"1\" actual=\"1\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "core/math" as *
#import "std/test" as test

fn main <()*>i32> ():
    let a <i32> 1;
    let b <i32> 1;
    let c <i32> 2;
    let d <i32> 3;
    let actual <i32> if and eq a b lt c d 1 0;
    let report:
        test::test_report_new "overload_nested_call_arg_position_bool_chain_literals"
        |> test::test_report_push test::assert_eq_i32 "literal locals keep comparison overloads" 1 actual
    let shown test::test_report_print_stdout report
    test::test_report_exit_code shown
```

## overload_new_resolve_with_typed_block_context

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *

fn main <()*>i32> ():
    let st <Stack<i32>>:
        new
        |> unwrap_ok<Stack<i32>, Diag>
    let v <Vec<i32>>:
        new
        |> unwrap_ok<Vec<i32>, StdErrorKind>
    let sn <i32> len<i32> &st;
    free<i32> st;
    let vn <i32> len<i32> &v;
    free<i32> v;
    if and eq sn 0 eq vn 0 1 0
```

## overload_new_resolve_with_typed_block_and_pipe

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "alloc/collections/vec" as *
#import "alloc/collections/stack" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *

fn main <()*>i32> ():
    let st <Stack<i32>>:
        new
        |> unwrap_ok<Stack<i32>, Diag>
        |> push 10
        |> unwrap_ok<Stack<i32>, Diag>
    let n <i32> len<i32> &st;
    free<i32> st;
    if eq n 1 1 0
```

## overload_nested_call_arg_position_add_sub

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    if eq add 10 sub 8 3 15 1 0
```

## overload_nested_call_chain_add_mul

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> add mul 2 3 add 4 5;
    if eq v 15 1 0
```

## overload_different_arity_from_param_context_unary_is_error

neplg2:test[compile_fail]
diag_code: type.overload.ambiguous
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn calc <(i32)->i32> (a):
    add a 1

fn calc <(i32,i32)->i32> (a, b):
    add a b

fn use_unary <(i32,(i32)->i32)->i32> (a, f):
    f a

fn main <()->i32> ():
    use_unary 5 calc
```

## overload_different_arity_from_param_context_binary_is_error

neplg2:test[compile_fail]
diag_code: type.overload.ambiguous
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn calc <(i32)->i32> (a):
    add a 1

fn calc <(i32,i32)->i32> (a, b):
    add a b

fn use_binary <(i32,i32,(i32,i32)->i32)->i32> (a, b, f):
    f a b

fn main <()->i32> ():
    use_binary 3 4 calc
```

## overload_different_arity_with_pipe_unary_is_error

neplg2:test[compile_fail]
diag_code: type.overload.ambiguous
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn calc <(i32)->i32> (a):
    add a 1

fn calc <(i32,i32)->i32> (a, b):
    add a b

fn use_unary <(i32,(i32)->i32)->i32> (a, f):
    f a

fn main <()->i32> ():
    5 |> use_unary calc
```

## overload_select_by_parameter_context

neplg2:test
ret: 12
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn choose <(i32)->i32> (v):
    v

fn choose <(i32)->bool> (v):
    ne v 0

fn take_i32 <(i32)->i32> (v):
    v

fn take_bool <(bool)->i32> (v):
    if v 2 0

fn main <()->i32> ():
    add take_i32 choose 10 take_bool choose 1
```

## overload_select_by_explicit_result_ascription

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn convert <(i32)->i32> (v):
    v

fn convert <(i32)->bool> (v):
    ne v 0

fn main <()->i32> ():
    let b <bool> <bool> convert 9;
    if b 1 0
```

## overload_ambiguous_same_input_no_context

neplg2:test[compile_fail]
diag_code: type.overload.ambiguous
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn cast_like <(i32)->i32> (v):
    v

fn cast_like <(i32)->bool> (v):
    ne v 0

fn main <()->i32> ():
    let tmp cast_like 1
    0
```

## overload_no_matching_by_argument_type

neplg2:test[compile_fail]
diag_code: type.overload.no_match
```neplg2
#entry main
#indent 4
#target core

fn parse <(i32,i32)->i32> (a, b):
    a

fn parse <(bool,bool)->i32> (a, b):
    if a 1 0

fn main <()->i32> ():
    parse 1 true
```

## overload_too_many_arguments_reports_stack_extra

neplg2:test[compile_fail]
diag_code: type.stack.extra_values
```neplg2
#entry main
#indent 4
#target core

fn f <(i32)->i32> (a):
    a

fn f <(i32,i32)->i32> (a, b):
    a

fn main <()->i32> ():
    f 1 2 3
```

## overload_pipe_select_by_first_arg_type

neplg2:test
ret: 3
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn kind <(i32)->i32> (v):
    1

fn kind <(bool)->i32> (v):
    2

fn main <()->i32> ():
    let a <i32>:
        5
        |> kind
    let b <i32>:
        true
        |> kind
    add a b
```

## overload_pipe_chain_numeric_overloads

neplg2:test
ret: 14
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32>:
        3
        |> add 4
        |> mul 2
    v
```

## overload_pipe_type_mismatch_reports_no_matching

neplg2:test[compile_fail]
diag_code: type.overload.no_match
```neplg2
#entry main
#indent 4
#target core

fn need_i32 <(i32)->i32> (v):
    v

fn main <()->i32> ():
    let _v <i32>:
        true
        |> need_i32;
    0
```

## overload_cast_mixed_i32_i64_i128

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *
#import "core/cast" as *

fn main <()->i32> ():
    let a <i32> 7;
    let b <i64> cast a;
    let c <i128> cast b;
    let d <i64> cast c;
    let e <i64> add d <i64> cast 5;
    let ok1 <bool> eq d <i64> cast 7;
    let ok2 <bool> eq e <i64> cast 12;
    if and ok1 ok2 1 0
```

## overload_cast_mixed_requires_ascription

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target core
#import "core/cast" as *

fn main <()->i32> ():
    // 返り値型が未指定の cast は曖昧になる
    let v cast 10
    0
```

## overload_cast_inferred_from_fn_return_annotation

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn choose <(i32)->i32> (v):
    v

fn choose <(i32)->bool> (v):
    ne v 0

fn make_i32 <()->i32> ():
    choose 1

fn make_bool <()->bool> ():
    choose 1

fn main <()->i32> ():
    if make_bool make_i32 0
```

## overload_mixed_annotations_block_call_pipe_lambda

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn pick <(i32)->i32> (v):
    v

fn pick <(i32)->bool> (v):
    ne v 0

fn apply_i32 <((i32)->i32,i32)->i32> (f, x):
    f x

fn main <()->i32> ():
    let inc <(i32)->i32> (x):
        add x 1

    let base <i32>:
        <i32> block:
            apply_i32 inc 6
    let v <i32> add base 3;

    let ok_pick <bool> <bool> pick 1;
    if and ok_pick eq v 10 1 0
```

## overload_pipe_annotations_with_mixed_cast_i32_i64_i128

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *
#import "core/cast" as *

fn main <()->i32> ():
    let seed <i64> <i64> cast 5;
    let v64 <i64>:
        seed
        |> add <i64> cast 7

    let v128 <i128> <i128> cast v64;
    let back <i32> <i32> cast v128;
    if eq back 12 1 0
```

## overload_trait_method_type_args_not_supported

neplg2:test[compile_fail]
diag_code: type.trait_method.type_args_unsupported
```neplg2
#entry main
#indent 4
#target core

trait Show:
    fn show <(Self)->i32> (x):
        0

fn main <()->i32> ():
    Show::show<i32> 1
```

## overload_trait_method_not_found

neplg2:test[compile_fail]
diag_code: type.trait_method.not_found
```neplg2
#entry main
#indent 4
#target core

trait Show:
    fn show <(Self)->i32> (x):
        0

fn main <()->i32> ():
    Show::missing 1
```

## overload_trait_bound_unsatisfied

neplg2:test[compile_fail]
diag_code: type.trait_bound.unsatisfied
```neplg2
#entry main
#indent 4
#target core

trait Show:
    fn show <(Self)->i32> (x):
        x

impl Show for i32:
    fn show <(i32)->i32> (x):
        x

fn main <()->i32> ():
    Show::show true
```

## overload_type_annotation_direct_block_colon

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32>:
        <i32>:
            3
            |> add 4
    if eq v 7 1 0
```

## overload_type_annotation_block_colon_with_nested_calls

neplg2:test
ret: 1
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn choose <(i32)->i32> (v):
    add v 1

fn choose <(i32)->bool> (v):
    ne v 0

fn main <()->i32> ():
    let v <i32>:
        <i32>:
            <i32> choose add 2 3
    if eq v 6 1 0
```

## overload_invalid_field_access_reports_field_diag

neplg2:test[compile_fail]
diag_code: type.field.invalid_access
```neplg2
#entry main
#indent 4
#target core

fn main <()->i32> ():
    let v <i32> 10;
    v.len
```

## capability_directive_is_trait_local_only

neplg2:test[compile_fail]
diag_code: parser.token.unexpected
```neplg2
#entry main
#indent 4

#capability copy

fn main <()->i32> ():
    0
```
