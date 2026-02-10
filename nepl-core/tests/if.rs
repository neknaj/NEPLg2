mod harness;
use harness::run_main_i32;

#[test]
fn if_a_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm
    #import "core/math" as *

fn main <()->i32> ():
    let a <i32> if true 0 1;
    a
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn if_b_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm
    #import "core/math" as *

fn main <()->i32> ():
    let b <i32> if true then 0 else 1;
    b
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn if_c_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm
    #import "core/math" as *

fn main <()->i32> ():
    let c <i32> if:
        true
        0
        1
    c
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn if_d_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm
    #import "core/math" as *

fn main <()->i32> ():
    let d <i32> if:
        cond true
        then 0
        else 1
    d
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn if_e_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm
    #import "core/math" as *

fn main <()->i32> ():
    let e <i32> if:
        true
        then:
            0
        else:
            1
    e
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn if_f_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm
    #import "core/math" as *

fn main <()->i32> ():
    let f <i32> if true 0 if true 1 2;
    f
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn if_c_variant_lt_condition() {
    let src = r#"
#entry main
#indent 4
#target wasm
    #import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        lt 1 2
        10
        20
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn if_c_variant_block_values() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        true
        add 1 2
        add 3 4
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn if_c_variant_cond_keyword() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond lt 2 3
        7
        8
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 7);
}

#[test]
fn if_mixed_cond_then_block_else_block() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond true
        then:
            11
        else:
            12
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 11);
}

#[test]
fn if_mixed_layout_then_inline_else() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        true
        then:
            21
        else 22
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 21);
}

#[test]
fn if_mixed_cond_inline_then_block_else_inline() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond lt 1 2
        then:
            31
        else 32
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 31);
}

#[test]
fn if_inline_false_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let x <i32> if false 0 1;
    x
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 1);
}

#[test]
fn if_block_false_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let x <i32> if:
        false
        100
        200
    x
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 200);
}

#[test]
fn if_mixed_cond_false_then_block_else_inline() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let x <i32> if:
        cond lt 2 1
        then:
            55
        else 66
    x
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 66);
}

#[test]
fn if_mixed_layout_then_inline_else_block_true() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        true
        then 11
        else:
            12
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 11);
}

#[test]
fn if_mixed_layout_then_inline_else_block_false() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        false
        then 11
        else:
            12
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 12);
}

#[test]
fn if_mixed_cond_then_inline_else_block_true() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond lt 1 2
        then 21
        else:
            22
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 21);
}

#[test]
fn if_mixed_cond_then_inline_else_block_false() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond lt 2 1
        then 21
        else:
            22
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 22);
}

#[test]
fn if_mixed_then_inline_else_block_then_expr_is_var() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let a <i32> 77;
    let v <i32> if:
        true
        then a
        else:
            99
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 77);
}

#[test]
fn if_mixed_then_inline_else_block_else_block_multi_stmt() {
    let src = r#"
#entry main
#indent 4
#target wasm
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
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 115);
}

#[test]
fn if_then_block_else_block_then_multi_stmt_last_expr_wins() {
    let src = r#"
#entry main
#indent 4
#target wasm
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
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 30);
}

#[test]
fn if_then_block_else_block_else_multi_stmt_last_expr_wins() {
    let src = r#"
#entry main
#indent 4
#target wasm
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
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 12);
}

#[test]
fn if_cond_keyword_with_then_else_inline_keywords() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond and lt 1 2 lt 3 4
        then 9
        else 10
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 9);
}

#[test]
fn if_inline_keywordless_with_complex_condition() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if and lt 1 2 lt 3 4 100 200;
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 100);
}

#[test]
fn if_inline_then_else_keywords_with_complex_condition_false() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if or lt 2 1 eq 1 2 then 7 else 8;
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 8);
}

#[test]
fn if_used_as_last_expression_inline() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    if true 123 456
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 123);
}

#[test]
fn if_used_as_last_expression_block() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    if:
        lt 1 2
        33
        44
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 33);
}

#[test]
fn if_in_function_argument_position() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    add 100 if false 1 2
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 102);
}

#[test]
fn if_nested_in_then_branch_mixed_layouts() {
    let src = r#"
#entry main
#indent 4
#target wasm
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
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 2);
}

#[test]
fn if_nested_in_else_branch_inline_then_else_blocks() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        false
        then:
            0
        else:
            if true then 7 else 8
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 7);
}

#[test]
fn if_chain_right_associative_like_expression() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if false 0 if false 1 if true 2 3;
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 2);
}

#[test]
fn if_block_three_line_variant_nested_expression_values() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        lt 10 20
        add 1 2
        mul 3 4
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn if_blocks_can_do_side_effect_and_return_value_true_branch() {
    let src = r#"
#entry main
#indent 4
#target wasm
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
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 190);
}

#[test]
fn if_blocks_can_do_side_effect_and_return_value_false_branch() {
    let src = r#"
#entry main
#indent 4
#target wasm
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
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 280);
}

#[test]
fn if_cond_keyword_then_block_else_inline_false() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond eq 1 2
        then:
            1
        else 2
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 2);
}

#[test]
fn if_cond_keyword_then_inline_else_block_nested_if_inside_else() {
    let src = r#"
#entry main
#indent 4
#target wasm
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
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 40);
}

#[test]
fn if_then_block_else_block_nested_ifs_each_branch() {
    let src = r#"
#entry main
#indent 4
#target wasm
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
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 1);
}

#[test]
fn if_then_inline_else_inline_inside_block_form_without_cond_keyword() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        lt 5 6
        then 70
        else 80
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 70);
}

#[test]
fn if_then_inline_else_inline_inside_block_form_with_cond_keyword() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        cond lt 6 5
        then 70
        else 80
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 80);
}

#[test]
fn if_then_block_else_block_condition_is_multiexpr() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        and lt 1 2 or eq 0 1 eq 2 2
        then:
            1
        else:
            2
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 1);
}

#[test]
fn if_block_variant_values_can_be_if_expressions_too() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main <()->i32> ():
    let v <i32> if:
        true
        if false 1 2
        if true 3 4
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 2);
}
