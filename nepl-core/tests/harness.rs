use nepl_core::loader::Loader;
use nepl_core::{compile_module, CompileOptions, CompileTarget};
use wasmi::{Engine, Extern, Linker, Module, Store, Caller};

/// Compile source to wasm bytes.
pub fn compile_src(src: &str) -> Vec<u8> {
    let loader = Loader::new(stdlib_root());
    let loaded = loader
        .load_inline("<test>".into(), src.to_string())
        .expect("load");
    let artifact = compile_module(
        loaded.module,
        CompileOptions {
            target: CompileTarget::Wasm,
        },
    )
    .expect("compile failure");
    artifact.wasm
}

/// Compile and run `main` returning i32 (or 0 if main is ())->()).
pub fn run_main_i32(src: &str) -> i32 {
    let wasm = compile_src(src);
    let engine = Engine::default();
    let module = Module::new(&engine, &*wasm).expect("module");
    let mut linker = Linker::new(&engine);
    // Minimal env for legacy stdio (if present)
    linker
        .func_wrap("env", "print_i32", |x: i32| {
            println!("{x}");
        })
        .unwrap();
    linker
        .func_wrap(
            "env",
            "print_str",
            |mut caller: Caller<'_, ()>, ptr: i32| {
                if let Some(Extern::Memory(mem)) = caller.get_export("memory") {
                    let data = mem.data(&caller);
                    let offset = ptr as usize;
                    if offset + 4 <= data.len() {
                        let len =
                            u32::from_le_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
                        let start = offset + 4;
                        if start + len <= data.len() {
                            let s = std::str::from_utf8(&data[start..start + len])
                                .unwrap_or("<utf8-error>");
                            println!("{s}");
                        }
                    }
                }
            },
        )
        .unwrap();
    let mut store = Store::new(&engine, ());
    let instance = linker
        .instantiate(&mut store, &module)
        .and_then(|pre| pre.start(&mut store))
        .expect("instantiate");
    if let Ok(f) = instance.get_typed_func::<(), i32>(&store, "main") {
        f.call(&mut store, ()).expect("call")
    } else if let Ok(fu) = instance.get_typed_func::<(), ()>(&store, "main") {
        fu.call(&mut store, ()).expect("call");
        0
    } else {
        panic!("main not found")
    }
}

fn stdlib_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib")
}
