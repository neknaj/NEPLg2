# Project: 小さな parser

parser は byte と char の境界を意識して作ります。text の先頭文字を読むときは、`str_char_at_result` で UTF-8 と範囲を確認します。

neplg2:test
ret: 0
```neplg2
| #entry main
| #indent 4
| #target std
|
#import "alloc/string" as *
#import "core/result" as *
#import "std/test" as *

enum TinyToken:
    LetterA
    DigitZero
    Other

fn token_code <(TinyToken)->i32> (token):
    match token:
        TinyToken::LetterA:
            1
        TinyToken::DigitZero:
            2
        TinyToken::Other:
            0

fn first_token <(str)->TinyToken> (source):
    match str_char_at_result source 0:
        Result::Err _e:
            TinyToken::Other
        Result::Ok c:
            match c:
                'a':
                    TinyToken::LetterA
                '0':
                    TinyToken::DigitZero
                _:
                    TinyToken::Other

fn main <()*>i32> ():
    let checks:
        checks_new
        |> checks_push check_eq_i32 1 token_code first_token "abc"
        |> checks_push check_eq_i32 2 token_code first_token "012"
        |> checks_push check_eq_i32 0 token_code first_token "xyz"
        |> checks_push check_eq_i32 0 token_code first_token ""
    checks_exit_code checks
```

文字種の分岐は `match` で書くと、escape 文字や ASCII 範囲の扱いを後から `core/char` の helper へ移しやすくなります。
