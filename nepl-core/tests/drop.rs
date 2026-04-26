use nepl_core::diagnostic::Diagnostic;
use nepl_core::loader::Loader;
use nepl_core::{compile_module, CompileOptions, CompileTarget};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use wasmi::{Engine, Linker, Module, Store};

mod harness;

fn stdlib_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib")
}

fn compile_drop_test(source: &str) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let mut loader = Loader::new(stdlib_root());
    let loaded = loader
        .load_inline(PathBuf::from("drop_test.nepl"), source.to_string())
        .expect("load");
    match compile_module(
        loaded.module,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    ) {
        Ok(artifact) => Ok(artifact.wasm),
        Err(nepl_core::error::CoreError::Diagnostics(ds)) => Err(ds),
        Err(other) => panic!("unexpected compile error: {other:?}"),
    }
}

fn run_drop_trace(source: &str) -> Vec<i32> {
    let wasm = harness::compile_src_with_options(
        source,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
        },
    );
    let engine = Engine::default();
    let module = Module::new(&engine, &*wasm).expect("module");
    let trace = Arc::new(Mutex::new(Vec::<i32>::new()));
    let mut linker = Linker::new(&engine);
    let host_trace = Arc::clone(&trace);
    linker
        .func_wrap("env", "tick", move |value: i32| {
            host_trace.lock().unwrap().push(value);
        })
        .unwrap();
    let mut store = Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &module)
        .and_then(|pre| pre.start(&mut store))
        .expect("instantiate");
    if let Ok(main) = instance.get_typed_func::<(), i32>(&store, "main") {
        let _ = main.call(&mut store, ()).expect("call");
    } else if let Ok(main) = instance.get_typed_func::<(), ()>(&store, "main") {
        main.call(&mut store, ()).expect("call");
    } else {
        panic!("main not found");
    }
    let out = trace.lock().unwrap().clone();
    out
}

#[test]
fn drop_capability_parses_and_compiles() {
    let source = r#"
#target wasm
#indent 4
#entry main
#no_prelude
#import "core/traits/drop" as *

struct Guard:
    id <i32>

impl Drop for Guard:
    fn drop <(&Guard)*>()> (self):
        ()

fn main <()->i32> ():
    let g <Guard> Guard 1;
    0
"#;
    let artifact = compile_drop_test(source).expect("drop trait should compile");
    assert!(!artifact.is_empty(), "generated wasm should not be empty");
}

#[test]
fn auto_drop_runs_at_scope_end() {
    let source = r#"
#target wasm
#indent 4
#entry main
#no_prelude
#import "core/traits/drop" as *
#extern "env" "tick" fn tick <(i32)*>()>

struct Guard:
    dummy <i32>

impl Drop for Guard:
    fn drop <(&Guard)*>()> (self):
        tick 7;
        ()

fn main <()->i32> ():
    let g <Guard> Guard 0;
    0
"#;
    assert_eq!(run_drop_trace(source), vec![7]);
}

#[test]
fn auto_drop_uses_lifo_order_in_nested_scope() {
    let source = r#"
#target wasm
#indent 4
#entry main
#no_prelude
#import "core/traits/drop" as *
#extern "env" "tick" fn tick <(i32)*>()>

struct OuterGuard:
    dummy <i32>
struct InnerAGuard:
    dummy <i32>
struct InnerBGuard:
    dummy <i32>

impl Drop for OuterGuard:
    fn drop <(&OuterGuard)*>()> (self):
        tick 1;
        ()

impl Drop for InnerAGuard:
    fn drop <(&InnerAGuard)*>()> (self):
        tick 2;
        ()

impl Drop for InnerBGuard:
    fn drop <(&InnerBGuard)*>()> (self):
        tick 3;
        ()

fn main <()->i32> ():
    let outer <OuterGuard> OuterGuard 0;
    let _ <i32> if true:
        then:
            let inner_a <InnerAGuard> InnerAGuard 0;
            let inner_b <InnerBGuard> InnerBGuard 0;
            1
        else:
            0
    0
"#;
    assert_eq!(run_drop_trace(source), vec![3, 2, 1]);
}

#[test]
fn auto_drop_partially_moved_struct_drops_remaining_fields() {
    let source = r#"
#target wasm
#indent 4
#entry main
#no_prelude
#import "core/field" as field
#import "core/traits/drop" as *
#extern "env" "tick" fn tick <(i32)*>()>

struct GuardA:
    dummy <i32>
struct GuardB:
    dummy <i32>
struct Pair:
    left <GuardA>
    right <GuardB>

impl Drop for GuardA:
    fn drop <(&GuardA)*>()> (self):
        tick 1;
        ()

impl Drop for GuardB:
    fn drop <(&GuardB)*>()> (self):
        tick 2;
        ()

impl Drop for Pair:
    fn drop <(&Pair)*>()> (self):
        tick 9;
        ()

fn main <()->i32> ():
    let p <Pair> Pair (GuardA 0) (GuardB 0);
    let left <GuardA> field::get p "left";
    0
"#;
    assert_eq!(run_drop_trace(source), vec![1, 2]);
}

#[test]
fn auto_drop_only_runs_taken_branch_locals() {
    let source = r#"
#target wasm
#indent 4
#entry main
#no_prelude
#import "core/traits/drop" as *
#extern "env" "tick" fn tick <(i32)*>()>

struct TrueGuard:
    dummy <i32>
struct FalseGuard:
    dummy <i32>

impl Drop for TrueGuard:
    fn drop <(&TrueGuard)*>()> (self):
        tick 10;
        ()

impl Drop for FalseGuard:
    fn drop <(&FalseGuard)*>()> (self):
        tick 20;
        ()

fn main <()->i32> ():
    let flag <bool> true;
    let _ <i32> if flag:
        then:
            let g <TrueGuard> TrueGuard 0;
            1
        else:
            let h <FalseGuard> FalseGuard 0;
            2
    0
"#;
    assert_eq!(run_drop_trace(source), vec![10]);
}

#[test]
fn auto_drop_handles_shadowing_as_distinct_bindings() {
    let source = r#"
#target wasm
#indent 4
#entry main
#no_prelude
#import "core/traits/drop" as *
#extern "env" "tick" fn tick <(i32)*>()>

struct OuterGuard:
    dummy <i32>
struct InnerGuard:
    dummy <i32>

impl Drop for OuterGuard:
    fn drop <(&OuterGuard)*>()> (self):
        tick 1;
        ()

impl Drop for InnerGuard:
    fn drop <(&InnerGuard)*>()> (self):
        tick 2;
        ()

fn main <()->i32> ():
    let g <OuterGuard> OuterGuard 0;
    let _ <i32> if true:
        then:
            let g <InnerGuard> InnerGuard 0;
            1
        else:
            0
    0
"#;
    assert_eq!(run_drop_trace(source), vec![2, 1]);
}

#[test]
fn conditionally_moved_value_does_not_force_drop_error() {
    let source = r#"
#target wasm
#indent 4
#entry main
#no_prelude
#import "core/traits/drop" as *

struct Guard:
    id <i32>

impl Drop for Guard:
    fn drop <(&Guard)*>()> (self):
        ()

fn consume <(Guard)*>()> (g):
    ()

fn main <()*>i32> ():
    let flag <bool> true;
    let g <Guard> Guard 1;
    if flag:
        then:
            consume g;
            1
        else:
            0
    0
"#;
    compile_drop_test(source).expect("conditional move should not trigger auto-drop diagnostics");
}

#[test]
fn drop_trait_requires_loader_visible_stdlib() {
    let source = r#"
#target wasm
#indent 4
#entry main
#no_prelude
#import "core/traits/drop" as *

struct Guard:
    id <i32>

impl Drop for Guard:
    fn drop <(&Guard)*>()> (self):
        ()

fn main <()->i32> ():
    let g <Guard> Guard 9;
    0
"#;
    let artifact = compile_drop_test(source).expect("loader-based compile should resolve Drop");
    assert!(!artifact.is_empty(), "generated wasm should not be empty");
}
