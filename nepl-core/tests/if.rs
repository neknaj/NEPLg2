use nepl_core::loader::Loader;

#[test]
fn compile_various_if_forms() {
    let src = r#"
#target wasi
#entry main
#indent 4
#entry main
#indent 4

fn main <()*>()> ():
    // 1-line if
    let _ <i32> if true 0 1;

    // 1-line with then/else
    let _ <i32> if true then 0 else 1;

    // multi-line with markers
    let _ <i32> if true:
        then 0
        else 1

    // multi-line with labeled blocks
    let _ <i32> if true:
        then:
            0
        else:
            1

    // nested/combined
    let _ <i32> if true 0 if true 1 2;
"#;

    let loader = Loader::new(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("stdlib"),
    );
    let loaded = loader
        .load_inline("<test>".into(), src.to_string())
        .expect("load");

    match nepl_core::compile_module(
        loaded.module.clone(),
        nepl_core::CompileOptions { target: None },
    ) {
        Ok(_) => {}
        Err(nepl_core::CoreError::Diagnostics(diags)) => {
            eprintln!("Diagnostics({})", diags.len());
            for d in diags {
                eprintln!("{}", d.message);
                let fid = d.primary.span.file_id;
                let start = d.primary.span.start;
                if let Some((line, col)) = loaded.source_map.line_col(fid, start) {
                    if let Some(path) = loaded.source_map.path(fid) {
                        eprintln!(" --> {}:{}:{}", path.display(), line + 1, col + 1);
                        if let Some(src) = loaded.source_map.get(fid) {
                            let s = d.primary.span.start as usize;
                            let e = d.primary.span.end as usize;
                            let e = e.min(src.len());
                            let snippet = &src[s..e];
                            eprintln!("> snippet: '{}'", snippet.replace("\n", " "));
                        }
                    }
                }
            }
            panic!("compile failed with diagnostics");
        }
        Err(e) => panic!("compile failed: {:?}", e),
    }
}
