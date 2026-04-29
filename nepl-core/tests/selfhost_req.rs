mod harness;
use harness::{compile_src, run_main_i32, run_main_wasi_i32};

// ============================================================================
// Self-Hosting Requirements / Feature Gap Tests
// ============================================================================
// 以下のテストケースは、NEPLコンパイラのセルフホスト（NEPL自身でNEPLコンパイラを書くこと）
// に向けて不足している機能を明確化するためのものです。
// 実装が進むにつれて、これらのテストを通過する通常の回帰テストへ移していきます。

// 1. ファイルI/O・パス操作 (File I/O & Path Manipulation)
// 必須度: 高
// 不足機能: ファイルの読み込み、書き込み、パスの結合など
// WASI環境下での `path_open`, `fd_read`, `fd_write` 等のラッパーが必要です。
#[test]
fn test_req_file_io() {
    let src = r#"
#entry main
#indent 4
#target std
// 想定: std/fs モジュールの追加、または std/stdio の拡張
#import "std/fs" as *
#import "std/stdio" as *
#import "core/result" as *

fn main <()*>i32> ():
    // 要件: ファイル I/O の失敗が Result で扱えること
    let path "__definitely_missing_selfhost_req_file__.txt";
    let res <Result<str, i32>> fs_read_to_string path;

    match res:
        Result::Ok _content:
            1
        Result::Err _e:
            0
"#;
    let v = run_main_wasi_i32(src);
    assert_eq!(v, 0);
}

// 2. バイト列/エンコード出力 (Byte Arrays / Encoding)
// 必須度: 高
// 不足機能: u8型、バイト配列(Vec<u8>)、ビット操作、バイナリ出力
// WASMバイナリを生成するために、i32ではなくバイト単位での精密な操作が必要です。
#[test]
fn test_req_byte_manipulation() {
    let src = r#"
#entry main
#indent 4
#import "alloc/collections/vec" as *
#import "alloc/diag/error" as *
#import "core/cast" as *
#import "core/option" as *

fn main <()*>i32> ():
    // 要件: u8 型 (現状は i32/bool/f32/str のみで u8 がない)
    let b1 <u8> cast 0xDE;
    let b2 <u8> cast 0xAD;

    // 要件: Vec<u8> (バイトバッファ)
    let mut buf <Vec<u8>> unwrap_ok new<u8>;
    set buf unwrap_ok push<u8> buf b1;
    set buf unwrap_ok push<u8> buf b2;

    // 要件: バイト単位のアクセス
    match get<u8> buf 0:
        Option::Some val:
            // i32へのキャスト等
            cast val
        Option::None:
            0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 222); // 0xDE
}

// 3. 文字列処理の強化 (String Processing Enhancements)
// 必須度: 中〜高
// 不足機能: slice, split, trim, starts_with, char/byte iterator
// パーサーを書くには `len` と `concat` だけでは不十分で、高度な文字列操作が必要です。
#[test]
fn test_req_string_utils() {
    let src = r#"
#entry main
#indent 4
#import "alloc/string" as *
#import "alloc/collections/vec" as *
#import "core/option" as *

fn main <()*>i32> ():
    let s "  fn main(a: i32)  ";

    // 要件: trim (前後の空白除去)
    let trimmed <str> str_trim s;

    // 要件: starts_with / ends_with
    let ok_starts_with_fn <bool> str_starts_with trimmed "fn";
    if:
        ok_starts_with_fn
        then:
            // 要件: split (区切り文字での分割)
            let parts <Vec<str>> str_split trimmed "(";
            let name_part <str> unwrap<str> get<str> parts 0; // "fn main"
            
            // 要件: substring / slice
            let func_name <str> str_slice name_part 3 len name_part; // "main"
            
            if:
                str_eq func_name "main"
                then 0
                else 2
        else 1
    
"#;
    // selfhost要件としては「コンパイル可能な文字列ユーティリティが揃っていること」を確認する。
    // 実行時の挙動差分（runner差）は tests/selfhost_req.n.md 側で run まで検証する。
    compile_src(src);
}

// 4. 文字列キーのMap/Set (String-keyed Map/Set)
// 必須度: 高
// 不足機能: generic Map/Set、あるいは String 専用の Map/Set
// シンボルテーブルや識別子の管理に不可欠です。現状は i32 キーのみです。
#[test]
fn test_req_string_map() {
    let src = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/hashmap" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/option" as *
#import "core/result" as *

fn must_hms <(Result<HashMap<str,i32,DefaultHash32>, Diag>)*>HashMap<str,i32,DefaultHash32>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    // 要件: キーに str を指定できる HashMap
    let map0 <HashMap<str,i32,DefaultHash32>> must_hms new DefaultHash32;
    let map1 <HashMap<str,i32,DefaultHash32>> must_hms insert map0 "foo" 10;
    let map <HashMap<str,i32,DefaultHash32>> must_hms insert map1 "bar" 20;
    
    let out <i32> match get &map "foo":
        Option::Some v:
            v
        Option::None:
            1
    free map
    out
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

// 5. 文字列ビルダ/フォーマット (String Builder / Formatting)
// 必須度: 中
// 不足機能: append 可能な文字列バッファ、format! 相当
// エラーメッセージ生成やコード生成で文字列連結を繰り返すと効率が悪いため。
#[test]
fn test_req_string_builder() {
    let src = r#"
#entry main
#indent 4
#import "alloc/string" as *

fn main <()*>i32> ():
    // 要件: StringBuilder のような可変文字列バッファ
    let mut sb <StringBuilder> string_builder_new;
    
    set sb sb_append sb "Error: ";
    set sb sb_append_i32 sb 404;
    set sb sb_append sb " Not Found";
    
    let res <str> sb_build sb;
    
    // "Error: 404 Not Found"
    len res
"#;
    let v = run_main_i32(src);
    assert_ne!(v, 0);
}

// 6. トレイトの拡張 (Trait Extensions)
// 必須度: 中〜高
// 不足機能: Ord, Hash, Eq などのトレイトサポート
// ジェネリックなデータ構造をユーザー定義型等で利用するために必要です。
#[test]
fn test_req_trait_extensions() {
    let src = r#"
#entry main
#indent 4
#target std
#import "alloc/collections/hashmap" as *
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/field" as field
#import "core/option" as *
#import "core/result" as *
#import "core/traits/copy" as *
#import "core/traits/hash" as *
#import "core/traits/hash_key" as *

// ユーザー定義型
struct Point:
    x <i32>
    y <i32>

// 要件: ユーザー定義型をMapのキーにするための HashKey trait 実装
impl HashKey for Point:
    fn eq <(Point, Point)->bool> (a, b):
        let ax <i32> field::get a "x"
        let ay <i32> field::get a "y"
        let bx <i32> field::get b "x"
        let by <i32> field::get b "y"
        and (eq ax bx) (eq ay by)

    fn hash32 <(Point)->i32> (self):
        xor field::get self "x" field::get self "y"

impl Clone for Point:
    fn clone <(&Point)->Point> (self):
        *self

impl Copy for Point:
    fn copy_mark <(Point)->Point> (self):
        self

fn must_hmp <(Result<HashMap<Point,str,DefaultHash32>, Diag>)*>HashMap<Point,str,DefaultHash32>> (r):
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn main <()*>i32> ():
    let map0 <HashMap<Point,str,DefaultHash32>> must_hmp new DefaultHash32;
    let map1 <HashMap<Point,str,DefaultHash32>> must_hmp insert map0 (Point 10 20) "Start";
    let out <i32> match get &map1 (Point 10 20):
        Option::Some name:
            len name
        Option::None:
            0
    free map1
    out
"#;
    let v = run_main_wasi_i32(src);
    assert_eq!(v, 5);
}
