mod harness;
use harness::run_main_i32;
use nepl_core::loader::Loader;
use nepl_core::typecheck;
use nepl_core::BuildProfile;
use nepl_core::CompileTarget;
use std::path::PathBuf;

fn stdlib_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib")
}

fn typecheck_src_with_target(src: &str, target: CompileTarget) {
    let mut loader = Loader::new(stdlib_root());
    let loaded = loader
        .load_inline(PathBuf::from("overload_regression.nepl"), src.to_string())
        .expect("load");
    let checked = typecheck::typecheck(
        &loaded.module,
        target,
        BuildProfile::Debug,
        Some(&loaded.source_map),
    );
    if !checked.diagnostics.is_empty() {
        panic!("typecheck failure: {:?}", checked.diagnostics);
    }
}

// Note: In NEPL, overloaded functions must have the same number of arguments (arity).
// Overloading is resolved based on the combination of:
// - Function Name
// - Argument Types
// - Return Type

#[test]
fn test_overload_cast_like() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

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
    if:
        res_bool
        then res_i32
        else 0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn test_overload_print_like() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

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
    
    add s1 s2
"#;
    // 1 + 2 = 3
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn test_explicit_type_annotation_prefix() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

// magic: Same input, different return types
fn magic <(i32)->i32> (v):
    add v 1

fn magic <(i32)->bool> (v):
    true

fn main <()*>i32> ():
    // Use <type> prefix expression to explicitly select overload
    // This is useful when type cannot be inferred from context
    
    // Force selection of (i32)->i32
    let v1 <i32> <i32> magic 10
    
    // Force selection of (i32)->bool
    let v2 <bool> <bool> magic 10
    
    if:
        v2
        then v1
        else 0
"#;
    // 10 + 1 = 11, and v2 is true
    let v = run_main_i32(src);
    assert_eq!(v, 11);
}

#[test]
fn grouped_argument_overload_uses_later_items_before_reduction() {
    let src = r#"
#entry main
#indent 4
#no_prelude

struct S:
    tag <()>

fn f <(i32)->i32> (_n):
    1

fn f <(S)->i32> (_s):
    2

fn main <()->i32> ():
    let x <i32> f S
    x
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 2);
}

#[test]
fn unit_like_struct_constructor_is_a_value() {
    let src = r#"
#entry main
#indent 4
#no_prelude

struct A:
    tag <()>

fn use_a <(A)->i32> (_a):
    1

fn main <()->i32> ():
    use_a A
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 1);
}

#[test]
fn grouped_constructor_argument_can_flow_into_generic_new_call() {
    let src = r#"
#target wasm
#indent 4
#no_prelude
#import "core/result" as *
#import "alloc/diag/error" as *
#import "core/traits/hash" as *
#import "alloc/collections/hashmap" as *

fn main <()*>i32> ():
    let hm <HashMap<i32, i32, DefaultHash32>> unwrap_ok new DefaultHash32
    free hm
    0
"#;
    typecheck_src_with_target(src, CompileTarget::Wasm);
}

#[test]
fn more_specific_get_overload_beats_generic_catchall() {
    let src = r#"
#target wasm
#indent 4
#no_prelude
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *
#import "alloc/diag/error" as *
#import "core/traits/hash" as *
#import "alloc/collections/hashmap" as *

fn main <()*>i32> ():
    let hm <HashMap<i32, i32, DefaultHash32>> unwrap_ok new DefaultHash32
    let out <i32> match get &hm 10:
        Option::Some v:
            v
        Option::None:
            0
    free hm
    out
"#;
    typecheck_src_with_target(src, CompileTarget::Wasm);
}

#[test]
fn annotated_let_prefers_specific_get_over_generic_field_get() {
    let src = r#"
#target wasm
#indent 4
#no_prelude
#import "core/option" as *
#import "core/result" as *
#import "alloc/diag/error" as *
#import "core/traits/hash" as *
#import "alloc/collections/hashmap" as *

fn must_hm <(Result<HashMap<i32, i32, DefaultHash32>, Diag>)*>HashMap<i32, i32, DefaultHash32>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let hm <HashMap<i32, i32, DefaultHash32>> must_hm new DefaultHash32
    let got <Option<i32>> get &hm 10
    let out <i32> match got:
        Option::Some v:
            v
        Option::None:
            0
    free hm
    out
"#;
    typecheck_src_with_target(src, CompileTarget::Wasm);
}
