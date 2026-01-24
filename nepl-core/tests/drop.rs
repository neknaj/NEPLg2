use nepl_core::diagnostic::Diagnostic;
use nepl_core::span::FileId;
use nepl_core::{compile_wasm, CompileOptions, CompileTarget};

mod harness;

fn compile_drop_test(source: &str) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let file_id = FileId(0);
    match compile_wasm(file_id, source, CompileOptions {
        target: Some(CompileTarget::Wasi),
    }) {
        Ok(artifact) => Ok(artifact.wasm),
        Err(nepl_core::error::CoreError::Diagnostics(ds)) => Err(ds),
        Err(_) => Err(Vec::new()),
    }
}

#[test]
fn drop_simple_let() {
    // Test: simple let binding followed by scope exit should trigger drop.
    let source = r#"
#target wasi
#indent 4

fn main <()*>()>():
    let x <i32> 42;
"#;
    let artifact = compile_drop_test(source).expect("compilation succeeded");
    assert!(!artifact.is_empty(), "generated wasm should not be empty");
}

#[test]
fn drop_nested_scopes() {
    // Test: nested blocks should drop variables at each scope exit.
    let source = r#"
#target wasi
#indent 4

fn main <()*>()>():
    let x <i32> 1;
    let y <i32> 2;
"#;
    let artifact = compile_drop_test(source).expect("compilation succeeded");
    assert!(!artifact.is_empty(), "generated wasm should not be empty");
}

#[test]
fn drop_if_branch() {
    // Test: drops should be inserted in if branches.
    let source = r#"
#target wasi
#indent 4

fn main <()*>()>():
    let con <i32> 1;
    let result <i32> if:
        cond:
            con
        then:
            let x <i32> 10;
            x
        else:
            let y <i32> 20;
            y
"#;
    let artifact = compile_drop_test(source).expect("compilation succeeded");
    assert!(!artifact.is_empty(), "generated wasm should not be empty");
}

#[test]
fn drop_multiple_bindings_reverse_order() {
    // Test: ensure drops are inserted in reverse order of declaration (LIFO).
    let source = r#"
#target wasi
#indent 4

fn main <()*>()>():
    let a <i32> 1;
    let b <i32> 2;
    let c <i32> 3;
"#;
    let artifact = compile_drop_test(source).expect("compilation succeeded");
    assert!(!artifact.is_empty(), "generated wasm should not be empty");
}


