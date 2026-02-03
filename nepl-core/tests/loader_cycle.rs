use nepl_core::loader::Loader;
use std::fs;
use std::path::PathBuf;

#[test]
fn import_cycle_is_error() {
    let dir = std::env::temp_dir().join("nepl_cycle_test");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let a = dir.join("a.nepl");
    let b = dir.join("b.nepl");
    fs::write(&a, "#import \"./b\"\n#entry main\nfn main <()->i32> (): 0\n").unwrap();
    fs::write(&b, "#import \"./a\"\n").unwrap();
    let mut loader = Loader::new(stdlib_root());
    let res = loader.load(&a);
    assert!(res.is_err());
    let _ = fs::remove_dir_all(&dir);
}

fn stdlib_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("stdlib")
}
