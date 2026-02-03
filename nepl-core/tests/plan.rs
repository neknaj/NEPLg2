// tests/plan.rs
//
// This test suite is meant to validate the *core* language semantics described in plan.md:
// - Offside rule / indentation blocks
// - `:` block expressions
// - Statement semantics and `;` (including multiple semicolons)
// - `if` newline/layout variants (cond/then/else keywords and sugar)
// - `while` as an expression returning unit, used as a statement
//
// We intentionally avoid std/stdio and any printing. All assertions are based on returning i32.

mod harness;
use harness::run_main_i32;

use nepl_core::span::FileId;
use nepl_core::{compile_wasm, CompileOptions, CompileTarget};

fn compile_err(src: &str) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    assert!(result.is_err(), "expected error, got {:?}", result);
}

#[test]
fn plan_block_returns_last_statement_value() {
    // plan.md: block value/type comes from the last statement
    // previous statements' values are discarded.
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    let y <i32> :
        add 1 2;
        add 3 4
        add 5 6
    y
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 11);
}

#[test]
fn plan_block_trailing_semicolon_makes_unit_and_breaks_i32_return() {
    // plan.md: if the last statement in a block has `;`,
    // the block result is treated as unit `()`.
    //
    // Here, main expects i32 but the last statement is semicolon-terminated.
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    add 1 2;
"#;

    compile_err(src);
}

#[test]
fn plan_semicolon_requires_exactly_one_value_growth() {
    // plan.md: `;` checks that exactly one value was produced on the stack
    // from the block baseline. `add 1 2 3;` should be rejected.
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    add 1 2 3;
    0
"#;

    compile_err(src);
}

#[test]
fn plan_multiple_semicolons_allowed() {
    // plan.md: multiple semicolons like `;;;` are allowed on a statement.
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    add 1 2;;
    add 3 4;;;
    add 5 6
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 11);
}

#[test]
fn plan_block_used_as_function_argument() {
    // plan.md: `:` block is an expression and can be used as a function argument.
    //
    // add 1 (block returning 5) => 6
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    add 1:
        add 2 3
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 6);
}

#[test]
fn plan_if_one_line_basic() {
    // plan.md: base form: if <cond> <then> <else>
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    if true 10 20
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn plan_if_one_line_then_else_keywords() {
    // plan.md: "then"/"else" keywords can be inserted.
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    if true then 10 else 20
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn plan_if_multiline_then_else() {
    // plan.md: `if <cond>:` allows writing then/else on the next indented lines.
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    if true:
        then 10
        else 20
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn plan_if_multiline_then_else_with_blocks() {
    // plan.md: `then:` and `else:` are sugar for `:` but only in if.
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    if true:
        then:
            add 1 2
        else:
            add 3 4
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn plan_if_colon_form_three_exprs() {
    // plan.md: `if:` can lay out cond/then/else as 3 expressions under one indent.
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    if:
        lt 1 2
        10
        20
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn plan_if_colon_form_with_cond_then_else_keywords() {
    // plan.md: `if:` plus optional cond/then/else keywords.
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    if:
        cond lt 1 2
        then 10
        else 20
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn plan_if_colon_form_with_then_else_keywords() {
    // plan.md: `if:` plus optional then/else keywords.
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    if:
        lt 1 2
        then 10
        else 20
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn plan_if_nested_inline_forms() {
    // plan.md: nested if in expression position (prefix nesting)
    //
    // if true 0 (if true 1 2) => 0
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    if true 0 if true 1 2
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn plan_if_else_if_inline_chain() {
    // plan.md: else-if style via nesting in else expression
    //
    // if false then 0 else if true then 1 else 2 => 1
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    if false then 0 else if true then 1 else 2
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 1);
}

#[test]
fn plan_while_is_unit_and_works_as_statement() {
    // plan.md: while has type (bool,())->() and is an expression.
    // This test checks:
    // - while compiles
    // - it can appear as a statement in fn body
    // - it correctly updates a mutable variable using set
    //
    // x starts at 0; loop while x < 10; each iteration x = x + 1
    // result is x == 10
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()*>i32> ():
    let mut x <i32> 0;

    while lt x 10:
        set x add x 1;

    x
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn plan_nested_colon_blocks_in_set_expression() {
    // Exercise the "nested `:` block as expression" style used in plan.md.
    //
    // Use blocks as the second argument to add/sub to ensure parsing + indentation works.
    // This also validates that blocks work inside arguments and do not break return value.
    //
    // Each loop iteration:
    //   x = add x (block => 2)
    //   x = sub x (block => 1)
    // Net +1, loop until x == 10
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()*>i32> ():
    let mut x <i32> 0;

    while lt x 10:
        set x add x:
            2
        set x sub x:
            1

    x
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn plan_if_expression_used_as_argument() {
    // plan.md: if is an expression and can be used as a function argument.
    //
    // x = 7
    // add 100 (if x < 10 then 1 else 2) => 101
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    let x <i32> 7;
    add 100 if lt x 10 1 2
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 101);
}

#[test]
fn plan_if_expression_used_as_argument_multiline() {
    // Similar to above, but using multiline `if:` layout.
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    let x <i32> 7;
    add 100:
        if:
            lt x 10
            1
            2
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 101);
}

#[test]
fn plan_compile_only_if_layout_variants() {
    // A compile-only smoke test for a variety of if layouts in one program.
    // This is here to ensure parsing stays aligned with plan.md even if execution is not checked.
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/math" as *

fn main <()->i32> ():
    let a <i32> if true 1 2;
    let b <i32> if true then 1 else 2;
    let c <i32> if true:
        then 1
        else 2
    let d <i32> if:
        true
        1
        2
    let e <i32> if:
        cond:
            true
        then:
            1
        else:
            2
    add add add add a b c d e
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 5);
}
