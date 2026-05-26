# traits_serde.n.md

## serialize_trait_for_primitives

[目的/もくてき]:

- `Serialize` trait が `Stringify` や[個別/こべつ][変換/へんかん][関数/かんすう]に[直接/ちょくせつ][依存/いぞん]しない[共通/きょうつう] helper として[使/つか]えることを[確/たし]かめます。
- `bool` / `i32` / `i64` / `str` の[代表的/だいひょうてき]な[直列化/ちょくれつか][結果/けっか]が[期待/きたい]どおりかを[確認/かくにん]します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
    ##: [3] ok
```neplg2
#entry main
#target std
#import "std/test" as *
#import "core/traits/serialize" as *
#import "core/result" as *
#import "core/cast" as *

fn main %impure fn unit i32 \unit:
    let mut checks checks_new;
    set checks checks_push checks check_str_eq "true" serialize true;
    set checks checks_push checks check_str_eq "42" serialize 42;
    set checks checks_push checks check_str_eq "9001" serialize %i64 cast 9001;
    set checks checks_push checks check_str_eq "abc" serialize "abc";
    let shown checks_print_report checks;
    checks_exit_code shown
```

## deserialize_trait_for_primitives

[目的/もくてき]:

- `Deserialize` trait が `str` から[基本型/きほんがた]を[復元/ふくげん]できることを[確/たし]かめます。
- [解析/かいせき][失敗/しっぱい]が `StdErrorKind::ParseError` に[正規化/せいきか]されることを[確認/かくにん]します。

neplg2:test[stdio, normalize_newlines]
exit_code: 0
stdout: mlstr:
    ##: Checked [ok,ok,ok]
    ##: [0] ok
    ##: [1] ok
    ##: [2] ok
```neplg2
#entry main
#target std
#import "std/test" as *
#import "core/traits/deserialize" as *
#import "alloc/diag/error" as *
#import "core/result" as *
#import "core/math" as *

fn main %impure fn unit i32 \unit:
    let mut checks checks_new;

    let parsed_i32 %Result i32 StdErrorKind deserialize "42"
    match parsed_i32:
        Result::Ok v:
            set checks checks_push checks check_eq_i32 42 v
        Result::Err _e:
            set checks checks_push checks Result::Err "deserialize i32 failed";

    let parsed_bool %Result bool StdErrorKind deserialize "false"
    match parsed_bool:
        Result::Ok v:
            set checks checks_push checks check not v
        Result::Err _e:
            set checks checks_push checks Result::Err "deserialize bool failed";

    let parse_error %Result i32 StdErrorKind deserialize "oops"
    match parse_error:
        Result::Ok _v:
            set checks checks_push checks Result::Err "deserialize i32 should fail on text";
        Result::Err e:
            match e:
                StdErrorKind::ParseError:
                    set checks checks_push checks Result::Ok unit;
                StdErrorKind::Failure:
                    set checks checks_push checks Result::Err "wrong error kind";
                StdErrorKind::OutOfMemory:
                    set checks checks_push checks Result::Err "wrong error kind";
                StdErrorKind::EmptyCollection:
                    set checks checks_push checks Result::Err "wrong error kind";
                StdErrorKind::IndexOutOfBounds:
                    set checks checks_push checks Result::Err "wrong error kind";
                StdErrorKind::KeyNotFound:
                    set checks checks_push checks Result::Err "wrong error kind";
                StdErrorKind::CapacityExceeded:
                    set checks checks_push checks Result::Err "wrong error kind";
                StdErrorKind::InvalidOperation:
                    set checks checks_push checks Result::Err "wrong error kind";
                StdErrorKind::InvalidUtf8:
                    set checks checks_push checks Result::Err "wrong error kind";
                StdErrorKind::IoError:
                    set checks checks_push checks Result::Err "wrong error kind";
                StdErrorKind::Other:
                    set checks checks_push checks Result::Err "wrong error kind";
    let shown checks_print_report checks;
    checks_exit_code shown
```
