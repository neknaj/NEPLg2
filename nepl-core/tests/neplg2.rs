use nepl_core::diagnostic_ids::DiagnosticId;
use nepl_core::error::CoreError;
use nepl_core::loader::Loader;
use nepl_core::span::FileId;
use nepl_core::{compile_wasm, BuildProfile, CompileOptions, CompileTarget};
mod harness;
use harness::{compile_src_with_options, run_main_i32, run_main_wasi_i32};

fn compile_ok(src: &str) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    assert!(result.is_ok(), "expected success, got {:?}", result);
}

fn compile_err(src: &str) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    assert!(result.is_err(), "expected error, got {:?}", result);
}

fn compile_ok_target(src: &str, target: CompileTarget) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(target),
            verbose: false,
            profile: None,
        },
    );
    assert!(result.is_ok(), "expected success, got {:?}", result);
}

fn compile_err_target(src: &str, target: CompileTarget) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(target),
            verbose: false,
            profile: None,
        },
    );
    assert!(result.is_err(), "expected error, got {:?}", result);
}

fn compile_ok_profile(src: &str, profile: BuildProfile) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: Some(profile),
        },
    );
    assert!(result.is_ok(), "expected success, got {:?}", result);
}

fn compile_err_profile(src: &str, profile: BuildProfile) {
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: Some(profile),
        },
    );
    assert!(result.is_err(), "expected error, got {:?}", result);
}

fn load_inline_with_stdlib(src: &str) -> nepl_core::loader::LoadResult {
    let stdlib_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib");
    let mut loader = Loader::new(stdlib_root);
    loader
        .load_inline(std::path::PathBuf::from("test.nepl"), src.to_string())
        .expect("load")
}

#[test]
fn llvm_mem_bulk_copy_stdlib_lowers_to_intrinsics() {
    let src = r#"
#entry main
#indent 4
#target llvm

#import "core/mem" as *

fn main <()->i32> ():
    mem_copy 16 24 4
    mem_move 32 16 4
    0
"#;
    let loaded = load_inline_with_stdlib(src);
    let ll = nepl_core::codegen_llvm::emit_ll_from_module_for_target(
        &loaded.module,
        CompileTarget::Llvm,
        BuildProfile::Debug,
        false,
    )
    .expect("stdlib mem bulk copy should emit LLVM IR without clang");
    assert!(ll.contains("declare void @llvm.memcpy.p0.p0.i32"));
    assert!(ll.contains("declare void @llvm.memmove.p0.p0.i32"));
    assert!(ll.contains("define void @mem_copy(i32 %dst, i32 %src, i32 %len)"));
    assert!(ll.contains("define void @mem_move(i32 %dst, i32 %src, i32 %len)"));
    assert!(ll.contains("call void @llvm.memcpy.p0.p0.i32"));
    assert!(ll.contains("call void @llvm.memmove.p0.p0.i32"));
}

#[test]
fn llvm_match_i32_literal_lowers_to_switch() {
    let src = r#"
#entry main
#indent 4
#target llvm

fn main <()->i32> ():
    let x <i32> 92
    match x:
        34:
            1
        92:
            2
        _:
            3
"#;
    let loaded = load_inline_with_stdlib(src);
    let ll = nepl_core::codegen_llvm::emit_ll_from_module_for_target(
        &loaded.module,
        CompileTarget::Llvm,
        BuildProfile::Debug,
        false,
    )
    .expect("i32 literal match should emit LLVM IR");
    assert!(ll.contains("switch i32"));
    assert!(ll.contains("i32 92, label"));
}

#[test]
fn compiles_literal_main() {
    let src = r#"
#entry main
fn main <() -> i32> ():
    #import "core/math" as *
    1
"#;
    compile_ok(src);
}

#[test]
fn compiles_add_block_expression() {
    let src = r#"
#entry main
#indent 4

#if[target=wasm]
fn add <(i32, i32) -> i32> (a, b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <() -> i32> ():
    #import "core/math" as *
    add 1:
        add 2 3
"#;
    compile_ok(src);
}

#[test]
fn set_type_mismatch_is_error() {
    let src = r#"
#entry main
fn main <() -> ()> ():
    let mut x <i32> 0;
    set x ();
"#;
    compile_err(src);
}

#[test]
fn pure_cannot_call_impure() {
    let src = r#"
#entry main
#indent 4

fn imp <(i32) *> i32> (x):
    #import "core/math" as *
    add x 1

fn pure <(i32) -> i32> (x):
    imp x

fn main <() -> i32> ():
    pure 1
"#;
    compile_err(src);
}

#[test]
fn iftarget_non_wasm_is_skipped() {
    let src = r#"
#entry main

#if[target=llvm]
fn bad <() -> i32> ():
    unknown_symbol

fn main <() -> i32> ():
    1
"#;
    compile_ok(src);
}

#[test]
fn invalid_iftarget_is_diagnostic() {
    let src = r#"
#entry main

#if[target=unknown_target]
fn bad <() -> i32> ():
    unknown_symbol

fn main <() -> i32> ():
    1
"#;
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    let CoreError::Diagnostics(diags) = result.expect_err("unknown target gate should fail") else {
        panic!("expected diagnostics");
    };
    assert!(
        diags
            .iter()
            .any(|diag| diag.id == Some(DiagnosticId::InvalidConditionalGate)),
        "missing invalid conditional gate diagnostic: {:?}",
        diags
    );
}

#[test]
fn invalid_ifprofile_is_diagnostic() {
    let src = r#"
#entry main

#if[profile=staging]
fn bad <() -> i32> ():
    unknown_symbol

fn main <() -> i32> ():
    1
"#;
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: Some(BuildProfile::Debug),
        },
    );
    let CoreError::Diagnostics(diags) = result.expect_err("unknown profile gate should fail")
    else {
        panic!("expected diagnostics");
    };
    assert!(
        diags
            .iter()
            .any(|diag| diag.id == Some(DiagnosticId::InvalidConditionalGate)),
        "missing invalid conditional gate diagnostic: {:?}",
        diags
    );
}

#[test]
fn invalid_iftarget_in_nested_block_is_diagnostic() {
    let src = r#"
#entry main
#indent 4

fn main <() -> i32> ():
    if true:
        then:
            #if[target=unknown_target]
            unknown_symbol
            1
        else:
            0
"#;
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    let CoreError::Diagnostics(diags) = result.expect_err("nested unknown target gate should fail")
    else {
        panic!("expected diagnostics");
    };
    assert!(
        diags
            .iter()
            .any(|diag| diag.id == Some(DiagnosticId::InvalidConditionalGate)),
        "missing invalid conditional gate diagnostic: {:?}",
        diags
    );
}

#[test]
fn ifprofile_debug_gate() {
    let src = r#"
#entry main

#if[profile=debug]
fn only_debug <() -> i32> ():
    123

fn main <() -> i32> ():
    only_debug
"#;
    compile_ok_profile(src, BuildProfile::Debug);
    compile_err_profile(src, BuildProfile::Release);
}

#[test]
fn ifprofile_release_skips_in_debug() {
    let src = r#"
#entry main

#if[profile=release]
fn only_release <() -> i32> ():
    unknown_symbol

fn main <() -> i32> ():
    0
"#;
    compile_ok_profile(src, BuildProfile::Debug);
    compile_err_profile(src, BuildProfile::Release);
}

#[test]
fn wasm_stack_mismatch_is_error() {
    let src = r#"
#entry main

#if[target=wasm]
fn add_one <(i32)->i32> (a):
    #wasm:
        local.get $a
        // missing value for add
        i32.add

fn main <() -> i32> ():
    #import "core/math" as *
    add_one 1
"#;
    compile_err(src);
}

#[test]
fn wasi_allows_wasm_gate() {
    let src = r#"
#entry main

#if[target=wasm]
fn only_wasm <() -> i32> ():
    123

fn main <() -> i32> ():
    only_wasm
"#;
    compile_ok_target(src, CompileTarget::Wasi);
}

#[test]
fn wasm_skips_wasi_gate() {
    let src = r#"
#entry main

#if[target=wasi]
fn only_wasi <() -> i32> ():
    unknown_symbol

fn main <() -> i32> ():
    0
"#;
    compile_ok_target(src, CompileTarget::Wasm);
    compile_err_target(src, CompileTarget::Wasi);
}

#[test]
fn import_and_prelude_directives_are_accepted() {
    let src = r#"
#entry main
#prelude std/prelude_base
#no_prelude
#import "core/math" as { add as plus, math::* }
#import "./part" as @merge

fn main <() -> i32> ():
    0
"#;
    compile_ok(src);
}

#[test]
fn string_literal_compiles() {
    let src = r#"
#entry main
#indent 4
#extern "env" "print_str" fn print <(str)*>()>

fn main <()*> ()> ():
    print "hello";
    ()
"#;
    compile_ok(src);
}

#[test]
fn pipe_injects_first_arg() {
    let src = r#"
#entry main
#indent 4

#if[target=wasm]
fn add <(i32,i32)->i32> (a,b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <()->i32> ():
    add 1 add 2 3 |> add 4
"#;
    compile_ok(src);
}

#[test]
fn pipe_requires_callable_target() {
    let src = r#"
#entry main
#indent 4

fn main <()->i32> ():
    1 |> 2
"#;
    compile_err(src);
}

#[test]
fn pipe_with_type_annotation_is_ok() {
    let src = r#"
#entry main
#indent 4

#if[target=wasm]
fn add <(i32,i32)->i32> (a,b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <()->i32> ():
    1 |> <i32> add 4
"#;
    compile_ok(src);
}

#[test]
fn pipe_with_double_type_annotation_is_ok() {
    let src = r#"
#entry main
#indent 4

#if[target=wasm]
fn add <(i32,i32)->i32> (a,b):
    #wasm:
        local.get $a
        local.get $b
        i32.add

fn main <()->i32> ():
    1 |> <i32> <i32> add 4
"#;
    compile_ok(src);
}

#[test]
fn pipe_target_missing_after_annotation_is_error() {
    let src = r#"
#entry main
#indent 4

fn main <()->i32> ():
    1 |> <i32> 2
"#;
    compile_err(src);
}

#[test]
fn wasi_import_rejected_on_wasm_target() {
    let src = r#"
#entry main
#indent 4
#extern "wasi_snapshot_preview1" "fd_write" fn fd_write <(i32,i32,i32,i32)->i32>
fn main <()->()> ():
    ()
"#;
    compile_err_target(src, CompileTarget::Wasm);
}

#[test]
fn name_conflict_enum_fn_is_error() {
    let src = r#"
#entry main
#indent 4

enum Foo:
    A

fn Foo <()->i32> ():
    0

fn main <()->i32> ():
    Foo
"#;
    compile_err(src);
}

#[test]
fn wasm_cannot_use_stdio() {
    let src = r#"
#entry main
#indent 4
#import "std/stdio" as *

fn main <()->()> ():
    print "hi"
"#;
    compile_err_target(src, CompileTarget::Wasm);
}

#[test]
fn run_add_returns_12() {
    let src = r#"
#entry main
#indent 4
#import "core/math" as *

fn main <()->i32> ():
    add 10 2
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 12);
}

#[test]
fn match_option_some_returns_value() {
    let src = r#"
#entry main
#indent 4
#import "core/option" as *

fn main <()* >i32> ():
    match some 5:
        Some v:
            v
        None:
            0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 5);
}

#[test]
fn list_get_out_of_bounds_err() {
    let src = r#"
#entry main
#indent 4
#import "alloc/collections/list" as *
#import "core/option" as *
#import "core/result" as *

fn main <()* >i32> ():
    let lst <List<i32>> unwrap_ok<List<i32>, Diag> new<i32>;
    let lst uwok cons<i32> 1 lst;
    let r get<i32> lst 10;
    match r:
        Some v:
            v
        None:
            0
"#;
    let v = run_main_wasi_i32(src);
    assert_eq!(v, 0);
}

#[test]
fn non_exhaustive_match_is_error() {
    let src = r#"
#entry main
#indent 4
#import "core/option" as *

fn main <()->i32> ():
    match some 1:
        Some v:
            v
"#;
    compile_err(src);
}

#[test]
fn target_directive_sets_default_to_wasi() {
    let src = r#"
#target wasi
#entry main
#indent 4
#import "std/stdio" as *

fn main <()* >()> ():
    print "ok"
"#;
    let wasm = compile_src_with_options(
        src,
        CompileOptions {
            target: None,
            verbose: false,
            profile: None,
        },
    );
    assert!(!wasm.is_empty());
}

#[test]
fn duplicate_target_directive_is_error() {
    let src = r#"
#target wasm
#target wasi
#entry main
fn main <()->i32> ():
    0
"#;
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: None,
            verbose: false,
            profile: None,
        },
    );
    assert!(result.is_err(), "expected error, got {:?}", result);
}

#[test]
fn overloads_by_param_type_are_allowed() {
    let src = r#"
#entry main
#indent 4

fn id <(i32)->i32> (x):
    x

fn id <(f32)->f32> (x):
    x

fn main <()->i32> ():
    let tmp id 1.0;
    id 1
"#;
    compile_ok(src);
}

#[test]
fn overloads_with_different_arity_are_error() {
    let src = r#"
#entry main
#indent 4

fn foo <(i32)->i32> (x):
    x

fn foo <(i32,i32)->i32> (a,b):
    a

fn main <()->i32> ():
    foo 1
"#;
    compile_err(src);
}

#[test]
fn overloads_ambiguous_return_type_is_error() {
    let src = r#"
#entry main
#indent 4

fn foo <(i32)->i32> (x):
    x

fn foo <(i32)->f32> (x):
    1.0

fn main <()->i32> ():
    let y foo 1;
    0
"#;
    compile_err(src);
}

#[test]
fn trait_method_call_with_impl_compiles() {
    let src = r#"
#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

impl Show for i32:
    fn show <(i32)->i32> (x):
        x

fn main <()->i32> ():
    Show::show 1
"#;
    compile_ok(src);
}

#[test]
fn trait_bound_satisfied_in_generic() {
    let src = r#"
#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

impl Show for i32:
    fn show <(i32)->i32> (x):
        x

fn call_show <.T: Show> <(.T)->i32> (x):
    Show::show x

fn main <()->i32> ():
    call_show 5
"#;
    compile_ok(src);
}

#[test]
fn generic_trait_impl_method_resolves_by_trait_args() {
    let src = r#"
#entry main
#indent 4

trait HashKey:
    #capability clone
    #capability copy
    fn clone <(Self)->Self> (self):
        self

    fn eq <(Self,Self)->bool> (a, b):
        eq a b

    fn hash32 <(Self)->i32> (self):
        0

impl HashKey for i32:
    fn clone <(i32)->i32> (self):
        self

    fn eq <(i32,i32)->bool> (a, b):
        eq a b

    fn hash32 <(i32)->i32> (self):
        self

trait Hasher<.K: HashKey>:
    #capability clone
    #capability copy
    fn hash32 <(Self,.K)->i32> (self, key):
        0

struct DefaultHash32:
    tag <()>

impl<.K: HashKey> Hasher<.K> for DefaultHash32:
    fn hash32 <(DefaultHash32,.K)->i32> (_self, key):
        HashKey::hash32 key

fn hash_with <.K: HashKey,.H: Hasher<.K>> <(.H,.K)->i32> (hasher, key):
    Hasher::hash32 hasher key

fn main <()->i32> ():
    hash_with DefaultHash32 9
"#;
    assert_eq!(run_main_i32(src), 9);
}

#[test]
fn generic_intrinsic_store_load_struct_preserves_fields() {
    let src = r#"
#entry main
#indent 4
#target std
#import "core/field" as field
#import "core/math" as *
#import "core/mem" as *

struct Point:
    x <i32>
    y <i32>

fn roundtrip <.T> <(.T)->.T> (x):
    let p <i32> alloc_raw size_of<.T>;
    store<.T> p x;
    load<.T> p

fn main <()*>i32> ():
    let p <Point> roundtrip<Point> Point 10 20;
    add mul field::get p "x" 100 field::get p "y"
"#;
    assert_eq!(run_main_wasi_i32(src), 1020);
}

#[test]
fn generic_hashkey_eq_after_load_uses_concrete_impl() {
    let src = r#"
#entry main
#indent 4
#target std
#import "core/field" as field
#import "core/math" as *
#import "core/mem" as *
#import "core/traits/hash_key" as *

struct Point:
    x <i32>
    y <i32>

impl HashKey for Point:
    fn clone <(Point)->Point> (self):
        self

    fn eq <(Point,Point)->bool> (a, b):
        let ax <i32> field::get a "x"
        let ay <i32> field::get a "y"
        let bx <i32> field::get b "x"
        let by <i32> field::get b "y"
        and (eq ax bx) (eq ay by)

    fn hash32 <(Point)->i32> (self):
        xor field::get self "x" field::get self "y"

fn same_after_store <.T: HashKey> <(.T,.T)->bool> (a, b):
    let p <i32> alloc_raw size_of<.T>;
    store<.T> p a;
    let saved <.T> load<.T> p;
    hashkey_eq saved b

fn main <()*>i32> ():
    if same_after_store<Point> (Point 10 20) (Point 10 20) 1 0
"#;
    assert_eq!(run_main_wasi_i32(src), 1);
}

#[test]
fn generic_hashkey_value_survives_hash_before_store() {
    let src = r#"
#entry main
#indent 4
#target std
#import "core/field" as field
#import "core/math" as *
#import "core/mem" as *
#import "core/traits/hash_key" as *

struct Point:
    x <i32>
    y <i32>

impl HashKey for Point:
    fn clone <(Point)->Point> (self):
        self

    fn eq <(Point,Point)->bool> (a, b):
        let ax <i32> field::get a "x"
        let ay <i32> field::get a "y"
        let bx <i32> field::get b "x"
        let by <i32> field::get b "y"
        and (eq ax bx) (eq ay by)

    fn hash32 <(Point)->i32> (self):
        xor field::get self "x" field::get self "y"

fn hash_then_store <.T: HashKey> <(.T)->.T> (x):
    let _h <i32> hashkey_hash32 x;
    let p <i32> alloc_raw size_of<.T>;
    store<.T> p x;
    load<.T> p

fn main <()*>i32> ():
    let p <Point> hash_then_store<Point> Point 10 20;
    add mul field::get p "x" 100 field::get p "y"
"#;
    assert_eq!(run_main_wasi_i32(src), 1020);
}

#[test]
fn hashmap_custom_struct_key_roundtrips_value() {
    let src = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/hashmap" as *
#import "alloc/diag/error" as *
#import "core/field" as field
#import "core/option" as *
#import "core/result" as *
#import "core/traits/hash" as *
#import "core/traits/hash_key" as *

struct Point:
    x <i32>
    y <i32>

impl HashKey for Point:
    fn clone <(Point)->Point> (self):
        self

    fn eq <(Point,Point)->bool> (a, b):
        let ax <i32> field::get a "x"
        let ay <i32> field::get a "y"
        let bx <i32> field::get b "x"
        let by <i32> field::get b "y"
        and (eq ax bx) (eq ay by)

    fn hash32 <(Point)->i32> (self):
        xor field::get self "x" field::get self "y"

fn must_hmp <(Result<HashMap<Point,i32,DefaultHash32>, Diag>)*>HashMap<Point,i32,DefaultHash32>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let map0 <HashMap<Point,i32,DefaultHash32>> must_hmp new DefaultHash32;
    let map1 <HashMap<Point,i32,DefaultHash32>> must_hmp insert map0 (Point 10 20) 99;
    match get map1 (Point 10 20):
        Option::Some n:
            n
        Option::None:
            0
"#;
    assert_eq!(run_main_wasi_i32(src), 99);
}

#[test]
fn result_i64_wildcard_match_does_not_reuse_arm_bind_local() {
    let src = r#"
#target wasm
#entry main
#indent 4
#import "core/result" as *

fn main <()->i32> ():
    let r <Result<i64,i32>> Result<i64,i32>::Err 1;
    let ok <bool> match r:
        Result::Ok _:
            false
        Result::Err _:
            true
    if ok 1 0
"#;
    assert_eq!(run_main_i32(src), 1);
}

#[test]
fn match_i32_literal_arm_returns_selected_case() {
    let src = r#"
#target wasm
#entry main
#indent 4

fn classify <(i32)->i32> (x):
    match x:
        34:
            1
        92:
            2
        _:
            3

fn main <()->i32> ():
    classify 92
"#;
    assert_eq!(run_main_i32(src), 2);
}

#[test]
fn match_i32_literal_wildcard_returns_default_case() {
    let src = r#"
#target wasm
#entry main
#indent 4

fn classify <(i32)->i32> (x):
    match x:
        34:
            1
        92:
            2
        _:
            3

fn main <()->i32> ():
    classify 7
"#;
    assert_eq!(run_main_i32(src), 3);
}

#[test]
fn match_bool_literal_arms_return_selected_case() {
    let src = r#"
#target wasm
#entry main
#indent 4

fn classify <(bool)->i32> (flag):
    match flag:
        true:
            1
        false:
            2

fn main <()->i32> ():
    classify false
"#;
    assert_eq!(run_main_i32(src), 2);
}

#[test]
fn match_i32_duplicate_literal_is_error() {
    let src = r#"
#target wasm
#entry main
#indent 4

fn main <()->i32> ():
    let x <i32> 1
    match x:
        1:
            10
        1:
            20
        _:
            0
"#;
    compile_err_profile(src, BuildProfile::Debug);
}

#[test]
fn match_i32_literal_without_wildcard_is_non_exhaustive() {
    let src = r#"
#target wasm
#entry main
#indent 4

fn main <()->i32> ():
    let x <i32> 1
    match x:
        1:
            10
        2:
            20
"#;
    compile_err_profile(src, BuildProfile::Debug);
}

#[test]
fn generic_store_after_generic_trait_probe_preserves_struct() {
    let src = r#"
#entry main
#indent 4
#target std
#import "core/field" as field
#import "core/math" as *
#import "core/mem" as *
#import "core/traits/hash_key" as *

struct Point:
    x <i32>
    y <i32>

impl HashKey for Point:
    fn clone <(Point)->Point> (self):
        self

    fn eq <(Point,Point)->bool> (a, b):
        let ax <i32> field::get a "x"
        let ay <i32> field::get a "y"
        let bx <i32> field::get b "x"
        let by <i32> field::get b "y"
        and (eq ax bx) (eq ay by)

    fn hash32 <(Point)->i32> (self):
        xor field::get self "x" field::get self "y"

fn probe <.T: HashKey> <(.T)->bool> (key):
    hashkey_eq key key

fn write_after_probe <.T: HashKey,.V> <(.T,.V)->.T> (key, value):
    let _ok <bool> probe<.T> key;
    let p <i32> alloc_raw add size_of<.T> size_of<.V>;
    store<.T> p key;
    store<.V> add p size_of<.T> value;
    load<.T> p

fn main <()*>i32> ():
    let p <Point> write_after_probe<Point,i32> (Point 10 20) 99;
    add mul field::get p "x" 100 field::get p "y"
"#;
    assert_eq!(run_main_wasi_i32(src), 1020);
}

#[test]
fn generic_store_uses_nested_address_call_without_stealing_value_arg() {
    let src = r#"
#entry main
#indent 4
#target std
#import "core/field" as field
#import "core/math" as *
#import "core/mem" as *

struct Point:
    x <i32>
    y <i32>

fn slot_ptr <.T,.V> <(i32,i32)->i32> (base, idx):
    add base mul idx add size_of<.T> size_of<.V>

fn write_nested <.T,.V> <(.T,.V)->.T> (key, value):
    let p <i32> alloc_raw add size_of<.T> size_of<.V>;
    store<.T> slot_ptr<.T,.V> p 0 key;
    store<.V> add p size_of<.T> value;
    load<.T> p

fn main <()*>i32> ():
    let p <Point> write_nested<Point,i32> (Point 10 20) 99;
    add mul field::get p "x" 100 field::get p "y"
"#;
    assert_eq!(run_main_wasi_i32(src), 1020);
}

#[test]
fn trait_bound_missing_impl_is_error() {
    let src = r#"
#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

fn call_show <.T: Show> <(.T)->i32> (x):
    Show::show x

fn main <()->i32> ():
    call_show 1
"#;
    compile_err(src);
}

#[test]
fn impl_generic_target_diagnostic_uses_type_expr_span() {
    let src = r#"
#entry main
#indent 4

trait Marker:
    fn mark <(Self)->i32> (x):
        0

impl Marker for .T:
    fn mark <(.T)->i32> (x):
        0

fn main <()->i32> ():
    0
"#;
    let target_start = src.find(".T").expect("generic impl target") as u32;
    let target_end = target_start + ".T".len() as u32;
    let result = compile_wasm(
        FileId(0),
        src,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    let CoreError::Diagnostics(diags) = result.expect_err("generic impl target should fail") else {
        panic!("expected diagnostics");
    };
    let diag = diags
        .iter()
        .find(|d| d.id == Some(DiagnosticId::TypeImplTargetMustBeConcrete))
        .unwrap_or_else(|| panic!("missing concrete impl target diagnostic: {:?}", diags));
    assert_eq!(diag.primary.span.file_id, FileId(0));
    assert_eq!(diag.primary.span.start, target_start);
    assert_eq!(diag.primary.span.end, target_end);
}

#[test]
fn trait_method_arity_mismatch_is_error() {
    let src = r#"
#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

impl Show for i32:
    fn show <(i32)->i32> (x):
        x

fn main <()->i32> ():
    Show::show 1 2
"#;
    compile_err(src);
}

#[test]
fn unknown_trait_bound_is_error() {
    let src = r#"
#entry main
#indent 4

trait Show:
    fn show <(Self)->i32> (x):
        x

fn call_show <.T: Missing> <(.T)->i32> (x):
    0

fn main <()->i32> ():
    0
"#;
    compile_err(src);
}

#[test]
fn unreachable_does_not_force_never_in_generic() {
    let src = r#"
#entry main
#indent 4

fn pick <.T> <(.T)->.T> (x):
    if:
        true
        then:
            x
        else:
            #intrinsic "unreachable" <> ()

fn main <()->i32> ():
    pick 1
"#;
    compile_ok(src);
}
