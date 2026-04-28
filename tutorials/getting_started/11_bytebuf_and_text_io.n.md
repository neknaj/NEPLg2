# ByteBuf と text I/O

binary data は `ByteBuf` のまま扱い、text として読むときだけ UTF-8 検証付き API で `str` に変換します。外部入力を検証なしで `str` にする書き方は避けます。

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "alloc/io" as *
#import "core/result" as *
#import "std/test" as *

fn roundtrip_text <(str)*>Result<str,str>> (text):
    match io_bytebuf_from_str_result text:
        Result::Err _e:
            Result<str,str>::Err "encode failed"
        Result::Ok bytes:
            match io_bytebuf_to_str_result bytes:
                Result::Err _e:
                    Result<str,str>::Err "decode failed"
                Result::Ok out:
                    Result<str,str>::Ok out

fn expect_text <(Result<str,str>,str)->Result<(),str>> (got, expected):
    match got:
        Result::Ok out:
            check_str_eq expected out
        Result::Err msg:
            Result<(),str>::Err msg

fn main <()*>i32> ():
    let checks:
        checks_new
        |> checks_push expect_text roundtrip_text "NEPL" "NEPL"
        |> checks_push expect_text roundtrip_text "Aあ" "Aあ"
    checks_exit_code checks
```

`ByteBuf` は所有権を持つ buffer です。`io_bytebuf_to_str_result` のように引数を消費する関数へ渡した後は、同じ値を再利用しません。
