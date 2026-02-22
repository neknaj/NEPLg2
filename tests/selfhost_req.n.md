# selfhost_req.rs 由来の doctest

このファイルは Rust テスト `selfhost_req.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## test_req_file_io

以前はコンパイル確認のみで、実行時に `fs_read_to_string` が成功するか（= 要件を満たすか）を検証していませんでした。
このテストはファイルI/Oの要件確認が目的なので、成功時に 0 を返すことを `ret: 0` で明示し、
失敗時はエラーコード（i32）が返ってテストが落ちるようにします。

注意: このテストは `test.nepl` が実行環境に存在することを前提にしています。

neplg2:test
ret: 0
```neplg2
#entry main
#indent 4
#target std
// 想定: std/fs モジュールの追加、または std/stdio の拡張
#import "std/fs" as *
#import "std/stdio" as *
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
```

## test_req_byte_manipulation

neplg2:test
ret: 222
```neplg2

#entry main
#indent 4
#import "alloc/vec" as *
#import "core/cast" as *
#import "core/option" as *

fn main <()*>i32> ():
    // 要件: u8 型 (現状は i32/bool/f32/str のみで u8 がない)
    let b1 <u8> cast 0xDE;
    let b2 <u8> cast 0xAD;
    
    // 要件: Vec<u8> (バイトバッファ)
    let mut buf <Vec<u8>> vec_new<u8>;
    set buf vec_push<u8> buf b1;
    set buf vec_push<u8> buf b2;
    
    // 要件: バイト単位のアクセス
    match vec_get<u8> buf 0:
        Option::Some val:
            // i32へのキャスト等
            cast val
        Option::None:
            0
```

## test_req_string_utils

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#import "alloc/string" as *
#import "alloc/vec" as *
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
            let name_part <str> unwrap<str> vec_get<str> parts 0; // "fn main"
            
            // 要件: substring / slice
            let func_name <str> str_slice name_part 3 len name_part; // "main"
            
            if:
                str_eq func_name "main"
                then 0
                else 2
        else 1
    
```

## test_req_string_map

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#import "alloc/collections/hashmap_str" as *
#import "alloc/string" as *
#import "core/option" as *

fn main <()*>i32> ():
    // 要件: キーに str を指定できる HashMap
    let mut map <i32> hashmap_str_new<i32>;
    
    hashmap_str_insert<i32> map "foo" 10;
    hashmap_str_insert<i32> map "bar" 20;
    
    match hashmap_str_get<i32> map "foo":
        Option::Some v:
            v
        Option::None:
            1
```

## test_req_string_builder

以前はコンパイル確認のみでした。
StringBuilder の操作結果 `"Error: 404 Not Found"` の長さが期待どおりになることを、返り値で検証します。
文字列の長さは 20（"Error: "=7, "404"=3, " Not Found"=10）なので `ret: 20` を追加しました。

neplg2:test
ret: 20
```neplg2
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
```

## test_req_trait_extensions

neplg2:test[compile_fail]
```neplg2

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
```
