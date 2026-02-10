use nepl_core::diagnostic::Diagnostic;
use nepl_core::loader::Loader;
use nepl_core::{compile_module, CompileOptions, CompileTarget};
use std::path::PathBuf;

mod harness;

fn compile_recursive_test(source: &str) -> Result<Vec<u8>, Vec<Diagnostic>> {
    let mut loader = Loader::new(stdlib_root());
    let loaded = loader
        .load_inline("<test>".into(), source.to_string())
        .expect("load");


    match compile_module(
        loaded.module,
        CompileOptions {
            target: Some(CompileTarget::Wasi),
            verbose: false,
            profile: None,
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
fn recursive_struct_enum() {
    let source = r#"
#target wasi
#indent 4
#import "alloc/vec" as *

struct A:
    b <Vec<B>>
enum B:
    A <A>

fn main <()*>()>():
    ()
"#;
    // this currently hangs if not fixed
    compile_recursive_test(source).expect("should succeed");
}
