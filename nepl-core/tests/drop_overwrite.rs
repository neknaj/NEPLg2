use nepl_core::{CompileOptions, CompileTarget};
use std::sync::{Arc, Mutex};
use wasmi::{Engine, Linker, Module, Store};

mod harness;

fn run_drop_trace(source: &str) -> Vec<i32> {
    let wasm = harness::compile_src_with_options(
        source,
        CompileOptions {
            target: Some(CompileTarget::Wasm),
            verbose: false,
            profile: None,
            test_mode: false,
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
    let main = instance
        .get_typed_func::<(), i32>(&store, "main")
        .expect("main");
    let _ = main.call(&mut store, ()).expect("call");
    let out = trace.lock().unwrap().clone();
    out
}

#[test]
fn auto_drop_runs_for_overwritten_value_after_rhs() {
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
        tick 2;
        ()

fn make_guard <()*>Guard> ():
    tick 1;
    Guard 1

fn main <()*>i32> ():
    let mut g <Guard> Guard 0;
    set g make_guard;
    0
"#;
    assert_eq!(run_drop_trace(source), vec![1, 2, 2]);
}
