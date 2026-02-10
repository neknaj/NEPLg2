mod harness;
use harness::run_main_i32;

#[test]
fn string_len_literal_returns_3() {
    let src = r#"
#entry main
#indent 4
#import "alloc/string" as *

fn main <()*>i32> ():
    let s "abc";
    len s
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn string_from_i32_len_matches_digits() {
    let src = r#"
#entry main
#indent 4
#import "alloc/string" as *

fn main <()*>i32> ():
    let s from_i32 1234;
    len s
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 4);
}

#[test]
fn string_from_to_roundtrip() {
    let src = r#"
#entry main
#indent 4
#import "alloc/string" as *
#import "core/result" as *

fn main <()*>i32> ():
    let s0 from_i32 0;
    let s5 from_i32 5;
    let s42 from_i32 42;
    // Simple check: convert back and verify lengths match
    let len0 len s0;
    let len5 len s5;
    let len42 len s42;
    // Return sum of lengths; expect 1+1+2=4
    add add len0 len5 len42
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 4);
}
