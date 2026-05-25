mod harness;
use harness::run_main_i32;
use nepl_core::loader::Loader;
use nepl_core::{compile_module_with_source_map, CompileOptions, CompileTarget};
use std::fs;
use std::path::PathBuf;

// plan.md より:
// 型注釈 `<T>` は何もしない関数 `(.T)->.T` と見做され、型推論で関数と纏めて処理される。
// つまり、型注釈は続く「式」に対して、関数呼び出しと同じように振舞う。

#[test]
fn test_neplg21_percent_type_annot_basic() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/math" as *

fn main %fn () i32 \():
    let a %i32 add %i32 10 %i32 20
    a
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 30);
}

#[test]
fn test_neplg21_prefix_generic_type_annot() {
    let src = r#"
#entry main
#indent 4
#target wasm
#import "core/option" as *

fn main %fn () i32 \():
    let opt %Option i32 some<i32> 42
    match opt:
        Option::Some v:
            v
        Option::None:
            0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 42);
}

#[test]
fn test_neplg21_prefix_generic_type_annot_forward_declared_user_type() {
    let src = r#"
#entry main
#indent 4
#target wasm
#no_prelude

fn make_later %fn () Later i32 \():
    Later 42

struct Later<.T>:
    value %.T

fn main %fn () i32 \():
    let later %Later i32 make_later
    42
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 42);
}

#[test]
fn test_neplg21_prefix_generic_type_annot_imported_user_type() {
    let dir = tempfile::tempdir().expect("create temporary NEPL module directory");
    let imported_path = dir.path().join("generic_box.nepl");
    fs::write(
        &imported_path,
        r#"
#indent 4
#no_prelude

pub struct ImportedBox<.T>:
    value %.T
"#,
    )
    .expect("write imported generic type module");

    let main_path = dir.path().join("main.nepl");
    fs::write(
        &main_path,
        r#"
#entry main
#indent 4
#target wasm
#no_prelude
#import "./generic_box" as *

fn main %fn () i32 \():
    let boxed %ImportedBox i32 ImportedBox 77
    77
"#,
    )
    .expect("write entry module using imported generic type");

    let mut loader = Loader::new(test_stdlib_root());
    let loaded = loader.load(&main_path).expect("load imported generic type");
    compile_module_with_source_map(
        loaded.module,
        Some(&loaded.source_map),
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    )
    .expect("compile imported generic type");
}

#[test]
fn test_neplg21_prefix_type_arity_preload_reads_cyclic_facade_exports() {
    let dir = tempfile::tempdir().expect("create temporary NEPL entry directory");
    fs::write(
        dir.path().join("facade.nepl"),
        r#"
#indent 4
#no_prelude

pub #import "./types" as @merge
pub #import "./cycle" as @merge
"#,
    )
    .expect("write cyclic facade module");

    fs::write(
        dir.path().join("types.nepl"),
        r#"
#indent 4
#no_prelude

pub struct Vec<.T>:
    value %.T
"#,
    )
    .expect("write facade type module");

    fs::write(
        dir.path().join("cycle.nepl"),
        r#"
#indent 4
#no_prelude
#import "./facade" as v

pub struct Diag:
    items %Vec i32
"#,
    )
    .expect("write module that needs facade arity while the facade is processing");

    let main_path = dir.path().join("main.nepl");
    fs::write(
        &main_path,
        r#"
#entry main
#indent 4
#target wasm
#no_prelude
#import "./facade" as *

fn main %fn () i32 \():
    0
"#,
    )
    .expect("write entry module importing cyclic facade");

    let mut loader = Loader::new(test_stdlib_root());
    loader
        .load(&main_path)
        .expect("load cyclic facade without losing prefix type arity hints");
}

#[test]
fn test_type_annot_basic() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    // 基本的なリテラルへの型注釈
    // 式 `123` は i32
    // `<i32>` を前置しても値は変わらず、型がチェックされる
    let a <i32> 123

    // 式の結果をそのまま返す
    a
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 123);
}

#[test]
fn test_type_annot_nested_expr() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    // 計算式全体への型注釈
    // add 10 20 は i32 を返す
    let a <i32> add 10 20

    // 部分式への型注釈も可能
    // `<i32> 10` も `<i32> 20` もただの i32 として振る舞う
    let b add <i32> 10 <i32> 20

    add a b
"#;
    // 30 + 30 = 60
    let v = run_main_i32(src);
    assert_eq!(v, 60);
}

#[test]
fn test_type_annot_on_let() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    // plan.md 94行目の例: let mut neg <bool> lt n 0
    // let 宣言の右辺式全体に対する型注釈

    let n 10

    // `<bool>` は `lt n 0` という式にかかる
    let neg <bool> lt n 0

    if:
        neg
        then 1
        else 0
"#;
    // 10 < 0 は false なので else 0
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn test_type_annot_block() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    // ブロック式全体への型注釈
    // ブロックの評価結果（最後の式の値）に対して型注釈がかかる

    let v <i32> block:
        let x 1
        let y 2
        add x y

    v
"#;
    // 1 + 2 = 3
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn test_type_annot_nested_annot() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    // 型注釈は重ねても良い
    // <i32> (<i32> 100) -> 100

    let v <i32> <i32> 100
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 100);
}

#[test]
fn test_type_annot_function_call() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn id <(i32)->i32> (x):
    x

fn main <()*>i32> ():
    // 関数適用の結果に対する型注釈
    // id 123 は i32 を返すので <i32> で注釈可能

    let v <i32> id 123
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 123);
}

#[test]
fn test_type_annot_complex_expr() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    // 複雑な式の中での型注釈
    // add (mul <i32> 2 3) (<i32> 4)

    let v <i32> add mul <i32> 2 3 <i32> 4
    v
"#;
    // (2*3) + 4 = 10
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn test_type_annot_if_expr() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    // if式全体、あるいは各ブランチへの型注釈

    let v <i32> if:
        <bool> true
        then <i32> 10
        else <i32> 20
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

#[test]
fn test_type_annot_while_condition() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    let mut i 0
    let mut sum 0

    // while の条件式に型注釈
    while <bool> lt i 3:
        do:
            set sum add sum i
            set i add i <i32> 1

    sum
"#;
    // 0 + 1 + 2 = 3
    let v = run_main_i32(src);
    assert_eq!(v, 3);
}

#[test]
fn test_type_annot_generic_like() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *
#import "core/option" as *

fn main <()*>i32> ():
    // ジェネリック型に対する型注釈
    // Option<i32> 型の値を生成し、それに型注釈をつける

    let opt <Option<i32>> some<i32> 42

    match opt:
        Option::Some v:
            v
        Option::None:
            0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 42);
}

#[test]
fn test_type_annot_deeply_nested() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    // 深くネストされた関数呼び出しと型注釈
    // add( add( <i32>1, <i32>2 ), <i32>3 )

    let v <i32> add <i32> add <i32> 1 <i32> 2 <i32> 3
    v
"#;
    // (1+2)+3 = 6
    let v = run_main_i32(src);
    assert_eq!(v, 6);
}

#[test]
fn test_type_annot_mixed_with_blocks() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()*>i32> ():
    // ブロックとインラインの混在

    let v <i32> add: // 関数の引数で改行しているのは正しい インデントは各引数の先頭が+1で揃う
        <i32> block: // 型注釈付きの無名ブロックも正しい ブロックなので返り値はx
            let x 10
            x
        <i32> 20
    v
"#;
    // 10 + 20 = 30
    let v = run_main_i32(src);
    assert_eq!(v, 30);
}

fn test_stdlib_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib")
}
