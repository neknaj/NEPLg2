# selfhost_req.rs 由来の doctest

このファイルは Rust テスト `selfhost_req.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## test_req_file_io

以前は host filesystem の positive-path read を前提にしていましたが、
doctest 実行環境の preopen に依存して不安定でした。
現在は「ファイル I/O の失敗が `Result::Err` として安全に返ること」を stable に確認します。

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

fn consume_str <(str)->()> (s):
    len s
    ()

fn main <()*>i32> ():
    // 要件: ファイル I/O の失敗が Result で扱えること
    let path "__definitely_missing_selfhost_req_file__.txt";
    let res <Result<str,i32>> fs_read_to_string path;

    match res:
        Result::Ok content:
            consume_str content;
            1
        Result::Err _e:
            0
```

## test_req_byte_manipulation

neplg2:test
ret: 222
```neplg2

#entry main
#indent 4
#import "alloc/collections/vec" as *
#import "alloc/diag/error" as *
#import "core/cast" as *
#import "core/option" as *
#import "core/field" as *

fn main <()*>i32> ():
    // 要件: u8 型 (現状は i32/bool/f32/str のみで u8 がない)
    let b1 <u8> cast 0xDE;
    let b2 <u8> cast 0xAD;

    // 要件: Vec<u8> (バイトバッファ)
    let mut buf <Vec<u8>> unwrap_ok new<u8>;
    set buf unwrap_ok push<u8> buf b1;
    set buf unwrap_ok push<u8> buf b2;

    // 要件: バイト単位のアクセス
    match get<u8> &buf 0:
        Option::Some val:
            // i32へのキャスト等
            let out <i32> cast val
            free<u8> buf;
            out
        Option::None:
            free<u8> buf;
            0
```

## test_req_string_utils

neplg2:test
ret: 0
```neplg2

#entry main
#indent 4
#import "alloc/string" as *
#import "core/math" as *

fn consume_str <(str)->()> (s):
    len s
    ()

fn main <()*>i32> ():
    let s "  fn main(a: i32)  ";

    // 要件: trim (前後の空白除去)
    let trimmed <str> str_trim s;

    // 要件: starts_with / ends_with
    let ok_starts_with_fn <bool> str_starts_with trimmed "fn";
    if:
        ok_starts_with_fn
        then:
            // 要件: delimiter search (分割せずに区切り位置を調べる)
            let open <i32> str_find trimmed "(";
            if:
                lt open 0
                then:
                    consume_str trimmed;
                    3
                else:
                    let name_part <str> str_slice trimmed 0 open; // "fn main"

                    // 要件: substring / slice
                    let func_name <str> str_slice name_part 3 len name_part; // "main"
                    let ok <bool> str_eq func_name "main"

                    consume_str func_name;
                    consume_str name_part;
                    consume_str trimmed;
                    if ok 0 2
        else:
            consume_str trimmed;
            1

```

## test_req_string_map

neplg2:test
ret: 10
```neplg2

#entry main
#indent 4
#target std
#import "alloc/collections/hashmap" as *
#import "core/traits/hash" as *
#import "alloc/diag/error" as *
#import "alloc/string" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *

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

    let got <i32> match get &map "foo":
        Option::Some v:
            v
        Option::None:
            1
    free map;
    got
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

neplg2:test
ret: 5
```neplg2

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
#import "core/field" as *
#import "core/math" as *

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
    let got <i32> match get &map1 (Point 10 20):
        Option::Some name:
            len name
        Option::None:
            0
    free map1;
    got
```
