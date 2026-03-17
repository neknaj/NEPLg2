mod harness;
use harness::run_main_i32;

#[test]
fn test_i32_literals_decimal() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    let a 123;
    let b -45;
    add a b
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 78);
}

#[test]
fn test_i32_literals_hex() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    let a 0x10;      // 16
    let b 0xFF;      // 255
    let c 0x0;       // 0
    add a add b c
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 271);
}

#[test]
fn test_f32_literals() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *
#import "core/cast" as *

fn main <()->i32> ():
    let a 1.5;
    let b -0.5;
    let c 10.0;
    // (1.5 + (-0.5)) * 10.0 = 1.0 * 10.0 = 10.0
    let res <f32> mul (add a b) c;
    cast res
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn test_u8_literals_and_wrapping_add() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *
#import "core/cast" as *

fn main <()->i32> ():
    let a <u8> cast 255;
    let b <u8> cast 1;
    // 255 + 1 should wrap to 0 for u8
    let c <u8> add a b;
    cast c
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn test_u8_wrapping_sub() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *
#import "core/cast" as *

fn main <()->i32> ():
    let a <u8> cast 0;
    let b <u8> cast 1;
    // 0 - 1 should wrap to 255 for u8
    let c <u8> sub a b;
    cast c
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 255);
}

#[test]
fn test_u8_wrapping_mul() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *
#import "core/cast" as *

fn main <()->i32> ():
    let a <u8> cast 16;
    let b <u8> cast 17;
    // 16 * 17 = 272. 272 % 256 = 16
    let c <u8> mul a b;
    cast c
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 16);
}

#[test]
fn test_u8_division_and_remainder() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *
#import "core/cast" as *

fn main <()->i32> ():
    let a <u8> cast 200;
    let b <u8> cast 20;
    let div_res <u8> div_u a b; // 10
    let rem_res <u8> rem_u a b; // 0
    add (cast div_res) (cast rem_res)
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn test_u8_comparisons() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *
#import "core/cast" as *

fn main <()->i32> ():
    let a <u8> cast 10;
    let b <u8> cast 20;
    let c <u8> cast 10;
    let mut score 0;
    if lt_u a b set score add score 1 ();
    if le_u a c set score add score 1 ();
    if gt_u b a set score add score 1 ();
    if ge_u b c set score add score 1 ();
    if eq a c   set score add score 1 ();
    if ne a b   set score add score 1 ();
    score
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 6);
}

#[test]
fn test_bitwise_operations() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    let a 0xC; // 12
    let b 0xA; // 10
    // and: 1000 (8)
    // or:  1110 (14)
    // xor: 0110 (6)
    // 8 + 14 + 6 = 28
    let r_and and a b;
    let r_or  or a b;
    let r_xor xor a b;
    add r_and add r_or r_xor
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 28);
}

#[test]
fn test_shift_operations() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    let a 8;
    let b -16;
    // shl 8 1 -> 16
    // shr_s -16 2 -> -4
    // shr_u 8 1 -> 4
    // 16 + (-4) + 4 = 16
    let r_shl shl a 1;
    let r_shr_s shr_s b 2;
    let r_shr_u shr_u a 1;
    add r_shl add r_shr_s r_shr_u
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 16);
}

#[test]
fn test_f32_comparisons() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    let mut score 0;
    if lt 1.0 2.0 set score add score 1 ();
    if le 2.0 2.0 set score add score 1 ();
    if gt 3.0 2.0 set score add score 1 ();
    if ge 3.0 3.0 set score add score 1 ();
    if eq 4.0 4.0 set score add score 1 ();
    if ne 4.0 5.0 set score add score 1 ();
    score
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 6);
}
