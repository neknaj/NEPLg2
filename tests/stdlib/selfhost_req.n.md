# selfhost_req.rs 由来の doctest

このファイルは Rust テスト `selfhost_req.rs` を .n.md 形式へ機械的に移植したものです。移植が難しい（複数ファイルや Rust 専用 API を使う）テストは `skip` として残しています。
## test_req_file_io

以前は host filesystem の positive-path read を前提にしていましたが、
doctest 実行環境の preopen に依存して不安定でした。
現在は「ファイル I/O の失敗が `Result::Err` として安全に返ること」を stable に確認します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"selfhost_req_file_io\" count=1 failed=0\nassertion index=0 status=ok kind=bool label=\"missing file returns err\" expected=\"true\" actual=\"true\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
// 想定: std/fs モジュールの追加、または std/stdio の拡張
#import "std/fs" as *
#import "std/stdio" as *
#import "alloc/string" as *
#import "core/result" as *
#import "std/test" as *

fn consume_str %fn str unit \s:
    len s
    unit

fn main %impure fn void i32 \void:
    // 要件: ファイル I/O の失敗が Result で扱えること
    let path "__definitely_missing_selfhost_req_file__.txt";
    let res %Result str i32 fs_read_to_string path;

    let ok %bool match res:
        Result::Ok content:
            consume_str content;
            false
        Result::Err _e:
            true
    let report:
        test_report_new "selfhost_req_file_io"
        |> test_report_push assert "missing file returns err" ok
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## test_req_byte_manipulation

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"selfhost_req_byte_manipulation\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"first byte as i32\" expected=\"222\" actual=\"222\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "alloc/collections/vec" as *
#import "alloc/diag/error" as *
#import "core/cast" as *
#import "core/option" as *
#import "core/result" as *
#import "core/field" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    // 要件: u8 型 (現状は i32/bool/f32/str のみで u8 がない)
    let b1 %u8 cast 0xDE;
    let b2 %u8 cast 0xAD;

    // 要件: Vec u8 (バイトバッファ)
    let mut buf %Vec u8 unwrap_ok new;
    set buf unwrap_ok push buf b1;
    set buf unwrap_ok push buf b2;

    // 要件: バイト単位のアクセス
    let first %Option u8 get &buf 0;
    let actual %i32 match first:
        Option::Some val:
            // i32へのキャスト等
            let out %i32 cast val
            free buf;
            out
        Option::None:
            free buf;
            0
    let report:
        test_report_new "selfhost_req_byte_manipulation"
        |> test_report_push assert_eq_i32 "first byte as i32" 222 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## test_req_string_utils

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"selfhost_req_string_utils\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"trim slice result code\" expected=\"0\" actual=\"0\" message=\"\"\n"
```neplg2

#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "core/math" as *
#import "std/test" as *

fn consume_str %fn str unit \s:
    len s
    unit

fn main %impure fn void i32 \void:
    let s "  fn main(a: i32)  ";

    // 要件: trim (前後の空白除去)
    let trimmed %str str_trim s;

    // 要件: starts_with / ends_with
    let ok_starts_with_fn %bool str_starts_with trimmed "fn";
    let actual %i32 if:
        ok_starts_with_fn
        then:
            // 要件: delimiter search (分割せずに区切り位置を調べる)
            let open %i32 str_find trimmed "(";
            if:
                lt open 0
                then:
                    consume_str trimmed;
                    3
                else:
                    let name_part %str str_slice trimmed 0 open; // "fn main"

                    // 要件: substring / slice
                    let func_name %str str_slice name_part 3 len name_part; // "main"
                    let ok %bool str_eq func_name "main"

                    consume_str func_name;
                    consume_str name_part;
                    consume_str trimmed;
                    if ok 0 2
        else:
            consume_str trimmed;
            1
    let report:
        test_report_new "selfhost_req_string_utils"
        |> test_report_push assert_eq_i32 "trim slice result code" 0 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown

```

## test_req_string_map

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"selfhost_req_string_map\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"hash map string key value\" expected=\"10\" actual=\"10\" message=\"\"\n"
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
#import "std/test" as *

fn must_hms %impure fn Result HashMap str i32 DefaultHash32 Diag HashMap str i32 DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn must_hms %impure fn Result HashMap str i32 DefaultHash32 HashMapUpdateError str i32 DefaultHash32 HashMap str i32 DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err e:
            let hm %HashMap str i32 DefaultHash32 hashmap_update_error_owner e;
            free hm;
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    // 要件: キーに str を指定できる HashMap
    let map0 %HashMap str i32 DefaultHash32 must_hms new DefaultHash32;
    let map1 %HashMap str i32 DefaultHash32 must_hms insert map0 "foo" 10;
    let map %HashMap str i32 DefaultHash32 must_hms insert map1 "bar" 20;

    let got %i32 match get &map "foo":
        Option::Some v:
            v
        Option::None:
            1
    free map;
    let report:
        test_report_new "selfhost_req_string_map"
        |> test_report_push assert_eq_i32 "hash map string key value" 10 got
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## test_req_string_builder

以前はコンパイル確認のみでした。
StringBuilder の操作結果 `"Error: 404 Not Found"` の長さが期待どおりになることを、stdout の assertion report で検証します。
文字列の長さは 20（"Error: "=7, "404"=3, " Not Found"=10）です。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"selfhost_req_string_builder\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"builder length\" expected=\"20\" actual=\"20\" message=\"\"\n"
```neplg2
#entry main
#indent 4
#target std
#import "alloc/string" as *
#import "std/test" as *

fn main %impure fn void i32 \void:
    // 要件: StringBuilder のような可変文字列バッファ
    let mut sb %StringBuilder string_builder_new;

    set sb sb_append sb "Error: ";
    set sb sb_append_i32 sb 404;
    set sb sb_append sb " Not Found";

    let res %str sb_build sb;

    // "Error: 404 Not Found"
    let actual %i32 len res
    let report:
        test_report_new "selfhost_req_string_builder"
        |> test_report_push assert_eq_i32 "builder length" 20 actual
    let shown test_report_print_stdout report
    test_report_exit_code shown
```

## test_req_trait_extensions

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: "test_report name=\"selfhost_req_trait_extensions\" count=1 failed=0\nassertion index=0 status=ok kind=eq_i32 label=\"trait key value length\" expected=\"5\" actual=\"5\" message=\"\"\n"
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
#import "std/test" as *

// ユーザー定義型
struct Point:
    x %i32
    y %i32

// 要件: ユーザー定義型をMapのキーにするための HashKey trait 実装
impl HashKey for Point:
    fn eq %fn Point fn Point bool \a\b:
        let ax %i32 field::get a "x"
        let ay %i32 field::get a "y"
        let bx %i32 field::get b "x"
        let by %i32 field::get b "y"
        and (eq ax bx) (eq ay by)

    fn hash32 %fn Point i32 \self:
        xor field::get self "x" field::get self "y"

impl Clone for Point:
    fn clone %fn &Point Point \self:
        *self

impl Copy for Point:
    fn copy_mark %fn Point Point \self:
        self

fn must_hmp %impure fn Result HashMap Point str DefaultHash32 Diag HashMap Point str DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err _d:
            #intrinsic "unreachable" <> ()

fn must_hmp %impure fn Result HashMap Point str DefaultHash32 HashMapUpdateError Point str DefaultHash32 HashMap Point str DefaultHash32 \r:
    match r:
        Result::Ok hm:
            hm
        Result::Err e:
            let hm %HashMap Point str DefaultHash32 hashmap_update_error_owner e;
            free hm;
            #intrinsic "unreachable" <> ()

fn main %impure fn void i32 \void:
    let map0 %HashMap Point str DefaultHash32 must_hmp new DefaultHash32;
    let map1 %HashMap Point str DefaultHash32 must_hmp insert map0 (Point 10 20) "Start";
    let got %i32 match get &map1 \Point 10 20:
        Option::Some name:
            len name
        Option::None:
            0
    free map1;
    let report:
        test_report_new "selfhost_req_trait_extensions"
        |> test_report_push assert_eq_i32 "trait key value length" 5 got
    let shown test_report_print_stdout report
    test_report_exit_code shown
```
