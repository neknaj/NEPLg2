use nepl_core::loader::Loader;

#[test]
fn show_loaded_files() {
    let src = r#"
#target wasi
#entry main
#indent 4
#import "std/stdio"
#use std::stdio::*

fn main <()* >()> ():
    print "ok"
"#;
    let loader = Loader::new(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("stdlib"),
    );
    let loaded = loader
        .load_inline("<test>".into(), src.to_string())
        .expect("load");
    for id in 0..20 {
        let fid = nepl_core::span::FileId(id);
        if let Some(p) = loaded.source_map.path(fid) {
            println!("FileId({}) = {}", id, p.display());
        } else {
            break;
        }
    }

    // Try to compile module and print diagnostics with source mapping
    match nepl_core::compile_module(
        loaded.module.clone(),
        nepl_core::CompileOptions { target: None },
    ) {
        Ok(artifact) => println!("compiled ok, wasm len {}", artifact.wasm.len()),
        Err(nepl_core::CoreError::Diagnostics(diags)) => {
            println!("Diagnostics({})", diags.len());
            for d in diags {
                let fid = d.primary.span.file_id;
                let start = d.primary.span.start;
                if let Some((line, col)) = loaded.source_map.line_col(fid, start) {
                    if let Some(path) = loaded.source_map.path(fid) {
                        println!("{}:{}:{}: {}", path.display(), line + 1, col + 1, d.message);
                        if let Some(src) = loaded.source_map.get(fid) {
                            let start = d.primary.span.start as usize;
                            let end = d.primary.span.end as usize;
                            let end = end.min(src.len());
                            let snippet = &src[start..end];
                            println!("> snippet: '{}'", snippet.replace("\n", " "));
                        }
                    }
                } else {
                    println!("DIAG: {}", d.message);
                }
            }
        }
        Err(e) => println!("compile error: {:?}", e),
    }
}
