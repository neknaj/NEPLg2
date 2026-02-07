use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let stdlib_root = manifest_dir.join("../stdlib");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let dest = out_dir.join("stdlib_entries.rs");

    let tests_root = stdlib_root.join("tests");
    let tests_backup_root = stdlib_root.join("tests_backup");

    let examples_root = manifest_dir.join("../examples");

    let mut std_files = Vec::new();
    collect_nepl_files(&stdlib_root, &mut std_files);
    std_files.retain(|p| {
        !p.starts_with(&tests_root) && !p.starts_with(&tests_backup_root)
    });
    sort_paths(&mut std_files);

    let mut test_files = Vec::new();
    if tests_root.exists() {
        collect_nepl_files(&tests_root, &mut test_files);
    }
    sort_paths(&mut test_files);

    let mut example_files = Vec::new();
    if examples_root.exists() {
        collect_nepl_files(&examples_root, &mut example_files);
    }
    sort_paths(&mut example_files);

    let readme_path = manifest_dir.join("../README.md");

    let mut out = String::new();
    out.push_str("pub static STD_LIB_ENTRIES: &[(&str, &str)] = &[\n");
    for path in &std_files {
        let rel = path.strip_prefix(&stdlib_root).unwrap();
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let abs = path.display();
        out.push_str(&format!(
            "    (\"{}\", include_str!(r#\"{}\"#)),\n",
            rel_str, abs
        ));
        println!("cargo:rerun-if-changed={}", abs);
    }
    out.push_str("];\n");

    out.push_str("pub static TEST_ENTRIES: &[(&str, &str)] = &[\n");
    for path in &test_files {
        let name = path
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .replace('\\', "/");
        let abs = path.display();
        out.push_str(&format!(
            "    (\"{}\", include_str!(r#\"{}\"#)),\n",
            name, abs
        ));
        println!("cargo:rerun-if-changed={}", abs);
    }
    out.push_str("];\n");

    out.push_str("pub static EXAMPLE_ENTRIES: &[(&str, &str)] = &[\n");
    for path in &example_files {
        let rel = path.strip_prefix(&examples_root).unwrap();
        let rel_str = rel.to_string_lossy().replace('\\', "/");
        let abs = path.display();
        out.push_str(&format!(
            "    (\"{}\", include_str!(r#\"{}\"#)),\n",
            rel_str, abs
        ));
        println!("cargo:rerun-if-changed={}", abs);
    }
    out.push_str("];\n");

    if readme_path.exists() {
        let abs = readme_path.display();
        out.push_str(&format!(
            "pub static README_CONTENT: &str = include_str!(r#\"{}\"#);\n",
            abs
        ));
        println!("cargo:rerun-if-changed={}", abs);
    } else {
        out.push_str("pub static README_CONTENT: &str = \"\";\n");
    }

    fs::write(dest, out).unwrap();
    println!("cargo:rerun-if-changed={}", stdlib_root.display());
    println!("cargo:rerun-if-changed={}", examples_root.display());
}

fn collect_nepl_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_nepl_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("nepl") {
            out.push(path);
        }
    }
}

fn sort_paths(paths: &mut Vec<PathBuf>) {
    paths.sort_by(|a, b| {
        let a_str = a.to_string_lossy();
        let b_str = b.to_string_lossy();
        a_str.cmp(&b_str)
    });
}
