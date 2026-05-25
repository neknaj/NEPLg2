use nepl_core::diagnostic::Diagnostic;
use nepl_core::diagnostic_codes::{
    DiagnosticCode, ResourceBorrowDiagnosticCode, ResourceCellDiagnosticCode,
    ResourceDiagnosticCode, ResourceMoveDiagnosticCode,
};
use nepl_core::loader::Loader;
use nepl_core::{compile_module_with_source_map, CompileOptions, CompileTarget};
use std::path::PathBuf;

mod harness;

fn compile_move_test(source: &str) -> Result<Vec<u8>, Vec<Diagnostic>> {
    compile_move_test_at_path(PathBuf::from("<test>"), source)
}

fn compile_raw_memory_boundary_move_test(source: &str) -> Result<Vec<u8>, Vec<Diagnostic>> {
    compile_move_test_at_path(stdlib_root().join("__raw_boundary_test.nepl"), source)
}

fn compile_move_test_at_path(path: PathBuf, source: &str) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let mut loader = Loader::new(stdlib_root());
    let loaded = loader.load_inline(path, source.to_string()).expect("load");

    match compile_module_with_source_map(
        loaded.module,
        Some(&loaded.source_map),
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

fn is_cell_diag(diag: &Diagnostic, code: ResourceCellDiagnosticCode) -> bool {
    diag.code == DiagnosticCode::Resource(ResourceDiagnosticCode::Cell(code))
}

fn is_cell_moved_or_possibly_moved(diag: &Diagnostic) -> bool {
    is_cell_diag(diag, ResourceCellDiagnosticCode::Moved)
        || is_cell_diag(diag, ResourceCellDiagnosticCode::PossiblyMoved)
}

fn is_move_diag(diag: &Diagnostic, code: ResourceMoveDiagnosticCode) -> bool {
    diag.code == DiagnosticCode::Resource(ResourceDiagnosticCode::Move(code))
}

fn is_moved_value_diag(diag: &Diagnostic) -> bool {
    is_move_diag(diag, ResourceMoveDiagnosticCode::UseMoved)
        || is_cell_diag(diag, ResourceCellDiagnosticCode::Moved)
}

fn is_possibly_moved_value_diag(diag: &Diagnostic) -> bool {
    is_move_diag(diag, ResourceMoveDiagnosticCode::UsePossiblyMoved)
        || is_move_diag(diag, ResourceMoveDiagnosticCode::LoopPossiblyMoved)
        || is_cell_moved_or_possibly_moved(diag)
}

fn is_borrow_diag(diag: &Diagnostic, code: ResourceBorrowDiagnosticCode) -> bool {
    diag.code == DiagnosticCode::Resource(ResourceDiagnosticCode::Borrow(code))
}

fn is_return_escape_diag(diag: &Diagnostic) -> bool {
    is_borrow_diag(diag, ResourceBorrowDiagnosticCode::ReturnEscape)
}

#[test]
fn move_simple_ok() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

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
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let x Wrapper::Val 1;
    let y <Wrapper> x; // x moved to y
    let z <Wrapper> x; // error: use of moved value x
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(is_moved_value_diag));
}

#[test]
fn move_in_branch() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

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
    assert!(errs.iter().any(is_possibly_moved_value_diag));
}

#[test]
fn move_in_loop() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let x Wrapper::Val 1;
    let cnd <bool> true;
    while cnd:
        let y <Wrapper> x; // moved in first iteration, error in next
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(is_possibly_moved_value_diag));
}

#[test]
fn move_reassign_non_copy() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

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
#import "core/mem" as *
#import "core/mem/raw" as *

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
fn move_live_reference_blocks_move() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let x Wrapper::Val 1;
    let r <&Wrapper> &x;
    let y <Wrapper> x;
    let z <&Wrapper> r;
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs
        .iter()
        .any(|d| is_borrow_diag(d, ResourceBorrowDiagnosticCode::MoveFromShared)));
}

#[test]
fn move_mut_reference_call_arg_is_temporary_borrow() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn touch_mut <(&mut Wrapper)->i32> (_x):
    1

fn consume <(Wrapper)->i32> (_x):
    0

fn main <()*>()>():
    let x Wrapper::Val 1;
    touch_mut &mut x;
    consume x;
    ()
"#;
    compile_move_test(source).expect("temporary mutable borrow should end after the call");
}

#[test]
fn move_call_mut_and_shared_reference_args_overlap_rejected() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn use_both <(&mut Wrapper,&Wrapper)->i32> (_a, _b):
    0

fn main <()*>()>():
    let x Wrapper::Val 1;
    use_both &mut x &x;
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs
        .iter()
        .any(|d| is_borrow_diag(d, ResourceBorrowDiagnosticCode::BorrowDuringUnique)));
}

#[test]
fn move_call_shared_and_mut_reference_args_overlap_rejected() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn use_both <(&Wrapper,&mut Wrapper)->i32> (_a, _b):
    0

fn main <()*>()>():
    let x Wrapper::Val 1;
    use_both &x &mut x;
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs
        .iter()
        .any(|d| is_borrow_diag(d, ResourceBorrowDiagnosticCode::UniqueDuringShared)));
}

#[test]
fn move_struct_mut_and_shared_reference_fields_overlap_rejected() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

struct RefPair:
    a <&mut Wrapper>
    b <&Wrapper>

fn main <()*>()>():
    let x Wrapper::Val 1;
    let p <RefPair> RefPair &mut x &x;
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs
        .iter()
        .any(|d| is_borrow_diag(d, ResourceBorrowDiagnosticCode::BorrowDuringUnique)));
}

#[test]
fn move_tuple_mut_and_shared_reference_items_overlap_rejected() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let x Wrapper::Val 1;
    let p Tuple:
        &mut x
        &x
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs
        .iter()
        .any(|d| is_borrow_diag(d, ResourceBorrowDiagnosticCode::BorrowDuringUnique)));
}

#[test]
fn move_unique_reference_blocks_owner_move_while_live() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let x Wrapper::Val 1;
    let r <&mut Wrapper> &mut x;
    let y <Wrapper> x;
    let keep <&mut Wrapper> r;
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs
        .iter()
        .any(|d| is_borrow_diag(d, ResourceBorrowDiagnosticCode::UseDuringUnique)));
}

#[test]
fn move_unique_reference_last_use_releases_owner() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let x Wrapper::Val 1;
    let r <&mut Wrapper> &mut x;
    let rr <&mut Wrapper> r;
    let y <Wrapper> x;
"#;
    compile_move_test(source).expect("last use of mutable reference should release the borrow");
}

#[test]
fn move_mut_reference_is_not_copy() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let x Wrapper::Val 1;
    let r <&mut Wrapper> &mut x;
    let rr <&mut Wrapper> r;
    let again <&mut Wrapper> r;
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(is_moved_value_diag));
}

#[test]
fn move_shared_borrow_blocks_unique_borrow() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let x Wrapper::Val 1;
    let r <&Wrapper> &x;
    let u <&mut Wrapper> &mut x;
    let keep <&Wrapper> r;
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs
        .iter()
        .any(|d| is_borrow_diag(d, ResourceBorrowDiagnosticCode::UniqueDuringShared)));
}

#[test]
fn move_unique_borrow_blocks_shared_borrow() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let x Wrapper::Val 1;
    let r <&mut Wrapper> &mut x;
    let s <&Wrapper> &x;
    let keep <&mut Wrapper> r;
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs
        .iter()
        .any(|d| is_borrow_diag(d, ResourceBorrowDiagnosticCode::BorrowDuringUnique)));
}

#[test]
fn move_copy_unique_borrow_blocks_shared_borrow() {
    let source = r#"
#target wasi
#indent 4

fn main <()*>()>():
    let x <i32> 1;
    let u <&mut i32> &mut x;
    let s <&i32> &x;
    let keep <&mut i32> u;
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs
        .iter()
        .any(|d| is_borrow_diag(d, ResourceBorrowDiagnosticCode::BorrowDuringUnique)));
}

#[test]
fn move_copy_shared_borrow_blocks_unique_borrow() {
    let source = r#"
#target wasi
#indent 4

fn main <()*>()>():
    let x <i32> 1;
    let s <&i32> &x;
    let u <&mut i32> &mut x;
    let keep <&i32> s;
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs
        .iter()
        .any(|d| is_borrow_diag(d, ResourceBorrowDiagnosticCode::UniqueDuringShared)));
}

#[test]
fn move_copy_shared_borrow_allows_owner_copy_while_reference_live() {
    let source = r#"
#target wasi
#indent 4

fn main <()*>()>():
    let x <i32> 1;
    let s <&i32> &x;
    let y <i32> x;
    let keep <&i32> s;
"#;
    compile_move_test(source)
        .expect("shared borrow of a Copy value should not block copying the owner value");
}

#[test]
fn move_branch_reference_last_use_releases_at_join() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let x Wrapper::Val 1;
    let r <&Wrapper> &x;
    let cnd <bool> true;
    if cnd:
        then:
            let rr <&Wrapper> r;
            ()
        else:
            ()
    let y <Wrapper> x;
"#;
    compile_move_test(source).expect("branch-local last use should release the borrow at join");
}

#[test]
fn move_branch_retained_borrow_blocks_later_move() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let x Wrapper::Val 1;
    let y Wrapper::Val 2;
    let mut r <&Wrapper> &x;
    let cnd <bool> true;
    if cnd:
        then:
            set r &y;
        else:
            ()
    let moved <Wrapper> y;
    let still_live <&Wrapper> r;
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs
        .iter()
        .any(|d| is_borrow_diag(d, ResourceBorrowDiagnosticCode::MoveFromShared)));
}

#[test]
fn move_borrow_after_move_err() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn main <()*>()>():
    let x Wrapper::Val 1;
    let y <Wrapper> x;   // x moved here
    let r <&Wrapper> &x; // error: borrow of moved value
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(is_moved_value_diag));
}

#[test]
fn move_return_local_reference_err() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn leak <()->&Wrapper> ():
    let x Wrapper::Val 1;
    &x

fn main <()*>()> ():
    let r <&Wrapper> leak;
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(is_return_escape_diag));
}

#[test]
fn move_block_local_reference_escape_err() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn main <()*>()> ():
    let r <&Wrapper> block:
        let x Wrapper::Val 1;
        &x
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(is_return_escape_diag));
}

#[test]
fn move_set_outer_reference_to_inner_local_err() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn main <()*>()> ():
    let outer Wrapper::Val 0;
    let mut r <&Wrapper> &outer;
    block:
        let inner Wrapper::Val 1;
        set r &inner
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(is_return_escape_diag));
}

#[test]
fn move_return_local_reference_inside_struct_err() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

struct RefBox:
    inner <&Wrapper>

fn leak <()->RefBox> ():
    let x Wrapper::Val 1;
    let b <RefBox> RefBox &x;
    b

fn main <()*>()> ():
    let b <RefBox> leak;
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(is_return_escape_diag));
}

#[test]
fn move_block_local_reference_inside_struct_escape_err() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

struct RefBox:
    inner <&Wrapper>

fn main <()*>()> ():
    let b <RefBox> block:
        let x Wrapper::Val 1;
        let local <RefBox> RefBox &x;
        local
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(is_return_escape_diag));
}

#[test]
fn move_set_outer_struct_reference_to_inner_local_err() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

struct RefBox:
    inner <&Wrapper>

fn main <()*>()> ():
    let outer Wrapper::Val 0;
    let mut b <RefBox> RefBox &outer;
    block:
        let inner Wrapper::Val 1;
        let local <RefBox> RefBox &inner;
        set b local
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(is_return_escape_diag));
}

#[test]
fn move_call_return_reference_to_block_local_err() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn id_ref <(&Wrapper)->&Wrapper> (x):
    x

fn main <()*>()> ():
    let r <&Wrapper> block:
        let x Wrapper::Val 1;
        id_ref &x
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(is_return_escape_diag));
}

#[test]
fn move_call_return_struct_reference_to_block_local_err() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

struct RefBox:
    inner <&Wrapper>

fn box_ref <(&Wrapper)->RefBox> (x):
    RefBox x

fn main <()*>()> ():
    let b <RefBox> block:
        let x Wrapper::Val 1;
        box_ref &x
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(is_return_escape_diag));
}

#[test]
fn move_pass_to_function_err() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

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
    assert!(errs.iter().any(is_moved_value_diag));
}

#[test]
fn move_struct_field_err() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

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
    assert!(errs.iter().any(is_moved_value_diag));
}

#[test]
fn move_distinct_owned_struct_fields_once_ok() {
    let source = r#"
#target wasi
#indent 4
#import "core/field" as field
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

struct Pair:
    left <Wrapper>
    right <Wrapper>

fn consume <(Wrapper)->()> (_w):
    ()

fn main <()*>()> ():
    let p <Pair> Pair (Wrapper::Val 1) (Wrapper::Val 2);
    let left <Wrapper> field::get p "left";
    let right <Wrapper> field::get p "right";
    consume left;
    consume right;
    ()
"#;
    compile_move_test(source).expect("distinct non-Copy fields should move exactly once");
}

#[test]
fn move_same_owned_struct_field_twice_rejected() {
    let source = r#"
#target wasi
#indent 4
#import "core/field" as field
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

struct Pair:
    left <Wrapper>
    right <Wrapper>

fn main <()*>()> ():
    let p <Pair> Pair (Wrapper::Val 1) (Wrapper::Val 2);
    let left <Wrapper> field::get p "left";
    let again <Wrapper> field::get p "left";
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(is_moved_value_diag));
}

#[test]
fn move_generic_distinct_owned_struct_fields_once_ok() {
    let source = r#"
#target wasi
#indent 4
#import "core/field" as field
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

struct Holder<.T, .E>:
    first <.T>
    second <.E>

fn split <.T, .E> <(Holder<.T, .E>)*>()> (h):
    let first <.T> field::get h "first"
    let second <.E> field::get h "second"
    ()

fn main <()*>()> ():
    let h <Holder<Wrapper, Wrapper>> Holder<Wrapper, Wrapper> (Wrapper::Val 1) (Wrapper::Val 2)
    split<Wrapper, Wrapper> h
"#;
    compile_move_test(source).expect("generic distinct non-Copy fields should move once each");
}

#[test]
fn move_generic_same_owned_struct_field_twice_rejected() {
    let source = r#"
#target wasi
#indent 4
#import "core/field" as field
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

struct Holder<.T, .E>:
    first <.T>
    second <.E>

fn split <.T, .E> <(Holder<.T, .E>)*>()> (h):
    let first <.T> field::get h "first"
    let again <.T> field::get h "first"
    ()

fn main <()*>()> ():
    let h <Holder<Wrapper, Wrapper>> Holder<Wrapper, Wrapper> (Wrapper::Val 1) (Wrapper::Val 2)
    split<Wrapper, Wrapper> h
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(is_moved_value_diag));
}

#[test]
fn move_owner_after_partial_field_move_rejected() {
    let source = r#"
#target wasi
#indent 4
#import "core/field" as field
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

struct Pair:
    left <Wrapper>
    right <Wrapper>

fn consume_pair <(Pair)->()> (_p):
    ()

fn main <()*>()> ():
    let p <Pair> Pair (Wrapper::Val 1) (Wrapper::Val 2);
    let left <Wrapper> field::get p "left";
    consume_pair p;
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(is_cell_moved_or_possibly_moved));
}

#[test]
fn move_raw_aggregate_copy_field_read_keeps_whole_place_available() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *
#import "core/field" as field
#import "core/math" as *

struct LocalToken:
    raw <(i32)->i32>

struct Holder:
    count <i32>
    token <LocalToken>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>()> ():
    let p <i32> 16
    store<Holder> p Holder 7 LocalToken @token_id
    let a <i32> field::get load<Holder> p "count"
    let b <i32> field::get load<Holder> p "count"
    let h <Holder> load<Holder> p
    ()
"#;
    compile_raw_memory_boundary_move_test(source)
        .expect("copy raw aggregate field reads should not move the owner");
}

#[test]
fn move_raw_aggregate_non_copy_field_move_blocks_whole_load() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *
#import "core/field" as field

struct LocalToken:
    raw <(i32)->i32>

struct Holder:
    count <i32>
    token <LocalToken>

fn token_id <(i32)->i32> (x):
    x

fn main <()*>()> ():
    let p <i32> 16
    store<Holder> p Holder 7 LocalToken @token_id
    let token <LocalToken> field::get load<Holder> p "token"
    let h <Holder> load<Holder> p
    ()
"#;
    let errs = compile_raw_memory_boundary_move_test(source).unwrap_err();
    assert!(errs.iter().any(is_cell_moved_or_possibly_moved));
}

#[test]
fn move_field_from_borrowed_owner_rejected() {
    let source = r#"
#target wasi
#indent 4
#import "core/field" as field
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

struct Pair:
    left <Wrapper>
    right <Wrapper>

fn observe <(&Pair)->i32> (_p):
    1

fn main <()*>()> ():
    let p <Pair> Pair (Wrapper::Val 1) (Wrapper::Val 2);
    let borrowed <&Pair> &p;
    let left <Wrapper> field::get p "left";
    observe borrowed;
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs
        .iter()
        .any(|d| is_borrow_diag(d, ResourceBorrowDiagnosticCode::MoveFromShared)));
}

#[test]
fn move_deref_copy_reference_ok() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

fn main <()*>()> ():
    let x <i32> 7;
    let y <i32> *&x;
    let z <i32> x;
    ()
"#;
    compile_move_test(source).expect("copy deref should not move the source");
}

#[test]
fn move_deref_non_copy_reference_rejected() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

fn main <()*>()> ():
    let x <Wrapper> Wrapper::Val 1;
    let y <Wrapper> *&x;
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs
        .iter()
        .any(|d| is_borrow_diag(d, ResourceBorrowDiagnosticCode::MoveFromShared)));
}

#[test]
fn move_deref_non_copy_field_reference_rejected() {
    let source = r#"
#target wasi
#indent 4
#import "core/field" as field
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

struct Pair:
    token <Wrapper>
    count <i32>

fn main <()*>()> ():
    let p <Pair> Pair (Wrapper::Val 1) 7;
    let token <Wrapper> *field::get_ref &p "token";
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs
        .iter()
        .any(|d| is_borrow_diag(d, ResourceBorrowDiagnosticCode::MoveFromShared)));
}

#[test]
fn move_branch_reinit_mixed() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

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
    assert!(errs.iter().any(is_possibly_moved_value_diag));
}

#[test]
fn move_nested_match_potentially_moved() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

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
    assert!(errs.iter().any(is_possibly_moved_value_diag));
}

#[test]
fn move_in_match_arms() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

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
    assert!(errs.iter().any(is_possibly_moved_value_diag));
}

#[test]
fn move_match_reference_payload_blocks_owner_move_while_live() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

enum RefOpt:
    Some <&Wrapper>
    None

fn main <()*>()>():
    let x Wrapper::Val 1;
    let e <RefOpt> RefOpt::Some &x;
    match e:
        Some r:
            let y <Wrapper> x;
            let keep <&Wrapper> r;
            ()
        None:
            ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs
        .iter()
        .any(|d| is_borrow_diag(d, ResourceBorrowDiagnosticCode::MoveFromShared)));
}

#[test]
fn move_match_reference_payload_last_use_releases_owner() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

enum RefOpt:
    Some <&Wrapper>
    None

fn main <()*>()>():
    let x Wrapper::Val 1;
    let e <RefOpt> RefOpt::Some &x;
    match e:
        Some r:
            let keep <&Wrapper> r;
            let y <Wrapper> x;
            ()
        None:
            ()
"#;
    compile_move_test(source)
        .expect("reference payload borrow should release after the binding's last use");
}

#[test]
fn move_loop_owned_accumulator_reassigned_after_result_ok() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

enum Wrapper:
    Val <i32>

fn step <(Wrapper)->Result<Wrapper, i32>> (w):
    Result<Wrapper, i32>::Ok w

fn main <()*>()> ():
    let mut cur Wrapper::Val 0;
    let mut i <i32> 0;
    while lt i 3:
        match step cur:
            Result::Ok next:
                set cur next
                set i add i 1
            Result::Err _e:
                #intrinsic "unreachable" <> ()
    let out <Wrapper> cur;
    ()
"#;
    compile_move_test(source).expect("diverging Err arm should not merge moved accumulator state");
}

#[test]
fn move_loop_owned_accumulator_err_continue_without_reinit_rejected() {
    let source = r#"
#target wasi
#indent 4
#import "core/mem" as *
#import "core/mem/raw" as *
#import "core/math" as *
#import "core/result" as *

enum Wrapper:
    Val <i32>

fn step <(Wrapper)->Result<Wrapper, i32>> (w):
    Result<Wrapper, i32>::Ok w

fn main <()*>()> ():
    let mut cur Wrapper::Val 0;
    let mut i <i32> 0;
    while lt i 3:
        match step cur:
            Result::Ok next:
                set cur next
                set i add i 1
            Result::Err _e:
                set i 3
    let out <Wrapper> cur;
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(is_possibly_moved_value_diag));
}

#[test]
fn move_borrowed_field_projection_keeps_owner_until_reference_last_use() {
    let source = r#"
#target wasi
#indent 4
#import "core/field" as field
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

struct Pair:
    token <Wrapper>
    count <i32>

fn observe <(&Wrapper)->i32> (_w):
    1

fn consume <(Pair)->i32> (_p):
    0

fn main <()*>()> ():
    let p <Pair> Pair (Wrapper::Val 1) 7;
    let token_ref <&Wrapper> field::get_ref &p "token";
    let count <i32> *field::get_ref &p "count";
    observe token_ref;
    consume p;
    ()
"#;
    compile_move_test(source).expect("field reference should borrow the owner without moving it");
}

#[test]
fn move_borrowed_field_projection_blocks_owner_move_while_live() {
    let source = r#"
#target wasi
#indent 4
#import "core/field" as field
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

struct Pair:
    token <Wrapper>
    count <i32>

fn observe <(&Wrapper)->i32> (_w):
    1

fn consume <(Pair)->i32> (_p):
    0

fn main <()*>()> ():
    let p <Pair> Pair (Wrapper::Val 1) 7;
    let token_ref <&Wrapper> field::get_ref &p "token";
    consume p;
    observe token_ref;
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs
        .iter()
        .any(|d| is_borrow_diag(d, ResourceBorrowDiagnosticCode::MoveFromShared)));
}

#[test]
fn move_borrowed_field_projection_escape_rejected() {
    let source = r#"
#target wasi
#indent 4
#import "core/field" as field
#import "core/mem" as *
#import "core/mem/raw" as *

enum Wrapper:
    Val <i32>

struct Pair:
    token <Wrapper>
    count <i32>

fn leak <()->&Wrapper> ():
    let p <Pair> Pair (Wrapper::Val 1) 7;
    field::get_ref &p "token"

fn main <()*>()> ():
    let r <&Wrapper> leak;
    ()
"#;
    let errs = compile_move_test(source).unwrap_err();
    assert!(errs.iter().any(is_return_escape_diag));
}
