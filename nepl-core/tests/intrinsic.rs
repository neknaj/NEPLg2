mod harness;

use harness::run_main_i32;

#[test]
fn intrinsic_size_and_align_direct() {
    let src = r#"
#target wasm
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *

fn main <()->i32> ():
    let s_i64 <i32> size_of<i64>;
    let a_i64 <i32> align_of<i64>;
    let s_f64 <i32> size_of<f64>;
    let a_f64 <i32> align_of<f64>;
    if:
        and eq s_i64 8 and eq a_i64 8 and eq s_f64 8 eq a_f64 8
        then:
            0
        else:
            1
"#;
    assert_eq!(run_main_i32(src), 0);
}

#[test]
fn intrinsic_load_store_i64() {
    let src = r#"
#target wasm
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/cast" as *

fn main <()->i32> ():
    let p <i32> alloc_raw 8;
    let a <i64> cast 12345;
    let b <i64> cast 67890;
    let v <i64> add a b;
    store<i64> p v;
    let got <i64> load<i64> p;
    dealloc_raw p 8;
    if eq got v 0 1
"#;
    assert_eq!(run_main_i32(src), 0);
}

#[test]
fn intrinsic_load_store_f64() {
    let src = r#"
#target wasm
#entry main
#indent 4
#import "core/math" as *
#import "core/mem" as *
#import "core/mem/allocator" as *
#import "core/mem/raw" as *
#import "core/cast" as *

fn main <()->i32> ():
    let p <i32> alloc_raw 8;
    let v <f64> cast 42;
    store<f64> p v;
    let got <f64> load<f64> p;
    dealloc_raw p 8;
    if eq got v 0 1
"#;
    assert_eq!(run_main_i32(src), 0);
}

#[test]
fn intrinsic_load_store_unit_no_stack_leak() {
    let src = r#"
#target wasm
#entry main
#indent 4
#import "core/result" as *

fn main <()->i32> ():
    let r <Result<(), str>> Result<(), str>::Ok ();
    match r:
        Result::Ok _u:
            0
        Result::Err _e:
            1
"#;
    assert_eq!(run_main_i32(src), 0);
}
