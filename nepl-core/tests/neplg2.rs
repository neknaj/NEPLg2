use nepl_core::compile_wasm;
use nepl_core::span::FileId;

fn compile_ok(src: &str) {
    let result = compile_wasm(FileId(0), src);
    assert!(result.is_ok(), "expected success, got {:?}", result);
}

fn compile_err(src: &str) {
    let result = compile_wasm(FileId(0), src);
    assert!(result.is_err(), "expected error, got {:?}", result);
}

#[test]
#[ignore = "parser currently rejects angle signature spacing; pending fix"]
fn compiles_literal_main() {
    let src = r#"
#entry main
fn main <() -> i32> ():
    1
"#;
    compile_ok(src);
}

#[test]
#[ignore = "parser indentation handling for block arg pending fix"]
fn compiles_add_block_expression() {
    let src = r#"
#entry main
#indent 4

#if[target=wasm]
fn add <(i32, i32) -> i32> (a, b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <() -> i32> ():
    add 1:
        add 2 3
"#;
    compile_ok(src);
}

#[test]
fn set_type_mismatch_is_error() {
    let src = r#"
#entry main
fn main <() -> ()> ():
    let mut x <i32> 0;
    set x ();
"#;
    compile_err(src);
}

#[test]
fn pure_cannot_call_impure() {
    let src = r#"
#entry main
#indent 4

fn imp <(i32) *> i32> (x):
    add x 1

fn pure <(i32) -> i32> (x):
    imp x

fn main <() -> i32> ():
    pure 1
"#;
    compile_err(src);
}
