# plan.rs 由来の doctest

このファイルは Rust テスト `plan.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## plan_block_returns_last_statement_value

neplg2:test
ret: 11
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    let y %i32 block:
        add 1 2;
        add 3 4
        add 5 6
    y
```

## plan_block_trailing_semicolon_makes_unit_and_breaks_i32_return

neplg2:test[compile_fail]
diag_code: type.return.mismatch
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    add 1 2;
```

## plan_semicolon_requires_exactly_one_value_growth

neplg2:test[compile_fail]
diag_code: type.stack.extra_values
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    add 1 2 3;
    0
```

## plan_multiple_semicolons_allowed

neplg2:test
ret: 11
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    add 1 2;;
    add 3 4;;;
    add 5 6
```

## plan_block_used_as_function_argument

neplg2:test
ret: 6
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    add 1 block:
        add 2 3
```

## plan_if_one_line_basic

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core

fn main %fn () i32 \():
    if true 10 20
```

## plan_if_one_line_then_else_keywords

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core

fn main %fn () i32 \():
    if true then 10 else 20
```

## plan_if_multiline_then_else

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core

fn main %fn () i32 \():
    if true:
        then 10
        else 20
```

## plan_if_multiline_then_else_with_blocks

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    if true:
        then:
            add 1 2
        else:
            add 3 4
```

## plan_if_colon_form_three_exprs

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    if:
        lt 1 2
        10
        20
```

## plan_if_colon_form_with_cond_then_else_keywords

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    if:
        cond lt 1 2
        then 10
        else 20
```

## plan_if_colon_form_with_then_else_keywords

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    if:
        lt 1 2
        then 10
        else 20
```

## plan_if_nested_inline_forms

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#target core

fn main %fn () i32 \():
    if true 0 if true 1 2
```

## plan_if_else_if_inline_chain

neplg2:test
ret: 1
```neplg2

#entry main
#indent 4
#target core

fn main %fn () i32 \():
    if false then 0 else if true then 1 else 2
```

## plan_while_is_unit_and_works_as_statement

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %impure fn () i32 \():
    let mut x %i32 0;

    while lt x 10:
        set x add x 1;

    x
```

## plan_nested_colon_blocks_in_set_expression

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %impure fn () i32 \():
    let mut x %i32 0;

    while lt x 10:
        do:
            set x add x block:
                2
            set x sub x block:
                1

    x
```

## plan_if_expression_used_as_argument

neplg2:test
ret: 101
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    let x %i32 7;
    add 100 if lt x 10 1 2
```

## plan_if_expression_used_as_argument_multiline

neplg2:test
ret: 101
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    let x %i32 7;
    add 100:
        if:
            lt x 10
            1
            2
```

## plan_compile_only_if_layout_variants

neplg2:test
ret: 5
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    let a %i32 if true 1 2;
    let b %i32 if true then 1 else 2;
    let c %i32 if true:
        then 1
        else 2
    let d %i32 if:
        true
        1
        2
    let e %i32 if:
        cond:
            true
        then:
            1
        else:
            2
    add add add add a b c d e
```

## plan_block_colon_trailing_comment_only_is_allowed

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    let v %i32 block: // trailing comment is allowed
        add 1 2
    v
```

## plan_block_colon_rejects_tokens_after_colon

neplg2:test[compile_fail]
diag_code: parser.token.expected
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    block: add 1 2
```

## plan_argument_offside_multiline_call_same_indent

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    add:
        add 1 2
        add 3 4
```

## plan_while_inline_cond_do_keywords

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %impure fn () i32 \():
    let mut i %i32 0;
    while cond lt i 3 do set i add i 1;
    i
```

## plan_while_block_cond_do_keywords

neplg2:test
ret: 4
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %impure fn () i32 \():
    let mut i %i32 0;
    while:
        cond:
            lt i 4
        do:
            set i add i 1
    i
```

## plan_function_literal_with_args_and_type_annotation

neplg2:test
ret: 6
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    let f %fn i32 i32 \x:
        add x 1
    f 5
```

## plan_fn_is_let_sugar_and_at_references_function_value

neplg2:test
ret: 8
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn inc %fn i32 i32 \x:
    add x 1

fn apply %fn i32 fn fn i32 i32 i32 \x\f:
    f x

fn main %fn () i32 \():
    apply 7 @inc
```

## plan_pipe_linebreak_without_extra_indent

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    let result %i32:
        add 1 add 2 3
        |> add 4
    result
```

## plan_single_line_block_nested_is_allowed

neplg2:test
ret: 99
```neplg2

#entry main
#indent 4
#target core

fn main %fn () i32 \():
    block block block 99
```

## plan_single_line_block_cannot_contain_multiline_block

neplg2:test[compile_fail]
diag_code: parser.token.unexpected
```neplg2

#entry main
#indent 4
#target core

fn main %fn () i32 \():
    block block:
        1
```

## plan_if_colon_consumes_exactly_three_expressions

neplg2:test[compile_fail]
diag_code: parser.token.expected
```neplg2

#entry main
#indent 4
#target core

fn main %fn () i32 \():
    if:
        true
        1
```

## plan_single_line_block_multiple_statements_with_semicolon

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    block let a 1; add a 2
```

## plan_single_line_block_trailing_semicolon_makes_unit

neplg2:test[compile_fail]
diag_code: type.annotation.mismatch
```neplg2

#entry main
#indent 4
#target core

fn main %fn () i32 \():
    let x %i32 block let a 1; a;
    x
```

## plan_block_line_with_two_statements_without_separator_is_error

neplg2:test[compile_fail]
diag_code: type.stack.extra_values
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    let _u %() block:
        add 1 2 add 3 4
    0
```

## plan_tuple_literal_multiline_basic

neplg2:test
ret: 2
```neplg2

#entry main
#indent 4
#target core
#import "core/field" as *

fn main %fn () i32 \():
    let t Tuple:
        1
        2
    get t 1
```

## plan_type_annotation_acts_like_identity_function

neplg2:test
ret: 3
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    let v %i32 %i32 add 1 2;
    v
```

## plan_function_literal_no_args_block_form

neplg2:test
ret: 9
```neplg2

#entry main
#indent 4
#target core
#import "core/math" as *

fn main %fn () i32 \():
    let f %fn () i32 \():
        add 4 5
    f
```
