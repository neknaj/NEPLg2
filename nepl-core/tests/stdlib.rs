mod harness;
use harness::run_main_i32;

#[test]
fn string_len_literal_returns_3() {
    let src = r#"
#entry main
#indent 4
#import "std/string"
#use std::string::*

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
#import "std/string"
#use std::string::*

fn main <()*>i32> ():
    let s from_i32 1234;
    len s
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 4);
}

#[test]
#[ignore]
fn string_from_to_roundtrip() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "std/string"
#use std::string::*
#import "std/result"
#use std::result::*
#import "std/math"
#use std::math::*

// check roundtrip for a set of representative values
fn check <(i32)*>i32> (x):
    let s <i32> from_i32 x;
    let r <ResultI32> to_i32 s;
    match r:
        Ok v:
            if eq v x 0 1
        Err e:
            1

fn main <()*>i32> ():
    let a <i32> check 0;
    let b <i32> check 5;
    let c <i32> check 42;
    let d <i32> check -7;
    let e <i32> check 2147483647;
    let f <i32> check -2147483648;
    // sum results; expect 0
    add add add add add a b c d e f
"#;

    let v = run_main_i32(src);
    assert_eq!(v, 0);
}
