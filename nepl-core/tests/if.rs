mod harness;
use harness::run_main_i32;

#[test]
fn if_a_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let a <i32> if true 0 1;
    a
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn if_b_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let b <i32> if true then 0 else 1;
    b
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn if_c_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let c <i32> if:
        true
        0
        1
    c
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn if_d_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let d <i32> if:
        cond true
        then 0
        else 1
    d
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn if_e_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let e <i32> if:
        true
        then:
            0
        else:
            1
    e
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn if_f_returns_expected() {
    let src = r#"
#entry main
#indent 4
#target wasm

fn main <()->i32> ():
    let f <i32> if true 0 if true 1 2;
    f
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}