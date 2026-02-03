mod harness;
use harness::run_main_i32;

// ============================================================================
// Self-Hosting Requirements / Feature Gap Tests
// ============================================================================
// 以下のテストケースは、NEPLコンパイラのセルフホスト（NEPL自身でNEPLコンパイラを書くこと）
// に向けて不足している機能を明確化するためのものです。
// 現状ではこれらの機能は未実装であるため、テストは失敗（コンパイルエラーまたは実行時エラー）します。
// 実装が進むにつれて、これらのテストを修正・通過させていくことを目指します。

// 1. ファイルI/O・パス操作 (File I/O & Path Manipulation)
// 必須度: 高
// 不足機能: ファイルの読み込み、書き込み、パスの結合など
// WASI環境下での `path_open`, `fd_read`, `fd_write` 等のラッパーが必要です。
#[test]
#[ignore] // 未実装のためスキップ
fn test_req_file_io() {
    let src = r#"
#entry main
#indent 4
// 想定: std/fs モジュールの追加、または std/stdio の拡張
#import "std/fs" as *
#import "core/result" as *

fn main <()*>i32> ():
    // 要件: ソースコードファイルを読み込めること
    let path "test.nepl";
    let res <Result<str, i32>> fs_read_to_string path;
    
    match res:
        Result::Ok content:
            // 読み込んだ内容を表示
            print content;
            0
        Result::Err e:
            e
"#;
    let _ = run_main_i32(src);
}

// 2. バイト列/エンコード出力 (Byte Arrays / Encoding)
// 必須度: 高
// 不足機能: u8型、バイト配列(Vec<u8>)、ビット操作、バイナリ出力
// WASMバイナリを生成するために、i32ではなくバイト単位での精密な操作が必要です。
#[test]
#[ignore] // 未実装のためスキップ
fn test_req_byte_manipulation() {
    let src = r#"
#entry main
#indent 4
#import "alloc/vec" as *

fn main <()*>i32> ():
    // 要件: u8 型 (現状は i32/bool/f32/str のみで u8 がない)
    let b1 <u8> 0xDE;
    let b2 <u8> 0xAD;
    
    // 要件: Vec<u8> (バイトバッファ)
    let mut buf <Vec<u8>> vec_new<u8> ();
    vec_push<u8> buf b1;
    vec_push<u8> buf b2;
    
    // 要件: バイト単位のアクセス
    let val <u8> vec_get<u8> buf 0;
    
    // i32へのキャスト等
    cast<u8, i32> val
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 222); // 0xDE
}

// 3. 文字列処理の強化 (String Processing Enhancements)
// 必須度: 中〜高
// 不足機能: slice, split, trim, starts_with, char/byte iterator
// パーサーを書くには `len` と `concat` だけでは不十分で、高度な文字列操作が必要です。
#[test]
#[ignore] // 未実装のためスキップ
fn test_req_string_utils() {
    let src = r#"
#entry main
#indent 4
#import "alloc/string" as *
#import "alloc/vec" as *

fn main <()*>i32> ():
    let s "  fn main(a: i32)  ";
    
    // 要件: trim (前後の空白除去)
    let trimmed <str> str_trim s;
    
    // 要件: starts_with / ends_with
    if not (str_starts_with trimmed "fn"):
        return 1;
        
    // 要件: split (区切り文字での分割)
    let parts <Vec<str>> str_split trimmed "(";
    let name_part <str> vec_get<str> parts 0; // "fn main"
    
    // 要件: substring / slice
    let func_name <str> str_slice name_part 3 (str_len name_part); // "main"
    
    if str_eq func_name "main":
        0
    else:
        2
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}

// 4. 文字列キーのMap/Set (String-keyed Map/Set)
// 必須度: 高
// 不足機能: generic Map/Set、あるいは String 専用の Map/Set
// シンボルテーブルや識別子の管理に不可欠です。現状は i32 キーのみです。
#[test]
#[ignore] // 未実装のためスキップ
fn test_req_string_map() {
    let src = r#"
#entry main
#indent 4
#import "alloc/collections/hashmap" as *
#import "alloc/string" as *

fn main <()*>i32> ():
    // 要件: キーに str を指定できる HashMap
    // 現状の HashMap は内部で i32 キー前提の実装になっている可能性が高い
    let mut map <HashMap<str, i32>> hashmap_new<str, i32> ();
    
    hashmap_insert<str, i32> map "foo" 10;
    hashmap_insert<str, i32> map "bar" 20;
    
    let v <i32> hashmap_get<str, i32> map "foo";
    v
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 10);
}

// 5. 文字列ビルダ/フォーマット (String Builder / Formatting)
// 必須度: 中
// 不足機能: append 可能な文字列バッファ、format! 相当
// エラーメッセージ生成やコード生成で文字列連結を繰り返すと効率が悪いため。
#[test]
#[ignore] // 未実装のためスキップ
fn test_req_string_builder() {
    let src = r#"
#entry main
#indent 4
#import "alloc/string" as *

fn main <()*>i32> ():
    // 要件: StringBuilder のような可変文字列バッファ
    let mut sb <StringBuilder> string_builder_new ();
    
    sb_append sb "Error: ";
    sb_append_i32 sb 404;
    sb_append sb " Not Found";
    
    let res <str> sb_build sb;
    
    // "Error: 404 Not Found"
    str_len res
"#;
    let v = run_main_i32(src);
    assert_ne!(v, 0);
}

// 6. トレイトの拡張 (Trait Extensions)
// 必須度: 中〜高
// 不足機能: Ord, Hash, Eq などのトレイトサポート
// ジェネリックなデータ構造をユーザー定義型等で利用するために必要です。
#[test]
#[ignore] // 未実装のためスキップ
fn test_req_trait_extensions() {
    let src = r#"
#entry main
#indent 4
#import "alloc/collections/hashmap" as *

// ユーザー定義型
struct Point:
    x <i32>
    y <i32>

// 要件: ユーザー定義型をMapのキーにするための Hash/Eq トレイト実装
// コンパイラが trait 実装を認識し、HashMap で利用できるようにする
impl Point:
    fn hash <(Point)->i32> (self):
        i32_xor self.x self.y

    fn eq <(Point, Point)->bool> (a, b):
        and (eq a.x b.x) (eq a.y b.y)

fn main <()*>i32> ():
    let p1 <Point> Point 10 20;
    let mut map <HashMap<Point, str>> hashmap_new<Point, str> ();
    
    hashmap_insert<Point, str> map p1 "Start";
    0
"#;
    let v = run_main_i32(src);
    assert_eq!(v, 0);
}
