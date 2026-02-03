use nepl_core::diagnostic::Diagnostic;
use nepl_core::loader::Loader;
use nepl_core::{compile_module, CompileOptions, CompileTarget};
use std::path::PathBuf;

mod harness;

fn compile_move_test(source: &str) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let mut loader = Loader::new(stdlib_root());
    let loaded = loader
        .load_inline("<test>".into(), source.to_string())
        .expect("load");


    match compile_module(
        loaded.module,
        CompileOptions {
            target: Some(CompileTarget::Wasi),
            verbose: false,
            profile: None,
        },
    ) {
        Ok(artifact) => Ok(artifact.wasm),
        Err(nepl_core::error::CoreError::Diagnostics(ds)) => {
            for d in &ds {
                eprintln!("DIAG: {}", d.message);
            }
            Err(ds)
        }
        Err(e) => {
            eprintln!("OTHER ERR: {:?}", e);
            Err(Vec::new())
        }
    }
}

fn stdlib_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib")
}

#[test]
fn move_simple_ok() {
    let source = r#"
#target wasi
#indent 4
#import "std/mem" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let x Wrapper::Val 1;
    let y <Wrapper> x; // x moved to y
"#;
    compile_move_test(source).expect("should succeed");
}

#[test]
fn move_use_after_move() {
    let source = r#"
#target wasi
#indent 4
#import "std/mem" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let x Wrapper::Val 1;
    let y <Wrapper> x; // x moved to y
    let z <Wrapper> x; // error: use of moved value x
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs
        .iter()
        .any(|d| d.message.contains("use of moved value")));
}

#[test]
fn move_in_branch() {
    let source = r#"
#target wasi
#indent 4
#import "std/mem" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let x Wrapper::Val 1;
    let cnd <bool> true;
    if cnd:
        then:
            let y <Wrapper> x; // conditionally moved
        else:
            ()
    let z <Wrapper> x; // error: potentially moved
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(|d| d.message.contains("potentially moved")));
}

#[test]
fn move_in_loop() {
    let source = r#"
#target wasi
#indent 4
#import "std/mem" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let x Wrapper::Val 1;
    let cnd <bool> true;
    while cnd:
        let y <Wrapper> x; // moved in first iteration, error in next
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(|d| d.message.contains("potentially moved")));
}

#[test]
fn move_reassign_non_copy() {
    let source = r#"
#target wasi
#indent 4
#import "std/mem" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let mut x Wrapper::Val 1;
    let y <Wrapper> x;      // moved
    set x = Wrapper::Val 2; // re-init 
    let z <Wrapper> x;      // OK
"#;
    compile_move_test(source).expect("re-init should be valid");
}

#[test]
fn move_reassign_copy() {
    let source = r#"
#target wasi
#indent 4

fn main <()*>()>():
    let mut x <i32> 1;
    let y <i32> x; // i32 is Copy, so x is NOT moved
    set x = 2;     // still valid
    let z <i32> x; // ok
"#;
    compile_move_test(source).expect("copy types should not move");
}
#[test]
fn move_reference_ok() {
    let source = r#"
#target wasi
#indent 4
#import "std/mem" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let x Wrapper::Val 1;
    let r <&Wrapper> &x; // x is borrowed, not moved
    let y <Wrapper> x;   // x is still valid and moved here
"#;
    compile_move_test(source).expect("references should not move the values");
}

#[test]
fn move_borrow_after_move_err() {
    let source = r#"
#target wasi
#indent 4
#import "std/mem" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let x Wrapper::Val 1;
    let y <Wrapper> x;   // x moved here
    let r <&Wrapper> &x; // error: borrow of moved value
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(|d| d.message.contains("borrow of moved value")));
}

#[test]
fn move_pass_to_function_err() {
    let source = r#"
#target wasi
#indent 4
#import "std/mem" as *

enum Wrapper:
    Val <i32>

fn consume <(Wrapper)->()> (w):
    ()

fn main <()*>()>():
    let x Wrapper::Val 1;
    consume x;
    let y <Wrapper> x; // error: use of moved value x
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(|d| d.message.contains("use of moved value")));
}

#[test]
fn move_struct_field_err() {
    let source = r#"
#target wasi
#indent 4
#import "std/mem" as *

enum Wrapper:
    Val <i32>

struct S:
    f <Wrapper>

fn main <()*>()>():
    let s <S> S Wrapper::Val 1;
    let a <Wrapper> s.f;
    let b <Wrapper> s.f; // error: use of moved value
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(|d| d.message.contains("use of moved value")));
}

#[test]
fn move_branch_reinit_mixed() {
    let source = r#"
#target wasi
#indent 4
#import "std/mem" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let mut x Wrapper::Val 1;
    let cnd <bool> true;
    if cnd:
        then:
            let y <Wrapper> x; // moved in then
        else:
            set x = Wrapper::Val 2; // re-init in else
    let z <Wrapper> x; // error: potentially moved
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(|d| d.message.contains("potentially moved")));
}

#[test]
fn move_nested_match_potentially_moved() {
    let source = r#"
#target wasi
#indent 4
#import "std/mem" as *

enum Wrapper:
    Val <i32>
enum BoolWrap:
    True
    False

fn main <()*>()>():
    let x Wrapper::Val 1;
    let a <BoolWrap> BoolWrap::True;
    match a:
        True:
            match a:
                True:
                    let y <Wrapper> x; // moved in inner arm
                False:
                    ()
        False:
            ()
    let z <Wrapper> x; // error: potentially moved
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(|d| d.message.contains("potentially moved")));
}

#[test]
fn move_in_match_arms() {
    let source = r#"
#target wasi
#indent 4
#import "std/mem" as *

enum Wrapper:
    Val <i32>
enum BoolWrap:
    True
    False

fn main <()*>()>():
    let x Wrapper::Val 1;
    let v <BoolWrap> BoolWrap::True;
    match v:
        True:
            let y <Wrapper> x; // moved in this arm
        False:
            ()
    let z <Wrapper> x; // error: potentially moved
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(|d| d.message.contains("potentially moved")));
}
