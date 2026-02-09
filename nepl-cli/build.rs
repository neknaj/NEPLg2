use std::process::Command;

fn main() {
    // Git の HEAD コミットハッシュを取得して、コンパイル時の環境変数として埋め込む
    // これにより、コード側では env! マクロで参照できるようになる
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());

    // もし外部（CI など）から NEPLG2_COMPILER_COMMIT が与えられていれば、それを優先する
    let commit = std::env::var("NEPLG2_COMPILER_COMMIT")
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| {
            Command::new("git")
                .args(["rev-parse", "HEAD"])
                .current_dir(&manifest_dir)
                .output()
                .ok()
                .and_then(|o| String::from_utf8(o.stdout).ok())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "unknown".to_string())
        });

    // 古い Cargo でも動くように、cargo:KEY=VALUE 形式を使う
    println!("cargo:rustc-env=NEPLG2_COMPILER_COMMIT={}", commit);

    // 変更検知（Git の状態が変わったら build.rs を再実行させる）
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/refs");
}
