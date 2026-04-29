# if.rs 由来の doctest

このファイルは Rust テスト `if.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## if_a_returns_expected

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target core
    #import "core/math" as *

fn main <()->i32> ():
    let a <i32> if true 0 1;
    a
```

## if_b_returns_expected

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target core
    #import "core/math" as *

fn main <()->i32> ():
    let b <i32> if true then 0 else 1;
    b
```

## if_c_returns_expected

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target core
    #import "core/math" as *

fn main <()->i32> ():
    let c <i32> if:
        true
        0
        1
    c
```

## if_d_returns_expected

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target core
    #import "core/math" as *

fn main <()->i32> ():
    let d <i32> if:
        cond true
        then 0
        else 1
    d
```

## if_e_returns_expected

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target core
    #import "core/math" as *

fn main <()->i32> ():
    let e <i32> if:
        true
        then:
            0
        else:
            1
    e
```

## if_f_returns_expected

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target core
    #import "core/math" as *

fn main <()->i32> ():
    let f <i32> if true 0 if true 1 2;
    f
```

## if_c_variant_lt_condition

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core
    #import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        lt 1 2
        10
        20
    v
```

## if_c_variant_block_values

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        true
        add 1 2
        add 3 4
    v
```

## if_c_variant_cond_keyword

neplg2:test
ret: 7
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond lt 2 3
        7
        8
    v
```

## if_mixed_cond_then_block_else_block

neplg2:test
ret: 11
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond true
        then:
            11
        else:
            12
    v
```

## if_mixed_cond_then_block_else_block_multi_expr_in_block

neplg2:test
ret: 11
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond true
        then:
            // ここはthen:によるblock
            5
            11
        else:
            // ここはelse:によるblock
            add 1 3;
            add 5 6
            12
    v
```

## if_mixed_layout_then_inline_else

neplg2:test
ret: 21
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        true
        then:
            21
        else 22
    v
```

## if_mixed_cond_inline_then_block_else_inline

neplg2:test
ret: 31
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond lt 1 2
        then:
            31
        else 32
    v
```

## if_inline_false_returns_expected

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let x <i32> if false 0 1;
    x
```

## if_block_false_returns_expected

neplg2:test
ret: 200
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let x <i32> if:
        false
        100
        200
    x
```

## if_mixed_cond_false_then_block_else_inline

neplg2:test
ret: 66
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let x <i32> if:
        cond lt 2 1
        then:
            55
        else 66
    x
```

## if_mixed_layout_then_inline_else_block_true

neplg2:test
ret: 11
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        true
        then 11
        else:
            12
    v
```

## if_mixed_layout_then_inline_else_block_false

neplg2:test
ret: 12
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        false
        then 11
        else:
            12
    v
```

## if_mixed_cond_then_inline_else_block_true

neplg2:test
ret: 21
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond lt 1 2
        then 21
        else:
            22
    v
```

## if_mixed_cond_then_inline_else_block_false

neplg2:test
ret: 22
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond lt 2 1
        then 21
        else:
            22
    v
```

## if_mixed_then_inline_else_block_then_expr_is_var

neplg2:test
ret: 77
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let a <i32> 77;
    let v <i32> if:
        true
        then a
        else:
            99
    v
```

## if_mixed_then_inline_else_block_else_block_multi_stmt

neplg2:test
ret: 115
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let a <i32> 5;
    let v <i32> if:
        false
        then 1
        else:
            let b <i32> add a 10;
            add b 100
    v
```

## if_then_block_else_block_then_multi_stmt_last_expr_wins

neplg2:test
ret: 30
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        true
        then:
            let x <i32> 10;
            let y <i32> 20;
            add x y
        else:
            0
    v
```

## if_then_block_else_block_else_multi_stmt_last_expr_wins

neplg2:test
ret: 12
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        false
        then:
            0
        else:
            let x <i32> 3;
            let y <i32> 4;
            mul x y
    v
```

## if_cond_keyword_with_then_else_inline_keywords

neplg2:test
ret: 9
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond and lt 1 2 lt 3 4
        then 9
        else 10
    v
```

## if_inline_keywordless_with_complex_condition

neplg2:test
ret: 100
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if and lt 1 2 lt 3 4 100 200;
    v
```

## if_inline_then_else_keywords_with_complex_condition_false

neplg2:test
ret: 8
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if or lt 2 1 eq 1 2 then 7 else 8;
    v
```

## if_used_as_last_expression_inline

neplg2:test
ret: 123
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    if true 123 456
```

## if_used_as_last_expression_block

neplg2:test
ret: 33
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    if:
        lt 1 2
        33
        44
```

## if_in_function_argument_position

neplg2:test
ret: 102
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    add 100 if false 1 2
```

## if_nested_in_then_branch_mixed_layouts

neplg2:test
ret: 2
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        true
        then:
            if:
                cond lt 2 1
                then 1
                else:
                    2
        else:
            9
    v
```

## if_nested_in_else_branch_inline_then_else_blocks

neplg2:test
ret: 7
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        false
        then:
            0
        else:
            if true then 7 else 8
    v
```

## if_chain_right_associative_like_expression

neplg2:test
ret: 2
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if false 0 if false 1 if true 2 3;
    v
```

## if_block_three_line_variant_nested_expression_values

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        lt 10 20
        add 1 2
        mul 3 4
    v
```

## if_blocks_can_do_side_effect_and_return_value_true_branch

neplg2:test
ret: 190
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let mut x <i32> 0;

    let y <i32> if:
        true
        then:
            set x 9;
            100
        else:
            set x 8;
            200

    add mul x 10 y
```

## if_blocks_can_do_side_effect_and_return_value_false_branch

neplg2:test
ret: 280
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let mut x <i32> 0;

    let y <i32> if:
        false
        then:
            set x 9;
            100
        else:
            set x 8;
            200

    add mul x 10 y
```

## if_cond_keyword_then_block_else_inline_false

neplg2:test
ret: 2
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond eq 1 2
        then:
            1
        else 2
    v
```

## if_cond_keyword_then_inline_else_block_nested_if_inside_else

neplg2:test
ret: 40
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond eq 1 2
        then 1
        else:
            if:
                true
                40
                50
    v
```

## if_then_block_else_block_nested_ifs_each_branch

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        lt 1 2
        then:
            if true 1 2
        else:
            if:
                true
                3
                4
    v
```

## if_then_inline_else_inline_inside_block_form_without_cond_keyword

neplg2:test
ret: 70
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        lt 5 6
        then 70
        else 80
    v
```

## if_then_inline_else_inline_inside_block_form_with_cond_keyword

neplg2:test
ret: 80
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond lt 6 5
        then 70
        else 80
    v
```

## if_then_block_else_block_condition_is_multiexpr

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        and lt 1 2 or eq 0 1 eq 2 2
        then:
            1
        else:
            2
    v
```

## if_block_variant_values_can_be_if_expressions_too

neplg2:test
ret: 2
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        true
        if false 1 2
        if true 3 4
    v
```

## reserved_cond_cannot_be_identifier

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    let cond 1;
    cond
```

## reserved_then_cannot_be_function_name

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4
#target core

fn then <()->i32> ():
    1

fn main <()->i32> ():
    then
```

## reserved_let_fn_cannot_be_identifier

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    let let 1;
    let fn 2;
    add let fn
```

## reserved_else_do_cannot_be_identifier

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    let else 1;
    let do 2;
    add else do
```

## if_cond_expr_colon_layout_then_else

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if lt 1 2:
        then 10
        else 20
    v
```

## if_cond_keyword_cond_expr_colon_layout_then_else

neplg2:test
ret: 40
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if cond lt 2 1:
        then 30
        else 40
    v
```

## if_layout_marker_order_error_then_before_cond

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    let v <i32> if:
        then 1
        cond true
        else 2
    v
```

## if_layout_then_marker_duplicate_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    let v <i32> if:
        cond true
        then 1
        then 2
    v
```

## if_layout_cond_marker_duplicate_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    let v <i32> if:
        cond true
        cond false
        then 2
        false 1
    v
```

## if_layout_missing_else_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    let v <i32> if:
        cond true
        then 1
    v
```


## if_layout_missing_then_error

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    let v <i32> if:
        cond true
        else 1
    v
```


## if_layout_missing_then_error_2

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    let v <i32> if:
        cond false
        else 1
    v
```

## if_layout_invalid_marker_order_reports_diag_code

neplg2:test[compile_fail]
```neplg2

#entry main
#indent 4
#target core

fn main <()->i32> ():
    let v <i32> if:
        then 1
        cond true
        else 0
    v
```

## if_nested_double_level_mixed_layout

neplg2:test
ret: 42
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond true
        then:
            if:
                cond lt 3 4
                then 42
                else 0
        else:
            if true 1 2
    v
```

## if_nested_inline_then_set_else_unit

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let mut x <i32> 0;
    let v <i32> if:
        true
        then:
            if:
                lt 1 2
                then set x 1
                else ()
            x
        else:
            0
    v
```

## if_nested_else_branch_block_and_inline_mix

neplg2:test
ret: 9
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        false
        then 1
        else:
            if:
                cond false
                then:
                    7
                else 9
    v
```

## if_condition_must_be_bool_reports_diag_code

neplg2:test[compile_fail]
diag_code: type.if.condition_mismatch
```neplg2
#entry main
#indent 4
#target core

fn main <()->i32> ():
    if 1 10 20
```

## while_body_must_be_unit_reports_diag_code

neplg2:test[compile_fail]
diag_code: type.while.body_mismatch
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let mut i <i32> 0;
    while lt i 3:
        add i 1
    0
```

## if_then_semicolon_turns_branch_into_unit_compile_fail

neplg2:test[compile_fail]
```neplg2
#entry main
#indent 4
#target core
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        true
        then:
            add 1 2;
        else:
            0
    v
```
