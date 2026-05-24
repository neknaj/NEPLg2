# Project: byte output

binary output を組み立てるときは `ByteBuilder` を使い、最後に `ByteBuf` として取り出します。text として確認する場合は UTF-8 検証付きで `str` に戻します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok]
    ##: [0] ok
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "alloc/io" as *
#import "core/result" as *
#import "std/test" as *

fn make_text_bytes %impure fn () Result str str \():
    match byte_builder_new:
        Result::Err _e:
            Result<str,str>::Err "builder allocation failed"
        Result::Ok b0:
            match byte_builder_push_char_utf8 b0 'A':
                Result::Err _e:
                    Result<str,str>::Err "push A failed"
                Result::Ok b1:
                    match byte_builder_push_char_utf8 b1 'あ':
                        Result::Err _e:
                            Result<str,str>::Err "push char failed"
                        Result::Ok b2:
                            match byte_builder_finish b2:
                                Result::Err _e:
                                    Result<str,str>::Err "finish failed"
                                Result::Ok bytes:
                                    match io_bytebuf_to_str_result bytes:
                                        Result::Err _e:
                                            Result<str,str>::Err "utf8 decode failed"
                                        Result::Ok text:
                                            Result<str,str>::Ok text

fn expect_text %fn Result str str fn str Result () str \got\expected:
    match got:
        Result::Ok text:
            check_str_eq expected text
        Result::Err msg:
            Result<(),str>::Err msg

fn main %impure fn () i32 \():
    let checks:
        checks_new
        |> checks_push expect_text make_text_bytes "Aあ"
    let shown checks_print_report checks
    checks_exit_code shown
```

`ByteBuilder` も owner です。各 `push` は builder を消費して新しい builder を返すため、成功した値だけを次の段階へ渡します。
