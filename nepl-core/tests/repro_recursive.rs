use nepl_core::diagnostic::Diagnostic;
use nepl_core::loader::Loader;
use nepl_core::{compile_module_with_source_map, CompileOptions, CompileTarget};
use std::path::PathBuf;

mod harness;

fn compile_recursive_test(source: &str) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let mut loader = Loader::new(stdlib_root());
    let loaded = loader
        .load_inline("<test>".into(), source.to_string())
        .expect("load");

    match compile_module_with_source_map(
        loaded.module,
        Some(&loaded.source_map),
        CompileOptions {
            target: Some(CompileTarget::Wasi),
            verbose: false,
            profile: None,
            test_mode: false,
        },
    ) {
        Ok(artifact) => Ok(artifact.wasm),
        Err(nepl_core::error::CoreError::Diagnostics(ds)) => {
            for d in &ds {
                eprintln!("DIAG: {}", d.message);
            }
            Err(ds)
        }
        Err(e) => {
            eprintln!("OTHER ERR: {:?}", e);
            Err(Vec::new())
        }
    }
}

fn stdlib_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib")
}

#[test]
fn recursive_struct_enum_instantiation() {
    let source = r#"
#target wasi
#indent 4
#import "core/field" as field
#import "core/result" as *
#import "alloc/collections/vec" as *

struct A:
    b <Vec<B>>
enum B:
    A <A>

fn main <()*>()>():
    let v <Vec<B>> unwrap_ok new<B>;
    let a <A> A v;
    let b <B> B::A a;
    match b:
        B::A a1:
            let v1 <Vec<B>> field::get a1 "b";
            free<B> v1
"#;
    compile_recursive_test(source).expect("should succeed");
}
